pub mod search;
pub mod rag;

use anyhow::Result;
use crate::config::Config;
use std::sync::Arc;
use regex::Regex;

pub struct ContextBundle {
    pub web: String,
    pub local: String,
    pub warnings: Vec<String>,
}

pub struct KnowledgeProvider {
    rag: Option<Arc<rag::RagSystem>>,
}

impl KnowledgeProvider {
    pub async fn new(config: &Config) -> Result<Self> {
        let rag = if config.rag.enabled {
            println!("[*] Initializing Local RAG System...");
            let sys = rag::RagSystem::new(config).await;
            Some(Arc::new(sys))
        } else {
            None
        };

        let provider = Self { rag };

        // Initial Ingestion (Non-blocking if possible, but for MVP we might await or spawn)
        if let Some(rag_sys) = &provider.rag {
             let rag_clone = rag_sys.clone();
             tokio::spawn(async move {
                 if let Err(e) = rag_clone.ingest().await {
                     eprintln!("[!] RAG Ingestion Failed: {}", e);
                 }
             });
        }

        Ok(provider)
    }

    pub async fn gather_context(&self, query: &str, config: &Config) -> Result<ContextBundle> {
        let mut bundle = ContextBundle {
            web: String::new(),
            local: String::new(),
            warnings: Vec::new(),
        };

        // 1. Web Search — always runs, strips labeled MCQ options from query only
        let search_query = clean_search_query(query);
        match search::perform_search(&search_query, &config.search).await {
            Ok(results) => {
                if !results.is_empty() {
                    bundle.web = results;
                }
            }
            Err(e) => {
                let msg = format!("Search failed: {}", e);
                eprintln!("{}", msg);
                bundle.warnings.push(msg);
            }
        }

        // 2. Local RAG — runs if system is initialized
        if let Some(rag) = &self.rag {
            match rag.query(query).await {
                Ok(results) => {
                    if !results.is_empty() {
                        let mut local = String::new();
                        for (i, doc) in results.iter().enumerate() {
                            local.push_str(&format!("[Document {}]: {}\n", i + 1, doc));
                        }
                        bundle.local = local;
                    }
                },
                Err(e) => {
                    let msg = format!("RAG Query Failed: {}", e);
                    eprintln!("[!] {}", msg);
                    bundle.warnings.push(msg);
                }
            }
        }

        Ok(bundle)
    }
}

/// Strip labeled MCQ options from the search query to improve result relevance.
/// Only strips when at least 2 labeled options are found (A/B/C/D or 1/2/3/4).
/// The full original text (including options) is still sent to the LLM.
fn clean_search_query(text: &str) -> String {
    let re = Regex::new(r"(?m)^\s*[A-Da-d1-4][.)]\s+.+$").unwrap();
    let matches: Vec<_> = re.find_iter(text).collect();
    if matches.len() >= 2 {
        re.replace_all(text, "").trim().to_string()
    } else {
        text.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_search_query_strips_labeled_mcq() {
        let input = "Which gas makes up most of Earth's atmosphere?\nA) Oxygen\nB) Nitrogen\nC) Carbon Dioxide\nD) Hydrogen";
        let result = clean_search_query(input);
        assert!(!result.contains("A) Oxygen"), "Should strip labeled options");
        assert!(result.contains("Which gas makes up most of Earth's atmosphere?"), "Should keep question");
    }

    #[test]
    fn test_clean_search_query_leaves_unlabeled() {
        let input = "Which gas makes up most of Earth's atmosphere?\nOxygen\nNitrogen\nCarbon Dioxide\nHydrogen";
        let result = clean_search_query(input);
        assert_eq!(result, input.trim(), "Should leave unlabeled options untouched");
    }

    #[test]
    fn test_clean_search_query_requires_two_matches() {
        // Only one labeled option — should not strip
        let input = "What is the capital?\nA) Paris";
        let result = clean_search_query(input);
        assert_eq!(result, input.trim());
    }

    #[test]
    fn test_clean_search_query_numeric_labels() {
        let input = "Pick the correct one:\n1) Alpha\n2) Beta\n3) Gamma";
        let result = clean_search_query(input);
        assert!(!result.contains("1) Alpha"), "Should strip numeric labeled options");
        assert!(result.contains("Pick the correct one:"), "Should keep question");
    }
}
