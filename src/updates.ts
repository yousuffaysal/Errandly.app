import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// Signed auto-updates from GitHub Releases. Updates are verified against the
// public key in tauri.conf.json before installing; unsigned or tampered
// updates are refused by the updater itself.

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "available"; version: string; notes: string; update: Update }
  | { kind: "installing"; percent: number }
  | { kind: "error"; message: string };

export async function checkForUpdate(): Promise<UpdateState> {
  try {
    const update = await check();
    if (!update) return { kind: "current" };
    return { kind: "available", version: update.version, notes: update.body ?? "", update };
  } catch (e) {
    // Offline, or no release published yet: not worth alarming anyone.
    return { kind: "error", message: e instanceof Error ? e.message : String(e) };
  }
}

export async function installUpdate(update: Update, onProgress: (percent: number) => void) {
  let total = 0;
  let done = 0;
  await update.downloadAndInstall((event) => {
    if (event.event === "Started") total = event.data.contentLength ?? 0;
    if (event.event === "Progress") {
      done += event.data.chunkLength;
      if (total) onProgress(Math.round((done / total) * 100));
    }
  });
  await relaunch();
}
