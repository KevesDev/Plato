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
    mut history: Vec<ChatMessage>, 
    config: InferenceConfig,
) -> Result<IpcResponse<String>, String> {
    
    // RAG INJECTION PIPELINE
    // Intercepts the last user message, searches the memory matrix, and mutates the prompt 
    // to include verified world lore with strict semantic boundaries.
    if let Some(last_msg) = history.last_mut() {
        if last_msg.role == "user" {
            let query = last_msg.content.clone();
            
            let vector_db_guard = state.vector_db.lock().await;
            if let Some(vector_db) = vector_db_guard.as_ref() {
                // Retrieve the top 3 most relevant passages for the active context window
                if let Ok(context_chunks) = vector_db.search_matrix(&query, 3).await {
                    if !context_chunks.is_empty() {
                        // Separate the chunks clearly so the AI knows they might not be related
                        let compiled_context = context_chunks.join("\n\n---\n\n");
                        last_msg.content = format!(
                            "System Context: You are Plato, an AGI world-building assistant. You have been provided with retrieved excerpts from the user's Story Vault below.\n\nCRITICAL RULES:\n1. You must answer the user's prompt STRICTLY AND ONLY using the provided lore.\n2. DO NOT invent, hallucinate, or weave together connections between distinct characters, entities, or locations unless the text explicitly states they are connected.\n3. If the answer is not clearly contained within the provided lore, you must explicitly state that you do not have enough information in the active matrix.\n\n[VERIFIED LORE]\n{}\n[/VERIFIED LORE]\n\nUser Prompt: {}", 
                            compiled_context, query
                        );
                    }
                }
            }
            // Explicitly drop the database lock to free up memory before hitting LLaMA
            drop(vector_db_guard); 
        }
    }

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