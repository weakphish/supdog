<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';

  let { blocks, onedit }: { blocks: Block[]; onedit: (id: string) => void } = $props();

  interface DateGroup {
    date: string;
    blocks: Block[];
  }

  const grouped: DateGroup[] = $derived.by(() => {
    const groups = new Map<string, Block[]>();
    for (const block of blocks) {
      const date = block.created_at.slice(0, 10);
      if (!groups.has(date)) groups.set(date, []);
      groups.get(date)!.push(block);
    }
    return Array.from(groups.entries())
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([date, blocks]) => ({ date, blocks }));
  });

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  }
</script>

{#if grouped.length > 0}
  <section>
    <h3 class="section-label">Blocks</h3>
    {#each grouped as group (group.date)}
      <div class="date-group">
        <a href="/journal/{group.date}" class="date-divider">{formatDate(group.date)}</a>
        {#each group.blocks as block (block.id)}
          <BlockItem {block} {onedit} ondelete={() => {}} />
        {/each}
      </div>
    {/each}
  </section>
{/if}

<style>
  .section-label {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: var(--space-3);
  }
  .date-group {
    margin-bottom: var(--space-4);
  }
  .date-divider {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-muted);
    text-decoration: none;
    margin-bottom: var(--space-2);
    padding-bottom: var(--space-1);
    border-bottom: 1px solid var(--border);
  }
  .date-divider:hover {
    color: var(--text-secondary);
  }
  .date-divider:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
