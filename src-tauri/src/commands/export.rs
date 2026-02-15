use tauri::State;
use serde::{Deserialize, Serialize};

use crate::db::queries::{devices as db_devices, alerts as db_alerts};
use crate::db::encryption;
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

/// Export devices with optional password encryption
#[tauri::command]
pub fn export_devices_encrypted(
    state: State<'_, AppState>,
    password: Option<String>,
) -> Result<String, String> {
    let conn = state.conn().map_err(|e| e.to_string())?;

    let devices = db_devices::get_all_devices(&conn).map_err(|e| e.to_string())?;
    let alerts = db_alerts::get_alerts(&conn, false).map_err(|e| e.to_string())?;

    let export = ExportData {
        version: 1,
        exported_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        devices,
        alerts,
    };

    let json = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;

    // Encrypt if password provided
    if let Some(pwd) = password {
        encryption::encrypt_notes(&json, &pwd)
    } else {
        Ok(json)
    }
}

/// Import devices with optional password decryption
#[tauri::command]
pub fn import_devices_encrypted(
    state: State<'_, AppState>,
    data: String,
    password: Option<String>,
) -> Result<ImportResult, String> {
    // Decrypt if password provided
    let json_data = if let Some(pwd) = password {
        encryption::decrypt_notes(&data, &pwd)?
    } else {
        data
    };

    let import_data: ExportData = serde_json::from_str(&json_data)
        .map_err(|e| format!("Invalid import data: {}", e))?;

    let conn = state.conn().map_err(|e| e.to_string())?;
    let mut imported = 0u32;
    let mut skipped = 0u32;

    for device in &import_data.devices {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_export_import_roundtrip() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        // Insert test device
        db_devices::insert_device(
            &conn,
            "dev1",
            Some("AA:BB:CC:DD:EE:FF"),
            Some("Test Vendor"),
            Some("test-host"),
            "computer",
            false,
            Some("192.168.1.100"),
        ).unwrap();

        // Export
        let devices = db_devices::get_all_devices(&conn).unwrap();
        let alerts = db_alerts::get_alerts(&conn, false).unwrap();

        let export = ExportData {
            version: 1,
            exported_at: chrono::Utc::now().to_string(),
            devices,
            alerts,
        };

        let json = serde_json::to_string(&export).unwrap();

        // Import (should skip existing)
        let import_data: ExportData = serde_json::from_str(&json).unwrap();
        assert_eq!(import_data.devices.len(), 1);
        assert_eq!(import_data.devices[0].mac_address, Some("AA:BB:CC:DD:EE:FF".to_string()));
    }

    #[test]
    fn test_encrypted_export_import() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        // Insert test device
        db_devices::insert_device(
            &conn,
            "dev1",
            Some("AA:BB:CC:DD:EE:FF"),
            None,
            None,
            "computer",
            false,
            Some("192.168.1.1"),
        ).unwrap();

        // Export with encryption
        let devices = db_devices::get_all_devices(&conn).unwrap();
        let alerts = db_alerts::get_alerts(&conn, false).unwrap();

        let export = ExportData {
            version: 1,
            exported_at: chrono::Utc::now().to_string(),
            devices,
            alerts,
        };

        let json = serde_json::to_string(&export).unwrap();
        let encrypted = encryption::encrypt_notes(&json, "password123").unwrap();

        // Verify it's encrypted (not plain JSON)
        assert!(serde_json::from_str::<ExportData>(&encrypted).is_err());

        // Decrypt and import
        let decrypted = encryption::decrypt_notes(&encrypted, "password123").unwrap();
        let import_data: ExportData = serde_json::from_str(&decrypted).unwrap();

        assert_eq!(import_data.devices.len(), 1);
    }

    #[test]
    fn test_wrong_password_import() {
        let json = r#"{"version":1,"exportedAt":"2024-01-01 00:00:00","devices":[],"alerts":[]}"#;
        let encrypted = encryption::encrypt_notes(json, "password123").unwrap();

        let result = encryption::decrypt_notes(&encrypted, "wrong_password");
        assert!(result.is_err());
    }
}
