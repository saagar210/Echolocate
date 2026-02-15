<script lang="ts">
  import { errorStore } from '../../stores/error.svelte.ts';

  const iconMap: Record<string, string> = {
    INVALID_INPUT: '⚠️',
    NETWORK_ERROR: '🌐',
    DATABASE_ERROR: '💾',
    SCAN_FAILED: '❌',
    COMMAND_NOT_FOUND: '🔍',
    PARSE_ERROR: '📝',
    PERMISSION_DENIED: '🔐',
    TIMEOUT: '⏱️',
    INTERNAL_ERROR: '⚡',
    FILE_NOT_FOUND: '📁',
    IO_ERROR: '💿',
  };

  const colorMap: Record<string, string> = {
    INVALID_INPUT: 'bg-yellow-600',
    NETWORK_ERROR: 'bg-red-600',
    DATABASE_ERROR: 'bg-orange-600',
    SCAN_FAILED: 'bg-red-700',
    COMMAND_NOT_FOUND: 'bg-yellow-700',
    PARSE_ERROR: 'bg-orange-600',
    PERMISSION_DENIED: 'bg-red-700',
    TIMEOUT: 'bg-yellow-600',
    INTERNAL_ERROR: 'bg-red-800',
    FILE_NOT_FOUND: 'bg-yellow-600',
    IO_ERROR: 'bg-red-600',
  };
</script>

{#if $errorStore.is_visible && $errorStore.current_error}
  <div
    class="fixed bottom-4 right-4 {colorMap[$errorStore.current_error.code] ||
      'bg-red-600'} text-white p-4 rounded-lg shadow-2xl max-w-md animate-fade-in z-50"
  >
    <div class="flex items-start gap-3">
      <span class="text-xl flex-shrink-0">
        {iconMap[$errorStore.current_error.code] || '❌'}
      </span>
      <div class="flex-1">
        <div class="font-bold text-sm uppercase tracking-wide">
          {$errorStore.current_error.code}
        </div>
        <div class="text-sm mt-1">{$errorStore.current_error.message}</div>
        {#if $errorStore.current_error.details}
          <details class="mt-2 text-xs opacity-90">
            <summary class="cursor-pointer hover:underline">Details</summary>
            <p class="mt-1 font-mono text-xs bg-black bg-opacity-20 p-2 rounded">
              {$errorStore.current_error.details}
            </p>
          </details>
        {/if}
        <button
          on:click={() => errorStore.clearError()}
          class="mt-2 text-xs underline hover:no-underline opacity-80 hover:opacity-100"
        >
          Dismiss
        </button>
      </div>
      <button
        on:click={() => errorStore.clearError()}
        class="text-xl flex-shrink-0 hover:opacity-80 leading-none"
      >
        ✕
      </button>
    </div>
  </div>
{/if}

<style>
  :global(.animate-fade-in) {
    animation: fade-in 0.3s ease-out;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
