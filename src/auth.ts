import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { listen } from "@tauri-apps/api/event";
import { api, OAUTH_EVENT } from "./api";

// Optional accounts. Without these two values the app runs fully local and
// hides sign-in. The anon key is public by design; access is enforced by
// Supabase row-level security, and no user files are ever sent.
const url = import.meta.env.VITE_SUPABASE_URL as string | undefined;
const anonKey = import.meta.env.VITE_SUPABASE_ANON_KEY as string | undefined;

export const supabase: SupabaseClient | null =
  url && anonKey
    ? createClient(url, anonKey, {
        auth: {
          // Keep the session in Errandly's owner-only local database, not the
          // Keychain (which prompts for a password) or browser storage.
          storage: {
            getItem: (key) => api.sessionGet(key),
            setItem: (key, value) => api.sessionSet(key, value),
            removeItem: (key) => api.sessionRemove(key),
          },
          persistSession: true,
          autoRefreshToken: true,
          detectSessionInUrl: false,
          // Desktop sign-in with Google returns a one-time code; PKCE makes it
          // useless to anyone without the verifier kept on this Mac.
          flowType: "pkce",
        },
      })
    : null;

export const accountsEnabled = supabase !== null;

/** Signs in with Google in the default browser and resolves once signed in. */
export async function signInWithGoogle(): Promise<void> {
  if (!supabase) throw new Error("Accounts aren’t set up in this build.");
  const client = supabase;
  const port = await api.startOAuthListener();
  const callback = new Promise<{ code: string | null; error: string | null }>((resolve) => {
    const un = listen<{ code: string | null; error: string | null }>(OAUTH_EVENT, (e) => {
      un.then((f) => f());
      resolve(e.payload);
    });
  });
  const { data, error } = await client.auth.signInWithOAuth({
    provider: "google",
    options: { redirectTo: `http://127.0.0.1:${port}/auth/callback`, skipBrowserRedirect: true },
  });
  if (error) throw error;
  await api.openAuthUrl(data.url);
  const result = await callback;
  if (!result.code) throw new Error(result.error ?? "Google sign-in didn’t finish.");
  const { error: exchangeError } = await client.auth.exchangeCodeForSession(result.code);
  if (exchangeError) throw exchangeError;
}
