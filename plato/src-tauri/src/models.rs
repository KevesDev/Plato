use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemHealthStatus {
    pub is_platform_supported: bool,
    pub available_memory_mb: u64,
    pub model_exists: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkspaceState {
    pub active_directory_path: Option<String>,
    pub indexed_file_count: u32,
    pub is_indexing: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error_message: Option<String>,
}


/// Payload structure emitted to the frontend during multi-part file downloads.
/// Ensures the UI has precise data to calculate the progress bar per chunk.
#[derive(Serialize, Deserialize, Clone)]
pub struct DownloadProgressEvent {
    pub part_current: u32,
    pub part_total: u32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}