use serde::{Deserialize, Serialize};

// --- Phase 1: Workspace & State Models ---

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkspaceState {
    pub active_directory_path: Option<String>,
    pub indexed_file_count: u32,
    pub is_indexing: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemHealthStatus {
    pub is_platform_supported: bool,
    pub available_memory_mb: u64,
    pub model_exists: bool,
}

// --- Phase 2: Inference & Engine Models ---

/**
 * MASTER SWITCH: Toggle this to ModelTarget::Production to switch 
 * the entire application stack to Command R+ (104B).
 */
pub const ACTIVE_MODEL: ModelTarget = ModelTarget::Development;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelTarget {
    Development, // Llama 3.1 8B
    Production,  // Command R+ 104B
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadProgressEvent {
    pub part_current: u32,
    pub part_total: u32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InferenceConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub min_keep: usize,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub repeat_last_n: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatTokenEvent {
    pub message_id: String,
    pub token: String,
    pub is_final: bool,
}