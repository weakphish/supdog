<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import BlockTree from '$lib/components/BlockTree.svelte';
  import DateNav from '$lib/components/DateNav.svelte';
  import { journal } from '$lib/stores/journal.svelte';

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
{:else if journal.blocks.length === 0}
  <p class="empty">No entries yet. Start typing to add one.</p>
{:else}
  <BlockTree blocks={journal.blocks} onedit={handleEdit} ondelete={handleDelete} />
{/if}

<style>
  .loading, .empty {
    color: var(--text-muted);
    font-size: var(--text-sm);
    padding: var(--space-8) 0;
  }
</style>
