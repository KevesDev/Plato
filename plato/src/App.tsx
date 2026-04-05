import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MainLayout } from './components/layout/MainLayout';
import { PlatoEditor } from './components/editor/PlatoEditor';
import { CopilotSidebar } from './components/copilot/CopilotSidebar';
import { OnboardingScreen } from './components/onboarding/OnboardingScreen';
import './App.css'; 

/**
 * Plato Application Root
 * Orchestrates the application lifecycle by verifying engine readiness
 * before mounting the primary writer interface.
 */
function App() {
    const [isEngineReady, setIsEngineReady] = useState<boolean>(false);

    useEffect(() => {
        // Invokes the Rust verification and download sequence upon application mount
        invoke('verify_and_download_model')
            .then(() => {
                setIsEngineReady(true);
            })
            .catch((error) => {
                console.error("Critical Engine Failure:", error);
            });
    }, []);

    // Block access to the main application until assets are verified
    if (!isEngineReady) {
        return <OnboardingScreen />;
    }

    return (
        <MainLayout 
            editor={<PlatoEditor />} 
            sidebar={<CopilotSidebar />} 
        />
    );
}

export default App;