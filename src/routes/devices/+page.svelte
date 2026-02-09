<script lang="ts">
	import DeviceList from '$lib/components/devices/DeviceList.svelte';
	import DeviceDetail from '$lib/components/devices/DeviceDetail.svelte';
	import ScanControls from '$lib/components/scanning/ScanControls.svelte';
	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import { devices, selectedDeviceId, selectedDevice } from '$lib/stores/devices.svelte';

	let searchQuery = $state('');
</script>

<div class="flex h-full">
	<div class="flex flex-1 flex-col">
		<!-- Toolbar -->
		<div class="flex items-center gap-4 border-b border-border px-4 py-3">
			<ScanControls />
			<div class="flex-1"></div>
			<div class="w-64">
				<SearchBar
					value={searchQuery}
					placeholder="Search devices..."
					onInput={(v) => searchQuery = v}
				/>
			</div>
		</div>

		<!-- Device table -->
		{#if $devices.length === 0}
			<div class="flex flex-1 flex-col items-center justify-center text-text-muted">
				<p class="mb-2 text-lg">No devices discovered yet</p>
				<p class="text-sm">Run a scan to discover devices on your network</p>
			</div>
		{:else}
			<div class="flex-1 overflow-auto">
				<DeviceList
					devices={$devices}
					selectedId={$selectedDeviceId}
					onSelectDevice={(id) => selectedDeviceId.set(id)}
					{searchQuery}
				/>
			</div>
		{/if}
	</div>

	<!-- Device detail sidebar -->
	{#if $selectedDevice}
		<div class="w-80 border-l border-border bg-bg-secondary overflow-y-auto">
			<DeviceDetail
				device={$selectedDevice}
				onClose={() => selectedDeviceId.set(null)}
			/>
		</div>
	{/if}
</div>
