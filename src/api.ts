import { invoke } from "@tauri-apps/api/core";
import type { AiStatus, Conversation, ConversationView, Project, Task } from "./types";

export const PROGRESS_EVENT = "errandly://progress";
export const INSTALL_EVENT = "errandly://install";

export const api = {
  aiStatus: () => invoke<AiStatus>("ai_status"),
  installModels: () => invoke<void>("install_models"),

  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string, description = "") => invoke<Project>("create_project", { name, description }),
  updateProject: (projectId: string, name: string, description = "") =>
    invoke<Project>("update_project", { projectId, name, description }),
  deleteProject: (projectId: string) => invoke<void>("delete_project", { projectId }),

  listConversations: (projectId: string) => invoke<Conversation[]>("list_conversations", { projectId }),
  createConversation: (projectId: string, persona: string) =>
    invoke<ConversationView>("create_conversation", { projectId, persona }),
  getConversation: (conversationId: string) => invoke<ConversationView>("get_conversation", { conversationId }),
  renameConversation: (conversationId: string, title: string) =>
    invoke<Conversation>("rename_conversation", { conversationId, title }),
  deleteConversation: (conversationId: string) => invoke<void>("delete_conversation", { conversationId }),
  setPersona: (conversationId: string, persona: string) => invoke<Conversation>("set_persona", { conversationId, persona }),
  setInstructions: (conversationId: string, instructions: string) =>
    invoke<void>("set_instructions", { conversationId, instructions }),
  attachFolder: (conversationId: string) => invoke<ConversationView>("attach_folder", { conversationId }),
  detachFolder: (conversationId: string) => invoke<ConversationView>("detach_folder", { conversationId }),
  sendMessage: (conversationId: string, text: string) => invoke<ConversationView>("send_message", { conversationId, text }),
  stopConversation: (conversationId: string) => invoke<boolean>("stop_conversation", { conversationId }),
  exportConversation: (conversationId: string) => invoke<boolean>("export_conversation", { conversationId }),

  approveTask: (taskId: string) => invoke<Task>("approve_task", { taskId }),
  cancelTask: (taskId: string) => invoke<Task>("cancel_task", { taskId }),
  undoTask: (taskId: string) => invoke<Task>("undo_task", { taskId }),
  getTask: (taskId: string) => invoke<Task>("get_task", { taskId }),

  secureGet: (key: string) => invoke<string | null>("secure_get", { key }),
  secureSet: (key: string, value: string) => invoke<void>("secure_set", { key, value }),
  secureRemove: (key: string) => invoke<void>("secure_remove", { key }),
};

/** Tauri rejects with the serialized AppError, which is a plain string. */
export const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
