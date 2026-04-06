use std::sync::Arc;
use std::path::PathBuf;
use tokio::task;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
// Removed the deprecated TextEmbeddingUserDefined import
use fastembed::{TextEmbedding, InitOptions};

/**
 * Native, lightweight vector memory matrix.
 * Strictly local, file-based persistence without compiler panics.
 */
#[derive(Serialize, Deserialize, Clone)]
pub struct MemoryRecord {
    pub file_path: String,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Clone)]
pub struct VectorDatabase {
    pub db_path: PathBuf,
    pub records: Arc<RwLock<Vec<MemoryRecord>>>,
}

impl VectorDatabase {
    pub async fn new(db_dir: &PathBuf) -> Result<Self, String> {
        // Ensure the data directory exists
        if !db_dir.exists() {
            std::fs::create_dir_all(db_dir).map_err(|e| format!("Failed to create DB directory: {}", e))?;
        }
        
        let file_path = db_dir.join("matrix.json");
        let mut records = Vec::new();

        // Load existing memories if they exist
        if file_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&file_path) {
                if let Ok(parsed) = serde_json::from_str(&data) {
                    records = parsed;
                }
            }
        }

        Ok(Self { 
            db_path: file_path,
            records: Arc::new(RwLock::new(records)),
        })
    }

    pub async fn ingest_documents(&self, documents: Vec<(String, String)>) -> Result<usize, String> {
        if documents.is_empty() { return Ok(0); }
        let doc_count = documents.len();
        
        let embeddings_result = task::spawn_blocking(move || {
            let options = InitOptions::new(fastembed::EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(false);
                
            // FIX: Declared as `mut` to allow FastEmbed to internally cache tensors
            let mut model = TextEmbedding::try_new(options)
                .map_err(|e| format!("Embedding Model Initialization Failed: {}", e))?;

            let texts: Vec<&str> = documents.iter().map(|(_, text)| text.as_str()).collect();
            let embeddings = model.embed(texts, None).map_err(|e| format!("Embedding Generation Failed: {}", e))?;
            
            Ok::<_, String>((documents, embeddings))
        }).await.map_err(|e| format!("Embedding Thread Panic: {}", e))??;

        let (docs, embeddings) = embeddings_result;
        
        let mut new_records = Vec::new();
        for (i, (path, text)) in docs.into_iter().enumerate() {
            new_records.push(MemoryRecord {
                file_path: path,
                text,
                vector: embeddings[i].clone(),
            });
        }

        // Thread-safe state mutation
        let mut records_lock = self.records.write().await;
        records_lock.extend(new_records);
        
        // Persist the matrix directly to disk
        let data = serde_json::to_string(&*records_lock).map_err(|e| format!("Serialization Failed: {}", e))?;
        std::fs::write(&self.db_path, data).map_err(|e| format!("Disk Write Failed: {}", e))?;

        Ok(doc_count)
    }

    /**
     * Executes a mathematical Cosine Similarity search against the entire memory matrix.
     * Returns the 'top_k' most semantically relevant text chunks.
     */
    pub async fn search_matrix(&self, query: &str, top_k: usize) -> Result<Vec<String>, String> {
        let records = self.records.read().await.clone();
        if records.is_empty() { return Ok(Vec::new()); }

        let query_clone = query.to_string();
        
        let search_result = task::spawn_blocking(move || {
            let options = InitOptions::new(fastembed::EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(false);
                
            let mut model = TextEmbedding::try_new(options)
                .map_err(|e| format!("Embedding Model Initialization Failed: {}", e))?;
                
            let embeddings = model.embed(vec![&query_clone], None)
                .map_err(|e| format!("Embedding Generation Failed: {}", e))?;
                
            let query_vector = &embeddings[0];
            
            // Execute Cosine Similarity equation across the tensor array
            let mut scored_records: Vec<(f32, String)> = records.iter().map(|record| {
                let mut dot_product = 0.0;
                let mut norm_a = 0.0;
                let mut norm_b = 0.0;
                for (a, b) in query_vector.iter().zip(record.vector.iter()) {
                    dot_product += a * b;
                    norm_a += a * a;
                    norm_b += b * b;
                }
                let similarity = if norm_a > 0.0 && norm_b > 0.0 {
                    dot_product / (norm_a.sqrt() * norm_b.sqrt())
                } else {
                    0.0
                };
                (similarity, record.text.clone())
            }).collect();
            
            // Sort by descending similarity score
            scored_records.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            
            let top_results: Vec<String> = scored_records.into_iter()
                .take(top_k)
                .map(|(_, text)| text)
                .collect();
                
            Ok::<_, String>(top_results)
        }).await.map_err(|e| format!("Search Thread Panic: {}", e))??;

        Ok(search_result)
    }
}