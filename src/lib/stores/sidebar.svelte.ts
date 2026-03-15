import { getAllTags, getMindMaps } from '$lib/api';
import type { Tag, MindMap } from '$lib/types';

let collapsed = $state(false);
let tags = $state<Tag[]>([]);
let mindMaps = $state<MindMap[]>([]);

export function sidebarState() {
  return {
    get collapsed() { return collapsed; },
    set collapsed(v: boolean) { collapsed = v; },
    get tags() { return tags; },
    get mindMaps() { return mindMaps; },
    toggle() { collapsed = !collapsed; },
    async loadTags() {
      tags = await getAllTags();
    },
    async loadMindMaps() {
      mindMaps = await getMindMaps();
    }
  };
}
