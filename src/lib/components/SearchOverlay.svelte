<script lang="ts">
  import { goto } from '$app/navigation';
  import { searchStore } from '$lib/stores/search.svelte';
  import TagPill from './TagPill.svelte';

  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (searchStore.open && inputEl) {
      inputEl.focus();
    }
  });

  let debounceTimer: ReturnType<typeof setTimeout>;

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => void searchStore.doSearch(value), 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      searchStore.open = false;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      searchStore.moveUp();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      searchStore.moveDown();
    } else if (e.key === 'Enter') {
      const result = searchStore.selectedResult();
      if (result?.daily_note_date) {
        void goto(`/journal/${result.daily_note_date}`);
        searchStore.open = false;
      }
    }
  }
</script>

{#if searchStore.open}
  <div
    class="search-backdrop"
    onclick={() => { searchStore.open = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') searchStore.open = false; }}
    role="presentation"
  >
    <div
      class="search-panel"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-label="Search"
      aria-modal="true"
    >
      <input
        bind:this={inputEl}
        class="search-input"
        placeholder="Search..."
        value={searchStore.query}
        oninput={handleInput}
        onkeydown={handleKeydown}
        aria-label="Search query"
        autocomplete="off"
      />

      {#if searchStore.results.length > 0}
        <div class="search-results" role="listbox" aria-label="Search results">
          {#each searchStore.results as result, i (result.block.id)}
            <button
              class="search-result"
              class:selected={i === searchStore.selectedIndex}
              onclick={() => {
                if (result.daily_note_date) void goto(`/journal/${result.daily_note_date}`);
                searchStore.open = false;
              }}
              role="option"
              aria-selected={i === searchStore.selectedIndex}
            >
              <span class="result-content">{result.block.content}</span>
              {#if result.parent_content}
                <span class="result-parent">{result.parent_content}</span>
              {/if}
              <div class="result-meta">
                {#if result.daily_note_date}
                  <span class="result-date">{result.daily_note_date}</span>
                {/if}
                {#each result.block.tags as tag}
                  <TagPill name={tag} />
                {/each}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.2);
    display: flex;
    justify-content: center;
    padding-top: 15vh;
    z-index: 100;
  }
  .search-panel {
    background: var(--bg-surface);
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    width: 560px;
    max-height: 60vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-self: flex-start;
  }
  .search-input {
    border: none;
    outline: none;
    font: inherit;
    font-size: var(--text-lg);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border);
  }
  .search-results {
    overflow-y: auto;
    padding: var(--space-2) 0;
  }
  .search-result {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--space-2) var(--space-6);
    font: inherit;
  }
  .search-result:hover, .search-result.selected {
    background: var(--bg-hover);
  }
  .result-content {
    display: block;
    font-size: var(--text-sm);
  }
  .result-parent {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-muted);
    margin-top: 2px;
  }
  .result-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 4px;
  }
  .result-date {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }
</style>
