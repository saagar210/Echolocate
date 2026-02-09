export interface NetworkInterface {
	id: string;
	name: string;
	ipAddress: string | null;
	subnetMask: string | null;
	macAddress: string | null;
	gatewayIp: string | null;
	isActive: boolean;
}

export interface AppSettings {
	defaultInterfaceId: string | null;
	scanIntervalSecs: number;
	portRange: 'top100' | 'top1000';
	theme: 'dark' | 'light';
	graphRepulsion: number;
	graphLinkDistance: number;
	graphGravity: number;
}

export interface LatencyPoint {
	latencyMs: number;
	measuredAt: string;
}

export interface PingResult {
	ip: string;
	latencyMs: number | null;
	success: boolean;
}
