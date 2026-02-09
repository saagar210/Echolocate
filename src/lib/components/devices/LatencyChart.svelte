<script lang="ts">
  import type { LatencyPoint } from '$lib/types/network';
  import { getLatencyHistory } from '$lib/services/tauri-bridge';

  let { deviceId }: { deviceId: string } = $props();

  let points = $state<LatencyPoint[]>([]);
  let loading = $state(true);

  $effect(() => {
    loadData(deviceId);
  });

  async function loadData(id: string) {
    loading = true;
    try {
      points = await getLatencyHistory(id, 24);
    } catch {
      points = [];
    }
    loading = false;
  }

  let stats = $derived(() => {
    if (points.length === 0) return null;
    const values = points.map(p => p.latencyMs);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const avg = values.reduce((a, b) => a + b, 0) / values.length;
    return { min, max, avg };
  });

  let svgPath = $derived(() => {
    if (points.length < 2) return '';

    const width = 280;
    const height = 60;
    const padding = 4;

    const values = points.map(p => p.latencyMs);
    const minVal = Math.min(...values);
    const maxVal = Math.max(...values);
    const range = maxVal - minVal || 1;

    const xStep = (width - padding * 2) / (values.length - 1);

    return values
      .map((v, i) => {
        const x = padding + i * xStep;
        const y = height - padding - ((v - minVal) / range) * (height - padding * 2);
        return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
      })
      .join(' ');
  });
</script>

<div class="mt-3">
  <h4 class="text-xs font-medium uppercase tracking-wider text-text-secondary mb-2">Latency (24h)</h4>

  {#if loading}
    <div class="h-16 animate-pulse rounded bg-bg-secondary"></div>
  {:else if points.length < 2}
    <p class="text-xs text-text-secondary">Not enough data yet</p>
  {:else}
    <svg viewBox="0 0 280 60" class="w-full h-16 rounded bg-bg-secondary">
      <path d={svgPath()} fill="none" stroke="var(--color-accent, #60a5fa)" stroke-width="1.5" />
    </svg>

    {#if stats()}
      <div class="flex justify-between text-xs text-text-secondary mt-1">
        <span>Min: {stats()!.min.toFixed(1)}ms</span>
        <span>Avg: {stats()!.avg.toFixed(1)}ms</span>
        <span>Max: {stats()!.max.toFixed(1)}ms</span>
      </div>
    {/if}
  {/if}
</div>
