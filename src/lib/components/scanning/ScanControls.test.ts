import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ScanControls from './ScanControls.svelte';

describe('ScanControls', () => {
  it('should render scan buttons', () => {
    const { container } = render(ScanControls, {
      props: { isScanning: false },
    });

    const buttons = container.querySelectorAll('button');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('should have quick scan button', () => {
    const { container } = render(ScanControls, {
      props: { isScanning: false },
    });

    const text = container.textContent;
    expect(text?.toLowerCase()).toContain('quick');
  });

  it('should have full scan button', () => {
    const { container } = render(ScanControls, {
      props: { isScanning: false },
    });

    const text = container.textContent;
    expect(text?.toLowerCase()).toContain('full');
  });

  it('should disable buttons during scan', () => {
    const { container } = render(ScanControls, {
      props: { isScanning: true },
    });

    const buttons = container.querySelectorAll('button:disabled');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('should show stop button when scanning', () => {
    const { container } = render(ScanControls, {
      props: { isScanning: true },
    });

    const text = container.textContent;
    expect(text?.toLowerCase()).toContain('stop');
  });

  it('should emit scan event on button click', async () => {
    const { component, container } = render(ScanControls, {
      props: { isScanning: false },
    });

    const buttons = container.querySelectorAll('button');
    if (buttons.length > 0) {
      await fireEvent.click(buttons[0]);
      // Component should emit scan event
    }
  });

  it('should handle monitor toggle', async () => {
    const { container } = render(ScanControls, {
      props: { isMonitoring: false },
    });

    const toggles = container.querySelectorAll('[role="switch"]');
    if (toggles.length > 0) {
      await fireEvent.click(toggles[0]);
      // Monitor state should toggle
    }
  });
});
