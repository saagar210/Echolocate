use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use crate::alerts::rules::{ConditionGroup, CustomAlertRule};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAlertRuleRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
    pub conditions: String, // JSON string
    pub severity: String,
    pub notify_desktop: bool,
    pub webhook_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub conditions: ConditionGroup,
    pub severity: String,
    pub notify_desktop: bool,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conditions: Option<ConditionGroup>,
    pub severity: Option<String>,
    pub notify_desktop: Option<bool>,
    pub webhook_url: Option<String>,
    pub is_enabled: Option<bool>,
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

/// Create a new custom alert rule.
pub fn create_custom_rule(
    conn: &Connection,
    id: &str,
    req: &CreateCustomRuleRequest,
) -> Result<CustomAlertRuleRecord, Box<dyn std::error::Error>> {
    let conditions_json = serde_json::to_string(&req.conditions)?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO custom_alert_rules
         (id, name, description, is_enabled, conditions, severity, notify_desktop, webhook_url, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, &req.name, &req.description, &conditions_json, &req.severity, req.notify_desktop, &req.webhook_url, &now, &now],
    )?;

    get_custom_rule(conn, id)?.ok_or_else(|| "Rule not found after creation".into())
}

/// Get a single custom alert rule by ID.
pub fn get_custom_rule(conn: &Connection, rule_id: &str) -> Result<Option<CustomAlertRuleRecord>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_enabled, conditions, severity, notify_desktop, webhook_url, created_at, updated_at
         FROM custom_alert_rules WHERE id = ?1"
    )?;

    let rule = stmt.query_row([rule_id], |row| {
        Ok(CustomAlertRuleRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_enabled: row.get(3)?,
            conditions: row.get(4)?,
            severity: row.get(5)?,
            notify_desktop: row.get(6)?,
            webhook_url: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }).optional()?;

    Ok(rule)
}

/// Get all custom alert rules.
pub fn get_custom_rules(conn: &Connection) -> Result<Vec<CustomAlertRuleRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_enabled, conditions, severity, notify_desktop, webhook_url, created_at, updated_at
         FROM custom_alert_rules ORDER BY created_at DESC"
    )?;

    let rules = stmt.query_map([], |row| {
        Ok(CustomAlertRuleRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_enabled: row.get(3)?,
            conditions: row.get(4)?,
            severity: row.get(5)?,
            notify_desktop: row.get(6)?,
            webhook_url: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    rules.collect()
}

/// Update a custom alert rule.
pub fn update_custom_rule(
    conn: &Connection,
    rule_id: &str,
    updates: &UpdateCustomRuleRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(ref name) = updates.name {
        conn.execute(
            "UPDATE custom_alert_rules SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, &now, rule_id],
        )?;
    }
    if let Some(ref description) = updates.description {
        conn.execute(
            "UPDATE custom_alert_rules SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![description, &now, rule_id],
        )?;
    }
    if let Some(ref conditions) = updates.conditions {
        let conditions_json = serde_json::to_string(conditions)?;
        conn.execute(
            "UPDATE custom_alert_rules SET conditions = ?1, updated_at = ?2 WHERE id = ?3",
            params![&conditions_json, &now, rule_id],
        )?;
    }
    if let Some(ref severity) = updates.severity {
        conn.execute(
            "UPDATE custom_alert_rules SET severity = ?1, updated_at = ?2 WHERE id = ?3",
            params![severity, &now, rule_id],
        )?;
    }
    if let Some(notify) = updates.notify_desktop {
        conn.execute(
            "UPDATE custom_alert_rules SET notify_desktop = ?1, updated_at = ?2 WHERE id = ?3",
            params![notify, &now, rule_id],
        )?;
    }
    if let Some(ref webhook) = updates.webhook_url {
        conn.execute(
            "UPDATE custom_alert_rules SET webhook_url = ?1, updated_at = ?2 WHERE id = ?3",
            params![webhook, &now, rule_id],
        )?;
    }
    if let Some(enabled) = updates.is_enabled {
        conn.execute(
            "UPDATE custom_alert_rules SET is_enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled, &now, rule_id],
        )?;
    }

    Ok(())
}

/// Delete a custom alert rule.
pub fn delete_custom_rule(conn: &Connection, rule_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM custom_alert_rules WHERE id = ?1",
        [rule_id],
    )?;
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

    #[test]
    fn test_create_custom_rule() {
        use crate::alerts::rules::Condition;
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let req = CreateCustomRuleRequest {
            name: "Test Rule".to_string(),
            description: Some("A test rule".to_string()),
            conditions: ConditionGroup::Simple(Condition::HasOpenPorts),
            severity: "warning".to_string(),
            notify_desktop: true,
            webhook_url: Some("https://example.com/webhook".to_string()),
        };

        let rule = create_custom_rule(&conn, "custom_rule_1", &req).unwrap();
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.severity, "warning");
        assert!(rule.notify_desktop);
        assert!(rule.is_enabled);
    }

    #[test]
    fn test_get_custom_rule() {
        use crate::alerts::rules::Condition;
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let req = CreateCustomRuleRequest {
            name: "Test Rule".to_string(),
            description: None,
            conditions: ConditionGroup::Simple(Condition::IsOnline),
            severity: "critical".to_string(),
            notify_desktop: false,
            webhook_url: None,
        };

        create_custom_rule(&conn, "custom_rule_1", &req).unwrap();
        let rule = get_custom_rule(&conn, "custom_rule_1").unwrap();
        assert!(rule.is_some());

        let rule = rule.unwrap();
        assert_eq!(rule.id, "custom_rule_1");
        assert_eq!(rule.name, "Test Rule");
    }

    #[test]
    fn test_get_all_custom_rules() {
        use crate::alerts::rules::Condition;
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let req1 = CreateCustomRuleRequest {
            name: "Rule 1".to_string(),
            description: None,
            conditions: ConditionGroup::Simple(Condition::IsOnline),
            severity: "info".to_string(),
            notify_desktop: false,
            webhook_url: None,
        };

        let req2 = CreateCustomRuleRequest {
            name: "Rule 2".to_string(),
            description: None,
            conditions: ConditionGroup::Simple(Condition::IsTrusted),
            severity: "warning".to_string(),
            notify_desktop: true,
            webhook_url: None,
        };

        create_custom_rule(&conn, "rule_1", &req1).unwrap();
        create_custom_rule(&conn, "rule_2", &req2).unwrap();

        let rules = get_custom_rules(&conn).unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_update_custom_rule() {
        use crate::alerts::rules::Condition;
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let req = CreateCustomRuleRequest {
            name: "Original".to_string(),
            description: None,
            conditions: ConditionGroup::Simple(Condition::IsOnline),
            severity: "info".to_string(),
            notify_desktop: false,
            webhook_url: None,
        };

        create_custom_rule(&conn, "rule_1", &req).unwrap();

        let updates = UpdateCustomRuleRequest {
            name: Some("Updated".to_string()),
            description: Some("New description".to_string()),
            conditions: Some(ConditionGroup::Simple(Condition::IsTrusted)),
            severity: Some("critical".to_string()),
            notify_desktop: Some(true),
            webhook_url: None,
            is_enabled: Some(false),
        };

        update_custom_rule(&conn, "rule_1", &updates).unwrap();

        let rule = get_custom_rule(&conn, "rule_1").unwrap().unwrap();
        assert_eq!(rule.name, "Updated");
        assert_eq!(rule.severity, "critical");
        assert!(rule.notify_desktop);
        assert!(!rule.is_enabled);
    }

    #[test]
    fn test_delete_custom_rule() {
        use crate::alerts::rules::Condition;
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        let req = CreateCustomRuleRequest {
            name: "Test".to_string(),
            description: None,
            conditions: ConditionGroup::Simple(Condition::IsOnline),
            severity: "info".to_string(),
            notify_desktop: false,
            webhook_url: None,
        };

        create_custom_rule(&conn, "rule_1", &req).unwrap();
        let rules = get_custom_rules(&conn).unwrap();
        assert_eq!(rules.len(), 1);

        delete_custom_rule(&conn, "rule_1").unwrap();
        let rules = get_custom_rules(&conn).unwrap();
        assert_eq!(rules.len(), 0);
    }
}
