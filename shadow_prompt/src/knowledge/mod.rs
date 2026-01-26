pub mod search;

use anyhow::Result;
use crate::config::Config;

pub struct KnowledgeProvider;

impl KnowledgeProvider {
    pub async fn gather_context(query: &str, config: &Config) -> Result<String> {
        let mut context = String::new();

        // 1. Web Search
        if config.search.enabled {
            match search::perform_search(query, config.search.max_results).await {
                Ok(results) => {
                    if !results.is_empty() {
                        context.push_str("Based on web search results:\n");
                        context.push_str(&results);
                        context.push_str("\n\n");
                    }
                }
                Err(e) => {
                    eprintln!("Search failed: {}", e); 
                    // Fail gracefully, don't crash flow
                }
            }
        }

        // 2. Local RAG (Future)
        // if config.rag.enabled { ... }

        Ok(context)
    }
}
