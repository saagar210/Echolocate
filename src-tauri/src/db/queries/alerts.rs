use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: String,
    pub alert_type: String,
    pub device_id: Option<String>,
    pub message: String,
    pub severity: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertRule {
    pub id: String,
    pub rule_type: String,
    pub is_enabled: bool,
    pub severity: String,
    pub notify_desktop: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuleUpdate {
    pub is_enabled: Option<bool>,
    pub severity: Option<String>,
    pub notify_desktop: Option<bool>,
}

/// Insert a new alert.
pub fn insert_alert(
    conn: &Connection,
    id: &str,
    alert_type: &str,
    device_id: Option<&str>,
    message: &str,
    severity: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO alerts (id, alert_type, device_id, message, severity)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, alert_type, device_id, message, severity],
    )?;
    Ok(())
}

/// Get alerts, optionally filtering to unread only.
pub fn get_alerts(conn: &Connection, unread_only: bool) -> Result<Vec<Alert>, rusqlite::Error> {
    let sql = if unread_only {
        "SELECT id, alert_type, device_id, message, severity, is_read, created_at
         FROM alerts WHERE is_read = 0 ORDER BY created_at DESC"
    } else {
        "SELECT id, alert_type, device_id, message, severity, is_read, created_at
         FROM alerts ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;
    let alerts = stmt.query_map([], |row| {
        Ok(Alert {
            id: row.get(0)?,
            alert_type: row.get(1)?,
            device_id: row.get(2)?,
            message: row.get(3)?,
            severity: row.get(4)?,
            is_read: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    alerts.collect()
}

/// Mark a single alert as read.
pub fn mark_alert_read(conn: &Connection, alert_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE alerts SET is_read = 1 WHERE id = ?1",
        [alert_id],
    )?;
    Ok(())
}

/// Mark all alerts as read.
pub fn mark_all_alerts_read(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("UPDATE alerts SET is_read = 1 WHERE is_read = 0", [])?;
    Ok(())
}

/// Get all alert rules.
pub fn get_alert_rules(conn: &Connection) -> Result<Vec<AlertRule>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, rule_type, is_enabled, severity, notify_desktop FROM alert_rules"
    )?;

    let rules = stmt.query_map([], |row| {
        Ok(AlertRule {
            id: row.get(0)?,
            rule_type: row.get(1)?,
            is_enabled: row.get(2)?,
            severity: row.get(3)?,
            notify_desktop: row.get(4)?,
        })
    })?;

    rules.collect()
}

/// Update an alert rule.
pub fn update_alert_rule(
    conn: &Connection,
    rule_id: &str,
    updates: &AlertRuleUpdate,
) -> Result<(), rusqlite::Error> {
    if let Some(enabled) = updates.is_enabled {
        conn.execute(
            "UPDATE alert_rules SET is_enabled = ?1 WHERE id = ?2",
            params![enabled, rule_id],
        )?;
    }
    if let Some(ref severity) = updates.severity {
        conn.execute(
            "UPDATE alert_rules SET severity = ?1 WHERE id = ?2",
            params![severity, rule_id],
        )?;
    }
    if let Some(notify) = updates.notify_desktop {
        conn.execute(
            "UPDATE alert_rules SET notify_desktop = ?1 WHERE id = ?2",
            params![notify, rule_id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_insert_and_get_alerts() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        // Insert device to satisfy FK constraint
        crate::db::queries::devices::insert_device(
            &conn, "dev1", Some("AA:BB:CC:DD:EE:FF"), None, None, "unknown", false, None,
        ).unwrap();

        insert_alert(&conn, "alert1", "new_device", Some("dev1"), "New device found", "info").unwrap();
        insert_alert(&conn, "alert2", "port_changed", Some("dev1"), "Port 80 opened", "warning").unwrap();

        let all = get_alerts(&conn, false).unwrap();
        assert_eq!(all.len(), 2);

        let unread = get_alerts(&conn, true).unwrap();
        assert_eq!(unread.len(), 2);

        mark_alert_read(&conn, "alert1").unwrap();
        let unread = get_alerts(&conn, true).unwrap();
        assert_eq!(unread.len(), 1);

        mark_all_alerts_read(&conn).unwrap();
        let unread = get_alerts(&conn, true).unwrap();
        assert_eq!(unread.len(), 0);
    }

    #[test]
    fn test_alert_rules() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let rules = get_alert_rules(&conn).unwrap();
        assert_eq!(rules.len(), 4); // Seeded by migration

        let update = AlertRuleUpdate {
            is_enabled: Some(false),
            severity: None,
            notify_desktop: None,
        };
        update_alert_rule(&conn, "rule_new_device", &update).unwrap();

        let rules = get_alert_rules(&conn).unwrap();
        let rule = rules.iter().find(|r| r.id == "rule_new_device").unwrap();
        assert!(!rule.is_enabled);
    }
}
