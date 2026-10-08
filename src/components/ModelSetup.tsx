import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Download } from "lucide-react";
import { api, errorText, INSTALL_EVENT } from "../api";
import type { AiStatus, InstallEvent } from "../types";

/** First-run setup: downloads the shared base model and creates the four Errandly models. */
export function ModelSetup({ ai, onDone }: { ai: AiStatus; onDone: () => void }) {
  const [progress, setProgress] = useState<InstallEvent | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    const un = listen<InstallEvent>(INSTALL_EVENT, (e) => setProgress(e.payload));
    return () => {
      un.then((f) => f());
    };
  }, []);

  if (!ai.reachable) {
    return (
      <div className="ew-setup-card">
        <strong>Errandly’s local AI isn’t running.</strong>
        <span>Start it once with “brew services start ollama”. It will start on its own after that.</span>
      </div>
    );
  }

  async function install() {
    setError("");
    setProgress({ percent: 0, label: "Starting" });
    try {
      await api.installModels();
      onDone();
    } catch (e) {
      setError(errorText(e));
      setProgress(null);
    }
  }

  const missing = ai.personas.filter((p) => !p.installed).map((p) => p.name);
  return (
    <div className="ew-setup-card">
      {progress ? (
        <>
          <strong>{progress.label}…</strong>
          <div className="ew-setup-bar">
            <i style={{ width: `${progress.percent}%` }} />
          </div>
          <span>Setting up {missing.join(", ")} on this Mac. You can keep the window open while it works.</span>
        </>
      ) : (
        <>
          <strong>Set up Errandly’s models</strong>
          <span>
            {ai.baseInstalled
              ? "Arip, Shadow, Suf 4 and Howen 2 take a few seconds to prepare."
              : "One download (about 2.5 GB) powers all four models: Arip, Shadow, Suf 4 and Howen 2. Everything runs on this Mac."}
          </span>
          {error && <span className="ew-setup-error">{error}</span>}
          <button className="ew-followup ew-approve" onClick={install}>
            <Download size={14} />
            {ai.baseInstalled ? "Prepare models" : "Download and set up"}
          </button>
        </>
      )}
    </div>
  );
}
