<script lang="ts">
	import type { Alert } from '$lib/types/alert';
	import { markAlertRead } from '$lib/services/tauri-bridge';
	import { markRead } from '$lib/stores/alerts.svelte';

	let { alert, onDeviceClick }: { alert: Alert; onDeviceClick?: (id: string) => void } = $props();

	function severityColor(severity: string): string {
		switch (severity) {
			case 'critical': return 'text-danger';
			case 'warning': return 'text-warning';
			default: return 'text-info';
		}
	}

	function typeIcon(type: string): string {
		switch (type) {
			case 'newDevice': return '+';
			case 'deviceDeparted': return '−';
			case 'portChanged': return '⇄';
			case 'unknownDevice': return '?';
			default: return '•';
		}
	}

	function timeAgo(dateStr: string): string {
		const date = new Date(dateStr + 'Z');
		const now = new Date();
		const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);
		if (seconds < 60) return 'just now';
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
		if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
		return `${Math.floor(seconds / 86400)}d ago`;
	}

	async function handleClick() {
		if (!alert.isRead) {
			try {
				await markAlertRead(alert.id);
				markRead(alert.id);
			} catch (e) {
				console.error('Failed to mark alert read:', e);
			}
		}
		if (alert.deviceId && onDeviceClick) {
			onDeviceClick(alert.deviceId);
		}
	}
</script>

<button
	onclick={handleClick}
	class="w-full text-left px-4 py-3 border-b border-border/30 transition-colors hover:bg-bg-tertiary/30
		{alert.isRead ? 'opacity-60' : ''}"
>
	<div class="flex items-start gap-3">
		<span class="mt-0.5 text-sm {severityColor(alert.severity)}">
			{typeIcon(alert.alertType)}
		</span>
		<div class="flex-1 min-w-0">
			<p class="text-sm text-text-primary {alert.isRead ? '' : 'font-medium'}">
				{alert.message}
			</p>
			<p class="mt-0.5 text-xs text-text-muted">
				{timeAgo(alert.createdAt)}
			</p>
		</div>
		{#if !alert.isRead}
			<span class="mt-1.5 h-2 w-2 rounded-full bg-accent flex-shrink-0"></span>
		{/if}
	</div>
</button>
