use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub id: String,
    pub scan_type: String,
    pub status: String,
    pub devices_found: u32,
    pub new_devices: u32,
    pub duration_ms: Option<u64>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Create a new scan record.
pub fn create_scan(
    conn: &Connection,
    id: &str,
    interface_id: Option<&str>,
    scan_type: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO scans (id, interface_id, scan_type, status)
         VALUES (?1, ?2, ?3, 'running')",
        params![id, interface_id, scan_type],
    )?;
    Ok(())
}

/// Mark a scan as completed with results.
pub fn complete_scan(
    conn: &Connection,
    scan_id: &str,
    devices_found: u32,
    new_devices: u32,
    duration_ms: u64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE scans SET
            status = 'completed',
            devices_found = ?2,
            new_devices = ?3,
            duration_ms = ?4,
            completed_at = datetime('now')
         WHERE id = ?1",
        params![scan_id, devices_found, new_devices, duration_ms],
    )?;
    Ok(())
}

/// Mark a scan as failed.
pub fn fail_scan(conn: &Connection, scan_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE scans SET status = 'failed', completed_at = datetime('now') WHERE id = ?1",
        [scan_id],
    )?;
    Ok(())
}

/// Get scan history, newest first.
pub fn get_scan_history(conn: &Connection, limit: u32) -> Result<Vec<ScanSummary>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, scan_type, status, devices_found, new_devices, duration_ms, started_at, completed_at
         FROM scans
         ORDER BY started_at DESC
         LIMIT ?1"
    )?;

    let scans = stmt.query_map([limit], |row| {
        Ok(ScanSummary {
            id: row.get(0)?,
            scan_type: row.get(1)?,
            status: row.get(2)?,
            devices_found: row.get(3)?,
            new_devices: row.get(4)?,
            duration_ms: row.get(5)?,
            started_at: row.get(6)?,
            completed_at: row.get(7)?,
        })
    })?;

    scans.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_scan_lifecycle() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        create_scan(&conn, "scan1", None, "quick").unwrap();

        let history = get_scan_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, "running");

        complete_scan(&conn, "scan1", 5, 2, 3500).unwrap();

        let history = get_scan_history(&conn, 10).unwrap();
        assert_eq!(history[0].status, "completed");
        assert_eq!(history[0].devices_found, 5);
        assert_eq!(history[0].new_devices, 2);
        assert_eq!(history[0].duration_ms, Some(3500));
    }
}
