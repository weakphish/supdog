<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import BlockTree from '$lib/components/BlockTree.svelte';
  import DateNav from '$lib/components/DateNav.svelte';
  import { journal } from '$lib/stores/journal.svelte';
  import { reparentBlock } from '$lib/api';
  import type { Block } from '$lib/types';

  onMount(() => {
    const unlistenPromise = listen('journal-refresh', () => {
      void journal.refresh();
    });
    return () => { void unlistenPromise.then(fn => fn()); };
  });

  // Reactive: reload when date param changes.
  // void wraps async call so $effect body stays synchronous.
  $effect(() => {
    const date = page.params.date;
    if (date) {
      void journal.loadDate(date);
    }
  });

  function handleEdit(_id: string) {
    void journal.refresh();
  }

  function handleDelete(id: string) {
    void journal.removeBlock(id);
  }

  let newBlockInput = $state('');
  let newBlockEl: HTMLInputElement | undefined = $state();

  async function createNewBlock() {
    const content = newBlockInput.trim();
    if (!content) return;
    newBlockInput = '';
    const pos = journal.blocks.length;
    await journal.addBlock(null, content, 'bullet', pos);
    newBlockEl?.focus();
  }

  function handleNewBlockKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      void createNewBlock();
    }
  }

  function findBlock(blocks: Block[], id: string): Block | null {
    for (const b of blocks) {
      if (b.id === id) return b;
      const found = findBlock(b.children ?? [], id);
      if (found) return found;
    }
    return null;
  }

  function findParentBlock(blocks: Block[], childId: string, parent: Block | null = null): Block | null {
    for (const b of blocks) {
      if (b.id === childId) return parent;
      const found = findParentBlock(b.children ?? [], childId, b);
      if (found) return found;
    }
    return null;
  }

  function findPreviousSibling(blocks: Block[], blockId: string): Block | null {
    for (let i = 1; i < blocks.length; i++) {
      if (blocks[i].id === blockId) return blocks[i - 1];
    }
    for (const b of blocks) {
      const found = findPreviousSibling(b.children ?? [], blockId);
      if (found) return found;
    }
    return null;
  }

  async function handleIndent(blockId: string) {
    try {
      const prev = findPreviousSibling(journal.blocks, blockId);
      if (prev) {
        await reparentBlock(blockId, prev.id, (prev.children ?? []).length);
        await journal.refresh();
      }
    } catch (e) {
      console.error('Failed to indent block:', e);
    }
  }

  async function handleOutdent(blockId: string) {
    try {
      const parent = findParentBlock(journal.blocks, blockId);
      if (parent) {
        const grandparent = findParentBlock(journal.blocks, parent.id);
        await reparentBlock(blockId, grandparent?.id ?? null, parent.position + 1);
        await journal.refresh();
      }
    } catch (e) {
      console.error('Failed to outdent block:', e);
    }
  }

  function localDateStr(d: Date): string {
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft' && e.altKey) {
      const d = new Date(page.params.date + 'T00:00:00');
      d.setDate(d.getDate() - 1);
      void goto(`/journal/${localDateStr(d)}`);
    }
    if (e.key === 'ArrowRight' && e.altKey) {
      const d = new Date(page.params.date + 'T00:00:00');
      d.setDate(d.getDate() + 1);
      void goto(`/journal/${localDateStr(d)}`);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<DateNav date={page.params.date} />

{#if journal.loading}
  <p class="loading">Loading...</p>
{:else}
  {#if journal.blocks.length > 0}
    <BlockTree blocks={journal.blocks} onedit={handleEdit} ondelete={handleDelete}
      onindent={(id) => void handleIndent(id)}
      onoutdent={(id) => void handleOutdent(id)}
    />
  {/if}
  <input
    bind:this={newBlockEl}
    bind:value={newBlockInput}
    class="new-block-input"
    placeholder={journal.blocks.length === 0 ? 'Start typing…' : 'New block…'}
    onkeydown={handleNewBlockKeydown}
    autofocus={journal.blocks.length === 0}
  />
{/if}

<style>
  .loading {
    color: var(--text-muted);
    font-size: var(--text-sm);
    padding: var(--space-8) 0;
  }
  .new-block-input {
    display: block;
    width: 100%;
    border: none;
    outline: none;
    font: inherit;
    font-size: var(--text-sm);
    color: var(--text-primary);
    padding: var(--space-1) 0;
    margin-top: var(--space-1);
    background: transparent;
  }
  .new-block-input::placeholder {
    color: var(--text-muted);
  }
</style>
