import { useCallback, useEffect, useState } from "react";
import { api, errorText } from "./api";
import { basename } from "./paths";
import { TaskPanel, StatusBadge } from "./components/TaskPanel";
import type { AiStatus, Grant, Task } from "./types";
import "./styles.css";

const EXAMPLES = [
  "Organize this folder by file type",
  "Sort these into Business, University, Personal and Images",
  "Separate installers and archives from documents",
];

export default function App() {
  const [ai, setAi] = useState<AiStatus | null>(null);
  const [model, setModel] = useState("");
  const [grants, setGrants] = useState<Grant[]>([]);
  const [grantId, setGrantId] = useState("");
  const [tasks, setTasks] = useState<Task[]>([]);
  const [current, setCurrent] = useState<Task | null>(null);
  const [instruction, setInstruction] = useState("");
  const [planning, setPlanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [g, t] = await Promise.all([api.listGrants(), api.listTasks()]);
    setGrants(g);
    setTasks(t);
    setGrantId((id) => (g.some((x) => x.id === id) ? id : (g[0]?.id ?? "")));
  }, []);

  const checkAi = useCallback(async () => {
    const s = await api.aiStatus();
    setAi(s);
    setModel((m) => m || (s.models.includes(s.defaultModel) ? s.defaultModel : (s.models[0] ?? s.defaultModel)));
  }, []);

  useEffect(() => {
    refresh().catch((e) => setError(errorText(e)));
  }, [refresh]);

  const run = async <T,>(fn: () => Promise<T>): Promise<T | undefined> => {
    setError(null);
    try {
      return await fn();
    } catch (e) {
      setError(errorText(e));
    } finally {
      refresh().catch(() => {});
    }
  };

  const addFolder = () =>
    run(async () => {
      const g = await api.pickFolder();
      if (g) setGrantId(g.id);
    });

  const plan = async () => {
    setPlanning(true);
    const t = await run(() => api.planTask(instruction, grantId, model));
    setPlanning(false);
    if (t) setCurrent(t);
  };

  // The planning task's id is only returned when planning ends, so find it in history.
  const cancelPlanning = () =>
    run(async () => {
      const running = (await api.listTasks()).find((t) => t.status === "planning");
      if (running) await api.cancelTask(running.id);
    });

  const openTask = (id: string) => run(async () => setCurrent(await api.getTask(id)));

  const modelReady = ai?.reachable && ai.models.includes(model);

  // Poll quickly while setup is incomplete so starting Ollama or pulling a model shows up at once.
  useEffect(() => {
    checkAi();
    const t = setInterval(checkAi, modelReady ? 15000 : 4000);
    return () => clearInterval(t);
  }, [checkAi, modelReady]);

  const selectedGrant = grants.find((g) => g.id === grantId);

  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">ActionDesk</span>
        <div className="ai">
          <span className={`dot ${modelReady ? "ok" : ai?.reachable ? "warn" : "off"}`} />
          {ai?.reachable ? (
            <select value={model} onChange={(e) => setModel(e.target.value)} aria-label="Local model">
              {!ai.models.includes(model) && <option value={model}>{model} (not installed)</option>}
              {ai.models.map((m) => (
                <option key={m}>{m}</option>
              ))}
            </select>
          ) : (
            <span className="muted">Local AI offline</span>
          )}
        </div>
      </header>

      <aside className="sidebar">
        <button className="primary block" onClick={() => setCurrent(null)}>
          + New task
        </button>

        <h3>Folders</h3>
        <ul className="list">
          {grants.map((g) => (
            <li key={g.id} className={g.id === grantId ? "active" : ""}>
              <button className="link" title={g.path} onClick={() => setGrantId(g.id)}>
                {basename(g.path)}
              </button>
              <button
                className="icon"
                title="Revoke access"
                aria-label={`Revoke access to ${g.path}`}
                onClick={() => run(() => api.revokeGrant(g.id))}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
        <button className="ghost block" onClick={addFolder}>
          + Grant a folder…
        </button>

        <h3>History</h3>
        <ul className="list history">
          {tasks.map((t) => (
            <li key={t.id} className={t.id === current?.id ? "active" : ""}>
              <button className="link" onClick={() => openTask(t.id)} title={t.instruction}>
                <span className="ellipsis">{t.instruction}</span>
                <StatusBadge task={t} />
              </button>
            </li>
          ))}
          {tasks.length === 0 && <li className="muted small">No tasks yet</li>}
        </ul>
      </aside>

      <main className="main">
        {error && (
          <div className="alert" role="alert">
            {error}
            <button className="icon" aria-label="Dismiss" onClick={() => setError(null)}>
              ×
            </button>
          </div>
        )}

        {ai && !modelReady && <SetupHint ai={ai} model={model} />}

        {current ? (
          <TaskPanel task={current} onChange={setCurrent} run={run} />
        ) : (
          <section className="composer">
            <h1>What can I do for you?</h1>
            <p className="muted">Your AI runs on your Mac. Your files stay with you.</p>

            {grants.length === 0 ? (
              <div className="card empty">
                <p>Grant ActionDesk access to a folder to get started. It can only see folders you choose here.</p>
                <button className="primary" onClick={addFolder}>
                  Choose a folder…
                </button>
              </div>
            ) : (
              <div className="card">
                <label className="field">
                  <span>Folder</span>
                  <select value={grantId} onChange={(e) => setGrantId(e.target.value)}>
                    {grants.map((g) => (
                      <option key={g.id} value={g.id}>
                        {g.path}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="field">
                  <span>Task</span>
                  <textarea
                    rows={3}
                    value={instruction}
                    placeholder="Describe how to organize this folder…"
                    onChange={(e) => setInstruction(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && instruction.trim()) plan();
                    }}
                  />
                </label>
                <div className="chips">
                  {EXAMPLES.map((ex) => (
                    <button key={ex} className="chip" onClick={() => setInstruction(ex)}>
                      {ex}
                    </button>
                  ))}
                </div>
                <div className="actions">
                  {planning ? (
                    <>
                      <span className="spinner" aria-hidden />
                      <span className="muted">
                        Planning on {model} in {basename(selectedGrant?.path ?? "")}… this can take a minute.
                      </span>
                      <button className="ghost" onClick={cancelPlanning}>
                        Cancel
                      </button>
                    </>
                  ) : (
                    <button className="primary" disabled={!instruction.trim() || !grantId || !modelReady} onClick={plan}>
                      Plan task
                    </button>
                  )}
                </div>
                <p className="muted small">Nothing changes on disk until you review and approve the plan.</p>
              </div>
            )}
          </section>
        )}
      </main>

      <footer className="statusbar">
        <span>{ai?.reachable ? "Local inference" : "Offline"}</span>
        <span>Model: {model || "none"}</span>
        <span>Data: ~/Library/Application Support/ActionDesk</span>
      </footer>
    </div>
  );
}

function SetupHint({ ai, model }: { ai: AiStatus; model: string }) {
  return (
    <div className="card hint">
      {!ai.reachable ? (
        <>
          <strong>Local AI isn't running.</strong> Install Ollama and start it, then pull a model:
          <pre>brew install ollama{"\n"}ollama serve{"\n"}ollama pull {ai.defaultModel}</pre>
        </>
      ) : (
        <>
          <strong>{model} isn't downloaded yet.</strong> Pull it (≈2 GB) with:
          <pre>ollama pull {model}</pre>
        </>
      )}
    </div>
  );
}
