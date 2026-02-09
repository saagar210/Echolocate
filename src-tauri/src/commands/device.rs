use tauri::State;

use crate::db::queries::devices as db_devices;
use crate::state::AppState;

#[tauri::command]
pub fn get_devices(state: State<'_, AppState>) -> Result<Vec<db_devices::Device>, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_devices::get_all_devices(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<db_devices::Device, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_devices::get_device_by_id(&conn, &device_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Device not found: {}", device_id))
}

#[tauri::command]
pub fn update_device(
    state: State<'_, AppState>,
    device_id: String,
    updates: db_devices::DeviceUpdate,
) -> Result<db_devices::Device, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_devices::update_device(&conn, &device_id, &updates).map_err(|e| e.to_string())?;
    db_devices::get_device_by_id(&conn, &device_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Device not found: {}", device_id))
}

#[tauri::command]
pub fn delete_device(state: State<'_, AppState>, device_id: String) -> Result<(), String> {
    let conn = state.conn().map_err(|e| e.to_string())?;
    db_devices::delete_device(&conn, &device_id).map_err(|e| e.to_string())
}
