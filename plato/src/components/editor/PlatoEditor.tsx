import React, { useMemo } from 'react';
import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary';
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { PaginatedCanvasWrapper } from './plugins/PaginatedCanvasWrapper';
import { AIGeneratedNode } from './nodes/AIGeneratedNode';
import { MentionNode } from './nodes/MentionNode';
import AICompletionPlugin from './plugins/AICompletionPlugin';
import { LorebookMentionPlugin } from './plugins/LorebookMentionPlugin';
import { theme as editorTheme } from './theme';

export const PlatoEditor: React.FC = () => {
    const initialConfig = useMemo(() => ({
        namespace: 'PlatoEditor',
        theme: editorTheme,
        onError: (error: Error) => {
            console.error('[Lexical Error]:', error);
        },
        nodes: [AIGeneratedNode, MentionNode],
    }), []);

    return (
        <LexicalComposer initialConfig={initialConfig}>
            <div className="relative h-full w-full bg-slate-50 dark:bg-slate-950 overflow-hidden">
                <PaginatedCanvasWrapper>
                    <RichTextPlugin
                        contentEditable={
                            <ContentEditable 
                                className="min-h-[1056px] w-[816px] py-[96px] px-[96px] outline-none text-slate-900 dark:text-slate-100 leading-relaxed" 
                            />
                        }
                        placeholder={
                            <div className="absolute top-[96px] left-[96px] text-slate-400 pointer-events-none select-none italic">
                                Start weaving your story...
                            </div>
                        }
                        ErrorBoundary={LexicalErrorBoundary}
                    />
                    <HistoryPlugin />
                    <AICompletionPlugin />
                    <LorebookMentionPlugin />
                </PaginatedCanvasWrapper>
            </div>
        </LexicalComposer>
    );
};