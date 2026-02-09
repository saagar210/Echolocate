use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub id: String,
    pub name: String,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub mac_address: Option<String>,
    pub gateway_ip: Option<String>,
    pub is_active: bool,
}

/// Discover all network interfaces on this machine.
pub fn get_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = parse_ifconfig();
    let gateway = get_default_gateway();

    // Set gateway for the interface that has a route to it
    if let Some(ref gw) = gateway {
        for iface in &mut interfaces {
            if let Some(ref ip) = iface.ip_address {
                if same_subnet(ip, gw) {
                    iface.gateway_ip = Some(gw.clone());
                }
            }
        }
    }

    interfaces
}

/// Parse ifconfig output to enumerate interfaces.
fn parse_ifconfig() -> Vec<NetworkInterface> {
    let output = match Command::new("ifconfig").output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run ifconfig: {}", e);
            return Vec::new();
        }
    };

    let iface_re = Regex::new(r"^(\w+):").unwrap();
    let inet_re = Regex::new(r"inet (\d+\.\d+\.\d+\.\d+).*?netmask (0x[0-9a-f]+|[\d.]+)").unwrap();
    let ether_re = Regex::new(r"ether ([0-9a-f:]+)").unwrap();
    let status_re = Regex::new(r"status: (\w+)").unwrap();

    let mut interfaces = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_ip: Option<String> = None;
    let mut current_mask: Option<String> = None;
    let mut current_mac: Option<String> = None;
    let mut current_active = false;

    let flush = |name: &Option<String>,
                 ip: &Option<String>,
                 mask: &Option<String>,
                 mac: &Option<String>,
                 active: bool,
                 interfaces: &mut Vec<NetworkInterface>| {
        if let Some(ref n) = name {
            // Skip loopback and virtual interfaces
            if n == "lo0" || n.starts_with("utun") || n.starts_with("bridge")
                || n.starts_with("awdl") || n.starts_with("llw")
                || n.starts_with("anpi") || n.starts_with("ap")
            {
                return;
            }

            let has_ip = ip.is_some();
            interfaces.push(NetworkInterface {
                id: n.clone(),
                name: n.clone(),
                ip_address: ip.clone(),
                subnet_mask: mask.clone(),
                mac_address: mac.clone(),
                gateway_ip: None,
                is_active: active && has_ip,
            });
        }
    };

    for line in output.lines() {
        if let Some(caps) = iface_re.captures(line) {
            // Flush previous interface
            flush(
                &current_name, &current_ip, &current_mask,
                &current_mac, current_active, &mut interfaces,
            );

            current_name = Some(caps[1].to_string());
            current_ip = None;
            current_mask = None;
            current_mac = None;
            current_active = line.contains("UP") && line.contains("RUNNING");
        }

        if let Some(caps) = inet_re.captures(line) {
            let ip = caps[1].to_string();
            let mask_raw = caps[2].to_string();
            // Skip IPv4 link-local
            if !ip.starts_with("127.") {
                current_ip = Some(ip);
                current_mask = Some(convert_netmask(&mask_raw));
            }
        }

        if let Some(caps) = ether_re.captures(line) {
            current_mac = Some(caps[1].to_string());
        }

        if let Some(caps) = status_re.captures(line) {
            current_active = &caps[1] == "active";
        }
    }

    // Flush last interface
    flush(
        &current_name, &current_ip, &current_mask,
        &current_mac, current_active, &mut interfaces,
    );

    interfaces
}

/// Get the default gateway IP from the routing table.
fn get_default_gateway() -> Option<String> {
    let output = Command::new("netstat")
        .args(["-rn"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let gw_re = Regex::new(r"default\s+(\d+\.\d+\.\d+\.\d+)").unwrap();

    gw_re.captures(&text).map(|caps| caps[1].to_string())
}

/// Convert hex netmask (0xffffff00) or dotted notation to dotted notation.
fn convert_netmask(mask: &str) -> String {
    if mask.starts_with("0x") {
        let hex = mask.trim_start_matches("0x");
        if let Ok(val) = u32::from_str_radix(hex, 16) {
            return format!(
                "{}.{}.{}.{}",
                (val >> 24) & 0xff,
                (val >> 16) & 0xff,
                (val >> 8) & 0xff,
                val & 0xff,
            );
        }
    }
    mask.to_string()
}

/// Check if two IPs are on the same /24 subnet (simple heuristic).
fn same_subnet(ip1: &str, ip2: &str) -> bool {
    let parts1: Vec<&str> = ip1.split('.').collect();
    let parts2: Vec<&str> = ip2.split('.').collect();
    if parts1.len() != 4 || parts2.len() != 4 {
        return false;
    }
    parts1[0] == parts2[0] && parts1[1] == parts2[1] && parts1[2] == parts2[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_netmask_hex() {
        assert_eq!(convert_netmask("0xffffff00"), "255.255.255.0");
        assert_eq!(convert_netmask("0xffff0000"), "255.255.0.0");
    }

    #[test]
    fn test_convert_netmask_dotted() {
        assert_eq!(convert_netmask("255.255.255.0"), "255.255.255.0");
    }

    #[test]
    fn test_same_subnet() {
        assert!(same_subnet("192.168.1.42", "192.168.1.1"));
        assert!(!same_subnet("192.168.1.42", "192.168.2.1"));
        assert!(!same_subnet("10.0.0.1", "192.168.1.1"));
    }
}
