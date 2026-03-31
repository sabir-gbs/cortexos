/**
 * @cortexos/ai-client
 *
 * AI system surfaces and UX components for CortexOS.
 * Based on SPEC 19: AI System Surfaces and UX.
 */

// Types
export type {
  AIUUID,
  MessageRole,
  AIModelInfo,
  AIMessage,
  AIConversationContext,
  AIConversationMetadata,
  AIConversation,
  AIActionCategory,
  AIAction,
  AIActionResult,
  AIActionTarget,
  AISurfaceState,
  AISurfaceError,
  AIContextPayload,
  AISurfaceConfig,
  AssistantPanelProps,
  ActionConfirmationProps,
} from './types';

export { DEFAULT_SURFACE_CONFIG } from './types';

// Components
export { AssistantPanel } from './components/AssistantPanel';
export { ActionConfirmation } from './components/ActionConfirmation';

// Hooks
export { useAIContext } from './hooks/useAIContext';
export { useAIActions } from './hooks/useAIActions';
export type { AIActionHandler } from './hooks/useAIActions';
