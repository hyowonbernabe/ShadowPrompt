-- ShadowPrompt access codes: a code unlocks an OpenRouter API key.
-- Design: docs/superpowers/specs/2026-11-06-access-code-login-design.md
--
-- * The table lives in a private (non-exposed) schema: the Data API can't read or list it.
-- * The app (anon / publishable key) can only call the four functions below.
-- * Codes are stored only as a SHA-256 hash; each API key is encrypted with its own code,
--   so a database dump reveals neither codes nor keys.
-- * No lockout / rate limiting on purpose (owner decision 2026-11-06): code strength is the
--   user's choice.
-- * The functions are SECURITY DEFINER because they ARE the access control: knowing the code
--   is the authorisation. They take no auth.uid() — there are no Supabase Auth users here.

create extension if not exists pgcrypto with schema extensions;

create schema if not exists private;
revoke all on schema private from public, anon, authenticated;

create table if not exists private.accounts (
  code_hash   text primary key,
  api_key_enc bytea not null,
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now()
);
alter table private.accounts enable row level security; -- defence in depth; no policies = no access
revoke all on private.accounts from public, anon, authenticated;

create or replace function private.code_hash(p_code text)
returns text
language sql
immutable
set search_path = ''
as $$
  select encode(extensions.digest(convert_to(p_code, 'UTF8'), 'sha256'), 'hex')
$$;

-- Returns the API key for a code, or null if the code doesn't exist.
create or replace function public.sp_login(p_code text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_enc bytea;
begin
  if p_code is null or length(p_code) = 0 or length(p_code) > 200 then
    return null;
  end if;
  select api_key_enc into v_enc from private.accounts where code_hash = private.code_hash(p_code);
  if v_enc is null then
    return null;
  end if;
  return extensions.pgp_sym_decrypt(v_enc, p_code);
end;
$$;

-- 'ok' | 'taken' | 'invalid'
create or replace function public.sp_signup(p_code text, p_api_key text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
begin
  if p_code is null or length(p_code) = 0 or length(p_code) > 200
     or p_api_key is null or length(p_api_key) = 0 or length(p_api_key) > 500 then
    return 'invalid';
  end if;
  insert into private.accounts (code_hash, api_key_enc)
  values (private.code_hash(p_code), extensions.pgp_sym_encrypt(p_api_key, p_code));
  return 'ok';
exception when unique_violation then
  return 'taken';
end;
$$;

-- Change the key and/or the code. Pass null for whatever stays the same.
-- 'ok' | 'not_found' | 'taken' | 'invalid'
create or replace function public.sp_update(p_code text, p_new_code text, p_new_api_key text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_key  text;
  v_code text := coalesce(nullif(p_new_code, ''), p_code);
begin
  v_key := public.sp_login(p_code);
  if v_key is null then
    return 'not_found';
  end if;
  v_key := coalesce(nullif(p_new_api_key, ''), v_key);
  if length(v_code) > 200 or length(v_key) > 500 then
    return 'invalid';
  end if;
  update private.accounts
     set code_hash   = private.code_hash(v_code),
         api_key_enc = extensions.pgp_sym_encrypt(v_key, v_code),
         updated_at  = now()
   where code_hash = private.code_hash(p_code);
  return 'ok';
exception when unique_violation then
  return 'taken';
end;
$$;

-- 'ok' | 'not_found'
create or replace function public.sp_delete(p_code text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
begin
  delete from private.accounts where code_hash = private.code_hash(p_code);
  if found then
    return 'ok';
  end if;
  return 'not_found';
end;
$$;

-- Only these four are callable by the app; nothing else is.
revoke all on function private.code_hash(text) from public, anon, authenticated;
revoke all on function public.sp_login(text) from public;
revoke all on function public.sp_signup(text, text) from public;
revoke all on function public.sp_update(text, text, text) from public;
revoke all on function public.sp_delete(text) from public;
grant execute on function public.sp_login(text) to anon, authenticated;
grant execute on function public.sp_signup(text, text) to anon, authenticated;
grant execute on function public.sp_update(text, text, text) to anon, authenticated;
grant execute on function public.sp_delete(text) to anon, authenticated;
