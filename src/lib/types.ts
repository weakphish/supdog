export type BlockType = 'bullet' | 'h1' | 'h2' | 'h3' | 'quote' | 'code' | 'task';
export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'cancelled';
export type Priority = 'high' | 'med' | 'low';

export interface Block {
  id: string;
  content: string;
  block_type: BlockType;
  parent_id: string | null;
  daily_note_id: string | null;
  position: number;
  status: TaskStatus | null;
  priority: Priority | null;
  due_date: string | null;
  created_at: string;
  updated_at: string;
  tags: string[];
  children: Block[];
}

export interface DailyNote {
  id: string;
  date: string;
}

export interface Tag {
  id: string;
  name: string;
  parent_id: string | null;
}

export interface BlockLink {
  id: string;
  source_id: string;
  target_id: string;
  created_at: string;
}

export interface MindMap {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface MindMapNode {
  id: string;
  mind_map_id: string;
  block_id: string;
  x: number;
  y: number;
}

export interface NodeWithBlock {
  node: MindMapNode;
  block: Block;
}

export interface SearchResult {
  block: Block;
  parent_content: string | null;
  daily_note_date: string | null;
}
