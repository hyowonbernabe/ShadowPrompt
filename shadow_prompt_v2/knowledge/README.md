# Knowledge folder

Drop subject-specific reference material here. Markdown files (`.md`, `.markdown`, `.txt`) are loaded at daemon startup and injected as a cached system block on every LLM call.

## Layout

```
knowledge/
  _default/          # always loaded (optional)
    background.md
  english/           # loaded when active_subjects = ["english"]
    grammar.md
    poetry.md
  biology/
    cells.md
```

Folders are subject names. Add as many as you like. Activate via `config.toml`:

```toml
[knowledge]
enabled = true
active_subjects = ["english", "biology"]
cache_ttl = "1h"
max_chars = 800000
```

## How caching works

Anthropic prompt caching marks the reference block once per first query, then reuses it at 10% of normal input cost on every subsequent query within the TTL window. Each cache hit refreshes the TTL.

5m TTL: writes are 1.25x. Good for short sessions.
1h TTL: writes are 2x. Good for exam-length sessions.

Loaded content under 1024 tokens (about 4000 chars) cannot be cached by Anthropic; you'll see a warning in the log. The block is still injected just paid at full price.

## Tips

- Files are concatenated alphabetically within each subject. Use prefixes like `01-intro.md`, `02-rules.md` to control order.
- Total content is capped at `max_chars` (default 800k chars or about 200k tokens). Excess is truncated with a log warning.
- Restart the daemon after editing files; no hot reload.
