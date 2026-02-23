"use client";

import { useState } from "react";
import Link from "next/link";
import {
  llmModels,
  toolAPIs,
  channels,
  type LLMModel,
  type ToolAPI,
  type ChannelOption,
  type SubAgent,
  getDomainById,
  getSubAgentById,
  getAllLLMsForSubAgent,
  getAllToolsForSubAgent,
} from "@/lib/domains";

interface AgentSidebarProps {
  selectedDomain: string | null;
  selectedSubAgent: string | null;
  selectedLLM: string | null;
  selectedTool: string | null;
  selectedChannel: string | null;
  onSelectLLM: (llmId: string) => void;
  onSelectTool: (toolId: string) => void;
  onSelectChannel: (channelId: string) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

function SidebarDropdown<T extends { id: string; name: string }>({
  label,
  labelKo,
  icon,
  items,
  selectedId,
  onSelect,
  renderItem,
  collapsed,
}: {
  label: string;
  labelKo: string;
  icon: JSX.Element;
  items: T[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  renderItem: (item: T) => JSX.Element;
  collapsed: boolean;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const selected = items.find((i) => i.id === selectedId);

  if (collapsed) {
    return (
      <div className="relative group">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex h-10 w-10 items-center justify-center rounded-lg text-dark-400 hover:bg-dark-800 hover:text-dark-200 transition-all"
          title={labelKo}
        >
          {icon}
        </button>
        {/* Tooltip */}
        <div className="absolute left-12 top-1/2 -translate-y-1/2 hidden group-hover:block z-50">
          <div className="rounded-lg bg-dark-800 border border-dark-700 px-3 py-1.5 text-xs text-dark-200 whitespace-nowrap shadow-lg">
            {labelKo}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="px-3">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm hover:bg-dark-800/50 transition-all"
      >
        <div className="flex items-center gap-2">
          <span className="text-dark-500">{icon}</span>
          <div className="text-left">
            <span className="text-xs font-medium text-dark-300">{labelKo}</span>
            <span className="text-[10px] text-dark-600 ml-1">{label}</span>
          </div>
        </div>
        <svg
          className={`h-3.5 w-3.5 text-dark-500 transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={2}
          stroke="currentColor"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
        </svg>
      </button>

      {/* Selected value badge */}
      {selected && !isOpen && (
        <div className="mx-3 mt-1 mb-1">
          <div className="flex items-center gap-1.5 rounded-md bg-primary-500/10 border border-primary-500/20 px-2.5 py-1.5">
            <div className="h-1.5 w-1.5 rounded-full bg-primary-400" />
            <span className="text-xs text-primary-300 truncate">{selected.name}</span>
          </div>
        </div>
      )}

      {/* Dropdown */}
      {isOpen && (
        <div className="mx-1 mt-1 rounded-lg border border-dark-700 bg-dark-900 shadow-lg max-h-60 overflow-y-auto custom-scrollbar animate-fade-in">
          {items.length === 0 ? (
            <div className="px-3 py-4 text-center text-xs text-dark-500">
              에이전트를 먼저 선택하세요
            </div>
          ) : (
            items.map((item) => (
              <button
                key={item.id}
                onClick={() => {
                  onSelect(item.id);
                  setIsOpen(false);
                }}
                className={`w-full text-left px-3 py-2 text-xs transition-all ${
                  selectedId === item.id
                    ? "bg-primary-500/10 text-primary-300"
                    : "text-dark-300 hover:bg-dark-800/80 hover:text-dark-200"
                }`}
              >
                {renderItem(item)}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export default function AgentSidebar({
  selectedDomain,
  selectedSubAgent,
  selectedLLM,
  selectedTool,
  selectedChannel,
  onSelectLLM,
  onSelectTool,
  onSelectChannel,
  collapsed,
  onToggleCollapse,
}: AgentSidebarProps) {
  // Get contextual items based on selected sub-agent
  let contextLLMs: LLMModel[] = llmModels;
  let contextTools: ToolAPI[] = toolAPIs;

  if (selectedDomain && selectedSubAgent) {
    const subAgent = getSubAgentById(selectedDomain, selectedSubAgent);
    if (subAgent) {
      contextLLMs = getAllLLMsForSubAgent(subAgent);
      contextTools = getAllToolsForSubAgent(subAgent);
    }
  }

  // Get active sub-agent info
  let activeSubAgent: SubAgent | undefined;
  let activeDomainName = "";
  if (selectedDomain && selectedSubAgent) {
    const domain = getDomainById(selectedDomain);
    activeSubAgent = getSubAgentById(selectedDomain, selectedSubAgent);
    activeDomainName = domain?.nameKo || "";
  }

  const llmIcon = (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456z" />
    </svg>
  );

  const toolIcon = (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M11.42 15.17l-5.1 5.1a2.121 2.121 0 11-3-3l5.1-5.1m0 0L15.17 4.93a2.121 2.121 0 013 3l-7.75 7.24z" />
    </svg>
  );

  const channelIcon = (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M8.625 12a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375M21 12c0 4.556-4.03 8.25-9 8.25a9.764 9.764 0 01-2.555-.337A5.972 5.972 0 015.41 20.97a5.969 5.969 0 01-.474-.065 4.48 4.48 0 00.978-2.025c.09-.457-.133-.901-.467-1.226C3.93 16.178 3 14.189 3 12c0-4.556 4.03-8.25 9-8.25s9 3.694 9 8.25z" />
    </svg>
  );

  return (
    <aside
      className={`flex flex-col border-r border-dark-800/50 bg-dark-950/95 transition-all duration-300 ${
        collapsed ? "w-14" : "w-64"
      }`}
    >
      {/* Collapse toggle */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-dark-800/50">
        {!collapsed && (
          <span className="text-xs font-semibold text-dark-400 uppercase tracking-wider">
            Agent Config
          </span>
        )}
        <button
          onClick={onToggleCollapse}
          className="flex h-7 w-7 items-center justify-center rounded-md text-dark-500 hover:bg-dark-800 hover:text-dark-300 transition-all"
          title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          <svg
            className={`h-3.5 w-3.5 transition-transform duration-300 ${collapsed ? "rotate-180" : ""}`}
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={2}
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
          </svg>
        </button>
      </div>

      {/* Active agent info */}
      {!collapsed && activeSubAgent && (
        <div className="px-3 py-3 border-b border-dark-800/50">
          <div className="rounded-lg bg-dark-800/50 border border-dark-700/50 px-3 py-2.5">
            <div className="text-[10px] text-dark-500 uppercase tracking-wider mb-1">Active Agent</div>
            <div className="text-sm font-medium text-dark-100">{activeSubAgent.nameKo}</div>
            <div className="text-[10px] text-dark-500 mt-0.5">{activeDomainName}</div>
          </div>
        </div>
      )}

      {/* Dropdowns */}
      <div className="flex-1 overflow-y-auto custom-scrollbar py-2 space-y-3">
        {/* LLM Selection */}
        <SidebarDropdown<LLMModel>
          label="LLM Model"
          labelKo="LLM 모델"
          icon={llmIcon}
          items={contextLLMs}
          selectedId={selectedLLM}
          onSelect={onSelectLLM}
          collapsed={collapsed}
          renderItem={(model) => (
            <div>
              <div className="flex items-center justify-between">
                <span className="font-medium">{model.name}</span>
                <span className={`text-[9px] px-1.5 py-0.5 rounded-full ${
                  model.tier === "free"
                    ? "bg-green-500/10 text-green-400"
                    : model.tier === "pro"
                      ? "bg-primary-500/10 text-primary-400"
                      : "bg-secondary-500/10 text-secondary-400"
                }`}>
                  {model.tier}
                </span>
              </div>
              <div className="text-[10px] text-dark-500 mt-0.5">{model.provider} - {model.descriptionKo}</div>
            </div>
          )}
        />

        {/* Tool (API) Selection */}
        <SidebarDropdown<ToolAPI>
          label="Tool (API)"
          labelKo="도구 (API)"
          icon={toolIcon}
          items={contextTools}
          selectedId={selectedTool}
          onSelect={onSelectTool}
          collapsed={collapsed}
          renderItem={(tool) => (
            <div>
              <div className="flex items-center justify-between">
                <span className="font-medium">{tool.name}</span>
                <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-dark-700 text-dark-400">
                  {tool.apiType}
                </span>
              </div>
              <div className="text-[10px] text-dark-500 mt-0.5">{tool.descriptionKo}</div>
            </div>
          )}
        />

        {/* Channel Selection */}
        <SidebarDropdown<ChannelOption>
          label="Channel"
          labelKo="채널"
          icon={channelIcon}
          items={channels}
          selectedId={selectedChannel}
          onSelect={onSelectChannel}
          collapsed={collapsed}
          renderItem={(ch) => (
            <div>
              <span className="font-medium">{ch.name}</span>
              <div className="text-[10px] text-dark-500 mt-0.5">{ch.description}</div>
            </div>
          )}
        />
      </div>

      {/* Bottom menu items */}
      <div className={`border-t border-dark-800/50 py-2 ${collapsed ? "px-1" : "px-3"} space-y-1`}>
        {/* Payment */}
        <Link
          href="/workspace/payment"
          className={`flex items-center gap-2 rounded-lg transition-all text-dark-400 hover:bg-dark-800/50 hover:text-dark-200 ${
            collapsed ? "h-10 w-10 justify-center" : "px-3 py-2"
          }`}
          title="결제"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 8.25h19.5M2.25 9h19.5m-16.5 5.25h6m-6 2.25h3m-3.75 3h15a2.25 2.25 0 002.25-2.25V6.75A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25v10.5A2.25 2.25 0 004.5 19.5z" />
          </svg>
          {!collapsed && <span className="text-xs font-medium">결제 Payment</span>}
        </Link>

        {/* My Page */}
        <Link
          href="/workspace/mypage"
          className={`flex items-center gap-2 rounded-lg transition-all text-dark-400 hover:bg-dark-800/50 hover:text-dark-200 ${
            collapsed ? "h-10 w-10 justify-center" : "px-3 py-2"
          }`}
          title="마이페이지"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" />
          </svg>
          {!collapsed && <span className="text-xs font-medium">마이페이지 My Page</span>}
        </Link>
      </div>
    </aside>
  );
}
