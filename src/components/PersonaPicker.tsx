import { Mark } from "./Mark";
import { useEffect, useRef, useState } from "react";
import { Check, ChevronDown, Sparkles } from "lucide-react";
import type { Persona } from "../types";

/** Chooses which of Errandly's four models answers in this conversation. */
export function PersonaPicker({ personas, value, onChange }: {
  personas: Persona[];
  value: string;
  onChange: (id: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const current = personas.find((p) => p.id === value) ?? personas[0];

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const esc = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", esc);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", esc);
    };
  }, [open]);

  return (
    <div className="ew-persona" ref={ref}>
      <button
        type="button"
        className="ew-compose-model ew-persona-button"
        onClick={() => setOpen(!open)}
        aria-haspopup="listbox"
        aria-expanded={open}
      >
        <Sparkles size={14} />
        {current?.name ?? "Errandly"}
        <span className="ew-model-tag">{current?.tagline ?? "Local"}</span>
        <ChevronDown size={12} />
      </button>
      {open && (
        <div className="ew-persona-menu" role="listbox" aria-label="Choose a model">
          <p className="ew-persona-heading">ERRANDLY MODELS · ON THIS MAC</p>
          {personas.map((p) => (
            <button
              key={p.id}
              role="option"
              aria-selected={p.id === value}
              className={`ew-persona-option ${p.id === value ? "is-active" : ""}`}
              onClick={() => {
                onChange(p.id);
                setOpen(false);
              }}
            >
              <span className="ew-persona-mark"><Mark /></span>
              <span className="ew-persona-body">
                <strong>
                  {p.name} <small>{p.tagline}</small>
                  {!p.installed && <em>Not set up</em>}
                </strong>
                <span className="ew-persona-behavior">{p.behavior}</span>
                <span className="ew-persona-skills">
                  {p.skills.map((s) => (
                    <i key={s}>{s}</i>
                  ))}
                  {p.coming.map((s) => (
                    <i key={s} className="soon">
                      Soon: {s}
                    </i>
                  ))}
                </span>
              </span>
              {p.id === value && <Check size={14} className="ew-persona-check" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
