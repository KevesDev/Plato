import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { FileText } from 'lucide-react';

interface Message {
    id: string;
    role: 'user' | 'ai';
    content: string;
}

interface ChatTokenEvent {
    message_id: string;
    token: string;
    is_final: boolean;
}

export const CopilotSidebar: React.FC = () => {
    const [input, setInput] = useState('');
    const [isStreaming, setIsStreaming] = useState(false);
    const [messages, setMessages] = useState<Message[]>([
        { id: 'system-1', role: 'ai', content: 'Plato Engine initialized. Ready for prompt.' }
    ]);

    useEffect(() => {
        const unlisten = listen<ChatTokenEvent>('chat_token', (event) => {
            const { message_id, token, is_final } = event.payload;
            setMessages(prev => {
                const existingIndex = prev.findIndex(m => m.id === message_id);
                if (existingIndex >= 0) {
                    const updated = [...prev];
                    updated[existingIndex].content += token;
                    return updated;
                } else {
                    return [...prev, { id: message_id, role: 'ai', content: token }];
                }
            });
            if (is_final) setIsStreaming(false);
        });
        return () => { unlisten.then(f => f()); };
    }, []);

    const handleSend = async () => {
        if (!input.trim() || isStreaming) return;
        const prompt = input.trim();
        const messageId = Date.now().toString();
        const responseId = `ai-${messageId}`;
        setMessages(prev => [...prev, { id: messageId, role: 'user', content: prompt }]);
        setInput('');
        setIsStreaming(true);
        try {
            const config = { temperature: 0.7, top_p: 0.9, min_keep: 1, top_k: 40, repeat_penalty: 1.1, repeat_last_n: 64 };
            await invoke('stream_chat_completion', { messageId: responseId, prompt, config });
        } catch (error) {
            setMessages(prev => [...prev, { id: `err-${Date.now()}`, role: 'ai', content: `[Engine Failure]: ${error}` }]);
            setIsStreaming(false);
        }
    };

    return (
        <div className="flex flex-col h-full bg-white dark:bg-gray-900 border-l border-slate-200 dark:border-slate-800">
            <div className="p-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-slate-50 dark:bg-slate-900">
                <h2 className="font-semibold text-slate-800 dark:text-slate-200">Plato Copilot</h2>
                <div className="flex items-center gap-2 text-xs text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/30 px-2 py-1 rounded border border-blue-100 dark:border-blue-800">
                    <FileText size={14} />
                    <span>0 Files Indexed</span>
                </div>
            </div>
            <div className="flex-grow overflow-y-auto p-4 space-y-4">
                {messages.map(msg => (
                    <div key={msg.id} className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                        <div className={`max-w-[85%] p-3 rounded-lg text-sm shadow-sm whitespace-pre-wrap ${msg.role === 'user' ? 'bg-blue-600 text-white' : 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700'}`}>
                            {msg.content}
                            {msg.role === 'ai' && isStreaming && msg.id === messages[messages.length - 1]?.id && (
                                <span className="inline-block w-2 h-4 ml-1 bg-slate-400 animate-pulse align-middle" />
                            )}
                        </div>
                    </div>
                ))}
            </div>
            <div className="p-4 border-t border-gray-200 dark:border-gray-800 bg-slate-50 dark:bg-slate-900">
                <div className="relative flex items-center">
                    <textarea 
                        className="w-full pl-3 pr-10 py-3 bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none h-[60px] text-slate-900 dark:text-slate-100"
                        placeholder={isStreaming ? "Plato is drafting..." : "Ask about the lore..."}
                        value={input}
                        disabled={isStreaming}
                        onChange={(e) => setInput(e.target.value)}
                        onKeyDown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSend(); } }}
                    />
                    <button onClick={handleSend} disabled={isStreaming || !input.trim()} className={`absolute right-2 p-2 transition-colors ${isStreaming || !input.trim() ? 'text-slate-300 dark:text-slate-700' : 'text-slate-400 hover:text-blue-600 dark:hover:text-blue-400'}`}>
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="22" y1="2" x2="11" y2="13"></line><polygon points="22 2 15 22 11 13 2 9 22 2"></polygon></svg>
                    </button>
                </div>
            </div>
        </div>
    );
};