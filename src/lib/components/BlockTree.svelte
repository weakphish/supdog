<script lang="ts">
  import type { Block } from '$lib/types';
  import BlockItem from './BlockItem.svelte';

  let { blocks, depth = 0, onedit, ondelete, onindent, onoutdent }: {
    blocks: Block[];
    depth?: number;
    onedit: (id: string) => void;
    ondelete: (id: string) => void;
    onindent?: (blockId: string) => void;
    onoutdent?: (blockId: string) => void;
  } = $props();
</script>

<div class="block-tree" style="padding-left: {depth > 0 ? 'var(--indent)' : '0'}">
  {#each blocks as block (block.id)}
    <BlockItem {block} {onedit} {ondelete} {onindent} {onoutdent} />
    {#if (block.children ?? []).length > 0}
      <svelte:self blocks={block.children ?? []} depth={depth + 1} {onedit} {ondelete} {onindent} {onoutdent} />
    {/if}
  {/each}
</div>

<style>
  .block-tree {
    /* Minimal — indentation via inline style */
  }
</style>
