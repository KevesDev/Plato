import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { FileText, Square, Send } from 'lucide-react';

interface Message {
    id: string;
    role: 'user' | 'ai';
    content: string;
}

export const CopilotSidebar: React.FC = () => {
    const [input, setInput] = useState('');
    const [isStreaming, setIsStreaming] = useState(false);
    const [messages, setMessages] = useState<Message[]>([
        { id: 'sys-1', role: 'ai', content: 'Plato AGI initialized. I am ready to help you write.' }
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
        return () => { if (unlistenFn) unlistenFn(); listenerBound.current = false; };
    }, []);

    const handleStop = async () => {
        await invoke('abort_inference');
        setIsStreaming(false);
    };

    const handleSend = async () => {
        if (!input.trim() || isStreaming) return;
        const userText = input.trim();
        const responseId = `ai-${Date.now()}`;
        
        // Build history to send to backend for memory
        const history = messages.map(m => ({ role: m.role === 'ai' ? 'assistant' : 'user', content: m.content }));
        
        setMessages(prev => [...prev, { id: `u-${Date.now()}`, role: 'user', content: userText }]);
        setInput('');
        setIsStreaming(true);
        
        try {
            const config = { temperature: 0.7, top_p: 0.9, min_keep: 1, top_k: 40, repeat_penalty: 1.15, repeat_last_n: 64 };
            await invoke('stream_chat_completion', { message_id: responseId, history, config });
        } catch (error) {
            setIsStreaming(false);
        }
    };

    return (
        <div className="flex flex-col h-full bg-white dark:bg-gray-900 border-l border-slate-200 dark:border-slate-800">
            <div className="p-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between">
                <h2 className="font-semibold text-slate-800 dark:text-slate-200">Plato Copilot</h2>
                <div className="flex items-center gap-2 text-xs text-blue-600 bg-blue-50 px-2 py-1 rounded border border-blue-100">
                    <FileText size={14} /> <span>{messages.length} Turns</span>
                </div>
            </div>
            <div className="flex-grow overflow-y-auto p-4 space-y-4">
                {messages.map(msg => (
                    <div key={msg.id} className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                        <div className={`max-w-[90%] p-3 rounded-lg text-sm ${msg.role === 'user' ? 'bg-blue-600 text-white' : 'bg-slate-100 text-slate-800 border'}`}>
                            {msg.content}
                        </div>
                    </div>
                ))}
            </div>
            <div className="p-4 border-t border-gray-200 bg-slate-50">
                <div className="relative">
                    <textarea 
                        className="w-full pl-3 pr-12 py-3 bg-white border border-slate-300 rounded-lg text-sm h-[80px] resize-none"
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