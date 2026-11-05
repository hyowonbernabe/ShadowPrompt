# Cloud docs

Study materials every ShadowPrompt user's AI can search. Only `.md` and `.txt` files
under `materials/` are published. Anyone who has the URL can read them.

Hosting is GitHub Pages, built by GitHub Actions. Both are free for public repos.
The workflow is `.github/workflows/cloud_docs.yml`.

## Adding or updating materials

1. On github.com, open this repo and go to `cloud_docs/materials/` on `main`.
2. Click **Add file → Upload files**, drag in files (folders are fine), then click **Commit changes**.
3. About 1 minute later the **Actions** tab shows "Cloud docs" finished. Users pick up the new
   version on their next question (GitHub's cache can add up to 10 minutes).

To edit or delete a doc, open it on GitHub and use the pencil or trash icon.

**PDF / Word / PowerPoint:** convert to Markdown once, on your computer, then upload the `.md`:

```bash
pip install "markitdown[all]"
```

```bash
markitdown notes.pdf -o notes.md
```

## One-time setup

1. Go to repo **Settings → Pages**. Under **Build and deployment → Source**, pick **GitHub Actions**.
2. The bundle URL is `https://<owner>.github.io/<repo>/bundle.json`. It is set as
   `DEFAULT_CLOUD_URL` in `shadow_prompt_v3/src/config/schema.rs`.
