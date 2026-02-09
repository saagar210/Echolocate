<script lang="ts">
	let { value = '', placeholder = 'Search...', onInput }: {
		value?: string;
		placeholder?: string;
		onInput: (value: string) => void;
	} = $props();
</script>

<div class="relative">
	<input
		type="text"
		{value}
		{placeholder}
		oninput={(e) => onInput((e.target as HTMLInputElement).value)}
		class="w-full rounded-lg border border-border bg-bg-primary px-3 py-2 pl-8 text-sm text-text-primary
			placeholder:text-text-muted outline-none focus:border-accent"
	/>
	<span class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted text-xs">⌕</span>
</div>
