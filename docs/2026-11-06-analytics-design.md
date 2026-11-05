# Usage analytics — design

Date: 2026-11-06. Owner asked for "PostHog but ours": GitHub download counts plus in-app usage
("what do they use the most"), shipped inside v3.3.0 with no change to how the app behaves.
Owner's only stated preference: the more information, the better. Everything else below was
Claude's call and is marked as such.

## Decisions

- **Backend: the existing Supabase project** (same one as access-code login). No third-party
  analytics service. *(Claude's call: already shipped, free, no new dependency.)*
- **Write-only from the app.** The app's public key can only call `sp_track` to add events. It
  can't read any event back. Reading is `sp_stats`, gated by a dashboard password whose SHA-256
  is in the migration; the password itself lives only in `analytics/admin.secret` (gitignored).
- **No user content, ever.** Never sent: what the user typed, copied or captured, questions,
  answers, form contents, page addresses, the access code, API keys. Sent: which hotkey/feature
  ran, how long it took, whether it worked, which model answered, token counts, app
  version/build, Windows version, country (from Cloudflare's request header, server side).
  Failures are sent as a fixed category (the same short reason the overlay shows, plus the HTTP
  code), never the raw error text: raw errors can quote the model's answer, OpenRouter echoing
  the prompt, or form contents. Crashes send only file:line, not the panic text.
  *(Claude's call: "more info" stops at anything that would make the data itself sensitive.)*
- **Identity without files.** Install = a hash of the machine's `MachineGuid` (no file written,
  so self-delete / panic-wipe leave nothing behind). Account = the same SHA-256 the server already
  keys accounts by, never the code. Session = per process launch.
- **Never in the way.** `track` only pushes to an in-memory queue (capped at 1000, oldest dropped).
  A background thread with its own small runtime sends batches of up to 100 every 10 s, with 3
  tries (1 s, 2 s backoff) per round. Server down = events wait or drop; the app doesn't notice.
- **Exits.** Panic-kill and self-delete never wait for analytics (speed under pressure is the
  point), so their last few seconds of events are lost. Closing the launch dialog waits at most
  3 s to send. A crash waits at most 1 s, and not at all when it happens on the keyboard/mouse
  hook thread (a wait there would lag input across the whole PC). The panic hook only *tries*
  analytics locks, so a crash while one is held skips the report instead of hanging.
- **Server never trusts event values.** `sp_stats` reads numbers/booleans from props through
  type-checked helpers, so one malformed or fake event can't break the dashboard; `sp_track`
  drops only the bad fields, not the batch.
- **No opt-out switch.** *(Claude's call, YAGNI; add a config flag if anyone asks.)*
- **Dashboard:** `analytics/dashboard.html`, a single static page opened locally. It calls
  `sp_stats` and GitHub's public releases API (download counts per release asset).

## Known limits

- Anyone holding the public key can post fake events (same trust model as the login RPCs). Batch
  size and field lengths are capped server side; there is no volume cap.
- The dashboard password's unsalted SHA-256 is in the committed migration. Safe only because the
  password is 43 random characters; change it with an `insert` into `private.analytics_admin`
  (and delete the old row) in the SQL editor, never by editing the migration.
- No automatic cleanup of old events; on the free plan that is roughly a million events before
  storage matters.
- GitHub only counts downloads of release files (the zip, which `install.ps1` also fetches), not
  clones or views of the install script itself.
