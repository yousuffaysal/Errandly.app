import { useEffect, useRef, useState } from "react";
import { ArrowUp, Check } from "lucide-react";
import type { Persona, Profile } from "../types";

type Field = "name" | "role" | "work" | "helpWith" | "tone" | "language" | "notes";
interface Step {
  field: Field;
  ask: (p: Profile) => string;
  chips?: string[];
  multi?: boolean;
  optional?: boolean;
  placeholder?: string;
}

const STEPS: Step[] = [
  { field: "name", ask: () => "Hi, I’m Errandly. I’d like to get to know you a little, so I can help the way a good assistant would. What should I call you?", placeholder: "Your name" },
  {
    field: "role",
    ask: (p) => `Lovely to meet you, ${p.name || "friend"}. What do you do?`,
    chips: ["Student", "Researcher", "Business owner", "Freelancer", "Professional"],
    placeholder: "Or tell me in your own words",
  },
  { field: "work", ask: () => "What are you working on these days?", optional: true, placeholder: "e.g. my MSc thesis, an online shop, client projects" },
  {
    field: "helpWith",
    ask: () => "Where would a hand help most? Pick as many as you like.",
    chips: ["Organizing files", "Study and research", "Invoices and expenses", "Client work", "Reports and documents", "Planning my week"],
    multi: true,
    optional: true,
  },
  { field: "tone", ask: () => "How should I talk with you?", chips: ["Short and direct", "Warm and detailed", "Formal and precise"] },
  { field: "language", ask: () => "Which language do you prefer?", chips: ["English", "Bangla", "Hindi", "Arabic"], placeholder: "Or type another" },
  {
    field: "notes",
    ask: () => "Last one. Anything else I should know? Your city, your hours, how you like files named…",
    optional: true,
    placeholder: "Anything at all, or skip",
  },
];

/** The persona that fits someone's answers best. */
export function recommendPersona(p: Profile): string {
  const all = `${p.role} ${p.helpWith.join(" ")}`.toLowerCase();
  if (/student|research|study/.test(all)) return "suf-4";
  if (/business|invoice|expense|client|freelanc/.test(all)) return "howen-2";
  return "ario";
}

const empty: Profile = { name: "", role: "", work: "", helpWith: [], tone: "", language: "", notes: "", completed: false };

export function Onboarding({ initialName, personas, onFinish }: {
  initialName?: string;
  personas: Persona[];
  onFinish: (profile: Profile, persona: string) => Promise<void>;
}) {
  const [profile, setProfile] = useState<Profile>({ ...empty, name: initialName ?? "" });
  const [step, setStep] = useState(0);
  const [input, setInput] = useState(initialName ?? "");
  const [picked, setPicked] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const endRef = useRef<HTMLDivElement>(null);
  const done = step >= STEPS.length;
  const current = STEPS[step];
  const recommended = personas.find((p) => p.id === recommendPersona(profile));

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [step]);

  function answer(value: string | string[]) {
    if (!current) return;
    const next = { ...profile, [current.field]: value };
    setProfile(next);
    setInput("");
    setPicked([]);
    setStep(step + 1);
  }

  function submitText() {
    const text = input.trim();
    if (current.multi) {
      const all = text ? [...picked, text] : picked;
      if (all.length) answer(all);
      else if (current.optional) answer([]);
      return;
    }
    if (text) answer(text);
    else if (current.optional) answer("");
  }

  async function finish(skipped = false) {
    setSaving(true);
    try {
      await onFinish({ ...profile, completed: true }, skipped ? "ario" : recommendPersona(profile));
    } finally {
      setSaving(false);
    }
  }

  const display = (field: Field, p: Profile) => {
    const v = p[field];
    return Array.isArray(v) ? (v.length ? v.join(", ") : "Skipped") : v || "Skipped";
  };

  return (
    <div className="ew-onboarding">
      <div className="ew-onboarding-scroll">
        <div className="ew-conversation">
          <div className="ew-welcome ew-onboarding-head">
            <span className="ew-flower">✳</span>
            <p>GETTING TO KNOW YOU</p>
            <h1>A little about you.</h1>
            <div>Seven quick questions, so Errandly can help like a real personal assistant.<br />Your answers stay on this Mac.</div>
          </div>

          {STEPS.slice(0, Math.min(step + 1, STEPS.length)).map((s, i) => (
            <div key={s.field}>
              <article className="ew-message ew-assistant">
                <div className="ew-message-label">
                  <span className="ew-small-mark">✳</span>
                  <strong>Errandly</strong>
                  <small>Your thinking partner</small>
                </div>
                <div className="ew-message-text">{s.ask(profile)}</div>
              </article>
              {i < step && (
                <article className="ew-message ew-user">
                  <div className="ew-message-label">
                    <span className="ew-small-avatar">{(profile.name || "Y").charAt(0).toUpperCase()}</span>
                    <strong>You</strong>
                  </div>
                  <div className="ew-message-text">{display(s.field, profile)}</div>
                </article>
              )}
            </div>
          ))}

          {done && (
            <article className="ew-message ew-assistant">
              <div className="ew-message-label">
                <span className="ew-small-mark">✳</span>
                <strong>Errandly</strong>
                <small>Your thinking partner</small>
              </div>
              <div className="ew-message-text">
                Thank you, {profile.name || "friend"}. Here’s what I’ll keep in mind. You can change it anytime in Settings → Personalization.
              </div>
              <div className="ew-report ew-profile-card">
                <div className="ew-report-heading">
                  <div>
                    <span className="ew-report-icon">✳</span>
                    <strong>
                      {profile.name || "You"}
                      <small>{[profile.role, profile.work].filter(Boolean).join(" · ") || "Your profile"}</small>
                    </strong>
                  </div>
                  <span className="ew-ready ok"><Check size={11} />Saved on this Mac</span>
                </div>
                <div className="ew-categories">
                  {(["helpWith", "tone", "language", "notes"] as Field[]).map((f) => (
                    <div key={f}>
                      <span className="ew-profile-label">{{ helpWith: "Help with", tone: "Tone", language: "Language", notes: "Notes" }[f as "helpWith"]}</span>
                      <strong>{display(f, profile)}</strong>
                    </div>
                  ))}
                </div>
                {recommended && (
                  <div className="ew-report-foot">
                    ✳ I’ll pair you with <b>&nbsp;{recommended.name}</b>, {recommended.tagline.toLowerCase()}. You can switch models in any chat.
                  </div>
                )}
              </div>
              <div className="ew-actions">
                <button className="ew-followup ew-approve" disabled={saving} onClick={() => finish()}>
                  {saving ? "Getting things ready…" : "Let’s get started"}
                </button>
                <button className="ew-followup" onClick={() => { setStep(0); setInput(profile.name); }}>
                  Start over
                </button>
              </div>
            </article>
          )}
          <div ref={endRef} />
        </div>
      </div>

      {!done && current && (
        <div className="ew-compose-wrap">
          {current.chips && (
            <div className="ew-chips">
              {current.chips.map((c) => {
                const on = picked.includes(c);
                return (
                  <button
                    key={c}
                    className={`ew-chip ${on ? "is-on" : ""}`}
                    onClick={() => (current.multi ? setPicked(on ? picked.filter((x) => x !== c) : [...picked, c]) : answer(c))}
                  >
                    {current.multi && on && <Check size={12} />}
                    {c}
                  </button>
                );
              })}
            </div>
          )}
          <form
            className="ew-compose"
            onSubmit={(e) => {
              e.preventDefault();
              submitText();
            }}
          >
            <textarea
              autoFocus
              aria-label="Your answer"
              placeholder={current.placeholder ?? (current.multi ? "Add your own, or continue" : "Type your answer")}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                  e.preventDefault();
                  submitText();
                }
              }}
            />
            <div className="ew-compose-tools">
              <span className="ew-context-count ew-step-count">
                Question {step + 1} of {STEPS.length}
              </span>
              {(current.optional || current.multi) && (
                <button type="button" className="ew-followup ew-skip" onClick={submitText}>
                  {current.multi && picked.length ? "Continue" : "Skip"}
                </button>
              )}
              <button className="ew-send" disabled={!input.trim() && !(current.multi && picked.length)} aria-label="Send answer">
                <ArrowUp size={19} />
              </button>
            </div>
          </form>
          <p className="ew-disclaimer">
            <button className="ew-link" disabled={saving} onClick={() => finish(true)}>
              Skip the introductions for now
            </button>
          </p>
        </div>
      )}
    </div>
  );
}
