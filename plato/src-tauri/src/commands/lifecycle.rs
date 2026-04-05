use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tokio::task;
use crate::engine::PlatoEngineState;
use crate::engine::inference::InferenceEngine;
use crate::models::IpcResponse;

const MODEL_DIR: &str = "models";
// Configured for the lightweight Dev model to fit in 32GB RAM
const PRIMARY_CHUNK: &str = "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf";

/**
 * Orchestrates the transition of the LlamaModel from disk to RAM.
 * Executed immediately after the onboarding download succeeds.
 */
#[tauri::command]
pub async fn initialize_engine(
    app: AppHandle, 
    state: State<'_, PlatoEngineState>
) -> Result<IpcResponse<bool>, String> {
    let mut base_path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base_path.push(MODEL_DIR);
    base_path.push(PRIMARY_CHUNK);

    let mut engine_guard = state.engine.lock().await;

    // Prevent redundant memory allocations if the engine is already active
    if engine_guard.is_none() {
        let model_path = base_path.clone();
        
        // Disk I/O and C++ binding initialization is heavily blocking.
        // Offloading to a dedicated thread ensures the application remains responsive.
        let engine = task::spawn_blocking(move || {
            InferenceEngine::new(model_path)
        }).await.map_err(|e| format!("Thread panic during initialization: {}", e))??;
        
        *engine_guard = Some(engine);
    }

    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}