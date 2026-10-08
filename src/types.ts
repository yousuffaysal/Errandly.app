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
  plan: PlanMeta | null;
  error: string | null;
  createdAt: string;
  completedAt: string | null;
  undoneAt: string | null;
  steps: Step[];
}

export interface Grant {
  id: string;
  path: string;
  createdAt: string;
}

export interface AiStatus {
  reachable: boolean;
  models: string[];
  defaultModel: string;
}
