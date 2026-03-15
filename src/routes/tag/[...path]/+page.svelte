<script lang="ts">
  import { page } from '$app/state';
  import TagPageTasks from '$lib/components/TagPageTasks.svelte';
  import TagPageBlocks from '$lib/components/TagPageBlocks.svelte';
  import { tagPage } from '$lib/stores/tags.svelte';

  $effect(() => {
    const path = page.params.path;
    if (path) {
      void tagPage.loadTag(path);
    }
  });

  function handleEdit(_id: string) {
    void tagPage.refresh();
  }
</script>

<h1 class="tag-header">#{tagPage.currentTag}</h1>

{#if tagPage.loading}
  <p class="loading">Loading...</p>
{:else}
  <TagPageTasks tasks={tagPage.tasks} onedit={handleEdit} />
  <TagPageBlocks blocks={tagPage.blocks} onedit={handleEdit} />
{/if}

<style>
  .tag-header {
    font-size: var(--text-3xl);
    font-weight: 700;
    margin-bottom: var(--space-8);
  }
  .loading {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
