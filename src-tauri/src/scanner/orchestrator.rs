use std::time::Instant;

use tauri::{AppHandle, Emitter};

use crate::db::queries::{devices as db_devices, scans as db_scans, ports as db_ports};
use crate::scanner::{passive, ping, port, ScanConfig, ScanResult, ScanType, PortRange};
use crate::state::AppState;

/// Progress update sent to the frontend during a scan.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgress {
    scan_id: String,
    phase: String,
    devices_found: u32,
    percent_complete: f64,
}

/// Run a scan based on the provided configuration.
pub async fn run_scan(
    app: AppHandle,
    state: &AppState,
    config: ScanConfig,
) -> Result<ScanResult, String> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let start = Instant::now();

    // Record scan start
    {
        let conn = state.conn().map_err(|e| e.to_string())?;
        db_scans::create_scan(&conn, &scan_id, Some(&config.interface_id), &scan_type_str(&config.scan_type))
            .map_err(|e| e.to_string())?;
    }

    emit_progress(&app, &scan_id, "discovery", 0, 0.0);

    // Phase 1: Device discovery (passive ARP table scan)
    let discovered = passive::scan_arp_table();
    let device_count = discovered.len() as u32;

    emit_progress(&app, &scan_id, "discovery", device_count, 20.0);

    // Phase 2: Ping sweep for latency (if not passive-only)
    let ping_results = if !matches!(config.scan_type, ScanType::Passive) {
        emit_progress(&app, &scan_id, "ping", device_count, 30.0);
        let ips: Vec<String> = discovered.iter().map(|d| d.ip.clone()).collect();
        ping::ping_sweep(&ips, 20).await
    } else {
        Vec::new()
    };

    emit_progress(&app, &scan_id, "enriching", device_count, 50.0);

    // Phase 3: Enrich with OUI data and persist to database
    let mut new_device_count = 0u32;

    {
        let conn = state.conn().map_err(|e| e.to_string())?;

        for device in &discovered {
            let vendor = device
                .mac
                .as_deref()
                .and_then(|mac| state.oui_db.lookup(mac))
                .map(|s| s.to_string());

            let latency = ping_results
                .iter()
                .find(|(ip, _)| ip == &device.ip)
                .and_then(|(_, lat)| *lat);

            // Check if device already exists (by MAC)
            let existing_id = device
                .mac
                .as_deref()
                .and_then(|mac| db_devices::get_device_by_mac(&conn, mac).ok().flatten());

            let device_id = if let Some(id) = existing_id {
                // Update existing device
                db_devices::touch_device(&conn, &id).map_err(|e| e.to_string())?;
                if let Some(ref _ip) = Some(&device.ip) {
                    db_devices::upsert_device_ip(&conn, &id, &device.ip)
                        .map_err(|e| e.to_string())?;
                }
                id
            } else {
                // Insert new device
                let id = uuid::Uuid::new_v4().to_string();
                let device_type = if device.is_gateway { "router" } else { "unknown" };
                db_devices::insert_device(
                    &conn,
                    &id,
                    device.mac.as_deref(),
                    vendor.as_deref(),
                    device.hostname.as_deref(),
                    device_type,
                    device.is_gateway,
                    Some(&device.ip),
                )
                .map_err(|e| e.to_string())?;
                new_device_count += 1;
                id
            };

            // Record latency
            if let Some(lat) = latency {
                db_devices::record_latency(&conn, &device_id, lat)
                    .map_err(|e| e.to_string())?;
            }

            // Emit device discovered event
            if let Ok(Some(full_device)) = db_devices::get_device_by_id(&conn, &device_id) {
                let _ = app.emit("scan:device-discovered", &full_device);
            }
        }
    }

    // Phase 4: Port scan (full scan only)
    if matches!(config.scan_type, ScanType::Full) {
        emit_progress(&app, &scan_id, "port_scan", device_count, 60.0);

        let ports_to_scan = match config.port_range {
            PortRange::Top100 => port::top_100_ports(),
            PortRange::Top1000 => port::top_100_ports(), // TODO: add top 1000
            PortRange::Custom(ref ports) => ports.clone(),
        };

        for (i, device) in discovered.iter().enumerate() {
            let progress = 60.0 + (30.0 * (i as f64 / discovered.len().max(1) as f64));
            emit_progress(&app, &scan_id, "port_scan", device_count, progress);

            let results = port::scan_ports(&device.ip, &ports_to_scan, 100, 2000).await;

            if !results.is_empty() {
                let conn = state.conn().map_err(|e| e.to_string())?;

                // Find the device ID for this IP
                let device_id = device.mac.as_deref()
                    .and_then(|mac| db_devices::get_device_by_mac(&conn, mac).ok().flatten());

                if let Some(ref dev_id) = device_id {
                    for pr in &results {
                        db_ports::insert_port(
                            &conn,
                            dev_id,
                            &scan_id,
                            pr.port,
                            "tcp",
                            &pr.state.to_string(),
                            pr.service_name.as_deref(),
                            pr.banner.as_deref(),
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    // Finalize
    let duration_ms = start.elapsed().as_millis() as u64;

    {
        let conn = state.conn().map_err(|e| e.to_string())?;
        db_scans::complete_scan(&conn, &scan_id, device_count, new_device_count, duration_ms)
            .map_err(|e| e.to_string())?;
    }

    let result = ScanResult {
        scan_id: scan_id.clone(),
        devices_found: device_count,
        new_devices: new_device_count,
        duration_ms,
    };

    emit_progress(&app, &scan_id, "completed", device_count, 100.0);
    let _ = app.emit("scan:completed", &result);

    log::info!(
        "Scan {} completed: {} devices found, {} new, {}ms",
        scan_id, device_count, new_device_count, duration_ms
    );

    Ok(result)
}

fn emit_progress(app: &AppHandle, scan_id: &str, phase: &str, devices_found: u32, percent: f64) {
    let _ = app.emit("scan:progress", ScanProgress {
        scan_id: scan_id.to_string(),
        phase: phase.to_string(),
        devices_found,
        percent_complete: percent,
    });
}

fn scan_type_str(scan_type: &ScanType) -> &'static str {
    match scan_type {
        ScanType::Quick => "quick",
        ScanType::Full => "full",
        ScanType::PortOnly => "port_only",
        ScanType::Passive => "passive",
    }
}
