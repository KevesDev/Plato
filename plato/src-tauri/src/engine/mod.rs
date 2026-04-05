use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::AtomicBool;
use crate::engine::inference::InferenceEngine;
use crate::engine::vector_db::VectorDatabase;

pub mod inference;
pub mod vector_db;

/**
 * Global application state housing thread-safe handles to the heavy subsystems.
 */
pub struct PlatoEngineState {
    pub engine: Arc<Mutex<Option<InferenceEngine>>>,
    pub abort_signal: Arc<AtomicBool>,
    // Thread-safe handle to the semantic memory layer
    pub vector_db: Arc<Mutex<Option<VectorDatabase>>>,
}

impl PlatoEngineState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            abort_signal: Arc::new(AtomicBool::new(false)),
            vector_db: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for PlatoEngineState {
    fn default() -> Self {
        Self::new()
    }
}