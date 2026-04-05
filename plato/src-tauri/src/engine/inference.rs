use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
use crate::models::{ChatTokenEvent, InferenceConfig, ChatMessage, ACTIVE_MODEL, ModelTarget};

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
        history: Vec<ChatMessage>, 
        config: InferenceConfig,
        abort_signal: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let message_id_clone = message_id.clone();
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);

        task::spawn_blocking(move || -> Result<(), String> {
            let mut tokens_list = Vec::new();

            let get_id = |s: &str| -> LlamaToken {
                let t = model.str_to_token(s, AddBos::Never).unwrap_or_default();
                if !t.is_empty() { t[0] } else { LlamaToken(0) }
            };

            let start_turn = get_id("<|START_OF_TURN_TOKEN|>");
            let end_turn = get_id("<|END_OF_TURN_TOKEN|>");
            let sys_role = get_id("<|SYSTEM_TOKEN|>");
            let user_role = get_id("<|USER_TOKEN|>");
            let bot_role = get_id("<|CHATBOT_TOKEN|>");

            tokens_list.push(model.token_bos());

            if matches!(ACTIVE_MODEL, ModelTarget::Development) {
                // SYSTEM: Stronger persona alignment and negative constraints
                tokens_list.push(start_turn);
                tokens_list.push(sys_role);
                let system_prompt = "You are Plato, a creative writing AGI assistant. You care for the user and provide concise responses. CRITICAL: Do NOT introduce yourself or say 'As Plato'. Just start the answer.";
                tokens_list.extend(model.str_to_token(system_prompt, AddBos::Never).unwrap_or_default());
                tokens_list.push(end_turn);

                // HISTORY: Sequential ingestion for context memory
                for msg in history {
                    let role_token = if msg.role == "user" { user_role } else { bot_role };
                    tokens_list.push(start_turn);
                    tokens_list.push(role_token);
                    tokens_list.extend(model.str_to_token(&msg.content, AddBos::Never).unwrap_or_default());
                    tokens_list.push(end_turn);
                }

                tokens_list.push(start_turn);
                tokens_list.push(bot_role);
            }

            let max_context_size: u32 = 2048;
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(max_context_size))
                .with_n_threads(8);
            
            let mut ctx = model.new_context(&backend, ctx_params).map_err(|e| e.to_string())?;
            let mut batch = LlamaBatch::new(1024, 1);
            let last_idx = tokens_list.len().saturating_sub(1);

            for (i, &token) in tokens_list.iter().enumerate() {
                batch.add(token, i as i32, &[0], i == last_idx).map_err(|e| e.to_string())?;
            }
            ctx.decode(&mut batch).map_err(|e| e.to_string())?;

            let mut current_pos = batch.n_tokens();
            let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();

            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::penalties(64, config.repeat_penalty, 0.0, 0.0),
                LlamaSampler::top_k(config.top_k),
                LlamaSampler::top_p(config.top_p, 1),
                LlamaSampler::temp(config.temperature),
                LlamaSampler::dist(seed), 
            ]);

            for &token in &tokens_list { sampler.accept(token); }

            loop {
                // ABORT CHECK: Immediate termination on UI signal
                if abort_signal.load(Ordering::Relaxed) { break; }

                let logit_idx = batch.n_tokens() - 1;
                let new_token_id = sampler.sample(&ctx, logit_idx);
                sampler.accept(new_token_id);

                if new_token_id == end_turn || model.is_eog_token(new_token_id) { break; }
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