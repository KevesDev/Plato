use std::fs;
use std::path::Path;
use crate::models::{IpcResponse, WorkspaceState};

/**
 * Scans a target directory for valid text-based files (.md, .txt) to prepare
 * for LanceDB ingestion. Returns a strict WorkspaceState to the frontend.
 */
#[tauri::command]
pub async fn scan_workspace_directory(path: String) -> IpcResponse<WorkspaceState> {
    let dir_path = Path::new(&path);
    
    if !dir_path.exists() || !dir_path.is_dir() {
        return IpcResponse {
            success: false,
            data: None,
            error_message: Some("Invalid directory path provided.".to_string()),
        };
    }

    let mut valid_file_count: u32 = 0;

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" || extension == "txt" {
                        valid_file_count += 1;
                    }
                }
            }
        }
    }

    let state = WorkspaceState {
        active_directory_path: Some(path),
        indexed_file_count: valid_file_count,
        is_indexing: false,
    };

    IpcResponse {
        success: true,
        data: Some(state),
        error_message: None,
    }
}