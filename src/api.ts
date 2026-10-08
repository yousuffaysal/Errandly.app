import { invoke } from "@tauri-apps/api/core";
import type {
  AiStatus, Conversation, ConversationView, Grant, Preferences, Profile, Project, SettingsView, StorageReport, Task,
} from "./types";

export const PROGRESS_EVENT = "errandly://progress";
export const INSTALL_EVENT = "errandly://install";
export const REPLY_EVENT = "errandly://reply";
export const OAUTH_EVENT = "errandly://oauth";

export const api = {
  aiStatus: () => invoke<AiStatus>("ai_status"),
  installModels: () => invoke<void>("install_models"),
  warmUp: (persona: string) => invoke<void>("warm_up", { persona }),

  getSettings: () => invoke<SettingsView>("get_settings"),
  setActiveAccount: (accountId: string | null) => invoke<SettingsView>("set_active_account", { accountId }),
  saveProfile: (profile: Profile) => invoke<Profile>("save_profile", { profile }),
  savePreferences: (preferences: Preferences) => invoke<Preferences>("save_preferences", { preferences }),
  storageReport: () => invoke<StorageReport>("storage_report"),
  clearCache: () => invoke<void>("clear_cache"),
  listGrants: () => invoke<Grant[]>("list_grants"),
  revokeGrant: (grantId: string) => invoke<void>("revoke_grant", { grantId }),
  exportAllData: () => invoke<boolean>("export_all_data"),
  deleteAllConversations: () => invoke<void>("delete_all_conversations"),

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
  exportCard: (messageId: number) => invoke<boolean>("export_card", { messageId }),

  approveTask: (taskId: string) => invoke<Task>("approve_task", { taskId }),
  cancelTask: (taskId: string) => invoke<Task>("cancel_task", { taskId }),
  undoTask: (taskId: string) => invoke<Task>("undo_task", { taskId }),
  getTask: (taskId: string) => invoke<Task>("get_task", { taskId }),

  startOAuthListener: () => invoke<number>("start_oauth_listener"),
  openAuthUrl: (url: string) => invoke<void>("open_auth_url", { url }),

  sessionGet: (key: string) => invoke<string | null>("session_get", { key }),
  sessionSet: (key: string, value: string) => invoke<void>("session_set", { key, value }),
  sessionRemove: (key: string) => invoke<void>("session_remove", { key }),
};

/** Tauri rejects with the serialized AppError, which is a plain string. */
export const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
