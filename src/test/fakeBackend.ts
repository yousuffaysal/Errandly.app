import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Conversation, Message, Profile, ResultCard } from "../types";

/**
 * An in-memory stand-in for the Rust backend, speaking the same commands, so
 * smoke tests drive the real UI end to end without a model or a disk.
 */
export function fakeBackend(opts: { profileCompleted?: boolean; reply?: (text: string) => { text: string; card?: ResultCard } } = {}) {
  const calls: { cmd: string; args: Record<string, unknown> }[] = [];
  let profile: Profile = {
    name: opts.profileCompleted ? "Yusuf" : "",
    role: "",
    work: "",
    helpWith: [],
    tone: "",
    language: "",
    notes: "",
    completed: opts.profileCompleted ?? false,
  };
  const preferences = { defaultPersona: "ario", textSize: "medium", width: "medium", motion: "system", crashReports: false, autoUpdate: false, usageStats: false };
  const conv: Conversation = {
    id: "c1", projectId: "p1", title: "A fresh start", persona: "ario", grantId: null, folder: null, files: [],
    instructions: "", messageCount: 0, createdAt: "2026-10-08T10:00:00Z", updatedAt: "2026-10-08T10:00:00Z",
  };
  const messages: Message[] = [];
  let nextId = 1;
  const persona = (id: string, name: string) => ({
    id, name, tagline: "The organizer", behavior: "Tidy.", skills: ["Organize any folder"], coming: [], installed: true,
  });
  const view = () => ({ conversation: { ...conv, messageCount: messages.length }, messages: [...messages] });

  mockWindows("main");
  mockIPC(
    (cmd, args) => {
      calls.push({ cmd, args: (args ?? {}) as Record<string, unknown> });
      const a = (args ?? {}) as Record<string, never>;
      switch (cmd) {
        case "set_active_account":
        case "get_settings":
          return { profile, preferences, version: "0.1.0" };
        case "save_profile":
          profile = a.profile;
          return profile;
        case "save_preferences":
          return a.preferences;
        case "list_projects":
          return [{ id: "p1", name: "Personal", description: "", conversationCount: 1, isDefault: true, createdAt: "" }];
        case "list_conversations":
          return [conv];
        case "get_conversation":
        case "create_conversation":
          return view();
        case "set_persona":
          return conv;
        case "ai_status":
          return { reachable: true, baseInstalled: true, runtimeBundled: true, personas: [persona("ario", "Ario"), persona("shadow", "Shadow")] };
        case "send_message": {
          const now = new Date().toISOString();
          messages.push({ id: nextId++, role: "user", text: a.text, taskId: null, card: null, createdAt: now });
          const r = opts.reply?.(a.text) ?? { text: "Hello! I’m Ario." };
          messages.push({ id: nextId++, role: "assistant", text: r.text, taskId: null, card: r.card ?? null, createdAt: now });
          return view();
        }
        case "regenerate": {
          messages.pop();
          const r = opts.reply?.("again") ?? { text: "A second answer." };
          messages.push({ id: nextId++, role: "assistant", text: r.text, taskId: null, card: null, createdAt: new Date().toISOString() });
          return view();
        }
        case "attach_files":
          conv.files = ["/Users/x/Documents/Lecture 4.pdf"];
          return view();
        case "export_pdf":
          return true;
        case "storage_report":
          return { database: 2_000_000, models: 2_500_000_000, cache: 1_000, available: 16_000_000_000, dataDir: "~/x" };
        case "list_grants":
          return [];
        case "warm_up":
        case "log_crash":
          return null;
        case "unsent_crashes":
          return [];
        case "list_commands":
          return [
            { name: "organize", title: "Organize folder", hint: "Sort the attached folder", takesText: false, needsFolder: true },
            { name: "email", title: "Write an email", hint: "A ready-to-send email", takesText: true, needsFolder: false },
            { name: "explain", title: "Explain simply", hint: "A clear explanation", takesText: true, needsFolder: false },
          ];
        case "usage_pending":
          return { installId: "test", version: "0.0.0", days: [] };
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );
  return { calls };
}
