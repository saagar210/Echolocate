<script lang="ts">
  import { onMount } from 'svelte';
  import { customRules } from '$lib/stores/custom-rules.svelte';
  import type { CustomAlertRule } from '$lib/stores/custom-rules.svelte';
  import CustomRuleForm from './CustomRuleForm.svelte';
  import { Edit, Trash2, Plus, ChevronDown, Eye } from 'lucide-svelte';

  let showForm = false;
  let editingRule: CustomAlertRule | null = null;
  let expandedRules = new Set<string>();

  onMount(() => {
    customRules.fetchRules();
  });

  function toggleExpanded(ruleId: string) {
    if (expandedRules.has(ruleId)) {
      expandedRules.delete(ruleId);
    } else {
      expandedRules.add(ruleId);
    }
    expandedRules = expandedRules;
  }

  function startEdit(rule: CustomAlertRule) {
    editingRule = rule;
    showForm = true;
  }

  async function handleDelete(ruleId: string) {
    if (confirm('Delete this rule?')) {
      await customRules.deleteRule(ruleId);
    }
  }

  function handleFormSave(event: CustomEvent<CustomAlertRule>) {
    showForm = false;
    editingRule = null;
  }

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleString();
  }

  function getSeverityColor(severity: string) {
    switch (severity) {
      case 'critical':
        return 'bg-red-100 text-red-800';
      case 'warning':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-blue-100 text-blue-800';
    }
  }

  function getSeverityIcon(severity: string) {
    switch (severity) {
      case 'critical':
        return '🚨';
      case 'warning':
        return '⚠️';
      default:
        return 'ℹ️';
    }
  }
</script>

<div class="space-y-4">
  {#if !showForm}
    <div class="flex items-center justify-between">
      <h2 class="text-2xl font-bold">Custom Alert Rules</h2>
      <button
        on:click={() => {
          editingRule = null;
          showForm = true;
        }}
        class="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
      >
        <Plus size={16} /> New Rule
      </button>
    </div>
  {/if}

  {#if $customRules.error}
    <div class="rounded-lg bg-red-50 p-3 text-sm text-red-700">
      {$customRules.error}
      <button
        on:click={() => customRules.clearError()}
        class="ml-2 text-red-600 hover:text-red-800"
      >
        ✕
      </button>
    </div>
  {/if}

  {#if showForm}
    <CustomRuleForm
      rule={editingRule}
      onSave={() => {
        showForm = false;
        editingRule = null;
        customRules.fetchRules();
      }}
      onCancel={() => {
        showForm = false;
        editingRule = null;
      }}
    />
  {/if}

  {#if $customRules.loading}
    <div class="text-center text-gray-500">Loading rules...</div>
  {:else if $customRules.rules.length === 0}
    <div class="rounded-lg border-2 border-dashed border-gray-300 bg-gray-50 p-8 text-center">
      <p class="text-gray-500">No custom rules yet</p>
      <button
        on:click={() => {
          editingRule = null;
          showForm = true;
        }}
        class="mt-3 text-blue-600 hover:text-blue-800 font-medium"
      >
        Create your first rule
      </button>
    </div>
  {:else}
    <div class="space-y-3">
      {#each $customRules.rules as rule (rule.id)}
        <div class="rounded-lg border border-gray-300 bg-white">
          <!-- Rule Header -->
          <div
            class="flex items-center justify-between gap-4 p-4 cursor-pointer hover:bg-gray-50"
            on:click={() => toggleExpanded(rule.id)}
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <button
                  class="p-0 text-gray-400 hover:text-gray-600"
                  on:click|stopPropagation={() => toggleExpanded(rule.id)}
                >
                  <ChevronDown
                    size={18}
                    class="transition-transform {expandedRules.has(rule.id) ? 'rotate-0' : '-rotate-90'}"
                  />
                </button>
                <label class="flex items-center gap-2 flex-1 min-w-0">
                  <input
                    type="checkbox"
                    checked={rule.isEnabled}
                    on:change={(e) => {
                      customRules.updateRule(rule.id, { isEnabled: e.currentTarget.checked });
                    }}
                    on:click|stopPropagation
                    class="rounded border-gray-300"
                  />
                  <div class="min-w-0">
                    <h3 class="font-semibold text-gray-900 truncate">{rule.name}</h3>
                    {#if rule.description}
                      <p class="text-xs text-gray-500 truncate">{rule.description}</p>
                    {/if}
                  </div>
                </label>
              </div>
            </div>

            <!-- Badges -->
            <div class="flex items-center gap-2">
              <span class="px-2 py-1 rounded text-xs font-medium {getSeverityColor(rule.severity)}">
                {getSeverityIcon(rule.severity)} {rule.severity.toUpperCase()}
              </span>
              {#if rule.notifyDesktop}
                <span class="px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800">
                  🔔 Desktop
                </span>
              {/if}
              {#if rule.webhookUrl}
                <span class="px-2 py-1 rounded text-xs font-medium bg-purple-100 text-purple-800">
                  🔗 Webhook
                </span>
              {/if}
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-2" on:click|stopPropagation>
              <button
                on:click={() => startEdit(rule)}
                class="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded"
                title="Edit rule"
              >
                <Edit size={16} />
              </button>
              <button
                on:click={() => handleDelete(rule.id)}
                class="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                title="Delete rule"
              >
                <Trash2 size={16} />
              </button>
            </div>
          </div>

          <!-- Rule Details (Expanded) -->
          {#if expandedRules.has(rule.id)}
            <div class="border-t border-gray-200 bg-gray-50 p-4 space-y-3">
              <!-- Conditions Display -->
              <div>
                <p class="text-xs font-semibold text-gray-600 mb-2">CONDITIONS</p>
                <div class="bg-white p-3 rounded border border-gray-200 font-mono text-xs text-gray-700 max-h-64 overflow-y-auto">
                  <pre>{JSON.stringify(rule.conditions, null, 2)}</pre>
                </div>
              </div>

              <!-- Webhook URL -->
              {#if rule.webhookUrl}
                <div>
                  <p class="text-xs font-semibold text-gray-600 mb-2">WEBHOOK URL</p>
                  <p class="text-xs text-gray-700 break-all font-mono">{rule.webhookUrl}</p>
                </div>
              {/if}

              <!-- Timestamps -->
              <div class="grid grid-cols-2 gap-4 text-xs text-gray-600">
                <div>
                  <p class="font-semibold">Created</p>
                  <p>{formatDate(rule.createdAt)}</p>
                </div>
                <div>
                  <p class="font-semibold">Updated</p>
                  <p>{formatDate(rule.updatedAt)}</p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
