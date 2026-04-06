import { $applyNodeReplacement, TextNode, SerializedTextNode, NodeKey } from 'lexical';

export type SerializedMentionNode = SerializedTextNode & {
    mentionName: string;
};

export class MentionNode extends TextNode {
    __mention: string;

    static getType(): string {
        return 'mention';
    }

    static clone(node: MentionNode): MentionNode {
        return new MentionNode(node.__mention, node.__text, node.__key);
    }

    static importJSON(serializedNode: SerializedMentionNode): MentionNode {
        const node = $createMentionNode(serializedNode.mentionName);
        node.setTextContent(serializedNode.text);
        node.setFormat(serializedNode.format);
        node.setDetail(serializedNode.detail);
        node.setMode(serializedNode.mode);
        node.setStyle(serializedNode.style);
        return node;
    }

    exportJSON(): SerializedMentionNode {
        return {
            ...super.exportJSON(),
            mentionName: this.__mention,
            type: 'mention',
            version: 1,
        };
    }

    constructor(mentionName: string, text?: string, key?: NodeKey) {
        super(text ?? mentionName, key);
        this.__mention = mentionName;
    }

    createDOM(config: any): HTMLElement {
        const dom = super.createDOM(config);
        // Styled as a sleek, interactive pill
        dom.className = 'px-1.5 py-0.5 mx-0.5 bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300 rounded-md font-medium text-sm border border-blue-200 dark:border-blue-800 shadow-sm cursor-default select-all';
        return dom;
    }

    isTextEntity(): true {
        return true;
    }
}

export function $createMentionNode(mentionName: string): MentionNode {
    const mentionNode = new MentionNode(mentionName);
    mentionNode.setMode('segmented').toggleDirectionless();
    return $applyNodeReplacement(mentionNode);
}

export function $isMentionNode(node: any): node is MentionNode {
    return node instanceof MentionNode;
}