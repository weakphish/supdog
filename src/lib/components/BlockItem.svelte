<script lang="ts">
  import type { Block, TaskStatus } from '$lib/types';
  import TaskCheckbox from './TaskCheckbox.svelte';
  import TagPill from './TagPill.svelte';
  import BlockEditor from './BlockEditor.svelte';
  import { updateBlock } from '$lib/api';

  let { block, onedit, ondelete, onindent, onoutdent }: {
    block: Block;
    onedit: (id: string) => void;
    ondelete: (id: string) => void;
    onindent?: (blockId: string) => void;
    onoutdent?: (blockId: string) => void;
  } = $props();

  let editing = $state(false);
  let editContent = $state(block.content);

  $effect(() => {
    if (!editing) {
      editContent = block.content;
    }
  });

  function startEdit() {
    editing = true;
    editContent = block.content;
  }

  async function handleCommit(newContent: string) {
    try {
      if (newContent !== block.content) {
        await updateBlock(block.id, newContent);
      }
      editing = false;
      onedit(block.id);
    } catch (e) {
      console.error('Failed to update block:', e);
      editing = false;
    }
  }

  function handleCancel() {
    editing = false;
  }

  async function handleStatusChange(newStatus: TaskStatus) {
    try {
      await updateBlock(block.id, undefined, undefined, newStatus);
      onedit(block.id);
    } catch (e) {
      console.error('Failed to update block status:', e);
    }
  }

  const isDone = $derived(block.status === 'done');
  const isCancelled = $derived(block.status === 'cancelled');
</script>

<div class="block-item" class:done={isDone} class:cancelled={isCancelled}>
  <div class="block-line">
    {#if block.block_type === 'task' && block.status}
      <TaskCheckbox status={block.status} onchange={(s) => void handleStatusChange(s)} />
    {/if}

    {#if editing}
      <BlockEditor
        blockId={block.id}
        {block}
        initialContent={editContent}
        oncommit={(c) => void handleCommit(c)}
        oncancel={handleCancel}
        {onindent}
        {onoutdent}
      />
    {:else}
      <span
        class="block-content"
        class:task-content={block.block_type === 'task'}
        onclick={startEdit}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') startEdit(); }}
        role="button"
        tabindex="0"
      >
        {block.content}
      </span>
    {/if}

    {#each block.tags as tag}
      <TagPill name={tag} />
    {/each}
  </div>
</div>

<style>
  .block-item {
    padding: var(--space-1) 0;
  }
  .block-line {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }
  .block-content {
    cursor: text;
    flex: 1;
  }
  .block-content:hover {
    background: var(--bg-hover);
    border-radius: 2px;
  }
  .done .block-content {
    text-decoration: line-through;
    color: var(--text-muted);
  }
  .cancelled .block-content {
    text-decoration: line-through;
    color: var(--text-muted);
  }

</style>
