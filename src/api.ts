import { invoke } from "@tauri-apps/api/core";
import type { AiStatus, Conversation, ConversationView, Task } from "./types";

export const PROGRESS_EVENT = "errandly://progress";

export const api = {
  aiStatus: () => invoke<AiStatus>("ai_status"),

  listConversations: () => invoke<Conversation[]>("list_conversations"),
  createConversation: () => invoke<ConversationView>("create_conversation"),
  getConversation: (conversationId: string) => invoke<ConversationView>("get_conversation", { conversationId }),
  setInstructions: (conversationId: string, instructions: string) =>
    invoke<void>("set_instructions", { conversationId, instructions }),
  attachFolder: (conversationId: string) => invoke<ConversationView>("attach_folder", { conversationId }),
  detachFolder: (conversationId: string) => invoke<ConversationView>("detach_folder", { conversationId }),
  sendMessage: (conversationId: string, text: string, model: string) =>
    invoke<ConversationView>("send_message", { conversationId, text, model }),
  stopConversation: (conversationId: string) => invoke<boolean>("stop_conversation", { conversationId }),
  exportConversation: (conversationId: string) => invoke<boolean>("export_conversation", { conversationId }),

  approveTask: (taskId: string) => invoke<Task>("approve_task", { taskId }),
  cancelTask: (taskId: string) => invoke<Task>("cancel_task", { taskId }),
  undoTask: (taskId: string) => invoke<Task>("undo_task", { taskId }),
  getTask: (taskId: string) => invoke<Task>("get_task", { taskId }),
};

/** Tauri rejects with the serialized AppError, which is a plain string. */
export const errorText = (e: unknown) => (typeof e === "string" ? e : String(e));
