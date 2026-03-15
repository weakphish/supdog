<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';
  import BlockTree from './BlockTree.svelte';

  let { tasks, onedit }: { tasks: Block[]; onedit: (id: string) => void } = $props();
</script>

{#if tasks.length > 0}
  <section class="tasks-section">
    <h3 class="section-label">Open Tasks</h3>
    {#each tasks as task (task.id)}
      <div class="task-with-context">
        <BlockItem block={task} {onedit} ondelete={() => {}} />
        {#if task.children.length > 0}
          <BlockTree blocks={task.children} depth={1} {onedit} ondelete={() => {}} />
        {/if}
      </div>
    {/each}
  </section>
{/if}

<style>
  .tasks-section {
    margin-bottom: var(--space-8);
  }
  .section-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: var(--space-3);
  }
  .task-with-context {
    margin-bottom: var(--space-3);
  }
</style>
