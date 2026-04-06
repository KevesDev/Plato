import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen, RefreshCw, Wand2, AtSign, Square } from 'lucide-react';

export interface WorkspaceState {
    active_directory_path: string | null;
    indexed_file_count: number;
    is_indexing: boolean;
}

interface ToolbarProps {
    workspace: WorkspaceState;
    setWorkspace: React.Dispatch<React.SetStateAction<WorkspaceState>>;
}

export const Toolbar: React.FC<ToolbarProps> = ({ workspace, setWorkspace }) => {
    const [isAutowriting, setIsAutowriting] = useState(false);
    
    useEffect(() => {
        const handleWorkspaceUpdate = (e: Event) => {
            const customEvent = e as CustomEvent;
            if (customEvent.detail && typeof customEvent.detail.indexed_file_count === 'number') {
                setWorkspace(prev => ({ 
                    ...prev, 
                    indexed_file_count: customEvent.detail.indexed_file_count 
                }));
            }
        };

        const handleAutowriteState = (e: Event) => {
            const customEvent = e as CustomEvent;
            setIsAutowriting(customEvent.detail);
        };
        
        window.addEventListener('plato-workspace-update', handleWorkspaceUpdate);
        window.addEventListener('plato-autowrite-state', handleAutowriteState);
        return () => {
            window.removeEventListener('plato-workspace-update', handleWorkspaceUpdate);
            window.removeEventListener('plato-autowrite-state', handleAutowriteState);
        };
    }, [setWorkspace]);

    const handleSelectWorkspace = async () => {
        try {
            const response = await invoke<{ success: boolean; data?: WorkspaceState }>('select_and_scan_workspace');
            if (response.success && response.data) {
                setWorkspace(response.data);
            }
        } catch (error) {
            console.error('[Workspace Error]:', error);
        }
    };

    const handleSyncDatabase = async () => {
        if (!workspace.active_directory_path || workspace.is_indexing) return;
        
        setWorkspace(prev => ({ ...prev, is_indexing: true }));
        try {
            const response = await invoke<{ success: boolean; data?: number }>('start_workspace_ingestion', { 
                path: workspace.active_directory_path 
            });
            
            if (response.success) {
                window.dispatchEvent(new CustomEvent('plato-sys-msg', { 
                    detail: `Memory synchronization complete. I have woven ${response.data} passages from your vault into my consciousness. I am ready.`
                }));
            }
        } catch (error) {
            console.error('[Ingestion Error]:', error);
        } finally {
            setWorkspace(prev => ({ ...prev, is_indexing: false }));
        }
    };

    const handleAutowrite = () => {
        window.dispatchEvent(new CustomEvent('plato-action-autowrite'));
    };

    const handleStopAutowrite = () => {
        window.dispatchEvent(new CustomEvent('plato-action-stop-autowrite'));
    };

    const handleLinkMention = () => {
        window.dispatchEvent(new CustomEvent('plato-action-link'));
    };

    return (
        <div className="flex items-center justify-between px-4 py-2 bg-slate-100 dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 text-sm h-12 flex-shrink-0">
            <div className="flex items-center">
                <div className="font-semibold text-slate-700 dark:text-slate-300 mr-6">Plato IDE</div>
                
                <div className="flex items-center gap-4">
                    <button 
                        onClick={handleSelectWorkspace}
                        className="flex items-center gap-1.5 text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors cursor-pointer"
                    >
                        <FolderOpen size={14} /> 
                        <span>{workspace.active_directory_path ? 'Change Vault' : 'Open Vault'}</span>
                    </button>

                    {workspace.active_directory_path && (
                        <div className="flex items-center gap-4 border-l border-slate-300 dark:border-slate-700 pl-4">
                            <span className="text-slate-500 dark:text-slate-400 text-xs">
                                {workspace.indexed_file_count} files linked
                            </span>
                            <button 
                                onClick={handleSyncDatabase}
                                disabled={workspace.is_indexing}
                                className={`flex items-center gap-1.5 transition-colors cursor-pointer ${workspace.is_indexing ? 'text-indigo-400 cursor-wait' : 'text-indigo-600 hover:text-indigo-800 dark:text-indigo-400 dark:hover:text-indigo-300'}`}
                            >
                                <RefreshCw size={14} className={workspace.is_indexing ? "animate-spin" : ""} />
                                <span>{workspace.is_indexing ? "Syncing Matrix..." : "Sync DB"}</span>
                            </button>
                        </div>
                    )}
                </div>
            </div>

            {/* Editor Action Buttons */}
            <div className="flex items-center gap-3">
                <button
                    onClick={handleLinkMention}
                    onMouseDown={(e) => e.preventDefault()} // CRITICAL: Prevents focus stealing from the editor
                    disabled={!workspace.active_directory_path}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                    title="Link Lorebook Entity (@)"
                >
                    <AtSign size={14} />
                    <span className="font-medium hidden sm:inline">Link Entity</span>
                </button>
                
                {isAutowriting ? (
                    <button
                        onClick={handleStopAutowrite}
                        onMouseDown={(e) => e.preventDefault()}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-red-600 dark:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors cursor-pointer border border-red-200 dark:border-red-900 shadow-sm animate-pulse"
                        title="Stop Generation (Esc)"
                    >
                        <Square size={14} fill="currentColor" />
                        <span className="font-medium hidden sm:inline">Stop</span>
                    </button>
                ) : (
                    <button
                        onClick={handleAutowrite}
                        onMouseDown={(e) => e.preventDefault()}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-amber-600 dark:text-amber-500 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors cursor-pointer border border-amber-200 dark:border-amber-900 shadow-sm"
                        title="Plato Autowrite (Ctrl+J)"
                    >
                        <Wand2 size={14} />
                        <span className="font-medium hidden sm:inline">Autowrite</span>
                    </button>
                )}
            </div>
        </div>
    );
};