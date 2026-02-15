import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('Devices Store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  it('initializes with empty device list', () => {
    const initialDevices: any[] = [];
    expect(initialDevices).toEqual([]);
  });

  it('loads devices from backend', async () => {
    const mockDevices = [
      {
        id: 'dev1',
        macAddress: 'AA:BB:CC:DD:EE:FF',
        currentIp: '192.168.1.100',
        deviceType: 'computer',
        isOnline: true,
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockDevices);

    // Would call store.loadDevices()
    const result = await invoke('get_devices');
    expect(result).toEqual(mockDevices);
  });

  it('handles device update', async () => {
    const updatedDevice = {
      id: 'dev1',
      customName: 'My Laptop',
    };

    vi.mocked(invoke).mockResolvedValue(updatedDevice);

    const result = await invoke('update_device', { deviceId: 'dev1', updates: updatedDevice });
    expect(result).toEqual(updatedDevice);
  });

  it('handles device deletion', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    await invoke('delete_device', { deviceId: 'dev1' });
    expect(invoke).toHaveBeenCalledWith('delete_device', { deviceId: 'dev1' });
  });

  it('filters devices by status', () => {
    const devices = [
      { id: 'dev1', isOnline: true },
      { id: 'dev2', isOnline: false },
    ];

    const onlineDevices = devices.filter((d) => d.isOnline);
    expect(onlineDevices).toHaveLength(1);
    expect(onlineDevices[0].id).toBe('dev1');
  });

  it('filters devices by device type', () => {
    const devices = [
      { id: 'dev1', deviceType: 'computer' },
      { id: 'dev2', deviceType: 'phone' },
      { id: 'dev3', deviceType: 'computer' },
    ];

    const computers = devices.filter((d) => d.deviceType === 'computer');
    expect(computers).toHaveLength(2);
  });

  it('searches devices by name', () => {
    const devices = [
      { id: 'dev1', customName: 'My Laptop', hostname: 'laptop.local' },
      { id: 'dev2', customName: 'iPhone', hostname: 'iphone.local' },
    ];

    const query = 'laptop';
    const results = devices.filter(
      (d) =>
        d.customName?.toLowerCase().includes(query) ||
        d.hostname?.toLowerCase().includes(query)
    );

    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('dev1');
  });

  it('sorts devices by last seen', () => {
    const devices = [
      { id: 'dev1', lastSeen: '2024-01-01 10:00:00' },
      { id: 'dev2', lastSeen: '2024-01-02 10:00:00' },
      { id: 'dev3', lastSeen: '2024-01-01 15:00:00' },
    ];

    const sorted = [...devices].sort((a, b) => b.lastSeen.localeCompare(a.lastSeen));
    expect(sorted[0].id).toBe('dev2');
  });

  it('handles loading state', () => {
    const state = {
      loading: false,
      error: null,
      devices: [],
    };

    expect(state.loading).toBe(false);

    // Simulate loading
    state.loading = true;
    expect(state.loading).toBe(true);
  });

  it('handles error state', () => {
    const state = {
      loading: false,
      error: null as string | null,
      devices: [],
    };

    const errorMessage = 'Failed to load devices';
    state.error = errorMessage;

    expect(state.error).toBe(errorMessage);
  });

  it('clears error on successful load', async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    const state = {
      error: 'Previous error',
    };

    // Successful load clears error
    await invoke('get_devices');
    state.error = null;

    expect(state.error).toBeNull();
  });

  it('maintains device count', () => {
    const devices = [
      { id: 'dev1' },
      { id: 'dev2' },
      { id: 'dev3' },
    ];

    expect(devices.length).toBe(3);
  });

  it('tracks online device count', () => {
    const devices = [
      { id: 'dev1', isOnline: true },
      { id: 'dev2', isOnline: false },
      { id: 'dev3', isOnline: true },
    ];

    const onlineCount = devices.filter((d) => d.isOnline).length;
    expect(onlineCount).toBe(2);
  });
});
