import { useState, useRef, useEffect, useCallback, type FormEvent, type KeyboardEvent } from "react";
import { t, type Locale } from "../lib/i18n";
import { renderMarkdown } from "../lib/markdown";
import type { ChatSession, ChatMessage } from "../lib/storage";

interface ChatProps {
  chat: ChatSession | null;
  locale: Locale;
  isConnected: boolean;
  onSendMessage: (content: string) => Promise<void>;
  onRetry: (messages: ChatMessage[]) => void;
  onOpenSettings: () => void;
  onToggleSidebar: () => void;
  sidebarOpen: boolean;
}

export function Chat({
  chat,
  locale,
  isConnected,
  onSendMessage,
  onRetry,
  onOpenSettings,
  onToggleSidebar,
  sidebarOpen,
}: ChatProps) {
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const messages = chat?.messages ?? [];

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages.length, scrollToBottom]);

  useEffect(() => {
    if (!isLoading) {
      textareaRef.current?.focus();
    }
  }, [isLoading]);

  const handleSubmit = useCallback(
    async (e?: FormEvent) => {
      e?.preventDefault();
      const trimmed = input.trim();
      if (!trimmed || isLoading) return;

      setInput("");
      setIsLoading(true);

      // Reset textarea height
      if (textareaRef.current) {
        textareaRef.current.style.height = "auto";
      }

      try {
        await onSendMessage(trimmed);
      } finally {
        setIsLoading(false);
      }
    },
    [input, isLoading, onSendMessage],
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [handleSubmit],
  );

  const handleTextareaInput = useCallback(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = "auto";
      el.style.height = Math.min(el.scrollHeight, 120) + "px";
    }
  }, []);

  const canSend = input.trim().length > 0 && !isLoading && isConnected;

  return (
    <div className="chat-container">
      {/* Header */}
      <div className="chat-header">
        <button
          className="chat-header-toggle"
          onClick={onToggleSidebar}
          aria-label="Toggle sidebar"
        >
          {sidebarOpen ? "\u2715" : "\u2630"}
        </button>
        <span className="chat-header-title">
          {chat?.title ?? t("app_title", locale)}
        </span>
        <div className="chat-header-status">
          <div className={`status-dot ${isConnected ? "connected" : ""}`} />
          <span>{isConnected ? t("connected", locale) : t("disconnected", locale)}</span>
        </div>
      </div>

      {/* Messages */}
      <div className="chat-messages">
        {messages.length === 0 ? (
          <div className="chat-welcome">
            <div className="chat-welcome-icon">MoA</div>
            <h2>{t("welcome_title", locale)}</h2>
            <p>{t("welcome_subtitle", locale)}</p>
            <p>
              {isConnected
                ? t("welcome_hint", locale)
                : t("not_connected_hint", locale)}
            </p>
            {!isConnected && (
              <button className="chat-welcome-connect" onClick={onOpenSettings}>
                {t("login", locale)}
              </button>
            )}
          </div>
        ) : (
          <div className="chat-messages-inner">
            {messages.map((msg) => (
              <MessageBubble
                key={msg.id}
                message={msg}
                locale={locale}
                onRetry={
                  msg.role === "error"
                    ? () => onRetry(messages)
                    : undefined
                }
              />
            ))}

            {isLoading && (
              <div className="thinking-indicator">
                <div className="thinking-avatar">
                  <span style={{ color: "#fff", fontSize: 14, fontWeight: 600 }}>M</span>
                </div>
                <div className="thinking-dots">
                  <span />
                  <span />
                  <span />
                </div>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Input */}
      <div className="chat-input-area">
        <form onSubmit={handleSubmit} className="chat-input-wrapper">
          <textarea
            ref={textareaRef}
            className="chat-input"
            placeholder={t("type_message", locale)}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              handleTextareaInput();
            }}
            onKeyDown={handleKeyDown}
            rows={1}
            disabled={!isConnected}
          />
          <button
            type="submit"
            className="chat-send-btn"
            disabled={!canSend}
            aria-label={t("send", locale)}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          </button>
        </form>
      </div>
    </div>
  );
}

/* --- MessageBubble sub-component --- */

interface MessageBubbleProps {
  message: ChatMessage;
  locale: Locale;
  onRetry?: () => void;
}

function MessageBubble({ message, locale, onRetry }: MessageBubbleProps) {
  const isUser = message.role === "user";
  const isError = message.role === "error";

  return (
    <div className={`message ${message.role}`}>
      {!isUser && (
        <div className="message-avatar">
          {isError ? "!" : "M"}
        </div>
      )}
      <div className="message-content">
        {isUser ? (
          <div className="message-bubble">{message.content}</div>
        ) : (
          <div
            className="message-bubble"
            dangerouslySetInnerHTML={{
              __html: isError
                ? escapeForHtml(message.content)
                : renderMarkdown(message.content),
            }}
          />
        )}
        {message.model && (
          <div className="message-model">{t("model", locale)}: {message.model}</div>
        )}
        {isError && onRetry && (
          <button className="message-retry-btn" onClick={onRetry}>
            {t("retry", locale)}
          </button>
        )}
      </div>
    </div>
  );
}

function escapeForHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\n/g, "<br>");
}
