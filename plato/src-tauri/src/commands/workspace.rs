use std::path::PathBuf;
use std::fs;
use tokio::task;
use rfd::AsyncFileDialog;
use tauri::{AppHandle, State, Manager};
use crate::models::{IpcResponse, WorkspaceState};
use crate::engine::PlatoEngineState;
use crate::engine::vector_db::VectorDatabase;

#[tauri::command]
pub async fn select_and_scan_workspace() -> Result<IpcResponse<WorkspaceState>, String> {
    let folder = AsyncFileDialog::new()
        .set_title("Select Story Vault")
        .pick_folder()
        .await;

    let path = match folder {
        Some(f) => f.path().to_path_buf(),
        None => {
            return Ok(IpcResponse {
                success: false,
                data: None,
                error_message: Some("Directory selection canceled by user.".to_string()),
            });
        }
    };

    let path_str = path.to_string_lossy().to_string();

    let state = task::spawn_blocking(move || {
        let mut count = 0;
        let mut stack = vec![path];

        while let Some(current_path) = stack.pop() {
            if let Ok(entries) = fs::read_dir(current_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            stack.push(entry.path());
                        } else if file_type.is_file() {
                            if let Some(ext) = entry.path().extension() {
                                let ext_str = ext.to_string_lossy().to_lowercase();
                                if ext_str == "txt" || ext_str == "md" {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        WorkspaceState {
            active_directory_path: Some(path_str),
            indexed_file_count: count,
            is_indexing: false,
        }
    }).await.map_err(|e| format!("Directory traversal panic: {}", e))?;

    Ok(IpcResponse {
        success: true,
        data: Some(state),
        error_message: None,
    })
}

/**
 * Triggers the heavy ingestion pipeline. Reads all identified text files, chunks them into 
 * semantic paragraphs, and dispatches them in batches to the LanceDB embedding engine.
 */
#[tauri::command]
pub async fn start_workspace_ingestion(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    path: String
) -> Result<IpcResponse<usize>, String> {
    
    // 1. Initialize Vector DB if it is not already running
    let mut vector_db_lock = state.vector_db.lock().await;
    if vector_db_lock.is_none() {
        let mut db_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        db_path.push("lancedb");
        *vector_db_lock = Some(VectorDatabase::new(&db_path).await?);
    }
    let vector_db = vector_db_lock.as_ref().unwrap().clone();
    drop(vector_db_lock); // Release the lock immediately to prevent UI blocking

    let path_clone = path.clone();

    // 2. Deep read and logical chunking operation (Offloaded to worker thread)
    let chunks = task::spawn_blocking(move || {
        let mut results = Vec::new();
        let mut stack = vec![PathBuf::from(path_clone)];
        
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            stack.push(entry.path());
                        } else if file_type.is_file() {
                            if let Some(ext) = entry.path().extension() {
                                let ext_str = ext.to_string_lossy().to_lowercase();
                                if ext_str == "txt" || ext_str == "md" {
                                    if let Ok(content) = fs::read_to_string(entry.path()) {
                                        // Logical Chunking: Split documents by double-newlines (paragraphs)
                                        let paragraphs: Vec<&str> = content.split("\n\n").collect();
                                        for p in paragraphs {
                                            let trimmed = p.trim();
                                            // Ignore extremely short fragments that lack semantic value
                                            if trimmed.len() > 30 { 
                                                results.push((entry.path().to_string_lossy().to_string(), trimmed.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }).await.map_err(|e| format!("Chunking panic: {}", e))?;

    // 3. Batched Ingestion to prevent FastEmbed OOM limits
    let mut total_ingested = 0;
    for chunk_batch in chunks.chunks(100) {
        let count = vector_db.ingest_documents(chunk_batch.to_vec()).await?;
        total_ingested += count;
    }

    Ok(IpcResponse {
        success: true,
        data: Some(total_ingested),
        error_message: None
    })
}