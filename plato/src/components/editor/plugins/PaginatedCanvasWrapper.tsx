import React, { ReactNode } from 'react';

interface PaginatedCanvasWrapperProps {
    children: ReactNode;
}

/**
 * PaginatedCanvasWrapper
 * Enforces a strict physical page boundary for the editor canvas.
 * This abstracts layout styling away from the Lexical internal engine,
 * ensuring the text renderer is not responsible for document pagination visual logic.
 */
export const PaginatedCanvasWrapper: React.FC<PaginatedCanvasWrapperProps> = ({ children }) => {
    return (
        <div 
            className="flex justify-center w-full min-h-screen bg-gray-100 dark:bg-gray-900 py-12 overflow-y-auto"
            style={{ 
                /* * Inline styles used exclusively for absolute dimensional constraints
                 * standardizing the document to an 8.5x11 aspect ratio width equivalent.
                 */
                containerType: 'inline-size' 
            }}
        >
            <div 
                className="w-full max-w-[816px] min-h-[1056px] bg-white dark:bg-gray-800 shadow-lg ring-1 ring-gray-200 dark:ring-gray-700 px-16 py-20"
                style={{
                    /* Ensures the typing cursor remains visible over standard page boundaries */
                    boxSizing: 'border-box'
                }}
            >
                {children}
            </div>
        </div>
    );
};