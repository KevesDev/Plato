import React, { ReactNode } from 'react';

interface MainLayoutProps {
    toolbar: ReactNode;
    editor: ReactNode;
    sidebar: ReactNode;
}

export const MainLayout: React.FC<MainLayoutProps> = ({ toolbar, editor, sidebar }) => {
    return (
        <div className="flex flex-col w-screen h-screen overflow-hidden bg-gray-50 dark:bg-gray-950">
            {toolbar}
            <div className="flex-grow flex overflow-hidden">
                <main className="flex-grow flex flex-col h-full overflow-hidden">
                    {editor}
                </main>
                <aside className="w-[400px] flex-shrink-0 h-full border-l border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xl z-10">
                    {sidebar}
                </aside>
            </div>
        </div>
    );
};