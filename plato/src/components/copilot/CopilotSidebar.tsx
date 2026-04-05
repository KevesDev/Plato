import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { FolderOpen, Square, Send, FileText, DatabaseZap } from 'lucide-react';

interface Message {
    id: string;
    role: 'user' | 'ai';
    content: string;
}

interface WorkspaceState {
    active_directory_path: string | null;
    indexed_file_count: number;
    is_indexing: boolean;
}

/**
 * Production-grade Copilot Sidebar.
 * Manages conversation history, real-time token streaming, and workspace directory mapping.
 */
export const CopilotSidebar: React.FC = () => {
    const [input, setInput] = useState('');
    const [isStreaming, setIsStreaming] = useState(false);
    const [messages, setMessages] = useState<Message[]>([
        { id: 'sys-init', role: 'ai', content: 'Plato AGI online. Ready to build your world.' }
    ]);
    const [workspace, setWorkspace] = useState<WorkspaceState>({
        active_directory_path: null,
        indexed_file_count: 0,
        is_indexing: false
    });
    const listenerBound = useRef(false);

    useEffect(() => {
        let unlistenFn: UnlistenFn | undefined;
        const setup = async () => {
            if (listenerBound.current) return;
            listenerBound.current = true;
            unlistenFn = await listen<{message_id: string, token: string, is_final: boolean}>('chat_token', (event) => {
                const { message_id, token, is_final } = event.payload;
                setMessages(prev => {
                    const idx = prev.findIndex(m => m.id === message_id);
                    if (idx >= 0) {
                        const updated = [...prev];
                        updated[idx].content += token;
                        return updated;
                    } else {
                        return [...prev, { id: message_id, role: 'ai', content: token }];
                    }
                });
                if (is_final) setIsStreaming(false);
            });
        };
        setup();
        return () => { if (unlistenFn) unlistenFn(); listenerBound.current = false; };
    }, []);

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
                setMessages(prev => [...prev, { 
                    id: `sys-${Date.now()}`, 
                    role: 'ai', 
                    // Elevated the success message to match the sentient AGI persona
                    content: `Memory synchronization complete. I have woven ${response.data} passages from your vault into my consciousness. I am ready.` 
                }]);
            }
        } catch (error) {
            console.error('[Ingestion Error]:', error);
        } finally {
            setWorkspace(prev => ({ ...prev, is_indexing: false }));
        }
    };

    const handleStop = async () => {
        await invoke('abort_inference');
        setIsStreaming(false);
    };

    const handleSend = async () => {
        if (!input.trim() || isStreaming) return;
        
        const userText = input.trim();
        const userMessage: Message = { id: `u-${Date.now()}`, role: 'user', content: userText };
        const responseId = `ai-${Date.now()}`;
        
        /**
         * Assembles the chronological conversation context by appending the 
         * current user prompt prior to dispatching to the inference engine.
         */
        const currentMessages = [...messages, userMessage];
        const history = currentMessages.map(m => ({ 
            role: m.role === 'ai' ? 'assistant' : 'user', 
            content: m.content 
        }));
        
        setMessages(currentMessages);
        setInput('');
        setIsStreaming(true);
        
        try {
            const config = { temperature: 0.7, top_p: 0.9, min_keep: 1, top_k: 40, repeat_penalty: 1.2, repeat_last_n: 64 };
            await invoke('stream_chat_completion', { messageId: responseId, history, config });
        } catch (error) {
            console.error('[Plato IPC Error]:', error);
            setIsStreaming(false);
        }
    };

    return (
        <div className="flex flex-col h-full bg-white dark:bg-gray-900 border-l border-slate-200 dark:border-slate-800 font-sans">
            <div className="p-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-slate-50 dark:bg-slate-900">
                <h2 className="font-semibold text-slate-800 dark:text-slate-200">Plato Copilot</h2>
                
                <div className="flex items-center gap-2">
                    {workspace.active_directory_path && (
                        <button 
                            onClick={handleSyncDatabase}
                            disabled={workspace.is_indexing}
                            className={`flex items-center gap-2 text-xs px-3 py-1.5 rounded-md border transition-colors shadow-sm ${workspace.is_indexing ? 'bg-indigo-50 text-indigo-400 border-indigo-200 cursor-wait' : 'bg-indigo-50 text-indigo-600 border-indigo-200 hover:bg-indigo-100 cursor-pointer'}`}
                            title="Sync Lore to Memory Matrix"
                        >
                            <DatabaseZap size={14} className={workspace.is_indexing ? "animate-pulse" : ""} />
                            <span className="font-medium">{workspace.is_indexing ? "Syncing..." : "Sync DB"}</span>
                        </button>
                    )}
                    <button 
                        onClick={handleSelectWorkspace}
                        className="flex items-center gap-2 text-xs text-blue-600 bg-blue-50 px-3 py-1.5 rounded-md border border-blue-200 hover:bg-blue-100 transition-colors shadow-sm cursor-pointer"
                        title={workspace.active_directory_path || "Select your Story Vault"}
                    >
                        <FolderOpen size={14} /> 
                        <span className="font-medium">{workspace.indexed_file_count} Files</span>
                    </button>
                </div>
            </div>
            <div className="flex-grow overflow-y-auto p-4 space-y-4">
                {messages.map(msg => (
                    <div key={msg.id} className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                        <div className={`max-w-[90%] p-3 rounded-lg text-sm shadow-sm ${msg.role === 'user' ? 'bg-blue-600 text-white shadow-blue-200' : 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700'}`}>
                            {msg.content}
                            {msg.role === 'ai' && isStreaming && msg.id === messages[messages.length - 1]?.id && (
                                <span className="inline-block w-2 h-4 ml-1 bg-slate-400 animate-pulse align-middle" />
                            )}
                        </div>
                    </div>
                ))}
            </div>
            <div className="p-4 border-t border-gray-200 dark:border-gray-800 bg-slate-50 dark:bg-slate-900">
                <div className="relative">
                    <textarea 
                        className="w-full pl-3 pr-12 py-3 bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-lg text-sm h-[80px] resize-none outline-none focus:ring-1 focus:ring-blue-500 text-slate-900 dark:text-slate-100"
                        placeholder="Message Plato..."
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
                    />
                    <div className="absolute right-2 bottom-2 flex flex-col gap-2">
                        {isStreaming ? (
                            <button onClick={handleStop} className="p-2 text-red-500 hover:bg-red-50 rounded-full transition-colors">
                                <Square size={18} fill="currentColor" />
                            </button>
                        ) : (
                            <button onClick={handleSend} disabled={!input.trim()} className="p-2 text-blue-600 hover:bg-blue-50 disabled:text-slate-300">
                                <Send size={18} />
                            </button>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};