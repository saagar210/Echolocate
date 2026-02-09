<script lang="ts">
	import type { PortInfo } from '$lib/types/device';

	let { ports }: { ports: PortInfo[] } = $props();
</script>

<div class="overflow-hidden rounded border border-border">
	<table class="w-full text-xs">
		<thead>
			<tr class="bg-bg-tertiary/50 text-text-muted">
				<th class="px-2 py-1 text-left font-medium">Port</th>
				<th class="px-2 py-1 text-left font-medium">Service</th>
				<th class="px-2 py-1 text-left font-medium">State</th>
			</tr>
		</thead>
		<tbody>
			{#each ports as port}
				<tr class="border-t border-border/50">
					<td class="px-2 py-1 font-mono text-text-primary">{port.port}/{port.protocol}</td>
					<td class="px-2 py-1 text-text-secondary">{port.serviceName ?? '—'}</td>
					<td class="px-2 py-1">
						<span class="text-success">{port.state}</span>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
