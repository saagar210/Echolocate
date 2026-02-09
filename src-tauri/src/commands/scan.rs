use tauri::{AppHandle, State};

use crate::db::queries::scans as db_scans;
use crate::scanner::{orchestrator, ScanConfig, ScanResult};
use crate::state::AppState;

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ScanConfig,
) -> Result<ScanResult, String> {
    orchestrator::run_scan(app, &state, config).await
}

#[tauri::command]
pub async fn stop_scan(_scan_id: String) -> Result<(), String> {
    // TODO: Implement scan cancellation via CancellationToken
    log::warn!("Scan cancellation not yet implemented");
    Ok(())
}

#[tauri::command]
pub fn get_scan_history(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<db_scans::ScanSummary>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_scans::get_scan_history(&conn, limit).map_err(|e| e.to_string())
}
