import { getOrCreateDailyNote, getBlocksForDate, createBlock, deleteBlock as apiDeleteBlock } from '$lib/api';
import type { Block, DailyNote } from '$lib/types';

let currentDate = $state('');
let dailyNote = $state<DailyNote | null>(null);
let blocks = $state<Block[]>([]);
let loading = $state(false);

export const journal = {
  get currentDate() { return currentDate; },
  get dailyNote() { return dailyNote; },
  get blocks() { return blocks; },
  get loading() { return loading; },

  async loadDate(date: string) {
    loading = true;
    currentDate = date;
    try {
      dailyNote = await getOrCreateDailyNote(date);
      blocks = await getBlocksForDate(date);
    } catch (e) {
      console.error('Failed to load journal:', e);
    } finally {
      loading = false;
    }
  },

  async refresh() {
    if (currentDate) {
      try {
        blocks = await getBlocksForDate(currentDate);
      } catch (e) {
        console.error('Failed to refresh journal:', e);
      }
    }
  },

  async addBlock(parentId: string | null, content: string, blockType: string, position: number) {
    if (!dailyNote) return;
    try {
      await createBlock(dailyNote.id, parentId, content, blockType as import('$lib/types').BlockType, position);
      await journal.refresh();
    } catch (e) {
      console.error('Failed to add block:', e);
    }
  },

  async removeBlock(id: string) {
    try {
      await apiDeleteBlock(id);
      await journal.refresh();
    } catch (e) {
      console.error('Failed to remove block:', e);
    }
  }
};
