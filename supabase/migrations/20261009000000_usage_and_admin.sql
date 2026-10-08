-- Anonymous, opt-in usage statistics and the admin dashboard's data.
--
-- Privacy (PRD §24.2, §31.3): the app only sends daily counts under a random
-- install id that is not linked to any account. No names, emails, chats,
-- file names or paths. Users opt in in Settings; it is off by default.
--
-- Security: the tables are not readable through the public API. The app can
-- only call record_usage(); the dashboard can only call admin_stats(), which
-- checks that the signed-in user is listed in public.admins.

-- Who may see the dashboard. Add yourself after running this file:
--   insert into public.admins (email) values ('you@example.com');
create table if not exists public.admins (
  email text primary key
);
alter table public.admins enable row level security;
-- No policies: nobody can read or change this table through the API.

create table if not exists public.usage_daily (
  install_id  uuid not null,
  day         date not null,
  app_version text not null check (char_length(app_version) <= 32),
  opens       int  not null default 0 check (opens between 0 and 10000),
  messages    int  not null default 0 check (messages between 0 and 100000),
  tasks       int  not null default 0 check (tasks between 0 and 100000),
  updated_at  timestamptz not null default now(),
  primary key (install_id, day)
);
alter table public.usage_daily enable row level security;
-- No policies: written only through record_usage(), read only through admin_stats().

-- Called by the app. Sends a day's totals; re-sending the same day replaces
-- them, so retries never double-count. Days far in the past or future are refused.
create or replace function public.record_usage(
  p_install_id uuid, p_day date, p_version text, p_opens int, p_messages int, p_tasks int
) returns void
language plpgsql security definer set search_path = public as $$
begin
  if p_day < current_date - 30 or p_day > current_date + 1 then
    raise exception 'day out of range';
  end if;
  insert into usage_daily (install_id, day, app_version, opens, messages, tasks, updated_at)
  values (p_install_id, p_day, left(p_version, 32), p_opens, p_messages, p_tasks, now())
  on conflict (install_id, day) do update
    set app_version = excluded.app_version, opens = excluded.opens,
        messages = excluded.messages, tasks = excluded.tasks, updated_at = now();
end $$;
revoke all on function public.record_usage(uuid, date, text, int, int, int) from public;
grant execute on function public.record_usage(uuid, date, text, int, int, int) to anon, authenticated;

-- Called by the admin dashboard. Aggregates only; refuses anyone not in admins.
create or replace function public.admin_stats() returns json
language plpgsql security definer set search_path = public, auth as $$
declare
  result json;
begin
  if not exists (select 1 from admins where lower(email) = lower(auth.jwt() ->> 'email')) then
    raise exception 'not an admin';
  end if;
  select json_build_object(
    'users_total',     (select count(*) from auth.users),
    'users_new_7d',    (select count(*) from auth.users where created_at > now() - interval '7 days'),
    'users_active_7d', (select count(*) from auth.users where last_sign_in_at > now() - interval '7 days'),
    'users_by_provider', (select coalesce(json_object_agg(p, n), '{}') from (
        select coalesce(raw_app_meta_data ->> 'provider', 'email') p, count(*) n from auth.users group by 1) s),
    'signups_by_day',  (select coalesce(json_agg(json_build_object('day', d, 'count', n) order by d), '[]') from (
        select created_at::date d, count(*) n from auth.users
        where created_at > now() - interval '30 days' group by 1) s),
    'installs_today',  (select count(*) from usage_daily where day = current_date),
    'installs_7d',     (select count(distinct install_id) from usage_daily where day > current_date - 7),
    'installs_30d',    (select count(distinct install_id) from usage_daily where day > current_date - 30),
    'messages_7d',     (select coalesce(sum(messages), 0) from usage_daily where day > current_date - 7),
    'tasks_7d',        (select coalesce(sum(tasks), 0) from usage_daily where day > current_date - 7),
    'active_by_day',   (select coalesce(json_agg(json_build_object('day', d, 'installs', i, 'tasks', t) order by d), '[]') from (
        select day d, count(distinct install_id) i, sum(tasks) t from usage_daily
        where day > current_date - 30 group by 1) s),
    'versions',        (select coalesce(json_object_agg(v, n), '{}') from (
        select app_version v, count(distinct install_id) n from usage_daily
        where day > current_date - 7 group by 1) s),
    'crashes_7d',      (select count(*) from crash_reports where created_at > now() - interval '7 days'),
    'recent_crashes',  (select coalesce(json_agg(json_build_object('at', created_at, 'kind', kind, 'version', app_version,
                          'detail', left(detail, 400)) order by created_at desc), '[]')
                        from (select * from crash_reports order by created_at desc limit 10) c)
  ) into result;
  return result;
end $$;
revoke all on function public.admin_stats() from public;
grant execute on function public.admin_stats() to authenticated;
