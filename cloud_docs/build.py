"""Bundle cloud_docs/materials/ into public/bundle.json for GitHub Pages.

Stdlib only. Run from cloud_docs/ (Pages root dir): `python build.py`.
Each doc records its text, a sha256 of that text with CRLF normalised to LF (so a Windows
checkout of the same file hashes identically on the client), and the unix time of the last
git commit that touched it. See docs/superpowers/specs/2026-11-05-cloud-knowledge-design.md.
"""
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
MATERIALS = HERE / "materials"
OUT = HERE / "public" / "bundle.json"
EXTS = {".md", ".txt"}


def git(*args):
    return subprocess.run(["git", *args], cwd=HERE, capture_output=True, text=True).stdout.strip()


def main():
    now = int(time.time())
    if git("rev-parse", "--is-shallow-repository") == "true":
        git("fetch", "--unshallow")  # commit dates need full history; failure just means fallback dates
    docs = {}
    for path in sorted(MATERIALS.rglob("*")):
        rel = path.relative_to(MATERIALS).as_posix()
        if not path.is_file() or path.suffix.lower() not in EXTS or rel == "README.md":
            continue
        try:
            text = path.read_bytes().decode("utf-8").replace("\r\n", "\n")
        except UnicodeDecodeError:
            print(f"skipped (not UTF-8): {rel}", file=sys.stderr)
            continue
        modified = git("log", "-1", "--format=%ct", "--", str(path))
        docs[rel] = {
            "text": text,
            "sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
            "modified": int(modified) if modified.isdigit() else now,
        }
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps({"built_at": now, "docs": docs}, ensure_ascii=False), encoding="utf-8")
    print(f"bundled {len(docs)} docs -> {OUT}")


if __name__ == "__main__":
    main()
