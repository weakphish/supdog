import { invoke } from '@tauri-apps/api/core';
import type { Block, DailyNote, Tag, BlockLink, MindMap, MindMapNode, NodeWithBlock, SearchResult } from './types';

// Daily Notes
export const getOrCreateDailyNote = (date: string) =>
  invoke<DailyNote>('get_or_create_daily_note', { date });

// Blocks
export const getBlocksForDate = (date: string) =>
  invoke<Block[]>('get_blocks_for_date', { date });

export const createBlock = (dailyNoteId: string, parentId: string | null, content: string, blockType: string, position: number) =>
  invoke<Block>('create_block', { dailyNoteId, parentId, content, blockType, position });

export const updateBlock = (id: string, content?: string, blockType?: string, status?: string, priority?: string, dueDate?: string) =>
  invoke<void>('update_block', { id, content, blockType, status, priority, dueDate });

export const deleteBlock = (id: string) =>
  invoke<void>('delete_block', { id });

export const reorderBlock = (id: string, newParentId: string | null, newPosition: number) =>
  invoke<void>('reorder_block', { id, newParentId, newPosition });

export const reparentBlock = (id: string, newParentId: string | null, position: number) =>
  invoke<void>('reparent_block', { id, newParentId, position });

// Tags
export const getAllTags = () =>
  invoke<Tag[]>('get_all_tags');

export const createTag = (name: string) =>
  invoke<Tag>('create_tag', { name });

export const addTagToBlock = (blockId: string, tagId: string) =>
  invoke<void>('add_tag_to_block', { blockId, tagId });

export const removeTagFromBlock = (blockId: string, tagId: string) =>
  invoke<void>('remove_tag_from_block', { blockId, tagId });

export const getBlocksByTag = (tagName: string) =>
  invoke<{ tasks: Block[]; blocks: Block[] }>('get_blocks_by_tag', { tagName });

// Links
export const createLink = (sourceId: string, targetId: string) =>
  invoke<BlockLink>('create_link', { sourceId, targetId });

export const deleteLink = (id: string) =>
  invoke<void>('delete_link', { id });

export const getBacklinks = (blockId: string) =>
  invoke<Block[]>('get_backlinks', { blockId });

export const getForwardLinks = (blockId: string) =>
  invoke<Block[]>('get_forward_links', { blockId });

// Search
export const search = (query: string, blockTypeFilter?: string, tagFilter?: string, statusFilter?: string) =>
  invoke<SearchResult[]>('search', { query, blockTypeFilter, tagFilter, statusFilter });

// Mind Maps
export const createMindMap = (name: string) =>
  invoke<MindMap>('create_mind_map', { name });

export const getMindMaps = () =>
  invoke<MindMap[]>('get_mind_maps');

export const deleteMindMap = (id: string) =>
  invoke<void>('delete_mind_map', { id });

export const addMindMapNode = (mindMapId: string, content: string, x: number, y: number) =>
  invoke<MindMapNode>('add_mind_map_node', { mindMapId, content, x, y });

export const updateNodePosition = (nodeId: string, x: number, y: number) =>
  invoke<void>('update_node_position', { nodeId, x, y });

export const getMindMapNodes = (mindMapId: string) =>
  invoke<MindMapNode[]>('get_mind_map_nodes', { mindMapId });

export const getMindMapNodesWithBlocks = (mindMapId: string) =>
  invoke<NodeWithBlock[]>('get_mind_map_nodes_with_blocks', { mindMapId });

export const sendNodesToJournal = (blockIds: string[], date: string) =>
  invoke<void>('send_nodes_to_journal', { blockIds, date });
