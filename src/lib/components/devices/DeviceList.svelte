<script lang="ts">
	import type { Device } from '$lib/types/device';
	import StatusBadge from '../ui/StatusBadge.svelte';

	let {
		devices,
		selectedId = null,
		onSelectDevice,
		searchQuery = ''
	}: {
		devices: Device[];
		selectedId: string | null;
		onSelectDevice: (id: string) => void;
		searchQuery?: string;
	} = $props();

	type SortKey = 'status' | 'name' | 'ip' | 'mac' | 'vendor' | 'type' | 'latency';
	type SortDir = 'asc' | 'desc';

	let sortKey: SortKey = $state('status');
	let sortDir: SortDir = $state('asc');

	function toggleSort(key: SortKey) {
		if (sortKey === key) {
			sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = key;
			sortDir = 'asc';
		}
	}

	let filteredDevices = $derived(() => {
		let list = devices;

		// Filter
		if (searchQuery) {
			const q = searchQuery.toLowerCase();
			list = list.filter(
				(d) =>
					(d.customName?.toLowerCase().includes(q)) ||
					(d.hostname?.toLowerCase().includes(q)) ||
					(d.currentIp?.toLowerCase().includes(q)) ||
					(d.macAddress?.toLowerCase().includes(q)) ||
					(d.vendor?.toLowerCase().includes(q))
			);
		}

		// Sort
		list = [...list].sort((a, b) => {
			let cmp = 0;
			switch (sortKey) {
				case 'status':
					cmp = (a.isOnline === b.isOnline) ? 0 : a.isOnline ? -1 : 1;
					break;
				case 'name':
					cmp = displayName(a).localeCompare(displayName(b));
					break;
				case 'ip':
					cmp = compareIps(a.currentIp, b.currentIp);
					break;
				case 'mac':
					cmp = (a.macAddress ?? '').localeCompare(b.macAddress ?? '');
					break;
				case 'vendor':
					cmp = (a.vendor ?? 'zzz').localeCompare(b.vendor ?? 'zzz');
					break;
				case 'type':
					cmp = a.deviceType.localeCompare(b.deviceType);
					break;
				case 'latency':
					cmp = (a.latencyMs ?? 9999) - (b.latencyMs ?? 9999);
					break;
			}
			return sortDir === 'desc' ? -cmp : cmp;
		});

		return list;
	});

	function displayName(d: Device): string {
		return d.customName ?? d.hostname ?? d.vendor ?? d.currentIp ?? 'Unknown';
	}

	function compareIps(a: string | null, b: string | null): number {
		if (!a && !b) return 0;
		if (!a) return 1;
		if (!b) return -1;
		const pa = a.split('.').map(Number);
		const pb = b.split('.').map(Number);
		for (let i = 0; i < 4; i++) {
			if (pa[i] !== pb[i]) return pa[i] - pb[i];
		}
		return 0;
	}

	function sortIndicator(key: SortKey): string {
		if (sortKey !== key) return '';
		return sortDir === 'asc' ? ' ↑' : ' ↓';
	}
</script>

<div class="overflow-auto">
	<table class="w-full text-sm">
		<thead class="sticky top-0 bg-bg-secondary">
			<tr class="text-left text-xs text-text-muted border-b border-border">
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('status')}>
					Status{sortIndicator('status')}
				</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('name')}>
					Name{sortIndicator('name')}
				</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('ip')}>
					IP{sortIndicator('ip')}
				</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('mac')}>
					MAC{sortIndicator('mac')}
				</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('vendor')}>
					Vendor{sortIndicator('vendor')}
				</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('type')}>
					Type{sortIndicator('type')}
				</th>
				<th class="px-3 py-2">Ports</th>
				<th class="cursor-pointer px-3 py-2 hover:text-text-primary" onclick={() => toggleSort('latency')}>
					Latency{sortIndicator('latency')}
				</th>
			</tr>
		</thead>
		<tbody>
			{#each filteredDevices() as device (device.id)}
				<tr
					class="cursor-pointer border-b border-border/30 transition-colors
						{device.id === selectedId ? 'bg-accent/10' : 'hover:bg-bg-tertiary/30'}"
					onclick={() => onSelectDevice(device.id)}
				>
					<td class="px-3 py-2">
						<StatusBadge online={device.isOnline} />
					</td>
					<td class="px-3 py-2 font-medium text-text-primary">
						{displayName(device)}
					</td>
					<td class="px-3 py-2 font-mono text-text-secondary">
						{device.currentIp ?? '—'}
					</td>
					<td class="px-3 py-2 font-mono text-xs text-text-muted">
						{device.macAddress ?? '—'}
					</td>
					<td class="px-3 py-2 text-text-secondary">
						{device.vendor ?? '—'}
					</td>
					<td class="px-3 py-2 capitalize text-text-secondary">
						{device.deviceType}
					</td>
					<td class="px-3 py-2 text-text-muted">
						{#if device.openPorts.length > 0}
							{device.openPorts.map(p => p.port).join(', ')}
						{:else}
							—
						{/if}
					</td>
					<td class="px-3 py-2 text-text-secondary">
						{device.latencyMs !== null ? `${device.latencyMs.toFixed(1)} ms` : '—'}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
