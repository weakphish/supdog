import { getAllTags, getMindMaps } from '$lib/api';
import type { Tag, MindMap } from '$lib/types';

let collapsed = $state(false);
let tags = $state<Tag[]>([]);
let mindMaps = $state<MindMap[]>([]);

export const sidebar = {
  get collapsed() { return collapsed; },
  set collapsed(v: boolean) { collapsed = v; },
  get tags() { return tags; },
  get mindMaps() { return mindMaps; },
  toggle() { collapsed = !collapsed; },
  async loadTags() {
    try {
      tags = await getAllTags();
    } catch (e) {
      console.error('Failed to load tags:', e);
    }
  },
  async loadMindMaps() {
    try {
      mindMaps = await getMindMaps();
    } catch (e) {
      console.error('Failed to load mind maps:', e);
    }
  }
};
