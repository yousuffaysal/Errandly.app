import { useEffect, useRef, useState } from "react";
import { Check, ChevronDown, Folder, Pencil, Plus, Trash2, X } from "lucide-react";
import type { Project } from "../types";

/** The sidebar's workspace switcher: pick, create, rename and delete projects. */
export function ProjectMenu({ projects, activeId, onSelect, onCreate, onRename, onDelete }: {
  projects: Project[];
  activeId: string;
  onSelect: (id: string) => void;
  onCreate: (name: string) => Promise<void>;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}) {
  const [open, setOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<string | null>(null);
  const [name, setName] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const active = projects.find((p) => p.id === activeId);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) reset();
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  function reset() {
    setOpen(false);
    setCreating(false);
    setEditing(null);
    setConfirming(null);
    setName("");
  }

  async function submit() {
    if (!name.trim()) return;
    if (editing) await onRename(editing, name);
    else await onCreate(name);
    reset();
  }

  const form = (
    <form
      className="ew-project-form"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <input
        autoFocus
        value={name}
        maxLength={40}
        placeholder={editing ? "Project name" : "New project name"}
        aria-label="Project name"
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Escape" && reset()}
      />
      <button type="submit" className="ew-icon" aria-label="Save project" disabled={!name.trim()}>
        <Check size={14} />
      </button>
      <button type="button" className="ew-icon" aria-label="Cancel" onClick={reset}>
        <X size={14} />
      </button>
    </form>
  );

  return (
    <div className="ew-project" ref={ref}>
      <button className="ew-personal" onClick={() => (open ? reset() : setOpen(true))} aria-expanded={open}>
        <span className="ew-folder">
          <Folder size={16} />
        </span>
        <span className="ew-project-name">{active?.name ?? "Personal"}</span>
        <ChevronDown size={14} />
      </button>
      {open && (
        <div className="ew-project-menu">
          {projects.map((p) =>
            editing === p.id ? (
              <div key={p.id}>{form}</div>
            ) : confirming === p.id ? (
              <div key={p.id} className="ew-confirm">
                <span>
                  Delete “{p.name}” and its {p.conversationCount} conversation(s)? Your files stay untouched.
                </span>
                <div>
                  <button className="ew-danger" onClick={() => onDelete(p.id).then(reset)}>
                    Delete
                  </button>
                  <button onClick={() => setConfirming(null)}>Keep</button>
                </div>
              </div>
            ) : (
              <div key={p.id} className={`ew-project-row ${p.id === activeId ? "is-active" : ""}`}>
                <button
                  className="ew-project-pick"
                  onClick={() => {
                    onSelect(p.id);
                    reset();
                  }}
                >
                  <span>{p.name}</span>
                  <small>{p.conversationCount}</small>
                </button>
                <button
                  className="ew-icon"
                  aria-label={`Rename ${p.name}`}
                  onClick={() => {
                    setEditing(p.id);
                    setName(p.name);
                  }}
                >
                  <Pencil size={12} />
                </button>
                {!p.isDefault && (
                  <button className="ew-icon" aria-label={`Delete ${p.name}`} onClick={() => setConfirming(p.id)}>
                    <Trash2 size={12} />
                  </button>
                )}
              </div>
            ),
          )}
          {creating ? (
            form
          ) : (
            <button className="ew-project-new" onClick={() => setCreating(true)}>
              <Plus size={14} />
              New project
            </button>
          )}
        </div>
      )}
    </div>
  );
}
