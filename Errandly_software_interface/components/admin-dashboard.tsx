'use client';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { createClient, type Session } from '@supabase/supabase-js';

// Errandly's private dashboard. Downloads come from GitHub's public release
// counts. Everything else comes from Supabase's admin_stats(), which only
// answers for emails listed in the public.admins table.

const SUPABASE_URL = process.env.NEXT_PUBLIC_SUPABASE_URL ?? '';
const SUPABASE_KEY = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? '';
const REPO = 'yousuffaysal/Errandly.app';

interface Stats {
  users_total: number; users_new_7d: number; users_active_7d: number;
  users_by_provider: Record<string, number>;
  signups_by_day: { day: string; count: number }[];
  installs_today: number; installs_7d: number; installs_30d: number;
  messages_7d: number; tasks_7d: number;
  active_by_day: { day: string; installs: number; tasks: number }[];
  versions: Record<string, number>;
  crashes_7d: number;
  recent_crashes: { at: string; kind: string; version: string; detail: string }[];
}
interface ReleaseCounts { tag: string; published: string; downloads: number; updateChecks: number }

const fmt = (n: number) => n.toLocaleString();

export default function AdminDashboard() {
  const supabase = useMemo(() => (SUPABASE_URL && SUPABASE_KEY ? createClient(SUPABASE_URL, SUPABASE_KEY) : null), []);
  const [session, setSession] = useState<Session | null>(null);
  const [stats, setStats] = useState<Stats | null>(null);
  const [releases, setReleases] = useState<ReleaseCounts[] | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!supabase) return;
    supabase.auth.getSession().then(({ data }) => setSession(data.session));
    const { data } = supabase.auth.onAuthStateChange((_e, s) => setSession(s));
    return () => data.subscription.unsubscribe();
  }, [supabase]);

  const load = useCallback(async () => {
    if (!supabase) return;
    setLoading(true);
    setError('');
    const [rpc, gh] = await Promise.all([
      supabase.rpc('admin_stats'),
      fetch(`https://api.github.com/repos/${REPO}/releases`).then((r) => (r.ok ? r.json() : [])).catch(() => []),
    ]);
    if (rpc.error) setError(rpc.error.message.includes('not an admin') ? 'This account is not an admin.' : rpc.error.message);
    else setStats(rpc.data as Stats);
    setReleases(
      (gh as { tag_name: string; published_at: string | null; draft: boolean; assets: { name: string; download_count: number }[] }[])
        .filter((r) => !r.draft)
        .map((r) => ({
          tag: r.tag_name,
          published: r.published_at ?? '',
          // Each download of the installer counts once (the versioned and fixed-name copies).
          downloads: r.assets.filter((a) => a.name.endsWith('.dmg')).reduce((s, a) => s + a.download_count, 0),
          // Installed apps fetch latest.json when they check for updates, roughly once per launch.
          updateChecks: r.assets.find((a) => a.name === 'latest.json')?.download_count ?? 0,
        })),
    );
    setLoading(false);
  }, [supabase]);

  useEffect(() => {
    if (session) load();
  }, [session, load]);

  if (!supabase) {
    return <main className="admin"><div className="admin-card"><h1>Admin</h1><p>Set NEXT_PUBLIC_SUPABASE_URL and NEXT_PUBLIC_SUPABASE_ANON_KEY to use this page.</p></div></main>;
  }
  if (!session) {
    return (
      <main className="admin admin-center">
        <div className="admin-card">
          <h1>Errandly admin</h1>
          <p>Sign in with an admin account.</p>
          <button
            className="button"
            onClick={() => supabase.auth.signInWithOAuth({ provider: 'google', options: { redirectTo: `${window.location.origin}/admin/` } })}
          >
            Continue with Google
          </button>
        </div>
      </main>
    );
  }

  const totalDownloads = releases?.reduce((s, r) => s + r.downloads, 0) ?? 0;
  const totalChecks = releases?.reduce((s, r) => s + r.updateChecks, 0) ?? 0;
  const maxActive = Math.max(1, ...(stats?.active_by_day.map((d) => d.installs) ?? [0]));
  const maxSignups = Math.max(1, ...(stats?.signups_by_day.map((d) => d.count) ?? [0]));

  return (
    <main className="admin">
      <header className="admin-head">
        <div>
          <h1>Errandly admin</h1>
          <p>{session.user.email}</p>
        </div>
        <div className="admin-actions">
          <button className="button small" onClick={load} disabled={loading}>{loading ? 'Loading…' : 'Refresh'}</button>
          <button className="text-button" onClick={() => supabase.auth.signOut()}>Sign out</button>
        </div>
      </header>
      {error && <div className="admin-error">{error}</div>}

      <section className="admin-grid">
        <Tile label="Downloads (all versions)" value={totalDownloads} hint="Installer downloads from GitHub" />
        <Tile label="Active installs · 7 days" value={stats?.installs_7d} hint="People who shared usage" />
        <Tile label="Tasks completed · 7 days" value={stats?.tasks_7d} hint="Plans, summaries, reports" />
        <Tile label="Accounts" value={stats?.users_total} hint={stats ? `+${stats.users_new_7d} this week` : ''} />
        <Tile label="Active today" value={stats?.installs_today} hint="Installs that shared usage" />
        <Tile label="Messages · 7 days" value={stats?.messages_7d} />
        <Tile label="Update checks (all time)" value={totalChecks} hint="≈ app launches with updates on" />
        <Tile label="Crashes · 7 days" value={stats?.crashes_7d} hint="From users who opted in" />
      </section>

      <section className="admin-two">
        <div className="admin-card">
          <h2>Active installs per day</h2>
          <Bars rows={(stats?.active_by_day ?? []).map((d) => [d.day.slice(5), d.installs, `${d.installs} installs · ${d.tasks} tasks`])} max={maxActive} />
        </div>
        <div className="admin-card">
          <h2>New accounts per day</h2>
          <Bars rows={(stats?.signups_by_day ?? []).map((d) => [d.day.slice(5), d.count, `${d.count} sign-ups`])} max={maxSignups} />
        </div>
      </section>

      <section className="admin-two">
        <div className="admin-card">
          <h2>Downloads by version</h2>
          <table>
            <thead><tr><th>Version</th><th>Published</th><th>Downloads</th><th>Update checks</th></tr></thead>
            <tbody>
              {(releases ?? []).map((r) => (
                <tr key={r.tag}><td>{r.tag}</td><td>{r.published.slice(0, 10)}</td><td>{fmt(r.downloads)}</td><td>{fmt(r.updateChecks)}</td></tr>
              ))}
              {releases?.length === 0 && <tr><td colSpan={4}>No published releases yet.</td></tr>}
            </tbody>
          </table>
        </div>
        <div className="admin-card">
          <h2>Accounts and versions</h2>
          <table>
            <tbody>
              {Object.entries(stats?.users_by_provider ?? {}).map(([p, n]) => <tr key={p}><td>Sign-up with {p}</td><td>{fmt(n)}</td></tr>)}
              <tr><td>Signed in this week</td><td>{fmt(stats?.users_active_7d ?? 0)}</td></tr>
              {Object.entries(stats?.versions ?? {}).map(([v, n]) => <tr key={v}><td>Running {v} (7 days)</td><td>{fmt(n)}</td></tr>)}
            </tbody>
          </table>
        </div>
      </section>

      <section className="admin-card">
        <h2>Recent crashes</h2>
        {(stats?.recent_crashes ?? []).length === 0 ? <p>None reported. 🎉</p> : (
          <ul className="admin-crashes">
            {stats!.recent_crashes.map((c, i) => (
              <li key={i}><strong>{new Date(c.at).toLocaleString()} · {c.kind} · {c.version}</strong><pre>{c.detail}</pre></li>
            ))}
          </ul>
        )}
      </section>
      <p className="admin-note">Usage numbers come only from people who chose to share anonymous statistics, so real usage is higher.</p>
    </main>
  );
}

function Tile({ label, value, hint }: { label: string; value: number | undefined; hint?: string }) {
  return (
    <div className="admin-tile">
      <span>{label}</span>
      <strong>{value === undefined ? '–' : fmt(value)}</strong>
      {hint && <small>{hint}</small>}
    </div>
  );
}

function Bars({ rows, max }: { rows: [string, number, string][]; max: number }) {
  if (rows.length === 0) return <p className="admin-empty">No data yet.</p>;
  return (
    <div className="admin-bars">
      {rows.map(([label, n, title]) => (
        <div key={label} title={title}>
          <i style={{ height: `${Math.max(4, (n / max) * 100)}%` }} />
          <small>{label}</small>
        </div>
      ))}
    </div>
  );
}
