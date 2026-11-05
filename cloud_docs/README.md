# Cloud docs

Study materials every ShadowPrompt user's AI can search. Only `.md` and `.txt` files
under `materials/` are published. Anyone with the URL can read them.

## Adding or updating materials

1. On github.com, open this repo and go to `cloud_docs/materials/`.
2. Click **Add file → Upload files**, drag files in (folders are fine), then click **Commit changes**.
3. About a minute later the new version is live. Users pick it up on their next question.

To edit or delete a doc, open it on GitHub and use the pencil icon or the trash icon.

**PDF / Word / PowerPoint:** convert to Markdown first, once, on your computer:

```bash
pip install "markitdown[all]"
```

```bash
markitdown notes.pdf -o notes.md
```

Upload the `.md` file. Keep the original wherever you like, just not in this folder.

## One-time setup (Cloudflare Pages, free)

1. Sign in at dash.cloudflare.com, then go to **Workers & Pages → Create → Pages → Connect to Git**, and pick this repo.
2. Build settings:
   - Framework preset: **None**
   - Root directory: `cloud_docs`
   - Build command: `python build.py`
   - Build output directory: `public`
3. After the first deploy, go to **Settings → Builds → Build watch paths**, set the include path to `cloud_docs/*`, and save.
   This stops code-only pushes from triggering a rebuild.
4. Copy the site URL (`https://<name>.pages.dev`). The app's address is that URL followed by `/bundle.json`.
   Put it in `DEFAULT_CLOUD_URL` in `shadow_prompt_v3/src/config/schema.rs` and rebuild the app.
