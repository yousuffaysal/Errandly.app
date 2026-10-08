import { invoke } from "@tauri-apps/api/core";
import { supabase } from "./auth";

// Crash reports: always written to ~/Library/Logs/Errandly on this Mac; only
// uploaded when the user opts in (Settings → Privacy and data). See crash.rs.

let sending = false;

/** Turns uploading on or off, and uploads anything waiting when turned on. */
export function setCrashSending(enabled: boolean) {
  sending = enabled && supabase !== null;
  if (sending) flushCrashes().catch(() => {});
}

export function reportCrash(kind: "ui", detail: string) {
  invoke("log_crash", { kind, detail })
    .then(() => (sending ? flushCrashes() : undefined))
    .catch(() => {});
}

async function flushCrashes() {
  if (!supabase) return;
  const pending = await invoke<{ name: string; kind: string; detail: string }[]>("unsent_crashes");
  for (const c of pending) {
    const { error } = await supabase.from("crash_reports").insert({
      kind: c.kind,
      detail: c.detail,
      app_version: __APP_VERSION__,
    });
    if (error) return; // Try again next time; logs stay on disk.
    await invoke("mark_crash_sent", { name: c.name });
  }
}

/** Records errors that escape React (async code, event handlers). */
export function installCrashHandlers() {
  window.addEventListener("error", (e) => reportCrash("ui", `${e.message}\n${e.error?.stack ?? ""}`));
  window.addEventListener("unhandledrejection", (e) => {
    const r = e.reason;
    reportCrash("ui", r instanceof Error ? `${r.message}\n${r.stack ?? ""}` : `Unhandled rejection: ${String(r)}`);
  });
}
