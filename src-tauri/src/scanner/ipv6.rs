/// IPv6 network discovery module - platform-specific implementations for IPv6 neighbor discovery
use crate::db::queries::devices::Device;
use crate::commands::validate::Validator;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct IPv6Device {
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub hostname: Option<String>,
    pub is_link_local: bool,
}

/// Discover IPv6 devices on the network
pub fn discover_ipv6_devices() -> Result<Vec<IPv6Device>, String> {
    #[cfg(target_os = "macos")]
    return discover_ipv6_devices_macos();

    #[cfg(target_os = "linux")]
    return discover_ipv6_devices_linux();

    #[cfg(target_os = "windows")]
    return discover_ipv6_devices_windows();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    Err("IPv6 discovery not supported on this platform".to_string())
}

/// macOS IPv6 neighbor discovery using `ndp` command
#[cfg(target_os = "macos")]
fn discover_ipv6_devices_macos() -> Result<Vec<IPv6Device>, String> {
    // Use 'ndp -i' to list IPv6 neighbors
    let output = Command::new("ndp")
        .arg("-i")
        .arg("-n")
        .output()
        .map_err(|e| format!("Failed to run ndp command: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in output_str.lines() {
        // Format: IPv6 address / MAC address (state)
        // Example: 2001:db8::1 (08:00:27:00:00:00) reachable
        let line = line.trim();
        if line.is_empty() || line.starts_with("Hostname") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let ip = parts[0];
        if !ip.contains(':') {
            // Not IPv6
            continue;
        }

        // Validate IPv6 format
        if Validator::validate_ipv6(ip).is_err() {
            continue;
        }

        // Extract MAC address from parentheses
        let mac = if line.contains('(') && line.contains(')') {
            let start = line.find('(').unwrap() + 1;
            let end = line.find(')').unwrap();
            let mac_str = &line[start..end];
            Some(mac_str.to_uppercase())
        } else {
            None
        };

        let is_link_local = ip.starts_with("fe80:");

        devices.push(IPv6Device {
            ip_address: ip.to_string(),
            mac_address: mac,
            hostname: None,
            is_link_local,
        });
    }

    Ok(devices)
}

/// Linux IPv6 neighbor discovery using `ip neigh show`
#[cfg(target_os = "linux")]
fn discover_ipv6_devices_linux() -> Result<Vec<IPv6Device>, String> {
    // Use 'ip neigh show' which works for both IPv4 and IPv6
    let output = Command::new("ip")
        .arg("neigh")
        .arg("show")
        .output()
        .map_err(|e| format!("Failed to run ip neigh show command: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in output_str.lines() {
        // Format: IPv6 dev interface lladdr MAC STATE
        // Example: 2001:db8::1 dev eth0 lladdr aa:bb:cc:dd:ee:ff REACHABLE
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let ip = parts[0];

        // Check if IPv6
        if !ip.contains(':') {
            continue;
        }

        // Validate IPv6 format
        if Validator::validate_ipv6(ip).is_err() {
            continue;
        }

        // Skip FAILED entries (no MAC address)
        if parts.contains(&"FAILED") {
            continue;
        }

        // Extract MAC address (lladdr position)
        let mut mac = None;
        for i in 0..parts.len() - 1 {
            if parts[i] == "lladdr" && i + 1 < parts.len() {
                mac = Some(parts[i + 1].to_uppercase());
                break;
            }
        }

        let is_link_local = ip.starts_with("fe80:");

        devices.push(IPv6Device {
            ip_address: ip.to_string(),
            mac_address: mac,
            hostname: None,
            is_link_local,
        });
    }

    Ok(devices)
}

/// Windows IPv6 neighbor discovery using PowerShell
#[cfg(target_os = "windows")]
fn discover_ipv6_devices_windows() -> Result<Vec<IPv6Device>, String> {
    // Use PowerShell Get-NetNeighbor for IPv6
    let script = "Get-NetNeighbor -AddressFamily IPv6 | ConvertTo-Csv -NoTypeInformation";
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run PowerShell command: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    let mut header_read = false;

    for line in output_str.lines() {
        if line.is_empty() {
            continue;
        }

        // Skip header line
        if !header_read {
            header_read = true;
            continue;
        }

        // CSV format: "IPAddress","LinkLayerAddress"
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }

        let ip = parts[0].trim_matches('"');

        // Validate IPv6 format
        if Validator::validate_ipv6(ip).is_err() {
            continue;
        }

        let mac_raw = parts[1].trim_matches('"');
        // Convert from hyphen to colon format: AA-BB-CC-DD-EE-FF -> AA:BB:CC:DD:EE:FF
        let mac = if !mac_raw.is_empty() && mac_raw != "-" {
            Some(mac_raw.replace("-", ":"))
        } else {
            None
        };

        let is_link_local = ip.starts_with("fe80:");

        devices.push(IPv6Device {
            ip_address: ip.to_string(),
            mac_address: mac,
            hostname: None,
            is_link_local,
        });
    }

    Ok(devices)
}

/// Resolve IPv6 hostname via reverse DNS lookup
pub fn resolve_ipv6_hostname(ip: &str) -> Option<String> {
    match ip.parse::<std::net::Ipv6Addr>() {
        Ok(addr) => {
            // Perform reverse DNS lookup
            match std::net::IpAddr::V6(addr) {
                std::net::IpAddr::V6(ipv6_addr) => {
                    // This would require a DNS library for proper implementation
                    // For now, return None as reverse DNS is complex
                    let _ = ipv6_addr;
                    None
                }
                _ => None,
            }
        }
        Err(_) => None,
    }
}

/// Detect if an IPv6 address is link-local (fe80::/10)
pub fn is_ipv6_link_local(ip: &str) -> bool {
    ip.starts_with("fe80:")
}

/// Detect if an IPv6 address is link-local multicast (ff02::/8)
pub fn is_ipv6_multicast(ip: &str) -> bool {
    ip.starts_with("ff")
}

/// Detect if an IPv6 address is loopback (::1)
pub fn is_ipv6_loopback(ip: &str) -> bool {
    ip == "::1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ipv6_link_local() {
        assert!(is_ipv6_link_local("fe80::1"));
        assert!(is_ipv6_link_local("fe80::2"));
        assert!(!is_ipv6_link_local("2001:db8::1"));
        assert!(!is_ipv6_link_local("::1"));
    }

    #[test]
    fn test_is_ipv6_multicast() {
        assert!(is_ipv6_multicast("ff02::1"));
        assert!(is_ipv6_multicast("ff00::1"));
        assert!(!is_ipv6_multicast("2001:db8::1"));
        assert!(!is_ipv6_multicast("fe80::1"));
    }

    #[test]
    fn test_is_ipv6_loopback() {
        assert!(is_ipv6_loopback("::1"));
        assert!(!is_ipv6_loopback("::2"));
        assert!(!is_ipv6_loopback("fe80::1"));
        assert!(!is_ipv6_loopback("2001:db8::1"));
    }

    #[test]
    fn test_ipv6_device_creation() {
        let device = IPv6Device {
            ip_address: "2001:db8::1".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: None,
            is_link_local: false,
        };

        assert_eq!(device.ip_address, "2001:db8::1");
        assert_eq!(device.mac_address, Some("AA:BB:CC:DD:EE:FF".to_string()));
        assert!(!device.is_link_local);
    }

    #[test]
    fn test_ipv6_link_local_device() {
        let device = IPv6Device {
            ip_address: "fe80::1".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: None,
            is_link_local: true,
        };

        assert!(device.is_link_local);
    }
}
