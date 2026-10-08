import { useEffect, useState } from "react";
import type { User } from "@supabase/supabase-js";
import { LogIn, LogOut, Settings, X } from "lucide-react";
import { accountsEnabled, signInWithGoogle, supabase } from "../auth";
import { errorText } from "../api";

type Mode = "sign-in" | "sign-up" | "reset";

/**
 * Signed-in user, kept in sync with Supabase: `undefined` while the saved
 * session is still being read, `null` when signed out or accounts are off.
 */
export function useUser() {
  const [user, setUser] = useState<User | null | undefined>(supabase ? undefined : null);
  useEffect(() => {
    if (!supabase) return;
    supabase.auth.getSession().then(({ data }) => setUser(data.session?.user ?? null));
    const { data } = supabase.auth.onAuthStateChange((_e, session) => setUser(session?.user ?? null));
    return () => data.subscription.unsubscribe();
  }, []);
  return user;
}

/**
 * A friendly name from the account: the Google name, the name given at
 * sign-up, or else the email's first part ("yusuf.faisal9t+work" → "Yusuf Faisal").
 */
export function accountName(user: User | null | undefined): string {
  if (!user) return "";
  const meta = user.user_metadata ?? {};
  const given = (meta.full_name || meta.name || "") as string;
  if (given.trim()) return given.trim();
  const local = (user.email ?? "").split("@")[0].split("+")[0];
  return local
    .split(/[._-]+/)
    .map((w) => w.replace(/\d.*$/, ""))
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

/** The sidebar's bottom profile row. */
export function ProfileRow({ user, profileName, onSignIn, onSettings }: {
  user: User | null;
  profileName: string;
  onSignIn: () => void;
  onSettings: () => void;
}) {
  const name = profileName || accountName(user);
  return (
    <div className="ew-profile">
      <span className="ew-avatar">{(name || "Y").charAt(0).toUpperCase()}</span>
      <div className="ew-profile-text">
        {name || "Your workspace"}
        <small>{user ? user.email : "Personal · On this Mac"}</small>
      </div>
      <button className="ew-icon" title="Settings (⌘,)" aria-label="Settings" onClick={onSettings}>
        <Settings size={17} />
      </button>
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

  async function google() {
    setBusy(true);
    setMessage({ ok: true, text: "Continue in your browser. Errandly will sign you in when you’re done." });
    try {
      await signInWithGoogle();
      onClose();
    } catch (e) {
      setMessage({ ok: false, text: errorText(e) });
    } finally {
      setBusy(false);
    }
  }

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
        {mode !== "reset" && (
          <>
            <button type="button" className="ew-google" onClick={google} disabled={busy}>
              <svg width="16" height="16" viewBox="0 0 48 48" aria-hidden>
                <path fill="#FFC107" d="M43.6 20.5H42V20H24v8h11.3C33.7 32.7 29.2 36 24 36c-6.6 0-12-5.4-12-12s5.4-12 12-12c3.1 0 5.8 1.2 7.9 3.1l5.7-5.7C34 6.1 29.3 4 24 4 12.9 4 4 12.9 4 24s8.9 20 20 20 20-8.9 20-20c0-1.3-.1-2.4-.4-3.5z" />
                <path fill="#FF3D00" d="M6.3 14.7l6.6 4.8C14.7 15.1 19 12 24 12c3.1 0 5.8 1.2 7.9 3.1l5.7-5.7C34 6.1 29.3 4 24 4 16.3 4 9.7 8.3 6.3 14.7z" />
                <path fill="#4CAF50" d="M24 44c5.2 0 9.9-2 13.4-5.2l-6.2-5.2C29.2 35.1 26.7 36 24 36c-5.2 0-9.6-3.3-11.3-8l-6.5 5C9.5 39.6 16.2 44 24 44z" />
                <path fill="#1976D2" d="M43.6 20.5H42V20H24v8h11.3c-.8 2.2-2.2 4.2-4.1 5.6l6.2 5.2C37 39.2 44 34 44 24c0-1.3-.1-2.4-.4-3.5z" />
              </svg>
              Continue with Google
            </button>
            <div className="ew-or"><span />or with email<span /></div>
          </>
        )}
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
