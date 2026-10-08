import { ChevronRight, Download, PanelLeftClose, PanelRight } from "lucide-react";

export function TopBar({ projectName, title, status, onMenu, onExport, contextOpen, onToggleContext }: {
  projectName: string;
  title: string;
  status: { tone: "" | "is-warn" | "is-off"; label: string };
  onMenu: () => void;
  onExport: () => void;
  contextOpen: boolean;
  onToggleContext: () => void;
}) {
  return (
    <div className="ew-topbar">
      <button className="ew-mobile-menu ew-icon" aria-label="Open navigation" onClick={onMenu}>
        <PanelLeftClose size={18} />
      </button>
      <span className="ew-breadcrumb">
        {projectName}
        <ChevronRight size={13} />
      </span>
      <span className="ew-chat-title">{title}</span>
      <div className="ew-top-actions">
        <span className={`ew-preview-dot ${status.tone}`} /> <span className="ew-preview-label">{status.label}</span>
        <button className="ew-icon" onClick={onExport} title="Export conversation" aria-label="Export conversation">
          <Download size={17} />
        </button>
        <button
          className={`ew-icon ${contextOpen ? "selected" : ""}`}
          onClick={onToggleContext}
          aria-label="Toggle conversation context"
          aria-expanded={contextOpen}
        >
          <PanelRight size={18} />
        </button>
      </div>
    </div>
  );
}
