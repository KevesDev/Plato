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

/// Emitted by the Inference Engine during text generation.
/// Carries a unique identifier to ensure the frontend appends the token
/// to the correct message block in the chat history.
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatTokenEvent {
    pub message_id: String,
    pub token: String,
    pub is_final: bool,
}