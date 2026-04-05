import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getSelection, $isRangeSelection, COMMAND_PRIORITY_LOW, createCommand, LexicalCommand } from 'lexical';
import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export const TRIGGER_AI_GEN_COMMAND: LexicalCommand<void> = createCommand();

export default function AICompletionPlugin(): null {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        let unlistenFn: UnlistenFn | undefined;

        const setupListener = async () => {
            unlistenFn = await listen<{ message_id: string; token: string; is_final: boolean }>(
                'chat_token',
                (event) => {
                    const { message_id, token } = event.payload;
                    if (!message_id.startsWith('inline-')) return;

                    editor.update(() => {
                        const selection = $getSelection();
                        if ($isRangeSelection(selection)) {
                            selection.insertText(token);
                        }
                    });
                }
            );
        };

        setupListener();

        const unregisterCommand = editor.registerCommand(
            TRIGGER_AI_GEN_COMMAND,
            () => {
                const selection = $getSelection();
                if ($isRangeSelection(selection)) {
                    const textContent = selection.anchor.getNode().getTextContent();
                    const messageId = `inline-${Date.now()}`;
                    
                    invoke('stream_chat_completion', {
                        messageId,
                        history: [{ role: 'user', content: `Continue this story: ${textContent}` }],
                        config: { 
                            temperature: 0.8, 
                            top_p: 0.9, 
                            min_keep: 1, 
                            top_k: 40, 
                            repeat_penalty: 1.1, 
                            repeat_last_n: 64 
                        }
                    });
                }
                return true;
            },
            COMMAND_PRIORITY_LOW
        );

        const onKeyDown = (event: KeyboardEvent) => {
            if ((event.ctrlKey || event.metaKey) && event.key === 'j') {
                event.preventDefault();
                editor.dispatchCommand(TRIGGER_AI_GEN_COMMAND, undefined);
            }
        };

        window.addEventListener('keydown', onKeyDown);

        return () => {
            if (unlistenFn) unlistenFn();
            unregisterCommand();
            window.removeEventListener('keydown', onKeyDown);
        };
    }, [editor]);

    return null;
}