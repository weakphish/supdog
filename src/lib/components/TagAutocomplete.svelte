<script lang="ts">
  import { onMount } from 'svelte';
  import { getAllTags, createTag } from '$lib/api';
  import type { Tag } from '$lib/types';

  let { query, onselect, onclose }: {
    query: string;
    onselect: (tag: Tag) => void;
    onclose: () => void;
  } = $props();

  let allTags = $state<Tag[]>([]);
  let selectedIndex = $state(0);

  const filtered = $derived(
    allTags.filter(t => t.name.toLowerCase().includes(query.toLowerCase())).slice(0, 8)
  );

  // Reset selection when filtered results change
  $effect(() => {
    filtered; // subscribe to filtered
    selectedIndex = 0;
  });

  onMount(() => {
    getAllTags().then(t => allTags = t);
  });

  export function handleKeydown(e: KeyboardEvent): boolean {
    if (e.key === 'ArrowDown') { selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1); return true; }
    if (e.key === 'ArrowUp') { selectedIndex = Math.max(selectedIndex - 1, 0); return true; }
    if (e.key === 'Enter' && filtered.length > 0) {
      onselect(filtered[selectedIndex]);
      return true;
    }
    if (e.key === 'Escape') { onclose(); return true; }
    return false;
  }
</script>

<div class="autocomplete">
  {#each filtered as tag, i (tag.id)}
    <button
      class="autocomplete-item"
      class:selected={i === selectedIndex}
      onclick={() => onselect(tag)}
    >
      #{tag.name}
    </button>
  {/each}
  {#if filtered.length === 0 && query.length > 0}
    <button class="autocomplete-item create" onclick={async () => { const tag = await createTag(query); onselect(tag); }}>Create #{query}</button>
  {/if}
</div>

<style>
  .autocomplete {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    box-shadow: var(--shadow-md);
    min-width: 200px;
    z-index: 50;
  }
  .autocomplete-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: var(--text-sm);
  }
  .autocomplete-item:hover, .autocomplete-item.selected {
    background: var(--bg-hover);
  }
  .create {
    color: var(--text-muted);
    font-style: italic;
  }
</style>
