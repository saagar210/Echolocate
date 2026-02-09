use tauri::State;
use serde::{Deserialize, Serialize};

use crate::db::queries::{devices as db_devices, alerts as db_alerts};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub version: u32,
    pub exported_at: String,
    pub devices: Vec<db_devices::Device>,
    pub alerts: Vec<db_alerts::Alert>,
}

#[tauri::command]
pub fn export_devices(state: State<'_, AppState>) -> Result<String, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;

    let devices = db_devices::get_all_devices(&conn).map_err(|e| e.to_string())?;
    let alerts = db_alerts::get_alerts(&conn, false).map_err(|e| e.to_string())?;

    let export = ExportData {
        version: 1,
        exported_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        devices,
        alerts,
    };

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_devices(
    state: State<'_, AppState>,
    json_data: String,
) -> Result<ImportResult, String> {
    let data: ExportData = serde_json::from_str(&json_data)
        .map_err(|e| format!("Invalid import data: {}", e))?;

    let conn = state.conn().map_err(|e| e.to_string())?;
    let mut imported = 0u32;
    let mut skipped = 0u32;

    for device in &data.devices {
        // Check if device already exists by MAC
        let exists = device.mac_address.as_deref()
            .and_then(|mac| db_devices::get_device_by_mac(&conn, mac).ok().flatten())
            .is_some();

        if exists {
            skipped += 1;
            continue;
        }

        let id = uuid::Uuid::new_v4().to_string();
        db_devices::insert_device(
            &conn,
            &id,
            device.mac_address.as_deref(),
            device.vendor.as_deref(),
            device.hostname.as_deref(),
            &device.device_type,
            device.is_gateway,
            device.current_ip.as_deref(),
        ).map_err(|e| e.to_string())?;

        imported += 1;
    }

    Ok(ImportResult { imported, skipped })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: u32,
    pub skipped: u32,
}
