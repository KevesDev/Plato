import type { EditorThemeClasses } from 'lexical';

/**
 * Plato Editor Theme Configuration
 * Defines the strict mapping between Lexical's internal state and the DOM CSS classes.
 * This ensures absolute separation of logic and presentation layer.
 */
export const platoEditorTheme: EditorThemeClasses = {
    ltr: 'plato-text-ltr',
    rtl: 'plato-text-rtl',
    placeholder: 'plato-editor-placeholder',
    paragraph: 'plato-editor-paragraph',
    quote: 'plato-editor-quote',
    heading: {
        h1: 'plato-editor-h1',
        h2: 'plato-editor-h2',
        h3: 'plato-editor-h3',
        h4: 'plato-editor-h4',
        h5: 'plato-editor-h5',
        h6: 'plato-editor-h6',
    },
    list: {
        nested: {
            listitem: 'plato-editor-nested-listitem',
        },
        ol: 'plato-editor-list-ol',
        ul: 'plato-editor-list-ul',
        listitem: 'plato-editor-listitem',
        listitemChecked: 'plato-editor-listitem-checked',
        listitemUnchecked: 'plato-editor-listitem-unchecked',
    },
    text: {
        bold: 'plato-text-bold',
        italic: 'plato-text-italic',
        underline: 'plato-text-underline',
        strikethrough: 'plato-text-strikethrough',
        underlineStrikethrough: 'plato-text-underline-strikethrough',
        code: 'plato-text-code',
    },
    // Custom nodes defined below
    aiGenerated: 'plato-text-ai-generated',
};