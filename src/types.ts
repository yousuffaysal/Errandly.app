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
  isDefault: boolean;
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
  card: ResultCard | null;
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
  runtimeBundled: boolean;
}

export interface InstallEvent {
  percent: number;
  label: string;
}

export interface Profile {
  name: string;
  role: string;
  work: string;
  helpWith: string[];
  tone: string;
  language: string;
  notes: string;
  completed: boolean;
}

export type TextSize = "small" | "medium" | "large";
export type Width = "narrow" | "medium" | "wide";
export type Motion = "system" | "reduced";

export interface Preferences {
  defaultPersona: string;
  textSize: TextSize;
  width: Width;
  motion: Motion;
  crashReports: boolean;
  autoUpdate: boolean;
  usageStats: boolean;
}

export interface SettingsView {
  profile: Profile;
  preferences: Preferences;
  version: string;
}

export interface StorageReport {
  database: number;
  models: number | null;
  cache: number;
  available: number | null;
  dataDir: string;
}

export interface Grant {
  id: string;
  path: string;
  createdAt: string;
}

export interface DocSummary {
  name: string;
  summary: string;
  keyPoints: string[];
  chars: number;
  truncated: boolean;
  flagged: boolean;
}

export interface DocumentsCard {
  type: "documents";
  folder: string;
  documents: DocSummary[];
  overview: string | null;
  unreadable: [string, string][];
}

export interface SheetGroup {
  name: string;
  total: number;
  rows: number;
  share: number;
}

export interface SpreadsheetCard {
  type: "spreadsheet";
  file: string;
  sheet: string | null;
  title: string;
  insights: string[];
  rejectedInsights: number;
  analysis: {
    rows: number;
    valueColumn: string | null;
    groupColumn: string | null;
    total: number | null;
    groups: SheetGroup[];
    otherGroups: number;
    skippedRows: number;
    numeric: { name: string; count: number; sum: number; mean: number; min: number; max: number }[];
  };
}

export type ResultCard = DocumentsCard | SpreadsheetCard;
