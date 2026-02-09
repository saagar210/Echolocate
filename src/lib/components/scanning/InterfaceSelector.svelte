<script lang="ts">
	import { interfaces, settings, activeInterface } from '$lib/stores/settings.svelte';
	import { updateSettings } from '$lib/services/tauri-bridge';

	let active = $derived($activeInterface);

	async function handleChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		const newSettings = { ...$settings, defaultInterfaceId: target.value || null };
		settings.set(newSettings);
		try {
			await updateSettings(newSettings);
		} catch (e) {
			console.error('Failed to update settings:', e);
		}
	}
</script>

{#if $interfaces.length > 0}
	<select
		value={active?.id ?? ''}
		onchange={handleChange}
		class="rounded-lg border border-border bg-bg-secondary px-2 py-1.5 text-xs text-text-secondary"
	>
		{#each $interfaces.filter(i => i.isActive) as iface}
			<option value={iface.id}>
				{iface.name} ({iface.ipAddress ?? 'no IP'})
			</option>
		{/each}
	</select>
{/if}
