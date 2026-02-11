import { describe, it, expect } from 'vitest';
import { devicesToGraph, getEdgeColor } from './graph-layout';
import type { Device } from '$lib/types/device';

function makeDevice(overrides: Partial<Device>): Device {
	return {
		id: 'dev-1',
		macAddress: null,
		vendor: null,
		hostname: null,
		customName: null,
		deviceType: 'unknown',
		osGuess: null,
		osConfidence: 0,
		isTrusted: false,
		isGateway: false,
		notes: null,
		currentIp: null,
		isOnline: true,
		latencyMs: null,
		openPorts: [],
		firstSeen: '2026-01-01 00:00:00',
		lastSeen: '2026-01-01 00:00:00',
		...overrides
	};
}

describe('graph-layout', () => {
	it('builds star links around the gateway', () => {
		const gateway = makeDevice({ id: 'gw', isGateway: true, deviceType: 'router' });
		const a = makeDevice({ id: 'a' });
		const b = makeDevice({ id: 'b' });

		const graph = devicesToGraph([gateway, a, b]);
		expect(graph.nodes).toHaveLength(3);
		expect(graph.links).toHaveLength(2);
		expect(graph.links.map((l) => l.target)).toEqual(['gw', 'gw']);
	});

	it('maps latency bands to edge colors', () => {
		expect(getEdgeColor(null)).toBe('#334155');
		expect(getEdgeColor(5)).toBe('#22c55e');
		expect(getEdgeColor(20)).toBe('#eab308');
		expect(getEdgeColor(80)).toBe('#ef4444');
	});
});
