<script lang="ts">
	import type { Device } from '$lib/types/device';

	let { device, x, y }: { device: Device; x: number; y: number } = $props();

	function timeAgo(dateStr: string): string {
		const date = new Date(dateStr + 'Z');
		const now = new Date();
		const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

		if (seconds < 60) return 'just now';
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
		if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
		return `${Math.floor(seconds / 86400)}d ago`;
	}
</script>

<div
	class="pointer-events-none fixed z-50 rounded-lg border border-border bg-bg-secondary px-3 py-2 text-xs shadow-lg"
	style="left: {x + 12}px; top: {y - 8}px;"
>
	<div class="mb-1 font-semibold text-text-primary">
		{device.customName ?? device.hostname ?? device.currentIp ?? 'Unknown'}
	</div>

	<div class="space-y-0.5 text-text-secondary">
		{#if device.currentIp}
			<div class="font-mono">{device.currentIp}</div>
		{/if}
		{#if device.macAddress}
			<div class="font-mono text-text-muted">{device.macAddress}</div>
		{/if}
		{#if device.vendor}
			<div>{device.vendor}</div>
		{/if}
		{#if device.latencyMs !== null}
			<div>{device.latencyMs.toFixed(1)} ms</div>
		{/if}
		<div class="text-text-muted">Last seen: {timeAgo(device.lastSeen)}</div>
	</div>
</div>
