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
    context_entities: Option<Vec<String>>
) -> Result<IpcResponse<String>, String> {
    
    let mut compiled_context = Vec::new();
    let mut is_autowrite = false;

    let vector_db_guard = state.vector_db.lock().await;
    if let Some(vector_db) = vector_db_guard.as_ref() {
        if let Some(entities) = context_entities {
            // Task 3.3: EDITOR MODE (Explicit Tagging)
            // Uses exact string matching to guarantee retrieval of the Lorebook cards.
            is_autowrite = true;
            for entity in entities {
                if let Some(chunk) = vector_db.get_exact_lore_entry(&entity).await {
                    compiled_context.push(chunk);
                }
            }
        } else if let Some(last_msg) = history.last() {
            // SIDEBAR MODE (Implicit Search)
            if last_msg.role == "user" {
                let prompt = &last_msg.content;
                
                // 1. Auto-detect explicit Lorebook names in the user's prompt (Fixes the N/A bug)
                let auto_lore = vector_db.get_lore_entries_in_text(prompt).await;
                compiled_context.extend(auto_lore.clone());

                // 2. Perform standard mathematical search for implicit world lore
                if let Ok(chunks) = vector_db.search_matrix(prompt, 3).await {
                    for chunk in chunks {
                        if !compiled_context.contains(&chunk) {
                            compiled_context.push(chunk);
                        }
                    }
                }
            }
        }
    }
    drop(vector_db_guard); 

    if !compiled_context.is_empty() {
        let context_str = compiled_context.join("\n\n---\n\n");

        if is_autowrite {
            if let Some(system_msg) = history.iter_mut().find(|m| m.role == "system") {
                system_msg.content = format!(
                    "{}\n\n[VERIFIED WORLD LORE FOR TAGGED ENTITIES]\n{}\n[/VERIFIED WORLD LORE]\n\nCRITICAL: You must adhere strictly to the personalities, traits, and facts described in the Verified Lore above. Do not contradict this lore.",
                    system_msg.content, context_str
                );
            }
        } else {
            if let Some(last_msg) = history.last_mut() {
                let original_query = last_msg.content.clone();
                last_msg.content = format!(
                    "System Context: You are Plato, an analytical world-building assistant. You have been provided with retrieved excerpts from the user's Story Vault below.\n\nCRITICAL RULES:\n1. You must answer the user's prompt STRICTLY AND ONLY using the provided lore.\n2. DO NOT invent, hallucinate, or weave together connections between distinct characters, entities, or locations unless the text explicitly states they are connected.\n3. If the answer is not clearly contained within the provided lore, you must explicitly state that you do not have enough information in the active matrix.\n\n[VERIFIED LORE]\n{}\n[/VERIFIED LORE]\n\nUser Prompt: {}", 
                    context_str, original_query
                );
            }
        }
    }

    let engine_mutex = state.engine.clone();
    let abort_signal = state.abort_signal.clone();
    
    abort_signal.store(false, Ordering::Relaxed);

    let engine_lock = engine_mutex.lock().await;

    if let Some(engine) = engine_lock.as_ref() {
        let engine_handle = engine.clone();
        
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