"use client";

import ChatWidget from "@/components/ChatWidget";

export default function ChatPage() {
  return (
    <div className="flex h-screen flex-col pt-[65px]">
      <ChatWidget className="flex-1 relative" />
    </div>
  );
}
