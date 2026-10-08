import { useEffect, useState } from "react";
import type { User } from "@supabase/supabase-js";
import { LogIn, LogOut, X } from "lucide-react";
import { accountsEnabled, supabase } from "../auth";
import { errorText } from "../api";

type Mode = "sign-in" | "sign-up" | "reset";

/** Signed-in user, kept in sync with Supabase. Always null when accounts are off. */
export function useUser() {
  const [user, setUser] = useState<User | null>(null);
  useEffect(() => {
    if (!supabase) return;
    supabase.auth.getUser().then(({ data }) => setUser(data.user ?? null));
    const { data } = supabase.auth.onAuthStateChange((_e, session) => setUser(session?.user ?? null));
    return () => data.subscription.unsubscribe();
  }, []);
  return user;
}

/** The sidebar's bottom profile row. */
export function ProfileRow({ user, onSignIn }: { user: User | null; onSignIn: () => void }) {
  const name = (user?.user_metadata?.name as string | undefined) || user?.email?.split("@")[0];
  return (
    <div className="ew-profile">
      <span className="ew-avatar">{(name ?? "Y").charAt(0).toUpperCase()}</span>
      <div className="ew-profile-text">
        {user ? name : "Your workspace"}
        <small>{user ? user.email : "Personal · On this Mac"}</small>
      </div>
      {accountsEnabled &&
        (user ? (
          <button className="ew-icon" title="Sign out" aria-label="Sign out" onClick={() => supabase?.auth.signOut()}>
            <LogOut size={16} />
          </button>
        ) : (
          <button className="ew-icon" title="Sign in" aria-label="Sign in" onClick={onSignIn}>
            <LogIn size={16} />
          </button>
        ))}
    </div>
  );
}

export function AuthDialog({ onClose }: { onClose: () => void }) {
  const [mode, setMode] = useState<Mode>("sign-in");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);

  useEffect(() => {
    const esc = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    document.addEventListener("keydown", esc);
    return () => document.removeEventListener("keydown", esc);
  }, [onClose]);

  if (!supabase) return null;
  const auth = supabase.auth;

  async function submit() {
    setBusy(true);
    setMessage(null);
    try {
      if (mode === "sign-in") {
        const { error } = await auth.signInWithPassword({ email, password });
        if (error) throw error;
        onClose();
      } else if (mode === "sign-up") {
        const { data, error } = await auth.signUp({ email, password, options: { data: { name: name.trim() } } });
        if (error) throw error;
        if (data.session) onClose();
        else setMessage({ ok: true, text: "Check your email to confirm your account, then sign in here." });
      } else {
        const { error } = await auth.resetPasswordForEmail(email);
        if (error) throw error;
        setMessage({ ok: true, text: "If that email has an account, a reset link is on its way." });
      }
    } catch (e) {
      setMessage({ ok: false, text: errorText(e) });
    } finally {
      setBusy(false);
    }
  }

  const titles: Record<Mode, [string, string]> = {
    "sign-in": ["Welcome back.", "Sign in to your Errandly account."],
    "sign-up": ["A little space, yours.", "Create a free Errandly account."],
    reset: ["Let’s get you back in.", "We’ll email you a reset link."],
  };

  return (
    <div className="ew-modal-scrim" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="ew-modal" role="dialog" aria-modal="true" aria-labelledby="auth-title">
        <button className="ew-icon ew-modal-close" onClick={onClose} aria-label="Close">
          <X size={16} />
        </button>
        <span className="ew-flower ew-modal-flower">✳</span>
        <h2 id="auth-title">{titles[mode][0]}</h2>
        <p className="ew-modal-sub">{titles[mode][1]}</p>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          {mode === "sign-up" && (
            <label>
              Name
              <input value={name} onChange={(e) => setName(e.target.value)} autoComplete="name" />
            </label>
          )}
          <label>
            Email
            <input type="email" required value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" autoFocus />
          </label>
          {mode !== "reset" && (
            <label>
              Password
              <input
                type="password"
                required
                minLength={8}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoComplete={mode === "sign-up" ? "new-password" : "current-password"}
              />
            </label>
          )}
          {message && <p className={`ew-modal-message ${message.ok ? "ok" : "bad"}`}>{message.text}</p>}
          <button className="ew-modal-submit" disabled={busy}>
            {busy ? "One moment…" : mode === "sign-in" ? "Sign in" : mode === "sign-up" ? "Create account" : "Send reset link"}
          </button>
        </form>
        <div className="ew-modal-links">
          {mode !== "sign-in" && <button onClick={() => setMode("sign-in")}>I have an account</button>}
          {mode !== "sign-up" && <button onClick={() => setMode("sign-up")}>Create an account</button>}
          {mode === "sign-in" && <button onClick={() => setMode("reset")}>Forgot password?</button>}
        </div>
        <p className="ew-modal-note">
          Your account is only for signing in. Your files, chats and models stay on this Mac.
        </p>
      </div>
    </div>
  );
}
