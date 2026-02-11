/**
 * Typed Tauri event subscriptions.
 * This is the ONLY file that calls listen().
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Device } from '$lib/types/device';
import type { ScanProgress, ScanResult } from '$lib/types/scan';
import type { Alert } from '$lib/types/alert';

export interface EventHandlers {
	onScanProgress: (progress: ScanProgress) => void;
	onDeviceDiscovered: (device: Device) => void;
	onScanCompleted: (result: ScanResult) => void;
	onScanError: (error: { message: string }) => void;
	onDeviceUpdated: (device: Device) => void;
	onDeviceDeparted: (data: { deviceId: string }) => void;
	onAlertNew: (alert: Alert) => void;
	onMonitorStatus: (status: { isRunning: boolean; nextScanIn: number | null }) => void;
}

const EVENT_NAMES = {
	SCAN_PROGRESS: 'scan:progress',
	DEVICE_DISCOVERED: 'scan:device-discovered',
	SCAN_COMPLETED: 'scan:completed',
	SCAN_ERROR: 'scan:error',
	DEVICE_UPDATED: 'device:updated',
	DEVICE_DEPARTED: 'device:departed',
	ALERT_NEW: 'alert:new',
	MONITOR_STATUS: 'monitor:status'
} as const;

/**
 * Subscribe to all Tauri backend events.
 * Returns an array of unlisten functions for cleanup.
 */
export async function subscribeAll(handlers: EventHandlers): Promise<UnlistenFn[]> {
	const unlisteners = await Promise.all([
		listen<ScanProgress>(EVENT_NAMES.SCAN_PROGRESS, (event) => {
			handlers.onScanProgress(event.payload);
		}),
		listen<Device>(EVENT_NAMES.DEVICE_DISCOVERED, (event) => {
			handlers.onDeviceDiscovered(event.payload);
		}),
		listen<ScanResult>(EVENT_NAMES.SCAN_COMPLETED, (event) => {
			handlers.onScanCompleted(event.payload);
		}),
		listen<{ message: string }>(EVENT_NAMES.SCAN_ERROR, (event) => {
			handlers.onScanError(event.payload);
		}),
		listen<Device>(EVENT_NAMES.DEVICE_UPDATED, (event) => {
			handlers.onDeviceUpdated(event.payload);
		}),
		listen<{ deviceId: string }>(EVENT_NAMES.DEVICE_DEPARTED, (event) => {
			handlers.onDeviceDeparted(event.payload);
		}),
		listen<Alert>(EVENT_NAMES.ALERT_NEW, (event) => {
			handlers.onAlertNew(event.payload);
		}),
		listen<{ isRunning: boolean; nextScanIn: number | null }>(EVENT_NAMES.MONITOR_STATUS, (event) => {
			handlers.onMonitorStatus(event.payload);
		})
	]);

	return unlisteners;
}

/**
 * Unsubscribe from all events.
 */
export function unsubscribeAll(unlisteners: UnlistenFn[]): void {
	for (const unlisten of unlisteners) {
		unlisten();
	}
}
