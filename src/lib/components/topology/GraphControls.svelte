<script lang="ts">
	let {
		onZoomIn,
		onZoomOut,
		onFitToScreen
	}: {
		onZoomIn: () => void;
		onZoomOut: () => void;
		onFitToScreen: () => void;
	} = $props();
</script>

<div class="flex flex-col gap-1 rounded-lg border border-border bg-bg-secondary p-1">
	<button
		onclick={onZoomIn}
		class="rounded px-2 py-1 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary"
		title="Zoom in"
	>
		+
	</button>
	<button
		onclick={onZoomOut}
		class="rounded px-2 py-1 text-sm text-text-secondary hover:bg-bg-tertiary hover:text-text-primary"
		title="Zoom out"
	>
		−
	</button>
	<div class="border-t border-border"></div>
	<button
		onclick={onFitToScreen}
		class="rounded px-2 py-1 text-xs text-text-secondary hover:bg-bg-tertiary hover:text-text-primary"
		title="Fit to screen"
	>
		⊡
	</button>
</div>
