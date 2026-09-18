// Memory capability — design doc §4. On-demand, sandboxed to one `knowledge/` folder, no
// preloading/RAG, no `active_subjects` concept (dropped entirely — `list_docs` just enumerates
// the whole tree).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::json;

use crate::llm::messages::ToolDef;
use crate::llm::Tool;

pub struct KnowledgeStore {
    root: PathBuf,
    max_doc_bytes: usize,
}

impl KnowledgeStore {
    pub fn new(root: PathBuf, max_doc_bytes: usize) -> Self {
        Self { root, max_doc_bytes }
    }

    /// Enumerates every file under the sandboxed root, relative paths only. No path outside
    /// `root` is ever reachable — see `resolve` below.
    fn list_docs(&self) -> anyhow::Result<Vec<String>> {
        let mut out = Vec::new();
        walk(&self.root, &self.root, &mut out)?;
        out.sort();
        Ok(out)
    }

    /// Resolves a model-supplied relative name to a real path, rejecting anything that would
    /// escape `self.root` (no `..`, no absolute paths, no symlink traversal out of the sandbox).
    fn resolve(&self, name: &str) -> anyhow::Result<PathBuf> {
        let candidate = self.root.join(name);
        let canon_root = self.root.canonicalize()?;
        let canon_candidate = candidate
            .canonicalize()
            .map_err(|_| anyhow::anyhow!("no such document: {name}"))?;
        if !canon_candidate.starts_with(&canon_root) {
            anyhow::bail!("path escapes knowledge/ sandbox: {name}");
        }
        Ok(canon_candidate)
    }

    fn read_doc(&self, name: &str) -> anyhow::Result<String> {
        let path = self.resolve(name)?;
        let bytes = std::fs::read(&path)?;
        let capped = &bytes[..bytes.len().min(self.max_doc_bytes)];
        let truncated = bytes.len() > self.max_doc_bytes;
        let mut text = String::from_utf8_lossy(capped).into_owned();
        if truncated {
            text.push_str(&format!(
                "\n\n[...{} bytes truncated, file is larger than the {}-byte cap...]",
                bytes.len() - self.max_doc_bytes,
                self.max_doc_bytes
            ));
        }
        Ok(text)
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

pub struct ListDocsTool(pub std::sync::Arc<KnowledgeStore>);
pub struct ReadDocTool(pub std::sync::Arc<KnowledgeStore>);

#[async_trait]
impl Tool for ListDocsTool {
    fn name(&self) -> &'static str {
        "list_docs"
    }

    fn definition(&self) -> ToolDef {
        crate::llm::messages::ToolDef {
            kind: "function",
            function: crate::llm::messages::ToolFunctionDef {
                name: self.name().to_string(),
                description: "List every document available in the knowledge folder.".to_string(),
                parameters: json!({"type": "object", "properties": {}}),
            },
        }
    }

    async fn execute(&self, _args_json: &str) -> anyhow::Result<String> {
        let docs = self.0.list_docs()?;
        Ok(serde_json::to_string(&docs)?)
    }
}

#[async_trait]
impl Tool for ReadDocTool {
    fn name(&self) -> &'static str {
        "read_doc"
    }

    fn definition(&self) -> ToolDef {
        crate::llm::messages::ToolDef {
            kind: "function",
            function: crate::llm::messages::ToolFunctionDef {
                name: self.name().to_string(),
                description: "Read one document from the knowledge folder by its name, as returned by list_docs.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": { "name": { "type": "string" } },
                    "required": ["name"]
                }),
            },
        }
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        #[derive(serde::Deserialize)]
        struct Args {
            name: String,
        }
        let args: Args = serde_json::from_str(args_json)
            .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
        self.0.read_doc(&args.name)
    }
}
