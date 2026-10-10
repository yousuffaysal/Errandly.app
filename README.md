# Errandly.app

## Errandly — beta

A local-first agentic desktop assistant for macOS (Apple Silicon), built from
[prd.md](prd.md). Everything runs on the Mac: the AI models, your files, your chats.

What it does today (the PRD's three P0 workflows):

| Ask | What happens |
|---|---|
| "Organize this folder by file type" | A plan of folders and moves → you approve → files move → checked on disk → undo anytime |
| "Summarize the documents in this folder" | PDF, Word, text and Markdown files are read locally and summarized; save as Markdown |
| "Analyze the sales spreadsheet" | Excel/CSV totals and groupings **calculated by code**; the model only explains them; save as an Excel report with a chart |
| "Rename my scans" (`/rename`) | Files with meaningless names (`scan_0034.pdf`, `IMG_4821.png`) are read and renamed after their contents, e.g. `Invoice - Acme Corporation - 2026-03-14.pdf`; only details found in the file make it into the name |
| "Clean my Downloads" (`/clean`) | Exact duplicates, installers for apps already installed (or 30+ days old), archives already unzipped and unfinished downloads go **to the Trash**, each with its reason; Undo brings them back |

Scanned PDFs and images of text (PNG, JPG, HEIC, …) are read with Apple's Vision text
recognition, on the Mac, so they can be summarized, asked about and renamed too.

Each message is routed to the right agent; anything else is a normal chat with
one of four local models. Work happens only in folders you add to a conversation.

## Run

```sh
pnpm install
pnpm tauri dev      # first run fetches the bundled AI runtime (~160 MB download, 41 MB on disk)
```

On first launch the app offers **Download and set up**: it pulls the shared base model
once (~2.5 GB) and creates Errandly's four models from it. Cmd +/- zooms the window.
No Homebrew needed: Errandly starts its own bundled runtime (or uses an Ollama that's
already running).

```sh
pnpm test          # UI smoke tests (real React app, fake backend)
pnpm test:rust     # Rust core: security, agents, executor, undo, crash recovery, storage
pnpm test:ollama   # live: installs the models, then chats, plans, summarizes and analyzes
```

CI runs all of the above (except the live model tests) plus clippy and dependency
audits on every push. Releases: see [docs/RELEASING.md](docs/RELEASING.md).

## Errandly's models

All four are created in Ollama from one open-weight base (Phi-4-mini, MIT; see
[NOTICE.md](NOTICE.md)), so the weights are downloaded once. Each has its own voice,
sampling and organizing style, defined in `src-tauri/src/ai/personas.rs`.

| Model | Role | Organizes by |
|---|---|---|
| **Ario** | The organizer: decisive, tidy | 3–6 broad familiar folders |
| **Shadow** | The careful one: privacy-first | Conservative; sensitive files into `Private`; unsure → left in place |
| **Suf 4** | The scholar | Course / subject, lectures, assignments, papers |
| **Howen 2** | The business partner | Invoices, receipts, contracts, reports, clients |

## Accounts (optional)

Sign-in uses Supabase Auth (free tier). Copy `.env.example` to `.env.local` and fill in
your project URL and anon key. Without them, the app runs fully local and hides sign-in.
The session is stored in Errandly's local database (owner-only file permissions), not the
Keychain, so signing in never shows a system password prompt. Accounts are only for identity: files, chats
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
src/                      React UI: components/Workspace.tsx (state) + components/workspace/
                          (Sidebar, TopBar, MessageList, Composer, ContextPanel), TaskCard,
                          ResultCards, Settings, Onboarding, Account; auth.ts, crash.ts, updates.ts
                          workspace.css (the design, unchanged) + app.css (app additions)
src-tauri/src/
  agents/   router.rs (intent) planner.rs executor.rs verifier.rs (organize)
            documents.rs (summaries, prompt-injection defusing) analyst.rs (spreadsheets)
  ai/       Llm trait, ollama.rs (loopback-only client, model install), personas.rs,
            runtime.rs (starts/stops the bundled Ollama)
  security/ permissions.rs (path checks) validation.rs (folder names)
  storage/  sqlite.rs (migrations, WAL) repo.rs (tasks, grants) conversations.rs projects.rs
  session.rs   sign-in session storage in the local database
  tools/    files.rs (scan, no-overwrite move) documents.rs (PDF/Word/text) spreadsheets.rs
  crash.rs     local crash logs; opt-in upload
```

Data: `~/Library/Application Support/Errandly/database/errandly.sqlite`.

## Not yet

Scanned-PDF text recognition (OCR), nested-folder scans (top level only, ≤1000 files),
Intel Macs, dark theme, App Sandbox + security-scoped bookmarks (needed for the Mac App Store).
