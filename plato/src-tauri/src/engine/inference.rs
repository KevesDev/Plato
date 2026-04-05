use std::path::PathBuf;
use std::sync::Arc;
use std::num::NonZeroU32;
use std::time::{SystemTime, UNIX_EPOCH};
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
use crate::models::{ChatTokenEvent, InferenceConfig, ACTIVE_MODEL, ModelTarget};

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
     * Executes inference via Thread-Locked Synchronous Context.
     * Prevents NaN logit collapse by explicitly binding to 8 threads (P-cores)
     * to prevent hybrid architecture desynchronization during matrix multiplication.
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

            // 1. EXACT LLAMA 3.1 BINARY IDS
            let bos = LlamaToken(128000);
            let start = LlamaToken(128006);
            let end = LlamaToken(128007);
            let eot = LlamaToken(128009);
            let lf = LlamaToken(198); // The Newline Token

            tokens_list.push(bos);

            match ACTIVE_MODEL {
                ModelTarget::Development => {
                    // Absolute Binary Precision: Inject exactly what the Attention Heads demand.
                    tokens_list.push(start);
                    tokens_list.extend(model.str_to_token("system", AddBos::Never).unwrap_or_default());
                    tokens_list.push(end);
                    tokens_list.push(lf); tokens_list.push(lf); // \n\n
                    tokens_list.extend(model.str_to_token("You are Plato, a creative writing assistant.", AddBos::Never).unwrap_or_default());
                    tokens_list.push(eot);
                    
                    tokens_list.push(start);
                    tokens_list.extend(model.str_to_token("user", AddBos::Never).unwrap_or_default());
                    tokens_list.push(end);
                    tokens_list.push(lf); tokens_list.push(lf); // \n\n
                    tokens_list.extend(model.str_to_token(&prompt, AddBos::Never).unwrap_or_default());
                    tokens_list.push(eot);
                    
                    tokens_list.push(start);
                    tokens_list.extend(model.str_to_token("assistant", AddBos::Never).unwrap_or_default());
                    tokens_list.push(end);
                    tokens_list.push(lf); tokens_list.push(lf); // \n\n
                },
                ModelTarget::Production => {
                    tokens_list.extend(model.str_to_token(&prompt, AddBos::Never).unwrap_or_default());
                }
            }

            let max_context_size: u32 = 2048;
            
            // 2. HARDWARE FIX: Restrict to 8 threads (P-Cores only) 
            // This prevents E-Core desync and the resulting NaN math collapse.
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(max_context_size))
                .with_n_threads(8);
            
            let mut ctx = model.new_context(&backend, ctx_params).map_err(|e| e.to_string())?;
            let mut batch = LlamaBatch::new(512, 1);
            let last_idx = tokens_list.len().saturating_sub(1);

            // Ingest prompt into KV Cache
            for (i, &token) in tokens_list.iter().enumerate() {
                batch.add(token, i as i32, &[0], i == last_idx).map_err(|e| e.to_string())?;
            }
            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            let mut current_pos = batch.n_tokens();
            
            // 3. Stochastic Sampler 
            let temp = if config.temperature <= 0.0 { 0.7 } else { config.temperature };
            let repeat_penalty = if config.repeat_penalty <= 1.0 { 1.1 } else { config.repeat_penalty };
            let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();

            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::penalties(64, repeat_penalty, 0.0, 0.0),
                LlamaSampler::top_k(40),
                LlamaSampler::top_p(0.95, 1),
                LlamaSampler::temp(temp),
                LlamaSampler::dist(seed), 
            ]);

            for &token in &tokens_list { sampler.accept(token); }

            loop {
                let logit_idx = batch.n_tokens() - 1;
                let new_token_id = sampler.sample(&ctx, logit_idx);
                sampler.accept(new_token_id);

                if new_token_id == eot || model.is_eog_token(new_token_id) { break; }
                if (current_pos as u32) >= max_context_size - 1 { break; }

                #[allow(deprecated)]
                let token_bytes = model.token_to_bytes(new_token_id, Special::Tokenize).unwrap_or_default();
                let token_str = String::from_utf8_lossy(&token_bytes).to_string();

                if token_str.contains("<|eot_id|>") { break; }

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