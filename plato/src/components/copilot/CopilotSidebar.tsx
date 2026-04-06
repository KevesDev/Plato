import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Square, Send, MessageSquare, BookOpen } from 'lucide-react';
import { LorebookTab } from './LorebookTab';
import { WorkspaceState } from '../layout/Toolbar';

interface Message {
    id: string;
    role: 'user' | 'ai';
    content: string;
}

interface CopilotSidebarProps {
    workspace: WorkspaceState;
}

export const CopilotSidebar: React.FC<CopilotSidebarProps> = ({ workspace }) => {
    const [activeTab, setActiveTab] = useState<'chat' | 'lorebook'>('chat');
    const [input, setInput] = useState('');
    const [isStreaming, setIsStreaming] = useState(false);
    const [messages, setMessages] = useState<Message[]>([
        { id: 'sys-init', role: 'ai', content: 'Plato AGI online. Ready to build your world.' }
    ]);
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

        const handleSysMsg = (e: Event) => {
            const customEvent = e as CustomEvent;
            setMessages(prev => [...prev, { 
                id: `sys-${Date.now()}`, 
                role: 'ai', 
                content: customEvent.detail 
            }]);
        };
        window.addEventListener('plato-sys-msg', handleSysMsg);

        return () => { 
            if (unlistenFn) unlistenFn(); 
            listenerBound.current = false; 
            window.removeEventListener('plato-sys-msg', handleSysMsg);
        };
    }, []);

    const handleStop = async () => {
        await invoke('abort_inference');
        setIsStreaming(false);
    };

    const handleSend = async () => {
        if (!input.trim() || isStreaming) return;
        
        const userText = input.trim();
        const userMessage: Message = { id: `u-${Date.now()}`, role: 'user', content: userText };
        const responseId = `ai-${Date.now()}`;
        
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
            // CRITICAL FIX: Using snake_case
            await invoke('stream_chat_completion', { 
                messageId: responseId, 
                history, 
                config,
                context_entities: null
            });
        } catch (error) {
            console.error('[Plato IPC Error]:', error);
            setIsStreaming(false);
        }
    };

    return (
        <div className="flex flex-col h-full bg-white dark:bg-gray-900 border-l border-slate-200 dark:border-slate-800 font-sans">
            <div className="p-4 border-b border-gray-200 dark:border-gray-800 bg-slate-50 dark:bg-slate-900 space-y-4">
                <div className="flex items-center justify-between">
                    <h2 className="font-semibold text-slate-800 dark:text-slate-200">Plato Copilot</h2>
                </div>

                <div className="flex p-1 bg-slate-200/50 dark:bg-slate-800/50 rounded-lg">
                    <button
                        onClick={() => setActiveTab('chat')}
                        className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-colors ${activeTab === 'chat' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}`}
                    >
                        <MessageSquare size={16} /> Chat
                    </button>
                    <button
                        onClick={() => setActiveTab('lorebook')}
                        className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-sm font-medium rounded-md transition-colors ${activeTab === 'lorebook' ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-white shadow-sm' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}`}
                    >
                        <BookOpen size={16} /> Lorebook
                    </button>
                </div>
            </div>

            {activeTab === 'lorebook' ? (
                <LorebookTab workspace={workspace} />
            ) : (
                <>
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
                </>
            )}
        </div>
    );
};