pub mod search;
pub mod rag;

use anyhow::Result;
use crate::config::Config;
use std::sync::Arc;

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

    pub async fn gather_context(&self, query: &str, config: &Config) -> Result<(String, Vec<String>)> {
        let mut context = String::new();
        let mut warnings = Vec::new();

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
                    let msg = format!("Search failed: {}", e);
                    eprintln!("{}", msg); 
                    warnings.push(msg);
                }
            }
        }

        // 2. Local RAG
        if let Some(rag) = &self.rag {
            match rag.query(query).await {
                Ok(results) => {
                    if !results.is_empty() {
                        context.push_str("Based on your knowledge base:\n");
                        for (i, doc) in results.iter().enumerate() {
                            context.push_str(&format!("[Document {}]: {}\n", i + 1, doc));
                        }
                        context.push_str("\n\n");
                    }
                },
                Err(e) => {
                    let msg = format!("RAG Query Failed: {}", e);
                    eprintln!("[!] {}", msg);
                    warnings.push(msg);
                }
            }
        }

        Ok((context, warnings))
    }
}
