import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, errorText, PROGRESS_EVENT, REPLY_EVENT } from "../api";
import type { AiStatus, Conversation, ConversationView, Message, Profile, ProgressEvent, Project, ReplyEvent, SettingsView } from "../types";
import { accountName, AuthDialog, useUser } from "./Account";
import { Onboarding } from "./Onboarding";
import { SettingsDialog } from "./Settings";
import { setCrashSending } from "../crash";
import { setUsageSharing } from "../usage";
import { Composer } from "./workspace/Composer";
import { ContextPanel } from "./workspace/ContextPanel";
import { MessageList, STAGES } from "./workspace/MessageList";
import { Sidebar } from "./workspace/Sidebar";
import { TopBar } from "./workspace/TopBar";
import { UpdateBanner } from "./UpdateBanner";

/** Remembers the last project per account, as a convenience only. */
const lastProject = {
  key: (owner: string) => `errandly-last-project:${owner}`,
  get(owner: string) {
    try {
      return localStorage.getItem(this.key(owner));
    } catch {
      return null;
    }
  },
  set(owner: string, id: string) {
    try {
      localStorage.setItem(this.key(owner), id);
    } catch {
      // Not essential.
    }
  },
};

/** State and actions for the main window; the pieces live in ./workspace. */
export default function Workspace() {
  const user = useUser();
  const accountId = user === undefined ? undefined : (user?.id ?? null);
  const owner = accountId ?? "local";

  const [settings, setSettings] = useState<SettingsView | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectId, setProjectId] = useState<string | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [view, setView] = useState<ConversationView | null>(null);
  const [thinking, setThinking] = useState<Record<string, ProgressEvent>>({});
  const [streaming, setStreaming] = useState<Record<string, string>>({});
  const [instructions, setInstructions] = useState("");
  const [saved, setSaved] = useState(true);
  const [notice, setNotice] = useState("");
  const [ai, setAi] = useState<AiStatus | null>(null);
  const [context, setContext] = useState(() => window.innerWidth > 1000);
  const [sidebar, setSidebar] = useState(false);
  const [authOpen, setAuthOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [historyVersion, setHistoryVersion] = useState(0);
  const endRef = useRef<HTMLDivElement>(null);
  const activeId = useRef<string | null>(null);

  const conv = view?.conversation;
  const busy = conv ? Boolean(thinking[conv.id]) : false;
  const persona = ai?.personas.find((p) => p.id === conv?.persona) ?? ai?.personas[0];
  const modelReady = Boolean(ai?.reachable && persona?.installed);
  const defaultPersona = settings?.preferences.defaultPersona ?? "ario";
  const onboarding = settings !== null && !settings.profile.completed;
  const fail = useCallback((e: unknown) => setNotice(errorText(e)), []);

  // Crash reports leave this Mac only if the user opted in.
  const crashReports = settings?.preferences.crashReports ?? false;
  useEffect(() => setCrashSending(crashReports), [crashReports]);
  const usageStats = settings?.preferences.usageStats ?? false;
  useEffect(() => setUsageSharing(usageStats), [usageStats]);

  // Once we know who is signed in (or that nobody is), switch the backend to
  // that account first, then load its profile and projects.
  useEffect(() => {
    if (accountId === undefined) return;
    let live = true;
    (async () => {
      const s = await api.setActiveAccount(accountId);
      const ps = await api.listProjects();
      if (!live) return;
      setSettings(s);
      setProjects(ps);
      const remembered = lastProject.get(accountId ?? "local");
      setProjectId(ps.some((p) => p.id === remembered) ? remembered : (ps[0]?.id ?? null));
    })().catch(fail);
    return () => {
      live = false;
    };
  }, [accountId, fail]);

  const open = useCallback(async (id: string) => {
    activeId.current = id;
    const v = await api.getConversation(id);
    if (activeId.current === id) {
      setView(v);
      setInstructions(v.conversation.instructions);
      setSaved(true);
    }
  }, []);

  const show = useCallback((v: ConversationView) => {
    if (v.conversation.id === activeId.current) setView(v);
  }, []);

  const startConversation = useCallback(async (pid: string, personaId: string) => {
    const v = await api.createConversation(pid, personaId);
    activeId.current = v.conversation.id;
    setView(v);
    setInstructions("");
    setSaved(true);
  }, []);

  const refreshList = useCallback(async () => {
    if (!projectId) return;
    setConversations(await api.listConversations(projectId));
    setProjects(await api.listProjects());
  }, [projectId]);

  // Load the active project: open its latest conversation, or start one.
  useEffect(() => {
    if (!projectId || !settings) return;
    let live = true;
    lastProject.set(owner, projectId);
    (async () => {
      const list = await api.listConversations(projectId);
      if (!live) return;
      setConversations(list);
      if (list.length) await open(list[0].id);
      else {
        await startConversation(projectId, settings.preferences.defaultPersona);
        if (live) setConversations(await api.listConversations(projectId));
      }
    })().catch(fail);
    return () => {
      live = false;
    };
    // `settings` only supplies the default model here; it is not a trigger.
  }, [projectId, owner, historyVersion, open, startConversation, fail]);

  // Planning progress and streamed replies from the Rust backend.
  useEffect(() => {
    const unProgress = listen<ProgressEvent>(PROGRESS_EVENT, (e) =>
      setThinking((t) => (t[e.payload.conversationId] ? { ...t, [e.payload.conversationId]: e.payload } : t)),
    );
    const unReply = listen<ReplyEvent>(REPLY_EVENT, (e) =>
      setStreaming((s) => ({ ...s, [e.payload.conversationId]: e.payload.text })),
    );
    return () => {
      unProgress.then((f) => f());
      unReply.then((f) => f());
    };
  }, []);

  const checkAi = useCallback(() => api.aiStatus().then(setAi).catch(() => {}), []);
  useEffect(() => {
    checkAi();
    const t = setInterval(checkAi, modelReady ? 15000 : 4000);
    return () => clearInterval(t);
  }, [checkAi, modelReady]);

  // Load the conversation's model into memory ahead of the first message.
  const personaId = conv?.persona;
  useEffect(() => {
    if (modelReady && personaId) api.warmUp(personaId).catch(() => {});
  }, [modelReady, personaId]);

  const live = conv ? streaming[conv.id] : undefined;
  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [view?.messages.length, busy, live]);

  // Auto-save instructions shortly after typing stops.
  useEffect(() => {
    if (!conv || saved) return;
    const id = conv.id;
    const t = setTimeout(() => api.setInstructions(id, instructions).then(() => setSaved(true), fail), 500);
    return () => clearTimeout(t);
  }, [instructions, saved, conv, fail]);

  // ⌘, opens Settings, as in other Mac apps.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === ",") {
        e.preventDefault();
        setSettingsOpen(true);
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  const attempt = async <T,>(fn: () => Promise<T>): Promise<T | undefined> => {
    try {
      return await fn();
    } catch (e) {
      fail(e);
      return undefined;
    }
  };

  async function send(value: string) {
    if (!conv || !value.trim() || busy) return;
    if (!modelReady) {
      setNotice("Set up Errandly’s models first. It only takes one download.");
      return;
    }
    const id = conv.id;
    const text = value.trim();
    const optimistic: Message = { id: -Date.now(), role: "user", text, taskId: null, card: null, createdAt: new Date().toISOString() };
    setView((v) => (v && v.conversation.id === id ? { ...v, messages: [...v.messages, optimistic] } : v));
    setThinking((t) => ({ ...t, [id]: { conversationId: id, stage: 0, label: STAGES[0] } }));
    try {
      show(await api.sendMessage(id, text));
    } catch (e) {
      fail(e);
      if (activeId.current === id) await open(id).catch(() => {});
    } finally {
      setThinking(({ [id]: _, ...rest }) => rest);
      setStreaming(({ [id]: _, ...rest }) => rest);
      refreshList().catch(() => {});
    }
  }

  const withConv = (fn: (id: string) => Promise<ConversationView>) => () =>
    conv &&
    attempt(async () => {
      show(await fn(conv.id));
      await refreshList();
    });

  async function choosePersona(id: string) {
    if (!conv) return;
    const c = await attempt(() => api.setPersona(conv.id, id));
    if (c) setView((v) => (v && v.conversation.id === c.id ? { ...v, conversation: c } : v));
  }

  async function newChat() {
    setSidebar(false);
    if (!projectId || (view && view.messages.length === 0)) return;
    await attempt(async () => {
      await startConversation(projectId, defaultPersona);
      await refreshList();
    });
  }

  async function finishOnboarding(profile: Profile, personaId: string, shareUsage: boolean) {
    if (!settings) return;
    await attempt(async () => {
      const saved = await api.saveProfile(profile);
      const preferences = await api.savePreferences({ ...settings.preferences, defaultPersona: personaId, usageStats: shareUsage });
      setSettings({ ...settings, profile: saved, preferences });
      if (conv && view?.messages.length === 0) await choosePersona(personaId);
    });
  }

  const reload = useCallback(() => {
    if (activeId.current) open(activeId.current).catch(fail);
    refreshList().catch(() => {});
  }, [open, refreshList, fail]);

  const project = projects.find((p) => p.id === projectId);
  const status = modelReady
    ? { tone: "" as const, label: `On-device · ${persona?.name}` }
    : ai?.reachable
      ? { tone: "is-warn" as const, label: "Models not set up" }
      : { tone: "is-off" as const, label: "Local AI offline" };

  return (
    <main
      id="main"
      className={`ew ${context && !onboarding ? "with-context" : ""} ew-width-${settings?.preferences.width ?? "medium"} ${
        settings?.preferences.motion === "reduced" ? "ew-reduced" : ""
      }`}
    >
      <Sidebar
        open={sidebar}
        projects={projects}
        projectId={projectId ?? ""}
        conversations={conversations}
        activeId={conv?.id}
        user={user ?? null}
        profileName={settings?.profile.name ?? ""}
        onNewChat={newChat}
        onSelectProject={(id) => {
          setSidebar(false);
          setProjectId(id);
        }}
        onCreateProject={async (name) => {
          const p = await attempt(() => api.createProject(name));
          if (p) {
            setProjects(await api.listProjects());
            setProjectId(p.id);
          }
        }}
        onRenameProject={async (id, name) => {
          if (await attempt(() => api.updateProject(id, name))) setProjects(await api.listProjects());
        }}
        onDeleteProject={async (id) => {
          await attempt(async () => {
            await api.deleteProject(id);
            const ps = await api.listProjects();
            setProjects(ps);
            if (id === projectId) setProjectId(ps[0]?.id ?? null);
          });
        }}
        onSelect={(id) => {
          setSidebar(false);
          open(id).catch(fail);
        }}
        onRename={async (id, title) => {
          const c = await attempt(() => api.renameConversation(id, title));
          if (!c) return false;
          setView((v) => (v && v.conversation.id === c.id ? { ...v, conversation: c } : v));
          await refreshList();
          return true;
        }}
        onDelete={async (id) => {
          await attempt(async () => {
            await api.deleteConversation(id);
            if (!projectId) return;
            const list = await api.listConversations(projectId);
            setConversations(list);
            setProjects(await api.listProjects());
            if (id === conv?.id) {
              if (list.length) await open(list[0].id);
              else await startConversation(projectId, defaultPersona);
            }
          });
        }}
        onSignIn={() => setAuthOpen(true)}
        onSettings={() => setSettingsOpen(true)}
      />
      {sidebar && <button className="ew-scrim" aria-label="Close navigation" onClick={() => setSidebar(false)} />}

      <section className="ew-main">
        {onboarding ? (
          <Onboarding
            key={owner}
            initialName={settings?.profile.name || accountName(user)}
            personas={ai?.personas ?? []}
            onFinish={finishOnboarding}
          />
        ) : (
          <>
            <UpdateBanner enabled={settings?.preferences.autoUpdate ?? false} />
            <TopBar
              projectName={project?.name ?? "Personal"}
              title={conv?.title ?? ""}
              status={status}
              onMenu={() => setSidebar(true)}
              onExport={() => conv && attempt(async () => (await api.exportConversation(conv.id)) && setNotice("Conversation saved."))}
              contextOpen={context}
              onToggleContext={() => setContext(!context)}
            />
            <MessageList
              ref={endRef}
              messages={view?.messages ?? null}
              personaName={persona?.name ?? "Errandly"}
              initial={(settings?.profile.name || "Y").charAt(0).toUpperCase()}
              busy={busy}
              progress={conv ? thinking[conv.id] : undefined}
              live={live}
              onSuggestion={send}
              onChanged={reload}
              onError={fail}
            />
            <Composer
              conv={conv}
              ai={ai}
              modelReady={modelReady}
              busy={busy}
              notice={notice}
              onDismissNotice={() => setNotice("")}
              onSend={send}
              onStop={() => conv && api.stopConversation(conv.id).catch(fail)}
              onAttach={withConv(api.attachFolder)}
              onPersona={choosePersona}
              onModelsReady={checkAi}
            />
          </>
        )}
      </section>

      {context && conv && view && !onboarding && (
        <ContextPanel
          conv={conv}
          persona={persona}
          messageCount={view.messages.length}
          instructions={instructions}
          saved={saved}
          onInstructions={(text) => {
            setInstructions(text);
            setSaved(false);
          }}
          onAttach={withConv(api.attachFolder)}
          onDetach={withConv(api.detachFolder)}
          onExport={() => attempt(async () => (await api.exportConversation(conv.id)) && setNotice("Conversation saved."))}
          onClose={() => setContext(false)}
        />
      )}

      {authOpen && <AuthDialog onClose={() => setAuthOpen(false)} />}
      {settingsOpen && settings && (
        <SettingsDialog
          settings={settings}
          ai={ai}
          user={user ?? null}
          onClose={() => setSettingsOpen(false)}
          onChange={setSettings}
          onSignIn={() => {
            setSettingsOpen(false);
            setAuthOpen(true);
          }}
          onRedoOnboarding={() => {
            setSettingsOpen(false);
            setSettings({ ...settings, profile: { ...settings.profile, completed: false } });
          }}
          onAiChanged={checkAi}
          onHistoryCleared={() => setHistoryVersion((v) => v + 1)}
        />
      )}
    </main>
  );
}
