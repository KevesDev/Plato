import React, { ReactNode } from 'react';

interface MainLayoutProps {
    editor: ReactNode;
    sidebar: ReactNode;
}

/**
 * MainLayout
 * Establishes the primary application grid. Strictly separates the writing environment
 * from the Copilot interface to ensure independent scrolling and state management.
 */
export const MainLayout: React.FC<MainLayoutProps> = ({ editor, sidebar }) => {
    return (
        <div className="flex w-screen h-screen overflow-hidden bg-gray-50 dark:bg-gray-950">
            <main className="flex-grow flex flex-col h-full overflow-hidden">
                {editor}
            </main>
            <aside className="w-[400px] flex-shrink-0 h-full border-l border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xl z-10">
                {sidebar}
            </aside>
        </div>
    );
};