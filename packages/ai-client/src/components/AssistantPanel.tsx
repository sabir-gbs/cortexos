import React, { useState, useRef, useEffect, useCallback } from 'react';
import type {
  AssistantPanelProps,
  AIMessage,
  AISurfaceState,
} from '../types';

/** CortexOS design tokens */
const COLORS = {
  bg: '#1a1a2e',
  bgLighter: '#16213e',
  bgSurface: '#0f3460',
  text: '#e0e0e0',
  textMuted: '#a0a0b0',
  accent: '#533483',
  accentLight: '#7b68ee',
  border: '#2a2a4a',
  error: '#e74c3c',
  userBubble: '#533483',
  assistantBubble: '#16213e',
  inputBg: '#0f3460',
  overlay: 'rgba(0, 0, 0, 0.4)',
} as const;

const PANEL_WIDTH = 380;

/** Format a timestamp for display */
function formatTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleTimeString(undefined, {
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return '';
  }
}

/** Determine if a typing indicator should be shown */
function isTypingState(state: AISurfaceState | undefined): boolean {
  return state?.type === 'loading' || state?.type === 'streaming';
}

/** Single message bubble */
function MessageBubble({ message }: { message: AIMessage }) {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  const bubbleStyle: React.CSSProperties = {
    maxWidth: '85%',
    padding: '8px 12px',
    borderRadius: isUser ? '12px 12px 4px 12px' : '12px 12px 12px 4px',
    backgroundColor: isSystem
      ? COLORS.bgSurface
      : isUser
        ? COLORS.userBubble
        : COLORS.assistantBubble,
    color: COLORS.text,
    fontSize: '13px',
    lineHeight: '1.5',
    wordBreak: 'break-word',
    alignSelf: isUser ? 'flex-end' : 'flex-start',
    border: isSystem ? `1px solid ${COLORS.border}` : 'none',
    fontStyle: isSystem ? 'italic' : 'normal',
    opacity: isSystem ? 0.8 : 1,
  };

  const timeStyle: React.CSSProperties = {
    fontSize: '10px',
    color: COLORS.textMuted,
    marginTop: '4px',
    textAlign: isUser ? 'right' : 'left',
  };

  return (
    <div style={bubbleStyle}>
      <div>{message.content}</div>
      <div style={timeStyle}>{formatTime(message.timestamp)}</div>
    </div>
  );
}

/** Animated typing indicator */
function TypingIndicator() {
  const dotStyle: React.CSSProperties = {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
    backgroundColor: COLORS.accentLight,
    display: 'inline-block',
    animation: 'pulse 1.4s ease-in-out infinite',
  };

  return (
    <div
      role="status"
      aria-label="AI is generating a response"
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '4px',
        padding: '8px 12px',
        alignSelf: 'flex-start',
        backgroundColor: COLORS.assistantBubble,
        borderRadius: '12px 12px 12px 4px',
      }}
    >
      <span style={{ ...dotStyle, animationDelay: '0s' }} />
      <span style={{ ...dotStyle, animationDelay: '0.2s' }} />
      <span style={{ ...dotStyle, animationDelay: '0.4s' }} />
      <style>{`
        @keyframes pulse {
          0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
          40% { opacity: 1; transform: scale(1); }
        }
      `}</style>
    </div>
  );
}

/** Error message display */
function ErrorMessage({ state }: { state: AISurfaceState }) {
  if (state.type !== 'error') return null;
  const { error } = state;

  let message = 'An unexpected error occurred.';
  if (error.type === 'providerUnreachable') {
    message = `Unable to connect to ${error.provider}. Please check your internet connection and provider settings.`;
  } else if (error.type === 'authenticationFailed') {
    message = `Authentication with ${error.provider} failed. Please verify your API key in Settings > AI.`;
  } else if (error.type === 'modelUnavailable') {
    message = `The model '${error.model}' is currently unavailable on ${error.provider}. Please try again or select a different model.`;
  } else if (error.type === 'timeout') {
    message = `The AI response timed out after ${error.timeoutSeconds} seconds. You can increase the timeout in Settings > AI.`;
  } else if (error.type === 'internalError') {
    message = error.message;
  }

  return (
    <div
      role="alert"
      style={{
        backgroundColor: 'rgba(231, 76, 60, 0.15)',
        border: `1px solid ${COLORS.error}`,
        borderRadius: '8px',
        padding: '8px 12px',
        color: '#ff6b6b',
        fontSize: '12px',
        lineHeight: '1.5',
        alignSelf: 'stretch',
      }}
    >
      {message}
    </div>
  );
}

/** Rate limited display */
function RateLimitedMessage({ retryAfterMs }: { retryAfterMs: number }) {
  const seconds = Math.ceil(retryAfterMs / 1000);
  return (
    <div
      style={{
        backgroundColor: 'rgba(255, 193, 7, 0.15)',
        border: '1px solid #ffc107',
        borderRadius: '8px',
        padding: '8px 12px',
        color: '#ffc107',
        fontSize: '12px',
        alignSelf: 'stretch',
      }}
    >
      Rate limited. Please wait {seconds} seconds and try again.
    </div>
  );
}

/**
 * AssistantPanel - The global AI assistant panel for CortexOS
 *
 * A slide-in panel from the right side that provides conversation history,
 * message input, and model disclosure.
 */
export function AssistantPanel({
  isOpen,
  onClose,
  messages,
  onSendMessage,
  modelName,
  providerName,
  surfaceState,
  showDisclosure = true,
}: AssistantPanelProps) {
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const isInputDisabled = surfaceState?.type === 'loading' || surfaceState?.type === 'streaming';

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Focus input when panel opens
  useEffect(() => {
    if (isOpen) {
      // Small delay to allow animation
      const timer = setTimeout(() => {
        inputRef.current?.focus();
      }, 150);
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  const handleSend = useCallback(() => {
    const trimmed = inputValue.trim();
    if (!trimmed || isInputDisabled) return;
    onSendMessage(trimmed);
    setInputValue('');
  }, [inputValue, isInputDisabled, onSendMessage]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [handleSend, onClose],
  );

  // Panel overlay
  const overlayStyle: React.CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: COLORS.overlay,
    zIndex: 9998,
    opacity: isOpen ? 1 : 0,
    pointerEvents: isOpen ? 'auto' : 'none',
    transition: 'opacity 0.2s ease',
  };

  // Panel container
  const panelStyle: React.CSSProperties = {
    position: 'fixed',
    top: 0,
    right: 0,
    bottom: 0,
    width: `${PANEL_WIDTH}px`,
    maxWidth: '100vw',
    backgroundColor: COLORS.bg,
    borderLeft: `1px solid ${COLORS.border}`,
    display: 'flex',
    flexDirection: 'column',
    zIndex: 9999,
    transform: isOpen ? 'translateX(0)' : 'translateX(100%)',
    transition: 'transform 0.25s ease',
    boxShadow: isOpen ? '-4px 0 24px rgba(0, 0, 0, 0.3)' : 'none',
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  };

  // Header bar
  const headerStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    borderBottom: `1px solid ${COLORS.border}`,
    backgroundColor: COLORS.bgLighter,
    flexShrink: 0,
  };

  // Title
  const titleStyle: React.CSSProperties = {
    color: COLORS.text,
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
  };

  // Disclosure badge
  const badgeStyle: React.CSSProperties = {
    fontSize: '10px',
    color: COLORS.textMuted,
    backgroundColor: COLORS.bgSurface,
    padding: '2px 8px',
    borderRadius: '10px',
    border: `1px solid ${COLORS.border}`,
    whiteSpace: 'nowrap',
  };

  // Close button
  const closeButtonStyle: React.CSSProperties = {
    background: 'none',
    border: 'none',
    color: COLORS.textMuted,
    fontSize: '20px',
    cursor: 'pointer',
    padding: '4px 8px',
    borderRadius: '4px',
    lineHeight: 1,
    transition: 'background-color 0.15s',
  };

  // Messages area
  const messagesStyle: React.CSSProperties = {
    flex: 1,
    overflowY: 'auto',
    padding: '12px 16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  };

  // Input area
  const inputAreaStyle: React.CSSProperties = {
    padding: '12px 16px',
    borderTop: `1px solid ${COLORS.border}`,
    backgroundColor: COLORS.bgLighter,
    flexShrink: 0,
  };

  // Textarea
  const textareaStyle: React.CSSProperties = {
    width: '100%',
    minHeight: '36px',
    maxHeight: '96px', // ~4 lines
    padding: '8px 12px',
    backgroundColor: COLORS.inputBg,
    color: COLORS.text,
    border: `1px solid ${COLORS.border}`,
    borderRadius: '8px',
    fontSize: '13px',
    lineHeight: '1.5',
    resize: 'none',
    outline: 'none',
    fontFamily: 'inherit',
    boxSizing: 'border-box',
  };

  // Send button
  const sendButtonStyle: React.CSSProperties = {
    marginTop: '8px',
    width: '100%',
    padding: '8px',
    backgroundColor: isInputDisabled ? COLORS.border : COLORS.accent,
    color: COLORS.text,
    border: 'none',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 500,
    cursor: isInputDisabled ? 'not-allowed' : 'pointer',
    transition: 'background-color 0.15s',
    opacity: isInputDisabled ? 0.6 : 1,
  };

  const emptyStateStyle: React.CSSProperties = {
    color: COLORS.textMuted,
    fontSize: '13px',
    textAlign: 'center',
    padding: '32px 16px',
    lineHeight: '1.6',
  };

  return (
    <>
      {/* Backdrop overlay */}
      <div style={overlayStyle} onClick={onClose} aria-hidden="true" />

      {/* Panel */}
      <div
        style={panelStyle}
        role="complementary"
        aria-label="AI Assistant Panel"
        aria-live="polite"
      >
        {/* Header */}
        <div style={headerStyle}>
          <h2 style={titleStyle}>AI Assistant</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            {showDisclosure && modelName && (
              <span
                style={badgeStyle}
                aria-label={`Using ${providerName ?? 'AI'} model ${modelName}`}
              >
                {providerName ? `${providerName} / ` : ''}
                {modelName}
              </span>
            )}
            <button
              style={closeButtonStyle}
              onClick={onClose}
              aria-label="Close assistant panel"
              title="Close (Esc)"
            >
              x
            </button>
          </div>
        </div>

        {/* Messages */}
        <div style={messagesStyle}>
          {messages.length === 0 && (
            <div style={emptyStateStyle}>
              Ask me anything. I can help with writing, analysis, code, and
              more.
            </div>
          )}
          {messages.map((msg) => (
            <MessageBubble key={msg.messageId} message={msg} />
          ))}
          {surfaceState?.type === 'error' && (
            <ErrorMessage state={surfaceState} />
          )}
          {surfaceState?.type === 'rateLimited' && (
            <RateLimitedMessage retryAfterMs={surfaceState.retryAfterMs} />
          )}
          {isTypingState(surfaceState) && <TypingIndicator />}
          <div ref={messagesEndRef} />
        </div>

        {/* Input area */}
        <div style={inputAreaStyle}>
          <textarea
            ref={inputRef}
            style={textareaStyle}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={
              isInputDisabled ? 'Waiting for response...' : 'Type a message...'
            }
            disabled={isInputDisabled}
            rows={1}
            aria-label="Message input"
          />
          <button
            style={sendButtonStyle}
            onClick={handleSend}
            disabled={isInputDisabled || !inputValue.trim()}
            aria-label="Send message"
          >
            Send
          </button>
        </div>
      </div>
    </>
  );
}

export default AssistantPanel;
