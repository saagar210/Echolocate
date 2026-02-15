<script lang="ts">
  import { onMount } from 'svelte';
  import { customRules } from '$lib/stores/custom-rules.svelte';
  import type { CustomAlertRule, ConditionGroup } from '$lib/stores/custom-rules.svelte';
  import ConditionBuilder from './ConditionBuilder.svelte';
  import { Save, X } from 'lucide-svelte';

  export let rule: CustomAlertRule | null = null;
  export let onSave: (rule: CustomAlertRule) => void = () => {};
  export let onCancel: () => void = () => {};

  let name = '';
  let description = '';
  let severity = 'info';
  let notifyDesktop = true;
  let webhookUrl = '';
  let conditions: ConditionGroup = { type: 'is_online' };
  let saving = false;
  let error = '';

  onMount(() => {
    if (rule) {
      name = rule.name;
      description = rule.description || '';
      severity = rule.severity;
      notifyDesktop = rule.notifyDesktop;
      webhookUrl = rule.webhookUrl || '';
      conditions = rule.conditions;
    }
  });

  async function handleSubmit() {
    if (!name.trim()) {
      error = 'Rule name is required';
      return;
    }

    saving = true;
    error = '';

    try {
      if (rule) {
        // Update existing rule
        const updated = await customRules.updateRule(rule.id, {
          name,
          description: description || undefined,
          severity,
          notifyDesktop,
          webhookUrl: webhookUrl || undefined,
          conditions,
        });
        onSave(updated);
      } else {
        // Create new rule
        const created = await customRules.createRule(
          name,
          description || undefined,
          conditions,
          severity,
          notifyDesktop,
          webhookUrl || undefined,
        );
        onSave(created);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-6 rounded-lg border border-gray-300 bg-white p-6">
  <h2 class="text-xl font-bold">
    {rule ? 'Edit Rule' : 'Create Custom Alert Rule'}
  </h2>

  {#if error}
    <div class="rounded-lg bg-red-50 p-3 text-sm text-red-700">
      {error}
    </div>
  {/if}

  <div class="space-y-4">
    <!-- Rule Name -->
    <div>
      <label for="name" class="block text-sm font-medium text-gray-700">
        Rule Name <span class="text-red-500">*</span>
      </label>
      <input
        id="name"
        type="text"
        bind:value={name}
        placeholder="e.g., Alert on untrusted devices with open ports"
        class="mt-1 w-full rounded border border-gray-300 px-3 py-2"
      />
    </div>

    <!-- Description -->
    <div>
      <label for="description" class="block text-sm font-medium text-gray-700">
        Description
      </label>
      <textarea
        id="description"
        bind:value={description}
        placeholder="Optional description for this rule"
        rows="2"
        class="mt-1 w-full rounded border border-gray-300 px-3 py-2"
      />
    </div>

    <!-- Conditions -->
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-2">
        Conditions
      </label>
      <ConditionBuilder
        bind:condition={conditions}
        onUpdate={(cond) => (conditions = cond)}
        onDelete={() => (conditions = { type: 'is_online' })}
      />
    </div>

    <!-- Severity -->
    <div>
      <label for="severity" class="block text-sm font-medium text-gray-700">
        Severity
      </label>
      <select
        id="severity"
        bind:value={severity}
        class="mt-1 w-full rounded border border-gray-300 px-3 py-2"
      >
        <option value="info">ℹ️ Info</option>
        <option value="warning">⚠️ Warning</option>
        <option value="critical">🚨 Critical</option>
      </select>
    </div>

    <!-- Notifications -->
    <div>
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          bind:checked={notifyDesktop}
          class="rounded border-gray-300"
        />
        <span class="text-sm font-medium text-gray-700">Send desktop notification</span>
      </label>
    </div>

    <!-- Webhook URL -->
    <div>
      <label for="webhook" class="block text-sm font-medium text-gray-700">
        Webhook URL (optional)
      </label>
      <input
        id="webhook"
        type="url"
        bind:value={webhookUrl}
        placeholder="https://example.com/webhook"
        class="mt-1 w-full rounded border border-gray-300 px-3 py-2 text-sm"
      />
      <p class="mt-1 text-xs text-gray-500">
        POST alerts as JSON to this URL when rule is triggered
      </p>
    </div>
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-3 border-t border-gray-200 pt-4">
    <button
      on:click={onCancel}
      class="rounded border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
    >
      <X size={16} class="inline mr-1" /> Cancel
    </button>
    <button
      on:click={handleSubmit}
      disabled={saving}
      class="flex items-center gap-1 rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
    >
      <Save size={16} />
      {saving ? 'Saving...' : 'Save Rule'}
    </button>
  </div>
</div>
