use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use futures_util::StreamExt;
use crate::models::{IpcResponse, DownloadProgressEvent};

const MODEL_DIR: &str = "models";
const MODEL_FILES: &[(&str, &str)] = &[
    ("c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00002-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00003-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00004-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00005-of-00006.gguf"),
    ("c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf", "https://huggingface.co/bartowski/c4ai-command-r-plus-GGUF/resolve/main/c4ai-command-r-plus-Q4_K_M-00006-of-00006.gguf"),
];

#[tauri::command]
pub async fn verify_and_download_model(app: AppHandle) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);

    if !base_path.exists() {
        fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
    }

    let mut all_files_exist = true;
    for (filename, _) in MODEL_FILES {
        if !base_path.join(filename).exists() {
            all_files_exist = false;
            break;
        }
    }

    if all_files_exist {
        return Ok(IpcResponse { success: true, data: Some(true), error_message: None });
    }

    let client = reqwest::Client::new();
    let total_parts = MODEL_FILES.len() as u32;

    for (index, (filename, url)) in MODEL_FILES.iter().enumerate() {
        let part_current = (index + 1) as u32;
        let file_path = base_path.join(filename);

        if file_path.exists() { continue; }

        let res = client.get(*url).send().await.map_err(|e| e.to_string())?;
        let total_bytes = res.content_length().unwrap_or(0);

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