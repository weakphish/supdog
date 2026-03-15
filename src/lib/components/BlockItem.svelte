<script lang="ts">
  import type { Block, TaskStatus } from '$lib/types';
  import TaskCheckbox from './TaskCheckbox.svelte';
  import TagPill from './TagPill.svelte';
  import { updateBlock } from '$lib/api';

  let { block, onedit, ondelete }: {
    block: Block;
    onedit: (id: string) => void;
    ondelete: (id: string) => void;
  } = $props();

  let editing = $state(false);
  let editContent = $state(block.content);

  function startEdit() {
    editing = true;
    editContent = block.content;
  }

  async function commitEdit() {
    if (editContent !== block.content) {
      await updateBlock(block.id, editContent);
    }
    editing = false;
    onedit(block.id);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void commitEdit();
    }
    if (e.key === 'Escape') {
      editing = false;
    }
  }

  async function handleStatusChange(newStatus: TaskStatus) {
    await updateBlock(block.id, undefined, undefined, newStatus);
    onedit(block.id);
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
      <input
        class="block-editor"
        bind:value={editContent}
        onkeydown={handleKeydown}
        onblur={() => void commitEdit()}
        autofocus
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
  .block-editor {
    flex: 1;
    border: none;
    outline: none;
    font: inherit;
    padding: var(--space-1);
    background: var(--bg-muted);
    border-radius: 2px;
  }
</style>
