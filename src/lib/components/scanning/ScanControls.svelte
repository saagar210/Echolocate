<script lang="ts">
	import { startScan, startMonitor, stopMonitor } from '$lib/services/tauri-bridge';
	import { isScanning, monitoringActive } from '$lib/stores/scan.svelte';
	import { activeInterface, settings } from '$lib/stores/settings.svelte';
	import InterfaceSelector from './InterfaceSelector.svelte';

	let scanning = $derived($isScanning);
	let monitoring = $derived($monitoringActive);

	async function handleQuickScan() {
		const iface = $activeInterface;
		if (!iface) return;

		try {
			await startScan({
				interfaceId: iface.id,
				scanType: 'quick',
				portRange: $settings.portRange
			});
		} catch (e) {
			console.error('Scan failed:', e);
		}
	}

	async function handleFullScan() {
		const iface = $activeInterface;
		if (!iface) return;

		try {
			await startScan({
				interfaceId: iface.id,
				scanType: 'full',
				portRange: $settings.portRange
			});
		} catch (e) {
			console.error('Scan failed:', e);
		}
	}

	async function toggleMonitor() {
		try {
			if (monitoring) {
				await stopMonitor();
			} else {
				await startMonitor($settings.scanIntervalSecs);
			}
		} catch (e) {
			console.error('Failed to toggle monitor:', e);
		}
	}
</script>

<div class="flex items-center gap-2">
	<button
		onclick={handleQuickScan}
		disabled={scanning}
		class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-bg-primary transition-colors
			hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
	>
		{scanning ? 'Scanning...' : 'Quick Scan'}
	</button>

	<button
		onclick={handleFullScan}
		disabled={scanning}
		class="rounded-lg border border-border bg-bg-secondary px-3 py-1.5 text-sm font-medium text-text-primary transition-colors
			hover:bg-bg-tertiary disabled:opacity-50 disabled:cursor-not-allowed"
	>
		Full Scan
	</button>

	<button
		onclick={toggleMonitor}
		disabled={scanning}
		class="rounded-lg border border-border bg-bg-secondary px-3 py-1.5 text-sm font-medium transition-colors
			{monitoring
				? 'text-warning hover:bg-warning/10'
				: 'text-text-primary hover:bg-bg-tertiary'}
			disabled:opacity-50 disabled:cursor-not-allowed"
	>
		{monitoring ? 'Stop Monitor' : 'Start Monitor'}
	</button>

	<InterfaceSelector />
</div>
