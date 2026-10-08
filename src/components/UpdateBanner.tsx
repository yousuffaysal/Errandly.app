import { useEffect, useState } from "react";
import { ArrowUpRight, X } from "lucide-react";
import { checkForUpdate, installUpdate, type UpdateState } from "../updates";

/** A quiet banner when a new version is ready. Checks shortly after launch. */
export function UpdateBanner({ enabled }: { enabled: boolean }) {
  const [state, setState] = useState<UpdateState>({ kind: "idle" });
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if (!enabled) return;
    const t = setTimeout(() => checkForUpdate().then(setState), 8000);
    return () => clearTimeout(t);
  }, [enabled]);

  if (dismissed || (state.kind !== "available" && state.kind !== "installing")) return null;
  return (
    <div className="ew-update" role="status">
      {state.kind === "available" ? (
        <>
          <span>
            Errandly {state.version} is ready.
          </span>
          <button
            className="ew-followup ew-approve"
            onClick={() => installUpdate(state.update, (percent) => setState({ kind: "installing", percent }))}
          >
            Restart to update
            <ArrowUpRight size={13} />
          </button>
          <button className="ew-icon" aria-label="Later" onClick={() => setDismissed(true)}>
            <X size={13} />
          </button>
        </>
      ) : (
        <span>Updating Errandly… {state.percent}%</span>
      )}
    </div>
  );
}
