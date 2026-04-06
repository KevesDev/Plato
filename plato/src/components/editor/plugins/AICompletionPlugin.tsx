import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getSelection, $isRangeSelection, $getRoot, $nodesOfType, COMMAND_PRIORITY_LOW, createCommand, LexicalCommand } from 'lexical';
import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { MentionNode } from '../nodes/MentionNode';

export const TRIGGER_AI_GEN_COMMAND: LexicalCommand<void> = createCommand();
export const STOP_AI_GEN_COMMAND: LexicalCommand<void> = createCommand();

export default function AICompletionPlugin(): null {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        let unlistenFn: UnlistenFn | undefined;

        const setupListener = async () => {
            unlistenFn = await listen<{ message_id: string; token: string; is_final: boolean }>(
                'chat_token',
                (event) => {
                    const { message_id, token, is_final } = event.payload;
                    if (!message_id.startsWith('inline-')) return;

                    editor.update(() => {
                        const selection = $getSelection();
                        if ($isRangeSelection(selection)) {
                            selection.insertText(token);
                        }
                    });

                    if (is_final) {
                        window.dispatchEvent(new CustomEvent('plato-autowrite-state', { detail: false }));
                    }
                }
            );
        };

        setupListener();

        const unregisterTrigger = editor.registerCommand(
            TRIGGER_AI_GEN_COMMAND,
            () => {
                window.dispatchEvent(new CustomEvent('plato-autowrite-state', { detail: true }));
                
                let storyContext = "";
                let mentionedEntities: string[] = [];

                editor.getEditorState().read(() => {
                    const rawText = $getRoot().getTextContent();
                    storyContext = rawText.replace(/@/g, '');

                    const mentionNodes = $nodesOfType(MentionNode);
                    mentionedEntities = Array.from(new Set(mentionNodes.map(n => n.__mention)));
                });

                const messageId = `inline-${Date.now()}`;
                
                invoke('stream_chat_completion', {
                    messageId,
                    history: [
                        { 
                            role: 'system', 
                            content: 'You are an expert creative writing assistant. Your task is to continue the narrative seamlessly from where the user left off. CRITICAL RULES:\n1. Maintain impeccable grammar, punctuation, and proper capitalization.\n2. Do not include any meta-commentary, greetings, or introductory phrases. Output ONLY the story continuation.\n3. Do NOT use the "@" symbol to tag characters or locations. Write their names normally in standard prose.' 
                        },
                        { 
                            role: 'user', 
                            content: `Here is the story so far:\n\n${storyContext}\n\nContinue writing the story directly from here:` 
                        }
                    ],
                    config: { 
                        temperature: 0.8, 
                        top_p: 0.9, 
                        min_keep: 1, 
                        top_k: 40, 
                        repeat_penalty: 1.1, 
                        repeat_last_n: 64 
                    },
                    // CRITICAL FIX: Rust demands snake_case for IPC deserialization
                    context_entities: mentionedEntities.length > 0 ? mentionedEntities : null
                }).catch((err) => {
                    console.error("[Autowrite Error]:", err);
                    window.dispatchEvent(new CustomEvent('plato-autowrite-state', { detail: false }));
                });
                
                return true;
            },
            COMMAND_PRIORITY_LOW
        );

        const unregisterStop = editor.registerCommand(
            STOP_AI_GEN_COMMAND,
            () => {
                invoke('abort_inference');
                window.dispatchEvent(new CustomEvent('plato-autowrite-state', { detail: false }));
                return true;
            },
            COMMAND_PRIORITY_LOW
        );

        const onKeyDown = (event: KeyboardEvent) => {
            if ((event.ctrlKey || event.metaKey) && event.key === 'j') {
                event.preventDefault();
                editor.dispatchCommand(TRIGGER_AI_GEN_COMMAND, undefined);
            }
            if (event.key === 'Escape') {
                editor.dispatchCommand(STOP_AI_GEN_COMMAND, undefined);
            }
        };

        const handleAutowriteAction = () => {
            editor.dispatchCommand(TRIGGER_AI_GEN_COMMAND, undefined);
        };

        const handleStopAction = () => {
            editor.dispatchCommand(STOP_AI_GEN_COMMAND, undefined);
        };

        window.addEventListener('keydown', onKeyDown);
        window.addEventListener('plato-action-autowrite', handleAutowriteAction);
        window.addEventListener('plato-action-stop-autowrite', handleStopAction);

        return () => {
            if (unlistenFn) unlistenFn();
            unregisterTrigger();
            unregisterStop();
            window.removeEventListener('keydown', onKeyDown);
            window.removeEventListener('plato-action-autowrite', handleAutowriteAction);
            window.removeEventListener('plato-action-stop-autowrite', handleStopAction);
        };
    }, [editor]);

    return null;
}