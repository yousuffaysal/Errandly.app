import React from "react";
import ReactDOM from "react-dom/client";
import { ErrorBoundary } from "./components/ErrorBoundary";
import Workspace from "./components/Workspace";
import { installCrashHandlers } from "./crash";
import "./workspace.css";
import "./app.css";

installCrashHandlers();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <Workspace />
    </ErrorBoundary>
  </React.StrictMode>,
);
