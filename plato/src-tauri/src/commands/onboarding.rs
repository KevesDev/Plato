use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use crate::models::{IpcResponse, DownloadProgressEvent};

const MODEL_DIR: &str = "models";

// Corrected URLs pointing to the exact sub-directory, using /resolve/ for raw binary download
const MODEL_FILES: &[(&str, &str)] = &[
    ("c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf"),
];

/**
 * Performs a rigorous integrity check on local model assets.
 * Validates file size against the remote source. 
 * Any file under 1MB is flagged as an HTML error redirect and immediately purged.
 */
#[tauri::command]
pub async fn check_model_status(app: AppHandle) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);

    if !base_path.exists() {
        return Ok(IpcResponse { success: true, data: Some(false), error_message: None });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    let mut ready = true;

    for (filename, url) in MODEL_FILES {
        let file_path = base_path.join(filename);
        
        if !file_path.exists() {
            ready = false;
            continue;
        }

        let local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
        let mut is_corrupted = false;

        // Model chunks are ~15GB. Anything under 1MB is guaranteed to be a network error page.
        if local_size < 1_000_000 {
            is_corrupted = true;
        } else if let Ok(res) = client.head(*url).send().await {
            if res.status().is_success() {
                if let Some(remote_size) = res.content_length() {
                    if local_size != remote_size {
                        is_corrupted = true;
                    }
                }
            }
        }

        if is_corrupted {
            let _ = fs::remove_file(&file_path);
            ready = false;
        }
    }

    Ok(IpcResponse { success: true, data: Some(ready), error_message: None })
}

/**
 * Initiates the multi-part model download sequence.
 * Enforces strict network status and payload size validation before writing to disk.
 */
#[tauri::command]
pub async fn start_model_download(app: AppHandle) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);

    if !base_path.exists() {
        fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
    }

    let client = reqwest::Client::new();
    let total_parts = MODEL_FILES.len() as u32;

    for (index, (filename, url)) in MODEL_FILES.iter().enumerate() {
        let part_current = (index + 1) as u32;
        let file_path = base_path.join(filename);

        let res = client.get(*url).send().await.map_err(|e| e.to_string())?;
        
        // Trap HTTP errors (403 Forbidden, 404 Not Found)
        if !res.status().is_success() {
            return Err(format!("Network Failure: HuggingFace returned HTTP {}", res.status()));
        }

        let total_bytes = res.content_length().unwrap_or(0);

        // Trap HTML redirects (Gated models require auth)
        if total_bytes < 1_000_000 {
            return Err("Model access denied. HuggingFace returned an HTML redirect. Ensure the model repository is public or you have accepted the license agreement.".to_string());
        }

        let local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
        if file_path.exists() && local_size == total_bytes {
            continue; 
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| e.to_string())?;

        let mut stream = res.bytes_stream();
        let mut downloaded_bytes: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            
            downloaded_bytes += chunk.len() as u64;

            let _ = app.emit("download_progress", DownloadProgressEvent {
                part_current,
                part_total: total_parts,
                downloaded_bytes,
                total_bytes,
            });
        }
    }

    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}