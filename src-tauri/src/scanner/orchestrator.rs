use std::time::Instant;

use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::alerts::{engine as alert_engine, notifier};
use crate::db::queries::{devices as db_devices, scans as db_scans, ports as db_ports};
use crate::network::resolver;
use crate::scanner::{fingerprint, passive, ping, port, ScanConfig, ScanResult, ScanType, PortRange};
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
/// Supports cancellation via the provided CancellationToken.
pub async fn run_scan(
    app: AppHandle,
    state: &AppState,
    config: ScanConfig,
    cancel: CancellationToken,
) -> Result<ScanResult, String> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let start = Instant::now();

    // Snapshot previous device state for alert diffing
    let previous_devices = {
        let conn = state.conn().map_err(|e| e.to_string())?;
        db_devices::get_all_devices(&conn).map_err(|e| e.to_string())?
    };

    // Record scan start
    {
        let conn = state.conn().map_err(|e| e.to_string())?;
        db_scans::create_scan(&conn, &scan_id, Some(&config.interface_id), &scan_type_str(&config.scan_type))
            .map_err(|e| e.to_string())?;
    }

    emit_progress(&app, &scan_id, "discovery", 0, 0.0);

    // Check cancellation between phases
    if cancel.is_cancelled() {
        return fail_scan(state, &scan_id, "Scan cancelled");
    }

    // Phase 1: Device discovery (passive ARP table scan)
    let discovered = passive::scan_arp_table();
    let device_count = discovered.len() as u32;

    emit_progress(&app, &scan_id, "discovery", device_count, 20.0);

    if cancel.is_cancelled() {
        return fail_scan(state, &scan_id, "Scan cancelled");
    }

    // Phase 2: Ping sweep for latency (if not passive-only)
    let ping_results = if !matches!(config.scan_type, ScanType::Passive) {
        emit_progress(&app, &scan_id, "ping", device_count, 30.0);
        let ips: Vec<String> = discovered.iter().map(|d| d.ip.clone()).collect();
        ping::ping_sweep(&ips, 20).await
    } else {
        Vec::new()
    };

    if cancel.is_cancelled() {
        return fail_scan(state, &scan_id, "Scan cancelled");
    }

    // Phase 3: Hostname resolution (concurrent, 2s timeout per host)
    emit_progress(&app, &scan_id, "resolving", device_count, 40.0);
    let ips_for_resolve: Vec<String> = discovered
        .iter()
        .filter(|d| d.hostname.is_none())
        .map(|d| d.ip.clone())
        .collect();

    let hostname_results = if !ips_for_resolve.is_empty() {
        resolver::resolve_hostnames(&ips_for_resolve).await
    } else {
        Vec::new()
    };

    if cancel.is_cancelled() {
        return fail_scan(state, &scan_id, "Scan cancelled");
    }

    emit_progress(&app, &scan_id, "enriching", device_count, 50.0);

    // Phase 4: Enrich with OUI data and persist to database
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

            // Merge resolved hostname (prefer ARP-discovered hostname)
            let hostname = device.hostname.clone().or_else(|| {
                hostname_results
                    .iter()
                    .find(|(ip, _)| ip == &device.ip)
                    .and_then(|(_, h)| h.clone())
            });

            // Check if device already exists (by MAC)
            let existing_id = device
                .mac
                .as_deref()
                .and_then(|mac| db_devices::get_device_by_mac(&conn, mac).ok().flatten());

            let device_id = if let Some(id) = existing_id {
                // Update existing device
                db_devices::touch_device(&conn, &id).map_err(|e| e.to_string())?;
                db_devices::upsert_device_ip(&conn, &id, &device.ip)
                    .map_err(|e| e.to_string())?;

                // Update hostname if we resolved one and device doesn't have one yet
                if let Some(ref hn) = hostname {
                    db_devices::update_hostname(&conn, &id, hn)
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
                    hostname.as_deref(),
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

        // Mark devices as departed (previously online, not seen this scan)
        let current_macs: Vec<&str> = discovered
            .iter()
            .filter_map(|d| d.mac.as_deref())
            .collect();

        for prev in &previous_devices {
            if prev.is_online {
                let still_here = prev
                    .mac_address
                    .as_deref()
                    .map(|mac| current_macs.contains(&mac))
                    .unwrap_or(false);

                if !still_here {
                    let _ = app.emit("device:departed", &prev);
                }
            }
        }
    }

    if cancel.is_cancelled() {
        return fail_scan(state, &scan_id, "Scan cancelled");
    }

    // Phase 5: Port scan (full scan only)
    if matches!(config.scan_type, ScanType::Full) {
        emit_progress(&app, &scan_id, "port_scan", device_count, 60.0);

        let ports_to_scan = match config.port_range {
            PortRange::Top100 => port::top_100_ports(),
            PortRange::Top1000 => port::top_100_ports(), // TODO: add top 1000
            PortRange::Custom(ref ports) => ports.clone(),
        };

        for (i, device) in discovered.iter().enumerate() {
            if cancel.is_cancelled() {
                return fail_scan(state, &scan_id, "Scan cancelled");
            }

            let progress = 60.0 + (30.0 * (i as f64 / discovered.len().max(1) as f64));
            emit_progress(&app, &scan_id, "port_scan", device_count, progress);

            let results = port::scan_ports(&device.ip, &ports_to_scan, 100, 2000).await;

            if !results.is_empty() {
                let conn = state.conn().map_err(|e| e.to_string())?;

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

    // Phase 6: OS fingerprinting & device classification (full scan only)
    if matches!(config.scan_type, ScanType::Full) {
        emit_progress(&app, &scan_id, "fingerprinting", device_count, 92.0);

        let conn = state.conn().map_err(|e| e.to_string())?;

        for device in &discovered {
            let device_id = device.mac.as_deref()
                .and_then(|mac| db_devices::get_device_by_mac(&conn, mac).ok().flatten());

            if let Some(ref dev_id) = device_id {
                // Get the ports we just scanned for this device
                let port_results: Vec<port::PortResult> = db_ports::get_latest_ports(&conn, dev_id)
                    .unwrap_or_default()
                    .iter()
                    .map(|p| port::PortResult {
                        port: p.port,
                        state: port::PortState::Open,
                        service_name: p.service_name.clone(),
                        banner: None,
                    })
                    .collect();

                let vendor = device.mac.as_deref()
                    .and_then(|mac| state.oui_db.lookup(mac))
                    .map(|s| s.to_string());

                // OS fingerprinting
                if let Some(os_guess) = fingerprint::guess_os(&port_results, vendor.as_deref()) {
                    db_devices::update_os_guess(&conn, dev_id, &os_guess.os, os_guess.confidence)
                        .map_err(|e| e.to_string())?;
                }

                // Device classification
                let current_os = db_devices::get_device_by_id(&conn, dev_id)
                    .ok()
                    .flatten()
                    .and_then(|d| d.os_guess);

                let device_type = fingerprint::classify_device(
                    &port_results,
                    vendor.as_deref(),
                    current_os.as_deref(),
                    device.is_gateway,
                );

                if device_type != "unknown" {
                    db_devices::update_device_type(&conn, dev_id, device_type)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // Phase 7: Alert evaluation
    emit_progress(&app, &scan_id, "alerts", device_count, 95.0);

    {
        let conn = state.conn().map_err(|e| e.to_string())?;
        let current_devices = db_devices::get_all_devices(&conn).map_err(|e| e.to_string())?;

        match alert_engine::evaluate_alerts(&conn, &previous_devices, &current_devices) {
            Ok(generated) => {
                // Emit each alert to frontend
                for alert in &generated {
                    let _ = app.emit("alert:new", &AlertEvent {
                        alert_type: alert.alert_type.clone(),
                        device_id: alert.device_id.clone(),
                        message: alert.message.clone(),
                        severity: alert.severity.clone(),
                    });
                }

                // Send desktop notifications
                notifier::notify(&app, &generated);

                if !generated.is_empty() {
                    log::info!("Generated {} alerts from scan", generated.len());
                }
            }
            Err(e) => {
                log::error!("Alert evaluation failed: {}", e);
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

/// Alert event emitted to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AlertEvent {
    alert_type: String,
    device_id: Option<String>,
    message: String,
    severity: String,
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

/// Mark a scan as failed in the DB and return an error.
fn fail_scan(state: &AppState, scan_id: &str, reason: &str) -> Result<ScanResult, String> {
    if let Ok(conn) = state.conn() {
        let _ = db_scans::fail_scan(&conn, scan_id);
    }
    Err(reason.to_string())
}
