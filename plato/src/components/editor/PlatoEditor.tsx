import React from 'react';
import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin';
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary';

import { platoEditorTheme } from './theme';
import { AIGeneratedNode } from './nodes/AIGeneratedNode';
import { PaginatedCanvasWrapper } from './plugins/PaginatedCanvasWrapper';

/**
 * PlatoEditor Composition Root
 * Initializes the Lexical environment and mounts the necessary plugins.
 * Exposes a clean React component to the main application shell.
 */
export const PlatoEditor: React.FC = () => {
    const initialConfig = {
        namespace: 'PlatoEditor',
        theme: platoEditorTheme,
        // All custom nodes must be explicitly registered here before they can be instantiated
        nodes: [
            AIGeneratedNode
        ],
        onError: (error: Error) => {
            console.error('Lexical Engine Fault:', error);
        },
    };

    return (
        <LexicalComposer initialConfig={initialConfig}>
            <div className="relative flex-grow h-full overflow-hidden">
                <PaginatedCanvasWrapper>
                    <RichTextPlugin
                        contentEditable={
                            <ContentEditable 
                                className="outline-none min-h-full prose dark:prose-invert max-w-none text-gray-900 dark:text-gray-100" 
                            />
                        }
                        placeholder={
                            <div className="absolute top-20 left-16 text-gray-400 pointer-events-none select-none">
                                Begin drafting...
                            </div>
                        }
                        ErrorBoundary={LexicalErrorBoundary}
                    />
                </PaginatedCanvasWrapper>
                
                {/* Standard plugins for professional editor behavior */}
                <HistoryPlugin />
            </div>
        </LexicalComposer>
    );
};