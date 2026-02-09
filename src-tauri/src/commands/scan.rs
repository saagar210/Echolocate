use tauri::{AppHandle, State};
use tokio_util::sync::CancellationToken;

use crate::db::queries::scans as db_scans;
use crate::scanner::{orchestrator, ScanConfig, ScanResult};
use crate::state::AppState;

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ScanConfig,
) -> Result<ScanResult, String> {
    let cancel = CancellationToken::new();

    // Store the cancellation token so stop_scan can trigger it
    {
        let mut guard = state.scan_cancel.lock().map_err(|e| e.to_string())?;
        *guard = Some(cancel.clone());
    }

    let result = orchestrator::run_scan(app, &state, config, cancel).await;

    // Clear the token after scan completes
    {
        let mut guard = state.scan_cancel.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }

    result
}

#[tauri::command]
pub async fn stop_scan(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.scan_cancel.lock().map_err(|e| e.to_string())?;
    if let Some(ref token) = *guard {
        token.cancel();
        log::info!("Scan cancellation requested");
    } else {
        log::warn!("No active scan to cancel");
    }
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
