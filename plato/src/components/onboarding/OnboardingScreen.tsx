import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { BrainCircuit, AlertTriangle, RefreshCcw } from 'lucide-react';
import { DownloadProgressEvent } from '../../types/ipc.types';

interface OnboardingProps {
    onComplete: () => void;
}

export const OnboardingScreen: React.FC<OnboardingProps> = ({ onComplete }) => {
    const [progress, setProgress] = useState<DownloadProgressEvent | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const unlisten = listen<DownloadProgressEvent>('download_progress', (event) => {
            setProgress(event.payload);
        });

        invoke('start_model_download')
            .then(() => onComplete())
            .catch((err) => {
                console.error("Initialization Failed:", err);
                setError(err.toString());
            });

        return () => {
            unlisten.then(f => f());
        };
    }, [onComplete]);

    const percentComplete = progress && progress.total_bytes > 0 
        ? Math.min(100, Math.round((progress.downloaded_bytes / progress.total_bytes) * 100))
        : 0;

    const downloadedGb = progress ? (progress.downloaded_bytes / 1e9).toFixed(2) : "0.00";
    const totalGb = progress ? (progress.total_bytes / 1e9).toFixed(2) : "0.00";

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-50 dark:bg-slate-950 w-screen h-screen">
            <div className="max-w-md w-full p-8 bg-white dark:bg-slate-900 shadow-2xl rounded-2xl border border-slate-200 dark:border-slate-800 flex flex-col items-center">
                
                {error ? (
                    <div className="w-full flex flex-col items-center animate-in fade-in zoom-in duration-300">
                        <div className="w-16 h-16 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded-full flex items-center justify-center mb-6">
                            <AlertTriangle size={32} />
                        </div>
                        <h1 className="text-2xl font-bold text-slate-900 dark:text-white mb-2 text-center">Connection Error</h1>
                        <p className="text-sm text-slate-500 dark:text-slate-400 text-center mb-8 px-4">
                            {error}
                        </p>
                        <button 
                            onClick={() => window.location.reload()}
                            className="flex items-center gap-2 px-6 py-2.5 bg-slate-900 dark:bg-white text-white dark:text-slate-900 rounded-lg hover:opacity-90 transition-opacity font-medium"
                        >
                            <RefreshCcw size={16} />
                            Retry Initialization
                        </button>
                    </div>
                ) : (
                    <>
                        <div className="w-16 h-16 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center mb-6">
                            <BrainCircuit size={32} />
                        </div>
                        <h1 className="text-2xl font-bold text-slate-900 dark:text-white mb-2 text-center">Initializing Plato</h1>
                        <p className="text-sm text-slate-500 dark:text-slate-400 text-center mb-8">Establishing local intelligence engine.</p>
                        
                        {progress ? (
                            <div className="w-full">
                                <div className="flex justify-between text-xs font-semibold text-slate-700 dark:text-slate-300 mb-2">
                                    <span>Downloading Part {progress.part_current} of {progress.part_total}</span>
                                    <span>{percentComplete}%</span>
                                </div>
                                <div className="w-full h-2.5 bg-slate-200 dark:bg-slate-800 rounded-full overflow-hidden mb-2">
                                    <div className="h-full bg-blue-600 transition-all duration-300 ease-out" style={{ width: `${percentComplete}%` }} />
                                </div>
                                <div className="flex justify-end text-[10px] uppercase tracking-wider text-slate-500 dark:text-slate-400 font-medium">
                                    {downloadedGb} GB / {totalGb} GB
                                </div>
                            </div>
                        ) : (
                            <div className="flex flex-col items-center gap-4">
                                <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />
                                <span className="text-xs text-slate-500 animate-pulse">Requesting model chunks...</span>
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
};