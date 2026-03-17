const STORAGE_KEY_CHATS = "zeroclaw_chats";
const STORAGE_KEY_ACTIVE_CHAT = "zeroclaw_active_chat";

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

  let text = firstUserMsg.content.trim();

  // Strip attachment metadata lines (e.g. "[첨부: file.hwp]" or "[Attached: file.pdf]")
  text = text.replace(/\n*\[(첨부|Attached):.*\]\s*$/i, "").trim();

  // If nothing left after stripping attachments, show the attachment info
  if (!text) {
    const attachMatch = firstUserMsg.content.match(/\[(첨부|Attached):\s*(.+)\]/i);
    if (attachMatch) return `📎 ${attachMatch[2].substring(0, 30)}`;
    return "New Chat";
  }

  // Truncate for display
  if (text.length <= 40) return text;
  return text.substring(0, 37) + "...";
}
