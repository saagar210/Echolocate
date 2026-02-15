/// IPv6 scanning integration tests
#[cfg(test)]
mod tests {
    use crate::scanner::ipv6;

    #[test]
    fn test_ipv6_link_local_filtering() {
        // Link-local addresses should be filtered out in production scans
        assert!(ipv6::is_ipv6_link_local("fe80::1"));
        assert!(ipv6::is_ipv6_link_local("fe80::f816:3eff:fe12:3456"));
        assert!(!ipv6::is_ipv6_link_local("2001:db8::1"));
        assert!(!ipv6::is_ipv6_link_local("::1"));
    }

    #[test]
    fn test_ipv6_address_classification() {
        // Test various IPv6 address types
        assert!(ipv6::is_ipv6_loopback("::1"));
        assert!(ipv6::is_ipv6_multicast("ff02::1"));
        assert!(ipv6::is_ipv6_link_local("fe80::1"));

        // Test non-matches
        assert!(!ipv6::is_ipv6_loopback("2001:db8::1"));
        assert!(!ipv6::is_ipv6_multicast("2001:db8::1"));
        assert!(!ipv6::is_ipv6_link_local("2001:db8::1"));
    }

    #[test]
    fn test_ipv6_device_creation() {
        let device = ipv6::IPv6Device {
            ip_address: "2001:db8::1".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: None,
            is_link_local: false,
        };

        assert_eq!(device.ip_address, "2001:db8::1");
        assert!(!device.is_link_local);
        assert_eq!(device.mac_address.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn test_ipv6_with_mac_resolution() {
        let device_with_mac = ipv6::IPv6Device {
            ip_address: "2001:db8::1".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: None,
            is_link_local: false,
        };

        let device_without_mac = ipv6::IPv6Device {
            ip_address: "2001:db8::2".to_string(),
            mac_address: None,
            hostname: None,
            is_link_local: false,
        };

        assert!(device_with_mac.mac_address.is_some());
        assert!(device_without_mac.mac_address.is_none());
    }

    #[test]
    fn test_ipv6_global_unicast_detection() {
        // Global unicast addresses start with 2 or 3 (2000::/3)
        assert!(!ipv6::is_ipv6_link_local("2001:db8::1"));
        assert!(!ipv6::is_ipv6_link_local("2001:4860:4860::8888"));
    }

    #[test]
    fn test_ipv6_ula_detection() {
        // ULA addresses (fc::/7) are private
        let ula = "fd12:3456:789a::1";
        assert!(!ipv6::is_ipv6_link_local(ula));
        assert!(!ipv6::is_ipv6_multicast(ula));
        assert!(!ipv6::is_ipv6_loopback(ula));
    }

    #[test]
    fn test_ipv6_address_scope_types() {
        // Test multiple address scope types
        let addrs = vec![
            ("::1", "loopback"),                    // loopback
            ("fe80::1", "link_local"),              // link-local
            ("ff02::1", "multicast"),               // multicast
            ("2001:db8::1", "global_unicast"),      // global unicast
            ("fd00::1", "ula"),                     // ULA private
        ];

        for (addr, kind) in addrs {
            match kind {
                "loopback" => assert!(ipv6::is_ipv6_loopback(addr)),
                "link_local" => assert!(ipv6::is_ipv6_link_local(addr)),
                "multicast" => assert!(ipv6::is_ipv6_multicast(addr)),
                _ => {
                    // Global unicast and ULA
                    assert!(!ipv6::is_ipv6_loopback(addr));
                    assert!(!ipv6::is_ipv6_link_local(addr));
                }
            }
        }
    }

    #[test]
    fn test_ipv6_device_validation() {
        use crate::commands::validate::Validator;

        let valid_ipv6 = vec![
            "::1",
            "fe80::1",
            "2001:db8::1",
            "2001:4860:4860::8888",
            "fd00::1",
        ];

        for ip in valid_ipv6 {
            assert!(
                Validator::validate_ipv6(ip).is_ok(),
                "Failed to validate: {}",
                ip
            );
        }
    }

    #[test]
    fn test_ipv6_invalid_addresses() {
        use crate::commands::validate::Validator;

        let invalid_ipv6 = vec![
            "gggg::1",           // Invalid hex
            "not-ipv6",          // Not an IP
            "192.168.1.1",       // IPv4
            "2001:db8:::1",      // Too many colons
            "",                  // Empty
        ];

        for ip in invalid_ipv6 {
            assert!(
                Validator::validate_ipv6(ip).is_err(),
                "Should reject invalid IPv6: {}",
                ip
            );
        }
    }
}
