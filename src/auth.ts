import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { api } from "./api";

// Optional accounts. Without these two values the app runs fully local and
// hides sign-in. The anon key is public by design; access is enforced by
// Supabase row-level security, and no user files are ever sent.
const url = import.meta.env.VITE_SUPABASE_URL as string | undefined;
const anonKey = import.meta.env.VITE_SUPABASE_ANON_KEY as string | undefined;

export const supabase: SupabaseClient | null =
  url && anonKey
    ? createClient(url, anonKey, {
        auth: {
          // Keep the session in the macOS Keychain instead of browser storage.
          storage: {
            getItem: (key) => api.secureGet(key),
            setItem: (key, value) => api.secureSet(key, value),
            removeItem: (key) => api.secureRemove(key),
          },
          persistSession: true,
          autoRefreshToken: true,
          detectSessionInUrl: false,
        },
      })
    : null;

export const accountsEnabled = supabase !== null;
