use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use super::engine::GeneratedAlert;

/// Send desktop notifications for alerts that have notify_desktop enabled.
pub fn notify(app: &AppHandle, alerts: &[GeneratedAlert]) {
    for alert in alerts {
        if !alert.notify_desktop {
            continue;
        }

        let title = match alert.severity.as_str() {
            "critical" => "Echolocate - Critical Alert",
            "warning" => "Echolocate - Warning",
            _ => "Echolocate",
        };

        if let Err(e) = app
            .notification()
            .builder()
            .title(title)
            .body(&alert.message)
            .show()
        {
            log::warn!("Failed to send notification: {}", e);
        }
    }
}
