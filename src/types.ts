// Mirrors the Rust types serialized by src-tauri/src/storage/repo.rs.

export type TaskStatus =
  | "created"
  | "planning"
  | "awaiting_approval"
  | "executing"
  | "verifying"
  | "completed"
  | "partially_completed"
  | "failed"
  | "cancelled";

export type StepStatus =
  | "pending"
  | "started"
  | "done"
  | "failed"
  | "skipped"
  | "undone"
  | "undo_failed";

export type Step =
  | { seq: number; op: "create_folder"; path: string; status: StepStatus; error: string | null }
  | { seq: number; op: "move_file"; from: string; to: string; status: StepStatus; error: string | null };

export interface PlanMeta {
  categories: string[];
  scannedFiles: number;
  leftInPlace: string[];
  rejectedOutputs: number;
}

export interface Task {
  id: string;
  instruction: string;
  status: TaskStatus;
  modelId: string;
  root: string;
  conversationId: string | null;
  plan: PlanMeta | null;
  error: string | null;
  createdAt: string;
  completedAt: string | null;
  undoneAt: string | null;
  steps: Step[];
}

export interface Project {
  id: string;
  name: string;
  description: string;
  conversationCount: number;
  createdAt: string;
}

export interface Conversation {
  id: string;
  projectId: string;
  title: string;
  persona: string;
  grantId: string | null;
  folder: string | null;
  instructions: string;
  messageCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface Message {
  id: number;
  role: "user" | "assistant";
  text: string;
  taskId: string | null;
  createdAt: string;
}

export interface ConversationView {
  conversation: Conversation;
  messages: Message[];
}

export interface ReplyEvent {
  conversationId: string;
  text: string;
}

export interface ProgressEvent {
  conversationId: string;
  stage: number;
  label: string;
}

export interface Persona {
  id: string;
  name: string;
  tagline: string;
  behavior: string;
  skills: string[];
  coming: string[];
  installed: boolean;
}

export interface AiStatus {
  reachable: boolean;
  baseInstalled: boolean;
  personas: Persona[];
}

export interface InstallEvent {
  percent: number;
  label: string;
}
