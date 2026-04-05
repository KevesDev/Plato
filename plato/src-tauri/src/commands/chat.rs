use tauri::{AppHandle, State};
use crate::engine::PlatoEngineState;
use crate::models::IpcResponse;

/**
 * Secures the engine lock and delegates the user prompt to the inference pipeline.
 * Returns immediately upon stream completion, relying on side-effect IPC events
 * to deliver the actual payload to the UI in real-time.
 */
#[tauri::command]
pub async fn stream_chat_completion(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    message_id: String,
    prompt: String,
) -> Result<IpcResponse<bool>, String> {
    let engine_guard = state.engine.lock().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        engine.stream_response(app, message_id, prompt).await?;
        Ok(IpcResponse { success: true, data: Some(true), error_message: None })
    } else {
        Err("Critical Failure: Inference Engine memory address is null. Boot sequence failed.".to_string())
    }
}