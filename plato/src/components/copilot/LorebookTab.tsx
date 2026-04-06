import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DatabaseZap, CheckCircle2, FolderOpen } from 'lucide-react';
import { WorkspaceState } from '../layout/Toolbar';

interface LorebookTabProps {
    workspace: WorkspaceState;
}

export const LorebookTab: React.FC<LorebookTabProps> = ({ workspace }) => {
    const [name, setName] = useState('');
    const [category, setCategory] = useState('Character');
    const [description, setDescription] = useState('');
    const [isSaving, setIsSaving] = useState(false);
    const [saveStatus, setSaveStatus] = useState<'idle' | 'success'>('idle');

    if (!workspace.active_directory_path) {
        return (
            <div className="flex flex-col items-center justify-center h-full p-6 text-center bg-white dark:bg-gray-900">
                <FolderOpen className="text-slate-400 dark:text-slate-500 mb-4" size={32} />
                <h3 className="font-semibold text-slate-700 dark:text-slate-300 mb-2">Vault Unlinked</h3>
                <p className="text-sm text-slate-500 dark:text-slate-400">You must open a Story Vault from the top toolbar before injecting explicit memories into the matrix.</p>
            </div>
        );
    }

    const handleSaveEntity = async () => {
        if (!name.trim() || !description.trim()) return;

        setIsSaving(true);
        setSaveStatus('idle');

        try {
            const response = await invoke<{ success: boolean }>('save_lore_entity', {
                name: name.trim(),
                category,
                description: description.trim()
            });

            if (response.success) {
                setSaveStatus('success');
                setName('');
                setDescription('');
                setTimeout(() => setSaveStatus('idle'), 3000);
            }
        } catch (error) {
            console.error('[Lorebook IPC Error]:', error);
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <div className="flex flex-col h-full p-4 space-y-4 overflow-y-auto bg-white dark:bg-gray-900">
            <div>
                <h3 className="text-sm font-semibold text-slate-800 dark:text-slate-200">Explicit Memory</h3>
                <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                    Define exact rules, characters, or locations to directly anchor Plato's consciousness.
                </p>
            </div>

            <div className="space-y-3">
                <div>
                    <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">Entity Name</label>
                    <input 
                        type="text" 
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder="e.g., The Obsidian Citadel"
                        className="w-full px-3 py-2 text-sm bg-slate-50 dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-md outline-none focus:ring-1 focus:ring-blue-500 text-slate-900 dark:text-slate-100"
                    />
                </div>

                <div>
                    <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">Category</label>
                    <select 
                        value={category}
                        onChange={(e) => setCategory(e.target.value)}
                        className="w-full px-3 py-2 text-sm bg-slate-50 dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-md outline-none focus:ring-1 focus:ring-blue-500 text-slate-900 dark:text-slate-100"
                    >
                        <option value="Character">Character</option>
                        <option value="Location">Location</option>
                        <option value="Item">Item</option>
                        <option value="Faction">Faction</option>
                        <option value="Concept">World Concept / Rule</option>
                    </select>
                </div>

                <div>
                    <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">Description & Lore</label>
                    <textarea 
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="Define the absolute facts about this entity..."
                        className="w-full h-40 px-3 py-2 text-sm bg-slate-50 dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-md resize-none outline-none focus:ring-1 focus:ring-blue-500 text-slate-900 dark:text-slate-100"
                    />
                </div>

                <button 
                    onClick={handleSaveEntity}
                    disabled={isSaving || !name.trim() || !description.trim()}
                    className={`w-full flex items-center justify-center gap-2 py-2.5 rounded-md text-sm font-medium transition-colors shadow-sm cursor-pointer
                        ${isSaving 
                            ? 'bg-indigo-100 text-indigo-400 cursor-wait' 
                            : saveStatus === 'success'
                                ? 'bg-green-600 text-white hover:bg-green-700'
                                : 'bg-indigo-600 text-white hover:bg-indigo-700 disabled:bg-slate-100 disabled:text-slate-400 disabled:cursor-not-allowed dark:disabled:bg-slate-800'
                        }`}
                >
                    {isSaving ? (
                        <><DatabaseZap size={16} className="animate-pulse" /> Injecting Matrix...</>
                    ) : saveStatus === 'success' ? (
                        <><CheckCircle2 size={16} /> Saved to Memory</>
                    ) : (
                        <><DatabaseZap size={16} /> Save to Memory Matrix</>
                    )}
                </button>
            </div>
        </div>
    );
};