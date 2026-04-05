export interface SystemHealthStatus {
    isPlatformSupported: boolean;
    availableMemoryMb: number;
    modelExists: boolean;
}

export interface WorkspaceState {
    activeDirectoryPath: string | null;
    indexedFileCount: number;
    isIndexing: boolean;
}

export interface IpcResponse<T> {
    success: boolean;
    data?: T;
    errorMessage?: string;
}