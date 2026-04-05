pub mod inference;

use std::sync::Arc;
use tokio::sync::Mutex;
use self::inference::InferenceEngine;

/**
 * PlatoEngineState
 * Manages the persistent lifecycle of the LLM context.
 * This structure is managed by Tauri and injected into commands to ensure
 * the 62.8GB model remains resident in memory between prompts.
 */
pub struct PlatoEngineState {
    pub engine: Arc<Mutex<Option<InferenceEngine>>>,
}

impl Default for PlatoEngineState {
    fn default() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
        }
    }
}