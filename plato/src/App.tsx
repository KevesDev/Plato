import React from 'react';
import { MainLayout } from './components/layout/MainLayout';
import { PlatoEditor } from './components/editor/PlatoEditor';
import { CopilotSidebar } from './components/copilot/CopilotSidebar';
import './App.css'; // Tailwind directives should be located here

function App() {
  return (
    <MainLayout 
        editor={<PlatoEditor />} 
        sidebar={<CopilotSidebar />} 
    />
  );
}

export default App;