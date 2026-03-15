<script lang="ts">
  import { sidebar } from '$lib/stores/sidebar.svelte';
  import type { Tag } from '$lib/types';

  function todayDate(): string {
    return new Date().toISOString().split('T')[0];
  }

  function topLevelTags(tags: Tag[]): Tag[] {
    return tags.filter(t => !t.parent_id);
  }
</script>

<nav class="sidebar" class:collapsed={sidebar.collapsed}>
  <div class="sidebar-header">
    <button class="toggle" onclick={() => sidebar.toggle()}
        aria-label={sidebar.collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        aria-expanded={!sidebar.collapsed}>
      {sidebar.collapsed ? '→' : '←'}
    </button>
  </div>

  {#if !sidebar.collapsed}
    <div class="sidebar-content">
      <a href="/journal/{todayDate()}" class="nav-item">Journal</a>

      <div class="nav-section">
        <span class="nav-label">Tags</span>
        {#each topLevelTags(sidebar.tags) as tag}
          <a href="/tag/{tag.name}" class="nav-item nav-indent">{tag.name}</a>
        {/each}
      </div>

      <div class="nav-section">
        <span class="nav-label">Mind Maps</span>
        {#each sidebar.mindMaps as mm}
          <a href="/mindmap/{mm.id}" class="nav-item nav-indent">{mm.name}</a>
        {/each}
      </div>
    </div>
  {/if}
</nav>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100vh;
    border-right: 1px solid var(--border);
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    transition: width var(--transition-normal);
    overflow: hidden;
    flex-shrink: 0;
  }
  .sidebar.collapsed {
    width: var(--sidebar-collapsed-width);
  }
  .sidebar-header {
    padding: var(--space-3);
    display: flex;
    justify-content: flex-end;
  }
  .toggle {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-muted);
    padding: var(--space-1) var(--space-2);
  }
  .toggle:hover {
    color: var(--text-primary);
  }
  .sidebar-content {
    padding: 0 var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .nav-item {
    display: block;
    padding: var(--space-1) var(--space-2);
    color: var(--text-secondary);
    text-decoration: none;
    font-size: var(--text-sm);
    border-radius: 4px;
  }
  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .nav-item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .nav-indent {
    padding-left: var(--space-6);
  }
  .nav-section {
    margin-top: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .nav-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 0 var(--space-2);
  }
</style>
