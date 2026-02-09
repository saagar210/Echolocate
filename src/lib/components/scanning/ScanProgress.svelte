<script lang="ts">
	import type { ScanProgress as ScanProgressType } from '$lib/types/scan';

	let { progress }: { progress: ScanProgressType } = $props();

	function phaseLabel(phase: string): string {
		switch (phase) {
			case 'discovery': return 'Discovering devices';
			case 'ping': return 'Measuring latency';
			case 'enriching': return 'Enriching data';
			case 'port_scan': return 'Scanning ports';
			case 'completed': return 'Complete';
			default: return phase;
		}
	}
</script>

<div class="bg-bg-secondary/80 backdrop-blur-sm px-4 py-2">
	<div class="flex items-center justify-between text-xs text-text-secondary mb-1">
		<span>{phaseLabel(progress.phase)}</span>
		<span>{progress.devicesFound} device{progress.devicesFound !== 1 ? 's' : ''} found</span>
	</div>
	<div class="h-1 w-full rounded-full bg-bg-tertiary overflow-hidden">
		<div
			class="h-full rounded-full bg-accent transition-all duration-300"
			style="width: {progress.percentComplete}%"
		></div>
	</div>
</div>
