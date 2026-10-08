import { useState } from "react";
import { api } from "../api";
import { basename, relative } from "../paths";
import type { Step, StepStatus, Task } from "../types";

type Run = <T>(fn: () => Promise<T>) => Promise<T | undefined>;

const STATUS_LABEL: Record<Task["status"], string> = {
  created: "Created",
  planning: "Planning",
  awaiting_approval: "Awaiting approval",
  executing: "Executing",
  verifying: "Verifying",
  completed: "Completed",
  partially_completed: "Partially completed",
  failed: "Failed",
  cancelled: "Cancelled",
};

export function StatusBadge({ task }: { task: Task }) {
  const label = task.undoneAt ? "Undone" : STATUS_LABEL[task.status];
  const tone = task.undoneAt ? "neutral" : task.status;
  return <span className={`badge ${tone}`}>{label}</span>;
}

export function TaskPanel({ task, onChange, run }: { task: Task; onChange: (t: Task) => void; run: Run }) {
  const [busy, setBusy] = useState<"executing" | "undoing" | null>(null);

  const act = async (kind: typeof busy, fn: () => Promise<Task>) => {
    setBusy(kind);
    const t = await run(fn);
    setBusy(null);
    if (t) onChange(t);
  };

  return (
    <section className="task">
      <div className="task-head">
        <div>
          <h2>{task.instruction}</h2>
          <p className="muted small">
            {task.root} · {task.modelId}
          </p>
        </div>
        <StatusBadge task={task} />
      </div>

      {task.error && <div className="alert soft">{task.error}</div>}

      {task.status === "awaiting_approval" ? (
        <Approval
          task={task}
          busy={busy === "executing"}
          onApprove={() => act("executing", () => api.approveTask(task.id))}
          onReject={() => act(null, () => api.cancelTask(task.id))}
          onStop={() => run(() => api.cancelTask(task.id))}
        />
      ) : (
        <Result
          task={task}
          busy={busy === "undoing"}
          onUndo={() => act("undoing", () => api.undoTask(task.id))}
        />
      )}
    </section>
  );
}

function Approval(props: { task: Task; busy: boolean; onApprove: () => void; onReject: () => void; onStop: () => void }) {
  const { task, busy } = props;
  const moves = task.steps.filter((s) => s.op === "move_file");
  const folders = task.steps.filter((s) => s.op === "create_folder");
  const groups = groupMoves(task.root, task.steps);
  const plan = task.plan;

  if (moves.length === 0) {
    return (
      <div className="card">
        <p>The model didn't assign any files to a folder, so there is nothing to do.</p>
        <button className="ghost" onClick={props.onReject}>
          Dismiss
        </button>
      </div>
    );
  }

  return (
    <div className="card approval">
      <h3>
        ActionDesk wants to organize {moves.length} of {plan?.scannedFiles ?? moves.length} files.
      </h3>
      <dl className="counts">
        <Count label="Create folders" n={folders.length} />
        <Count label="Move files" n={moves.length} />
        <Count label="Rename files" n={moves.filter((m) => basename(m.from) !== basename(m.to)).length} />
        <Count label="Delete files" n={0} />
      </dl>
      <p className="muted small">Inside: {task.root}</p>

      <div className="groups">
        {groups.map(([folder, items]) => (
          <details key={folder} open={groups.length <= 4}>
            <summary>
              <strong>{folder}/</strong> <span className="muted">{items.length} files</span>
              {!task.steps.some((s) => s.op === "create_folder" && relative(task.root, s.path) === folder) && (
                <span className="muted small"> (existing folder)</span>
              )}
            </summary>
            <ul>
              {items.map((m) => (
                <li key={m.seq}>
                  {basename(m.from)}
                  {basename(m.from) !== basename(m.to) && (
                    <span className="muted"> → {basename(m.to)} (name taken)</span>
                  )}
                </li>
              ))}
            </ul>
          </details>
        ))}
        {plan && plan.leftInPlace.length > 0 && (
          <details>
            <summary>
              <strong>Left in place</strong> <span className="muted">{plan.leftInPlace.length} files</span>
            </summary>
            <ul>
              {plan.leftInPlace.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          </details>
        )}
      </div>

      {plan && plan.rejectedOutputs > 0 && (
        <p className="muted small">
          {plan.rejectedOutputs} model suggestion(s) failed validation and were discarded.
        </p>
      )}

      <div className="actions end">
        {busy ? (
          <>
            <span className="spinner" aria-hidden />
            <span className="muted">Moving files…</span>
            <button className="ghost" onClick={props.onStop}>
              Stop
            </button>
          </>
        ) : (
          <>
            <button className="ghost" onClick={props.onReject}>
              Cancel
            </button>
            <button className="primary" onClick={props.onApprove}>
              Approve &amp; Execute
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function Result({ task, busy, onUndo }: { task: Task; busy: boolean; onUndo: () => void }) {
  const tally = task.steps.reduce<Partial<Record<StepStatus, number>>>((acc, s) => {
    acc[s.status] = (acc[s.status] ?? 0) + 1;
    return acc;
  }, {});
  const canUndo = !task.undoneAt && (tally.done ?? 0) > 0;

  return (
    <div className="card">
      <dl className="counts">
        <Count label="Done" n={tally.done ?? 0} />
        <Count label="Failed" n={tally.failed ?? 0} />
        <Count label="Skipped" n={tally.skipped ?? 0} />
        {task.undoneAt && <Count label="Restored" n={tally.undone ?? 0} />}
      </dl>
      {task.steps.length > 0 && (
        <ul className="steps">
          {task.steps.map((s) => (
            <li key={s.seq} className={s.status}>
              <span className="step-status">{s.status.replace("_", " ")}</span>
              <span className="ellipsis">{describe(task.root, s)}</span>
              {s.error && <span className="muted small"> — {s.error}</span>}
            </li>
          ))}
        </ul>
      )}
      {canUndo && (
        <div className="actions end">
          <button className="ghost" disabled={busy} onClick={onUndo}>
            {busy ? "Undoing…" : "Undo this task"}
          </button>
        </div>
      )}
    </div>
  );
}

const Count = ({ label, n }: { label: string; n: number }) => (
  <div>
    <dt>{label}</dt>
    <dd>{n}</dd>
  </div>
);

function describe(root: string, s: Step) {
  return s.op === "create_folder"
    ? `Create folder ${relative(root, s.path)}/`
    : `Move ${relative(root, s.from)} → ${relative(root, s.to)}`;
}

function groupMoves(root: string, steps: Step[]) {
  const groups = new Map<string, Extract<Step, { op: "move_file" }>[]>();
  for (const s of steps) {
    if (s.op !== "move_file") continue;
    const folder = relative(root, s.to).split("/")[0];
    groups.set(folder, [...(groups.get(folder) ?? []), s]);
  }
  return [...groups.entries()];
}
