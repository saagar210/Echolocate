<script lang="ts">
	let { online, isNew = false }: { online: boolean; isNew?: boolean } = $props();
</script>

<span
	class="inline-block h-2.5 w-2.5 rounded-full
		{online ? 'bg-success' : 'bg-text-muted'}
		{isNew ? 'animate-pulse ring-2 ring-warning/50' : ''}"
	title={online ? 'Online' : 'Offline'}
></span>
