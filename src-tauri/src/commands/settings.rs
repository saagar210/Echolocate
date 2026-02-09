use tauri::State;

use crate::db::queries::settings as db_settings;
use crate::network::interface;
use crate::scanner::ping;
use crate::state::AppState;

#[tauri::command]
pub fn get_interfaces() -> Vec<interface::NetworkInterface> {
    interface::get_interfaces()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<db_settings::AppSettings, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_settings::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: db_settings::AppSettings,
) -> Result<(), String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_settings::update_settings(&conn, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_monitor(
    _state: State<'_, AppState>,
    _interval_secs: u64,
) -> Result<(), String> {
    // TODO: Implement continuous monitoring loop
    log::warn!("Continuous monitoring not yet implemented");
    Ok(())
}

#[tauri::command]
pub async fn stop_monitor(_state: State<'_, AppState>) -> Result<(), String> {
    // TODO: Implement monitoring stop
    log::warn!("Continuous monitoring stop not yet implemented");
    Ok(())
}

#[tauri::command]
pub fn get_latency_history(
    state: State<'_, AppState>,
    device_id: String,
    hours: u32,
) -> Result<Vec<db_settings::LatencyPoint>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_settings::get_latency_history(&conn, &device_id, hours).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResult {
    pub ip: String,
    pub latency_ms: Option<f64>,
    pub success: bool,
}

#[tauri::command]
pub async fn ping_device(ip: String) -> Result<PingResult, String> {
    let latency = ping::ping(&ip).await;
    Ok(PingResult {
        ip,
        success: latency.is_some(),
        latency_ms: latency,
    })
}
