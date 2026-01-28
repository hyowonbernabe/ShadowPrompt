use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
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
    embedding_model: TextEmbedding,
    config: Config,
}

impl RagSystem {
    pub async fn new(config: &Config) -> Result<Self> {
        // Initialize Embedding Model
        // We use BGE-Small-EN-V1.5 which is small and fast.
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::BGESmallENV15;
        options.show_download_progress = true;
        options.cache_dir = get_exe_dir().join("data").join("models");

        let model = TextEmbedding::try_new(options).context("Failed to initialize FastEmbed")?;

        Ok(Self {
            embedding_model: model,
            config: config.clone(),
        })
    }

    pub async fn ingest(&self) -> Result<usize> {
        if !self.config.rag.enabled {
             return Ok(0);
        }

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
        let embeddings = self.embedding_model.embed(texts, None)?;

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

        println!("[RAG] Saved index to {:?}", index_file_path);

        Ok(index.documents.len())
    }

    pub async fn query(&self, text: &str) -> Result<Vec<String>> {
         if !self.config.rag.enabled {
             return Ok(vec![]);
        }

        let exe_dir = get_exe_dir();
        let index_base = exe_dir.join(&self.config.rag.index_path);
        let index_file_path = if self.config.rag.index_path.ends_with(".json") {
            index_base
        } else {
            index_base.join("index.json")
        };

        if !index_file_path.exists() {
            // Try to ingest if missing? Or just return empty.
            // Let's just return empty to be safe/fast.
            return Ok(vec![]);
        }

        // Load Index
        let content = fs::read_to_string(&index_file_path).context("Failed to read index file")?;
        let index: RagIndex = serde_json::from_str(&content).context("Failed to parse index file")?;

        if index.documents.is_empty() {
            return Ok(vec![]);
        }

        // Embed Query
        let query_embeddings = self.embedding_model.embed(vec![text.to_string()], None)?;
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
