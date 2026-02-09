use crate::scanner::port::PortResult;

/// OS fingerprinting result.
#[derive(Debug, Clone)]
pub struct OsGuess {
    pub os: String,
    pub confidence: f64,
}

/// Guess the OS based on open port signatures.
/// Uses well-known port patterns to infer the operating system.
pub fn guess_os(ports: &[PortResult], vendor: Option<&str>) -> Option<OsGuess> {
    let open_ports: Vec<u16> = ports.iter().map(|p| p.port).collect();

    // iOS: port 62078 (iphone-sync / lockdownd)
    if open_ports.contains(&62078) {
        return Some(OsGuess {
            os: "iOS".to_string(),
            confidence: 0.85,
        });
    }

    // macOS: AFP (548) or AirDrop-related (5353 + high port)
    if open_ports.contains(&548) {
        return Some(OsGuess {
            os: "macOS".to_string(),
            confidence: 0.80,
        });
    }

    // Windows: SMB (445) + RPC (135)
    if open_ports.contains(&445) && open_ports.contains(&135) {
        return Some(OsGuess {
            os: "Windows".to_string(),
            confidence: 0.85,
        });
    }

    // Windows: just SMB (445) without Linux indicators
    if open_ports.contains(&445) && !open_ports.contains(&22) {
        return Some(OsGuess {
            os: "Windows".to_string(),
            confidence: 0.60,
        });
    }

    // Linux: SSH (22) without Windows indicators
    if open_ports.contains(&22) && !open_ports.contains(&445) && !open_ports.contains(&135) {
        return Some(OsGuess {
            os: "Linux".to_string(),
            confidence: 0.55,
        });
    }

    // Printer: IPP (631) or JetDirect (9100)
    if open_ports.contains(&631) || open_ports.contains(&9100) {
        // Could be any OS, but printers often have their own firmware
        return Some(OsGuess {
            os: "Printer firmware".to_string(),
            confidence: 0.70,
        });
    }

    // Router/AP: HTTP (80) + HTTPS (443) with no SSH and few other ports
    if open_ports.contains(&80) && open_ports.len() <= 3 {
        if let Some(v) = vendor {
            let v_lower = v.to_lowercase();
            if v_lower.contains("ubiquiti") || v_lower.contains("mikrotik")
                || v_lower.contains("cisco") || v_lower.contains("netgear")
                || v_lower.contains("tp-link") || v_lower.contains("asus")
                || v_lower.contains("linksys") || v_lower.contains("arris")
            {
                return Some(OsGuess {
                    os: "Router firmware".to_string(),
                    confidence: 0.75,
                });
            }
        }
    }

    // Vendor-based fallback guesses
    if let Some(v) = vendor {
        let v_lower = v.to_lowercase();
        if v_lower.contains("apple") {
            return Some(OsGuess {
                os: "macOS/iOS".to_string(),
                confidence: 0.40,
            });
        }
        if v_lower.contains("samsung") || v_lower.contains("oneplus")
            || v_lower.contains("xiaomi") || v_lower.contains("huawei")
        {
            return Some(OsGuess {
                os: "Android".to_string(),
                confidence: 0.50,
            });
        }
        if v_lower.contains("microsoft") {
            return Some(OsGuess {
                os: "Windows".to_string(),
                confidence: 0.45,
            });
        }
        if v_lower.contains("raspberry") {
            return Some(OsGuess {
                os: "Linux".to_string(),
                confidence: 0.70,
            });
        }
    }

    None
}

/// Classify device type based on vendor, ports, and OS guess.
pub fn classify_device(
    ports: &[PortResult],
    vendor: Option<&str>,
    os_guess: Option<&str>,
    is_gateway: bool,
) -> &'static str {
    if is_gateway {
        return "router";
    }

    let open_ports: Vec<u16> = ports.iter().map(|p| p.port).collect();

    // Printer detection
    if open_ports.contains(&9100) || open_ports.contains(&631) {
        if let Some(v) = vendor {
            let v_lower = v.to_lowercase();
            if v_lower.contains("hp") || v_lower.contains("epson")
                || v_lower.contains("canon") || v_lower.contains("brother")
                || v_lower.contains("xerox") || v_lower.contains("lexmark")
            {
                return "printer";
            }
        }
        return "printer";
    }

    // Phone detection
    if let Some(os) = os_guess {
        let os_lower = os.to_lowercase();
        if os_lower.contains("ios") || os_lower.contains("android") {
            return "phone";
        }
    }
    if open_ports.contains(&62078) {
        return "phone";
    }

    // Media device detection
    if let Some(v) = vendor {
        let v_lower = v.to_lowercase();
        if v_lower.contains("sonos") || v_lower.contains("roku")
            || v_lower.contains("amazon") || v_lower.contains("google")
            || v_lower.contains("chromecast")
        {
            // Could be media device (smart speaker, streaming stick)
            if open_ports.len() <= 5 {
                return "media";
            }
        }

        // IoT detection
        if v_lower.contains("espressif") || v_lower.contains("tuya")
            || v_lower.contains("shenzhen") || v_lower.contains("wemo")
            || v_lower.contains("nest") || v_lower.contains("ring")
            || v_lower.contains("wyze") || v_lower.contains("lifx")
        {
            return "iot";
        }

        // Router/AP detection by vendor
        if v_lower.contains("ubiquiti") || v_lower.contains("mikrotik")
            || v_lower.contains("cisco") || v_lower.contains("netgear")
            || v_lower.contains("tp-link") || v_lower.contains("linksys")
            || v_lower.contains("arris") || v_lower.contains("asus")
        {
            if open_ports.contains(&80) || open_ports.contains(&443) {
                return "router";
            }
        }
    }

    // Computer detection (SSH, RDP, or many open ports)
    if open_ports.contains(&22) || open_ports.contains(&3389) || open_ports.contains(&548)
        || open_ports.contains(&445) || open_ports.len() >= 5
    {
        return "computer";
    }

    if let Some(os) = os_guess {
        let os_lower = os.to_lowercase();
        if os_lower.contains("windows") || os_lower.contains("macos") || os_lower.contains("linux") {
            return "computer";
        }
    }

    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::port::{PortResult, PortState};

    fn port(p: u16) -> PortResult {
        PortResult {
            port: p,
            state: PortState::Open,
            service_name: None,
            banner: None,
        }
    }

    #[test]
    fn test_ios_detection() {
        let ports = vec![port(62078)];
        let guess = guess_os(&ports, None).unwrap();
        assert_eq!(guess.os, "iOS");
        assert!(guess.confidence >= 0.8);
    }

    #[test]
    fn test_windows_detection() {
        let ports = vec![port(135), port(445), port(139)];
        let guess = guess_os(&ports, None).unwrap();
        assert_eq!(guess.os, "Windows");
    }

    #[test]
    fn test_macos_detection() {
        let ports = vec![port(548), port(22)];
        let guess = guess_os(&ports, None).unwrap();
        assert_eq!(guess.os, "macOS");
    }

    #[test]
    fn test_linux_detection() {
        let ports = vec![port(22), port(80)];
        let guess = guess_os(&ports, None).unwrap();
        assert_eq!(guess.os, "Linux");
    }

    #[test]
    fn test_classify_printer() {
        let ports = vec![port(9100), port(80)];
        assert_eq!(classify_device(&ports, Some("HP Inc"), None, false), "printer");
    }

    #[test]
    fn test_classify_phone() {
        let ports = vec![port(62078)];
        assert_eq!(classify_device(&ports, Some("Apple"), None, false), "phone");
    }

    #[test]
    fn test_classify_router() {
        assert_eq!(classify_device(&[], None, None, true), "router");
    }

    #[test]
    fn test_classify_computer_by_os() {
        let ports = vec![];
        assert_eq!(classify_device(&ports, None, Some("Windows"), false), "computer");
    }

    #[test]
    fn test_vendor_android_guess() {
        let guess = guess_os(&[], Some("Samsung Electronics")).unwrap();
        assert_eq!(guess.os, "Android");
    }
}
