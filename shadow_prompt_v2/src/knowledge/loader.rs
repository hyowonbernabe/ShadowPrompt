// Walk knowledge/_default + knowledge/<subject>/ for *.md files,
// concatenate in deterministic order, return as one string.

use std::fs;
use std::path::Path;

use crate::config::schema::KnowledgeConfig;

#[derive(Debug, Clone, Default)]
pub struct KnowledgeBundle {
    pub text: String,
    pub source_count: usize,
    pub char_count: usize,
    pub approx_tokens: usize,
}

impl KnowledgeBundle {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

pub fn load(install_dir: &Path, cfg: &KnowledgeConfig) -> KnowledgeBundle {
    if !cfg.enabled {
        return KnowledgeBundle::default();
    }
    let root = install_dir.join("knowledge");
    if !root.is_dir() {
        return KnowledgeBundle::default();
    }

    let mut subjects: Vec<String> = vec!["_default".to_string()];
    subjects.extend(cfg.active_subjects.iter().cloned());

    let mut out = String::new();
    let mut count = 0usize;

    for subject in &subjects {
        let dir = root.join(subject);
        if !dir.is_dir() {
            continue;
        }
        let mut files: Vec<_> = match fs::read_dir(&dir) {
            Ok(rd) => rd.flatten().collect(),
            Err(e) => {
                log::warn!("knowledge: failed to read {}: {e}", dir.display());
                continue;
            }
        };
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let path = entry.path();
            let ext_ok = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(), "md" | "markdown" | "txt"))
                .unwrap_or(false);
            if !ext_ok || !path.is_file() {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let header = format!("\n\n# {}/{}\n\n", subject, path.file_name().unwrap().to_string_lossy());
                    out.push_str(&header);
                    out.push_str(content.trim_end());
                    count += 1;

                    if out.len() > cfg.max_chars {
                        log::warn!(
                            "knowledge: max_chars cap {} reached after {} file(s); truncating",
                            cfg.max_chars, count
                        );
                        out.truncate(cfg.max_chars);
                        break;
                    }
                }
                Err(e) => log::warn!("knowledge: failed to read {}: {e}", path.display()),
            }
        }
        if out.len() > cfg.max_chars {
            break;
        }
    }

    let trimmed = out.trim().to_string();
    let char_count = trimmed.len();
    let approx_tokens = char_count / 4;

    log::info!(
        "knowledge: loaded {} file(s), {} chars (~{} tokens)",
        count, char_count, approx_tokens
    );
    if approx_tokens > 0 && approx_tokens < 1024 {
        log::warn!(
            "knowledge: ~{approx_tokens} tokens is below Anthropic's 1024-token cache minimum; \
             caching will be disabled for this block (still injected, just paid full price)"
        );
    }

    KnowledgeBundle {
        text: trimmed,
        source_count: count,
        char_count,
        approx_tokens,
    }
}
