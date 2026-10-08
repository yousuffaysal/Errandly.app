import { useState } from "react";
import type { User } from "@supabase/supabase-js";
import { Check, MessageSquare, Pencil, Plus, Search, ShieldCheck, Trash2 } from "lucide-react";
import type { Conversation, Project } from "../../types";
import { ProfileRow } from "../Account";
import { ProjectMenu } from "../ProjectMenu";
import logo from "../../assets/errandly-logo.png";

export interface SidebarProps {
  open: boolean;
  projects: Project[];
  projectId: string;
  conversations: Conversation[];
  activeId: string | undefined;
  user: User | null;
  profileName: string;
  onNewChat: () => void;
  onSelectProject: (id: string) => void;
  onCreateProject: (name: string) => Promise<void>;
  onRenameProject: (id: string, name: string) => Promise<void>;
  onDeleteProject: (id: string) => Promise<void>;
  onSelect: (id: string) => void;
  onRename: (id: string, title: string) => Promise<boolean>;
  onDelete: (id: string) => Promise<void>;
  onSignIn: () => void;
  onSettings: () => void;
}

/** Brand, new conversation, search, project switcher and editable history. */
export function Sidebar(props: SidebarProps) {
  const [search, setSearch] = useState("");
  const [renaming, setRenaming] = useState<{ id: string; title: string } | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);
  const filtered = props.conversations.filter((c) => c.title.toLowerCase().includes(search.toLowerCase()));

  return (
    <aside className={`ew-sidebar ${props.open ? "is-open" : ""}`}>
      <div className="ew-brand">
        <img className="ew-logo" src={logo} alt="Errandly" draggable={false} />
        <span className="ew-beta">BETA</span>
      </div>
      <button className="ew-new" onClick={props.onNewChat}>
        <Plus size={17} />
        New conversation<kbd>＋</kbd>
      </button>
      <label className="ew-search">
        <Search size={16} />
        <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search conversations" aria-label="Search conversations" />
      </label>
      <div className="ew-space-label">PROJECT</div>
      <ProjectMenu
        projects={props.projects}
        activeId={props.projectId}
        onSelect={(id) => {
          setSearch("");
          props.onSelectProject(id);
        }}
        onCreate={props.onCreateProject}
        onRename={props.onRenameProject}
        onDelete={props.onDeleteProject}
      />
      <div className="ew-history-title">
        Conversations<span>{props.conversations.length}</span>
      </div>
      <nav className="ew-history" aria-label="Chat history">
        {filtered.map((c) =>
          renaming?.id === c.id ? (
            <form
              key={c.id}
              className="ew-rename"
              onSubmit={async (e) => {
                e.preventDefault();
                if (await props.onRename(c.id, renaming.title)) setRenaming(null);
              }}
            >
              <input
                autoFocus
                value={renaming.title}
                maxLength={80}
                aria-label="Conversation name"
                onChange={(e) => setRenaming({ id: c.id, title: e.target.value })}
                onKeyDown={(e) => e.key === "Escape" && setRenaming(null)}
              />
              <button type="submit" className="ew-icon" aria-label="Save name">
                <Check size={13} />
              </button>
            </form>
          ) : deleting === c.id ? (
            <div key={c.id} className="ew-confirm">
              <span>Delete this conversation? Files on your Mac stay untouched.</span>
              <div>
                <button className="ew-danger" onClick={() => props.onDelete(c.id).then(() => setDeleting(null))}>
                  Delete
                </button>
                <button onClick={() => setDeleting(null)}>Keep</button>
              </div>
            </div>
          ) : (
            <div key={c.id} className={`ew-history-row ${props.activeId === c.id ? "active" : ""}`}>
              <button className={props.activeId === c.id ? "active" : ""} onClick={() => props.onSelect(c.id)}>
                <MessageSquare size={15} />
                <span>{c.title}</span>
                {props.activeId === c.id && <span className="ew-selected-dot" />}
              </button>
              <span className="ew-row-actions">
                <button className="ew-icon" aria-label={`Rename ${c.title}`} onClick={() => setRenaming({ id: c.id, title: c.title })}>
                  <Pencil size={12} />
                </button>
                <button className="ew-icon" aria-label={`Delete ${c.title}`} onClick={() => setDeleting(c.id)}>
                  <Trash2 size={12} />
                </button>
              </span>
            </div>
          ),
        )}
        {filtered.length === 0 && <p className="ew-empty-search">No conversations found.</p>}
      </nav>
      <div className="ew-sidebar-bottom">
        <div className="ew-private">
          <ShieldCheck size={19} />
          <div>
            A space that’s yours<small>Chat history saved on this Mac</small>
          </div>
        </div>
        <ProfileRow user={props.user} profileName={props.profileName} onSignIn={props.onSignIn} onSettings={props.onSettings} />
      </div>
    </aside>
  );
}
