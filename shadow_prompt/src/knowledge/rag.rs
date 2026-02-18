use anyhow::{Result, Context};
use std::fs;

use glob::glob;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use serde::{Deserialize, Serialize};
use crate::config::{Config, get_exe_dir};

use std::collections::HashMap;

// Simple Document struct for In-Memory/JSON Storage
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Document {
    id: String,
    path: String,
    content: String,
    embedding: Vec<f32>,
    last_modified: u64,
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


    fn get_files_to_embed(
        &self, 
        root_path: &std::path::Path, 
        index_file_path: &std::path::Path
    ) -> Result<(Vec<Document>, Vec<(String, String, u64)>)> {
        // Load existing index if available
        let mut existing_docs: HashMap<String, Document> = HashMap::new();
        if index_file_path.exists() {
             if let Ok(content) = fs::read_to_string(&index_file_path) {
                 if let Ok(existing_index) = serde_json::from_str::<RagIndex>(&content) {
                     for doc in existing_index.documents {
                         existing_docs.insert(doc.path.clone(), doc);
                     }
                     println!("[RAG] Loaded {} existing documents from index.", existing_docs.len());
                 }
             }
        }

        let root_path_str = root_path.display().to_string();
        
        let mut docs_to_embed = Vec::new();
        let mut final_docs = Vec::new();

        let patterns = vec![
            format!("{}/*.md", root_path_str),
            format!("{}/*.txt", root_path_str),
            format!("{}/**/*.md", root_path_str),
            format!("{}/**/*.txt", root_path_str),
        ];

        let mut found_paths = std::collections::HashSet::new();

        for pattern in patterns {
            for entry in glob(&pattern).context("Failed to read glob pattern")? {
                match entry {
                    Ok(path) => {
                        let path_str = path.display().to_string();
                        if found_paths.contains(&path_str) {
                            continue;
                        }
                        found_paths.insert(path_str.clone());

                        let metadata = fs::metadata(&path)?;
                        let modified = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

                        let mut reuse = false;
                        if let Some(existing_doc) = existing_docs.get(&path_str) {
                            if existing_doc.last_modified == modified {
                                final_docs.push(existing_doc.clone());
                                reuse = true;
                            }
                        }

                        if !reuse {
                             let content = fs::read_to_string(&path).unwrap_or_default();
                             if !content.trim().is_empty() {
                                 docs_to_embed.push((path_str, content, modified));
                             }
                        }
                    },
                    Err(e) => eprintln!("[RAG] Error reading file: {:?}", e),
                }
            }
        }
        
        Ok((final_docs, docs_to_embed))
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

        println!("[RAG] Scanning knowledge folder: {}", root_path.display());
        
        let (mut final_docs, docs_to_embed) = self.get_files_to_embed(&root_path, &index_file_path)?;

        if docs_to_embed.is_empty() && final_docs.is_empty() {
             return Ok(0);
        }

        if !docs_to_embed.is_empty() {
            println!("[RAG] Found {} new/modified documents. Generating embeddings...", docs_to_embed.len());
            let texts: Vec<String> = docs_to_embed.iter().map(|(_, c, _)| c.clone()).collect();
            let embeddings = embedding_model.embed(texts, None)?;

            for (i, embedding) in embeddings.into_iter().enumerate() {
                let (path, content, modified) = &docs_to_embed[i];
                final_docs.push(Document {
                    id: uuid::Uuid::new_v4().to_string(), // Generate unique ID
                    path: path.clone(),
                    content: content.clone(),
                    embedding,
                    last_modified: *modified,
                });
            }
        } else {
            println!("[RAG] No new documents to embed. Using cached index.");
        }

        let index = RagIndex {
            documents: final_docs,
        };

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
            last_modified: 0,
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
    async fn test_rag_incremental_logic() -> Result<()> {
        use std::io::Write;
        
        let temp_dir = std::env::temp_dir().join(format!("shadow_prompt_test_{}", uuid::Uuid::new_v4()));
        let knowledge_dir = temp_dir.join("knowledge");
        let data_dir = temp_dir.join("data");
        let index_path = data_dir.join("index.json");

        fs::create_dir_all(&knowledge_dir)?;
        fs::create_dir_all(&data_dir)?;

        // Create doc1.txt
        let doc1_path = knowledge_dir.join("doc1.txt");
        {
            let mut f = fs::File::create(&doc1_path)?;
            f.write_all(b"content 1")?;
        }
        
        // Sleep to ensure modification time is distinct if we needed to wait (but we set it manually in index for first test)
        // actually we read it from FS.
        
        let metadata = fs::metadata(&doc1_path)?;
        let modified = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

        // Create initial index with doc1 having correct timestamp
        let index = RagIndex {
            documents: vec![
                Document {
                    id: "1".to_string(),
                    path: doc1_path.display().to_string(),
                    content: "content 1".to_string(),
                    embedding: vec![],
                    last_modified: modified,
                },
                Document {
                    id: "2".to_string(),
                    path: knowledge_dir.join("doc2_missing.txt").display().to_string(),
                    content: "content 2".to_string(),
                    embedding: vec![],
                    last_modified: 12345,
                }
            ]
        };
        
        let json = serde_json::to_string(&index)?;
        fs::write(&index_path, json)?;

        // Setup RagSystem
        let mut config = Config::default();
        config.rag.enabled = true;
        let rag = RagSystem {
            embedding_model: None,
            config: config.clone(),
            cached_index: tokio::sync::RwLock::new(None),
            is_operational: true,
            init_error: None,
        };

        // TEST 1: No changes
        let (reused, to_embed) = rag.get_files_to_embed(&knowledge_dir, &index_path)?;
        
        // doc1 should be reused
        assert_eq!(reused.len(), 1, "Should reuse 1 document");
        assert_eq!(reused[0].path, doc1_path.display().to_string());
        // doc2 is missing from disk, so it is dropped (neither reused nor embedded)
        // to_embed should be empty
        assert_eq!(to_embed.len(), 0, "Should have 0 docs to embed");


        // TEST 2: Modify doc1
        // Wait a bit to ensure FS timestamp granularity (some systems are 1s, others 100ns)
        // But simply sleeping 1.1s is safe for most.
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        
        {
            let mut f = fs::File::create(&doc1_path)?;
            f.write_all(b"content 1 modified")?;
        }

        let (reused_2, to_embed_2) = rag.get_files_to_embed(&knowledge_dir, &index_path)?;
        
        // doc1 is modified, so it should NOT be reused
        assert_eq!(reused_2.len(), 0, "Should reuse 0 documents (doc1 changed)");
        // doc1 should be in to_embed
        assert_eq!(to_embed_2.len(), 1, "Should have 1 doc to embed");
        assert_eq!(to_embed_2[0].0, doc1_path.display().to_string());
        assert_eq!(to_embed_2[0].1, "content 1 modified");


        // TEST 3: Add new file
        let doc3_path = knowledge_dir.join("doc3.txt");
        {
             let mut f = fs::File::create(&doc3_path)?;
             f.write_all(b"content 3")?;
        }
        
        let (reused_3, to_embed_3) = rag.get_files_to_embed(&knowledge_dir, &index_path)?;
        // doc1 is still modified compared to disk index (we haven't updated index on disk in this test sequence)
        // so doc1 is re-embedded. doc3 is new, so embedded.
        // Wait, logic:
        // index on disk has doc1 with OLD timestamp.
        // file on disk has NEW timestamp.
        // So doc1 -> to_embed.
        // doc3 -> to_embed.
        
        assert_eq!(reused_3.len(), 0);
        assert_eq!(to_embed_3.len(), 2);
        
        // Clean up
        let _ = fs::remove_dir_all(temp_dir);

        Ok(())
    }

    #[tokio::test]
    async fn test_rag_operational_flag() {
        // Create a dummy config
        let mut config = Config::default();
        config.rag.enabled = true;

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
