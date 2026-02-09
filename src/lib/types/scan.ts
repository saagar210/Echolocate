export type ScanType = 'quick' | 'full' | 'portOnly' | 'passive';

export type PortRange = 'top100' | 'top1000' | { custom: number[] };

export interface ScanConfig {
	interfaceId: string;
	scanType: ScanType;
	portRange: PortRange;
}

export interface ScanProgress {
	scanId: string;
	phase: string;
	devicesFound: number;
	percentComplete: number;
}

export interface ScanResult {
	scanId: string;
	devicesFound: number;
	newDevices: number;
	durationMs: number;
}

export interface ScanSummary {
	id: string;
	scanType: ScanType;
	status: string;
	devicesFound: number;
	newDevices: number;
	durationMs: number | null;
	startedAt: string;
	completedAt: string | null;
}
