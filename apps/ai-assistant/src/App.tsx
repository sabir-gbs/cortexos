import React, { useState, useRef, useEffect, useCallback } from 'react';

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

const CANNED_RESPONSES = [
  "I can help you with that. Let me look into it.",
  "That's a great question. Here's what I think...",
  "Sure, I can assist with that.",
  "Let me process that request for you.",
  "Interesting! Here's my take on that.",
  "I'm here to help. Could you tell me more?",
  "Got it. I'll work on that right away.",
  "Thanks for asking! Here's what I know.",
];

const SYSTEM_MESSAGE: Message = {
  role: 'system',
  content: 'AI Assistant (Ctrl+Shift+A)',
};

export function App() {
  const [messages, setMessages] = useState<Message[]>([SYSTEM_MESSAGE]);
  const [input, setInput] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const scrollToBottom = useCallback(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, isTyping, scrollToBottom]);

  const sendMessage = useCallback(() => {
    const text = input.trim();
    if (!text || isTyping) return;

    const userMessage: Message = { role: 'user', content: text };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsTyping(true);

    // Simulate assistant response with a short delay
    const delay = 400 + Math.random() * 800;
    setTimeout(() => {
      const responseText =
        text.toLowerCase().includes('hello') || text.toLowerCase().includes('hi')
          ? 'Hello! How can I help you today?'
          : CANNED_RESPONSES[Math.floor(Math.random() * CANNED_RESPONSES.length)];

      const assistantMessage: Message = { role: 'assistant', content: responseText };
      setMessages((prev) => [...prev, assistantMessage]);
      setIsTyping(false);
    }, delay);
  }, [input, isTyping]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
      }
    },
    [sendMessage],
  );

  const hasUserMessages = messages.some((m) => m.role === 'user');

  return (
    <div className="ai-assistant">
      <div className="message-list" ref={scrollRef}>
        {messages.map((msg, idx) => (
          <div key={idx} className={`message-row message-${msg.role}`}>
            {msg.role === 'assistant' && (
              <div className="avatar avatar-assistant">AI</div>
            )}
            <div className={`bubble bubble-${msg.role}`}>
              {msg.content}
            </div>
            {msg.role === 'user' && (
              <div className="avatar avatar-user">You</div>
            )}
          </div>
        ))}
        {isTyping && (
          <div className="message-row message-assistant">
            <div className="avatar avatar-assistant">AI</div>
            <div className="bubble bubble-assistant typing-indicator">
              <span className="dot" />
              <span className="dot" />
              <span className="dot" />
            </div>
          </div>
        )}
        {!hasUserMessages && !isTyping && (
          <div className="empty-state">Ask me anything...</div>
        )}
      </div>

      <div className="input-area">
        <input
          ref={inputRef}
          type="text"
          className="chat-input"
          placeholder="Type a message..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isTyping}
          autoFocus
        />
        <button
          className="send-button"
          onClick={sendMessage}
          disabled={!input.trim() || isTyping}
          aria-label="Send message"
        >
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z" />
          </svg>
        </button>
      </div>
    </div>
  );
}
