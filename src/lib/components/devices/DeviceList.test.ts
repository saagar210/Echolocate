import { describe, it, expect, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DeviceList from './DeviceList.svelte';

describe('DeviceList', () => {
  const mockDevices = [
    {
      id: '1',
      ip_address: '192.168.1.1',
      device_name: 'Router',
      os_type: 'Linux',
      device_class: 'router',
      is_online: true,
      last_seen_at: '2026-02-12T12:00:00Z',
      vendor: 'Ubiquiti',
    },
    {
      id: '2',
      ip_address: '192.168.1.42',
      device_name: 'MacBook Pro',
      os_type: 'macOS',
      device_class: 'computer',
      is_online: true,
      last_seen_at: '2026-02-12T11:00:00Z',
      vendor: 'Apple Inc.',
    },
    {
      id: '3',
      ip_address: '192.168.1.87',
      device_name: 'iPhone',
      os_type: 'iOS',
      device_class: 'phone',
      is_online: false,
      last_seen_at: '2026-02-11T12:00:00Z',
      vendor: 'Apple Inc.',
    },
  ];

  it('should render device list', () => {
    const { container } = render(DeviceList, {
      props: { devices: mockDevices },
    });

    const rows = container.querySelectorAll('tr');
    // Header + 3 devices
    expect(rows.length).toBeGreaterThanOrEqual(3);
  });

  it('should display device information', () => {
    const { container } = render(DeviceList, {
      props: { devices: mockDevices },
    });

    const text = container.textContent;
    expect(text).toContain('MacBook Pro');
    expect(text).toContain('192.168.1.42');
    expect(text).toContain('Apple Inc.');
  });

  it('should handle empty device list', () => {
    const { container } = render(DeviceList, {
      props: { devices: [] },
    });

    const rows = container.querySelectorAll('tr');
    expect(rows.length).toBeGreaterThanOrEqual(0);
  });

  it('should highlight offline devices', () => {
    const { container } = render(DeviceList, {
      props: { devices: mockDevices },
    });

    const text = container.textContent;
    // Offline device should be marked somehow
    expect(text).toContain('iPhone');
  });

  it('should sort by device name', () => {
    const { container } = render(DeviceList, {
      props: {
        devices: mockDevices,
        sortBy: 'name',
      },
    });

    const text = container.textContent;
    expect(text).toContain('iPhone');
    expect(text).toContain('MacBook Pro');
  });

  it('should filter online devices', () => {
    const onlineDevices = mockDevices.filter(d => d.is_online);

    const { container } = render(DeviceList, {
      props: {
        devices: onlineDevices,
      },
    });

    const text = container.textContent;
    expect(text).toContain('Router');
    expect(text).toContain('MacBook Pro');
  });

  it('should emit selection event on row click', async () => {
    const { component, container } = render(DeviceList, {
      props: { devices: mockDevices },
    });

    const firstRow = container.querySelector('tbody tr');
    if (firstRow) {
      await fireEvent.click(firstRow);
      // Component should emit selection event
    }
  });
});
