# Errandly.app

## ActionDesk AI — Phase 0 prototype

Local-first agentic desktop assistant (see [prd.md](prd.md)). This prototype covers
PRD §27 Phase 0: interpret an instruction with a local model, generate a plan,
get approval, execute it with authorized local tools, verify the result, and undo it.

The interface is the Errandly workspace design (`Errandly_software_interface/app/workspace`),
wired to the real backend: conversations, a context panel with the attached folder and
instructions, real planning progress, and plan/result cards with Approve and Undo.

Each message is routed by the local model (organize / summarize / spreadsheet / chat).
The one workflow implemented is the **file organizer** (PRD §11 / §34 Workflow A); the
others reply honestly that they arrive in the next phase.

## Run

```sh
# one-time
brew install ollama
brew services start ollama   # local inference on 127.0.0.1, starts at login
pnpm install

# dev
pnpm tauri dev
```

On first launch the app offers **Download and set up**: it pulls the shared base model
once (~2.5 GB) and creates Errandly's four models from it. Cmd +/- zooms the window.

```sh
pnpm test:rust     # Rust core: security, planner, executor, undo, crash recovery, storage
pnpm test:ollama   # live: installs the models, then chats and plans with each of them
```

## Errandly's models

All four are created in Ollama from one open-weight base (Phi-4-mini, MIT; see
[NOTICE.md](NOTICE.md)), so the weights are downloaded once. Each has its own voice,
sampling and organizing style, defined in `src-tauri/src/ai/personas.rs`.

| Model | Role | Organizes by |
|---|---|---|
| **Arip** | The organizer: decisive, tidy | 3–6 broad familiar folders |
| **Shadow** | The careful one: privacy-first | Conservative; sensitive files into `Private`; unsure → left in place |
| **Suf 4** | The scholar | Course / subject, lectures, assignments, papers |
| **Howen 2** | The business partner | Invoices, receipts, contracts, reports, clients |

## Accounts (optional)

Sign-in uses Supabase Auth (free tier). Copy `.env.example` to `.env.local` and fill in
your project URL and anon key. Without them, the app runs fully local and hides sign-in.
Sessions are stored in the macOS Keychain. Accounts are only for identity: files, chats
and models stay on the Mac.

## Projects and history

Conversations live in projects (the PRD's workspaces). Projects and conversations can be
renamed and deleted; deleting never touches files on disk, and task records are kept for
the audit trail.

## How a task runs

1. **Grant** – the user picks a folder in the native picker (opened from Rust, so
   the webview can't grant itself paths). Grants live in SQLite; `/` and `~` are refused.
2. **Plan** – the model is asked two schema-constrained questions: which folder
   names to use, then which folder each *numbered* file goes in. It never writes a
   path. Out-of-range indices, unknown folders, duplicates and unsafe names are
   discarded and counted. Operations are built by deterministic code.
3. **Approve** – the plan is stored as `pending` steps; the UI shows the §18.3
   approval card. Rejecting it changes nothing.
4. **Execute** – each step is re-validated (inside the grant, no `..`, no symlinks,
   flat organize shape only), journaled `started`, performed, journaled `done`.
   Moves use `renamex_np(RENAME_EXCL)`, so an existing file is never overwritten.
5. **Verify** – the disk is checked against the journal; partial results are
   reported as `partially_completed`, never as success.
6. **Undo** – done steps are reversed newest-first; folders are removed only if
   the task created them and they are empty.

On startup, tasks left `executing` by a crash are reconciled from the journal + disk.

## Layout

```
src/                      React UI: components/ (Workspace, TaskCard, PersonaPicker, ProjectMenu,
                          ModelSetup, Account), auth.ts (Supabase),
                          workspace.css (the design, unchanged) + app.css (app additions)
src-tauri/src/
  agents/   router.rs (intent) plan.rs (tool registry + validation) planner.rs executor.rs verifier.rs
  ai/       Llm trait, ollama.rs (loopback-only client, model install), personas.rs
  security/ permissions.rs (path checks) validation.rs (folder names)
  storage/  sqlite.rs (migrations, WAL) repo.rs (tasks, grants) conversations.rs projects.rs
  keychain.rs  session storage in the macOS Keychain
  tools/    files.rs (scan, collision-free names, no-overwrite move)
```

Data: `~/Library/Application Support/Errandly/database/actiondesk.sqlite`. Keychain service: `studio.foxmen.errandly`.

## Not in Phase 0

Embedded llama.cpp runtime, model download manager, PDF/spreadsheet agents,
workspaces UI, storage manager, nested-folder scans (top level only, ≤1000 files),
App Sandbox + security-scoped bookmarks (needed before Mac App Store/sandboxed builds).
