pub mod search;
pub mod rag;

use anyhow::Result;
use crate::config::Config;
use crate::capabilities::ModelCapabilities;
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

        let model_has_search = ModelCapabilities::supports_search(config);

        // 1. Web Search - ONLY if model doesn't have built-in search
        if config.search.enabled && !model_has_search {
            match search::perform_search(query, &config.search).await {
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
        } else if model_has_search {
            info!("[*] Model has built-in search capability, skipping external search");
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
