<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import '../app.css';
	import { getDevices, getAlerts, getSettings, getInterfaces, startScan } from '$lib/services/tauri-bridge';
	import { subscribeAll, unsubscribeAll } from '$lib/services/tauri-events';
	import { setDevices, upsertDevice, markDeparted, devices, onlineCount } from '$lib/stores/devices.svelte';
	import { updateProgress, completeScan, updateMonitorStatus, isScanning } from '$lib/stores/scan.svelte';
	import { selectedDeviceId } from '$lib/stores/devices.svelte';
	import { setAlerts, addAlert, unreadCount } from '$lib/stores/alerts.svelte';
	import { setSettings, setInterfaces } from '$lib/stores/settings.svelte';
	import { errorStore, type AppError } from '$lib/stores/error.svelte';
	import Toast from '$lib/components/ui/Toast.svelte';
	import type { UnlistenFn } from '@tauri-apps/api/event';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let unlisteners: UnlistenFn[] = [];
	let currentPath = $derived($page.url.pathname);

	const navItems = [
		{ path: '/', label: 'Topology', icon: '◎' },
		{ path: '/devices', label: 'Devices', icon: '☰' },
		{ path: '/alerts', label: 'Alerts', icon: '⚡' },
		{ path: '/settings', label: 'Settings', icon: '⚙' }
	];

	onMount(async () => {
		try {
			// Load initial data in parallel
			const [deviceList, alertList, settingsData, interfaceList] = await Promise.all([
				getDevices().catch(() => []),
				getAlerts().catch(() => []),
				getSettings().catch(() => null),
				getInterfaces().catch(() => [])
			]);

			setDevices(deviceList);
			setAlerts(alertList);
			if (settingsData) setSettings(settingsData);
			setInterfaces(interfaceList);

			// Subscribe to real-time events
			unlisteners = await subscribeAll({
				onScanProgress: (progress) => updateProgress(progress),
				onDeviceDiscovered: (device) => upsertDevice(device),
				onScanCompleted: () => completeScan(),
				onScanError: (error) => {
					console.error('Scan error:', error);
					// Show error to user via toast
					const scanError: AppError = {
						code: 'SCAN_FAILED',
						message: error.message,
						timestamp: new Date().toISOString()
					};
					errorStore.addError(scanError);
					completeScan();
				},
				onDeviceUpdated: (device) => upsertDevice(device),
				onDeviceDeparted: ({ deviceId }) => markDeparted(deviceId),
				onAlertNew: (alert) => addAlert(alert),
				onMonitorStatus: ({ isRunning, nextScanIn }) =>
					updateMonitorStatus(isRunning, nextScanIn)
			});
		} catch (e) {
			console.error('Failed to initialize app:', e);
		}
	});

	function handleKeydown(e: KeyboardEvent) {
		// Cmd+R or Ctrl+R: Quick scan (prevent browser refresh)
		if ((e.metaKey || e.ctrlKey) && e.key === 'r') {
			e.preventDefault();
			if (!$isScanning) {
				isScanning.set(true);
				startScan({ interfaceId: 'auto', scanType: 'quick', portRange: 'top100' })
					.catch((err) => console.error('Scan failed:', err))
					.finally(() => completeScan());
			}
		}
		// Escape: Deselect device
		if (e.key === 'Escape') {
			selectedDeviceId.set(null);
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
		unsubscribeAll(unlisteners);
	});
</script>

<div class="flex h-screen flex-col bg-bg-primary text-text-primary">
	<!-- Global error toast -->
	<Toast />

	<!-- Top navigation bar -->
	<header class="flex h-12 items-center justify-between border-b border-border bg-bg-secondary px-4">
		<div class="flex items-center gap-4">
			<h1 class="text-lg font-bold tracking-tight">
				<span class="text-accent">◉</span> Echolocate
			</h1>

			<nav class="flex gap-1">
				{#each navItems as item}
					<a
						href={item.path}
						class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm transition-colors
							{currentPath === item.path
								? 'bg-bg-tertiary text-text-primary'
								: 'text-text-secondary hover:bg-bg-tertiary/50 hover:text-text-primary'}"
					>
						<span class="text-xs">{item.icon}</span>
						{item.label}
						{#if item.path === '/alerts' && $unreadCount > 0}
							<span class="ml-1 inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-danger px-1 text-[10px] font-bold text-white">
								{$unreadCount}
							</span>
						{/if}
					</a>
				{/each}
			</nav>
		</div>

		<div class="flex items-center gap-3 text-xs text-text-muted">
			{#if $isScanning}
				<span class="flex items-center gap-1.5">
					<span class="h-2 w-2 animate-pulse rounded-full bg-accent"></span>
					Scanning...
				</span>
			{/if}
			<span>{$onlineCount} device{$onlineCount !== 1 ? 's' : ''} online</span>
		</div>
	</header>

	<!-- Main content -->
	<main class="relative flex-1 overflow-hidden">
		{@render children()}
	</main>
</div>
