import React from "react";
import ReactDOM from "react-dom/client";
import Workspace from "./components/Workspace";
import "./workspace.css";
import "./app.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Workspace />
  </React.StrictMode>,
);
