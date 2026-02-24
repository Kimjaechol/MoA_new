import { useState } from "react";
import { t, type Locale } from "../lib/i18n";
import type { ChatSession } from "../lib/storage";
import type { DeviceInfo, ToolInfo } from "../lib/api";

interface SidebarProps {
  chats: ChatSession[];
  activeChatId: string | null;
  isOpen: boolean;
  locale: Locale;
  currentPage: string;
  devices: DeviceInfo[];
  channels: string[];
  tools: ToolInfo[];
  onNewChat: () => void;
  onSelectChat: (id: string) => void;
  onDeleteChat: (id: string) => void;
  onOpenSettings: () => void;
  onToggle: () => void;
}

export function Sidebar({
  chats,
  activeChatId,
  isOpen,
  locale,
  currentPage,
  devices,
  channels,
  tools,
  onNewChat,
  onSelectChat,
  onDeleteChat,
  onOpenSettings,
  onToggle,
}: SidebarProps) {
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    devices: true,
    channels: false,
    tools: false,
    chats: true,
  });

  const toggleSection = (key: string) => {
    setExpandedSections((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const handleDelete = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    onDeleteChat(id);
  };

  const onlineDevices = devices.filter((d) => d.is_online);

  return (
    <>
      <aside className={`sidebar ${isOpen ? "" : "closed"}`}>
        {/* Logo and New Chat */}
        <div className="sidebar-header">
          <div className="sidebar-logo">
            <div className="sidebar-logo-icon">MoA</div>
            <span className="sidebar-logo-text">{t("app_title", locale)}</span>
          </div>
          <button
            className="sidebar-new-chat-btn"
            onClick={onNewChat}
            title={t("new_chat", locale)}
          >
            +
          </button>
        </div>

        {/* Scrollable body with sections */}
        <div className="sidebar-body">

          {/* Devices section */}
          <div className="sidebar-section">
            <button
              className="sidebar-section-header"
              onClick={() => toggleSection("devices")}
            >
              <div className="sidebar-section-header-left">
                <svg className="sidebar-section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                  <line x1="8" y1="21" x2="16" y2="21" />
                  <line x1="12" y1="17" x2="12" y2="21" />
                </svg>
                <span>{t("sidebar_devices", locale)}</span>
                {onlineDevices.length > 0 && (
                  <span className="sidebar-section-badge">{onlineDevices.length}</span>
                )}
              </div>
              <svg
                className={`sidebar-section-chevron ${expandedSections.devices ? "expanded" : ""}`}
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
            {expandedSections.devices && (
              <div className="sidebar-section-content">
                {devices.length === 0 ? (
                  <div className="sidebar-section-empty">{t("sidebar_no_devices", locale)}</div>
                ) : (
                  devices.map((device) => (
                    <div key={device.device_id} className="sidebar-info-item">
                      <div className={`sidebar-status-dot ${device.is_online ? "online" : ""}`} />
                      <span className="sidebar-info-label">{device.device_name}</span>
                      {device.platform && (
                        <span className="sidebar-info-meta">{device.platform}</span>
                      )}
                    </div>
                  ))
                )}
              </div>
            )}
          </div>

          {/* Channels section */}
          <div className="sidebar-section">
            <button
              className="sidebar-section-header"
              onClick={() => toggleSection("channels")}
            >
              <div className="sidebar-section-header-left">
                <svg className="sidebar-section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                </svg>
                <span>{t("sidebar_channels", locale)}</span>
                {channels.length > 0 && (
                  <span className="sidebar-section-badge">{channels.length}</span>
                )}
              </div>
              <svg
                className={`sidebar-section-chevron ${expandedSections.channels ? "expanded" : ""}`}
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
            {expandedSections.channels && (
              <div className="sidebar-section-content">
                {channels.length === 0 ? (
                  <div className="sidebar-section-empty">{t("sidebar_no_channels", locale)}</div>
                ) : (
                  channels.map((ch) => (
                    <div key={ch} className="sidebar-info-item">
                      <div className="sidebar-status-dot online" />
                      <span className="sidebar-info-label">{ch}</span>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>

          {/* Tools section */}
          <div className="sidebar-section">
            <button
              className="sidebar-section-header"
              onClick={() => toggleSection("tools")}
            >
              <div className="sidebar-section-header-left">
                <svg className="sidebar-section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
                </svg>
                <span>{t("sidebar_tools", locale)}</span>
                {tools.length > 0 && (
                  <span className="sidebar-section-badge">{tools.length}</span>
                )}
              </div>
              <svg
                className={`sidebar-section-chevron ${expandedSections.tools ? "expanded" : ""}`}
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
            {expandedSections.tools && (
              <div className="sidebar-section-content">
                {tools.length === 0 ? (
                  <div className="sidebar-section-empty">{t("sidebar_no_tools", locale)}</div>
                ) : (
                  tools.map((tool) => (
                    <div key={tool.name} className="sidebar-info-item" title={tool.description}>
                      <span className="sidebar-tool-icon">
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <polyline points="4 17 10 11 4 5" />
                          <line x1="12" y1="19" x2="20" y2="19" />
                        </svg>
                      </span>
                      <span className="sidebar-info-label">{tool.name}</span>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>

          {/* Chats section */}
          <div className="sidebar-section sidebar-section-chats">
            <button
              className="sidebar-section-header"
              onClick={() => toggleSection("chats")}
            >
              <div className="sidebar-section-header-left">
                <svg className="sidebar-section-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" />
                </svg>
                <span>{t("sidebar_chats", locale)}</span>
                {chats.length > 0 && (
                  <span className="sidebar-section-badge">{chats.length}</span>
                )}
              </div>
              <svg
                className={`sidebar-section-chevron ${expandedSections.chats ? "expanded" : ""}`}
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
            {expandedSections.chats && (
              <div className="sidebar-section-content sidebar-chats-list">
                {chats.length === 0 ? (
                  <div className="sidebar-section-empty">{t("no_chats", locale)}</div>
                ) : (
                  chats.map((chat) => (
                    <div
                      key={chat.id}
                      className={`sidebar-chat-item ${chat.id === activeChatId ? "active" : ""}`}
                      onClick={() => onSelectChat(chat.id)}
                    >
                      <span className="sidebar-chat-title">{chat.title}</span>
                      <button
                        className="sidebar-chat-delete"
                        onClick={(e) => handleDelete(e, chat.id)}
                        title={t("delete_chat", locale)}
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <polyline points="3 6 5 6 21 6" />
                          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                      </button>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="sidebar-footer">
          <button
            className={`sidebar-footer-btn ${currentPage === "settings" ? "active" : ""}`}
            onClick={onOpenSettings}
          >
            <span className="icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
            </span>
            {t("settings", locale)}
          </button>
        </div>
      </aside>

      {/* Mobile overlay */}
      {isOpen && <div className="sidebar-overlay" onClick={onToggle} />}
    </>
  );
}
