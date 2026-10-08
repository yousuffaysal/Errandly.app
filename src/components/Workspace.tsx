import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  ArrowUp, ArrowUpRight, BookOpen, ChevronDown, ChevronRight, Command, Download, Folder, FolderOpen,
  LayoutGrid, MessageSquare, PanelLeftClose, PanelRight, Plus, Search, ShieldCheck, Sparkles, Square, X,
} from "lucide-react";
import { api, errorText, PROGRESS_EVENT } from "../api";
import { basename } from "../paths";
import type { AiStatus, Conversation, ConversationView, Message, ProgressEvent } from "../types";
import { TaskCard } from "./TaskCard";

const suggestions = [
  { icon: Folder, title: "Bring a little order", text: "Organize a folder of scattered files", prompt: "Help me organize this folder by file type." },
  { icon: BookOpen, title: "Find the useful parts", text: "Turn long documents into clear notes", prompt: "Help me summarize my research documents." },
  { icon: LayoutGrid, title: "Make the numbers click", text: "Build a report from your spreadsheets", prompt: "Help me create an expense report." },
];
const STAGE_FALLBACK = ["Understanding your request", "Choosing the right folders", "Sorting files"];

export default function Workspace() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [view, setView] = useState<ConversationView | null>(null);
  const [input, setInput] = useState("");
  const [search, setSearch] = useState("");
  const [context, setContext] = useState(() => window.innerWidth > 1000);
  const [sidebar, setSidebar] = useState(false);
  const [thinking, setThinking] = useState<Record<string, ProgressEvent>>({});
  const [instructions, setInstructions] = useState("");
  const [saved, setSaved] = useState(true);
  const [notice, setNotice] = useState("");
  const [ai, setAi] = useState<AiStatus | null>(null);
  const [model, setModel] = useState("");
  const endRef = useRef<HTMLDivElement>(null);
  const activeId = useRef<string | null>(null);

  const conv = view?.conversation;
  const busy = conv ? Boolean(thinking[conv.id]) : false;
  const progress = conv ? thinking[conv.id] : undefined;

  const show = useCallback((v: ConversationView) => {
    if (v.conversation.id === activeId.current || activeId.current === null) {
      activeId.current = v.conversation.id;
      setView(v);
    }
  }, []);
  const refreshList = useCallback(() => api.listConversations().then(setConversations), []);
  const fail = useCallback((e: unknown) => setNotice(errorText(e)), []);

  const open = useCallback(
    async (id: string) => {
      activeId.current = id;
      const v = await api.getConversation(id);
      if (activeId.current === id) {
        setView(v);
        setInstructions(v.conversation.instructions);
        setSaved(true);
      }
    },
    [],
  );

  // Initial load: open the most recent conversation, or start one.
  useEffect(() => {
    (async () => {
      const list = await api.listConversations();
      if (list.length) {
        setConversations(list);
        await open(list[0].id);
      } else {
        const v = await api.createConversation();
        activeId.current = v.conversation.id;
        setView(v);
        await refreshList();
      }
    })().catch(fail);
  }, [open, refreshList, fail]);

  // Real planning progress from the Rust backend.
  useEffect(() => {
    const un = listen<ProgressEvent>(PROGRESS_EVENT, (e) =>
      setThinking((t) => (t[e.payload.conversationId] ? { ...t, [e.payload.conversationId]: e.payload } : t)),
    );
    return () => {
      un.then((f) => f());
    };
  }, []);

  const modelReady = Boolean(ai?.reachable && ai.models.includes(model));
  const checkAi = useCallback(async () => {
    const s = await api.aiStatus();
    setAi(s);
    setModel((m) => m || (s.models.includes(s.defaultModel) ? s.defaultModel : (s.models[0] ?? s.defaultModel)));
  }, []);
  useEffect(() => {
    checkAi();
    const t = setInterval(checkAi, modelReady ? 15000 : 4000);
    return () => clearInterval(t);
  }, [checkAi, modelReady]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [view?.messages.length, busy]);

  // Auto-save instructions shortly after typing stops.
  useEffect(() => {
    if (!conv || saved) return;
    const id = conv.id;
    const t = setTimeout(() => {
      api.setInstructions(id, instructions).then(() => setSaved(true), fail);
    }, 500);
    return () => clearTimeout(t);
  }, [instructions, saved, conv, fail]);

  async function select(id: string) {
    setInput("");
    setSidebar(false);
    await open(id).catch(fail);
  }

  async function newChat() {
    setSidebar(false);
    setInput("");
    if (view && view.messages.length === 0) return;
    try {
      const v = await api.createConversation();
      activeId.current = v.conversation.id;
      setView(v);
      setInstructions("");
      setSaved(true);
      await refreshList();
    } catch (e) {
      fail(e);
    }
  }

  async function send(value = input) {
    if (!conv || !value.trim() || busy) return;
    if (!modelReady) {
      setNotice(setupText(ai, model));
      return;
    }
    const id = conv.id;
    const text = value.trim();
    const optimistic: Message = { id: -Date.now(), role: "user", text, taskId: null, createdAt: new Date().toISOString() };
    setView((v) => (v && v.conversation.id === id ? { ...v, messages: [...v.messages, optimistic] } : v));
    setInput("");
    setThinking((t) => ({ ...t, [id]: { conversationId: id, stage: 0, label: STAGE_FALLBACK[0] } }));
    try {
      show(await api.sendMessage(id, text, model));
    } catch (e) {
      fail(e);
      if (activeId.current === id) await open(id).catch(() => {});
    } finally {
      setThinking(({ [id]: _, ...rest }) => rest);
      refreshList().catch(() => {});
    }
  }

  function stop() {
    if (conv) api.stopConversation(conv.id).catch(fail);
  }

  async function attach() {
    if (!conv) return;
    try {
      show(await api.attachFolder(conv.id));
      await refreshList();
    } catch (e) {
      fail(e);
    }
  }

  async function detach() {
    if (!conv) return;
    try {
      show(await api.detachFolder(conv.id));
      await refreshList();
    } catch (e) {
      fail(e);
    }
  }

  async function download() {
    if (!conv) return;
    try {
      if (await api.exportConversation(conv.id)) setNotice("Conversation saved.");
    } catch (e) {
      fail(e);
    }
  }

  const reload = useCallback(() => {
    if (activeId.current) open(activeId.current).catch(fail);
    refreshList().catch(() => {});
  }, [open, refreshList, fail]);

  const filtered = conversations.filter((c) => c.title.toLowerCase().includes(search.toLowerCase()));
  const folderCount = conv?.folder ? 1 : 0;

  return (
    <main id="main" className={`ew ${context ? "with-context" : ""}`}>
      <aside className={`ew-sidebar ${sidebar ? "is-open" : ""}`}>
        <div className="ew-brand">
          <span className="ew-mark">e</span>errandly<span className="ew-beta">BETA</span>
        </div>
        <button className="ew-new" onClick={newChat}>
          <Plus size={17} />
          New conversation<kbd>＋</kbd>
        </button>
        <label className="ew-search">
          <Search size={16} />
          <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search conversations" aria-label="Search conversations" />
        </label>
        <div className="ew-space-label">WORKSPACE</div>
        <button className="ew-personal" onClick={() => { setSearch(""); setSidebar(false); }}>
          <span className="ew-folder"><Folder size={16} /></span>
          Personal workspace
          <ChevronDown size={14} />
        </button>
        <div className="ew-history-title">
          Conversations<span>{conversations.length}</span>
        </div>
        <nav className="ew-history" aria-label="Chat history">
          {filtered.map((c) => (
            <button key={c.id} className={conv?.id === c.id ? "active" : ""} onClick={() => select(c.id)}>
              <MessageSquare size={15} />
              <span>{c.title}</span>
              {conv?.id === c.id && <span className="ew-selected-dot" />}
            </button>
          ))}
          {filtered.length === 0 && <p className="ew-empty-search">No conversations found.</p>}
        </nav>
        <div className="ew-sidebar-bottom">
          <div className="ew-private">
            <ShieldCheck size={19} />
            <div>
              A space that’s yours<small>Chat history saved on this Mac</small>
            </div>
          </div>
          <div className="ew-profile">
            <span className="ew-avatar">Y</span>
            <div>
              Your workspace<small>Personal · On this Mac</small>
            </div>
          </div>
        </div>
      </aside>
      {sidebar && <button className="ew-scrim" aria-label="Close navigation" onClick={() => setSidebar(false)} />}

      <section className="ew-main">
        <div className="ew-topbar">
          <button className="ew-mobile-menu ew-icon" aria-label="Open navigation" onClick={() => setSidebar(true)}>
            <PanelLeftClose size={18} />
          </button>
          <span className="ew-breadcrumb">
            Personal
            <ChevronRight size={13} />
          </span>
          <span className="ew-chat-title">{conv?.title}</span>
          <div className="ew-top-actions">
            <span className={`ew-preview-dot ${modelReady ? "" : ai?.reachable ? "is-warn" : "is-off"}`} />{" "}
            <span className="ew-preview-label">
              {modelReady ? `Local AI · ${model}` : ai?.reachable ? `${model} not downloaded` : "Local AI offline"}
            </span>
            <button className="ew-icon" onClick={download} title="Export conversation" aria-label="Export conversation">
              <Download size={17} />
            </button>
            <button
              className={`ew-icon ${context ? "selected" : ""}`}
              onClick={() => setContext(!context)}
              aria-label="Toggle conversation context"
              aria-expanded={context}
            >
              <PanelRight size={18} />
            </button>
          </div>
        </div>

        <div className="ew-scroll">
          <div className="ew-conversation">
            {!view ? null : view.messages.length === 0 && !busy ? (
              <div className="ew-welcome">
                <span className="ew-flower">✳</span>
                <p>A LITTLE SPACE TO THINK</p>
                <h1>What’s on your mind?</h1>
                <div>
                  From a loose thought to a finished task.
                  <br />
                  Let’s make room for your best work.
                </div>
                <div className="ew-suggestions">
                  {suggestions.map((s) => (
                    <button key={s.title} onClick={() => send(s.prompt)}>
                      <s.icon size={21} />
                      <strong>{s.title}</strong>
                      <span>{s.text}</span>
                      <ArrowUpRight size={15} />
                    </button>
                  ))}
                </div>
              </div>
            ) : (
              <>
                {view.messages.map((m, i) => (
                  <MessageItem key={m.id} message={m} previous={view.messages[i - 1]} onChanged={reload} onError={fail} />
                ))}
              </>
            )}
            {busy && progress && (
              <div className="ew-thinking" role="status" aria-live="polite">
                <div className="ew-orbit">
                  <i />
                  <i />
                  <i />
                  <span>✳</span>
                </div>
                <div>
                  <strong>
                    {progress.label}
                    <span className="ew-dots"><i /><i /><i /></span>
                  </strong>
                  <p>A little thought. A little less on your plate.</p>
                  <div className="ew-progress">
                    {STAGE_FALLBACK.map((s, i) => (
                      <span className={i <= progress.stage ? "on" : ""} key={s} />
                    ))}
                  </div>
                </div>
                <span className="ew-simulation">ON DEVICE</span>
              </div>
            )}
            <div ref={endRef} />
          </div>
        </div>

        <div className="ew-compose-wrap">
          {notice && (
            <div className="ew-notice" role="status">
              {notice}
              <button onClick={() => setNotice("")} aria-label="Dismiss notification">
                <X size={13} />
              </button>
            </div>
          )}
          {!notice && ai && !modelReady && <div className="ew-notice ew-setup">{setupText(ai, model)}</div>}
          <form
            className={`ew-compose ${busy ? "is-thinking" : ""}`}
            onSubmit={(e) => {
              e.preventDefault();
              send();
            }}
          >
            <textarea
              aria-label="Message Errandly"
              placeholder="A question, a task, a little less on your plate…"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                  e.preventDefault();
                  send();
                }
              }}
            />
            <div className="ew-compose-tools">
              <button type="button" className="ew-icon" onClick={attach} aria-label="Add a folder to this conversation" title="Add a folder">
                <Plus size={20} />
              </button>
              <span className="ew-compose-model">
                <Sparkles size={14} />
                Errandly
                {ai && ai.models.length > 1 ? (
                  <select className="ew-model-tag ew-model-select" value={model} onChange={(e) => setModel(e.target.value)} aria-label="Local model">
                    {ai.models.map((m) => <option key={m}>{m}</option>)}
                  </select>
                ) : (
                  <span className="ew-model-tag">{model || "Local"}</span>
                )}
              </span>
              <span className="ew-context-count" title={conv?.folder ?? undefined}>
                <FolderOpen size={12} />
                {conv?.folder ? basename(conv.folder) : "No folder"}
              </span>
              {busy ? (
                <button type="button" className="ew-send" onClick={stop} aria-label="Stop">
                  <Square size={15} fill="currentColor" />
                </button>
              ) : (
                <button className="ew-send" disabled={!input.trim()} aria-label="Send message">
                  <ArrowUp size={19} />
                </button>
              )}
            </div>
          </form>
          <p className="ew-disclaimer">
            Runs on your Mac. Nothing changes until you approve.<span> Made for a calmer kind of work.</span>
          </p>
        </div>
      </section>

      {context && conv && (
        <aside className="ew-context">
          <div className="ew-context-heading">
            <span>Conversation context</span>
            <button className="ew-icon" onClick={() => setContext(false)} aria-label="Close context">
              <X size={16} />
            </button>
          </div>
          <div className="ew-context-body">
            <div className="ew-context-intro">
              <span className="ew-context-emblem"><Command size={21} /></span>
              <h2>The whole picture.</h2>
              <p>
                Everything this conversation
                <br />
                can draw from, in one place.
              </p>
            </div>
            <div className="ew-context-section">
              <h3>
                Folder<span>{folderCount}</span>
                <button className="ew-icon" onClick={attach} aria-label="Add a folder">
                  <Plus size={15} />
                </button>
              </h3>
              {conv.folder && (
                <div className="ew-file">
                  <span><Folder size={18} /></span>
                  <div title={conv.folder}>
                    {basename(conv.folder)}
                    <small>{conv.folder}</small>
                  </div>
                  <button aria-label={`Remove ${basename(conv.folder)}`} onClick={detach}>
                    <X size={12} />
                  </button>
                </div>
              )}
              <button className="ew-add-files" onClick={attach}>
                <FolderOpen size={14} />
                {conv.folder ? "Use a different folder" : "Add a folder to this chat"}
              </button>
            </div>
            <div className="ew-context-section">
              <h3>
                Instructions<span className="ew-auto-save">{saved ? "Auto-saved" : "Saving…"}</span>
              </h3>
              <textarea
                aria-label="Conversation instructions"
                value={instructions}
                placeholder="How would you like Errandly to help?"
                onChange={(e) => {
                  setInstructions(e.target.value);
                  setSaved(false);
                }}
              />
              <p className="ew-field-help">A little guidance goes a long way.</p>
            </div>
            <div className="ew-context-section">
              <h3>Conversation memory</h3>
              <div className="ew-memory">
                <span className="ew-memory-dot" />
                <div>
                  Keeping the thread<small>{view.messages.length} messages in this conversation</small>
                </div>
              </div>
              <p className="ew-memory-note">Your messages, instructions and folder stay together when you return to this chat.</p>
            </div>
            <button className="ew-export" onClick={download}>
              <Download size={14} />
              Export conversation
              <ArrowUpRight size={13} />
            </button>
          </div>
          <div className="ew-context-footer">
            <ShieldCheck size={14} />
            <span>
              Stored on this Mac.
              <br />
              You’re always in control.
            </span>
          </div>
        </aside>
      )}
    </main>
  );
}

function MessageItem({ message: m, previous, onChanged, onError }: {
  message: Message;
  previous?: Message;
  onChanged: () => void;
  onError: (e: unknown) => void;
}) {
  const day = dayLabel(m.createdAt);
  return (
    <>
      {(!previous || dayLabel(previous.createdAt) !== day) && (
        <div className="ew-day">
          <span />
          {day}
          <span />
        </div>
      )}
      <article className={`ew-message ew-${m.role}`}>
        <div className="ew-message-label">
          {m.role === "user" ? <span className="ew-small-avatar">Y</span> : <span className="ew-small-mark">✳</span>}
          <strong>{m.role === "user" ? "You" : "Errandly"}</strong>
          <small>{m.role === "user" ? "" : "Your thinking partner"}</small>
        </div>
        <div className="ew-message-text">{m.text}</div>
        {m.taskId && <TaskCard taskId={m.taskId} onChanged={onChanged} onError={onError} />}
      </article>
    </>
  );
}

function dayLabel(iso: string) {
  const d = new Date(iso);
  const today = new Date();
  const yesterday = new Date(today.getTime() - 86_400_000);
  if (d.toDateString() === today.toDateString()) return "TODAY";
  if (d.toDateString() === yesterday.toDateString()) return "YESTERDAY";
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" }).toUpperCase();
}

function setupText(ai: AiStatus | null, model: string) {
  if (!ai?.reachable) return "Local AI isn’t running. Start it with “brew services start ollama”.";
  return `Download the local model first: “ollama pull ${model}” (about 2 GB).`;
}
