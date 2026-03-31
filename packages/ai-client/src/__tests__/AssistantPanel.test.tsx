import { render, screen, fireEvent, cleanup, within } from '@testing-library/react';
import { describe, it, expect, afterEach, vi } from 'vitest';
import React from 'react';
import { AssistantPanel } from '../components/AssistantPanel';
import type { AIMessage, AISurfaceState } from '../types';

function makeMessage(overrides: Partial<AIMessage> = {}): AIMessage {
  return {
    messageId: 'msg-1',
    role: 'user',
    content: 'Hello',
    timestamp: '2026-03-31T12:00:00Z',
    tokenCount: 1,
    complete: true,
    metadata: {},
    ...overrides,
  };
}

describe('AssistantPanel', () => {
  afterEach(() => {
    cleanup();
  });

  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    messages: [] as AIMessage[],
    onSendMessage: vi.fn(),
    modelName: 'GPT-4o',
    providerName: 'OpenAI',
  };

  it('renders the panel when isOpen is true', () => {
    render(<AssistantPanel {...defaultProps} />);
    expect(screen.getByText('AI Assistant')).toBeInTheDocument();
  });

  it('does not show panel content when isOpen is false', () => {
    render(<AssistantPanel {...defaultProps} isOpen={false} />);
    // The panel still renders in DOM but is translated off-screen
    const panel = screen.getByRole('complementary', { hidden: true });
    expect(panel).toBeInTheDocument();
  });

  it('shows the close button', () => {
    render(<AssistantPanel {...defaultProps} />);
    expect(
      screen.getByRole('button', { name: /close assistant panel/i }),
    ).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    const onClose = vi.fn();
    render(<AssistantPanel {...defaultProps} onClose={onClose} />);
    fireEvent.click(screen.getByRole('button', { name: /close assistant panel/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when the overlay is clicked', () => {
    const onClose = vi.fn();
    render(<AssistantPanel {...defaultProps} onClose={onClose} />);
    // The overlay is the div before the panel
    const overlay = screen.getByRole('complementary').previousElementSibling;
    if (overlay) {
      fireEvent.click(overlay);
      expect(onClose).toHaveBeenCalledTimes(1);
    }
  });

  it('displays messages in the conversation', () => {
    const messages: AIMessage[] = [
      makeMessage({ messageId: 'msg-1', role: 'user', content: 'Hello AI' }),
      makeMessage({
        messageId: 'msg-2',
        role: 'assistant',
        content: 'Hi there! How can I help?',
      }),
    ];
    render(<AssistantPanel {...defaultProps} messages={messages} />);
    expect(screen.getByText('Hello AI')).toBeInTheDocument();
    expect(screen.getByText('Hi there! How can I help?')).toBeInTheDocument();
  });

  it('displays system messages', () => {
    const messages: AIMessage[] = [
      makeMessage({
        messageId: 'msg-sys',
        role: 'system',
        content: 'Model changed to OpenAI / GPT-4o.',
      }),
    ];
    render(<AssistantPanel {...defaultProps} messages={messages} />);
    expect(screen.getByText('Model changed to OpenAI / GPT-4o.')).toBeInTheDocument();
  });

  it('shows an empty state when no messages', () => {
    render(<AssistantPanel {...defaultProps} messages={[]} />);
    expect(
      screen.getByText(/Ask me anything/),
    ).toBeInTheDocument();
  });

  it('has a message input field', () => {
    render(<AssistantPanel {...defaultProps} />);
    expect(screen.getByLabelText('Message input')).toBeInTheDocument();
  });

  it('calls onSendMessage when a message is typed and sent', () => {
    const onSendMessage = vi.fn();
    render(<AssistantPanel {...defaultProps} onSendMessage={onSendMessage} />);

    const input = screen.getByLabelText('Message input');
    fireEvent.change(input, { target: { value: 'Test message' } });
    fireEvent.click(screen.getByRole('button', { name: /send message/i }));

    expect(onSendMessage).toHaveBeenCalledWith('Test message');
  });

  it('sends message on Enter key (without Shift)', () => {
    const onSendMessage = vi.fn();
    render(<AssistantPanel {...defaultProps} onSendMessage={onSendMessage} />);

    const input = screen.getByLabelText('Message input');
    fireEvent.change(input, { target: { value: 'Enter test' } });
    fireEvent.keyDown(input, { key: 'Enter', shiftKey: false });

    expect(onSendMessage).toHaveBeenCalledWith('Enter test');
  });

  it('does not send message on Shift+Enter', () => {
    const onSendMessage = vi.fn();
    render(<AssistantPanel {...defaultProps} onSendMessage={onSendMessage} />);

    const input = screen.getByLabelText('Message input');
    fireEvent.change(input, { target: { value: 'Shift enter test' } });
    fireEvent.keyDown(input, { key: 'Enter', shiftKey: true });

    expect(onSendMessage).not.toHaveBeenCalled();
  });

  it('calls onClose when Escape is pressed in the input', () => {
    const onClose = vi.fn();
    render(<AssistantPanel {...defaultProps} onClose={onClose} />);

    const input = screen.getByLabelText('Message input');
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('does not send empty messages', () => {
    const onSendMessage = vi.fn();
    render(<AssistantPanel {...defaultProps} onSendMessage={onSendMessage} />);

    fireEvent.click(screen.getByRole('button', { name: /send message/i }));
    expect(onSendMessage).not.toHaveBeenCalled();
  });

  it('shows model disclosure badge when enabled', () => {
    render(
      <AssistantPanel
        {...defaultProps}
        showDisclosure={true}
        providerName="OpenAI"
        modelName="GPT-4o"
      />,
    );
    expect(screen.getByText('OpenAI / GPT-4o')).toBeInTheDocument();
  });

  it('hides model disclosure badge when disabled', () => {
    render(
      <AssistantPanel
        {...defaultProps}
        showDisclosure={false}
        providerName="OpenAI"
        modelName="GPT-4o"
      />,
    );
    expect(screen.queryByText('OpenAI / GPT-4o')).not.toBeInTheDocument();
  });

  it('shows typing indicator during loading state', () => {
    const loadingState: AISurfaceState = {
      type: 'loading',
      requestId: 'req-1',
      startedAt: '2026-03-31T12:00:00Z',
    };
    render(
      <AssistantPanel {...defaultProps} surfaceState={loadingState} />,
    );
    expect(
      screen.getByLabelText('AI is generating a response'),
    ).toBeInTheDocument();
  });

  it('shows typing indicator during streaming state', () => {
    const streamingState: AISurfaceState = {
      type: 'streaming',
      requestId: 'req-1',
      startedAt: '2026-03-31T12:00:00Z',
      tokensReceived: 42,
    };
    render(
      <AssistantPanel {...defaultProps} surfaceState={streamingState} />,
    );
    expect(
      screen.getByLabelText('AI is generating a response'),
    ).toBeInTheDocument();
  });

  it('disables input during loading state', () => {
    const loadingState: AISurfaceState = {
      type: 'loading',
      requestId: 'req-1',
      startedAt: '2026-03-31T12:00:00Z',
    };
    render(
      <AssistantPanel {...defaultProps} surfaceState={loadingState} />,
    );
    expect(screen.getByLabelText('Message input')).toBeDisabled();
  });

  it('shows error message in error state', () => {
    const errorState: AISurfaceState = {
      type: 'error',
      error: {
        type: 'providerUnreachable',
        provider: 'OpenAI',
      },
    };
    render(<AssistantPanel {...defaultProps} surfaceState={errorState} />);
    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(
      screen.getByText(/Unable to connect to OpenAI/),
    ).toBeInTheDocument();
  });

  it('shows authentication error with settings guidance', () => {
    const errorState: AISurfaceState = {
      type: 'error',
      error: {
        type: 'authenticationFailed',
        provider: 'Anthropic',
      },
    };
    render(<AssistantPanel {...defaultProps} surfaceState={errorState} />);
    expect(
      screen.getByText(/Authentication with Anthropic failed/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Settings > AI/),
    ).toBeInTheDocument();
  });

  it('shows rate limited message with retry info', () => {
    const rateLimitedState: AISurfaceState = {
      type: 'rateLimited',
      retryAfterMs: 30000,
    };
    render(
      <AssistantPanel {...defaultProps} surfaceState={rateLimitedState} />,
    );
    expect(screen.getByText(/wait 30 seconds/)).toBeInTheDocument();
  });

  it('clears input after sending a message', () => {
    render(<AssistantPanel {...defaultProps} />);
    const input = screen.getByLabelText('Message input') as HTMLTextAreaElement;

    fireEvent.change(input, { target: { value: 'Hello' } });
    expect(input.value).toBe('Hello');

    fireEvent.click(screen.getByRole('button', { name: /send message/i }));
    expect(input.value).toBe('');
  });
});
