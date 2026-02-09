use regex::Regex;
use std::process::Command;

use super::DiscoveredDevice;

/// Parse the system ARP table to discover devices on the local network.
/// Works without elevated privileges.
pub fn scan_arp_table() -> Vec<DiscoveredDevice> {
    let output = match Command::new("arp").arg("-a").output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            log::error!("Failed to run arp -a: {}", e);
            return Vec::new();
        }
    };

    parse_arp_output(&output)
}

/// Parse arp -a output into discovered devices.
///
/// macOS format:
///   hostname (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]
///   ? (192.168.1.42) at dd:ee:ff:00:11:22 on en0 ifscope [ethernet]
///
/// Entries with "(incomplete)" MAC are skipped.
fn parse_arp_output(output: &str) -> Vec<DiscoveredDevice> {
    let re = Regex::new(
        r"(?:(\S+)\s+)?\((\d+\.\d+\.\d+\.\d+)\)\s+at\s+([0-9a-f:]+)"
    ).unwrap();

    let mut devices = Vec::new();
    let gateway_ip = get_gateway_ip();

    for line in output.lines() {
        // Skip incomplete entries
        if line.contains("(incomplete)") || line.contains("ff:ff:ff:ff:ff:ff") {
            continue;
        }

        if let Some(caps) = re.captures(line) {
            let hostname_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("?");
            let ip = caps[2].to_string();
            let mac = caps[3].to_string();

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

/// Get the default gateway IP from the routing table.
fn get_gateway_ip() -> Option<String> {
    let output = Command::new("netstat")
        .args(["-rn"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let gw_re = Regex::new(r"default\s+(\d+\.\d+\.\d+\.\d+)").unwrap();

    gw_re.captures(&text).map(|caps| caps[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ARP_OUTPUT: &str = r#"? (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]
macbook.local (192.168.1.42) at 11:22:33:44:55:66 on en0 ifscope [ethernet]
? (192.168.1.87) at de:ad:be:ef:ca:fe on en0 ifscope [ethernet]
? (192.168.1.255) at ff:ff:ff:ff:ff:ff on en0 ifscope [ethernet]
? (192.168.1.99) at (incomplete) on en0 ifscope [ethernet]
printer.local (192.168.1.50) at ab:cd:ef:12:34:56 on en0 ifscope [ethernet]"#;

    #[test]
    fn test_parse_arp_output() {
        let devices = parse_arp_output(SAMPLE_ARP_OUTPUT);

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
    fn test_empty_arp_table() {
        let devices = parse_arp_output("");
        assert!(devices.is_empty());
    }

    #[test]
    fn test_all_incomplete() {
        let output = "? (192.168.1.1) at (incomplete) on en0\n? (192.168.1.2) at (incomplete) on en0";
        let devices = parse_arp_output(output);
        assert!(devices.is_empty());
    }
}
