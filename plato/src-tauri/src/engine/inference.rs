use std::path::PathBuf;
use std::sync::Arc;
use std::num::NonZeroU32;
use tauri::{AppHandle, Emitter};
use tokio::task;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::AddBos;

#[allow(deprecated)]
use llama_cpp_2::model::Special;
use crate::models::ChatTokenEvent;

pub struct InferenceEngine {
    // Backend must be kept alive alongside the model
    pub backend: Arc<LlamaBackend>,
    pub model: Arc<LlamaModel>,
}

impl InferenceEngine {
    pub fn new(model_path: PathBuf) -> Result<Self, String> {
        let backend = LlamaBackend::init()
            .map_err(|e| format!("LlamaBackend initialization failed: {}", e))?;
        
        let model_params = LlamaModelParams::default();
        
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
            .map_err(|e| format!("Failed to load model from file: {}", e))?;

        Ok(Self { 
            backend: Arc::new(backend),
            model: Arc::new(model) 
        })
    }

    /**
     * Executes the true C++ generative neural network on a dedicated thread.
     * Evaluates the prompt and emits predicted tokens via IPC in real-time.
     */
    pub async fn stream_response(&self, app: AppHandle, message_id: String, prompt: String) -> Result<(), String> {
        let message_id_clone = message_id.clone();
        
        // Safely clone references to move into the blocking thread
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);

        task::spawn_blocking(move || -> Result<(), String> {
            let formatted_prompt = format!("<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", prompt);
            
            let tokens_list = model.str_to_token(&formatted_prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed: {}", e))?;

            let max_context_size: u32 = 2048;
            let mut ctx_params = LlamaContextParams::default();
            ctx_params = ctx_params.with_n_ctx(NonZeroU32::new(max_context_size));
            
            // Pass the backend reference as required by the API
            let mut ctx = model.new_context(&backend, ctx_params)
                .map_err(|e| format!("Context creation failed: {}", e))?;

            let mut batch = LlamaBatch::new(512, 1);
            let last_index = tokens_list.len() - 1;

            for (i, &token) in tokens_list.iter().enumerate() {
                let is_last = i == last_index;
                batch.add(token, i as i32, &[0], is_last).map_err(|e| e.to_string())?;
            }
            ctx.decode(&mut batch).map_err(|e| format!("Prompt evaluation failed: {}", e))?;

            let mut n_cur = batch.n_tokens();

            loop {
                // Determine the next token
                let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
                let mut candidates_p = llama_cpp_2::token::data_array::LlamaTokenDataArray::from_iter(candidates, false);
                
                // Use sample_token_greedy on the context, passing a mutable reference to the candidates array
                ctx.sample_candidates(&mut candidates_p);
                let new_token_id = ctx.sample_token_greedy(candidates_p);

                // Check EOS
                if new_token_id == model.token_eos() {
                    break;
                }

                // Structural protection against C++ segfaults (cast to match max_context_size type)
                if (n_cur as u32) >= max_context_size - 1 {
                    break;
                }

                #[allow(deprecated)]
                let token_bytes = model.token_to_bytes(new_token_id, Special::Tokenize)
                    .unwrap_or_default();
                let token_str = String::from_utf8_lossy(&token_bytes).to_string();

                // Stop manually if Llama 3 emits its string EOT marker
                if token_str.contains("<|eot_id|>") {
                    break;
                }

                let _ = app.emit("chat_token", ChatTokenEvent {
                    message_id: message_id_clone.clone(),
                    token: token_str,
                    is_final: false,
                });

                batch.clear();
                batch.add(new_token_id, n_cur, &[0], true).map_err(|e| e.to_string())?;
                n_cur += 1;
                
                ctx.decode(&mut batch).map_err(|e| format!("Decode loop failed: {}", e))?;
            }

            let _ = app.emit("chat_token", ChatTokenEvent {
                message_id: message_id_clone,
                token: "".to_string(),
                is_final: true,
            });

            Ok(())
        }).await.map_err(|e| format!("Thread panic: {}", e))??;

        Ok(())
    }
}