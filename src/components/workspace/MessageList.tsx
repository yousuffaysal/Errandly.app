import { Mark } from "../Mark";
import { forwardRef, useState } from "react";
import { ArrowUpRight, BookOpen, Check, Copy, Folder, LayoutGrid, RotateCcw } from "lucide-react";
import type { Message, ProgressEvent } from "../../types";
import { ResultCardView, SavePdf } from "../ResultCards";
import { TaskCard } from "../TaskCard";
import { Markdown } from "../Markdown";

/** Answers this long (an email, a plan, notes) can be saved as a PDF. */
const SAVEABLE_CHARS = 280;

const suggestions = [
  { icon: Folder, title: "Bring a little order", text: "Organize a folder of scattered files", prompt: "Help me organize this folder by file type." },
  { icon: BookOpen, title: "Find the useful parts", text: "Turn long documents into clear notes", prompt: "Summarize the documents in this folder." },
  { icon: LayoutGrid, title: "Make the numbers click", text: "Build a report from your spreadsheets", prompt: "Analyze the spreadsheets in this folder." },
];
export const STAGES = ["Understanding your request", "Working on it", "Putting it together"];

export interface MessageListProps {
  messages: Message[] | null;
  personaName: string;
  initial: string;
  busy: boolean;
  progress?: ProgressEvent;
  live?: string;
  onSuggestion: (prompt: string) => void;
  onChanged: () => void;
  onRegenerate: () => void;
  onError: (e: unknown) => void;
}

/** The welcome screen, the conversation, and what's happening right now. */
export const MessageList = forwardRef<HTMLDivElement, MessageListProps>(function MessageList(props, endRef) {
  const { messages, personaName, busy, progress, live } = props;
  return (
    <div className="ew-scroll">
      <div className="ew-conversation">
        {!messages ? null : messages.length === 0 && !busy ? (
          <div className="ew-welcome">
            <span className="ew-flower"><Mark /></span>
            <p>A LITTLE SPACE TO THINK</p>
            <h1>What’s on your mind?</h1>
            <div>
              From a loose thought to a finished task.
              <br />
              Let’s make room for your best work.
            </div>
            <div className="ew-suggestions">
              {suggestions.map((s) => (
                <button key={s.title} onClick={() => props.onSuggestion(s.prompt)}>
                  <s.icon size={21} />
                  <strong>{s.title}</strong>
                  <span>{s.text}</span>
                  <ArrowUpRight size={15} />
                </button>
              ))}
            </div>
          </div>
        ) : (
          messages.map((m, i) => (
            <MessageItem
              key={m.id}
              message={m}
              previous={messages[i - 1]}
              personaName={personaName}
              initial={props.initial}
              canRegenerate={!busy && i === messages.length - 1 && m.role === "assistant"}
              onChanged={props.onChanged}
              onRegenerate={props.onRegenerate}
              onError={props.onError}
            />
          ))
        )}
        {busy && live && (
          <article className="ew-message ew-assistant">
            <div className="ew-message-label">
              <span className="ew-small-mark"><Mark /></span>
              <strong>{personaName}</strong>
              <small>Your thinking partner</small>
            </div>
            <div className="ew-message-text ew-streaming"><Markdown text={live} /></div>
          </article>
        )}
        {busy && progress && !live && (
          <div className="ew-thinking" role="status" aria-live="polite">
            <div className="ew-orbit">
              <i />
              <i />
              <i />
              <span><Mark /></span>
            </div>
            <div>
              <strong>
                {progress.label}
                <span className="ew-dots"><i /><i /><i /></span>
              </strong>
              <p>{personaName} is thinking on this Mac. A little less on your plate.</p>
              <div className="ew-progress">
                {STAGES.map((s, i) => (
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
  );
});

function MessageItem({ message: m, previous, personaName, initial, canRegenerate, onChanged, onRegenerate, onError }: {
  message: Message;
  previous?: Message;
  personaName: string;
  initial: string;
  canRegenerate: boolean;
  onChanged: () => void;
  onRegenerate: () => void;
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
          {m.role === "user" ? <span className="ew-small-avatar">{initial}</span> : <span className="ew-small-mark"><Mark /></span>}
          <strong>{m.role === "user" ? "You" : personaName}</strong>
          <small>{m.role === "user" ? "" : "Your thinking partner"}</small>
        </div>
        <div className="ew-message-text">{m.role === "assistant" ? <Markdown text={m.text} /> : m.text}</div>
        {m.taskId && <TaskCard taskId={m.taskId} onChanged={onChanged} onError={onError} />}
        {m.card && <ResultCardView card={m.card} messageId={m.id} onError={onError} />}
        {m.role === "assistant" && !m.card && !m.taskId && m.text.length >= SAVEABLE_CHARS && (
          <div className="ew-actions">
            <SavePdf messageId={m.id} onError={onError} />
          </div>
        )}
        {m.id > 0 && (
          <div className="ew-message-tools">
            <CopyButton text={m.text} />
            {canRegenerate && (
              <button onClick={onRegenerate} aria-label="Regenerate answer" title="Answer again">
                <RotateCcw size={14} />
              </button>
            )}
          </div>
        )}
      </article>
    </>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () =>
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }, () => {});
  return (
    <button onClick={copy} aria-label={copied ? "Copied" : "Copy message"} title={copied ? "Copied" : "Copy"}>
      {copied ? <Check size={14} /> : <Copy size={14} />}
    </button>
  );
}

export function dayLabel(iso: string) {
  const d = new Date(iso);
  const today = new Date();
  const yesterday = new Date(today.getTime() - 86_400_000);
  if (d.toDateString() === today.toDateString()) return "TODAY";
  if (d.toDateString() === yesterday.toDateString()) return "YESTERDAY";
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric" }).toUpperCase();
}
