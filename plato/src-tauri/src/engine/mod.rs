use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::AtomicBool;
use crate::engine::inference::InferenceEngine;

pub mod inference;

pub struct PlatoEngineState {
    pub engine: Arc<Mutex<Option<InferenceEngine>>>,
    // Atomic signal allowing the UI to interrupt generation across threads
    pub abort_signal: Arc<AtomicBool>,
}

impl PlatoEngineState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            abort_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for PlatoEngineState {
    fn default() -> Self {
        Self::new()
    }
}