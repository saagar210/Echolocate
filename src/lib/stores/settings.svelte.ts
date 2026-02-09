import { writable, derived } from 'svelte/store';
import type { NetworkInterface, AppSettings } from '$lib/types/network';

/** Available network interfaces */
export const interfaces = writable<NetworkInterface[]>([]);

/** App settings */
export const settings = writable<AppSettings>({
	defaultInterfaceId: null,
	scanIntervalSecs: 60,
	portRange: 'top100',
	theme: 'dark',
	graphRepulsion: 300,
	graphLinkDistance: 100,
	graphGravity: 0.1
});

/** The currently active interface (derived from settings + interfaces) */
export const activeInterface = derived(
	[interfaces, settings],
	([$interfaces, $settings]) => {
		if ($settings.defaultInterfaceId) {
			return $interfaces.find((i) => i.id === $settings.defaultInterfaceId) ?? $interfaces.find((i) => i.isActive) ?? null;
		}
		return $interfaces.find((i) => i.isActive) ?? null;
	}
);

export function setInterfaces(list: NetworkInterface[]): void {
	interfaces.set(list);
}

export function setSettings(s: AppSettings): void {
	settings.set(s);
}
