use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_interface_id: Option<String>,
    pub scan_interval_secs: u64,
    pub port_range: String,
    pub theme: String,
    pub graph_repulsion: f64,
    pub graph_link_distance: f64,
    pub graph_gravity: f64,
}

/// Load all settings from the key-value store.
pub fn get_settings(conn: &Connection) -> Result<AppSettings, rusqlite::Error> {
    let get = |key: &str| -> Result<Option<String>, rusqlite::Error> {
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
    };

    Ok(AppSettings {
        default_interface_id: get("default_interface_id")?,
        scan_interval_secs: get("scan_interval_secs")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(60),
        port_range: get("port_range")?.unwrap_or_else(|| "top100".to_string()),
        theme: get("theme")?.unwrap_or_else(|| "dark".to_string()),
        graph_repulsion: get("graph_repulsion")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(300.0),
        graph_link_distance: get("graph_link_distance")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(100.0),
        graph_gravity: get("graph_gravity")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.1),
    })
}

/// Save all settings to the key-value store.
pub fn update_settings(conn: &Connection, settings: &AppSettings) -> Result<(), rusqlite::Error> {
    let set = |key: &str, value: &str| -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    };

    if let Some(ref id) = settings.default_interface_id {
        set("default_interface_id", id)?;
    }
    set("scan_interval_secs", &settings.scan_interval_secs.to_string())?;
    set("port_range", &settings.port_range)?;
    set("theme", &settings.theme)?;
    set("graph_repulsion", &settings.graph_repulsion.to_string())?;
    set("graph_link_distance", &settings.graph_link_distance.to_string())?;
    set("graph_gravity", &settings.graph_gravity.to_string())?;

    Ok(())
}

/// Get latency history for a device within a time window.
pub fn get_latency_history(
    conn: &Connection,
    device_id: &str,
    hours: u32,
) -> Result<Vec<LatencyPoint>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT latency_ms, measured_at FROM latency_history
         WHERE device_id = ?1
         AND measured_at >= datetime('now', ?2)
         ORDER BY measured_at ASC"
    )?;

    let hours_param = format!("-{} hours", hours);
    let points = stmt.query_map(params![device_id, hours_param], |row| {
        Ok(LatencyPoint {
            latency_ms: row.get(0)?,
            measured_at: row.get(1)?,
        })
    })?;

    points.collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyPoint {
    pub latency_ms: f64,
    pub measured_at: String,
}

/// Helper for optional query results.
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
    fn test_settings_roundtrip() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let mut settings = get_settings(&conn).unwrap();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.scan_interval_secs, 60);

        settings.theme = "light".to_string();
        settings.scan_interval_secs = 120;
        update_settings(&conn, &settings).unwrap();

        let loaded = get_settings(&conn).unwrap();
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.scan_interval_secs, 120);
    }
}
