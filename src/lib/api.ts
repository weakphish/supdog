import { invoke } from '@tauri-apps/api/core';
import type { Block, BlockType, DailyNote, Tag, BlockLink, MindMap, MindMapNode, NodeWithBlock, Priority, SearchResult, TaskStatus } from './types';

// Daily Notes
export const getOrCreateDailyNote = (date: string) =>
  invoke<DailyNote>('get_or_create_daily_note', { date });

// Blocks
export const getBlocksForDate = (date: string) =>
  invoke<Block[]>('get_blocks_for_date', { date });

export const createBlock = (dailyNoteId: string, parentId: string | null, content: string, blockType: BlockType, position: number) =>
  invoke<Block>('create_block', { daily_note_id: dailyNoteId, parent_id: parentId, content, block_type: blockType, position });

export const updateBlock = (id: string, content?: string, blockType?: BlockType, status?: TaskStatus, priority?: Priority, dueDate?: string) =>
  invoke<void>('update_block', { id, content, block_type: blockType, status, priority, due_date: dueDate });

export const deleteBlock = (id: string) =>
  invoke<void>('delete_block', { id });

export const reorderBlock = (id: string, newParentId: string | null, newPosition: number) =>
  invoke<void>('reorder_block', { id, new_parent_id: newParentId, new_position: newPosition });

export const reparentBlock = (id: string, newParentId: string | null, position: number) =>
  invoke<void>('reparent_block', { id, new_parent_id: newParentId, position });

// Tags
export const getAllTags = () =>
  invoke<Tag[]>('get_all_tags');

export const createTag = (name: string) =>
  invoke<Tag>('create_tag', { name });

export const addTagToBlock = (blockId: string, tagId: string) =>
  invoke<void>('add_tag_to_block', { block_id: blockId, tag_id: tagId });

export const removeTagFromBlock = (blockId: string, tagId: string) =>
  invoke<void>('remove_tag_from_block', { block_id: blockId, tag_id: tagId });

export const getTagsForBlock = (blockId: string) =>
  invoke<Tag[]>('get_tags_for_block', { block_id: blockId });

export const getBlocksByTag = (tagName: string) =>
  invoke<{ tasks: Block[]; blocks: Block[] }>('get_blocks_by_tag', { tag_name: tagName });

// Links
export const createLink = (sourceId: string, targetId: string) =>
  invoke<BlockLink>('create_link', { source_id: sourceId, target_id: targetId });

export const deleteLink = (id: string) =>
  invoke<void>('delete_link', { id });

export const getBacklinks = (blockId: string) =>
  invoke<Block[]>('get_backlinks', { block_id: blockId });

export const getForwardLinks = (blockId: string) =>
  invoke<Block[]>('get_forward_links', { block_id: blockId });

// Search
export const search = (query: string, blockTypeFilter?: string, tagFilter?: string, statusFilter?: string) =>
  invoke<SearchResult[]>('search', { query, block_type_filter: blockTypeFilter, tag_filter: tagFilter, status_filter: statusFilter });

// Mind Maps
export const createMindMap = (name: string) =>
  invoke<MindMap>('create_mind_map', { name });

export const getMindMaps = () =>
  invoke<MindMap[]>('get_mind_maps');

export const deleteMindMap = (id: string) =>
  invoke<void>('delete_mind_map', { id });

export const addMindMapNode = (mindMapId: string, content: string, x: number, y: number) =>
  invoke<MindMapNode>('add_mind_map_node', { mind_map_id: mindMapId, content, x, y });

export const updateNodePosition = (nodeId: string, x: number, y: number) =>
  invoke<void>('update_node_position', { node_id: nodeId, x, y });

export const getMindMapNodes = (mindMapId: string) =>
  invoke<MindMapNode[]>('get_mind_map_nodes', { mind_map_id: mindMapId });

export const getMindMapNodesWithBlocks = (mindMapId: string) =>
  invoke<NodeWithBlock[]>('get_mind_map_nodes_with_blocks', { mind_map_id: mindMapId });

export const sendNodesToJournal = (blockIds: string[], date: string) =>
  invoke<void>('send_nodes_to_journal', { block_ids: blockIds, date });
