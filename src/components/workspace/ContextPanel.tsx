import { ArrowUpRight, Command, Download, FileText, Folder, FolderOpen, Plus, ShieldCheck, X } from "lucide-react";
import { basename } from "../../paths";
import type { Conversation, Persona } from "../../types";

export function ContextPanel({ conv, persona, messageCount, instructions, saved, onInstructions, onAttach, onDetach, onAttachFiles, onDetachFile, onExport, onClose }: {
  conv: Conversation;
  persona: Persona | undefined;
  messageCount: number;
  instructions: string;
  saved: boolean;
  onInstructions: (text: string) => void;
  onAttach: () => void;
  onDetach: () => void;
  onAttachFiles: () => void;
  onDetachFile: (path: string) => void;
  onExport: () => void;
  onClose: () => void;
}) {
  return (
    <aside className="ew-context">
      <div className="ew-context-heading">
        <span>Conversation context</span>
        <button className="ew-icon" onClick={onClose} aria-label="Close context">
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
            Folder<span>{conv.folder ? 1 : 0}</span>
            <button className="ew-icon" onClick={onAttach} aria-label="Add a folder">
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
              <button aria-label={`Remove ${basename(conv.folder)}`} onClick={onDetach}>
                <X size={12} />
              </button>
            </div>
          )}
          <button className="ew-add-files" onClick={onAttach}>
            <FolderOpen size={14} />
            {conv.folder ? "Use a different folder" : "Add a folder to this chat"}
          </button>
        </div>
        <div className="ew-context-section">
          <h3>
            Files<span>{conv.files.length}</span>
            <button className="ew-icon" onClick={onAttachFiles} aria-label="Add files">
              <Plus size={15} />
            </button>
          </h3>
          {conv.files.map((f) => (
            <div className="ew-file" key={f}>
              <span><FileText size={17} /></span>
              <div title={f}>
                {basename(f)}
                <small>{f}</small>
              </div>
              <button aria-label={`Remove ${basename(f)}`} onClick={() => onDetachFile(f)}>
                <X size={12} />
              </button>
            </div>
          ))}
          <button className="ew-add-files" onClick={onAttachFiles}>
            <FileText size={14} />
            {conv.files.length ? "Add more files" : "Add files to read and ask about"}
          </button>
        </div>
        {persona && (
          <div className="ew-context-section">
            <h3>
              Model<span>{persona.tagline}</span>
            </h3>
            <div className="ew-memory">
              <span className="ew-memory-dot" />
              <div>
                {persona.name}
                <small>{persona.behavior}</small>
              </div>
            </div>
          </div>
        )}
        <div className="ew-context-section">
          <h3>
            Instructions<span className="ew-auto-save">{saved ? "Auto-saved" : "Saving…"}</span>
          </h3>
          <textarea
            aria-label="Conversation instructions"
            value={instructions}
            placeholder="How would you like Errandly to help?"
            onChange={(e) => onInstructions(e.target.value)}
          />
          <p className="ew-field-help">A little guidance goes a long way.</p>
        </div>
        <div className="ew-context-section">
          <h3>Conversation memory</h3>
          <div className="ew-memory">
            <span className="ew-memory-dot" />
            <div>
              Keeping the thread<small>{messageCount} messages in this conversation</small>
            </div>
          </div>
          <p className="ew-memory-note">Your messages, instructions, files and folder stay together when you return to this chat.</p>
        </div>
        <button className="ew-export" onClick={onExport}>
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
  );
}
