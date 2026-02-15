use serde::{Deserialize, Serialize};
use crate::db::queries::devices::Device;

/// Condition types that can be evaluated against a device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Device is online
    IsOnline,
    /// Device is trusted
    IsTrusted,
    /// Device is a gateway
    IsGateway,
    /// IP matches pattern (IPv4 CIDR or specific IP)
    IpMatches { pattern: String },
    /// MAC matches pattern (exact or wildcard)
    MacMatches { pattern: String },
    /// Device vendor contains text
    VendorContains { text: String },
    /// Device hostname contains text
    HostnameContains { text: String },
    /// Device has any port open
    HasOpenPorts,
    /// Device has specific port open
    PortOpen { port: u16 },
    /// Device has unknown OS
    OsUnknown,
    /// OS confidence is below threshold
    LowOsConfidence { threshold: f32 },
    /// Device not seen in X minutes
    NotSeenSince { minutes: i64 },
    /// Device is newly discovered (first_seen close to now)
    IsNewDevice { minutes: i64 },
    /// Latency exceeds threshold
    HighLatency { ms: u32 },
    /// Custom property match (key=value)
    CustomProperty { key: String, value: String },
}

/// Logical operators for combining conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operator", rename_all = "UPPERCASE")]
pub enum ConditionLogic {
    /// All conditions must be true
    And { conditions: Vec<ConditionGroup> },
    /// At least one condition must be true
    Or { conditions: Vec<ConditionGroup> },
    /// Condition must be false
    Not { condition: Box<ConditionGroup> },
}

/// A group is either a single condition or a logical combination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionGroup {
    Simple(Condition),
    Logical(ConditionLogic),
}

/// A custom alert rule with conditions and actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAlertRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
    pub conditions: ConditionGroup,
    pub severity: String,
    pub notify_desktop: bool,
    pub webhook_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Evaluate a condition against a device
pub fn evaluate_condition(condition: &Condition, device: &Device) -> bool {
    match condition {
        Condition::IsOnline => device.is_online,
        Condition::IsTrusted => device.is_trusted,
        Condition::IsGateway => device.is_gateway,
        Condition::IpMatches { pattern } => {
            if let Some(ip) = &device.current_ip {
                matches_ip_pattern(ip, pattern)
            } else {
                false
            }
        }
        Condition::MacMatches { pattern } => {
            if let Some(mac) = &device.mac_address {
                matches_mac_pattern(mac, pattern)
            } else {
                false
            }
        }
        Condition::VendorContains { text } => {
            device
                .vendor
                .as_ref()
                .map(|v| v.to_lowercase().contains(&text.to_lowercase()))
                .unwrap_or(false)
        }
        Condition::HostnameContains { text } => {
            device
                .hostname
                .as_ref()
                .map(|h| h.to_lowercase().contains(&text.to_lowercase()))
                .unwrap_or(false)
        }
        Condition::HasOpenPorts => !device.open_ports.is_empty(),
        Condition::PortOpen { port } => device.open_ports.contains(port),
        Condition::OsUnknown => device.os_guess.is_none(),
        Condition::LowOsConfidence { threshold } => device.os_confidence < *threshold as f64,
        Condition::NotSeenSince { minutes } => {
            // Check if device hasn't been seen for X minutes
            // This would require comparing last_seen with current time
            // For now, return false to indicate we need timestamp support
            false
        }
        Condition::IsNewDevice { minutes: _ } => {
            // Check if device was discovered recently
            // Would need timestamp comparison logic
            false
        }
        Condition::HighLatency { ms } => {
            device
                .latency_ms
                .map(|l| l > *ms)
                .unwrap_or(false)
        }
        Condition::CustomProperty { key, value } => {
            // This would require storing custom properties on Device
            // For now, return false
            false
        }
    }
}

/// Evaluate a condition group against a device
pub fn evaluate_condition_group(group: &ConditionGroup, device: &Device) -> bool {
    match group {
        ConditionGroup::Simple(cond) => evaluate_condition(cond, device),
        ConditionGroup::Logical(logic) => evaluate_logic(logic, device),
    }
}

/// Evaluate logical operators
fn evaluate_logic(logic: &ConditionLogic, device: &Device) -> bool {
    match logic {
        ConditionLogic::And { conditions } => {
            conditions.iter().all(|c| evaluate_condition_group(c, device))
        }
        ConditionLogic::Or { conditions } => {
            conditions.iter().any(|c| evaluate_condition_group(c, device))
        }
        ConditionLogic::Not { condition } => !evaluate_condition_group(condition, device),
    }
}

/// Check if IP matches pattern (IPv4 CIDR or exact IP)
fn matches_ip_pattern(ip: &str, pattern: &str) -> bool {
    if pattern == ip {
        return true;
    }

    // Simple CIDR matching (192.168.1.0/24)
    if pattern.contains('/') {
        return matches_cidr(ip, pattern);
    }

    false
}

/// Simple CIDR matching (approximate)
fn matches_cidr(ip: &str, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let (network, prefix_str) = (parts[0], parts[1]);
    let prefix = prefix_str.parse::<u8>().unwrap_or(0);

    let ip_octets: Vec<&str> = ip.split('.').collect();
    let net_octets: Vec<&str> = network.split('.').collect();

    if ip_octets.len() != 4 || net_octets.len() != 4 {
        return false;
    }

    // Compare relevant octets based on prefix
    let full_octets = (prefix / 8) as usize;
    let remaining_bits = prefix % 8;

    for i in 0..full_octets {
        if ip_octets[i] != net_octets[i] {
            return false;
        }
    }

    if full_octets < 4 && remaining_bits > 0 {
        let ip_octet = ip_octets[full_octets].parse::<u8>().unwrap_or(0);
        let net_octet = net_octets[full_octets].parse::<u8>().unwrap_or(0);

        let mask = !(255u8 >> remaining_bits);
        (ip_octet & mask) == (net_octet & mask)
    } else {
        true
    }
}

/// Check if MAC matches pattern (exact or wildcard with *)
fn matches_mac_pattern(mac: &str, pattern: &str) -> bool {
    let mac_upper = mac.to_uppercase();
    let pattern_upper = pattern.to_uppercase();

    if !pattern_upper.contains('*') {
        return mac_upper == pattern_upper;
    }

    // Simple wildcard matching: * matches any single group
    let mac_parts: Vec<&str> = mac_upper.split(':').collect();
    let pattern_parts: Vec<&str> = pattern_upper.split(':').collect();

    if mac_parts.len() != pattern_parts.len() {
        return false;
    }

    mac_parts
        .iter()
        .zip(pattern_parts.iter())
        .all(|(m, p)| *p == "*" || m == p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_device(
        id: &str,
        ip: &str,
        mac: &str,
        online: bool,
        trusted: bool,
    ) -> Device {
        Device {
            id: id.to_string(),
            mac_address: Some(mac.to_string()),
            vendor: Some("Test Vendor".to_string()),
            hostname: Some("test-host".to_string()),
            custom_name: None,
            device_type: "computer".to_string(),
            os_guess: Some("Linux".to_string()),
            os_confidence: 0.95,
            is_trusted: trusted,
            is_gateway: false,
            notes: None,
            current_ip: Some(ip.to_string()),
            is_online: online,
            latency_ms: Some(25),
            open_ports: vec![80, 443],
            first_seen: "2024-01-01 00:00:00".to_string(),
            last_seen: "2024-01-01 12:00:00".to_string(),
        }
    }

    #[test]
    fn test_evaluate_is_online() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::IsOnline;
        assert!(evaluate_condition(&cond, &device));

        let device_offline = make_test_device("dev2", "192.168.1.11", "AA:BB:CC:DD:EE:GG", false, false);
        assert!(!evaluate_condition(&cond, &device_offline));
    }

    #[test]
    fn test_evaluate_is_trusted() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, true);
        let cond = Condition::IsTrusted;
        assert!(evaluate_condition(&cond, &device));
    }

    #[test]
    fn test_ip_exact_match() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::IpMatches {
            pattern: "192.168.1.10".to_string(),
        };
        assert!(evaluate_condition(&cond, &device));
    }

    #[test]
    fn test_ip_cidr_match() {
        let device = make_test_device("dev1", "192.168.1.50", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::IpMatches {
            pattern: "192.168.1.0/24".to_string(),
        };
        assert!(evaluate_condition(&cond, &device));

        let device_outside = make_test_device("dev2", "192.168.2.50", "AA:BB:CC:DD:EE:GG", true, false);
        assert!(!evaluate_condition(&cond, &device_outside));
    }

    #[test]
    fn test_mac_exact_match() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::MacMatches {
            pattern: "AA:BB:CC:DD:EE:FF".to_string(),
        };
        assert!(evaluate_condition(&cond, &device));
    }

    #[test]
    fn test_mac_wildcard_match() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::MacMatches {
            pattern: "AA:BB:CC:DD:EE:*".to_string(),
        };
        assert!(evaluate_condition(&cond, &device));

        let cond_no_match = Condition::MacMatches {
            pattern: "AA:BB:CC:DD:FF:*".to_string(),
        };
        assert!(!evaluate_condition(&cond_no_match, &device));
    }

    #[test]
    fn test_vendor_contains() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::VendorContains {
            text: "vendor".to_string(),
        };
        assert!(evaluate_condition(&cond, &device));
    }

    #[test]
    fn test_has_open_ports() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::HasOpenPorts;
        assert!(evaluate_condition(&cond, &device));
    }

    #[test]
    fn test_port_open() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::PortOpen { port: 80 };
        assert!(evaluate_condition(&cond, &device));

        let cond_not_open = Condition::PortOpen { port: 8080 };
        assert!(!evaluate_condition(&cond_not_open, &device));
    }

    #[test]
    fn test_high_latency() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let cond = Condition::HighLatency { ms: 50 };
        assert!(!evaluate_condition(&cond, &device));

        let cond_high = Condition::HighLatency { ms: 20 };
        assert!(evaluate_condition(&cond_high, &device));
    }

    #[test]
    fn test_and_logic() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, true);
        let logic = ConditionLogic::And {
            conditions: vec![
                ConditionGroup::Simple(Condition::IsOnline),
                ConditionGroup::Simple(Condition::IsTrusted),
            ],
        };
        assert!(evaluate_logic(&logic, &device));

        let device_untrusted = make_test_device("dev2", "192.168.1.11", "AA:BB:CC:DD:EE:GG", true, false);
        assert!(!evaluate_logic(&logic, &device_untrusted));
    }

    #[test]
    fn test_or_logic() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let logic = ConditionLogic::Or {
            conditions: vec![
                ConditionGroup::Simple(Condition::IsTrusted),
                ConditionGroup::Simple(Condition::HasOpenPorts),
            ],
        };
        assert!(evaluate_logic(&logic, &device));
    }

    #[test]
    fn test_not_logic() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        let logic = ConditionLogic::Not {
            condition: Box::new(ConditionGroup::Simple(Condition::IsTrusted)),
        };
        assert!(evaluate_logic(&logic, &device));
    }

    #[test]
    fn test_complex_nested_logic() {
        let device = make_test_device("dev1", "192.168.1.10", "AA:BB:CC:DD:EE:FF", true, false);
        // (IsOnline AND HasOpenPorts) OR IsTrusted
        let logic = ConditionLogic::Or {
            conditions: vec![
                ConditionGroup::Logical(ConditionLogic::And {
                    conditions: vec![
                        ConditionGroup::Simple(Condition::IsOnline),
                        ConditionGroup::Simple(Condition::HasOpenPorts),
                    ],
                }),
                ConditionGroup::Simple(Condition::IsTrusted),
            ],
        };
        assert!(evaluate_logic(&logic, &device));
    }

    #[test]
    fn test_cidr_boundaries() {
        // Test /24 network
        assert!(matches_cidr("192.168.1.0", "192.168.1.0/24"));
        assert!(matches_cidr("192.168.1.255", "192.168.1.0/24"));
        assert!(!matches_cidr("192.168.2.0", "192.168.1.0/24"));

        // Test /16 network
        assert!(matches_cidr("192.168.1.1", "192.168.0.0/16"));
        assert!(matches_cidr("192.168.255.255", "192.168.0.0/16"));
        assert!(!matches_cidr("192.169.0.0", "192.168.0.0/16"));
    }
}
