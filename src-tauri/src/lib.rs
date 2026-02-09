pub mod alerts;
pub mod commands;
pub mod db;
pub mod network;
pub mod scanner;
pub mod state;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_pool = db::init_db(&app_data_dir)
                .expect("Failed to initialize database");

            let oui_db = network::oui::OuiDatabase::load(app.handle())
                .unwrap_or_else(|e| {
                    log::warn!("Failed to load OUI database: {}. Vendor lookups will be unavailable.", e);
                    network::oui::OuiDatabase::empty()
                });

            let app_state = AppState::new(db_pool, oui_db);
            app.manage(app_state);

            log::info!("Echolocate initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan::start_scan,
            commands::scan::stop_scan,
            commands::scan::get_scan_history,
            commands::device::get_devices,
            commands::device::get_device,
            commands::device::update_device,
            commands::device::delete_device,
            commands::alert::get_alerts,
            commands::alert::mark_alert_read,
            commands::alert::mark_all_alerts_read,
            commands::alert::get_alert_rules,
            commands::alert::update_alert_rule,
            commands::settings::get_interfaces,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::start_monitor,
            commands::settings::stop_monitor,
            commands::settings::get_latency_history,
            commands::settings::ping_device,
            commands::export::export_devices,
            commands::export::import_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
