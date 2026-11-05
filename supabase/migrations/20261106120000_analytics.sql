-- ShadowPrompt usage analytics: our own small PostHog.
-- Design: docs/2026-11-06-analytics-design.md. Client: shadow_prompt_v3/src/analytics/mod.rs.
--
-- * Events live in the private (non-exposed) schema, like accounts.
-- * The app (publishable key) can only ADD events, via sp_track. It can't read anything back.
-- * Reading is sp_stats, gated by a dashboard password. Only the password's SHA-256 is stored
--   here; the password itself is in analytics/admin.secret on the owner's machine (gitignored).
-- * No user content is ever sent by the app (see the client module header).

create table if not exists private.events (
  id          bigint generated always as identity primary key,
  received_at timestamptz not null default now(),
  at          timestamptz,
  name        text not null,
  install_id  text,
  session_id  text,
  account     text,
  version     text,
  build       text,
  dev         boolean not null default false,
  os          text,
  country     text,
  props       jsonb not null default '{}'::jsonb
);
create index if not exists events_received_at on private.events (received_at);
create index if not exists events_name_received_at on private.events (name, received_at);
alter table private.events enable row level security; -- no policies = no direct access
revoke all on private.events from public, anon, authenticated;

create table if not exists private.analytics_admin (
  secret_hash text primary key
);
alter table private.analytics_admin enable row level security;
revoke all on private.analytics_admin from public, anon, authenticated;
insert into private.analytics_admin (secret_hash)
values ('d5aca6fd637dad88279132d4f4536f96f552fb14144fc2f1e8e532fc5e55d944')
on conflict do nothing;

-- Safe readers for client-sent props: anything not of the expected JSON type (or absurd) reads
-- as null, so one bad or fake event can never make sp_stats fail.
create or replace function private.jnum(p jsonb)
returns numeric
language sql
immutable
set search_path = ''
as $$
  select case when jsonb_typeof(p) = 'number' and (p::text)::numeric between 0 and 1e12
              then (p::text)::numeric end
$$;

create or replace function private.jbool(p jsonb)
returns boolean
language sql
immutable
set search_path = ''
as $$
  select case when jsonb_typeof(p) = 'boolean' then (p::text)::boolean end
$$;
revoke all on function private.jnum(jsonb) from public, anon, authenticated;
revoke all on function private.jbool(jsonb) from public, anon, authenticated;

-- Accepts a batch of up to 100 events. Returns how many were stored.
create or replace function public.sp_track(p_events jsonb)
returns integer
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_country text;
  v_count   integer;
begin
  if p_events is null or jsonb_typeof(p_events) <> 'array' or jsonb_array_length(p_events) > 100 then
    return 0;
  end if;
  -- Supabase sits behind Cloudflare, which tags each request with the caller's country.
  begin
    v_country := current_setting('request.headers', true)::json ->> 'cf-ipcountry';
  exception when others then
    v_country := null;
  end;
  insert into private.events (at, name, install_id, session_id, account, version, build, dev, os, country, props)
  select
    case when (e ->> 'at') ~ '^[0-9]{10,14}$' then to_timestamp((e ->> 'at')::bigint / 1000.0) end,
    left(e ->> 'name', 64),
    left(e ->> 'install', 64),
    left(e ->> 'session', 64),
    left(e ->> 'account', 64),
    left(e ->> 'version', 32),
    left(e ->> 'build', 16),
    coalesce(private.jbool(e -> 'dev'), false),
    left(e ->> 'os', 128),
    left(v_country, 8),
    case when jsonb_typeof(e -> 'props') = 'object' then e -> 'props' else '{}'::jsonb end
  from jsonb_array_elements(p_events) e
  where jsonb_typeof(e) = 'object'
    and coalesce(e ->> 'name', '') <> ''
    and length(e::text) <= 8000;
  get diagnostics v_count = row_count;
  return v_count;
exception when others then
  return 0; -- a malformed batch is dropped, never an error the app has to handle
end;
$$;

-- Everything the dashboard shows, in one call. Null if the password is wrong.
create or replace function public.sp_stats(p_secret text, p_days integer default 30, p_include_dev boolean default false)
returns jsonb
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_since timestamptz := now() - make_interval(days => greatest(1, least(coalesce(p_days, 30), 3650)));
  v_out   jsonb;
begin
  if not exists (
    select 1 from private.analytics_admin
    where secret_hash = encode(extensions.digest(convert_to(coalesce(p_secret, ''), 'UTF8'), 'sha256'), 'hex')
  ) then
    return null;
  end if;

  with ev as (
    select * from private.events
    where received_at >= v_since and (p_include_dev or not dev)
  )
  select jsonb_build_object(
    'since', v_since,
    'totals', (select jsonb_build_object(
        'events', count(*),
        'installs', count(distinct install_id),
        'sessions', count(distinct session_id),
        'accounts', count(distinct account),
        'launches', count(*) filter (where name = 'app_start' and not coalesce(private.jbool(props -> 'resumed'), false)),
        'quit_at_login', count(*) filter (where name = 'launch_quit'),
        'crashes', count(*) filter (where name = 'crash'),
        'model_calls', count(*) filter (where name = 'llm_call'),
        'tokens', coalesce(sum(private.jnum(props -> 'prompt_tokens') + private.jnum(props -> 'completion_tokens')) filter (where name = 'llm_call'), 0)
      ) from ev),
    'all_time_installs', (select count(distinct install_id) from private.events where p_include_dev or not dev),
    'daily', coalesce((select jsonb_agg(d order by d.day) from (
        select date_trunc('day', received_at)::date as day,
               count(distinct install_id) as installs,
               count(distinct session_id) as sessions,
               count(*) filter (where name = 'hotkey') as hotkeys,
               count(*) filter (where name = 'llm_call') as model_calls
        from ev group by 1) d), '[]'::jsonb),
    'new_installs', coalesce((select jsonb_agg(n order by n.day) from (
        select first_day as day, count(*) as installs from (
          select install_id, min(received_at)::date as first_day from private.events
          where p_include_dev or not dev group by install_id) f
        where first_day >= v_since::date group by first_day) n), '[]'::jsonb),
    'hotkeys', coalesce((select jsonb_agg(h order by h.uses desc) from (
        select props ->> 'action' as action, count(*) as uses, count(distinct install_id) as installs
        from ev where name = 'hotkey' group by 1) h), '[]'::jsonb),
    'features', coalesce((select jsonb_agg(f order by f.runs desc) from (
        select props ->> 'feature' as feature,
               count(*) as runs,
               count(*) filter (where name = 'feature_done' and private.jbool(props -> 'ok')) as ok,
               count(*) filter (where name = 'feature_done' and not private.jbool(props -> 'ok')) as failed,
               count(*) filter (where name = 'feature_cancelled') as cancelled,
               round(avg(private.jnum(props -> 'ms')) filter (where name = 'feature_done')) as avg_ms,
               round((percentile_cont(0.5) within group (order by private.jnum(props -> 'ms'))
                      filter (where name = 'feature_done'))::numeric) as median_ms
        from ev where name in ('feature_done', 'feature_cancelled') group by 1) f), '[]'::jsonb),
    'models', coalesce((select jsonb_agg(m order by m.calls desc) from (
        select coalesce(props ->> 'model', '(failed: ' || coalesce(props ->> 'lead_model', '?') || ')') as model,
               count(*) as calls,
               count(*) filter (where private.jbool(props -> 'ok')) as ok,
               round(avg(private.jnum(props -> 'ms'))) as avg_ms,
               coalesce(sum(private.jnum(props -> 'prompt_tokens')), 0) as prompt_tokens,
               coalesce(sum(private.jnum(props -> 'completion_tokens')), 0) as completion_tokens,
               coalesce(sum(private.jnum(props -> 'cached_tokens')), 0) as cached_tokens
        from ev where name = 'llm_call' group by 1) m), '[]'::jsonb),
    'errors', coalesce((select jsonb_agg(x order by x.count desc) from (
        select name as kind,
               coalesce(props ->> 'error', props ->> 'reason', props ->> 'message', props ->> 'result') as error,
               props ->> 'feature' as feature,
               count(*) as count, count(distinct install_id) as installs, max(received_at) as last_seen
        from ev
        where (name in ('feature_done', 'llm_call') and not coalesce(private.jbool(props -> 'ok'), true))
           or name in ('error_shown', 'crash')
           or (name = 'account' and props ->> 'result' <> 'ok')
        group by 1, 2, 3 order by count(*) desc limit 50) x), '[]'::jsonb),
    'accounts', coalesce((select jsonb_agg(a order by a.count desc) from (
        select props ->> 'action' as action, (props ->> 'result' = 'ok') as ok, count(*) as count
        from ev where name = 'account' group by 1, 2) a), '[]'::jsonb),
    'key_source', coalesce((select jsonb_object_agg(k, c) from (
        select props ->> 'key_source' as k, count(*) as c from ev
        where name = 'launch_choice' and props ->> 'key_source' is not null group by 1) s), '{}'::jsonb),
    'versions', coalesce((select jsonb_object_agg(k, c) from (
        select coalesce(version, '?') as k, count(distinct install_id) as c from ev group by 1) s), '{}'::jsonb),
    'builds', coalesce((select jsonb_object_agg(k, c) from (
        select coalesce(build, '?') as k, count(distinct install_id) as c from ev group by 1) s), '{}'::jsonb),
    'os', coalesce((select jsonb_object_agg(k, c) from (
        select coalesce(nullif(os, ''), '?') as k, count(distinct install_id) as c from ev group by 1) s), '{}'::jsonb),
    'countries', coalesce((select jsonb_object_agg(k, c) from (
        select coalesce(country, '?') as k, count(distinct install_id) as c from ev group by 1) s), '{}'::jsonb),
    'top_installs', coalesce((select jsonb_agg(t order by t.events desc) from (
        select install_id, max(country) as country, max(version) as version,
               count(distinct received_at::date) as active_days,
               count(*) filter (where name = 'hotkey') as hotkeys,
               count(*) as events, max(received_at) as last_seen
        from ev group by install_id order by count(*) desc limit 25) t), '[]'::jsonb),
    'recent', coalesce((select jsonb_agg(r) from (
        select received_at, name, install_id, country, version, props
        from ev order by received_at desc limit 100) r), '[]'::jsonb)
  ) into v_out;

  return v_out;
end;
$$;

revoke all on function public.sp_track(jsonb) from public;
revoke all on function public.sp_stats(text, integer, boolean) from public;
grant execute on function public.sp_track(jsonb) to anon, authenticated;
grant execute on function public.sp_stats(text, integer, boolean) to anon, authenticated;
