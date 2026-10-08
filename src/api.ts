import { invoke } from "@tauri-apps/api/core";
import type { AiStatus, Grant, Task } from "./types";

export const api = {
  aiStatus: () => invoke<AiStatus>("ai_status"),
  pickFolder: () => invoke<Grant | null>("pick_and_grant_folder"),
  listGrants: () => invoke<Grant[]>("list_grants"),
  revokeGrant: (grantId: string) => invoke<void>("revoke_grant", { grantId }),
  planTask: (instruction: string, grantId: string, model: string) =>
    invoke<Task>("plan_task", { instruction, grantId, model }),
  approveTask: (taskId: string) => invoke<Task>("approve_task", { taskId }),
  cancelTask: (taskId: string) => invoke<Task>("cancel_task", { taskId }),
  undoTask: (taskId: string) => invoke<Task>("undo_task", { taskId }),
  getTask: (taskId: string) => invoke<Task>("get_task", { taskId }),
  listTasks: () => invoke<Task[]>("list_tasks"),
};

/** Tauri rejects with the serialized AppError, which is a plain string. */
export const errorText = (e: unknown) => (typeof e === "string" ? e : String(e));
