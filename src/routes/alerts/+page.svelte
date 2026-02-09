<script lang="ts">
	import AlertItem from '$lib/components/alerts/AlertItem.svelte';
	import { alerts, unreadCount, markAllRead } from '$lib/stores/alerts.svelte';
	import { markAllAlertsRead } from '$lib/services/tauri-bridge';
	import { selectedDeviceId } from '$lib/stores/devices.svelte';
	import { goto } from '$app/navigation';
	import type { AlertType } from '$lib/types/alert';

	let filter: AlertType | 'all' = $state('all');

	let filteredAlerts = $derived(
		filter === 'all' ? $alerts : $alerts.filter((a) => a.alertType === filter)
	);

	async function handleMarkAllRead() {
		try {
			await markAllAlertsRead();
			markAllRead();
		} catch (e) {
			console.error('Failed to mark all read:', e);
		}
	}

	function handleDeviceClick(deviceId: string) {
		selectedDeviceId.set(deviceId);
		goto('/');
	}
</script>

<div class="flex h-full flex-col">
	<!-- Toolbar -->
	<div class="flex items-center justify-between border-b border-border px-4 py-3">
		<div class="flex gap-1">
			{#each [
				{ value: 'all', label: 'All' },
				{ value: 'newDevice', label: 'New' },
				{ value: 'deviceDeparted', label: 'Departed' },
				{ value: 'portChanged', label: 'Port Change' },
				{ value: 'unknownDevice', label: 'Unknown' }
			] as option}
				<button
					onclick={() => filter = option.value as typeof filter}
					class="rounded-md px-3 py-1 text-xs transition-colors
						{filter === option.value
							? 'bg-bg-tertiary text-text-primary'
							: 'text-text-secondary hover:bg-bg-tertiary/50'}"
				>
					{option.label}
				</button>
			{/each}
		</div>

		{#if $unreadCount > 0}
			<button
				onclick={handleMarkAllRead}
				class="text-xs text-accent hover:text-accent-hover"
			>
				Mark all read ({$unreadCount})
			</button>
		{/if}
	</div>

	<!-- Alert list -->
	<div class="flex-1 overflow-y-auto">
		{#if filteredAlerts.length === 0}
			<div class="flex h-full items-center justify-center text-text-muted">
				<p>No alerts</p>
			</div>
		{:else}
			{#each filteredAlerts as alert (alert.id)}
				<AlertItem {alert} onDeviceClick={handleDeviceClick} />
			{/each}
		{/if}
	</div>
</div>
