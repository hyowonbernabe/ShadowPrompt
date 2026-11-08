-- Cloud-synced config: each account can store its config.toml (minus [account] credentials)
-- encrypted with the user's code, same pgcrypto pattern as api_key_enc.
-- Design: lets users keep their settings (hotkeys, visuals, models) across installs/updates.

alter table private.accounts add column if not exists config_enc bytea;

-- Save config for an account. Replaces any existing config. 'ok' | 'not_found'
create or replace function public.sp_save_config(p_code text, p_config text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
begin
  if p_code is null or length(p_code) = 0 or length(p_code) > 200
     or p_config is null or length(p_config) = 0 or length(p_config) > 50000 then
    return 'not_found';
  end if;
  update private.accounts
     set config_enc = extensions.pgp_sym_encrypt(p_config, p_code),
         updated_at = now()
   where code_hash = private.code_hash(p_code);
  if found then
    return 'ok';
  end if;
  return 'not_found';
end;
$$;

-- Load config for an account. Returns the decrypted TOML string, or null if no config stored.
create or replace function public.sp_load_config(p_code text)
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
  select config_enc into v_enc from private.accounts where code_hash = private.code_hash(p_code);
  if v_enc is null then
    return null;
  end if;
  return extensions.pgp_sym_decrypt(v_enc, p_code);
end;
$$;

revoke all on function public.sp_save_config(text, text) from public;
revoke all on function public.sp_load_config(text) from public;
grant execute on function public.sp_save_config(text, text) to anon, authenticated;
grant execute on function public.sp_load_config(text) to anon, authenticated;
