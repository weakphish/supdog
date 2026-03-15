import { getBlocksByTag } from '$lib/api';
import type { Block } from '$lib/types';

let currentTag = $state('');
let tasks = $state<Block[]>([]);
let blocks = $state<Block[]>([]);
let loading = $state(false);

export const tagPage = {
  get currentTag() { return currentTag; },
  get tasks() { return tasks; },
  get blocks() { return blocks; },
  get loading() { return loading; },

  async loadTag(tagName: string) {
    loading = true;
    currentTag = tagName;
    try {
      const result = await getBlocksByTag(tagName);
      if (currentTag !== tagName) return; // stale guard
      tasks = result.tasks;
      blocks = result.blocks;
    } catch (e) {
      console.error('Failed to load tag page:', e);
    } finally {
      if (currentTag === tagName) loading = false;
    }
  },

  async refresh() {
    const tag = currentTag;
    if (tag) {
      try {
        const result = await getBlocksByTag(tag);
        if (currentTag !== tag) return; // stale guard
        tasks = result.tasks;
        blocks = result.blocks;
      } catch (e) {
        console.error('Failed to refresh tag page:', e);
      }
    }
  }
};
