<script lang="ts">
  import {
    getMindMapNodesWithBlocks,
    addMindMapNode,
    updateNodePosition,
    sendNodesToJournal,
    getForwardLinks,
    createTag,
    addTagToBlock,
  } from '$lib/api';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import type { MindMapNode, Block, NodeWithBlock } from '$lib/types';

  let { mindMapId }: { mindMapId: string } = $props();

  interface CanvasNode {
    node: MindMapNode;
    block: Block;
  }

  let nodes = $state<CanvasNode[]>([]);
  let connections = $state<{ from: string; to: string }[]>([]);
  let selected = $state<Set<string>>(new Set());
  let dragging = $state<string | null>(null);
  let dragOffset = $state({ x: 0, y: 0 });
  let contextMenu = $state<{ x: number; y: number; nodeId: string } | null>(null);

  function localDateStr(): string {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  async function load() {
    const id = mindMapId;
    const raw = await getMindMapNodesWithBlocks(id);
    if (id !== mindMapId) return;
    const loaded = raw.map(r => ({ node: r.node, block: r.block }));

    const blockIds = new Set(loaded.map(n => n.node.block_id));
    const linksArrays = await Promise.all(loaded.map(n => getForwardLinks(n.node.block_id)));
    if (id !== mindMapId) return;

    const allConns: { from: string; to: string }[] = [];
    for (let i = 0; i < loaded.length; i++) {
      for (const target of linksArrays[i]) {
        if (blockIds.has(target.id)) {
          allConns.push({ from: loaded[i].node.block_id, to: target.id });
        }
      }
    }
    nodes = loaded;
    connections = allConns;
  }

  onMount(() => { void load(); });

  async function handleCanvasDblClick(e: MouseEvent) {
    const svg = e.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    await addMindMapNode(mindMapId, '', x, y);
    await load();
  }

  function handleNodeMouseDown(nodeId: string, e: MouseEvent) {
    e.stopPropagation();
    dragging = nodeId;
    const node = nodes.find(n => n.node.id === nodeId);
    if (node) {
      dragOffset = { x: e.clientX - node.node.x, y: e.clientY - node.node.y };
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (dragging) {
      const newX = e.clientX - dragOffset.x;
      const newY = e.clientY - dragOffset.y;
      nodes = nodes.map(n => n.node.id === dragging
        ? { ...n, node: { ...n.node, x: newX, y: newY } }
        : n
      );
    }
  }

  async function handleMouseUp() {
    if (dragging) {
      const nodeId = dragging;
      const node = nodes.find(n => n.node.id === nodeId);
      try {
        if (node) {
          const newX = node.node.x;
          const newY = node.node.y;
          nodes = nodes.map(n => n.node.id === nodeId
            ? { ...n, node: { ...n.node, x: newX, y: newY } }
            : n
          );
          await updateNodePosition(nodeId, newX, newY);
        }
      } finally {
        dragging = null;
      }
    }
  }

  function toggleSelect(nodeId: string, e: MouseEvent) {
    e.stopPropagation();
    if (e.shiftKey) {
      // Shift+click: add/remove from selection
      if (selected.has(nodeId)) {
        selected.delete(nodeId);
      } else {
        selected.add(nodeId);
      }
      selected = new Set(selected);
    } else {
      // Normal click: clear selection, select only this node
      selected = new Set([nodeId]);
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && contextMenu) {
      closeContextMenu();
    }
  }

  async function sendSelected() {
    const blockIds = nodes
      .filter(n => selected.has(n.node.id))
      .map(n => n.node.block_id);
    await sendNodesToJournal(blockIds, localDateStr());
    selected = new Set();
    await load();
  }

  function handleContextMenu(nodeId: string, e: MouseEvent) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, nodeId };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  async function convertToTask(nodeId: string) {
    const n = nodes.find(n => n.node.id === nodeId);
    if (n) {
      await invoke('update_block', { id: n.node.block_id, block_type: 'task', status: 'todo' });
      await load();
    }
    closeContextMenu();
  }

  async function addTagToNode(nodeId: string) {
    const tagName = prompt('Tag name:');
    if (!tagName) { closeContextMenu(); return; }
    const n = nodes.find(n => n.node.id === nodeId);
    if (n) {
      const tag = await createTag(tagName);
      await addTagToBlock(n.node.block_id, tag.id);
    }
    closeContextMenu();
  }

  async function sendOneToJournal(nodeId: string) {
    const n = nodes.find(n => n.node.id === nodeId);
    if (n) {
      await sendNodesToJournal([n.node.block_id], localDateStr());
      await load();
    }
    closeContextMenu();
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if contextMenu}
  <div
    class="context-menu-backdrop"
    onclick={closeContextMenu}
    onkeydown={(e) => { if (e.key === 'Escape') closeContextMenu(); }}
    role="presentation"
  >
    <div
      class="context-menu"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
      onclick={(e) => e.stopPropagation()}
      role="menu"
    >
      <button role="menuitem" onclick={() => contextMenu && void convertToTask(contextMenu.nodeId)}>Convert to task</button>
      <button role="menuitem" onclick={() => contextMenu && void addTagToNode(contextMenu.nodeId)}>Add tag</button>
      <button role="menuitem" onclick={() => contextMenu && void sendOneToJournal(contextMenu.nodeId)}>Send to journal</button>
    </div>
  </div>
{/if}

<div class="mind-map-toolbar">
  {#if selected.size > 0}
    <button onclick={() => void sendSelected()} class="toolbar-btn">
      Send {selected.size} to journal
    </button>
  {/if}
</div>

<svg
  class="mind-map-canvas"
  ondblclick={(e) => void handleCanvasDblClick(e)}
  onmousemove={handleMouseMove}
  onmouseup={() => void handleMouseUp()}
>
  {#each connections as conn}
    {@const from = nodes.find(n => n.node.block_id === conn.from)}
    {@const to = nodes.find(n => n.node.block_id === conn.to)}
    {#if from && to}
      <line
        x1={from.node.x}
        y1={from.node.y}
        x2={to.node.x}
        y2={to.node.y}
        stroke="var(--border-strong)"
        stroke-width="1"
      />
    {/if}
  {/each}

  {#each nodes as n (n.node.id)}
    <g
      transform="translate({n.node.x}, {n.node.y})"
      onmousedown={(e) => handleNodeMouseDown(n.node.id, e)}
      onclick={(e) => toggleSelect(n.node.id, e)}
      ondblclick={(e) => e.stopPropagation()}
      oncontextmenu={(e) => handleContextMenu(n.node.id, e)}
      class="mind-map-node"
      class:selected={selected.has(n.node.id)}
      role="button"
      tabindex="0"
      aria-label={n.block.content || 'empty node'}
      onkeydown={(e) => { if (e.key === ' ' || e.key === 'Enter') toggleSelect(n.node.id, e as unknown as MouseEvent); }}
    >
      <rect
        x="-60" y="-20" width="120" height="40"
        rx="6" ry="6"
        fill="var(--bg-surface)"
        stroke={selected.has(n.node.id) ? 'var(--accent)' : 'var(--border)'}
        stroke-width={selected.has(n.node.id) ? 2 : 1}
      />
      <text
        text-anchor="middle"
        dominant-baseline="central"
        font-size="13"
        fill="var(--text-primary)"
      >
        {n.block.content || ''}
      </text>
    </g>
  {/each}
</svg>

<style>
  .mind-map-toolbar {
    padding: var(--space-2) var(--space-4);
    display: flex;
    gap: var(--space-2);
    border-bottom: 1px solid var(--border);
    min-height: 48px;
    align-items: center;
  }
  .toolbar-btn {
    font-size: var(--text-sm);
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-surface);
    cursor: pointer;
  }
  .toolbar-btn:hover {
    background: var(--bg-hover);
  }
  .mind-map-canvas {
    width: 100%;
    height: calc(100vh - 48px);
    cursor: crosshair;
    display: block;
  }
  .mind-map-node {
    cursor: grab;
  }
  .mind-map-node:active {
    cursor: grabbing;
  }
  .context-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
  }
  .context-menu {
    position: fixed;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: var(--space-1) 0;
    box-shadow: var(--shadow-lg);
    z-index: 101;
    display: flex;
    flex-direction: column;
  }
  .context-menu button {
    padding: var(--space-2) var(--space-4);
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-primary);
  }
  .context-menu button:hover {
    background: var(--bg-hover);
  }
</style>
