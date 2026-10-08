import { useCallback, useEffect, useState, type ReactNode } from "react";
import type { User } from "@supabase/supabase-js";
import {
  Bot, Database, FolderLock, HardDrive, Info, LogIn, LogOut, Search, Settings as Gear, ShieldCheck, Sparkles, UserRound, X,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { checkForUpdate, installUpdate, type UpdateState } from "../updates";
import { api, errorText } from "../api";
import { accountsEnabled, supabase } from "../auth";
import { basename } from "../paths";
import type { AiStatus, Grant, Preferences, Profile, SettingsView, StorageReport } from "../types";

type Section = "general" | "account" | "personal" | "models" | "folders" | "privacy" | "storage" | "about";

const NAV: { group: string; items: { id: Section; label: string; icon: typeof Gear }[] }[] = [
  {
    group: "Settings",
    items: [
      { id: "general", label: "General", icon: Gear },
      { id: "account", label: "Account", icon: UserRound },
      { id: "personal", label: "Personalization", icon: Sparkles },
      { id: "models", label: "Models", icon: Bot },
    ],
  },
  {
    group: "This Mac",
    items: [
      { id: "folders", label: "Folders", icon: FolderLock },
      { id: "privacy", label: "Privacy and data", icon: ShieldCheck },
      { id: "storage", label: "Storage", icon: HardDrive },
      { id: "about", label: "About", icon: Info },
    ],
  },
];

export function SettingsDialog({ settings, ai, user, onClose, onChange, onSignIn, onRedoOnboarding, onAiChanged, onHistoryCleared }: {
  settings: SettingsView;
  ai: AiStatus | null;
  user: User | null;
  onClose: () => void;
  onChange: (s: SettingsView) => void;
  onSignIn: () => void;
  onRedoOnboarding: () => void;
  onAiChanged: () => void;
  onHistoryCleared: () => void;
}) {
  const [section, setSection] = useState<Section>("general");
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState("");
  const [update, setUpdate] = useState<UpdateState>({ kind: "idle" });
  const updateHint = {
    idle: "See whether a newer version is available.",
    checking: "Checking…",
    current: "You’re on the latest version.",
    available: "A new version is ready to install.",
    installing: "Installing…",
    error: "Couldn’t check right now. You may be offline, or no release has been published yet.",
  }[update.kind];

  useEffect(() => {
    const esc = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    document.addEventListener("keydown", esc);
    return () => document.removeEventListener("keydown", esc);
  }, [onClose]);

  const fail = (e: unknown) => setNotice(errorText(e));
  const savePrefs = async (patch: Partial<Preferences>) => {
    try {
      const preferences = await api.savePreferences({ ...settings.preferences, ...patch });
      onChange({ ...settings, preferences });
    } catch (e) {
      fail(e);
    }
  };
  const saveProfile = async (profile: Profile) => {
    try {
      onChange({ ...settings, profile: await api.saveProfile(profile) });
      setNotice("Saved.");
    } catch (e) {
      fail(e);
    }
  };

  const nav = NAV.map((g) => ({ ...g, items: g.items.filter((i) => i.label.toLowerCase().includes(query.toLowerCase())) })).filter(
    (g) => g.items.length,
  );

  return (
    <div className="ew-modal-scrim" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="ew-settings" role="dialog" aria-modal="true" aria-label="Settings">
        <nav className="ew-settings-nav">
          <label className="ew-settings-search">
            <Search size={14} />
            <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search" aria-label="Search settings" autoFocus />
          </label>
          {nav.map((g) => (
            <div key={g.group}>
              <p className="ew-settings-group">{g.group}</p>
              {g.items.map((i) => (
                <button key={i.id} className={section === i.id ? "is-active" : ""} onClick={() => { setSection(i.id); setNotice(""); }}>
                  <i.icon size={15} />
                  {i.label}
                </button>
              ))}
            </div>
          ))}
        </nav>

        <div className="ew-settings-body">
          <button className="ew-icon ew-settings-close" onClick={onClose} aria-label="Close settings">
            <X size={16} />
          </button>
          {notice && <p className="ew-settings-notice">{notice}</p>}

          {section === "general" && (
            <>
              <h2>Appearance</h2>
              <Row title="Text size" hint="How large everything in Errandly looks. ⌘+ and ⌘− also work.">
                <Segmented value={settings.preferences.textSize} options={[["small", "Small"], ["medium", "Medium"], ["large", "Large"]]} onChange={(v) => savePrefs({ textSize: v })} />
              </Row>
              <Row title="Conversation width" hint="Maximum width of the conversation and message box.">
                <Segmented value={settings.preferences.width} options={[["narrow", "Narrow"], ["medium", "Medium"], ["wide", "Wide"]]} onChange={(v) => savePrefs({ width: v })} />
              </Row>
              <Row title="Motion" hint="Reduce animation in streaming replies and other interface elements.">
                <Segmented value={settings.preferences.motion} options={[["system", "System"], ["reduced", "Reduced"]]} onChange={(v) => savePrefs({ motion: v })} />
              </Row>
              <h2>Conversations</h2>
              <Row title="Default model" hint="Used for new conversations. You can switch in any chat.">
                <select className="ew-settings-select" value={settings.preferences.defaultPersona} onChange={(e) => savePrefs({ defaultPersona: e.target.value })}>
                  {(ai?.personas ?? []).map((p) => (
                    <option key={p.id} value={p.id}>{p.name} · {p.tagline}</option>
                  ))}
                </select>
              </Row>
            </>
          )}

          {section === "account" && (
            <>
              <h2>Account</h2>
              {!accountsEnabled ? (
                <Row title="Accounts" hint="Errandly works fully without an account. Sign-in will appear here once this build is connected to Errandly accounts.">
                  <span className="ew-settings-pill">Not connected</span>
                </Row>
              ) : user ? (
                <>
                  <Row title={(user.user_metadata?.name as string) || "Signed in"} hint={user.email ?? ""}>
                    <button className="ew-settings-button" onClick={() => supabase?.auth.signOut()}>
                      <LogOut size={14} /> Sign out
                    </button>
                  </Row>
                  <Row title="Password" hint="We’ll email you a link to set a new password.">
                    <button
                      className="ew-settings-button"
                      onClick={() =>
                        user.email &&
                        supabase?.auth.resetPasswordForEmail(user.email).then(({ error }) => setNotice(error ? error.message : "Reset link sent."))
                      }
                    >
                      Send reset link
                    </button>
                  </Row>
                </>
              ) : (
                <Row title="Not signed in" hint="An account is optional. Your files, chats and models stay on this Mac either way.">
                  <button className="ew-settings-button primary" onClick={onSignIn}>
                    <LogIn size={14} /> Sign in or create account
                  </button>
                </Row>
              )}
            </>
          )}

          {section === "personal" && <PersonalSection profile={settings.profile} onSave={saveProfile} onRedo={onRedoOnboarding} />}

          {section === "models" && (
            <>
              <h2>Errandly’s models</h2>
              <p className="ew-settings-lead">All four run on this Mac and share one download, so switching between them costs nothing.</p>
              {(ai?.personas ?? []).map((p) => (
                <Row key={p.id} title={`${p.name} · ${p.tagline}`} hint={`${p.behavior} Skills: ${p.skills.join(", ")}.`}>
                  <span className={`ew-settings-pill ${p.installed ? "ok" : ""}`}>{p.installed ? "Ready" : "Not set up"}</span>
                </Row>
              ))}
              <Row title="Refresh models" hint="Re-teaches the four models their behaviour and skills. Takes a few seconds.">
                <button
                  className="ew-settings-button"
                  onClick={() =>
                    api.installModels().then(() => {
                      setNotice("Models refreshed.");
                      onAiChanged();
                    }, fail)
                  }
                >
                  Refresh
                </button>
              </Row>
            </>
          )}

          {section === "folders" && <FoldersSection onError={fail} />}

          {section === "privacy" && (
            <>
              <h2>Privacy and data</h2>
              <Row title="Where your data lives" hint="Files, chats, your profile and the models all stay on this Mac. Nothing is uploaded and there are no analytics.">
                <span className="ew-settings-pill ok">On this Mac</span>
              </Row>
              <Row
                title="Send crash reports"
                hint="Off by default. When on, crash details (the error and app version, never your chats or files) go to Foxmen Studio to help fix bugs. Crash logs are always kept on this Mac."
              >
                <Segmented
                  value={settings.preferences.crashReports ? "on" : "off"}
                  options={[["off", "Off"], ["on", "On"]]}
                  onChange={(v) => savePrefs({ crashReports: v === "on" })}
                />
              </Row>
              <Row
                title="Share anonymous usage statistics"
                hint="Off by default. When on, Errandly sends daily counts only (times opened, messages, tasks completed) under a random ID that isn’t linked to you. Never your name, email, chats or files."
              >
                <Segmented
                  value={settings.preferences.usageStats ? "on" : "off"}
                  options={[["off", "Off"], ["on", "On"]]}
                  onChange={(v) => savePrefs({ usageStats: v === "on" })}
                />
              </Row>
              <Row title="Crash logs" hint="~/Library/Logs/Errandly">
                <button className="ew-settings-button" onClick={() => invoke("open_crash_folder").catch(fail)}>
                  Show in Finder
                </button>
              </Row>
              <Row title="Export everything" hint="Your profile, preferences, projects and every conversation, as one JSON file.">
                <button className="ew-settings-button" onClick={() => api.exportAllData().then((ok) => ok && setNotice("Export saved."), fail)}>
                  Export
                </button>
              </Row>
              <DangerRow
                title="Delete all conversations"
                hint="Removes every conversation in every project. Files on your Mac are never touched."
                action="Delete all"
                onConfirm={() =>
                  api.deleteAllConversations().then(() => {
                    setNotice("All conversations deleted.");
                    onHistoryCleared();
                  }, fail)
                }
              />
              <DangerRow
                title="Forget my profile"
                hint="Clears what you told Errandly about yourself. You’ll be asked again next time."
                action="Forget"
                onConfirm={() =>
                  api.saveProfile({ name: "", role: "", work: "", helpWith: [], tone: "", language: "", notes: "", completed: false }).then((profile) => {
                    onChange({ ...settings, profile });
                    onClose();
                  }, fail)
                }
              />
            </>
          )}

          {section === "storage" && <StorageSection onError={fail} />}

          {section === "about" && (
            <>
              <h2>About Errandly</h2>
              <Row title="Version" hint="Beta. Built by Foxmen Studio.">
                <span className="ew-settings-pill">{settings.version}</span>
              </Row>
              <Row title="Updates" hint="Checks for new signed versions shortly after launch. Only the version number is sent.">
                <Segmented
                  value={settings.preferences.autoUpdate ? "on" : "off"}
                  options={[["on", "Automatic"], ["off", "Off"]]}
                  onChange={(v) => savePrefs({ autoUpdate: v === "on" })}
                />
              </Row>
              <Row title="Check now" hint={updateHint}>
                <button
                  className="ew-settings-button"
                  disabled={update.kind === "checking" || update.kind === "installing"}
                  onClick={async () => {
                    setUpdate({ kind: "checking" });
                    setUpdate(await checkForUpdate());
                  }}
                >
                  Check for updates
                </button>
                {update.kind === "available" && (
                  <button
                    className="ew-settings-button primary"
                    onClick={() => installUpdate(update.update, (percent) => setUpdate({ kind: "installing", percent }))}
                  >
                    Install {update.version}
                  </button>
                )}
              </Row>
              <Row title="Models" hint="Ario, Shadow, Suf 4 and Howen 2 are Errandly’s own on-device models. They run entirely on this Mac.">
                <span className="ew-settings-pill ok">On this Mac</span>
              </Row>
              <Row title="Acknowledgements" hint="Errandly includes open-source software. Licence notices are included with the app.">
                <span />
              </Row>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function Row({ title, hint, children }: { title: string; hint?: string; children: ReactNode }) {
  return (
    <div className="ew-settings-row">
      <div>
        <strong>{title}</strong>
        {hint && <p>{hint}</p>}
      </div>
      <div className="ew-settings-control">{children}</div>
    </div>
  );
}

function Segmented<T extends string>({ value, options, onChange }: { value: T; options: [T, string][]; onChange: (v: T) => void }) {
  return (
    <div className="ew-segmented" role="radiogroup">
      {options.map(([v, label]) => (
        <button key={v} role="radio" aria-checked={value === v} className={value === v ? "is-on" : ""} onClick={() => onChange(v)}>
          {label}
        </button>
      ))}
    </div>
  );
}

function DangerRow({ title, hint, action, onConfirm }: { title: string; hint: string; action: string; onConfirm: () => void }) {
  const [sure, setSure] = useState(false);
  return (
    <Row title={title} hint={hint}>
      {sure ? (
        <span className="ew-settings-confirm">
          <button className="ew-settings-button danger" onClick={() => { setSure(false); onConfirm(); }}>{action}</button>
          <button className="ew-settings-button" onClick={() => setSure(false)}>Cancel</button>
        </span>
      ) : (
        <button className="ew-settings-button" onClick={() => setSure(true)}>{action}</button>
      )}
    </Row>
  );
}

function PersonalSection({ profile, onSave, onRedo }: { profile: Profile; onSave: (p: Profile) => void; onRedo: () => void }) {
  const [draft, setDraft] = useState(profile);
  useEffect(() => setDraft(profile), [profile]);
  const field = (key: keyof Profile, label: string, hint: string, long = false) => (
    <label className="ew-settings-field">
      <span>{label}</span>
      {long ? (
        <textarea value={draft[key] as string} onChange={(e) => setDraft({ ...draft, [key]: e.target.value })} placeholder={hint} />
      ) : (
        <input value={draft[key] as string} onChange={(e) => setDraft({ ...draft, [key]: e.target.value })} placeholder={hint} />
      )}
    </label>
  );
  return (
    <>
      <h2>Personalization</h2>
      <p className="ew-settings-lead">What Errandly knows about you. Every model reads this before it replies or plans, so it can help personally. It never leaves this Mac.</p>
      <div className="ew-settings-form">
        {field("name", "What should Errandly call you?", "Your name")}
        {field("role", "What do you do?", "Student, business owner, researcher…")}
        {field("work", "What are you working on?", "Your thesis, a shop, client projects…")}
        <label className="ew-settings-field">
          <span>Where would help matter most?</span>
          <input
            value={draft.helpWith.join(", ")}
            onChange={(e) => setDraft({ ...draft, helpWith: e.target.value.split(",").map((s) => s.trimStart()) })}
            placeholder="Organizing files, invoices, study…"
          />
        </label>
        {field("tone", "How should Errandly talk to you?", "Short and direct, warm, formal…")}
        {field("language", "Preferred language", "English")}
        {field("notes", "Anything else Errandly should know", "Your city, hours, how you name files…", true)}
      </div>
      <div className="ew-settings-actions">
        <button className="ew-settings-button primary" onClick={() => onSave({ ...draft, helpWith: draft.helpWith.map((h) => h.trim()).filter(Boolean), completed: true })}>
          Save
        </button>
        <button className="ew-settings-button" onClick={onRedo}>Answer the questions again</button>
      </div>
    </>
  );
}

function FoldersSection({ onError }: { onError: (e: unknown) => void }) {
  const [grants, setGrants] = useState<Grant[] | null>(null);
  const load = useCallback(() => api.listGrants().then(setGrants, onError), [onError]);
  useEffect(() => {
    load();
  }, [load]);
  return (
    <>
      <h2>Folders</h2>
      <p className="ew-settings-lead">
        Errandly can only see folders you add to a conversation. Removing one here revokes access everywhere; the folder and its files stay exactly where they are.
      </p>
      {grants?.length === 0 && <p className="ew-settings-empty">No folders yet. Add one with the + button in any conversation.</p>}
      {grants?.map((g) => (
        <Row key={g.id} title={basename(g.path)} hint={g.path}>
          <button className="ew-settings-button" onClick={() => api.revokeGrant(g.id).then(load, onError)}>Remove access</button>
        </Row>
      ))}
    </>
  );
}

function StorageSection({ onError }: { onError: (e: unknown) => void }) {
  const [report, setReport] = useState<StorageReport | null>(null);
  const load = useCallback(() => api.storageReport().then(setReport, onError), [onError]);
  useEffect(() => {
    load();
  }, [load]);
  if (!report) return <h2>Storage</h2>;
  const rows: [string, string, number | null][] = [
    ["Conversations and settings", "Your local database", report.database],
    ["AI models", "One shared download for all four models", report.models],
    ["Temporary cache", "Safe to clear at any time", report.cache],
  ];
  return (
    <>
      <h2>Storage</h2>
      <p className="ew-settings-lead">Errandly uses your Mac’s own storage. Nothing is kept in the cloud.</p>
      {rows.map(([title, hint, bytes]) => (
        <Row key={title} title={title} hint={hint}>
          <span className="ew-settings-size">{bytes === null ? "—" : formatBytes(bytes)}</span>
        </Row>
      ))}
      <Row title="Available on this Mac" hint={report.available !== null && report.available < 5e9 ? "Running low. Errandly works best with at least 5 GB free." : report.dataDir}>
        <span className={`ew-settings-size ${report.available !== null && report.available < 5e9 ? "warn" : ""}`}>
          {report.available === null ? "—" : formatBytes(report.available)}
        </span>
      </Row>
      <Row title="Clear temporary cache" hint="Removes disposable files. Conversations, settings and models are kept.">
        <button className="ew-settings-button" onClick={() => api.clearCache().then(load, onError)}>
          <Database size={14} /> Clear cache
        </button>
      </Row>
    </>
  );
}

function formatBytes(n: number) {
  if (n >= 1e9) return `${(n / 1e9).toFixed(1)} GB`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(0)} MB`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(0)} KB`;
  return `${n} B`;
}
