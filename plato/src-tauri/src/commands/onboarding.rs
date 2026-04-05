use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use reqwest::header::RANGE;
use crate::models::{IpcResponse, DownloadProgressEvent, ACTIVE_MODEL, ModelTarget};

const MODEL_DIR: &str = "models";

const DEV_MODELS: &[(&str, &str)] = &[
    ("aya-23-8b-q4_k_m.gguf", "https://huggingface.co/AIronMind/aya-23-8B-Q4_K_M-GGUF/resolve/main/aya-23-8b-q4_k_m.gguf")
];

const PROD_MODELS: &[(&str, &str)] = &[
    ("c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M.gguf/c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf"),
];

fn get_active_models() -> &'static [(&'static str, &'static str)] {
    match ACTIVE_MODEL {
        ModelTarget::Development => DEV_MODELS,
        ModelTarget::Production => PROD_MODELS,
    }
}

async fn resolve_remote_parity(client: &reqwest::Client, url: &str) -> Result<(String, u64), String> {
    let res = client.head(url).send().await.map_err(|e| e.to_string())?;
    let final_url = res.url().to_string();
    let size = res.content_length().unwrap_or(0);
    Ok((final_url, size))
}

#[tauri::command]
pub async fn check_model_status(app: AppHandle) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);

    if !base_path.exists() {
        return Ok(IpcResponse { success: true, data: Some(false), error_message: None });
    }

    let client = reqwest::Client::new();
    let mut ready = true;

    for (filename, url) in get_active_models() {
        let file_path = base_path.join(filename);
        if !file_path.exists() {
            ready = false;
            continue;
        }

        let local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
        if let Ok((_, remote_size)) = resolve_remote_parity(&client, url).await {
            if remote_size > 0 && local_size != remote_size {
                ready = false;
            }
        }
    }
    Ok(IpcResponse { success: true, data: Some(ready), error_message: None })
}

#[tauri::command]
pub async fn start_model_download(app: AppHandle) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);

    if !base_path.exists() {
        fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
    }

    let client = reqwest::Client::new();
    let models = get_active_models();
    let total_parts = models.len() as u32;

    for (index, (filename, url)) in models.iter().enumerate() {
        let part_current = (index + 1) as u32;
        let file_path = base_path.join(filename);
        let local_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        let (direct_url, remote_size) = resolve_remote_parity(&client, url).await?;

        if local_size > 0 && remote_size > 0 && local_size == remote_size {
             let _ = app.emit("download_progress", DownloadProgressEvent { 
                part_current, part_total: total_parts, downloaded_bytes: remote_size, total_bytes: remote_size 
            });
            continue; 
        }

        let mut request = client.get(&direct_url);
        let mut is_resuming = false;

        if local_size > 0 && local_size < remote_size {
            request = request.header(RANGE, format!("bytes={}-", local_size));
            is_resuming = true;
        }

        let res = request.send().await.map_err(|e| e.to_string())?;
        if res.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            continue;
        }

        if !res.status().is_success() {
            return Err(format!("Storage Failure: HTTP {}", res.status()));
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(is_resuming)
            .write(true)
            .truncate(!is_resuming)
            .open(&file_path)
            .map_err(|e| e.to_string())?;

        let mut stream = res.bytes_stream();
        let mut downloaded_bytes: u64 = local_size;
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            downloaded_bytes += chunk.len() as u64;

            if last_emit.elapsed().as_millis() > 100 {
                let _ = app.emit("download_progress", DownloadProgressEvent { 
                    part_current, part_total: total_parts, downloaded_bytes, total_bytes: remote_size 
                });
                last_emit = std::time::Instant::now();
            }
        }

        let _ = app.emit("download_progress", DownloadProgressEvent { 
            part_current, part_total: total_parts, downloaded_bytes: remote_size, total_bytes: remote_size 
        });
    }
    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}