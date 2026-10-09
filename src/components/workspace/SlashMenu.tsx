import { useEffect, useState } from "react";
import { FolderOpen } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

export interface SlashCommand {
  name: string;
  title: string;
  hint: string;
  takesText: boolean;
  needsFolder: boolean;
}

let cache: SlashCommand[] | null = null;

/** The commands, loaded once from the backend (the single source of truth). */
export function useSlashCommands() {
  const [commands, setCommands] = useState<SlashCommand[]>(cache ?? []);
  useEffect(() => {
    if (cache) return;
    invoke<SlashCommand[]>("list_commands")
      .then((c) => {
        cache = c ?? [];
        setCommands(cache);
      })
      .catch(() => {});
  }, []);
  return commands;
}

/** The text after "/" while the user is still typing the command name, else null. */
export function slashQuery(input: string): string | null {
  const m = /^\/(\w*)$/.exec(input);
  return m ? m[1].toLowerCase() : null;
}

export function matching(commands: SlashCommand[], query: string) {
  return commands.filter((c) => c.name.startsWith(query) || c.title.toLowerCase().includes(query));
}

export function SlashMenu({ commands, active, hasFolder, onPick, onHover }: {
  commands: SlashCommand[];
  active: number;
  hasFolder: boolean;
  onPick: (c: SlashCommand) => void;
  onHover: (i: number) => void;
}) {
  if (commands.length === 0) {
    return (
      <div className="ew-slash" role="listbox" aria-label="Commands">
        <p className="ew-slash-empty">No command with that name.</p>
      </div>
    );
  }
  return (
    <div className="ew-slash" role="listbox" aria-label="Commands">
      <p className="ew-persona-heading">COMMANDS</p>
      {commands.map((c, i) => (
        <button
          key={c.name}
          type="button"
          role="option"
          aria-selected={i === active}
          className={`ew-slash-item ${i === active ? "is-active" : ""}`}
          onMouseDown={(e) => {
            e.preventDefault(); // keep focus in the text box
            onPick(c);
          }}
          onMouseEnter={() => onHover(i)}
        >
          <code>/{c.name}</code>
          <span className="ew-slash-text">
            <strong>{c.title}</strong>
            <small>{c.hint}</small>
          </span>
          {c.needsFolder && !hasFolder && (
            <em title="Add a folder to this conversation first">
              <FolderOpen size={11} /> needs a folder
            </em>
          )}
        </button>
      ))}
    </div>
  );
}
