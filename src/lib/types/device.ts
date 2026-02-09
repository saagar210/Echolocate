export type DeviceType = 'router' | 'computer' | 'phone' | 'tablet' | 'iot' | 'printer' | 'unknown';

export interface Device {
	id: string;
	macAddress: string | null;
	vendor: string | null;
	hostname: string | null;
	customName: string | null;
	deviceType: DeviceType;
	osGuess: string | null;
	osConfidence: number;
	isTrusted: boolean;
	isGateway: boolean;
	notes: string | null;
	currentIp: string | null;
	isOnline: boolean;
	latencyMs: number | null;
	openPorts: PortInfo[];
	firstSeen: string;
	lastSeen: string;
}

export interface PortInfo {
	port: number;
	protocol: string;
	state: string;
	serviceName: string | null;
	banner: string | null;
}

export interface DeviceUpdate {
	customName?: string | null;
	deviceType?: DeviceType;
	isTrusted?: boolean;
	notes?: string | null;
}

/** Node representation for d3-force graph */
export interface DeviceNode extends d3.SimulationNodeDatum {
	id: string;
	device: Device;
	radius: number;
	color: string;
	pinned: boolean;
}

/** Edge representation for d3-force graph */
export interface DeviceLink extends d3.SimulationLinkDatum<DeviceNode> {
	source: string | DeviceNode;
	target: string | DeviceNode;
	latencyMs: number | null;
}
