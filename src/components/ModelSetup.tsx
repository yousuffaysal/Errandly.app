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
    return ai.runtimeBundled ? (
      <div className="ew-setup-card">
        <strong>Starting Errandly’s local AI…</strong>
        <span>This takes a few seconds after opening the app.</span>
      </div>
    ) : (
      <div className="ew-setup-card">
        <strong>Errandly’s AI engine isn’t running.</strong>
        <span>Quit and reopen Errandly. If this keeps happening, download Errandly again from the website.</span>
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
              ? "Ario, Shadow, Suf 4 and Howen 2 take a few seconds to prepare."
              : "One download (about 2.5 GB) powers all four models: Ario, Shadow, Suf 4 and Howen 2. Everything runs on this Mac."}
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
