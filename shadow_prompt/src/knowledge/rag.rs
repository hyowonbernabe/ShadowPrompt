use anyhow::{Result, Context};
use std::fs;

use glob::glob;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use serde::{Deserialize, Serialize};
use crate::config::{Config, get_exe_dir};

// Simple Document struct for In-Memory/JSON Storage
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Document {
    id: String,
    path: String,
    content: String,
    embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct RagIndex {
    documents: Vec<Document>,
}

pub struct RagSystem {
    embedding_model: Option<TextEmbedding>,
    config: Config,
    cached_index: tokio::sync::RwLock<Option<RagIndex>>,
    is_operational: bool,
    init_error: Option<String>,
}

impl RagSystem {
    pub async fn new(config: &Config) -> Self {
        // Initialize Embedding Model
        // We use BGE-Small-EN-V1.5 which is small and fast.
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::BGESmallENV15;
        options.show_download_progress = true;
        options.cache_dir = get_exe_dir().join("data").join("models");

        let (model, is_operational, init_error) = match TextEmbedding::try_new(options) {
            Ok(m) => (Some(m), true, None),
            Err(e) => {
                let err_msg = e.to_string();
                eprintln!("[!] Failed to initialize FastEmbed: {}", err_msg);
                (None, false, Some(err_msg))
            }
        };

        Self {
            embedding_model: model,
            config: config.clone(),
            cached_index: tokio::sync::RwLock::new(None),
            is_operational,
            init_error,
        }
    }

    pub fn is_operational(&self) -> bool {
        self.is_operational
    }

    pub fn get_init_error(&self) -> Option<&str> {
        self.init_error.as_deref()
    }

    pub async fn ingest(&self) -> Result<usize> {
        if !self.config.rag.enabled || !self.is_operational {
             return Ok(0);
        }

        let embedding_model = match &self.embedding_model {
            Some(m) => m,
            None => return Ok(0),
        };

        let exe_dir = get_exe_dir();
        let root_path = exe_dir.join(&self.config.rag.knowledge_path);
        let index_base = exe_dir.join(&self.config.rag.index_path);
        
        if !root_path.exists() {
            fs::create_dir_all(&root_path).context("Failed to create knowledge directory")?;
            return Ok(0);
        }

        let index_file_path = if self.config.rag.index_path.ends_with(".json") {
            index_base.clone()
        } else {
            index_base.join("index.json")
        };

        if let Some(parent) = index_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let root_path_str = root_path.display().to_string();
        println!("[RAG] Scanning knowledge folder: {}", root_path_str);

        let mut new_docs = Vec::new();
        let patterns = vec![
            format!("{}/*.md", root_path_str),
            format!("{}/*.txt", root_path_str),
            format!("{}/**/*.md", root_path_str),
            format!("{}/**/*.txt", root_path_str),
        ];

        for pattern in patterns {
            for entry in glob(&pattern).context("Failed to read glob pattern")? {
                match entry {
                    Ok(path) => {
                        let content = fs::read_to_string(&path).unwrap_or_default();
                        if !content.trim().is_empty() {
                            new_docs.push((path.display().to_string(), content));
                        }
                    },
                    Err(e) => eprintln!("[RAG] Error reading file: {:?}", e),
                }
            }
        }

        // Deduplicate paths
        new_docs.sort_by(|a, b| a.0.cmp(&b.0));
        new_docs.dedup_by(|a, b| a.0 == b.0);

        if new_docs.is_empty() {
            return Ok(0);
        }

        println!("[RAG] Found {} unique documents. Generating embeddings...", new_docs.len());

        let texts: Vec<String> = new_docs.iter().map(|(_, c)| c.clone()).collect();
        let embeddings = embedding_model.embed(texts, None)?;

        let mut index = RagIndex::default();

        for (i, embedding) in embeddings.into_iter().enumerate() {
            let (path, content) = &new_docs[i];
            index.documents.push(Document {
                id: uuid::Uuid::new_v4().to_string(), // Generate unique ID
                path: path.clone(),
                content: content.clone(),
                embedding,
            });
        }

        // Save to Disk (JSON)
        let json = serde_json::to_string_pretty(&index)?;
        fs::write(&index_file_path, json)?;

        let count = index.documents.len();

        // Update Cache
        {
            let mut cache = self.cached_index.write().await;
            *cache = Some(index);
        }

        println!("[RAG] Saved index to {:?}", index_file_path);

        Ok(count)
    }

    pub async fn query(&self, text: &str) -> Result<Vec<String>> {
         if !self.config.rag.enabled {
             return Ok(vec![]);
        }

        if !self.is_operational {
            eprintln!("[!] RAG is not operational (initialization failed). Returning empty results.");
            return Ok(vec![]);
        }

        let embedding_model = match &self.embedding_model {
            Some(m) => m,
            None => return Ok(vec![]),
        };

        // Check if cache is empty
        // We use a block to drop the read lock before potentially acquiring a write lock or doing IO
        let needs_load = {
            let cache = self.cached_index.read().await;
            cache.is_none()
        };

        if needs_load {
            let exe_dir = get_exe_dir();
            let index_base = exe_dir.join(&self.config.rag.index_path);
            let index_file_path = if self.config.rag.index_path.ends_with(".json") {
                index_base
            } else {
                index_base.join("index.json")
            };

            if index_file_path.exists() {
                // Load Index
                // Note: We might be doing double work if multiple threads race here, 
                // but for this use case it's acceptable simplicity vs complexity of double-checked locking with async.
                if let Ok(content) = fs::read_to_string(&index_file_path) {
                     if let Ok(index) = serde_json::from_str::<RagIndex>(&content) {
                        let mut cache = self.cached_index.write().await;
                        *cache = Some(index);
                     } else {
                         eprintln!("[RAG] Failed to parse index file: {:?}", index_file_path);
                     }
                } else {
                    eprintln!("[RAG] Failed to read index file: {:?}", index_file_path);
                }
            }
        }

        // Now query from cache
        let cache = self.cached_index.read().await;
        let index = match &*cache {
            Some(i) => i,
            None => return Ok(vec![]), // Still no index, return empty
        };

        if index.documents.is_empty() {
            return Ok(vec![]);
        }

        // Embed Query
        let query_embeddings = embedding_model.embed(vec![text.to_string()], None)?;
        let query_vec = &query_embeddings[0];

        // Calculate Cosine Similarity
        let mut scores: Vec<(f32, &Document)> = index.documents.iter().map(|doc| {
            let score = cosine_similarity(query_vec, &doc.embedding);
            (score, doc)
        }).collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Filter and Collect
        let results: Vec<String> = scores.into_iter()
            .filter(|(score, _)| *score >= self.config.rag.min_score)
            .take(self.config.rag.max_results)
            .map(|(_, doc)| doc.content.clone())
            .collect();

        Ok(results)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rag_caching_logic() {
        // This test verifies that we can manipulate the cache directly
        // and that query handles it (even if it fails later due to missing model).
        
        let mut config = Config::default();
        config.rag.enabled = true;

        let rag = RagSystem {
            embedding_model: None, // No model, so query will return empty
            config: config.clone(),
            cached_index: tokio::sync::RwLock::new(None),
            is_operational: true, // Pretend it is operational
            init_error: None,
        };

        // 1. Verify cache is initially empty
        {
            let cache = rag.cached_index.read().await;
            assert!(cache.is_none());
        }

        // 2. Manually populate cache
        let mut index = RagIndex::default();
        index.documents.push(Document {
            id: "test".to_string(),
            path: "test.txt".to_string(),
            content: "hello world".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
        });

        {
            let mut cache = rag.cached_index.write().await;
            *cache = Some(index);
        }

        // 3. Verify cache is populated
        {
            let cache = rag.cached_index.read().await;
            assert!(cache.is_some());
            let idx = cache.as_ref().unwrap();
            assert_eq!(idx.documents.len(), 1);
        }
        
        // Note: We can't fully test query execution without a model, 
        // but we verified the structure and access patterns compile and work.
    }

    #[tokio::test]
    async fn test_rag_operational_flag() {
        // Create a dummy config
        let mut config = Config::default();
        config.rag.enabled = true;

        // Since fields are private, we need a way to construct it.
        // We can add a helper in the parent module that is only for tests.
        // Or we can just use the public constructor?
        // But we want to simulate failure.
        
        // Let's rely on the fact that if this compiles, it works.
        // If it compiles, it means my understanding of visibility was wrong or incomplete.
        // (Actually, checking docs: "Private items are visible to the current module and its descendants.")
        // Yes! "Private items are visible to the current module AND ITS DESCENDANTS."
        // Since `tests` is a descendant of `rag`, it can see private items of `rag`!
        // Struct fields are private to the module defining the struct.
        // Since `RagSystem` is defined in `rag`, its private fields are visible to `rag` and `rag`'s descendants (like `tests`).
        // So it is correct!
        
        let rag = RagSystem {
            embedding_model: None,
            config: config.clone(),
            cached_index: tokio::sync::RwLock::new(None),
            is_operational: false,
            init_error: Some("Simulated failure".to_string()),
        };

        // Test Query
        let result = rag.query("test").await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());

        // Test Ingest
        let ingest_result = rag.ingest().await;
        assert!(ingest_result.is_ok());
        assert_eq!(ingest_result.unwrap(), 0);
    }
}
