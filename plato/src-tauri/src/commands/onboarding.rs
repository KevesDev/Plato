use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use reqwest::header::RANGE;
use crate::models::{IpcResponse, DownloadProgressEvent};

const MODEL_DIR: &str = "models";

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

        if local_size < 1_000_000 {
            let _ = fs::remove_file(&file_path);
            ready = false;
        } else if let Ok(res) = client.head(*url).send().await {
            if res.status().is_success() {
                if let Some(remote_size) = res.content_length() {
                    // Prevent deletion if S3 accidentally strips the length header (returns 0)
                    if local_size != remote_size && remote_size > 0 {
                        ready = false;
                    }
                }
            }
        }
    }

    Ok(IpcResponse { success: true, data: Some(ready), error_message: None })
}

/**
 * Initiates the multi-part model download sequence with HTTP Range request support.
 * Throttles IPC emissions to prevent React UI freezing.
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

        let local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        if local_size > 0 && local_size < 1_000_000 {
            let _ = fs::remove_file(&file_path);
        }

        let clean_local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        let mut request = client.get(*url);
        let mut is_resuming = false;

        if clean_local_size > 0 {
            request = request.header(RANGE, format!("bytes={}-", clean_local_size));
            is_resuming = true;
        }

        let res = request.send().await.map_err(|e| e.to_string())?;
        
        if !res.status().is_success() {
            return Err(format!("Network Failure: HuggingFace returned HTTP {}", res.status()));
        }

        // Resolves the true file size without relying on a pre-flight HEAD request
        // Range requests return remaining bytes; standard requests return total bytes.
        let response_length = res.content_length().unwrap_or(0);
        let true_total_bytes = if is_resuming {
            clean_local_size + response_length
        } else {
            response_length
        };

        // Trap HTML redirects
        if true_total_bytes < 1_000_000 {
            return Err("Model access denied. HuggingFace returned an HTML redirect. Ensure the model repository is public.".to_string());
        }

        if clean_local_size == true_total_bytes {
            continue; 
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(is_resuming)
            .write(true)
            .truncate(!is_resuming)
            .open(&file_path)
            .map_err(|e| e.to_string())?;

        let mut stream = res.bytes_stream();
        let mut downloaded_bytes: u64 = clean_local_size;
        
        // Throttling mechanism: 100ms
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            
            downloaded_bytes += chunk.len() as u64;

            // Only emit to the UI once every 100ms to prevent React context locking
            if last_emit.elapsed().as_millis() > 100 {
                let _ = app.emit("download_progress", DownloadProgressEvent {
                    part_current,
                    part_total: total_parts,
                    downloaded_bytes,
                    total_bytes: true_total_bytes,
                });
                last_emit = std::time::Instant::now();
            }
        }

        // Guarantee a final 100% emit when the chunk completes
        let _ = app.emit("download_progress", DownloadProgressEvent {
            part_current,
            part_total: total_parts,
            downloaded_bytes: true_total_bytes,
            total_bytes: true_total_bytes,
        });
    }

    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}