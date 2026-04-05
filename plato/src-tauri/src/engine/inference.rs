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
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

#[allow(deprecated)]
use llama_cpp_2::model::Special;
use crate::models::{ChatTokenEvent, InferenceConfig};

pub struct InferenceEngine {
    pub backend: Arc<LlamaBackend>,
    pub model: Arc<LlamaModel>,
}

impl InferenceEngine {
    pub fn new(model_path: PathBuf) -> Result<Self, String> {
        let backend = LlamaBackend::init().map_err(|e| format!("Backend failed: {}", e))?;
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, model_path, &model_params).map_err(|e| format!("Load failed: {}", e))?;
        Ok(Self { backend: Arc::new(backend), model: Arc::new(model) })
    }

    /**
     * Executes inference via Compile-Safe Explicit Token Parsing.
     * Prevents loops by ensuring structural markers are treated as control IDs,
     * and compiles safely by leaving flash attention defaults intact.
     */
    pub async fn stream_response(
        &self, 
        app: AppHandle, 
        message_id: String, 
        prompt: String, 
        config: InferenceConfig
    ) -> Result<(), String> {
        let message_id_clone = message_id.clone();
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);

        task::spawn_blocking(move || -> Result<(), String> {
            let mut tokens_list = Vec::new();

            // 1. DYNAMIC CONTROL ID DISCOVERY
            let get_id = |s: &str, fallback: i32| -> LlamaToken {
                let t = model.str_to_token(s, AddBos::Never).unwrap_or_default();
                if t.len() == 1 { t[0] } else { LlamaToken(fallback) }
            };

            let bos = model.token_bos();
            let start = get_id("<|start_header_id|>", 128006);
            let end = get_id("<|end_header_id|>", 128007);
            let eot = get_id("<|eot_id|>", 128009);

            // 2. BINARY SEQUENCE CONSTRUCTION
            tokens_list.push(bos);
            tokens_list.push(start);
            tokens_list.extend(model.str_to_token("system", AddBos::Never).unwrap());
            tokens_list.push(end);
            tokens_list.extend(model.str_to_token("\nYou are a creative assistant.\n", AddBos::Never).unwrap());
            tokens_list.push(eot);
            tokens_list.push(start);
            tokens_list.extend(model.str_to_token("user", AddBos::Never).unwrap());
            tokens_list.push(end);
            tokens_list.extend(model.str_to_token(&format!("\n{}\n", prompt), AddBos::Never).unwrap());
            tokens_list.push(eot);
            tokens_list.push(start);
            tokens_list.extend(model.str_to_token("assistant", AddBos::Never).unwrap());
            tokens_list.push(end);
            tokens_list.extend(model.str_to_token("\n", AddBos::Never).unwrap());

            let max_context_size: u32 = 2048;
            let mut ctx_params = LlamaContextParams::default();
            
            // COMPILER FIX: Removed the .with_flash_attention_policy(false) call.
            ctx_params = ctx_params.with_n_ctx(NonZeroU32::new(max_context_size));
            
            let mut ctx = model.new_context(&backend, ctx_params).map_err(|e| e.to_string())?;
            let mut batch = LlamaBatch::new(512, 1);
            let last_idx = tokens_list.len() - 1;

            for (i, &token) in tokens_list.iter().enumerate() {
                batch.add(token, i as i32, &[0], i == last_idx).map_err(|e| e.to_string())?;
            }
            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            let mut current_pos = batch.n_tokens();
            
            // 3. STABILIZED SAMPLER CHAIN
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::penalties(64, 1.2, 0.0, 0.0),
                LlamaSampler::top_k(40),
                LlamaSampler::top_p(0.95, 1),
                LlamaSampler::temp(config.temperature),
                LlamaSampler::greedy(), // Deterministic selection 
            ]);

            for &token in &tokens_list { sampler.accept(token); }

            loop {
                let logit_idx = batch.n_tokens() - 1;
                let new_token_id = sampler.sample(&ctx, logit_idx);
                sampler.accept(new_token_id);

                if model.is_eog_token(new_token_id) { break; }
                if (current_pos as u32) >= max_context_size - 1 { break; }

                #[allow(deprecated)]
                let token_bytes = model.token_to_bytes(new_token_id, Special::Tokenize).unwrap_or_default();
                let token_str = String::from_utf8_lossy(&token_bytes).to_string();

                let _ = app.emit("chat_token", ChatTokenEvent { 
                    message_id: message_id_clone.clone(), token: token_str, is_final: false 
                });

                batch.clear();
                batch.add(new_token_id, current_pos, &[0], true).map_err(|e| e.to_string())?;
                current_pos += 1;
                ctx.decode(&mut batch).map_err(|e| e.to_string())?;
            }

            let _ = app.emit("chat_token", ChatTokenEvent { message_id: message_id_clone, token: "".to_string(), is_final: true });
            Ok(())
        }).await.map_err(|e| e.to_string())??;

        Ok(())
    }
}