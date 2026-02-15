<script lang="ts">
  import type { Condition, ConditionGroup, ConditionLogic } from '$lib/stores/custom-rules.svelte';
  import { ChevronDown, Trash2, Plus } from 'lucide-svelte';

  export let condition: ConditionGroup;
  export let onUpdate: (condition: ConditionGroup) => void;
  export let onDelete: () => void;
  export let depth = 0;

  const maxDepth = 5;
  const indentClass = `ml-${Math.min(depth * 2, 10)}`;

  function isLogical(cond: ConditionGroup): cond is ConditionLogic {
    return 'operator' in cond;
  }

  function handleConditionTypeChange(type: string) {
    const newCondition: Condition = createConditionByType(type);
    onUpdate(newCondition);
  }

  function createConditionByType(type: string): Condition {
    switch (type) {
      case 'is_online':
        return { type: 'is_online' };
      case 'is_trusted':
        return { type: 'is_trusted' };
      case 'is_gateway':
        return { type: 'is_gateway' };
      case 'ip_matches':
        return { type: 'ip_matches', pattern: '192.168.1.0/24' };
      case 'mac_matches':
        return { type: 'mac_matches', pattern: 'AA:BB:CC:DD:EE:*' };
      case 'vendor_contains':
        return { type: 'vendor_contains', text: '' };
      case 'hostname_contains':
        return { type: 'hostname_contains', text: '' };
      case 'has_open_ports':
        return { type: 'has_open_ports' };
      case 'port_open':
        return { type: 'port_open', port: 80 };
      case 'os_unknown':
        return { type: 'os_unknown' };
      case 'low_os_confidence':
        return { type: 'low_os_confidence', threshold: 0.5 };
      case 'high_latency':
        return { type: 'high_latency', ms: 100 };
      case 'custom_property':
        return { type: 'custom_property', key: '', value: '' };
      default:
        return { type: 'is_online' };
    }
  }

  function addCondition(operator: 'AND' | 'OR') {
    if (isLogical(condition)) {
      const logic = condition as ConditionLogic;
      if (logic.operator === operator) {
        logic.conditions.push({ type: 'is_online' });
        onUpdate(condition);
      } else {
        // Convert single condition to logical
        const wrapped: ConditionLogic = {
          operator,
          conditions: [condition, { type: 'is_online' }],
        };
        onUpdate(wrapped);
      }
    } else {
      const wrapped: ConditionLogic = {
        operator,
        conditions: [condition, { type: 'is_online' }],
      };
      onUpdate(wrapped);
    }
  }

  function handleLogicalChange(operator: string) {
    if (isLogical(condition)) {
      const logic = condition as ConditionLogic;
      logic.operator = operator as 'AND' | 'OR' | 'NOT';
      onUpdate(condition);
    }
  }

  function handlePropertyUpdate(key: string, value: any) {
    if (!isLogical(condition)) {
      const c = condition as any;
      c[key] = value;
      onUpdate(condition);
    }
  }
</script>

<div class="rounded-lg border border-gray-300 bg-gray-50 p-3 {indentClass}">
  {#if isLogical(condition)}
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <select
          value={condition.operator}
          on:change={(e) => handleLogicalChange(e.currentTarget.value)}
          class="rounded border border-gray-300 bg-white px-2 py-1 text-sm font-semibold"
        >
          <option value="AND">AND</option>
          <option value="OR">OR</option>
          <option value="NOT">NOT</option>
        </select>
        <button
          on:click={onDelete}
          class="rounded p-1 text-red-600 hover:bg-red-50"
          title="Delete condition group"
        >
          <Trash2 size={16} />
        </button>
      </div>

      {#if condition.operator === 'NOT'}
        <div class="space-y-2">
          <svelte:self
            condition={condition.condition}
            onUpdate={(cond) => {
              condition.condition = cond;
              onUpdate(condition);
            }}
            onDelete={onDelete}
            depth={depth + 1}
          />
        </div>
      {:else}
        <div class="space-y-2">
          {#each condition.conditions as cond, idx (idx)}
            <svelte:self
              condition={cond}
              onUpdate={(updated) => {
                condition.conditions[idx] = updated;
                onUpdate(condition);
              }}
              onDelete={() => {
                condition.conditions.splice(idx, 1);
                onUpdate(condition);
              }}
              depth={depth + 1}
            />
          {/each}
        </div>

        {#if depth < maxDepth}
          <div class="flex gap-2">
            <button
              on:click={() => addCondition('AND')}
              class="flex items-center gap-1 rounded bg-blue-100 px-2 py-1 text-xs text-blue-700 hover:bg-blue-200"
            >
              <Plus size={14} /> AND
            </button>
            <button
              on:click={() => addCondition('OR')}
              class="flex items-center gap-1 rounded bg-green-100 px-2 py-1 text-xs text-green-700 hover:bg-green-200"
            >
              <Plus size={14} /> OR
            </button>
          </div>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="space-y-3">
      <div class="flex items-center justify-between gap-2">
        <select
          value={condition.type}
          on:change={(e) => handleConditionTypeChange(e.currentTarget.value)}
          class="flex-1 rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        >
          <option value="is_online">Is Online</option>
          <option value="is_trusted">Is Trusted</option>
          <option value="is_gateway">Is Gateway</option>
          <option value="ip_matches">IP Matches (CIDR)</option>
          <option value="mac_matches">MAC Matches (pattern)</option>
          <option value="vendor_contains">Vendor Contains</option>
          <option value="hostname_contains">Hostname Contains</option>
          <option value="has_open_ports">Has Open Ports</option>
          <option value="port_open">Specific Port Open</option>
          <option value="os_unknown">OS Unknown</option>
          <option value="low_os_confidence">Low OS Confidence</option>
          <option value="high_latency">High Latency</option>
          <option value="custom_property">Custom Property</option>
        </select>

        <button
          on:click={onDelete}
          class="rounded p-1 text-red-600 hover:bg-red-50"
          title="Delete condition"
        >
          <Trash2 size={16} />
        </button>
      </div>

      <!-- Condition-specific inputs -->
      {#if condition.type === 'ip_matches'}
        <input
          type="text"
          value={condition.pattern}
          on:change={(e) => handlePropertyUpdate('pattern', e.currentTarget.value)}
          placeholder="192.168.1.0/24 or 192.168.1.1"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'mac_matches'}
        <input
          type="text"
          value={condition.pattern}
          on:change={(e) => handlePropertyUpdate('pattern', e.currentTarget.value)}
          placeholder="AA:BB:CC:DD:EE:* or exact MAC"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'vendor_contains' || condition.type === 'hostname_contains'}
        <input
          type="text"
          value={condition.text}
          on:change={(e) => handlePropertyUpdate('text', e.currentTarget.value)}
          placeholder="Search text (case-insensitive)"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'port_open'}
        <input
          type="number"
          value={condition.port}
          on:change={(e) => handlePropertyUpdate('port', parseInt(e.currentTarget.value))}
          placeholder="Port number"
          min="1"
          max="65535"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'low_os_confidence'}
        <input
          type="number"
          value={condition.threshold}
          on:change={(e) => handlePropertyUpdate('threshold', parseFloat(e.currentTarget.value))}
          placeholder="Threshold (0.0 - 1.0)"
          min="0"
          max="1"
          step="0.1"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'high_latency'}
        <input
          type="number"
          value={condition.ms}
          on:change={(e) => handlePropertyUpdate('ms', parseInt(e.currentTarget.value))}
          placeholder="Milliseconds"
          min="1"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {:else if condition.type === 'custom_property'}
        <input
          type="text"
          value={condition.key}
          on:change={(e) => handlePropertyUpdate('key', e.currentTarget.value)}
          placeholder="Property key"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
        <input
          type="text"
          value={condition.value}
          on:change={(e) => handlePropertyUpdate('value', e.currentTarget.value)}
          placeholder="Property value"
          class="w-full rounded border border-gray-300 bg-white px-2 py-1 text-sm"
        />
      {/if}

      {#if depth < maxDepth}
        <div class="flex gap-2">
          <button
            on:click={() => addCondition('AND')}
            class="flex items-center gap-1 rounded bg-blue-100 px-2 py-1 text-xs text-blue-700 hover:bg-blue-200"
          >
            <Plus size={14} /> AND
          </button>
          <button
            on:click={() => addCondition('OR')}
            class="flex items-center gap-1 rounded bg-green-100 px-2 py-1 text-xs text-green-700 hover:bg-green-200"
          >
            <Plus size={14} /> OR
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global([class*='ml-']) {
    margin-left: var(--ml-amount, 0);
  }
  :global(.ml-2) {
    --ml-amount: 0.5rem;
  }
  :global(.ml-4) {
    --ml-amount: 1rem;
  }
  :global(.ml-6) {
    --ml-amount: 1.5rem;
  }
  :global(.ml-8) {
    --ml-amount: 2rem;
  }
  :global(.ml-10) {
    --ml-amount: 2.5rem;
  }
</style>
