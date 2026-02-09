use crate::db::queries::alerts::{self as db_alerts, AlertRule};
use crate::db::queries::devices::Device;
use rusqlite::Connection;

/// Evaluate scan results against alert rules and generate alerts.
pub fn evaluate_alerts(
    conn: &Connection,
    previous_devices: &[Device],
    current_devices: &[Device],
) -> Result<Vec<GeneratedAlert>, rusqlite::Error> {
    let rules = db_alerts::get_alert_rules(conn)?;
    let mut alerts = Vec::new();

    // New device detection
    if let Some(rule) = find_rule(&rules, "new_device") {
        if rule.is_enabled {
            for device in current_devices {
                let is_new = !previous_devices.iter().any(|d| d.id == device.id);
                if is_new {
                    let name = device_display_name(device);
                    alerts.push(GeneratedAlert {
                        alert_type: "new_device".to_string(),
                        device_id: Some(device.id.clone()),
                        message: format!("New device discovered: {} ({})", name, device.current_ip.as_deref().unwrap_or("unknown IP")),
                        severity: rule.severity.clone(),
                        notify_desktop: rule.notify_desktop,
                    });
                }
            }
        }
    }

    // Untrusted device detection
    if let Some(rule) = find_rule(&rules, "untrusted_device") {
        if rule.is_enabled {
            for device in current_devices {
                let is_new = !previous_devices.iter().any(|d| d.id == device.id);
                if is_new && !device.is_trusted {
                    let name = device_display_name(device);
                    alerts.push(GeneratedAlert {
                        alert_type: "unknown_device".to_string(),
                        device_id: Some(device.id.clone()),
                        message: format!("Untrusted device on network: {} ({})", name, device.current_ip.as_deref().unwrap_or("unknown IP")),
                        severity: rule.severity.clone(),
                        notify_desktop: rule.notify_desktop,
                    });
                }
            }
        }
    }

    // Device departed detection
    if let Some(rule) = find_rule(&rules, "device_departed") {
        if rule.is_enabled {
            for prev_device in previous_devices {
                if prev_device.is_online {
                    let still_present = current_devices.iter().any(|d| d.id == prev_device.id);
                    if !still_present {
                        let name = device_display_name(prev_device);
                        alerts.push(GeneratedAlert {
                            alert_type: "device_departed".to_string(),
                            device_id: Some(prev_device.id.clone()),
                            message: format!("Device departed: {}", name),
                            severity: rule.severity.clone(),
                            notify_desktop: rule.notify_desktop,
                        });
                    }
                }
            }
        }
    }

    // Persist generated alerts
    for alert in &alerts {
        let alert_id = uuid::Uuid::new_v4().to_string();
        db_alerts::insert_alert(
            conn,
            &alert_id,
            &alert.alert_type,
            alert.device_id.as_deref(),
            &alert.message,
            &alert.severity,
        )?;
    }

    Ok(alerts)
}

#[derive(Debug, Clone)]
pub struct GeneratedAlert {
    pub alert_type: String,
    pub device_id: Option<String>,
    pub message: String,
    pub severity: String,
    pub notify_desktop: bool,
}

fn find_rule<'a>(rules: &'a [AlertRule], rule_type: &str) -> Option<&'a AlertRule> {
    rules.iter().find(|r| r.rule_type == rule_type)
}

fn device_display_name(device: &Device) -> String {
    device
        .custom_name
        .as_ref()
        .or(device.hostname.as_ref())
        .or(device.vendor.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            device
                .mac_address
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::queries::devices;

    /// Insert a test device into the database so FK constraints are satisfied.
    fn insert_test_device(conn: &rusqlite::Connection, id: &str, mac: &str) {
        devices::insert_device(conn, id, Some(mac), None, None, "unknown", false, None).unwrap();
    }

    fn make_device(id: &str, mac: &str, ip: &str, online: bool, trusted: bool) -> Device {
        Device {
            id: id.to_string(),
            mac_address: Some(mac.to_string()),
            vendor: None,
            hostname: None,
            custom_name: None,
            device_type: "unknown".to_string(),
            os_guess: None,
            os_confidence: 0.0,
            is_trusted: trusted,
            is_gateway: false,
            notes: None,
            current_ip: Some(ip.to_string()),
            is_online: online,
            latency_ms: None,
            open_ports: Vec::new(),
            first_seen: "2024-01-01 00:00:00".to_string(),
            last_seen: "2024-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn test_new_device_alert() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();
        insert_test_device(&conn, "dev1", "AA:BB:CC:DD:EE:FF");

        let previous = vec![];
        let current = vec![
            make_device("dev1", "AA:BB:CC:DD:EE:FF", "192.168.1.42", true, false),
        ];

        let alerts = evaluate_alerts(&conn, &previous, &current).unwrap();
        assert!(alerts.iter().any(|a| a.alert_type == "new_device"));
    }

    #[test]
    fn test_untrusted_device_alert() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();
        insert_test_device(&conn, "dev1", "AA:BB:CC:DD:EE:FF");

        let previous = vec![];
        let current = vec![
            make_device("dev1", "AA:BB:CC:DD:EE:FF", "192.168.1.42", true, false),
        ];

        let alerts = evaluate_alerts(&conn, &previous, &current).unwrap();
        assert!(alerts.iter().any(|a| a.alert_type == "unknown_device"));
    }

    #[test]
    fn test_trusted_device_no_untrusted_alert() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();
        insert_test_device(&conn, "dev1", "AA:BB:CC:DD:EE:FF");

        let previous = vec![];
        let current = vec![
            make_device("dev1", "AA:BB:CC:DD:EE:FF", "192.168.1.42", true, true),
        ];

        let alerts = evaluate_alerts(&conn, &previous, &current).unwrap();
        assert!(!alerts.iter().any(|a| a.alert_type == "unknown_device"));
    }

    #[test]
    fn test_device_departed_alert() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();
        insert_test_device(&conn, "dev1", "AA:BB:CC:DD:EE:FF");

        let previous = vec![
            make_device("dev1", "AA:BB:CC:DD:EE:FF", "192.168.1.42", true, true),
        ];
        let current = vec![];

        let alerts = evaluate_alerts(&conn, &previous, &current).unwrap();
        assert!(alerts.iter().any(|a| a.alert_type == "device_departed"));
    }

    #[test]
    fn test_no_alert_for_returning_device() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let dev = make_device("dev1", "AA:BB:CC:DD:EE:FF", "192.168.1.42", true, true);
        let previous = vec![dev.clone()];
        let current = vec![dev];

        // No alerts persisted since no new/departed devices
        let alerts = evaluate_alerts(&conn, &previous, &current).unwrap();
        assert!(alerts.is_empty());
    }
}
