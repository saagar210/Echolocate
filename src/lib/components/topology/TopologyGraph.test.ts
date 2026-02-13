import { describe, it, expect, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import TopologyGraph from './TopologyGraph.svelte';

describe('TopologyGraph', () => {
  let component: any;

  const mockDevices = [
    {
      id: '1',
      ip_address: '192.168.1.1',
      device_name: 'Router',
      os_type: 'Linux',
      device_class: 'router',
      is_online: true,
      mac_address: 'aa:bb:cc:dd:ee:ff',
    },
    {
      id: '2',
      ip_address: '192.168.1.42',
      device_name: 'MacBook',
      os_type: 'macOS',
      device_class: 'computer',
      is_online: true,
      mac_address: '11:22:33:44:55:66',
    },
  ];

  const mockLinks = [
    {
      source: '1',
      target: '2',
    },
  ];

  beforeEach(() => {
    component = render(TopologyGraph, {
      props: {
        devices: mockDevices,
        links: mockLinks,
      },
    });
  });

  it('should render SVG container', () => {
    const svg = component.container.querySelector('svg');
    expect(svg).toBeTruthy();
  });

  it('should render nodes for each device', () => {
    const circles = component.container.querySelectorAll('circle');
    expect(circles.length).toBeGreaterThanOrEqual(mockDevices.length);
  });

  it('should render links between devices', () => {
    const lines = component.container.querySelectorAll('line');
    expect(lines.length).toBeGreaterThan(0);
  });

  it('should have proper SVG dimensions', () => {
    const svg = component.container.querySelector('svg');
    expect(svg?.getAttribute('width')).toBeTruthy();
    expect(svg?.getAttribute('height')).toBeTruthy();
  });

  it('should update when props change', async () => {
    const newDevices = [
      ...mockDevices,
      {
        id: '3',
        ip_address: '192.168.1.87',
        device_name: 'iPhone',
        os_type: 'iOS',
        device_class: 'phone',
        is_online: true,
        mac_address: 'de:ad:be:ef:ca:fe',
      },
    ];

    // Re-render with new props
    const updated = render(TopologyGraph, {
      props: {
        devices: newDevices,
        links: mockLinks,
      },
    });

    const circles = updated.container.querySelectorAll('circle');
    expect(circles.length).toBeGreaterThan(mockDevices.length);
  });

  it('should handle empty device list', () => {
    const empty = render(TopologyGraph, {
      props: {
        devices: [],
        links: [],
      },
    });

    const svg = empty.container.querySelector('svg');
    expect(svg).toBeTruthy();
  });
});
