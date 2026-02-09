<script lang="ts">
	import { startScan } from '$lib/services/tauri-bridge';
	import { isScanning } from '$lib/stores/scan.svelte';
	import { activeInterface } from '$lib/stores/settings.svelte';
	import InterfaceSelector from './InterfaceSelector.svelte';

	let scanning = $derived($isScanning);

	async function handleQuickScan() {
		const iface = $activeInterface;
		if (!iface) return;

		try {
			await startScan({
				interfaceId: iface.id,
				scanType: 'quick',
				portRange: 'top100'
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
				portRange: 'top100'
			});
		} catch (e) {
			console.error('Scan failed:', e);
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

	<InterfaceSelector />
</div>
