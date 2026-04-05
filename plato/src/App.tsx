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
 * before mounting the primary writer interface.
 */
function App() {
    const [isEngineReady, setIsEngineReady] = useState<boolean>(false);
    const [isChecking, setIsChecking] = useState<boolean>(true);

    useEffect(() => {
        /**
         * Performs a non-blocking check for local model assets.
         * If assets are missing, the UI remains in Onboarding state.
         */
        const checkStatus = async () => {
            try {
                const response = await invoke<IpcResponse<boolean>>('check_model_status');
                if (response.success && response.data) {
                    setIsEngineReady(true);
                }
            } catch (error) {
                console.error("Critical Engine Failure:", error);
            } finally {
                setIsChecking(false);
            }
        };

        checkStatus();
    }, []);

    // Show a blank or loading state while the integrity check is running
    if (isChecking) {
        return <div className="w-screen h-screen bg-slate-50 dark:bg-slate-950" />;
    }

    // Force onboarding if the model integrity check fails
    if (!isEngineReady) {
        return <OnboardingScreen onComplete={() => setIsEngineReady(true)} />;
    }

    return (
        <MainLayout 
            editor={<PlatoEditor />} 
            sidebar={<CopilotSidebar />} 
        />
    );
}

export default App;