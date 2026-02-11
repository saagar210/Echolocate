use tauri::{AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

use crate::db::queries::settings as db_settings;
use crate::network::interface;
use crate::scanner::{orchestrator, ping, PortRange, ScanConfig, ScanType};
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

/// Monitor status event emitted to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorStatus {
    is_running: bool,
    next_scan_in: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanError {
    message: String,
}

#[tauri::command]
pub async fn start_monitor(
    app: AppHandle,
    state: State<'_, AppState>,
    interval_secs: u64,
) -> Result<(), String> {
    // Stop existing monitor if running
    stop_monitor_inner(&state)?;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    // We need an Arc-wrapped reference to AppState for the spawned task.
    // Since Tauri manages the state, we clone the DB pool and OUI db for the monitor.
    let db_pool = state.db.clone();
    let oui_db = state.oui_db.clone();
    let app_clone = app.clone();

    let handle = tokio::spawn(async move {
        log::info!(
            "Continuous monitoring started (interval: {}s)",
            interval_secs
        );
        let _ = app_clone.emit(
            "monitor:status",
            MonitorStatus {
                is_running: true,
                next_scan_in: Some(interval_secs),
            },
        );

        loop {
            // Wait for the interval or cancellation
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(interval_secs)) => {},
                _ = cancel_clone.cancelled() => {
                    log::info!("Monitoring loop cancelled");
                    break;
                }
            }

            if cancel_clone.is_cancelled() {
                break;
            }

            // Build a temporary AppState for the scan
            let monitor_state = crate::state::AppState::new(db_pool.clone(), oui_db.clone());
            let scan_cancel = CancellationToken::new();

            let config = monitor_scan_config(&db_pool);

            match orchestrator::run_scan(app_clone.clone(), &monitor_state, config, scan_cancel)
                .await
            {
                Ok(result) => {
                    log::info!(
                        "Monitor scan completed: {} devices, {} new",
                        result.devices_found,
                        result.new_devices
                    );
                }
                Err(e) => {
                    log::error!("Monitor scan failed: {}", e);
                    let _ = app_clone.emit("scan:error", ScanError { message: e });
                }
            }

            // Emit next scan countdown
            let _ = app_clone.emit(
                "monitor:status",
                MonitorStatus {
                    is_running: true,
                    next_scan_in: Some(interval_secs),
                },
            );
        }

        let _ = app_clone.emit(
            "monitor:status",
            MonitorStatus {
                is_running: false,
                next_scan_in: None,
            },
        );
        log::info!("Continuous monitoring stopped");
    });

    // Store handle and cancel token
    {
        let mut h = state.monitor_handle.lock().map_err(|e| e.to_string())?;
        *h = Some(handle);
    }
    {
        let mut c = state.monitor_cancel.lock().map_err(|e| e.to_string())?;
        *c = Some(cancel);
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_monitor(state: State<'_, AppState>) -> Result<(), String> {
    stop_monitor_inner(&state)
}

fn stop_monitor_inner(state: &AppState) -> Result<(), String> {
    // Cancel the monitoring loop
    {
        let mut c = state.monitor_cancel.lock().map_err(|e| e.to_string())?;
        if let Some(ref token) = *c {
            token.cancel();
        }
        *c = None;
    }

    // Abort the task handle
    {
        let mut h = state.monitor_handle.lock().map_err(|e| e.to_string())?;
        if let Some(handle) = h.take() {
            handle.abort();
        }
    }

    log::info!("Monitor stop requested");
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

fn monitor_scan_config(db_pool: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> ScanConfig {
    let settings = db_pool
        .get()
        .ok()
        .and_then(|conn| db_settings::get_settings(&conn).ok());

    match settings {
        Some(s) => ScanConfig {
            interface_id: s.default_interface_id.unwrap_or_else(|| "auto".to_string()),
            scan_type: ScanType::Quick,
            port_range: parse_port_range(&s.port_range),
        },
        None => {
            log::warn!("Monitor could not load app settings from DB; using default scan config");
            ScanConfig {
                interface_id: "auto".to_string(),
                scan_type: ScanType::Quick,
                port_range: PortRange::Top100,
            }
        }
    }
}

fn parse_port_range(port_range: &str) -> PortRange {
    match port_range {
        "top1000" => PortRange::Top1000,
        "top100" => PortRange::Top100,
        other => {
            log::warn!(
                "Unknown settings.port_range '{}'; defaulting monitor scans to top100",
                other
            );
            PortRange::Top100
        }
    }
}
