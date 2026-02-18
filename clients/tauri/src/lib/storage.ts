const STORAGE_KEY_CHATS = "moa_chats";
const STORAGE_KEY_ACTIVE_CHAT = "moa_active_chat";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "error";
  content: string;
  model?: string;
  timestamp: number;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  createdAt: number;
  updatedAt: number;
}

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substring(2, 8);
}

export function loadChats(): ChatSession[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_CHATS);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed;
  } catch {
    return [];
  }
}

export function saveChats(chats: ChatSession[]): void {
  localStorage.setItem(STORAGE_KEY_CHATS, JSON.stringify(chats));
}

export function getActiveChatId(): string | null {
  return localStorage.getItem(STORAGE_KEY_ACTIVE_CHAT);
}

export function setActiveChatId(id: string | null): void {
  if (id) {
    localStorage.setItem(STORAGE_KEY_ACTIVE_CHAT, id);
  } else {
    localStorage.removeItem(STORAGE_KEY_ACTIVE_CHAT);
  }
}

export function createNewChat(): ChatSession {
  return {
    id: generateId(),
    title: "New Chat",
    messages: [],
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };
}

export function createMessage(role: ChatMessage["role"], content: string, model?: string): ChatMessage {
  return {
    id: generateId(),
    role,
    content,
    model,
    timestamp: Date.now(),
  };
}

export function deriveChatTitle(messages: ChatMessage[]): string {
  const firstUserMsg = messages.find((m) => m.role === "user");
  if (!firstUserMsg) return "New Chat";
  const text = firstUserMsg.content.trim();
  if (text.length <= 40) return text;
  return text.substring(0, 37) + "...";
}
