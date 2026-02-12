/// Input validation for command parameters to prevent injection attacks
/// and invalid data from being processed.
use regex::Regex;

pub struct Validator;

impl Validator {
    /// Validate IPv4 format (0.0.0.0 to 255.255.255.255)
    /// Returns the validated IP string or an error message
    pub fn validate_ipv4(ip: &str) -> Result<String, String> {
        // Using simple numeric validation instead of lazy_static
        // Pattern: four decimal groups separated by dots
        let parts: Vec<&str> = ip.split('.').collect();

        if parts.len() != 4 {
            return Err(format!("Invalid IPv4 format: {} (expected 4 octets)", ip));
        }

        for part in &parts {
            match part.parse::<u8>() {
                Ok(_) => continue,
                Err(_) => return Err(format!("Invalid IPv4 octet: {} (must be 0-255)", part)),
            }
        }

        Ok(ip.to_string())
    }

    /// Validate IPv6 format
    /// Returns the validated IP string or an error message
    pub fn validate_ipv6(ip: &str) -> Result<String, String> {
        // Basic IPv6 validation - check if it parses
        match ip.parse::<std::net::Ipv6Addr>() {
            Ok(_) => Ok(ip.to_string()),
            Err(_) => Err(format!("Invalid IPv6 format: {}", ip)),
        }
    }

    /// Validate port number (1-65535)
    /// Returns the validated port or an error message
    pub fn validate_port(port: u16) -> Result<u16, String> {
        if port > 0 {
            Ok(port)
        } else {
            Err(format!("Port must be 1-65535, got {}", port))
        }
    }

    /// Validate device name (1-256 chars, no null bytes)
    /// Returns the validated name or an error message
    pub fn validate_device_name(name: &str) -> Result<String, String> {
        if name.is_empty() {
            return Err("Device name cannot be empty".to_string());
        }

        if name.len() > 256 {
            return Err(format!(
                "Device name exceeds 256 characters (got {})",
                name.len()
            ));
        }

        if name.contains('\0') {
            return Err("Device name contains invalid null bytes".to_string());
        }

        // Check for only whitespace
        if name.chars().all(|c| c.is_whitespace()) {
            return Err("Device name cannot be only whitespace".to_string());
        }

        Ok(name.to_string())
    }

    /// Validate hostname (DNS format or IP address)
    /// Returns the validated hostname or an error message
    pub fn validate_hostname(hostname: &str) -> Result<String, String> {
        if hostname.is_empty() {
            return Err("Hostname cannot be empty".to_string());
        }

        if hostname.len() > 253 {
            return Err(format!(
                "Hostname exceeds 253 characters (got {})",
                hostname.len()
            ));
        }

        // Allow IP addresses (IPv4 or IPv6)
        if Self::validate_ipv4(hostname).is_ok() || Self::validate_ipv6(hostname).is_ok() {
            return Ok(hostname.to_string());
        }

        // Check DNS label format: labels separated by dots
        // Each label: 1-63 chars, start/end with alphanumeric, may contain hyphens
        let labels: Vec<&str> = hostname.split('.').collect();

        if labels.is_empty() {
            return Err("Hostname has no labels".to_string());
        }

        for label in labels {
            if label.is_empty() || label.len() > 63 {
                return Err(format!("Invalid DNS label: '{}' (must be 1-63 chars)", label));
            }

            // Check first character
            if !label.chars().next().unwrap().is_alphanumeric() {
                return Err(format!(
                    "DNS label cannot start with non-alphanumeric: '{}'",
                    label
                ));
            }

            // Check last character
            if !label.chars().last().unwrap().is_alphanumeric() {
                return Err(format!(
                    "DNS label cannot end with non-alphanumeric: '{}'",
                    label
                ));
            }

            // Check allowed characters (alphanumeric and hyphen)
            for ch in label.chars() {
                if !ch.is_alphanumeric() && ch != '-' {
                    return Err(format!(
                        "DNS label contains invalid character '{}' in '{}'",
                        ch, label
                    ));
                }
            }
        }

        Ok(hostname.to_string())
    }

    /// Validate MAC address format (XX:XX:XX:XX:XX:XX)
    /// Accepts both colon and hyphen separators
    pub fn validate_mac_address(mac: &str) -> Result<String, String> {
        let mac_upper = mac.to_uppercase();

        // Try colon separator
        if mac_upper.contains(':') {
            let parts: Vec<&str> = mac_upper.split(':').collect();
            if parts.len() != 6 {
                return Err(format!(
                    "Invalid MAC address: {} (expected 6 octets separated by colons)",
                    mac
                ));
            }

            for part in parts {
                if part.len() != 2 {
                    return Err(format!("Invalid MAC octet: {} (expected 2 hex digits)", part));
                }
                if u8::from_str_radix(part, 16).is_err() {
                    return Err(format!("Invalid MAC octet: {} (not valid hex)", part));
                }
            }

            return Ok(mac.to_string());
        }

        // Try hyphen separator
        if mac_upper.contains('-') {
            let parts: Vec<&str> = mac_upper.split('-').collect();
            if parts.len() != 6 {
                return Err(format!(
                    "Invalid MAC address: {} (expected 6 octets separated by hyphens)",
                    mac
                ));
            }

            for part in parts {
                if part.len() != 2 {
                    return Err(format!("Invalid MAC octet: {} (expected 2 hex digits)", part));
                }
                if u8::from_str_radix(part, 16).is_err() {
                    return Err(format!("Invalid MAC octet: {} (not valid hex)", part));
                }
            }

            return Ok(mac.to_string());
        }

        Err(format!(
            "Invalid MAC address: {} (expected XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX)",
            mac
        ))
    }

    /// Validate notes/description field (1-1024 chars)
    pub fn validate_notes(notes: &str) -> Result<String, String> {
        if notes.is_empty() {
            return Ok(String::new()); // Empty notes are OK
        }

        if notes.len() > 1024 {
            return Err(format!(
                "Notes exceed 1024 characters (got {})",
                notes.len()
            ));
        }

        Ok(notes.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ipv4_valid() {
        assert!(Validator::validate_ipv4("192.168.1.1").is_ok());
        assert!(Validator::validate_ipv4("0.0.0.0").is_ok());
        assert!(Validator::validate_ipv4("255.255.255.255").is_ok());
        assert!(Validator::validate_ipv4("10.0.0.1").is_ok());
    }

    #[test]
    fn test_validate_ipv4_invalid() {
        assert!(Validator::validate_ipv4("256.1.1.1").is_err());
        assert!(Validator::validate_ipv4("192.168.1").is_err());
        assert!(Validator::validate_ipv4("192.168.1.1.1").is_err());
        assert!(Validator::validate_ipv4("not.an.ip.addr").is_err());
    }

    #[test]
    fn test_validate_ipv6_valid() {
        assert!(Validator::validate_ipv6("::1").is_ok());
        assert!(Validator::validate_ipv6("2001:db8::1").is_ok());
        assert!(Validator::validate_ipv6("::").is_ok());
    }

    #[test]
    fn test_validate_ipv6_invalid() {
        assert!(Validator::validate_ipv6("gggg::1").is_err());
        assert!(Validator::validate_ipv6("not-ipv6").is_err());
    }

    #[test]
    fn test_validate_port_valid() {
        assert!(Validator::validate_port(1).is_ok());
        assert!(Validator::validate_port(80).is_ok());
        assert!(Validator::validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(Validator::validate_port(0).is_err());
    }

    #[test]
    fn test_validate_device_name_valid() {
        assert!(Validator::validate_device_name("My Device").is_ok());
        assert!(Validator::validate_device_name("MacBook Pro").is_ok());
        assert!(Validator::validate_device_name("a").is_ok());
    }

    #[test]
    fn test_validate_device_name_invalid() {
        assert!(Validator::validate_device_name("").is_err());
        assert!(Validator::validate_device_name("   ").is_err());
        assert!(Validator::validate_device_name(&"x".repeat(257)).is_err());
    }

    #[test]
    fn test_validate_hostname_valid() {
        assert!(Validator::validate_hostname("example.com").is_ok());
        assert!(Validator::validate_hostname("sub.example.com").is_ok());
        assert!(Validator::validate_hostname("192.168.1.1").is_ok());
        assert!(Validator::validate_hostname("::1").is_ok());
        assert!(Validator::validate_hostname("my-device").is_ok());
    }

    #[test]
    fn test_validate_hostname_invalid() {
        assert!(Validator::validate_hostname("").is_err());
        assert!(Validator::validate_hostname("-invalid.com").is_err());
        assert!(Validator::validate_hostname("invalid-.com").is_err());
        assert!(Validator::validate_hostname("invalid..com").is_err());
    }

    #[test]
    fn test_validate_mac_address_valid() {
        assert!(Validator::validate_mac_address("aa:bb:cc:dd:ee:ff").is_ok());
        assert!(Validator::validate_mac_address("AA:BB:CC:DD:EE:FF").is_ok());
        assert!(Validator::validate_mac_address("aa-bb-cc-dd-ee-ff").is_ok());
        assert!(Validator::validate_mac_address("00:00:00:00:00:00").is_ok());
    }

    #[test]
    fn test_validate_mac_address_invalid() {
        assert!(Validator::validate_mac_address("aa:bb:cc:dd:ee").is_err()); // Too few
        assert!(Validator::validate_mac_address("aa:bb:cc:dd:ee:ff:gg").is_err()); // Invalid hex
        assert!(Validator::validate_mac_address("not-a-mac-addr").is_err());
    }

    #[test]
    fn test_validate_notes_valid() {
        assert!(Validator::validate_notes("").is_ok());
        assert!(Validator::validate_notes("Some notes").is_ok());
        assert!(Validator::validate_notes(&"x".repeat(1024)).is_ok());
    }

    #[test]
    fn test_validate_notes_invalid() {
        assert!(Validator::validate_notes(&"x".repeat(1025)).is_err());
    }
}
