import {
	forceSimulation,
	forceLink,
	forceManyBody,
	forceCenter,
	forceCollide,
	forceY,
	type Simulation,
	type SimulationNodeDatum,
	type SimulationLinkDatum
} from 'd3-force';
import type { Device } from '$lib/types/device';

export interface GraphNode extends SimulationNodeDatum {
	id: string;
	device: Device;
	radius: number;
	color: string;
	pinned: boolean;
}

export interface GraphLink extends SimulationLinkDatum<GraphNode> {
	source: string | GraphNode;
	target: string | GraphNode;
}

/** Get node radius based on device type */
export function getNodeRadius(device: Device): number {
	if (device.isGateway) return 24;
	switch (device.deviceType) {
		case 'router': return 24;
		case 'computer': return 18;
		case 'phone':
		case 'tablet': return 14;
		case 'iot': return 12;
		case 'printer': return 14;
		default: return 15;
	}
}

/** Get node color based on device type and OS */
export function getNodeColor(device: Device): string {
	if (device.isGateway) return '#f59e0b'; // amber
	if (device.osGuess) {
		const os = device.osGuess.toLowerCase();
		if (os.includes('macos') || os.includes('ios')) return '#3b82f6'; // blue
		if (os.includes('linux')) return '#22c55e'; // green
		if (os.includes('windows')) return '#a855f7'; // purple
		if (os.includes('android')) return '#22c55e'; // green
	}
	switch (device.deviceType) {
		case 'computer': return '#3b82f6';
		case 'phone':
		case 'tablet': return '#ec4899';
		case 'iot': return '#14b8a6';
		case 'printer': return '#8b5cf6';
		default: return '#64748b';
	}
}

/** Get latency-based edge color */
export function getEdgeColor(latencyMs: number | null): string {
	if (latencyMs === null) return '#334155';
	if (latencyMs < 10) return '#22c55e'; // green
	if (latencyMs < 50) return '#eab308'; // yellow
	return '#ef4444'; // red
}

/** Convert devices to graph nodes and links (star topology centered on gateway) */
export function devicesToGraph(devices: Device[]): { nodes: GraphNode[]; links: GraphLink[] } {
	const nodes: GraphNode[] = devices.map((device) => ({
		id: device.id,
		device,
		radius: getNodeRadius(device),
		color: getNodeColor(device),
		pinned: false
	}));

	const gateway = devices.find((d) => d.isGateway);
	const links: GraphLink[] = [];

	if (gateway) {
		for (const device of devices) {
			if (device.id !== gateway.id) {
				links.push({
					source: device.id,
					target: gateway.id
				});
			}
		}
	}

	return { nodes, links };
}

/** Create and configure a d3-force simulation */
export function createSimulation(
	nodes: GraphNode[],
	links: GraphLink[],
	width: number,
	height: number,
	options?: { repulsion?: number; linkDistance?: number; gravity?: number }
): Simulation<GraphNode, GraphLink> {
	const repulsion = options?.repulsion ?? 300;
	const linkDistance = options?.linkDistance ?? 100;
	const gravity = options?.gravity ?? 0.1;

	const simulation = forceSimulation<GraphNode>(nodes)
		.force(
			'link',
			forceLink<GraphNode, GraphLink>(links)
				.id((d) => d.id)
				.distance(linkDistance)
		)
		.force('charge', forceManyBody<GraphNode>().strength(-repulsion))
		.force('center', forceCenter(width / 2, height / 2))
		.force('gravity', forceY<GraphNode>(height / 2).strength(gravity))
		.force(
			'collision',
			forceCollide<GraphNode>().radius((d) => d.radius + 4)
		);

	return simulation;
}
