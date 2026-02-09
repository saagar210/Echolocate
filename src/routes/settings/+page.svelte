<script lang="ts">
	import { settings, interfaces, setSettings } from '$lib/stores/settings.svelte';
	import { updateSettings } from '$lib/services/tauri-bridge';

	let localSettings = $derived({ ...$settings });
	let saving = $state(false);

	async function handleSave() {
		saving = true;
		try {
			await updateSettings(localSettings);
			setSettings(localSettings);
		} catch (e) {
			console.error('Failed to save settings:', e);
		}
		saving = false;
	}
</script>

<div class="mx-auto max-w-2xl p-6">
	<h2 class="mb-6 text-xl font-bold text-text-primary">Settings</h2>

	<!-- Network -->
	<section class="mb-8">
		<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-text-muted">Network</h3>
		<div class="space-y-4">
			<div>
				<label class="mb-1 block text-sm text-text-secondary" for="interface">Default Interface</label>
				<select
					id="interface"
					bind:value={localSettings.defaultInterfaceId}
					class="w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary"
				>
					<option value={null}>Auto-detect</option>
					{#each $interfaces.filter(i => i.isActive) as iface}
						<option value={iface.id}>{iface.name} ({iface.ipAddress})</option>
					{/each}
				</select>
			</div>

			<div>
				<label class="mb-1 block text-sm text-text-secondary" for="interval">Scan Interval (seconds)</label>
				<input
					id="interval"
					type="number"
					bind:value={localSettings.scanIntervalSecs}
					min="10"
					max="600"
					class="w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary"
				/>
			</div>

			<div>
				<label class="mb-1 block text-sm text-text-secondary" for="ports">Port Range</label>
				<select
					id="ports"
					bind:value={localSettings.portRange}
					class="w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary"
				>
					<option value="top100">Top 100 ports</option>
					<option value="top1000">Top 1000 ports</option>
				</select>
			</div>
		</div>
	</section>

	<!-- Graph -->
	<section class="mb-8">
		<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-text-muted">Graph Physics</h3>
		<div class="space-y-4">
			<div>
				<label class="mb-1 flex justify-between text-sm text-text-secondary" for="repulsion">
					<span>Repulsion</span>
					<span class="text-text-muted">{localSettings.graphRepulsion}</span>
				</label>
				<input
					id="repulsion"
					type="range"
					bind:value={localSettings.graphRepulsion}
					min="50"
					max="800"
					step="10"
					class="w-full accent-accent"
				/>
			</div>

			<div>
				<label class="mb-1 flex justify-between text-sm text-text-secondary" for="link-distance">
					<span>Link Distance</span>
					<span class="text-text-muted">{localSettings.graphLinkDistance}</span>
				</label>
				<input
					id="link-distance"
					type="range"
					bind:value={localSettings.graphLinkDistance}
					min="30"
					max="300"
					step="10"
					class="w-full accent-accent"
				/>
			</div>
		</div>
	</section>

	<button
		onclick={handleSave}
		disabled={saving}
		class="rounded-lg bg-accent px-6 py-2 text-sm font-medium text-bg-primary hover:bg-accent-hover disabled:opacity-50"
	>
		{saving ? 'Saving...' : 'Save Settings'}
	</button>
</div>
