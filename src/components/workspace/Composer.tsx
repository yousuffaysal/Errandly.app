import { useRef, useState } from "react";
import { ArrowUp, FolderOpen, Plus, Square, X } from "lucide-react";
import { basename } from "../../paths";
import type { AiStatus, Conversation } from "../../types";
import { ModelSetup } from "../ModelSetup";
import { PersonaPicker } from "../PersonaPicker";
import { matching, SlashMenu, slashQuery, useSlashCommands, type SlashCommand } from "./SlashMenu";

export function Composer({ conv, ai, modelReady, busy, notice, onDismissNotice, onSend, onStop, onAttach, onPersona, onModelsReady }: {
  conv: Conversation | undefined;
  ai: AiStatus | null;
  modelReady: boolean;
  busy: boolean;
  notice: string;
  onDismissNotice: () => void;
  onSend: (text: string) => void;
  onStop: () => void;
  onAttach: () => void;
  onPersona: (id: string) => void;
  onModelsReady: () => void;
}) {
  const [input, setInput] = useState("");
  const [active, setActive] = useState(0);
  const [menuClosed, setMenuClosed] = useState(false);
  const box = useRef<HTMLTextAreaElement>(null);
  const commands = useSlashCommands();
  const query = slashQuery(input);
  const options = query === null ? [] : matching(commands, query);
  const menuOpen = query !== null && !menuClosed;

  const send = (text = input) => {
    if (!text.trim() || busy) return;
    onSend(text);
    setInput("");
  };

  // Commands that work on the folder run straight away; writing commands
  // wait for the text to work on.
  const pick = (c: SlashCommand) => {
    if (c.takesText) {
      setInput(`/${c.name} `);
      box.current?.focus();
    } else {
      send(`/${c.name}`);
    }
    setActive(0);
  };

  return (
    <div className="ew-compose-wrap">
      {notice && (
        <div className="ew-notice" role="status">
          {notice}
          <button onClick={onDismissNotice} aria-label="Dismiss notification">
            <X size={13} />
          </button>
        </div>
      )}
      {ai && !modelReady && <ModelSetup ai={ai} onDone={onModelsReady} />}
      {menuOpen && (
        <SlashMenu commands={options} active={active} hasFolder={Boolean(conv?.folder)} onPick={pick} onHover={setActive} />
      )}
      <form
        className={`ew-compose ${busy ? "is-thinking" : ""}`}
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <textarea
          aria-label="Message Errandly"
          placeholder="A question, a task, or type / for commands…"
          ref={box}
          value={input}
          onChange={(e) => {
            setInput(e.target.value);
            setActive(0);
            setMenuClosed(false);
          }}
          onKeyDown={(e) => {
            if (menuOpen && options.length > 0) {
              if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                e.preventDefault();
                const step = e.key === "ArrowDown" ? 1 : -1;
                setActive((a) => (a + step + options.length) % options.length);
                return;
              }
              if ((e.key === "Enter" && !e.shiftKey) || e.key === "Tab") {
                e.preventDefault();
                pick(options[Math.min(active, options.length - 1)]);
                return;
              }
            }
            if (menuOpen && e.key === "Escape") {
              e.preventDefault();
              setMenuClosed(true);
              return;
            }
            if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              send();
            }
          }}
        />
        <div className="ew-compose-tools">
          <button type="button" className="ew-icon" onClick={onAttach} aria-label="Add a folder to this conversation" title="Add a folder">
            <Plus size={20} />
          </button>
          {ai && conv && <PersonaPicker personas={ai.personas} value={conv.persona} onChange={onPersona} />}
          <span className="ew-context-count" title={conv?.folder ?? undefined}>
            <FolderOpen size={12} />
            {conv?.folder ? basename(conv.folder) : "No folder"}
          </span>
          {busy ? (
            <button type="button" className="ew-send" onClick={onStop} aria-label="Stop">
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
  );
}
