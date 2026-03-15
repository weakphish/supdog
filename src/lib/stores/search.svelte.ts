import { search as apiSearch } from '$lib/api';
import type { SearchResult } from '$lib/types';

let open = $state(false);
let query = $state('');
let results = $state<SearchResult[]>([]);
let selectedIndex = $state(0);
let loading = $state(false);

export const searchStore = {
  get open() { return open; },
  set open(v: boolean) {
    open = v;
    if (!v) { query = ''; results = []; selectedIndex = 0; }
  },
  get query() { return query; },
  set query(v: string) { query = v; },
  get results() { return results; },
  get selectedIndex() { return selectedIndex; },
  set selectedIndex(v: number) { selectedIndex = v; },
  get loading() { return loading; },

  async doSearch(q: string) {
    query = q;
    if (q.length < 2) { results = []; return; }
    loading = true;
    try {
      const r = await apiSearch(q);
      if (query !== q) return; // stale guard
      results = r;
      selectedIndex = 0;
    } catch (e) {
      console.error('Search failed:', e);
    } finally {
      if (query === q) loading = false;
    }
  },

  moveUp() {
    if (selectedIndex > 0) selectedIndex--;
  },

  moveDown() {
    if (selectedIndex < results.length - 1) selectedIndex++;
  },

  selectedResult(): SearchResult | null {
    return results[selectedIndex] ?? null;
  }
};
