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
      const note = await getOrCreateDailyNote(date);
      if (currentDate !== date) return; // navigated away, discard
      dailyNote = note;
      const b = await getBlocksForDate(date);
      if (currentDate !== date) return; // navigated away, discard
      blocks = b;
    } catch (e) {
      console.error('Failed to load journal:', e);
    } finally {
      if (currentDate === date) loading = false;
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
