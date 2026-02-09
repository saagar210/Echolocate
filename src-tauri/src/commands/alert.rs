use tauri::State;

use crate::db::queries::alerts as db_alerts;
use crate::state::AppState;

#[tauri::command]
pub fn get_alerts(
    state: State<'_, AppState>,
    unread_only: bool,
) -> Result<Vec<db_alerts::Alert>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::get_alerts(&conn, unread_only).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_alert_read(state: State<'_, AppState>, alert_id: String) -> Result<(), String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::mark_alert_read(&conn, &alert_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_all_alerts_read(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::mark_all_alerts_read(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_alert_rules(state: State<'_, AppState>) -> Result<Vec<db_alerts::AlertRule>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::get_alert_rules(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_alert_rule(
    state: State<'_, AppState>,
    rule_id: String,
    updates: db_alerts::AlertRuleUpdate,
) -> Result<db_alerts::AlertRule, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::update_alert_rule(&conn, &rule_id, &updates).map_err(|e| e.to_string())?;
    let rules = db_alerts::get_alert_rules(&conn).map_err(|e| e.to_string())?;
    rules
        .into_iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| format!("Rule not found: {}", rule_id))
}
