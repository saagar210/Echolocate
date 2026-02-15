import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface CustomAlertRule {
  id: string;
  name: string;
  description?: string;
  isEnabled: boolean;
  conditions: ConditionGroup;
  severity: string;
  notifyDesktop: boolean;
  webhookUrl?: string;
  createdAt: string;
  updatedAt: string;
}

export type Condition =
  | { type: 'is_online' }
  | { type: 'is_trusted' }
  | { type: 'is_gateway' }
  | { type: 'ip_matches'; pattern: string }
  | { type: 'mac_matches'; pattern: string }
  | { type: 'vendor_contains'; text: string }
  | { type: 'hostname_contains'; text: string }
  | { type: 'has_open_ports' }
  | { type: 'port_open'; port: number }
  | { type: 'os_unknown' }
  | { type: 'low_os_confidence'; threshold: number }
  | { type: 'high_latency'; ms: number }
  | { type: 'custom_property'; key: string; value: string };

export type ConditionLogic =
  | { operator: 'AND'; conditions: ConditionGroup[] }
  | { operator: 'OR'; conditions: ConditionGroup[] }
  | { operator: 'NOT'; condition: ConditionGroup };

export type ConditionGroup = Condition | ConditionLogic;

interface CustomRulesStore {
  rules: CustomAlertRule[];
  loading: boolean;
  error: string | null;
}

function createCustomRulesStore() {
  const { subscribe, set, update } = writable<CustomRulesStore>({
    rules: [],
    loading: false,
    error: null,
  });

  return {
    subscribe,

    async fetchRules() {
      update((state) => ({ ...state, loading: true, error: null }));
      try {
        const rules = await invoke<CustomAlertRule[]>('get_custom_rules');
        update((state) => ({ ...state, rules, loading: false }));
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        update((state) => ({ ...state, error: msg, loading: false }));
      }
    },

    async getRule(ruleId: string) {
      try {
        return await invoke<CustomAlertRule | null>('get_custom_rule', { ruleId });
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        update((state) => ({ ...state, error: msg }));
        return null;
      }
    },

    async createRule(
      name: string,
      description: string | undefined,
      conditions: ConditionGroup,
      severity: string,
      notifyDesktop: boolean,
      webhookUrl: string | undefined,
    ) {
      update((state) => ({ ...state, loading: true, error: null }));
      try {
        const rule = await invoke<CustomAlertRule>('create_custom_rule', {
          req: {
            name,
            description,
            conditions,
            severity,
            notifyDesktop,
            webhookUrl,
          },
        });
        update((state) => ({
          ...state,
          rules: [rule, ...state.rules],
          loading: false,
        }));
        return rule;
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        update((state) => ({ ...state, error: msg, loading: false }));
        throw error;
      }
    },

    async updateRule(
      ruleId: string,
      updates: {
        name?: string;
        description?: string;
        conditions?: ConditionGroup;
        severity?: string;
        notifyDesktop?: boolean;
        webhookUrl?: string;
        isEnabled?: boolean;
      },
    ) {
      update((state) => ({ ...state, loading: true, error: null }));
      try {
        const updated = await invoke<CustomAlertRule>('update_custom_rule', {
          ruleId,
          updates,
        });
        update((state) => ({
          ...state,
          rules: state.rules.map((r) => (r.id === ruleId ? updated : r)),
          loading: false,
        }));
        return updated;
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        update((state) => ({ ...state, error: msg, loading: false }));
        throw error;
      }
    },

    async deleteRule(ruleId: string) {
      update((state) => ({ ...state, loading: true, error: null }));
      try {
        await invoke('delete_custom_rule', { ruleId });
        update((state) => ({
          ...state,
          rules: state.rules.filter((r) => r.id !== ruleId),
          loading: false,
        }));
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        update((state) => ({ ...state, error: msg, loading: false }));
        throw error;
      }
    },

    clearError() {
      update((state) => ({ ...state, error: null }));
    },
  };
}

export const customRules = createCustomRulesStore();
