import { writable, derived } from 'svelte/store';
import type { Device } from '$lib/types/device';

/** All known devices, keyed by ID */
const deviceMap = writable<Map<string, Device>>(new Map());

/** Currently selected device ID */
export const selectedDeviceId = writable<string | null>(null);

/** All devices as a sorted array (online first, then by IP) */
export const devices = derived(deviceMap, ($map) => {
	return Array.from($map.values()).sort((a, b) => {
		// Online first
		if (a.isOnline !== b.isOnline) return a.isOnline ? -1 : 1;
		// Gateway first among online
		if (a.isGateway !== b.isGateway) return a.isGateway ? -1 : 1;
		// Then by IP
		return compareIps(a.currentIp, b.currentIp);
	});
});

/** Online device count */
export const onlineCount = derived(deviceMap, ($map) => {
	let count = 0;
	for (const d of $map.values()) {
		if (d.isOnline) count++;
	}
	return count;
});

/** The currently selected device (full object) */
export const selectedDevice = derived(
	[deviceMap, selectedDeviceId],
	([$map, $id]) => ($id ? $map.get($id) ?? null : null)
);

/** Replace all devices (initial load) */
export function setDevices(list: Device[]): void {
	const map = new Map<string, Device>();
	for (const d of list) {
		map.set(d.id, d);
	}
	deviceMap.set(map);
}

/** Add or update a single device */
export function upsertDevice(device: Device): void {
	deviceMap.update((map) => {
		const next = new Map(map);
		next.set(device.id, device);
		return next;
	});
}

/** Remove a device */
export function removeDevice(deviceId: string): void {
	deviceMap.update((map) => {
		const next = new Map(map);
		next.delete(deviceId);
		return next;
	});
	// Deselect if this device was selected
	selectedDeviceId.update((id) => (id === deviceId ? null : id));
}

/** Mark a device as offline (departed) */
export function markDeparted(deviceId: string): void {
	deviceMap.update((map) => {
		const device = map.get(deviceId);
		if (!device) return map;
		const next = new Map(map);
		next.set(deviceId, { ...device, isOnline: false });
		return next;
	});
}

/** Compare IP addresses for sorting */
function compareIps(a: string | null, b: string | null): number {
	if (!a && !b) return 0;
	if (!a) return 1;
	if (!b) return -1;
	const partsA = a.split('.').map(Number);
	const partsB = b.split('.').map(Number);
	for (let i = 0; i < 4; i++) {
		if (partsA[i] !== partsB[i]) return partsA[i] - partsB[i];
	}
	return 0;
}
