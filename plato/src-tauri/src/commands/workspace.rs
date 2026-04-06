use std::path::PathBuf;
use std::fs;
use tokio::task;
use rfd::AsyncFileDialog;
use tauri::{AppHandle, State, Manager};
use crate::models::{IpcResponse, WorkspaceState, PlatoConfig};
use crate::engine::PlatoEngineState;
use crate::engine::vector_db::VectorDatabase;

// Helper to reliably map the configuration file
fn get_config_path(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("plato_config.json");
    path
}

// Internal helper to scan a directory and count text files
fn scan_directory(path_str: &str) -> u32 {
    let mut count = 0;
    let mut stack = vec![PathBuf::from(path_str)];
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
    count
}

/**
 * Fires on boot. Verifies the user's previously saved workspace and re-mounts it.
 */
#[tauri::command]
pub async fn load_persisted_workspace(app: AppHandle) -> Result<IpcResponse<WorkspaceState>, String> {
    let config_path = get_config_path(&app);
    if config_path.exists() {
        if let Ok(config_data) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<PlatoConfig>(&config_data) {
                if let Some(path_str) = config.active_workspace {
                    if PathBuf::from(&path_str).exists() {
                        let path_clone = path_str.clone();
                        let count = task::spawn_blocking(move || scan_directory(&path_clone)).await.unwrap_or(0);
                        return Ok(IpcResponse {
                            success: true,
                            data: Some(WorkspaceState {
                                active_directory_path: Some(path_str),
                                indexed_file_count: count,
                                is_indexing: false,
                            }),
                            error_message: None,
                        });
                    }
                }
            }
        }
    }
    Ok(IpcResponse {
        success: true,
        data: Some(WorkspaceState {
            active_directory_path: None,
            indexed_file_count: 0,
            is_indexing: false,
        }),
        error_message: None,
    })
}

#[tauri::command]
pub async fn select_and_scan_workspace(app: AppHandle) -> Result<IpcResponse<WorkspaceState>, String> {
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
    
    // Save to global config
    let config_path = get_config_path(&app);
    let config = PlatoConfig { active_workspace: Some(path_str.clone()) };
    if let Ok(config_str) = serde_json::to_string(&config) {
        let _ = fs::write(config_path, config_str);
    }

    let state = task::spawn_blocking(move || {
        let count = scan_directory(&path_str);
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

#[tauri::command]
pub async fn start_workspace_ingestion(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    path: String
) -> Result<IpcResponse<usize>, String> {
    
    let mut vector_db_lock = state.vector_db.lock().await;
    if vector_db_lock.is_none() {
        let mut db_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        db_path.push("lancedb");
        *vector_db_lock = Some(VectorDatabase::new(&db_path).await?);
    }
    let vector_db = vector_db_lock.as_ref().unwrap().clone();
    drop(vector_db_lock); 

    let path_clone = path.clone();

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
                                        let paragraphs: Vec<&str> = content.split("\n\n").collect();
                                        for p in paragraphs {
                                            let trimmed = p.trim();
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

#[tauri::command]
pub async fn save_lore_entity(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    name: String,
    category: String,
    description: String
) -> Result<IpcResponse<bool>, String> {
    
    let mut vector_db_lock = state.vector_db.lock().await;
    if vector_db_lock.is_none() {
        let mut db_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        db_path.push("lancedb"); 
        *vector_db_lock = Some(VectorDatabase::new(&db_path).await?);
    }
    let vector_db = vector_db_lock.as_ref().unwrap().clone();
    drop(vector_db_lock);

    let virtual_path = format!("lorebook://{}/{}", category.to_lowercase(), name.to_lowercase().replace(" ", "_"));
    let semantic_content = format!("[Entity Type: {}] {}: {}", category, name, description);

    let count = vector_db.ingest_documents(vec![(virtual_path, semantic_content)]).await?;

    Ok(IpcResponse {
        success: count > 0,
        data: Some(count > 0),
        error_message: None
    })
}