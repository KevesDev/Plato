use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tokio::task;
use crate::engine::PlatoEngineState;
use crate::engine::inference::InferenceEngine;
use crate::models::{IpcResponse, ACTIVE_MODEL, ModelTarget};

const MODEL_DIR: &str = "models";

/**
 * Dynamically maps the target environment to the corresponding primary GGUF chunk.
 * The underlying C++ bindings automatically discover trailing sequential chunks.
 */
fn get_primary_chunk() -> &'static str {
    match ACTIVE_MODEL {
        ModelTarget::Development => "aya-23-8b-q4_k_m.gguf",
        ModelTarget::Production => "c4ai-command-r-plus-Q4_K_M-00001-of-00006.gguf",
    }
}

/**
 * Initializes the C++ binding context and mounts the model weights to system RAM.
 * Offloaded to a blocking thread to preserve application UI responsiveness during high I/O.
 */
#[tauri::command]
pub async fn initialize_engine(
    app: AppHandle, 
    state: State<'_, PlatoEngineState>
) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);
    base_path.push(get_primary_chunk());

    let mut engine_guard = state.engine.lock().await;

    if engine_guard.is_none() {
        let model_path = base_path.clone();
        
        let engine = task::spawn_blocking(move || {
            InferenceEngine::new(model_path)
        }).await.map_err(|e| format!("Thread panic during initialization: {}", e))??;
        
        *engine_guard = Some(engine);
    }

    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}