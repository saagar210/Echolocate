<script lang="ts">
	import TopologyGraph from '$lib/components/topology/TopologyGraph.svelte';
	import GraphControls from '$lib/components/topology/GraphControls.svelte';
	import ScanControls from '$lib/components/scanning/ScanControls.svelte';
	import ScanProgress from '$lib/components/scanning/ScanProgress.svelte';
	import DeviceDetail from '$lib/components/devices/DeviceDetail.svelte';
	import { devices, selectedDeviceId, selectedDevice } from '$lib/stores/devices.svelte';
	import { isScanning, scanProgress } from '$lib/stores/scan.svelte';

	let graphComponent: TopologyGraph | undefined = $state();
</script>

<div class="flex h-full">
	<!-- Main graph area -->
	<div class="relative flex-1">
		<!-- Scan controls overlay -->
		<div class="absolute left-4 top-4 z-10">
			<ScanControls />
		</div>

		<!-- Scan progress bar -->
		{#if $isScanning && $scanProgress}
			<div class="absolute left-0 right-0 top-0 z-10">
				<ScanProgress progress={$scanProgress} />
			</div>
		{/if}

		<!-- Graph controls -->
		<div class="absolute right-4 top-4 z-10">
			<GraphControls
				onZoomIn={() => graphComponent?.zoomIn()}
				onZoomOut={() => graphComponent?.zoomOut()}
				onFitToScreen={() => graphComponent?.fitToScreen()}
			/>
		</div>

		{#if $devices.length === 0}
			<div class="flex h-full flex-col items-center justify-center text-text-muted">
				<span class="mb-4 text-6xl opacity-20">◎</span>
				<p class="mb-2 text-lg">No devices discovered yet</p>
				<p class="text-sm">Run a scan to discover devices on your network</p>
			</div>
		{:else}
			<TopologyGraph
				bind:this={graphComponent}
				devices={$devices}
				selectedId={$selectedDeviceId}
				onSelectDevice={(id) => selectedDeviceId.set(id)}
			/>
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
