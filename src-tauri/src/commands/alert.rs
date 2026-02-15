use tauri::State;
use uuid::Uuid;

use crate::db::queries::alerts as db_alerts;
use crate::state::AppState;
use crate::alerts::rules::ConditionGroup;

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

#[tauri::command]
pub fn create_custom_rule(
    state: State<'_, AppState>,
    req: db_alerts::CreateCustomRuleRequest,
) -> Result<db_alerts::CustomAlertRuleRecord, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    let rule_id = Uuid::new_v4().to_string();
    db_alerts::create_custom_rule(&conn, &rule_id, &req)
        .map_err(|e| format!("Failed to create custom rule: {}", e))
}

#[tauri::command]
pub fn get_custom_rules(
    state: State<'_, AppState>,
) -> Result<Vec<db_alerts::CustomAlertRuleRecord>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::get_custom_rules(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_rule(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<Option<db_alerts::CustomAlertRuleRecord>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::get_custom_rule(&conn, &rule_id)
        .map_err(|e| format!("Failed to get custom rule: {}", e))
}

#[tauri::command]
pub fn update_custom_rule(
    state: State<'_, AppState>,
    rule_id: String,
    updates: db_alerts::UpdateCustomRuleRequest,
) -> Result<db_alerts::CustomAlertRuleRecord, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::update_custom_rule(&conn, &rule_id, &updates)
        .map_err(|e| format!("Failed to update custom rule: {}", e))?;
    db_alerts::get_custom_rule(&conn, &rule_id)
        .map_err(|e| format!("Failed to retrieve updated rule: {}", e))?
        .ok_or_else(|| format!("Rule not found after update: {}", rule_id))
}

#[tauri::command]
pub fn delete_custom_rule(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_alerts::delete_custom_rule(&conn, &rule_id).map_err(|e| e.to_string())
}
