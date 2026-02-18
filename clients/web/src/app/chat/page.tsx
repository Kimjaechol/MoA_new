"use client";

import { useState, useEffect, useRef, useCallback, FormEvent } from "react";
import Link from "next/link";
import { getClient, MoAClient, renderMarkdown } from "@/lib/api";
import type { ChatMessage } from "@/lib/api";

export default function ChatPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [connected, setConnected] = useState(false);
  const [showSetup, setShowSetup] = useState(false);
  const [serverUrl, setServerUrl] = useState("");
  const [pairingCode, setPairingCode] = useState("");
  const [setupError, setSetupError] = useState("");
  const [setupLoading, setSetupLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const clientRef = useRef<MoAClient | null>(null);

  useEffect(() => {
    const client = getClient();
    clientRef.current = client;
    setServerUrl(client.getServerUrl());
    setConnected(client.isConnected());
    if (!client.isConnected()) {
      setShowSetup(true);
    }
    setMessages(MoAClient.loadMessages());
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSetup = async (e: FormEvent) => {
    e.preventDefault();
    if (!clientRef.current) return;
    setSetupLoading(true);
    setSetupError("");

    try {
      clientRef.current.setServerUrl(serverUrl);
      await clientRef.current.healthCheck();

      if (pairingCode.trim()) {
        await clientRef.current.pair(pairingCode.trim());
      }

      setConnected(clientRef.current.isConnected() || !pairingCode.trim());
      setShowSetup(false);
    } catch (err) {
      setSetupError(
        err instanceof Error ? err.message : "Connection failed"
      );
    } finally {
      setSetupLoading(false);
    }
  };

  const sendMessage = useCallback(async () => {
    const text = input.trim();
    if (!text || loading || !clientRef.current) return;

    const userMsg = MoAClient.createMessage("user", text);
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);
    MoAClient.saveMessages(newMessages);
    setInput("");
    setLoading(true);

    try {
      const res = await clientRef.current.chat(text);
      const assistantMsg = MoAClient.createMessage(
        "assistant",
        res.response,
        res.model
      );
      const updated = [...newMessages, assistantMsg];
      setMessages(updated);
      MoAClient.saveMessages(updated);
    } catch (err) {
      const errorMsg = MoAClient.createMessage(
        "assistant",
        `오류: ${err instanceof Error ? err.message : "요청 실패"}`
      );
      const updated = [...newMessages, errorMsg];
      setMessages(updated);
      MoAClient.saveMessages(updated);
    } finally {
      setLoading(false);
    }
  }, [input, loading, messages]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const clearChat = () => {
    setMessages([]);
    MoAClient.clearMessages();
  };

  return (
    <div className="flex flex-col h-screen bg-gray-950 text-white">
      {/* Header */}
      <header className="flex items-center justify-between px-4 h-14 border-b border-gray-800 bg-gray-900/80 backdrop-blur shrink-0">
        <div className="flex items-center gap-3">
          <Link
            href="/"
            className="text-gray-400 hover:text-white transition-colors"
          >
            ← 홈
          </Link>
          <span className="text-sm font-medium">MoA Chat</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 text-xs text-gray-500">
            <span
              className={`w-2 h-2 rounded-full ${connected ? "bg-green-500" : "bg-red-500"}`}
            />
            {connected ? "연결됨" : "미연결"}
          </div>
          <button
            onClick={() => setShowSetup(true)}
            className="text-xs px-3 py-1.5 border border-gray-700 rounded-lg hover:border-gray-500 text-gray-400 hover:text-white transition-colors"
          >
            설정
          </button>
          <button
            onClick={clearChat}
            className="text-xs px-3 py-1.5 border border-gray-700 rounded-lg hover:border-gray-500 text-gray-400 hover:text-white transition-colors"
          >
            새 대화
          </button>
        </div>
      </header>

      {/* Setup Modal */}
      {showSetup && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 w-full max-w-md">
            <h2 className="text-lg font-semibold mb-1">서버 연결</h2>
            <p className="text-sm text-gray-400 mb-6">
              MoA 백엔드 서버에 연결하세요
            </p>

            <form onSubmit={handleSetup} className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  서버 URL
                </label>
                <input
                  type="url"
                  value={serverUrl}
                  onChange={(e) => setServerUrl(e.target.value)}
                  placeholder="https://your-app.railway.app"
                  className="w-full px-4 py-2.5 bg-gray-800 border border-gray-700 rounded-lg text-sm focus:border-indigo-500 focus:outline-none"
                  required
                />
              </div>

              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  페어링 코드 (선택)
                </label>
                <input
                  type="text"
                  value={pairingCode}
                  onChange={(e) => setPairingCode(e.target.value)}
                  placeholder="6자리 코드"
                  maxLength={6}
                  className="w-full px-4 py-2.5 bg-gray-800 border border-gray-700 rounded-lg text-sm focus:border-indigo-500 focus:outline-none font-mono tracking-widest"
                />
                <p className="text-xs text-gray-500 mt-1">
                  서버 로그에 표시된 코드를 입력하세요
                </p>
              </div>

              {setupError && (
                <p className="text-sm text-red-400 bg-red-500/10 px-3 py-2 rounded-lg">
                  {setupError}
                </p>
              )}

              <div className="flex gap-3 pt-2">
                <button
                  type="button"
                  onClick={() => setShowSetup(false)}
                  className="flex-1 px-4 py-2.5 border border-gray-700 rounded-lg text-sm text-gray-400 hover:text-white hover:border-gray-500 transition-colors"
                >
                  취소
                </button>
                <button
                  type="submit"
                  disabled={setupLoading}
                  className="flex-1 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
                >
                  {setupLoading ? "연결 중..." : "연결"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-3xl mx-auto px-4 py-6">
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-[60vh] text-center">
              <div className="w-16 h-16 bg-gradient-to-br from-indigo-600 to-purple-700 rounded-2xl flex items-center justify-center text-2xl font-bold mb-5">
                M
              </div>
              <h2 className="text-2xl font-semibold mb-2">MoA에 오신 것을 환영합니다</h2>
              <p className="text-gray-500 text-sm mb-1">
                무엇이든 물어보세요. AI가 도와드립니다.
              </p>
              <p className="text-gray-600 text-xs">
                Ask anything. Your AI agent is ready.
              </p>
            </div>
          ) : (
            messages.map((msg) => (
              <div
                key={msg.id}
                className={`flex gap-3 mb-6 ${msg.role === "user" ? "justify-end" : "justify-start"}`}
              >
                {msg.role === "assistant" && (
                  <div className="w-8 h-8 bg-gradient-to-br from-indigo-600 to-purple-700 rounded-lg flex items-center justify-center text-xs font-bold shrink-0 mt-0.5">
                    M
                  </div>
                )}
                <div
                  className={`max-w-[75%] ${msg.role === "user" ? "" : ""}`}
                >
                  <div
                    className={`px-4 py-3 rounded-2xl text-sm leading-relaxed ${
                      msg.role === "user"
                        ? "bg-indigo-600 text-white rounded-br-md"
                        : "bg-gray-900 border border-gray-800 text-gray-200 rounded-bl-md"
                    }`}
                  >
                    {msg.role === "assistant" ? (
                      <div
                        className="prose-chat"
                        dangerouslySetInnerHTML={{
                          __html: renderMarkdown(msg.content),
                        }}
                      />
                    ) : (
                      <p className="whitespace-pre-wrap">{msg.content}</p>
                    )}
                  </div>
                  {msg.model && (
                    <p className="text-[10px] text-gray-600 mt-1 px-1">
                      {msg.model}
                    </p>
                  )}
                </div>
              </div>
            ))
          )}

          {loading && (
            <div className="flex gap-3 mb-6">
              <div className="w-8 h-8 bg-gradient-to-br from-indigo-600 to-purple-700 rounded-lg flex items-center justify-center text-xs font-bold shrink-0">
                M
              </div>
              <div className="px-4 py-3 bg-gray-900 border border-gray-800 rounded-2xl rounded-bl-md">
                <div className="flex gap-1.5">
                  <span className="w-2 h-2 bg-gray-600 rounded-full animate-bounce" style={{ animationDelay: "0s" }} />
                  <span className="w-2 h-2 bg-gray-600 rounded-full animate-bounce" style={{ animationDelay: "0.15s" }} />
                  <span className="w-2 h-2 bg-gray-600 rounded-full animate-bounce" style={{ animationDelay: "0.3s" }} />
                </div>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* Input */}
      <div className="border-t border-gray-800 bg-gray-900/80 backdrop-blur p-4 shrink-0">
        <div className="max-w-3xl mx-auto flex items-end gap-3">
          <div className="flex-1 flex items-end bg-gray-800 border border-gray-700 rounded-xl px-4 py-1 focus-within:border-indigo-500 transition-colors">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="메시지를 입력하세요..."
              rows={1}
              className="flex-1 bg-transparent text-sm text-white py-2.5 resize-none focus:outline-none max-h-32"
              style={{ minHeight: "20px" }}
            />
          </div>
          <button
            onClick={sendMessage}
            disabled={!input.trim() || loading}
            className="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-800 disabled:text-gray-600 rounded-xl text-sm font-medium transition-colors shrink-0"
          >
            전송
          </button>
        </div>
        <p className="text-center text-[10px] text-gray-600 mt-2">
          Enter로 전송 · Shift+Enter로 줄바꿈
        </p>
      </div>
    </div>
  );
}
