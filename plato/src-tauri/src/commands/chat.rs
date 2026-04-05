use std::sync::atomic::Ordering;
use tauri::{AppHandle, State};
use crate::engine::PlatoEngineState;
use crate::models::{IpcResponse, InferenceConfig, ChatMessage};

#[tauri::command]
pub async fn abort_inference(state: State<'_, PlatoEngineState>) -> Result<IpcResponse<bool>, String> {
    state.abort_signal.store(true, Ordering::Relaxed);
    Ok(IpcResponse { success: true, data: Some(true), error_message: None })
}

#[tauri::command]
pub async fn stream_chat_completion(
    app: AppHandle,
    state: State<'_, PlatoEngineState>,
    message_id: String,
    history: Vec<ChatMessage>, 
    config: InferenceConfig,
) -> Result<IpcResponse<String>, String> {
    let engine_mutex = state.engine.clone();
    let abort_signal = state.abort_signal.clone();
    
    abort_signal.store(false, Ordering::Relaxed);

    let engine_lock = engine_mutex.lock().await;

    if let Some(engine) = engine_lock.as_ref() {
        let engine_handle = engine.clone();
        
        // Explicitly drop guard before spawning to ensure concurrency
        drop(engine_lock);
        
        tokio::spawn(async move {
            if let Err(e) = engine_handle.stream_response(app, message_id, history, config, abort_signal).await {
                eprintln!("[Engine Error]: {}", e);
            }
        });

        Ok(IpcResponse { success: true, data: Some("Stream Initialized".to_string()), error_message: None })
    } else {
        Ok(IpcResponse { 
            success: false, 
            data: None, 
            error_message: Some("Engine uninitialized. Please complete onboarding.".to_string()) 
        })
    }
}