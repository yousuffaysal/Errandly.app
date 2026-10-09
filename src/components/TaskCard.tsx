import { Mark } from "./Mark";
import { useCallback, useEffect, useState } from "react";
import { ArrowUpRight, Check, FolderTree, RotateCcw, ShieldCheck } from "lucide-react";
import { api } from "../api";
import { basename, relative } from "../paths";
import type { Step, Task } from "../types";

// The design's four report colours, extended in the same family for up to eight folders.
const COLORS = ["#7e9065", "#b0bd99", "#d1c7a9", "#dfe3d4", "#94a27f", "#c4cfb2", "#b9ad8c", "#e9ece0"];

type Move = Extract<Step, { op: "move_file" }>;

export function TaskCard({ taskId, onChanged, onError }: {
  taskId: string;
  onChanged: () => void;
  onError: (e: unknown) => void;
}) {
  const [task, setTask] = useState<Task | null>(null);
  const [working, setWorking] = useState<"moving" | "undoing" | null>(null);

  const load = useCallback(() => api.getTask(taskId).then(setTask, onError), [taskId, onError]);
  useEffect(() => {
    load();
  }, [load]);

  if (!task) return null;

  const run = async (kind: typeof working, fn: () => Promise<unknown>) => {
    setWorking(kind);
    try {
      await fn();
    } catch (e) {
      onError(e);
    } finally {
      setWorking(null);
      await load();
      onChanged();
    }
  };

  const moves = task.steps.filter((s): s is Move => s.op === "move_file");
  const created = new Set(task.steps.flatMap((s) => (s.op === "create_folder" ? [relative(task.root, s.path)] : [])));
  const groups = groupByFolder(task.root, moves);
  const renamed = moves.filter((m) => basename(m.from) !== basename(m.to)).length;
  const total = moves.length;
  const done = moves.filter((m) => m.status === "done" || m.status === "undone").length;
  const awaiting = task.status === "awaiting_approval";
  const left = task.plan?.leftInPlace ?? [];
  const max = Math.max(...groups.map(([, items]) => items.length), 1);
  const badge = statusBadge(task);

  return (
    <>
      <div className="ew-report">
        <div className="ew-report-heading">
          <div>
            <span className="ew-report-icon"><FolderTree size={19} /></span>
            <strong>
              {basename(task.root)}, in order
              <small>
                {awaiting ? "Proposed plan" : "Organized"} · {task.plan?.scannedFiles ?? total} files looked at · {task.modelId}
              </small>
            </strong>
          </div>
          <span className={`ew-ready ${badge.tone}`}>
            {badge.tone === "ok" && <Check size={11} />}
            {badge.label}
          </span>
        </div>

        <div className="ew-total">
          <span>
            {awaiting ? "Files to move" : task.undoneAt ? "Files restored" : "Files moved"}
            <strong>
              {awaiting ? total : task.undoneAt ? moves.filter((m) => m.status === "undone").length : done}
              <span> / {task.plan?.scannedFiles ?? total}</span>
            </strong>
          </span>
          <div className="ew-mini-chart" aria-hidden>
            {groups.map(([folder, items], i) => (
              <i key={folder} style={{ height: `${Math.max(12, (items.length / max) * 100)}%`, background: COLORS[i % COLORS.length] }} />
            ))}
          </div>
        </div>

        <div className="ew-category-bar ew-category-bar-live">
          {groups.map(([folder, items], i) => (
            <i key={folder} style={{ width: `${(items.length / total) * 100}%`, background: COLORS[i % COLORS.length] }} />
          ))}
        </div>

        <div className="ew-categories">
          {groups.map(([folder, items], i) => (
            <div key={folder}>
              <i style={{ background: COLORS[i % COLORS.length] }} />
              <span>
                {folder === IN_PLACE ? folder : `${folder}/`}
                {created.has(folder) && <em className="ew-new-folder"> new</em>}
              </span>
              <strong>{items.length}</strong>
              <small>{Math.round((items.length / total) * 100)}%</small>
            </div>
          ))}
        </div>

        <details className="ew-review" open={renamed > 0 && moves.length <= 30}>
          <summary>Review every change</summary>
          <ul>
            {moves.map((m) => (
              <li key={m.seq} className={m.status}>
                <span className="ew-review-name">{basename(m.from)}</span>
                <span className="ew-review-arrow">→</span>
                <span className="ew-review-to">{relative(task.root, m.to)}</span>
                {basename(m.from) !== basename(m.to) && <em className="ew-new-folder">renamed</em>}
                {!awaiting && <span className="ew-review-status">{m.status.replace("_", " ")}</span>}
                {m.error && <small>{m.error}</small>}
              </li>
            ))}
          </ul>
          {left.length > 0 && (
            <>
              <p className="ew-review-left">Left in place ({left.length})</p>
              <ul>
                {left.map((n) => <li key={n}><span className="ew-review-name">{n}</span></li>)}
              </ul>
            </>
          )}
        </details>

        <div className="ew-report-foot">
          <ShieldCheck size={13} />
          {awaiting
            ? `Nothing moves until you approve · ${left.length} left in place · nothing deleted`
            : task.undoneAt
              ? "Restored to where everything was"
              : "Checked on disk · undo anytime"}
        </div>
      </div>

      {task.plan && task.plan.rejectedOutputs > 0 && (
        <p className="ew-takeaway">
          <span>↳</span> {task.plan.rejectedOutputs} suggestion(s) from the model didn’t pass the safety checks and were set aside.
        </p>
      )}

      {working === "moving" ? (
        <div className="ew-thinking ew-thinking-inline" role="status" aria-live="polite">
          <div className="ew-orbit"><i /><i /><i /><span><Mark /></span></div>
          <div>
            <strong>Moving files<span className="ew-dots"><i /><i /><i /></span></strong>
            <p>Each move is recorded first, so it can be undone.</p>
          </div>
          <button className="ew-followup ew-stop" onClick={() => api.cancelTask(task.id).catch(onError)}>Stop</button>
        </div>
      ) : awaiting ? (
        <div className="ew-actions">
          <button className="ew-followup ew-approve" onClick={() => run("moving", () => api.approveTask(task.id))}>
            Approve &amp; organize
            <ArrowUpRight size={14} />
          </button>
          <button className="ew-followup" onClick={() => run(null, () => api.cancelTask(task.id))}>
            Not now
          </button>
        </div>
      ) : (
        !task.undoneAt &&
        done > 0 && (
          <div className="ew-actions">
            <button className="ew-followup" disabled={working !== null} onClick={() => run("undoing", () => api.undoTask(task.id))}>
              {working === "undoing" ? "Putting things back…" : "Undo"}
              <RotateCcw size={13} />
            </button>
          </div>
        )
      )}
    </>
  );
}

/** Files renamed without moving are grouped under this label. */
const IN_PLACE = "Renamed here";

function groupByFolder(root: string, moves: Move[]) {
  const groups = new Map<string, Move[]>();
  for (const m of moves) {
    const rel = relative(root, m.to);
    const folder = rel.includes("/") ? rel.split("/")[0] : IN_PLACE;
    groups.set(folder, [...(groups.get(folder) ?? []), m]);
  }
  return [...groups.entries()].sort((a, b) => b[1].length - a[1].length);
}

function statusBadge(task: Task): { label: string; tone: string } {
  if (task.undoneAt) return { label: "Undone", tone: "muted" };
  switch (task.status) {
    case "awaiting_approval":
      return { label: "Ready to review", tone: "" };
    case "executing":
    case "verifying":
      return { label: "Working", tone: "" };
    case "completed":
      return { label: "Done", tone: "ok" };
    case "partially_completed":
      return { label: "Partly done", tone: "warn" };
    case "failed":
      return { label: "Didn’t work", tone: "bad" };
    case "cancelled":
      return { label: "Set aside", tone: "muted" };
    default:
      return { label: task.status, tone: "" };
  }
}
