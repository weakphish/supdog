<script lang="ts">
  import { createBlock, getOrCreateDailyNote, createTag, addTagToBlock } from '$lib/api';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { emit } from '@tauri-apps/api/event';

  let input = $state('');
  let inputEl: HTMLInputElement | undefined = $state();
  let error = $state<string | null>(null);

  $effect(() => {
    inputEl?.focus();
  });

  function localDateStr(): string {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  }

  async function submit() {
    if (!input.trim()) return;

    let content = input.trim();
    let blockType: 'bullet' | 'task' = 'bullet';
    const tagNames: string[] = [];

    if (content.startsWith('[] ')) {
      blockType = 'task';
      content = content.slice(3);
    }

    const tagRegex = /#([\w/]+)/g;
    let match;
    while ((match = tagRegex.exec(content)) !== null) {
      tagNames.push(match[1]);
    }
    content = content.replace(/#[\w/]+/g, '').trim();

    error = null;
    try {
      const today = localDateStr();
      const note = await getOrCreateDailyNote(today);
      const block = await createBlock(note.id, null, content, blockType, Date.now());

      for (const name of tagNames) {
        const tag = await createTag(name);
        await addTagToBlock(block.id, tag.id);
      }

      await emit('journal-refresh');
      input = '';
      await getCurrentWindow().hide();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      void submit();
    }
    if (e.key === 'Escape') {
      input = '';
      void getCurrentWindow().hide();
    }
  }
</script>

<div class="capture">
  <input
    bind:this={inputEl}
    bind:value={input}
    class="capture-input"
    placeholder="What's on your mind? ([] for task, #tag)"
    onkeydown={handleKeydown}
    aria-label="Quick capture input"
  />
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <div class="hint">
    <span>bullet</span>
    <span>[] task</span>
    <span>#tag</span>
    <span>Enter ↵ · Esc to dismiss</span>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
  }
  .capture {
    padding: 16px;
    background: #fff;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .capture-input {
    border: none;
    outline: none;
    font-size: 18px;
    font-family: var(--font-sans, -apple-system, sans-serif);
    width: 100%;
  }
  .error {
    font-size: 11px;
    color: #c00;
    margin-top: 4px;
  }
  .hint {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 11px;
    color: #aaa;
  }
</style>
