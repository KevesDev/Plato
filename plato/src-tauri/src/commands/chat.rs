use tauri::{AppHandle, State};
use crate::engine::PlatoEngineState;
use crate::models::{IpcResponse, InferenceConfig};

#[tauri::command]
pub async fn stream_chat_completion(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    message_id: String,
    prompt: String,
    config: InferenceConfig,
) -> Result<IpcResponse<bool>, String> {
    let engine_guard = state.engine.lock().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        engine.stream_response(app, message_id, prompt, config).await?;
        Ok(IpcResponse { success: true, data: Some(true), error_message: None })
    } else {
        Err("Critical Failure: Inference Engine is uninitialized.".to_string())
    }
}