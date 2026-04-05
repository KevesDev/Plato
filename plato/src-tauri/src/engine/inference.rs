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
            let formatted_prompt = match ACTIVE_MODEL {
                ModelTarget::Development => format!("<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", prompt),
                ModelTarget::Production => format!("<|START_OF_TURN_TOKEN|><|USER_TOKEN|>{}<|END_OF_TURN_TOKEN|><|START_OF_TURN_TOKEN|><|CHATBOT_TOKEN|>", prompt),
            };

            let max_context_size: u32 = match ACTIVE_MODEL {
                ModelTarget::Development => 2048,
                ModelTarget::Production => 8192,
            };

            let tokens_list = model.str_to_token(&formatted_prompt, AddBos::Always).map_err(|e| e.to_string())?;
            let mut ctx_params = LlamaContextParams::default();
            ctx_params = ctx_params.with_n_ctx(NonZeroU32::new(max_context_size));
            
            let mut ctx = model.new_context(&backend, ctx_params).map_err(|e| e.to_string())?;
            let mut batch = LlamaBatch::new(512, 1);
            let last_index = tokens_list.len() - 1;

            for (i, &token) in tokens_list.iter().enumerate() {
                let is_last = i == last_index;
                batch.add(token, i as i32, &[0], is_last).map_err(|e| e.to_string())?;
            }
            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            let mut n_cur = batch.n_tokens();
            
            // Sampler chain combines creative variety with loop penalties.
            // Signatures verified for llama-cpp-2 v0.1.141.
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::dist(42), 
                LlamaSampler::temp(config.temperature), 
                LlamaSampler::top_p(config.top_p, config.min_keep), 
                LlamaSampler::top_k(config.top_k),
                LlamaSampler::penalties(config.repeat_last_n, config.repeat_penalty, 0.0, 0.0),
                LlamaSampler::greedy(),
            ]);

            loop {
                let new_token_id = sampler.sample(&ctx, batch.n_tokens() - 1);
                if model.is_eog_token(new_token_id) { break; }
                if (n_cur as u32) >= max_context_size - 1 { break; }

                #[allow(deprecated)]
                let token_bytes = model.token_to_bytes(new_token_id, Special::Tokenize).unwrap_or_default();
                let token_str = String::from_utf8_lossy(&token_bytes).to_string();

                let stop_marker = match ACTIVE_MODEL {
                    ModelTarget::Development => "<|eot_id|>",
                    ModelTarget::Production => "<|END_OF_TURN_TOKEN|>",
                };

                if token_str.contains(stop_marker) { break; }

                let _ = app.emit("chat_token", ChatTokenEvent { message_id: message_id_clone.clone(), token: token_str, is_final: false });

                batch.clear();
                batch.add(new_token_id, n_cur, &[0], true).map_err(|e| e.to_string())?;
                n_cur += 1;
                ctx.decode(&mut batch).map_err(|e| e.to_string())?;
            }

            let _ = app.emit("chat_token", ChatTokenEvent { message_id: message_id_clone, token: "".to_string(), is_final: true });
            Ok(())
        }).await.map_err(|e| e.to_string())??;

        Ok(())
    }
}