<script lang="ts">
  import { onMount } from 'svelte';
  import TagAutocomplete from './TagAutocomplete.svelte';
  import LinkSearch from './LinkSearch.svelte';
  import { addTagToBlock, createLink, updateBlock, reorderBlock } from '$lib/api';
  import type { Block } from '$lib/types';

  let { blockId, block, initialContent, oncommit, oncancel, onindent, onoutdent }: {
    blockId: string;
    block: Block;
    initialContent: string;
    oncommit: (content: string) => void;
    oncancel: () => void;
    onindent?: (blockId: string) => void;
    onoutdent?: (blockId: string) => void;
  } = $props();

  let content = $state(initialContent);
  let mode = $state<'normal' | 'tag' | 'link'>('normal');
  let triggerQuery = $state('');
  let inputEl: HTMLInputElement | undefined = $state();

  onMount(() => { inputEl?.focus(); });

  function handleInput(e: Event) {
    content = (e.target as HTMLInputElement).value;

    const hashMatch = content.match(/#([\w/]*)$/);
    if (hashMatch) {
      mode = 'tag';
      triggerQuery = hashMatch[1];
      return;
    }

    const linkMatch = content.match(/\[\[([^\]]*)$/);
    if (linkMatch) {
      mode = 'link';
      triggerQuery = linkMatch[1];
      return;
    }

    mode = 'normal';
    triggerQuery = '';
  }

  async function handleTagSelect(tag: { id: string; name: string }) {
    content = content.replace(/#[\w/]*$/, '').trim();
    await addTagToBlock(blockId, tag.id);
    mode = 'normal';
    inputEl?.focus();
  }

  async function handleLinkSelect(targetBlockId: string) {
    content = content.replace(/\[\[[^\]]*$/, '').trim();
    await createLink(blockId, targetBlockId);
    mode = 'normal';
    inputEl?.focus();
  }

  let tagAutocomplete: { handleKeydown: (e: KeyboardEvent) => boolean } | undefined;
  let linkSearch: { handleKeydown: (e: KeyboardEvent) => boolean } | undefined;

  function handleKeydown(e: KeyboardEvent) {
    if (mode === 'tag' && tagAutocomplete?.handleKeydown(e)) return;
    if (mode === 'link' && linkSearch?.handleKeydown(e)) return;

    if (e.key === 'Tab' && !e.shiftKey && onindent) {
      e.preventDefault();
      onindent(blockId);
      return;
    }
    if (e.key === 'Tab' && e.shiftKey && onoutdent) {
      e.preventDefault();
      onoutdent(blockId);
      return;
    }

    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      const isTask = block.block_type === 'task';
      void updateBlock(blockId, undefined, isTask ? 'bullet' : 'task', isTask ? undefined : 'todo')
        .then(() => oncommit(content));
      return;
    }

    if (e.key === 'ArrowUp' && e.altKey) {
      e.preventDefault();
      void reorderBlock(blockId, block.parent_id, Math.max(0, block.position - 1))
        .then(() => oncommit(content));
      return;
    }
    if (e.key === 'ArrowDown' && e.altKey) {
      e.preventDefault();
      void reorderBlock(blockId, block.parent_id, block.position + 1)
        .then(() => oncommit(content));
      return;
    }

    if (e.key === 'Enter' && !e.shiftKey && mode === 'normal') {
      e.preventDefault();
      oncommit(content);
    }
    if (e.key === 'Escape' && mode === 'normal') {
      oncancel();
    }
  }
</script>

<div class="block-editor-wrapper">
  <input
    bind:this={inputEl}
    class="block-editor-input"
    value={content}
    oninput={handleInput}
    onkeydown={handleKeydown}
  />

  {#if mode === 'tag'}
    <TagAutocomplete
      bind:this={tagAutocomplete}
      query={triggerQuery}
      onselect={(tag) => void handleTagSelect(tag)}
      onclose={() => { mode = 'normal'; }}
    />
  {/if}

  {#if mode === 'link'}
    <LinkSearch
      bind:this={linkSearch}
      query={triggerQuery}
      onselect={(id) => void handleLinkSelect(id)}
      onclose={() => { mode = 'normal'; }}
    />
  {/if}
</div>

<style>
  .block-editor-wrapper {
    position: relative;
    flex: 1;
  }
  .block-editor-input {
    width: 100%;
    border: none;
    outline: none;
    font: inherit;
    padding: var(--space-1);
    background: var(--bg-muted);
    border-radius: 2px;
  }
</style>
