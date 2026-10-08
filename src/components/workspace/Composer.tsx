import { useState } from "react";
import { ArrowUp, FolderOpen, Plus, Square, X } from "lucide-react";
import { basename } from "../../paths";
import type { AiStatus, Conversation } from "../../types";
import { ModelSetup } from "../ModelSetup";
import { PersonaPicker } from "../PersonaPicker";

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
  const send = () => {
    if (!input.trim() || busy) return;
    onSend(input);
    setInput("");
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
