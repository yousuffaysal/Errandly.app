# Errandly.app

## ActionDesk AI — Phase 0 prototype

Local-first agentic desktop assistant (see [prd.md](prd.md)). This prototype covers
PRD §27 Phase 0: interpret an instruction with a local model, generate a plan,
get approval, execute it with authorized local tools, verify the result, and undo it.

The one workflow implemented is the **file organizer** (PRD §11 / §34 Workflow A).

## Run

```sh
# one-time
brew install ollama
ollama serve &              # local inference, 127.0.0.1 only
ollama pull qwen2.5:3b      # ~2 GB; sized for 8 GB Macs
pnpm install

# dev
. "$HOME/.cargo/env"        # if cargo isn't on your PATH
pnpm tauri dev

# tests (Rust core: security, planner, executor, undo, crash recovery)
pnpm test:rust
```

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
src/                      React UI (App, TaskPanel, typed invoke wrappers)
src-tauri/src/
  agents/   plan.rs (tool registry + validation) planner.rs executor.rs verifier.rs
  ai/       Llm trait, ollama.rs (loopback-only client)
  security/ permissions.rs (path checks) validation.rs (folder names)
  storage/  sqlite.rs (migrations, WAL) repo.rs (typed queries)
  tools/    files.rs (scan, collision-free names, no-overwrite move)
```

Data: `~/Library/Application Support/ActionDesk/database/actiondesk.sqlite`.

## Not in Phase 0

Embedded llama.cpp runtime, model download manager, PDF/spreadsheet agents,
workspaces UI, storage manager, nested-folder scans (top level only, ≤1000 files),
App Sandbox + security-scoped bookmarks (needed before Mac App Store/sandboxed builds).
