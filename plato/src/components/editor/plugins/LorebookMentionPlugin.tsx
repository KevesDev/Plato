import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { LexicalTypeaheadMenuPlugin, MenuOption, useBasicTypeaheadTriggerMatch } from '@lexical/react/LexicalTypeaheadMenuPlugin';
import { $getSelection, $isRangeSelection, TextNode, LexicalEditor } from 'lexical';
import React, { useCallback, useEffect, useState } from 'react';
import ReactDOM from 'react-dom';
import { invoke } from '@tauri-apps/api/core';
import { $createMentionNode } from '../nodes/MentionNode';

class MentionOption extends MenuOption {
    name: string;
    constructor(name: string) {
        super(name);
        this.name = name;
    }
}

export function LorebookMentionPlugin() {
    const [editor] = useLexicalComposerContext();
    const [queryString, setQueryString] = useState<string | null>(null);
    const [options, setOptions] = useState<MentionOption[]>([]);
    const [lorebookNames, setLorebookNames] = useState<string[]>([]);

    useEffect(() => {
        const fetchLore = async () => {
            try {
                const response = await invoke<{ success: boolean; data?: string[] }>('get_lorebook_entries');
                if (response.success && response.data) {
                    console.log("[LorebookMention] Fetched entries from matrix:", response.data);
                    setLorebookNames(response.data);
                }
            } catch (err) {
                console.error("[LorebookMention] Failed to fetch entries", err);
            }
        };
        
        fetchLore();
        window.addEventListener('plato-workspace-update', fetchLore);
        return () => window.removeEventListener('plato-workspace-update', fetchLore);
    }, []);

    useEffect(() => {
        console.log("[LorebookMention] Query string updated to:", queryString);
        if (queryString) {
            const regex = new RegExp(queryString, 'i');
            const filtered = lorebookNames.filter((name) => regex.test(name));
            setOptions(filtered.map((name) => new MentionOption(name)));
        } else {
            setOptions(lorebookNames.map((name) => new MentionOption(name)));
        }
    }, [queryString, lorebookNames]);

    useEffect(() => {
        const handleLinkAction = () => {
            console.log("[LorebookMention] Toolbar Link action triggered");
            editor.update(() => {
                const selection = $getSelection();
                if ($isRangeSelection(selection)) {
                    const text = selection.getTextContent();
                    if (text.length > 0) {
                        selection.insertText('@' + text);
                    } else {
                        selection.insertText('@');
                    }
                }
            });

            // CRITICAL FIX: The Typeahead plugin requires a slight delay to allow the 
            // editor state to reconcile the newly inserted '@' before it will scan it.
            setTimeout(() => {
                editor.focus();
                console.log("[LorebookMention] Forced editor focus to trigger dropdown");
            }, 10);
        };
        window.addEventListener('plato-action-link', handleLinkAction);
        return () => window.removeEventListener('plato-action-link', handleLinkAction);
    }, [editor]);

    const basicMatch = useBasicTypeaheadTriggerMatch('@', {
        minLength: 0,
    });

    const triggerFn = useCallback(
        (text: string, currentEditor: LexicalEditor) => {
            const match = basicMatch(text, currentEditor);
            console.log("[Typeahead Matcher] Scanning text chunk:", text, "| Match Result:", match);
            return match;
        },
        [basicMatch]
    );

    const onSelectOption = useCallback(
        (
            selectedOption: MentionOption,
            nodeToReplace: TextNode | null,
            closeMenu: () => void,
        ) => {
            console.log("[LorebookMention] Option selected:", selectedOption.name);
            editor.update(() => {
                const mentionNode = $createMentionNode(selectedOption.name);
                if (nodeToReplace) {
                    nodeToReplace.replace(mentionNode);
                }
                mentionNode.selectNext();
            });
            closeMenu();
        },
        [editor],
    );

    return (
        <LexicalTypeaheadMenuPlugin<MentionOption>
            onQueryChange={setQueryString}
            onSelectOption={onSelectOption}
            triggerFn={triggerFn}
            options={options}
            menuRenderFn={(anchorElementRef, { selectedIndex, selectOptionAndCleanUp, setHighlightedIndex }) => {
                if (anchorElementRef.current == null || options.length === 0) {
                    return null;
                }
                
                return ReactDOM.createPortal(
                    <div 
                        className="absolute z-50 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-md shadow-xl min-w-[200px] py-1 mt-1 font-sans"
                        style={{
                            top: anchorElementRef.current.getBoundingClientRect().bottom,
                            left: anchorElementRef.current.getBoundingClientRect().left,
                        }}
                    >
                        <div className="px-3 py-1.5 text-xs font-semibold text-slate-500 border-b border-slate-100 dark:border-slate-800">
                            Link Lorebook Entity
                        </div>
                        {options.map((option, i) => (
                            <div
                                key={option.key}
                                onClick={() => {
                                    setHighlightedIndex(i);
                                    selectOptionAndCleanUp(option);
                                }}
                                onMouseEnter={() => setHighlightedIndex(i)}
                                className={`px-3 py-2 text-sm cursor-pointer transition-colors ${
                                    selectedIndex === i 
                                        ? 'bg-blue-50 dark:bg-slate-800 text-blue-700 dark:text-blue-300' 
                                        : 'text-slate-700 dark:text-slate-300'
                                }`}
                            >
                                {option.name}
                            </div>
                        ))}
                    </div>,
                    document.body
                );
            }}
        />
    );
}