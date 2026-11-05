# Access-code login — design

Date: 2026-11-06. Owner decisions in this document came from the conversation of that date.

## Intent

Instead of pasting an OpenRouter API key into the config, a user types a code (e.g. one they
chose themselves) and ShadowPrompt fetches the key stored under that code. Sign-up stores a key
under a new code. Codes travel on a USB stick / in someone's head; keys don't have to.

## Owner decisions

- **Backend: Supabase**, project `shadowprompt` (ref `ihvnbvxzuroogbcrzort`, org "ShadowPrompt",
  free plan), `https://ihvnbvxzuroogbcrzort.supabase.co`. The publishable key
  (`sb_publishable_ULGuLKiHYumaEKGm6tY2Qw_DvYEBDIZ`) is public by design and ships with the app.
  Claude's Supabase MCP access is project-scoped via `.mcp.json` (`project_ref=` URL parameter).
- **No lockout, no rate limiting, no minimum code strength.** A lockout would be a disaster during
  a test; picking a hard-to-guess code is the user's responsibility.
- **A code alone authorises everything** for its account (login, change key, change code, delete).
- **The dialog appears on every launch the user starts** — never "remember my choice". Restarts the
  app performs itself (restart hotkey, hook watchdog) skip it and keep the current key, so it can
  never pop up mid-test.
- **Saved login lives in `config.toml`** (one file for everything), in its own `[account]` section,
  plain text. No Windows DPAPI (owner: unreliable). The existing `[openrouter] api_key` is never
  overwritten by login/sign-up (project rule: never overwrite a user-set value).
- **Look:** dark, modern, rounded; same stack as the app (Rust + Win32, drawn by the app itself,
  no web view, no new UI framework). No taskbar entry, excluded from screen capture.

## Server (`supabase/migrations/20261106000000_access_codes.sql`, applied 2026-11-06)

- `private.accounts(code_hash, api_key_enc, created_at, updated_at)` in a non-exposed schema, RLS
  on with no policies, all privileges revoked from `anon`/`authenticated`.
- Codes stored only as SHA-256; each key encrypted with its own code (`pgp_sym_encrypt`).
- Four RPCs callable with the publishable key, `SECURITY DEFINER` because knowing the code *is*
  the authorisation: `sp_login(code) → key|null`, `sp_signup(code, key) → ok|taken|invalid`,
  `sp_update(code, new_code, new_key) → ok|not_found|taken|invalid` (null = unchanged),
  `sp_delete(code) → ok|not_found`.
- Verified live 2026-11-06: every RPC result above, table unreachable via the Data API, helper not
  callable. Security Advisor: 0 errors; its warnings are the four intended public RPCs plus
  Supabase's own `rls_auto_enable` event-trigger helper.

## Client (to build)

Launch dialog, every user-started launch:
- Saved login in `[account]` → **Continue as ••••644** (works offline, uses the saved key).
- Else key in `[openrouter] api_key` → **Continue with my key**.
- Always: **Log in with a code**, **Sign up** (sign-up pre-fills the config key if there is one).
- Login/sign-up writes `[account] code` + `api_key` into `config.toml` and continues.

Account screen on a new hotkey (`ctrl+shift+alt+k`, also Caps+Tab+K): change API key (checked
against OpenRouter first), change code, log out (clears `[account]`), delete account (confirm
twice). Key precedence at runtime: logged-in `[account]` key, else `[openrouter] api_key`. A 401 on
a saved account key → notice "Key out of date: log in again".
