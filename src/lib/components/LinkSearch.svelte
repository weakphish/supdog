<script lang="ts">
  import { search } from '$lib/api';
  import type { SearchResult } from '$lib/types';

  let { query, onselect, onclose }: {
    query: string;
    onselect: (blockId: string) => void;
    onclose: () => void;
  } = $props();

  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);

  $effect(() => {
    const q = query;
    if (q.length >= 2) {
      void search(q).then(r => {
        if (query !== q) return; // stale guard
        results = r.slice(0, 8);
        selectedIndex = 0;
      }).catch(e => console.error('Failed to search:', e));
    } else {
      results = [];
    }
  });

  export function handleKeydown(e: KeyboardEvent): boolean {
    if (e.key === 'ArrowDown') { selectedIndex = Math.min(selectedIndex + 1, results.length - 1); return true; }
    if (e.key === 'ArrowUp') { selectedIndex = Math.max(selectedIndex - 1, 0); return true; }
    if (e.key === 'Enter' && results.length > 0) {
      onselect(results[selectedIndex].block.id);
      return true;
    }
    if (e.key === 'Escape') { onclose(); return true; }
    return false;
  }
</script>

<div class="link-search">
  {#each results as result, i (result.block.id)}
    <button
      class="link-item"
      class:selected={i === selectedIndex}
      onclick={() => onselect(result.block.id)}
    >
      <span class="link-content">{result.block.content}</span>
      {#if result.daily_note_date}
        <span class="link-date">{result.daily_note_date}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .link-search {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: var(--shadow-md);
    min-width: 300px;
    z-index: 50;
  }
  .link-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: var(--text-sm);
    gap: var(--space-2);
  }
  .link-item:hover, .link-item.selected {
    background: var(--bg-hover);
  }
  .link-content { flex: 1; }
  .link-date {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
