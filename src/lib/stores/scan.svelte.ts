import { writable, derived } from 'svelte/store';
import type { ScanProgress, ScanSummary } from '$lib/types/scan';

/** Whether a scan is currently running */
export const isScanning = writable<boolean>(false);

/** Current scan progress */
export const scanProgress = writable<ScanProgress | null>(null);

/** Whether continuous monitoring is active */
export const monitoringActive = writable<boolean>(false);

/** Next monitor scan countdown in seconds */
export const nextScanIn = writable<number>(0);

/** Scan history */
export const scanHistory = writable<ScanSummary[]>([]);

/** Human-readable scan status */
export const scanStatus = derived(
	[isScanning, scanProgress, monitoringActive],
	([$scanning, $progress, $monitoring]) => {
		if ($scanning && $progress) {
			return `Scanning: ${$progress.phase} (${Math.round($progress.percentComplete)}%)`;
		}
		if ($scanning) return 'Scanning...';
		if ($monitoring) return 'Monitoring';
		return 'Idle';
	}
);

export function updateProgress(progress: ScanProgress): void {
	scanProgress.set(progress);
	isScanning.set(true);
}

export function completeScan(): void {
	isScanning.set(false);
	scanProgress.set(null);
}

export function updateMonitorStatus(running: boolean, countdown: number | null): void {
	monitoringActive.set(running);
	nextScanIn.set(countdown ?? 0);
}
