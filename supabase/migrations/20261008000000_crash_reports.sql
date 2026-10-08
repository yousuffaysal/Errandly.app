-- Opt-in crash reports from the Errandly desktop app.
-- Anyone may INSERT (reports are anonymous and contain no user content);
-- nobody can read them through the public API. Read them in the Supabase
-- dashboard (Table editor), never from the app.
create table if not exists public.crash_reports (
  id          bigint generated always as identity primary key,
  created_at  timestamptz not null default now(),
  kind        text not null check (kind in ('ui', 'panic')),
  detail      text not null check (char_length(detail) <= 16000),
  app_version text not null check (char_length(app_version) <= 32)
);

alter table public.crash_reports enable row level security;

create policy "anyone can submit a crash report"
  on public.crash_reports for insert
  to anon, authenticated
  with check (true);
-- No select, update or delete policies: reports are write-only from the app.
