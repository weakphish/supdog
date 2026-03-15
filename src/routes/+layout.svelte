<script lang="ts">
  import '../app.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { sidebar } from '$lib/stores/sidebar.svelte';
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    void sidebar.loadTags();
    void sidebar.loadMindMaps();
  });
</script>

<div class="app-shell">
  <Sidebar />
  <main class="main-content">
    <div class="content-inner">
      {@render children()}
    </div>
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }
  .main-content {
    flex: 1;
    overflow-y: auto;
  }
  .content-inner {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-8) var(--space-12);
  }
</style>
