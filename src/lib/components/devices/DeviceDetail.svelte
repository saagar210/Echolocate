<script lang="ts">
	import type { Device } from '$lib/types/device';
	import { updateDevice, deleteDevice, pingDevice } from '$lib/services/tauri-bridge';
	import { upsertDevice, removeDevice } from '$lib/stores/devices.svelte';
	import PortList from './PortList.svelte';
	import StatusBadge from '../ui/StatusBadge.svelte';

	let {
		device,
		onClose
	}: {
		device: Device;
		onClose: () => void;
	} = $props();

	let editingName = $state(false);
	let nameInput = $state(device.customName ?? device.hostname ?? '');
	let pingResult = $state<{ latency: number | null; success: boolean } | null>(null);
	let pinging = $state(false);

	function displayName(d: Device): string {
		return d.customName ?? d.hostname ?? d.vendor ?? d.currentIp ?? 'Unknown';
	}

	async function saveName() {
		try {
			const updated = await updateDevice(device.id, { customName: nameInput || null });
			upsertDevice(updated);
			editingName = false;
		} catch (e) {
			console.error('Failed to update device:', e);
		}
	}

	async function toggleTrust() {
		try {
			const updated = await updateDevice(device.id, { isTrusted: !device.isTrusted });
			upsertDevice(updated);
		} catch (e) {
			console.error('Failed to toggle trust:', e);
		}
	}

	async function handlePing() {
		if (!device.currentIp) return;
		pinging = true;
		try {
			const result = await pingDevice(device.currentIp);
			pingResult = { latency: result.latencyMs, success: result.success };
		} catch (e) {
			pingResult = { latency: null, success: false };
		}
		pinging = false;
	}

	async function handleDelete() {
		try {
			await deleteDevice(device.id);
			removeDevice(device.id);
			onClose();
		} catch (e) {
			console.error('Failed to delete device:', e);
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
</script>

<div class="flex flex-col h-full">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-border p-4">
		<div class="flex items-center gap-2">
			<StatusBadge online={device.isOnline} isNew={false} />
			{#if editingName}
				<input
					bind:value={nameInput}
					onkeydown={(e) => e.key === 'Enter' && saveName()}
					onblur={saveName}
					class="rounded border border-accent bg-bg-primary px-2 py-0.5 text-sm text-text-primary outline-none"
					autofocus
				/>
			{:else}
				<button
					class="text-sm font-semibold text-text-primary hover:text-accent"
					onclick={() => { editingName = true; nameInput = device.customName ?? device.hostname ?? ''; }}
				>
					{displayName(device)}
				</button>
			{/if}
		</div>
		<button
			onclick={onClose}
			class="text-text-muted hover:text-text-primary"
		>
			✕
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto p-4 space-y-4">
		<!-- Trust badge -->
		<div class="flex items-center gap-2">
			<button
				onclick={toggleTrust}
				class="rounded-full px-3 py-1 text-xs font-medium transition-colors
					{device.isTrusted
						? 'bg-success/20 text-success'
						: 'bg-warning/20 text-warning'}"
			>
				{device.isTrusted ? 'Trusted' : 'Untrusted'}
			</button>
			<span class="text-xs text-text-muted capitalize">{device.deviceType}</span>
		</div>

		<!-- Identity section -->
		<section>
			<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-text-muted">Identity</h3>
			<div class="space-y-1.5 text-sm">
				{#if device.currentIp}
					<div class="flex justify-between">
						<span class="text-text-secondary">IP</span>
						<span class="font-mono text-text-primary">{device.currentIp}</span>
					</div>
				{/if}
				{#if device.macAddress}
					<div class="flex justify-between">
						<span class="text-text-secondary">MAC</span>
						<span class="font-mono text-text-primary text-xs">{device.macAddress}</span>
					</div>
				{/if}
				{#if device.vendor}
					<div class="flex justify-between">
						<span class="text-text-secondary">Vendor</span>
						<span class="text-text-primary">{device.vendor}</span>
					</div>
				{/if}
				{#if device.hostname}
					<div class="flex justify-between">
						<span class="text-text-secondary">Hostname</span>
						<span class="text-text-primary">{device.hostname}</span>
					</div>
				{/if}
				{#if device.osGuess}
					<div class="flex justify-between">
						<span class="text-text-secondary">OS</span>
						<span class="text-text-primary">
							{device.osGuess}
							<span class="text-text-muted">({Math.round(device.osConfidence * 100)}%)</span>
						</span>
					</div>
				{/if}
			</div>
		</section>

		<!-- Network section -->
		<section>
			<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-text-muted">Network</h3>
			<div class="space-y-1.5 text-sm">
				{#if device.latencyMs !== null}
					<div class="flex justify-between">
						<span class="text-text-secondary">Latency</span>
						<span class="text-text-primary">{device.latencyMs.toFixed(1)} ms</span>
					</div>
				{/if}
				<div class="flex justify-between">
					<span class="text-text-secondary">First seen</span>
					<span class="text-text-primary">{timeAgo(device.firstSeen)}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-text-secondary">Last seen</span>
					<span class="text-text-primary">{timeAgo(device.lastSeen)}</span>
				</div>
			</div>
		</section>

		<!-- Ports section -->
		{#if device.openPorts.length > 0}
			<section>
				<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-text-muted">
					Open Ports ({device.openPorts.length})
				</h3>
				<PortList ports={device.openPorts} />
			</section>
		{/if}

		<!-- Actions section -->
		<section>
			<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-text-muted">Actions</h3>
			<div class="flex flex-wrap gap-2">
				<button
					onclick={handlePing}
					disabled={pinging || !device.currentIp}
					class="rounded border border-border px-3 py-1.5 text-xs text-text-secondary hover:bg-bg-tertiary disabled:opacity-50"
				>
					{pinging ? 'Pinging...' : 'Ping'}
				</button>
				<button
					onclick={handleDelete}
					class="rounded border border-danger/30 px-3 py-1.5 text-xs text-danger hover:bg-danger/10"
				>
					Delete
				</button>
			</div>

			{#if pingResult}
				<div class="mt-2 rounded bg-bg-primary p-2 text-xs font-mono">
					{#if pingResult.success}
						<span class="text-success">Reply: {pingResult.latency?.toFixed(1)} ms</span>
					{:else}
						<span class="text-danger">No response</span>
					{/if}
				</div>
			{/if}
		</section>

		<!-- Notes section -->
		<section>
			<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-text-muted">Notes</h3>
			<p class="text-xs text-text-secondary italic">
				{device.notes ?? 'No notes'}
			</p>
		</section>
	</div>
</div>
