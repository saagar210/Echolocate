<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { select } from 'd3-selection';
	import { zoom, zoomIdentity, type ZoomBehavior } from 'd3-zoom';
	import { drag, type DragBehavior } from 'd3-drag';
	import {
		createSimulation,
		devicesToGraph,
		getEdgeColor,
		type GraphNode,
		type GraphLink
	} from '$lib/services/graph-layout';
	import type { Device } from '$lib/types/device';
	import type { Simulation } from 'd3-force';
	import GraphTooltip from './GraphTooltip.svelte';

	let {
		devices,
		selectedId = null,
		onSelectDevice
	}: {
		devices: Device[];
		selectedId: string | null;
		onSelectDevice: (id: string | null) => void;
	} = $props();

	let svgElement: SVGSVGElement;
	let containerElement: HTMLDivElement;
	let width = $state(800);
	let height = $state(600);

	let simulation: Simulation<GraphNode, GraphLink> | null = null;
	let zoomBehavior: ZoomBehavior<SVGSVGElement, unknown>;
	let nodes: GraphNode[] = $state([]);
	let links: GraphLink[] = $state([]);

	// Tooltip state
	let tooltipDevice: Device | null = $state(null);
	let tooltipX = $state(0);
	let tooltipY = $state(0);

	let resizeObserver: ResizeObserver;

	onMount(() => {
		resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				width = entry.contentRect.width;
				height = entry.contentRect.height;
			}
		});
		resizeObserver.observe(containerElement);

		setupGraph();
	});

	onDestroy(() => {
		simulation?.stop();
		resizeObserver?.disconnect();
	});

	// React to device changes
	$effect(() => {
		if (devices && svgElement) {
			updateGraph(devices);
		}
	});

	function setupGraph() {
		const svg = select(svgElement);

		// Setup zoom
		zoomBehavior = zoom<SVGSVGElement, unknown>()
			.scaleExtent([0.2, 4])
			.on('zoom', (event) => {
				svg.select('g.graph-content').attr('transform', event.transform);
			});

		svg.call(zoomBehavior);

		// Click on background to deselect
		svg.on('click', (event) => {
			if (event.target === svgElement) {
				onSelectDevice(null);
			}
		});
	}

	function updateGraph(deviceList: Device[]) {
		const graph = devicesToGraph(deviceList);

		// Preserve positions of existing nodes
		const existingPositions = new Map<string, { x: number; y: number; pinned: boolean }>();
		for (const node of nodes) {
			if (node.x !== undefined && node.y !== undefined) {
				existingPositions.set(node.id, { x: node.x, y: node.y, pinned: node.pinned });
			}
		}

		for (const node of graph.nodes) {
			const existing = existingPositions.get(node.id);
			if (existing) {
				node.x = existing.x;
				node.y = existing.y;
				node.pinned = existing.pinned;
				if (existing.pinned) {
					node.fx = existing.x;
					node.fy = existing.y;
				}
			}
		}

		nodes = graph.nodes;
		links = graph.links;

		// Restart simulation
		simulation?.stop();
		simulation = createSimulation(nodes, links, width, height);

		simulation.on('tick', () => {
			// Trigger reactivity by reassigning
			nodes = [...nodes];
			links = [...links];
		});

		// Gently reheat for new nodes
		if (existingPositions.size > 0) {
			simulation.alpha(0.3).restart();
		}
	}

	function handleNodeClick(event: MouseEvent, node: GraphNode) {
		event.stopPropagation();
		onSelectDevice(node.id);
	}

	function handleNodeMouseEnter(event: MouseEvent, node: GraphNode) {
		tooltipDevice = node.device;
		tooltipX = event.clientX;
		tooltipY = event.clientY;
	}

	function handleNodeMouseLeave() {
		tooltipDevice = null;
	}

	function handleDragStart(event: any, node: GraphNode) {
		if (!simulation) return;
		if (!event.active) simulation.alphaTarget(0.3).restart();
		node.fx = node.x;
		node.fy = node.y;
	}

	function handleDrag(event: any, node: GraphNode) {
		node.fx = event.x;
		node.fy = event.y;
	}

	function handleDragEnd(event: any, node: GraphNode) {
		if (!simulation) return;
		if (!event.active) simulation.alphaTarget(0);
		// Pin the node where it was dropped
		node.pinned = true;
	}

	// Setup drag behavior on node elements
	$effect(() => {
		if (!svgElement) return;
		const svg = select(svgElement);

		const dragBehavior = drag<SVGGElement, GraphNode>()
			.on('start', handleDragStart)
			.on('drag', handleDrag)
			.on('end', handleDragEnd);

		svg.selectAll<SVGGElement, GraphNode>('g.node').call(dragBehavior);
	});

	// Public methods for graph controls
	export function zoomIn() {
		select(svgElement).transition().duration(300).call(zoomBehavior.scaleBy, 1.3);
	}

	export function zoomOut() {
		select(svgElement).transition().duration(300).call(zoomBehavior.scaleBy, 0.7);
	}

	export function fitToScreen() {
		select(svgElement)
			.transition()
			.duration(500)
			.call(zoomBehavior.transform, zoomIdentity);
	}

	function getLinkSource(link: GraphLink): { x: number; y: number } {
		const src = link.source as GraphNode;
		return { x: src.x ?? 0, y: src.y ?? 0 };
	}

	function getLinkTarget(link: GraphLink): { x: number; y: number } {
		const tgt = link.target as GraphNode;
		return { x: tgt.x ?? 0, y: tgt.y ?? 0 };
	}

	function getDeviceIcon(device: Device): string {
		if (device.isGateway) return '◆';
		switch (device.deviceType) {
			case 'router': return '◆';
			case 'computer': return '▪';
			case 'phone': return '▮';
			case 'tablet': return '▭';
			case 'iot': return '⬡';
			case 'printer': return '▦';
			default: return '?';
		}
	}

	function isNew(device: Device): boolean {
		const firstSeen = new Date(device.firstSeen + 'Z');
		const now = new Date();
		return now.getTime() - firstSeen.getTime() < 5 * 60 * 1000;
	}
</script>

<div bind:this={containerElement} class="h-full w-full">
	<svg
		bind:this={svgElement}
		{width}
		{height}
		class="h-full w-full"
	>
		<g class="graph-content">
			<!-- Edges -->
			{#each links as link (link)}
				{@const src = getLinkSource(link)}
				{@const tgt = getLinkTarget(link)}
				<line
					x1={src.x}
					y1={src.y}
					x2={tgt.x}
					y2={tgt.y}
					stroke={getEdgeColor((link.source as GraphNode).device?.latencyMs ?? null)}
					stroke-width="1.5"
					stroke-opacity="0.5"
				/>
			{/each}

			<!-- Nodes -->
			{#each nodes as node (node.id)}
				{@const isSelected = node.id === selectedId}
				{@const isNodeNew = isNew(node.device)}
				<g
					class="node cursor-pointer"
					transform="translate({node.x ?? 0}, {node.y ?? 0})"
					onclick={(e) => handleNodeClick(e, node)}
					onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleNodeClick(e, node); }}
					onmouseenter={(e) => handleNodeMouseEnter(e, node)}
					onmouseleave={handleNodeMouseLeave}
					role="button"
					tabindex="0"
				>
					<!-- Pulse animation for new devices -->
					{#if isNodeNew}
						<circle
							r={node.radius + 8}
							fill="none"
							stroke={node.color}
							stroke-width="2"
							opacity="0.3"
						>
							<animate
								attributeName="r"
								values="{node.radius + 4};{node.radius + 12};{node.radius + 4}"
								dur="2s"
								repeatCount="indefinite"
							/>
							<animate
								attributeName="opacity"
								values="0.4;0.1;0.4"
								dur="2s"
								repeatCount="indefinite"
							/>
						</circle>
					{/if}

					<!-- Selection ring -->
					{#if isSelected}
						<circle
							r={node.radius + 4}
							fill="none"
							stroke="#38bdf8"
							stroke-width="2"
						/>
					{/if}

					<!-- Main circle -->
					<circle
						r={node.radius}
						fill={node.device.isOnline ? node.color : '#1e293b'}
						stroke={node.device.isTrusted ? node.color : '#64748b'}
						stroke-width={node.device.isTrusted ? 2 : 1.5}
						stroke-dasharray={node.device.isTrusted ? 'none' : '4 2'}
						opacity={node.device.isOnline ? 1 : 0.4}
						class="transition-opacity duration-500"
					/>

					<!-- Device icon -->
					<text
						text-anchor="middle"
						dominant-baseline="central"
						fill={node.device.isOnline ? '#fff' : '#94a3b8'}
						font-size={node.radius * 0.8}
						class="pointer-events-none select-none"
					>
						{getDeviceIcon(node.device)}
					</text>

					<!-- Label below node -->
					<text
						y={node.radius + 14}
						text-anchor="middle"
						fill="#94a3b8"
						font-size="10"
						class="pointer-events-none select-none"
					>
						{node.device.customName ?? node.device.hostname ?? node.device.currentIp ?? ''}
					</text>
				</g>
			{/each}
		</g>
	</svg>

	{#if tooltipDevice}
		<GraphTooltip device={tooltipDevice} x={tooltipX} y={tooltipY} />
	{/if}
</div>
