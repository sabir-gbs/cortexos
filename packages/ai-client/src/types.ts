/**
 * Core types for @cortexos/ai-client
 * Based on SPEC 19: AI System Surfaces and UX
 */

/** Unique identifier format used throughout the AI client */
export type AIUUID = string;

/** Roles a message can have in a conversation */
export type MessageRole = 'user' | 'assistant' | 'system';

/** Provider and model information */
export interface AIModelInfo {
  provider: string;
  model: string;
}

/** A single message within a conversation */
export interface AIMessage {
  messageId: AIUUID;
  role: MessageRole;
  content: string;
  timestamp: string; // ISO 8601
  tokenCount: number;
  modelInfo?: AIModelInfo;
  complete: boolean;
  metadata: Record<string, string>;
}

/** Context for scoping a conversation */
export type AIConversationContext =
  | { type: 'global' }
  | { type: 'app'; appId: string; contextKey: string };

/** Metadata about a conversation */
export interface AIConversationMetadata {
  totalTokensUsed: number;
  primaryProvider: string;
  primaryModel: string;
  messageCount: number;
}

/** A full conversation with its messages */
export interface AIConversation {
  conversationId: AIUUID;
  context: AIConversationContext;
  messages: AIMessage[];
  createdAt: string; // ISO 8601
  updatedAt: string; // ISO 8601
  metadata: AIConversationMetadata;
}

/** Category of an AI action */
export type AIActionCategory =
  | { type: 'text'; operation: string }
  | { type: 'file'; operation: string }
  | { type: 'custom'; operationName: string };

/** Definition of an AI action that can be registered by apps */
export interface AIAction {
  actionId: string;
  appId: string;
  label: string;
  description: string;
  icon?: string;
  category: AIActionCategory;
  requiresConfirmation: boolean;
  isDestructive?: boolean;
}

/** Result from executing an AI action */
export interface AIActionResult {
  output: string;
  modifiedResources: string[];
  tokensUsed: number;
}

/** Target for an AI action */
export type AIActionTarget =
  | { type: 'selectedText'; content: string }
  | { type: 'selectedFiles'; paths: string[] }
  | { type: 'both'; text?: string; files: string[] };

/** State of an AI surface interaction */
export type AISurfaceState =
  | { type: 'idle' }
  | { type: 'loading'; requestId: AIUUID; startedAt: string }
  | { type: 'streaming'; requestId: AIUUID; startedAt: string; tokensReceived: number }
  | { type: 'complete'; requestId: AIUUID; totalTokens: number; durationMs: number }
  | { type: 'error'; requestId?: AIUUID; error: AISurfaceError }
  | { type: 'rateLimited'; retryAfterMs: number }
  | { type: 'quotaExceeded'; provider: string };

/** Error types for AI surface failures */
export type AISurfaceError =
  | { type: 'providerUnreachable'; provider: string }
  | { type: 'authenticationFailed'; provider: string }
  | { type: 'modelUnavailable'; provider: string; model: string }
  | { type: 'timeout'; timeoutSeconds: number }
  | { type: 'internalError'; message: string };

/** Context payload provided to AI for app-aware responses */
export interface AIContextPayload {
  appId: string;
  appName: string;
  activeFile?: string;
  selectedText?: string;
  selectedFiles?: string[];
  customContext?: Record<string, unknown>;
}

/** Configuration for an AI surface */
export interface AISurfaceConfig {
  showModelDisclosure: boolean;
  assistantPanelShortcut: string;
  autoApplyLowRiskActions: boolean;
  requestTimeoutMs: number;
  maxHistoryDisplay: number;
}

/** Default configuration values */
export const DEFAULT_SURFACE_CONFIG: AISurfaceConfig = {
  showModelDisclosure: true,
  assistantPanelShortcut: 'Ctrl+Shift+A',
  autoApplyLowRiskActions: false,
  requestTimeoutMs: 120_000,
  maxHistoryDisplay: 50,
};

/** Props for the AssistantPanel component */
export interface AssistantPanelProps {
  /** Whether the panel is currently open */
  isOpen: boolean;
  /** Callback when the panel should close */
  onClose: () => void;
  /** Messages in the current conversation */
  messages: AIMessage[];
  /** Callback when the user sends a message */
  onSendMessage: (content: string) => void;
  /** Display name of the active AI model */
  modelName: string;
  /** Optional display name of the active provider */
  providerName?: string;
  /** Current surface state */
  surfaceState?: AISurfaceState;
  /** Whether to show model disclosure */
  showDisclosure?: boolean;
}

/** Props for the ActionConfirmation component */
export interface ActionConfirmationProps {
  /** The action being confirmed */
  action: AIAction;
  /** Target for the action */
  target?: AIActionTarget;
  /** Callback when the user confirms the action */
  onConfirm: () => void;
  /** Callback when the user denies the action */
  onDeny: () => void;
  /** Whether the action is currently loading */
  isLoading?: boolean;
}
