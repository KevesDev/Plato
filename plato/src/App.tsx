import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MainLayout } from './components/layout/MainLayout';
import { Toolbar, WorkspaceState } from './components/layout/Toolbar';
import { PlatoEditor } from './components/editor/PlatoEditor';
import { CopilotSidebar } from './components/copilot/CopilotSidebar';
import { OnboardingScreen } from './components/onboarding/OnboardingScreen';
import { IpcResponse } from './types/ipc.types';
import './App.css'; 

function App() {
    const [isEngineReady, setIsEngineReady] = useState<boolean>(false);
    const [isChecking, setIsChecking] = useState<boolean>(true);
    const [engineError, setEngineError] = useState<string | null>(null);
    const [workspace, setWorkspace] = useState<WorkspaceState>({
        active_directory_path: null,
        indexed_file_count: 0,
        is_indexing: false
    });

    useEffect(() => {
        const checkStatusAndBoot = async () => {
            try {
                // 1. Instantly check disk for a previously saved vault
                const wsResponse = await invoke<IpcResponse<WorkspaceState>>('load_persisted_workspace');
                if (wsResponse.success && wsResponse.data) {
                    setWorkspace(wsResponse.data);
                }

                // 2. Load the AI Inference Engine into memory
                const response = await invoke<IpcResponse<boolean>>('check_model_status');
                if (response.success && response.data) {
                    await invoke('initialize_engine');
                    setIsEngineReady(true);
                }
            } catch (error) {
                console.error("Critical Engine Failure:", error);
                setEngineError(String(error));
            } finally {
                setIsChecking(false);
            }
        };

        checkStatusAndBoot();
    }, []);

    const handleOnboardingComplete = async () => {
        try {
            await invoke('initialize_engine');
            setIsEngineReady(true);
        } catch (error) {
            console.error("Failed to initialize engine after download:", error);
            setEngineError(String(error));
        }
    };

    if (isChecking) {
        return <div className="w-screen h-screen bg-slate-50 dark:bg-slate-950" />;
    }

    if (engineError) {
        return (
            <div className="w-screen h-screen flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-950 p-8">
                <div className="max-w-md w-full p-6 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-800 dark:text-red-200">
                    <h2 className="font-bold text-lg mb-2">Engine Initialization Failed</h2>
                    <p className="text-sm font-mono whitespace-pre-wrap">{engineError}</p>
                </div>
            </div>
        );
    }

    if (!isEngineReady) {
        return <OnboardingScreen onComplete={handleOnboardingComplete} />;
    }

    return (
        <MainLayout 
            toolbar={<Toolbar workspace={workspace} setWorkspace={setWorkspace} />}
            editor={<PlatoEditor />} 
            sidebar={<CopilotSidebar workspace={workspace} />} 
        />
    );
}

export default App;