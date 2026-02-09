use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub service_name: Option<String>,
    pub banner: Option<String>,
}

/// Insert a discovered port for a device/scan.
pub fn insert_port(
    conn: &Connection,
    device_id: &str,
    scan_id: &str,
    port: u16,
    protocol: &str,
    state: &str,
    service_name: Option<&str>,
    banner: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO device_ports (id, device_id, scan_id, port, protocol, state, service_name, banner)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, device_id, scan_id, port, protocol, state, service_name, banner],
    )?;
    Ok(())
}

/// Get the most recent port scan results for a device.
pub fn get_latest_ports(conn: &Connection, device_id: &str) -> Result<Vec<PortInfo>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT dp.port, dp.protocol, dp.state, dp.service_name, dp.banner
         FROM device_ports dp
         WHERE dp.device_id = ?1
         AND dp.scan_id = (
            SELECT dp2.scan_id FROM device_ports dp2
            WHERE dp2.device_id = ?1
            ORDER BY dp2.discovered_at DESC
            LIMIT 1
         )
         ORDER BY dp.port ASC"
    )?;

    let ports = stmt.query_map([device_id], |row| {
        Ok(PortInfo {
            port: row.get(0)?,
            protocol: row.get(1)?,
            state: row.get(2)?,
            service_name: row.get(3)?,
            banner: row.get(4)?,
        })
    })?;

    ports.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::queries::devices;

    #[test]
    fn test_insert_and_get_ports() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        // Create a device and scan first
        devices::insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None,
            None, "unknown", false, Some("192.168.1.1"),
        ).unwrap();

        conn.execute(
            "INSERT INTO scans (id, scan_type, status) VALUES ('scan1', 'full', 'completed')",
            [],
        ).unwrap();

        insert_port(&conn, "dev1", "scan1", 80, "tcp", "open", Some("http"), None).unwrap();
        insert_port(&conn, "dev1", "scan1", 443, "tcp", "open", Some("https"), None).unwrap();
        insert_port(&conn, "dev1", "scan1", 22, "tcp", "open", Some("ssh"), None).unwrap();

        let ports = get_latest_ports(&conn, "dev1").unwrap();
        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].port, 22); // Sorted by port
        assert_eq!(ports[1].port, 80);
        assert_eq!(ports[2].port, 443);
    }
}
