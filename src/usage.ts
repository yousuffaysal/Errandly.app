import { invoke } from "@tauri-apps/api/core";
import { supabase } from "./auth";

// Opt-in anonymous usage counts (see storage/usage.rs). Each day's totals are
// sent under a random install id; re-sending a day replaces it, so retries are
// safe. Nothing is counted or sent unless the user turned sharing on.

interface UsageReport {
  installId: string;
  version: string;
  days: { day: string; opens: number; messages: number; tasks: number }[];
}

let timer: ReturnType<typeof setInterval> | undefined;

async function flush() {
  if (!supabase) return;
  const report = await invoke<UsageReport>("usage_pending");
  for (const d of report.days) {
    const { error } = await supabase.rpc("record_usage", {
      p_install_id: report.installId,
      p_day: d.day,
      p_version: report.version,
      p_opens: d.opens,
      p_messages: d.messages,
      p_tasks: d.tasks,
    });
    if (error) return; // Offline or not set up yet: counts wait on this Mac.
    await invoke("usage_sent", { day: d.day });
  }
}

/** Starts or stops sending, following the user's choice. */
export function setUsageSharing(enabled: boolean) {
  if (timer) clearInterval(timer);
  timer = undefined;
  if (!enabled || !supabase) return;
  flush().catch(() => {});
  timer = setInterval(() => flush().catch(() => {}), 3 * 60 * 60 * 1000);
}
