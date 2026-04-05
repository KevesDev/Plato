import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MainLayout } from './components/layout/MainLayout';
import { PlatoEditor } from './components/editor/PlatoEditor';
import { CopilotSidebar } from './components/copilot/CopilotSidebar';
import { OnboardingScreen } from './components/onboarding/OnboardingScreen';
import { IpcResponse } from './types/ipc.types';
import './App.css'; 

/**
 * Plato Application Root
 * Orchestrates the application lifecycle by verifying engine readiness
 * and establishing the C++ memory lock before mounting the primary interfaces.
 */
function App() {
    const [isEngineReady, setIsEngineReady] = useState<boolean>(false);
    const [isChecking, setIsChecking] = useState<boolean>(true);
    const [engineError, setEngineError] = useState<string | null>(null);

    useEffect(() => {
        /**
         * Performs a non-blocking check for local model assets on application boot.
         * If assets exist, it immediately attempts to load the engine into memory.
         */
        const checkStatusAndBoot = async () => {
            try {
                const response = await invoke<IpcResponse<boolean>>('check_model_status');
                if (response.success && response.data) {
                    // Files exist. Now load the inference engine into RAM.
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

    /**
     * Triggered by the OnboardingScreen once the download completes 100%.
     * We must wait for the Rust thread to allocate the memory before removing the loading UI.
     */
    const handleOnboardingComplete = async () => {
        try {
            await invoke('initialize_engine');
            setIsEngineReady(true);
        } catch (error) {
            console.error("Failed to initialize engine after download:", error);
            setEngineError(String(error));
        }
    };

    // Show a blank/loading state while the integrity check is running
    if (isChecking) {
        return <div className="w-screen h-screen bg-slate-50 dark:bg-slate-950" />;
    }

    // Critical memory failure state
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

    // Force onboarding if the model integrity check fails
    if (!isEngineReady) {
        return <OnboardingScreen onComplete={handleOnboardingComplete} />;
    }

    return (
        <MainLayout 
            editor={<PlatoEditor />} 
            sidebar={<CopilotSidebar />} 
        />
    );
}

export default App;