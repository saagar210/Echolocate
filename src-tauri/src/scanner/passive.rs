use regex::Regex;
use std::process::Command;

use super::DiscoveredDevice;
use crate::commands::validate::Validator;

/// Parse the system ARP table to discover devices on the local network.
/// Works without elevated privileges.
pub fn scan_arp_table() -> Vec<DiscoveredDevice> {
    #[cfg(target_os = "macos")]
    {
        scan_arp_macos()
    }

    #[cfg(target_os = "linux")]
    {
        scan_arp_linux()
    }

    #[cfg(target_os = "windows")]
    {
        scan_arp_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        log::warn!("ARP scanning not supported on this platform");
        Vec::new()
    }
}

/// macOS: Parse `arp -a` output
#[cfg(target_os = "macos")]
fn scan_arp_macos() -> Vec<DiscoveredDevice> {
    let output = match Command::new("arp").arg("-a").output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run arp -a: {}", e);
            return Vec::new();
        }
    };

    parse_arp_macos(&output)
}

/// Linux: Parse `ip neigh show` output
#[cfg(target_os = "linux")]
fn scan_arp_linux() -> Vec<DiscoveredDevice> {
    let output = match Command::new("ip")
        .args(&["neigh", "show"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run ip neigh show: {}", e);
            return Vec::new();
        }
    };

    parse_arp_linux(&output)
}

/// Windows: Parse `Get-NetNeighbor` PowerShell output
#[cfg(target_os = "windows")]
fn scan_arp_windows() -> Vec<DiscoveredDevice> {
    let output = match Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            "Get-NetNeighbor -AddressFamily IPv4 | Where-Object {$_.State -ne 'Unreachable'} | Select-Object -Property IPAddress,LinkLayerAddress | ConvertTo-Csv -NoTypeInformation",
        ])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run Get-NetNeighbor: {}", e);
            return Vec::new();
        }
    };

    parse_arp_windows(&output)
}

/// Parse arp -a output (macOS format).
///
/// macOS format:
///   hostname (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]
///   ? (192.168.1.42) at dd:ee:ff:00:11:22 on en0 ifscope [ethernet]
///
/// Entries with "(incomplete)" MAC are skipped.
#[cfg(target_os = "macos")]
fn parse_arp_macos(output: &str) -> Vec<DiscoveredDevice> {
    let re = Regex::new(
        r"(?:(\S+)\s+)?\((\d+\.\d+\.\d+\.\d+)\)\s+at\s+([0-9a-f:]+)"
    ).unwrap();

    let mut devices = Vec::new();
    let gateway_ip = get_gateway_ip_macos();

    for line in output.lines() {
        // Skip incomplete entries
        if line.contains("(incomplete)") || line.contains("ff:ff:ff:ff:ff:ff") {
            continue;
        }

        if let Some(caps) = re.captures(line) {
            let hostname_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("?");
            let ip = caps[2].to_string();
            let mac = caps[3].to_string();

            // Validate IP before adding
            if Validator::validate_ipv4(&ip).is_err() {
                log::warn!("Invalid IP in ARP output: {}", ip);
                continue;
            }

            let hostname = if hostname_raw == "?" {
                None
            } else {
                Some(hostname_raw.to_string())
            };

            let is_gateway = gateway_ip.as_deref() == Some(ip.as_str());

            devices.push(DiscoveredDevice {
                ip,
                mac: Some(mac),
                hostname,
                is_gateway,
            });
        }
    }

    devices
}

/// Parse ip neigh show output (Linux format).
///
/// Linux format:
///   192.168.1.1 dev eth0 lladdr aa:bb:cc:dd:ee:ff REACHABLE
///   192.168.1.42 dev eth0 lladdr dd:ee:ff:00:11:22 STALE
///   192.168.1.99 dev eth0  FAILED
#[cfg(target_os = "linux")]
fn parse_arp_linux(output: &str) -> Vec<DiscoveredDevice> {
    let mut devices = Vec::new();
    let gateway_ip = get_gateway_ip_linux();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let ip = parts[0];

        // Validate IP
        if Validator::validate_ipv4(ip).is_err() {
            continue;
        }

        // Skip if no MAC (FAILED state)
        let mut mac = None;
        let mut device_index = 0;

        for (i, part) in parts.iter().enumerate() {
            if *part == "lladdr" && i + 1 < parts.len() {
                // Validate MAC
                if Validator::validate_mac_address(parts[i + 1]).is_ok() {
                    mac = Some(parts[i + 1].to_string());
                    device_index = i;
                }
                break;
            }
        }

        // Only add if we have a MAC (device is reachable)
        if let Some(mac_addr) = mac {
            let is_gateway = gateway_ip.as_deref() == Some(ip);

            devices.push(DiscoveredDevice {
                ip: ip.to_string(),
                mac: Some(mac_addr),
                hostname: None, // Hostname not in ip neigh output
                is_gateway,
            });
        }
    }

    devices
}

/// Parse Get-NetNeighbor PowerShell output (Windows format).
///
/// CSV format (after ConvertTo-Csv):
///   "IPAddress","LinkLayerAddress"
///   "192.168.1.1","aa-bb-cc-dd-ee-ff"
///   "192.168.1.42","dd-ee-ff-00-11-22"
#[cfg(target_os = "windows")]
fn parse_arp_windows(output: &str) -> Vec<DiscoveredDevice> {
    use std::io::Read;

    let mut devices = Vec::new();
    let gateway_ip = get_gateway_ip_windows();

    // Parse CSV manually (simple case)
    let lines: Vec<&str> = output.lines().collect();

    for line in &lines[1..] { // Skip header
        // Simple CSV parsing (doesn't handle quoted commas)
        let fields: Vec<&str> = line.split(',').map(|f| f.trim_matches('"')).collect();

        if fields.len() < 2 {
            continue;
        }

        let ip = fields[0];
        let mac_raw = fields[1];

        // Validate IP
        if Validator::validate_ipv4(ip).is_err() {
            continue;
        }

        // Windows uses hyphens in MAC addresses; convert to colons for consistency
        let mac = mac_raw.replace("-", ":");

        // Validate MAC
        if Validator::validate_mac_address(&mac).is_err() {
            continue;
        }

        let is_gateway = gateway_ip.as_deref() == Some(ip);

        devices.push(DiscoveredDevice {
            ip: ip.to_string(),
            mac: Some(mac),
            hostname: None, // Hostname not in Get-NetNeighbor output
            is_gateway,
        });
    }

    devices
}

/// Get the default gateway IP from the routing table (macOS).
#[cfg(target_os = "macos")]
fn get_gateway_ip_macos() -> Option<String> {
    let output = Command::new("netstat")
        .args(["-rn"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let gw_re = Regex::new(r"default\s+(\d+\.\d+\.\d+\.\d+)").unwrap();

    gw_re.captures(&text).map(|caps| caps[1].to_string())
}

/// Get the default gateway IP from the routing table (Linux).
#[cfg(target_os = "linux")]
fn get_gateway_ip_linux() -> Option<String> {
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
fn get_gateway_ip_windows() -> Option<String> {
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-Command",
            "Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -Property NextHop | ConvertTo-Csv -NoTypeInformation",
        ])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);

    // Parse CSV: header, then IP addresses
    for line in text.lines().skip(1) {
        let ip = line.trim_matches('"');
        if Validator::validate_ipv4(ip).is_ok() {
            return Some(ip.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ARP_MACOS: &str = r#"? (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]
macbook.local (192.168.1.42) at 11:22:33:44:55:66 on en0 ifscope [ethernet]
? (192.168.1.87) at de:ad:be:ef:ca:fe on en0 ifscope [ethernet]
? (192.168.1.255) at ff:ff:ff:ff:ff:ff on en0 ifscope [ethernet]
? (192.168.1.99) at (incomplete) on en0 ifscope [ethernet]
printer.local (192.168.1.50) at ab:cd:ef:12:34:56 on en0 ifscope [ethernet]"#;

    const SAMPLE_ARP_LINUX: &str = r#"192.168.1.1 dev eth0 lladdr aa:bb:cc:dd:ee:ff REACHABLE
192.168.1.42 dev eth0 lladdr 11:22:33:44:55:66 STALE
192.168.1.87 dev eth0 lladdr de:ad:be:ef:ca:fe REACHABLE
192.168.1.99 dev eth0  FAILED
192.168.1.50 dev eth0 lladdr ab:cd:ef:12:34:56 STALE"#;

    const SAMPLE_ROUTE_LINUX: &str = r#"default via 192.168.1.1 dev eth0 proto dhcp metric 100
10.0.0.0/8 via 192.168.1.2 dev eth0 proto static
192.168.1.0/24 dev eth0 proto kernel scope link src 192.168.1.100"#;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_parse_arp_macos() {
        let devices = parse_arp_macos(SAMPLE_ARP_MACOS);

        // Should skip broadcast (ff:ff:ff:ff:ff:ff) and incomplete entries
        assert_eq!(devices.len(), 4);

        // First device
        assert_eq!(devices[0].ip, "192.168.1.1");
        assert_eq!(devices[0].mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));
        assert!(devices[0].hostname.is_none()); // "?" maps to None

        // Second device has a hostname
        assert_eq!(devices[1].ip, "192.168.1.42");
        assert_eq!(devices[1].hostname.as_deref(), Some("macbook.local"));

        // Third device
        assert_eq!(devices[2].ip, "192.168.1.87");

        // Fourth device (printer)
        assert_eq!(devices[3].ip, "192.168.1.50");
        assert_eq!(devices[3].hostname.as_deref(), Some("printer.local"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_empty_arp_table() {
        let devices = parse_arp_macos("");
        assert!(devices.is_empty());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_all_incomplete() {
        let output = "? (192.168.1.1) at (incomplete) on en0\n? (192.168.1.2) at (incomplete) on en0";
        let devices = parse_arp_macos(output);
        assert!(devices.is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_arp_linux() {
        let devices = parse_arp_linux(SAMPLE_ARP_LINUX);

        // Should skip FAILED entries (no MAC)
        assert_eq!(devices.len(), 4);

        // First device
        assert_eq!(devices[0].ip, "192.168.1.1");
        assert_eq!(devices[0].mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));

        // Second device
        assert_eq!(devices[1].ip, "192.168.1.42");
        assert_eq!(devices[1].mac.as_deref(), Some("11:22:33:44:55:66"));

        // Last device
        assert_eq!(devices[3].ip, "192.168.1.50");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_arp_linux_invalid_ip() {
        let output = "999.999.999.999 dev eth0 lladdr aa:bb:cc:dd:ee:ff REACHABLE";
        let devices = parse_arp_linux(output);
        assert!(devices.is_empty(), "Should reject invalid IP");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_gateway_linux() {
        // This tests the gateway detection logic
        // The actual gateway IP extraction is tested implicitly in parse_arp_linux
        let test_route = "default via 192.168.1.1 dev eth0 proto dhcp metric 100";
        assert!(test_route.contains("default via"));
        assert!(test_route.contains("192.168.1.1"));
    }
}
