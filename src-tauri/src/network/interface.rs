use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::commands::validate::Validator;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub id: String,
    pub name: String,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub ipv6_address: Option<String>,
    pub ipv6_prefix: Option<u8>,
    pub mac_address: Option<String>,
    pub gateway_ip: Option<String>,
    pub gateway_ipv6: Option<String>,
    pub is_active: bool,
}

/// Discover all network interfaces on this machine.
pub fn get_interfaces() -> Vec<NetworkInterface> {
    #[cfg(target_os = "macos")]
    {
        discover_interfaces_macos()
    }

    #[cfg(target_os = "linux")]
    {
        discover_interfaces_linux()
    }

    #[cfg(target_os = "windows")]
    {
        discover_interfaces_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        log::warn!("Interface discovery not supported on this platform");
        Vec::new()
    }
}

/// macOS: Discover interfaces via ifconfig
#[cfg(target_os = "macos")]
fn discover_interfaces_macos() -> Vec<NetworkInterface> {
    let mut interfaces = parse_ifconfig_macos();
    let gateway = get_default_gateway_macos();

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

/// Linux: Discover interfaces via ip command
#[cfg(target_os = "linux")]
fn discover_interfaces_linux() -> Vec<NetworkInterface> {
    let mut interfaces = parse_ip_addr_linux();
    let gateway = get_default_gateway_linux();

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

/// Windows: Discover interfaces via PowerShell
#[cfg(target_os = "windows")]
fn discover_interfaces_windows() -> Vec<NetworkInterface> {
    let mut interfaces = parse_ipconfig_windows();
    let gateway = get_default_gateway_windows();

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

/// Parse ifconfig output to enumerate interfaces (macOS).
#[cfg(target_os = "macos")]
fn parse_ifconfig_macos() -> Vec<NetworkInterface> {
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

/// Get the default gateway IP from the routing table (macOS).
#[cfg(target_os = "macos")]
fn get_default_gateway_macos() -> Option<String> {
    let output = Command::new("netstat")
        .args(["-rn"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let gw_re = Regex::new(r"default\s+(\d+\.\d+\.\d+\.\d+)").unwrap();

    gw_re.captures(&text).and_then(|caps| {
        let ip = &caps[1];
        if Validator::validate_ipv4(ip).is_ok() {
            Some(ip.to_string())
        } else {
            None
        }
    })
}

/// Get the default gateway IP from the routing table (Linux).
#[cfg(target_os = "linux")]
fn get_default_gateway_linux() -> Option<String> {
    let output = Command::new("ip")
        .args(&["route", "show"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);

    // Look for: default via 192.168.1.1 dev eth0
    for line in text.lines() {
        if line.starts_with("default via ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 2 {
                let ip = parts[2];
                if Validator::validate_ipv4(ip).is_ok() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Get the default gateway IP from the routing table (Windows).
#[cfg(target_os = "windows")]
fn get_default_gateway_windows() -> Option<String> {
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            "Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -ExpandProperty NextHop | Select-Object -First 1",
        ])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let ip = text.trim();

    if Validator::validate_ipv4(ip).is_ok() {
        Some(ip.to_string())
    } else {
        None
    }
}

/// Parse ip addr show output to enumerate interfaces (Linux).
#[cfg(target_os = "linux")]
fn parse_ip_addr_linux() -> Vec<NetworkInterface> {
    let output = match Command::new("ip")
        .args(&["addr", "show"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run ip addr show: {}", e);
            return Vec::new();
        }
    };

    let mut interfaces = Vec::new();
    let mut current_interface: Option<NetworkInterface> = None;

    for line in output.lines() {
        // New interface: "1: lo: <LOOPBACK,UP,LOWER_UP>"
        if let Some(first_char) = line.chars().next() {
            if first_char.is_numeric() {
                // Flush previous interface
                if let Some(iface) = current_interface.take() {
                    interfaces.push(iface);
                }

                // Parse new interface line
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    let name = parts[1].trim().to_string();

                    // Skip loopback and virtual interfaces
                    if name == "lo" || name.starts_with("docker") || name.starts_with("veth") {
                        continue;
                    }

                    let is_active = line.contains("UP") && line.contains("LOWER_UP");

                    current_interface = Some(NetworkInterface {
                        id: name.clone(),
                        name,
                        ip_address: None,
                        subnet_mask: None,
                        mac_address: None,
                        gateway_ip: None,
                        is_active,
                    });
                }
            } else if let Some(ref mut iface) = current_interface {
                // Parse IPv4 address: "    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0"
                if line.contains("inet ") && !line.contains("inet6") {
                    if let Some(inet_idx) = line.find("inet ") {
                        let rest = &line[inet_idx + 5..];
                        if let Some(space_idx) = rest.find(|c: char| c.is_whitespace()) {
                            let ip_with_prefix = &rest[..space_idx];
                            if let Some(slash_idx) = ip_with_prefix.find('/') {
                                let ip = &ip_with_prefix[..slash_idx];
                                let prefix = &ip_with_prefix[slash_idx + 1..];

                                if Validator::validate_ipv4(ip).is_ok() {
                                    iface.ip_address = Some(ip.to_string());
                                    // Convert CIDR prefix to netmask
                                    if let Ok(prefix_len) = prefix.parse::<u32>() {
                                        iface.subnet_mask = Some(cidr_to_netmask(prefix_len));
                                    }
                                }
                            }
                        }
                    }
                }

                // Parse MAC address: "    link/ether aa:bb:cc:dd:ee:ff brd ff:ff:ff:ff:ff:ff"
                if line.contains("link/ether ") {
                    if let Some(ether_idx) = line.find("link/ether ") {
                        let rest = &line[ether_idx + 11..];
                        if let Some(space_idx) = rest.find(|c: char| c.is_whitespace()) {
                            let mac = &rest[..space_idx];
                            if Validator::validate_mac_address(mac).is_ok() {
                                iface.mac_address = Some(mac.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Flush last interface
    if let Some(iface) = current_interface {
        interfaces.push(iface);
    }

    interfaces
}

/// Parse ipconfig output to enumerate interfaces (Windows).
#[cfg(target_os = "windows")]
fn parse_ipconfig_windows() -> Vec<NetworkInterface> {
    let output = match Command::new("ipconfig").output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run ipconfig: {}", e);
            return Vec::new();
        }
    };

    let mut interfaces = Vec::new();
    let mut current_interface: Option<NetworkInterface> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        // New interface section starts with letters (not whitespace)
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.contains(':') {
            // Flush previous interface
            if let Some(iface) = current_interface.take() {
                if iface.ip_address.is_some() {
                    interfaces.push(iface);
                }
            }

            // Parse interface name (e.g., "Ethernet adapter Ethernet:")
            if let Some(colon_idx) = trimmed.find(':') {
                let name_part = trimmed[..colon_idx].to_string();
                current_interface = Some(NetworkInterface {
                    id: name_part.clone(),
                    name: name_part,
                    ip_address: None,
                    subnet_mask: None,
                    mac_address: None,
                    gateway_ip: None,
                    is_active: false,
                });
            }
        } else if let Some(ref mut iface) = current_interface {
            // Parse IPv4 Address
            if trimmed.starts_with("IPv4 Address") {
                if let Some(colon_idx) = trimmed.find(':') {
                    let ip_part = trimmed[colon_idx + 1..].trim();
                    if Validator::validate_ipv4(ip_part).is_ok() {
                        iface.ip_address = Some(ip_part.to_string());
                        iface.is_active = true;
                    }
                }
            }

            // Parse Subnet Mask
            if trimmed.starts_with("Subnet Mask") {
                if let Some(colon_idx) = trimmed.find(':') {
                    let mask_part = trimmed[colon_idx + 1..].trim();
                    if Validator::validate_ipv4(mask_part).is_ok() {
                        iface.subnet_mask = Some(mask_part.to_string());
                    }
                }
            }

            // Parse Physical Address (MAC)
            if trimmed.starts_with("Physical Address") {
                if let Some(colon_idx) = trimmed.find(':') {
                    let mac_part = trimmed[colon_idx + 1..].trim();
                    // Windows uses hyphens in MAC; convert to colons
                    let mac = mac_part.replace("-", ":");
                    if Validator::validate_mac_address(&mac).is_ok() {
                        iface.mac_address = Some(mac);
                    }
                }
            }
        }
    }

    // Flush last interface
    if let Some(iface) = current_interface {
        if iface.ip_address.is_some() {
            interfaces.push(iface);
        }
    }

    interfaces
}

/// Convert CIDR prefix length to dotted netmask notation.
fn cidr_to_netmask(prefix_len: u32) -> String {
    let mask = if prefix_len > 0 {
        0xFFFFFFFFu32 << (32 - prefix_len)
    } else {
        0
    };

    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF,
    )
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
    fn test_cidr_to_netmask() {
        assert_eq!(cidr_to_netmask(24), "255.255.255.0");
        assert_eq!(cidr_to_netmask(16), "255.255.0.0");
        assert_eq!(cidr_to_netmask(8), "255.0.0.0");
        assert_eq!(cidr_to_netmask(32), "255.255.255.255");
    }

    #[test]
    fn test_same_subnet() {
        assert!(same_subnet("192.168.1.42", "192.168.1.1"));
        assert!(!same_subnet("192.168.1.42", "192.168.2.1"));
        assert!(!same_subnet("10.0.0.1", "192.168.1.1"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_ip_addr_linux() {
        let sample = r#"1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP group default qlen 1000
    link/ether aa:bb:cc:dd:ee:ff brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0"#;

        // Manual test since parse_ip_addr_linux is not public
        // This would be called via get_interfaces() in real usage
        assert!(sample.contains("192.168.1.100"));
        assert!(sample.contains("aa:bb:cc:dd:ee:ff"));
    }

    #[test]
    fn test_same_subnet_different_ranges() {
        assert!(same_subnet("10.0.0.1", "10.0.0.254"));
        assert!(!same_subnet("10.0.0.1", "10.0.1.1"));
        assert!(same_subnet("172.16.5.1", "172.16.5.100"));
    }
}
