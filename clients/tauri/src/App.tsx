import { useState, useCallback, useEffect } from "react";
import { Chat } from "./components/Chat";
import { Sidebar } from "./components/Sidebar";
import { Settings } from "./components/Settings";
import { apiClient } from "./lib/api";
import { getStoredLocale, setStoredLocale, type Locale } from "./lib/i18n";
import {
  loadChats,
  saveChats,
  getActiveChatId,
  setActiveChatId,
  createNewChat,
  createMessage,
  deriveChatTitle,
  type ChatSession,
  type ChatMessage,
} from "./lib/storage";

type Page = "chat" | "settings";

function App() {
  const [page, setPage] = useState<Page>("chat");
  const [locale, setLocale] = useState<Locale>(getStoredLocale());
  const [chats, setChats] = useState<ChatSession[]>(() => loadChats());
  const [activeChatId, setActiveChatIdState] = useState<string | null>(() => getActiveChatId());
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [isConnected, setIsConnected] = useState(apiClient.isConnected());

  const activeChat = chats.find((c) => c.id === activeChatId) ?? null;

  useEffect(() => {
    saveChats(chats);
  }, [chats]);

  useEffect(() => {
    setActiveChatId(activeChatId);
  }, [activeChatId]);

  const handleLocaleChange = useCallback((newLocale: Locale) => {
    setLocale(newLocale);
    setStoredLocale(newLocale);
  }, []);

  const handleNewChat = useCallback(() => {
    const chat = createNewChat();
    setChats((prev) => [chat, ...prev]);
    setActiveChatIdState(chat.id);
    setPage("chat");
  }, []);

  const handleSelectChat = useCallback((id: string) => {
    setActiveChatIdState(id);
    setPage("chat");
  }, []);

  const handleDeleteChat = useCallback(
    (id: string) => {
      setChats((prev) => prev.filter((c) => c.id !== id));
      if (activeChatId === id) {
        setActiveChatIdState(null);
      }
    },
    [activeChatId],
  );

  const handleSendMessage = useCallback(
    async (content: string) => {
      let chatId = activeChatId;
      let isNew = false;

      if (!chatId) {
        const chat = createNewChat();
        chatId = chat.id;
        isNew = true;
        setChats((prev) => [chat, ...prev]);
        setActiveChatIdState(chatId);
      }

      const userMsg = createMessage("user", content);

      setChats((prev) =>
        prev.map((c) => {
          if (c.id !== chatId) return c;
          const updated = {
            ...c,
            messages: [...(isNew ? [] : c.messages), userMsg],
            updatedAt: Date.now(),
          };
          if (updated.messages.length === 1) {
            updated.title = deriveChatTitle(updated.messages);
          }
          return updated;
        }),
      );

      try {
        const response = await apiClient.chat(content);
        const assistantMsg = createMessage("assistant", response.response, response.model);

        setChats((prev) =>
          prev.map((c) => {
            if (c.id !== chatId) return c;
            return {
              ...c,
              messages: [...c.messages, assistantMsg],
              updatedAt: Date.now(),
            };
          }),
        );
      } catch (err) {
        const errorMsg = createMessage(
          "error",
          err instanceof Error ? err.message : "An unknown error occurred",
        );

        setChats((prev) =>
          prev.map((c) => {
            if (c.id !== chatId) return c;
            return {
              ...c,
              messages: [...c.messages, errorMsg],
              updatedAt: Date.now(),
            };
          }),
        );

        if (err instanceof Error && err.message.includes("Authentication expired")) {
          setIsConnected(false);
        }
      }
    },
    [activeChatId],
  );

  const handleRetry = useCallback(
    (messages: ChatMessage[]) => {
      const lastUserIdx = [...messages].reverse().findIndex((m) => m.role === "user");
      if (lastUserIdx === -1) return;

      const actualIdx = messages.length - 1 - lastUserIdx;
      const lastUserMsg = messages[actualIdx];

      setChats((prev) =>
        prev.map((c) => {
          if (c.id !== activeChatId) return c;
          return {
            ...c,
            messages: messages.slice(0, actualIdx),
            updatedAt: Date.now(),
          };
        }),
      );

      handleSendMessage(lastUserMsg.content);
    },
    [activeChatId, handleSendMessage],
  );

  const handleConnectionChange = useCallback((connected: boolean) => {
    setIsConnected(connected);
  }, []);

  return (
    <div className="app">
      <Sidebar
        chats={chats}
        activeChatId={activeChatId}
        isOpen={sidebarOpen}
        locale={locale}
        onNewChat={handleNewChat}
        onSelectChat={handleSelectChat}
        onDeleteChat={handleDeleteChat}
        onOpenSettings={() => setPage("settings")}
        onToggle={() => setSidebarOpen((p) => !p)}
        currentPage={page}
      />
      <main className={`main-content ${sidebarOpen ? "" : "sidebar-collapsed"}`}>
        {page === "chat" ? (
          <Chat
            chat={activeChat}
            locale={locale}
            isConnected={isConnected}
            onSendMessage={handleSendMessage}
            onRetry={handleRetry}
            onOpenSettings={() => setPage("settings")}
            onToggleSidebar={() => setSidebarOpen((p) => !p)}
            sidebarOpen={sidebarOpen}
          />
        ) : (
          <Settings
            locale={locale}
            onLocaleChange={handleLocaleChange}
            onConnectionChange={handleConnectionChange}
            onBack={() => setPage("chat")}
          />
        )}
      </main>
    </div>
  );
}

export default App;
