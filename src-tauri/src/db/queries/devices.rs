use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::db::queries::ports;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub mac_address: Option<String>,
    pub vendor: Option<String>,
    pub hostname: Option<String>,
    pub custom_name: Option<String>,
    pub device_type: String,
    pub os_guess: Option<String>,
    pub os_confidence: f64,
    pub is_trusted: bool,
    pub is_gateway: bool,
    pub notes: Option<String>,
    pub current_ip: Option<String>,
    pub is_online: bool,
    pub latency_ms: Option<f64>,
    pub open_ports: Vec<ports::PortInfo>,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdate {
    pub custom_name: Option<Option<String>>,
    pub device_type: Option<String>,
    pub is_trusted: Option<bool>,
    pub notes: Option<Option<String>>,
}

/// Insert a new device and its current IP.
pub fn insert_device(
    conn: &Connection,
    id: &str,
    mac_address: Option<&str>,
    vendor: Option<&str>,
    hostname: Option<&str>,
    device_type: &str,
    is_gateway: bool,
    ip_address: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO devices (id, mac_address, vendor, hostname, device_type, is_gateway)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, mac_address, vendor, hostname, device_type, is_gateway],
    )?;

    if let Some(ip) = ip_address {
        let ip_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO device_ips (id, device_id, ip_address, is_current)
             VALUES (?1, ?2, ?3, 1)",
            params![ip_id, id, ip],
        )?;
    }

    Ok(())
}

/// Get all known devices with their current IP and latest port data.
pub fn get_all_devices(conn: &Connection) -> Result<Vec<Device>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT
            d.id, d.mac_address, d.vendor, d.hostname, d.custom_name,
            d.device_type, d.os_guess, d.os_confidence, d.is_trusted, d.is_gateway,
            d.notes, d.first_seen, d.last_seen,
            di.ip_address,
            lh.latency_ms
         FROM devices d
         LEFT JOIN device_ips di ON di.device_id = d.id AND di.is_current = 1
         LEFT JOIN (
            SELECT device_id, latency_ms
            FROM latency_history
            WHERE id IN (SELECT MAX(id) FROM latency_history GROUP BY device_id)
         ) lh ON lh.device_id = d.id
         ORDER BY d.last_seen DESC"
    )?;

    let devices = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let last_seen: String = row.get(12)?;

        // Consider device online if seen in the last 5 minutes
        let is_online = is_recently_seen(&last_seen);

        Ok(Device {
            id,
            mac_address: row.get(1)?,
            vendor: row.get(2)?,
            hostname: row.get(3)?,
            custom_name: row.get(4)?,
            device_type: row.get(5)?,
            os_guess: row.get(6)?,
            os_confidence: row.get(7)?,
            is_trusted: row.get(8)?,
            is_gateway: row.get(9)?,
            notes: row.get(10)?,
            first_seen: row.get(11)?,
            last_seen,
            current_ip: row.get(13)?,
            latency_ms: row.get(14)?,
            is_online,
            open_ports: Vec::new(), // Populated separately if needed
        })
    })?;

    devices.collect()
}

/// Get a single device by ID, including its open ports.
pub fn get_device_by_id(conn: &Connection, device_id: &str) -> Result<Option<Device>, rusqlite::Error> {
    let mut devices = get_all_devices(conn)?;
    let device = devices.iter_mut().find(|d| d.id == device_id);

    match device {
        Some(d) => {
            d.open_ports = ports::get_latest_ports(conn, &d.id)?;
            Ok(Some(d.clone()))
        }
        None => Ok(None),
    }
}

/// Find a device by MAC address.
pub fn get_device_by_mac(conn: &Connection, mac: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT id FROM devices WHERE mac_address = ?1",
        [mac],
        |row| row.get(0),
    )
    .optional()
}

/// Update a device's user-editable fields.
pub fn update_device(conn: &Connection, device_id: &str, updates: &DeviceUpdate) -> Result<(), rusqlite::Error> {
    if let Some(ref name) = updates.custom_name {
        conn.execute(
            "UPDATE devices SET custom_name = ?1 WHERE id = ?2",
            params![name, device_id],
        )?;
    }
    if let Some(ref dtype) = updates.device_type {
        conn.execute(
            "UPDATE devices SET device_type = ?1 WHERE id = ?2",
            params![dtype, device_id],
        )?;
    }
    if let Some(trusted) = updates.is_trusted {
        conn.execute(
            "UPDATE devices SET is_trusted = ?1 WHERE id = ?2",
            params![trusted, device_id],
        )?;
    }
    if let Some(ref notes) = updates.notes {
        conn.execute(
            "UPDATE devices SET notes = ?1 WHERE id = ?2",
            params![notes, device_id],
        )?;
    }
    Ok(())
}

/// Delete a device and all associated data (cascades).
pub fn delete_device(conn: &Connection, device_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM devices WHERE id = ?1", [device_id])?;
    Ok(())
}

/// Update hostname for a device (only if it doesn't already have one).
pub fn update_hostname(conn: &Connection, device_id: &str, hostname: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE devices SET hostname = ?1 WHERE id = ?2 AND hostname IS NULL",
        params![hostname, device_id],
    )?;
    Ok(())
}

/// Update OS guess and confidence for a device.
pub fn update_os_guess(conn: &Connection, device_id: &str, os_guess: &str, confidence: f64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE devices SET os_guess = ?1, os_confidence = ?2 WHERE id = ?3",
        params![os_guess, confidence, device_id],
    )?;
    Ok(())
}

/// Update device type classification.
pub fn update_device_type(conn: &Connection, device_id: &str, device_type: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE devices SET device_type = ?1 WHERE id = ?2 AND device_type = 'unknown'",
        params![device_type, device_id],
    )?;
    Ok(())
}

/// Update last_seen timestamp for a device.
pub fn touch_device(conn: &Connection, device_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE devices SET last_seen = datetime('now') WHERE id = ?1",
        [device_id],
    )?;
    Ok(())
}

/// Upsert a device IP: mark old IPs as not current, insert or update the new one.
pub fn upsert_device_ip(conn: &Connection, device_id: &str, ip: &str) -> Result<(), rusqlite::Error> {
    // Check if this IP already exists for this device
    let existing: Option<String> = conn.query_row(
        "SELECT id FROM device_ips WHERE device_id = ?1 AND ip_address = ?2",
        params![device_id, ip],
        |row| row.get(0),
    ).optional()?;

    if let Some(ip_id) = existing {
        // Update existing: mark as current, update last_seen
        conn.execute(
            "UPDATE device_ips SET is_current = 0 WHERE device_id = ?1 AND id != ?2",
            params![device_id, ip_id],
        )?;
        conn.execute(
            "UPDATE device_ips SET is_current = 1, last_seen = datetime('now') WHERE id = ?1",
            [&ip_id],
        )?;
    } else {
        // New IP: mark all others as not current, insert new
        conn.execute(
            "UPDATE device_ips SET is_current = 0 WHERE device_id = ?1",
            [device_id],
        )?;
        let ip_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO device_ips (id, device_id, ip_address, is_current)
             VALUES (?1, ?2, ?3, 1)",
            params![ip_id, device_id, ip],
        )?;
    }

    Ok(())
}

/// Record a latency measurement for a device.
pub fn record_latency(conn: &Connection, device_id: &str, latency_ms: f64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO latency_history (device_id, latency_ms) VALUES (?1, ?2)",
        params![device_id, latency_ms],
    )?;
    Ok(())
}

/// Check if a timestamp is within the last 5 minutes.
fn is_recently_seen(timestamp: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .map(|dt| {
            let now = chrono::Utc::now().naive_utc();
            now.signed_duration_since(dt).num_minutes() < 5
        })
        .unwrap_or(false)
}

/// Trait extension for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_insert_and_get_device() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), Some("Apple"),
            Some("macbook.local"), "computer", false, Some("192.168.1.42"),
        ).unwrap();

        let devices = get_all_devices(&conn).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].mac_address.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
        assert_eq!(devices[0].current_ip.as_deref(), Some("192.168.1.42"));
        assert_eq!(devices[0].vendor.as_deref(), Some("Apple"));
    }

    #[test]
    fn test_update_device() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None,
            None, "unknown", false, Some("192.168.1.42"),
        ).unwrap();

        let updates = DeviceUpdate {
            custom_name: Some(Some("My Laptop".to_string())),
            device_type: Some("computer".to_string()),
            is_trusted: Some(true),
            notes: None,
        };
        update_device(&conn, "dev1", &updates).unwrap();

        let device = get_device_by_id(&conn, "dev1").unwrap().unwrap();
        assert_eq!(device.custom_name.as_deref(), Some("My Laptop"));
        assert_eq!(device.device_type, "computer");
        assert!(device.is_trusted);
    }

    #[test]
    fn test_delete_device() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None,
            None, "unknown", false, None,
        ).unwrap();

        delete_device(&conn, "dev1").unwrap();
        let device = get_device_by_id(&conn, "dev1").unwrap();
        assert!(device.is_none());
    }

    #[test]
    fn test_upsert_device_ip() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None,
            None, "unknown", false, Some("192.168.1.10"),
        ).unwrap();

        // Change IP via DHCP
        upsert_device_ip(&conn, "dev1", "192.168.1.42").unwrap();

        let device = get_device_by_id(&conn, "dev1").unwrap().unwrap();
        assert_eq!(device.current_ip.as_deref(), Some("192.168.1.42"));
    }

    #[test]
    fn test_get_device_by_mac() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None,
            None, "unknown", false, None,
        ).unwrap();

        let id = get_device_by_mac(&conn, "AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(id, Some("dev1".to_string()));

        let missing = get_device_by_mac(&conn, "11:22:33:44:55:66").unwrap();
        assert!(missing.is_none());
    }
}
