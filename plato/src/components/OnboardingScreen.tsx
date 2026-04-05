import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { BrainCircuit } from 'lucide-react';

interface DownloadProgressEvent {
    part_current: number;
    part_total: number;
    downloaded_bytes: number;
    total_bytes: number;
}

/**
 * OnboardingScreen
 * A full-screen blocking overlay that ensures all AI model assets are present locally
 * before allowing access to the editor. Handles multi-part dynamic progress tracking.
 */
export const OnboardingScreen: React.FC = () => {
    const [progress, setProgress] = useState<DownloadProgressEvent | null>(null);

    useEffect(() => {
        // Establishes the IPC event listener for the streaming byte chunks from Rust
        const unlisten = listen<DownloadProgressEvent>('download_progress', (event) => {
            setProgress(event.payload);
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    // Calculate percentage, guarding against division by zero
    const percentComplete = progress && progress.total_bytes > 0 
        ? Math.min(100, Math.round((progress.downloaded_bytes / progress.total_bytes) * 100))
        : 0;

    // Convert bytes to human-readable GB for the specific chunk
    const downloadedGb = progress ? (progress.downloaded_bytes / 1e9).toFixed(2) : "0.00";
    const totalGb = progress ? (progress.total_bytes / 1e9).toFixed(2) : "0.00";

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-gray-50 dark:bg-gray-950 w-screen h-screen">
            <div className="max-w-md w-full p-8 bg-white dark:bg-gray-900 shadow-2xl rounded-2xl border border-gray-200 dark:border-gray-800 flex flex-col items-center">
                
                <div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-500 rounded-full flex items-center justify-center mb-6">
                    <BrainCircuit size={32} />
                </div>
                
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2 text-center">
                    Initializing Plato
                </h1>
                
                <p className="text-sm text-gray-500 dark:text-gray-400 text-center mb-8">
                    Preparing the local intelligence engine. This happens once.
                </p>

                {/* Progress Tracking UI */}
                {progress ? (
                    <div className="w-full">
                        <div className="flex justify-between text-xs font-semibold text-gray-700 dark:text-gray-300 mb-2">
                            <span>Downloading Part {progress.part_current} of {progress.part_total}</span>
                            <span>{percentComplete}%</span>
                        </div>
                        
                        <div className="w-full h-3 bg-gray-200 dark:bg-gray-800 rounded-full overflow-hidden mb-2">
                            <div 
                                className="h-full bg-blue-600 transition-all duration-300 ease-out rounded-full"
                                style={{ width: `${percentComplete}%` }}
                            />
                        </div>

                        <div className="flex justify-end text-xs text-gray-500 dark:text-gray-400">
                            {downloadedGb} GB / {totalGb} GB
                        </div>
                    </div>
                ) : (
                    <div className="w-full flex justify-center">
                        <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />
                    </div>
                )}
            </div>
        </div>
    );
};