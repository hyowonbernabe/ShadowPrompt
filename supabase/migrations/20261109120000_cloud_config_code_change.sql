-- Changing an account's code must also re-encrypt its synced config, which is encrypted with the
-- code (20261109000000_cloud_config.sql). Before this, sp_update re-keyed only api_key_enc, so
-- after a code change sp_load_config could no longer decrypt config_enc.
-- Also: an oversized config now reports 'too_large' instead of looking like a missing account.

create or replace function public.sp_update(p_code text, p_new_code text, p_new_api_key text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_key  text;
  v_cfg  text;
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
  begin
    select extensions.pgp_sym_decrypt(config_enc, p_code) into v_cfg
      from private.accounts where code_hash = private.code_hash(p_code);
  exception when others then
    -- A config already left unreadable by an earlier code change: drop it rather than block
    -- the code change. The next Settings save uploads a fresh one.
    v_cfg := null;
  end;
  update private.accounts
     set code_hash   = private.code_hash(v_code),
         api_key_enc = extensions.pgp_sym_encrypt(v_key, v_code),
         config_enc  = case when v_cfg is null then null else extensions.pgp_sym_encrypt(v_cfg, v_code) end,
         updated_at  = now()
   where code_hash = private.code_hash(p_code);
  return 'ok';
exception when unique_violation then
  return 'taken';
end;
$$;

-- 'ok' | 'not_found' | 'too_large'
create or replace function public.sp_save_config(p_code text, p_config text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
begin
  if p_code is null or length(p_code) = 0 or length(p_code) > 200
     or p_config is null or length(p_config) = 0 then
    return 'not_found';
  end if;
  if length(p_config) > 50000 then
    return 'too_large';
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

revoke all on function public.sp_update(text, text, text) from public;
revoke all on function public.sp_save_config(text, text) from public;
grant execute on function public.sp_update(text, text, text) to anon, authenticated;
grant execute on function public.sp_save_config(text, text) to anon, authenticated;
