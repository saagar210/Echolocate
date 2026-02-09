/**
 * Typed wrappers for all Tauri IPC commands.
 * This is the ONLY file that calls invoke().
 */
import { invoke } from '@tauri-apps/api/core';
import type { Device, DeviceUpdate } from '$lib/types/device';
import type { ScanConfig, ScanResult, ScanSummary } from '$lib/types/scan';
import type { Alert, AlertRule, AlertRuleUpdate } from '$lib/types/alert';
import type { NetworkInterface, AppSettings, LatencyPoint, PingResult } from '$lib/types/network';

// ── Scanning ──

export async function startScan(config: ScanConfig): Promise<ScanResult> {
	return invoke('start_scan', { config });
}

export async function stopScan(): Promise<void> {
	return invoke('stop_scan');
}

export async function getScanHistory(limit: number = 50): Promise<ScanSummary[]> {
	return invoke('get_scan_history', { limit });
}

// ── Devices ──

export async function getDevices(): Promise<Device[]> {
	return invoke('get_devices');
}

export async function getDevice(deviceId: string): Promise<Device> {
	return invoke('get_device', { deviceId });
}

export async function updateDevice(deviceId: string, updates: DeviceUpdate): Promise<Device> {
	return invoke('update_device', { deviceId, updates });
}

export async function deleteDevice(deviceId: string): Promise<void> {
	return invoke('delete_device', { deviceId });
}

// ── Alerts ──

export async function getAlerts(unreadOnly: boolean = false): Promise<Alert[]> {
	return invoke('get_alerts', { unreadOnly });
}

export async function markAlertRead(alertId: string): Promise<void> {
	return invoke('mark_alert_read', { alertId });
}

export async function markAllAlertsRead(): Promise<void> {
	return invoke('mark_all_alerts_read');
}

export async function getAlertRules(): Promise<AlertRule[]> {
	return invoke('get_alert_rules');
}

export async function updateAlertRule(ruleId: string, updates: AlertRuleUpdate): Promise<AlertRule> {
	return invoke('update_alert_rule', { ruleId, updates });
}

// ── Network ──

export async function getInterfaces(): Promise<NetworkInterface[]> {
	return invoke('get_interfaces');
}

// ── Monitoring ──

export async function startMonitor(intervalSecs: number): Promise<void> {
	return invoke('start_monitor', { intervalSecs });
}

export async function stopMonitor(): Promise<void> {
	return invoke('stop_monitor');
}

// ── Settings ──

export async function getSettings(): Promise<AppSettings> {
	return invoke('get_settings');
}

export async function updateSettings(settings: AppSettings): Promise<void> {
	return invoke('update_settings', { settings });
}

// ── Latency ──

export async function getLatencyHistory(deviceId: string, hours: number = 24): Promise<LatencyPoint[]> {
	return invoke('get_latency_history', { deviceId, hours });
}

export async function pingDevice(ip: string): Promise<PingResult> {
	return invoke('ping_device', { ip });
}

// ── Export/Import ──

export async function exportDevices(): Promise<string> {
	return invoke('export_devices');
}

export async function importDevices(jsonData: string): Promise<{ imported: number; skipped: number }> {
	return invoke('import_devices', { jsonData });
}
