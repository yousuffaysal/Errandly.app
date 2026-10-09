import { Mark } from "./Mark";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportCrash } from "../crash";

/**
 * Catches rendering errors anywhere in the app, so a bug shows a calm
 * recovery screen instead of a blank window. The error is recorded locally.
 */
export class ErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    reportCrash("ui", `${error.name}: ${error.message}\n${error.stack ?? ""}\n${info.componentStack ?? ""}`);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="ew ew-crash">
        <div className="ew-modal">
          <span className="ew-flower ew-modal-flower"><Mark /></span>
          <h2>Something went wrong.</h2>
          <p className="ew-modal-sub">
            Errandly hit an unexpected problem. Your conversations and files are safe. They’re stored on this Mac and
            nothing was changed.
          </p>
          <pre className="ew-crash-detail">{this.state.error.message}</pre>
          <button className="ew-modal-submit" onClick={() => window.location.reload()}>
            Reload Errandly
          </button>
        </div>
      </div>
    );
  }
}
