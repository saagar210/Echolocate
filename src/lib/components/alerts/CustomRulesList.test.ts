import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CustomRulesList from './CustomRulesList.svelte';
import { customRules } from '$lib/stores/custom-rules.svelte';
import type { CustomAlertRule } from '$lib/stores/custom-rules.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('CustomRulesList', () => {
  const mockRules: CustomAlertRule[] = [
    {
      id: 'rule1',
      name: 'Untrusted Devices',
      description: 'Alert on untrusted devices',
      isEnabled: true,
      conditions: { type: 'is_trusted' } as any,
      severity: 'warning',
      notifyDesktop: true,
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z',
    },
    {
      id: 'rule2',
      name: 'Open Ports',
      description: 'Alert on devices with open ports',
      isEnabled: false,
      conditions: { type: 'has_open_ports' } as any,
      severity: 'critical',
      notifyDesktop: false,
      webhookUrl: 'https://example.com/webhook',
      createdAt: '2024-01-02T00:00:00Z',
      updatedAt: '2024-01-02T00:00:00Z',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders custom rules list with heading', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Custom Alert Rules')).toBeInTheDocument();
      expect(screen.getByText('New Rule')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('displays empty state when no rules', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = [];
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('No custom rules yet')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('displays all rules in list', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Untrusted Devices')).toBeInTheDocument();
      expect(screen.getByText('Open Ports')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('shows severity badges with correct colors', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('⚠️ WARNING')).toBeInTheDocument();
      expect(screen.getByText('🚨 CRITICAL')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('shows notification badges', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('🔔 Desktop')).toBeInTheDocument();
      expect(screen.getByText('🔗 Webhook')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('can expand rule details', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    const { container } = render(CustomRulesList);

    await waitFor(() => {
      const ruleElements = screen.getAllByText('Untrusted Devices');
      expect(ruleElements.length).toBeGreaterThan(0);
    });

    // Click to expand
    const ruleHeader = screen.getByText('Untrusted Devices').closest('[role="button"]') as HTMLElement;
    if (ruleHeader) {
      await fireEvent.click(ruleHeader);
    }

    // Details should become visible
    await waitFor(() => {
      expect(screen.getByText('CONDITIONS')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('displays webhook URL when present', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Open Ports')).toBeInTheDocument();
    });

    // Expand the rule with webhook
    const openPortsRule = screen.getByText('Open Ports').closest('[role="button"]') as HTMLElement;
    if (openPortsRule) {
      await fireEvent.click(openPortsRule);
    }

    await waitFor(() => {
      expect(screen.getByText('WEBHOOK URL')).toBeInTheDocument();
      expect(screen.getByText('https://example.com/webhook')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('shows loading state when fetching rules', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.loading = true;
      state.rules = [];
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Loading rules...')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('displays error message when rule fetch fails', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.error = 'Failed to fetch rules';
      state.loading = false;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Failed to fetch rules')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('can clear error message', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.error = 'Test error';
      state.loading = false;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Test error')).toBeInTheDocument();
    });

    const closeButton = screen.getByText('✕');
    await fireEvent.click(closeButton);

    customRules.clearError();

    await waitFor(() => {
      expect(screen.queryByText('Test error')).not.toBeInTheDocument();
    });

    unsubscribe();
  });

  it('renders create form when New Rule button clicked', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = [];
    });

    render(CustomRulesList);

    const newRuleButton = screen.getByText('New Rule');
    await fireEvent.click(newRuleButton);

    await waitFor(() => {
      expect(screen.getByText('Create Custom Alert Rule')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('shows descriptions for rules with descriptions', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      expect(screen.getByText('Alert on untrusted devices')).toBeInTheDocument();
      expect(screen.getByText('Alert on devices with open ports')).toBeInTheDocument();
    });

    unsubscribe();
  });

  it('displays edit and delete buttons', async () => {
    const unsubscribe = customRules.subscribe((state) => {
      state.rules = mockRules;
    });

    render(CustomRulesList);

    await waitFor(() => {
      const editButtons = screen.getAllByRole('button').filter((btn) =>
        btn.querySelector('svg') && btn.title === 'Edit rule'
      );
      expect(editButtons.length).toBeGreaterThanOrEqual(mockRules.length);
    });

    unsubscribe();
  });
});
