"use client";

import { useState, useCallback, useEffect } from "react";
import DomainNav from "@/components/DomainNav";
import AgentSidebar from "@/components/AgentSidebar";
import ChatWidget from "@/components/ChatWidget";
import {
  getDomainById,
  getSubAgentById,
  getLLMById,
  getToolById,
} from "@/lib/domains";

const STORAGE_KEY_WORKSPACE = "moa_workspace_state";

interface WorkspaceState {
  selectedDomain: string | null;
  selectedSubAgent: string | null;
  selectedLLM: string | null;
  selectedTool: string | null;
  selectedChannel: string | null;
  sidebarCollapsed: boolean;
}

function loadWorkspaceState(): WorkspaceState {
  if (typeof window === "undefined") {
    return {
      selectedDomain: null,
      selectedSubAgent: null,
      selectedLLM: null,
      selectedTool: null,
      selectedChannel: "web",
      sidebarCollapsed: false,
    };
  }
  try {
    const stored = localStorage.getItem(STORAGE_KEY_WORKSPACE);
    if (stored) return JSON.parse(stored);
  } catch {
    // ignore
  }
  return {
    selectedDomain: null,
    selectedSubAgent: null,
    selectedLLM: null,
    selectedTool: null,
    selectedChannel: "web",
    sidebarCollapsed: false,
  };
}

function saveWorkspaceState(state: WorkspaceState): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY_WORKSPACE, JSON.stringify(state));
  } catch {
    // ignore
  }
}

export default function WorkspacePage() {
  const [state, setState] = useState<WorkspaceState>(loadWorkspaceState);

  useEffect(() => {
    saveWorkspaceState(state);
  }, [state]);

  const handleSelectSubAgent = useCallback((domainId: string, subAgentId: string) => {
    const subAgent = getSubAgentById(domainId, subAgentId);
    setState((prev) => ({
      ...prev,
      selectedDomain: domainId,
      selectedSubAgent: subAgentId,
      // Auto-select recommended LLM and tool
      selectedLLM: subAgent?.recommendedLLMs[0] || prev.selectedLLM,
      selectedTool: subAgent?.recommendedTools[0] || prev.selectedTool,
    }));
  }, []);

  const handleSelectLLM = useCallback((llmId: string) => {
    setState((prev) => ({ ...prev, selectedLLM: llmId }));
  }, []);

  const handleSelectTool = useCallback((toolId: string) => {
    setState((prev) => ({ ...prev, selectedTool: toolId }));
  }, []);

  const handleSelectChannel = useCallback((channelId: string) => {
    setState((prev) => ({ ...prev, selectedChannel: channelId }));
  }, []);

  const handleToggleCollapse = useCallback(() => {
    setState((prev) => ({ ...prev, sidebarCollapsed: !prev.sidebarCollapsed }));
  }, []);

  // Build context info for display
  const domain = state.selectedDomain ? getDomainById(state.selectedDomain) : null;
  const subAgent = state.selectedDomain && state.selectedSubAgent
    ? getSubAgentById(state.selectedDomain, state.selectedSubAgent)
    : null;
  const llm = state.selectedLLM ? getLLMById(state.selectedLLM) : null;
  const tool = state.selectedTool ? getToolById(state.selectedTool) : null;

  return (
    <div className="flex h-screen flex-col pt-[65px]">
      {/* Top: Domain Navigation */}
      <DomainNav
        selectedDomain={state.selectedDomain}
        selectedSubAgent={state.selectedSubAgent}
        onSelectSubAgent={handleSelectSubAgent}
      />

      {/* Main area: Sidebar + Chat */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left: Agent Sidebar */}
        <AgentSidebar
          selectedDomain={state.selectedDomain}
          selectedSubAgent={state.selectedSubAgent}
          selectedLLM={state.selectedLLM}
          selectedTool={state.selectedTool}
          selectedChannel={state.selectedChannel}
          onSelectLLM={handleSelectLLM}
          onSelectTool={handleSelectTool}
          onSelectChannel={handleSelectChannel}
          collapsed={state.sidebarCollapsed}
          onToggleCollapse={handleToggleCollapse}
        />

        {/* Center: Chat area */}
        <div className="flex flex-1 flex-col overflow-hidden">
          {/* Active config bar */}
          <div className="flex items-center gap-2 border-b border-dark-800/50 px-4 py-2 bg-dark-950/50 overflow-x-auto">
            {subAgent ? (
              <>
                <span className="text-[10px] text-dark-500 uppercase tracking-wider flex-shrink-0">Active:</span>
                <div className="flex items-center gap-1.5 flex-shrink-0">
                  <span className="text-xs font-medium text-dark-200">{subAgent.nameKo}</span>
                </div>
                <span className="text-dark-700">|</span>
                {llm && (
                  <div className="flex items-center gap-1 flex-shrink-0">
                    <svg className="h-3 w-3 text-primary-400" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09z" />
                    </svg>
                    <span className="text-xs text-primary-300">{llm.name}</span>
                  </div>
                )}
                {tool && (
                  <>
                    <span className="text-dark-700">|</span>
                    <div className="flex items-center gap-1 flex-shrink-0">
                      <svg className="h-3 w-3 text-accent-400" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M11.42 15.17l-5.1 5.1a2.121 2.121 0 11-3-3l5.1-5.1m0 0L15.17 4.93a2.121 2.121 0 013 3l-7.75 7.24z" />
                      </svg>
                      <span className="text-xs text-accent-300">{tool.name}</span>
                    </div>
                  </>
                )}
              </>
            ) : (
              <div className="flex items-center gap-2">
                <svg className="h-3.5 w-3.5 text-dark-500" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
                </svg>
                <span className="text-xs text-dark-500">
                  상단 메뉴에서 도메인과 에이전트를 선택하세요
                </span>
              </div>
            )}
          </div>

          {/* Chat Widget or Welcome */}
          {subAgent ? (
            <ChatWidget className="flex-1 relative" />
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-center max-w-lg px-8">
                <div className="flex h-20 w-20 mx-auto items-center justify-center rounded-2xl bg-primary-500/10 border border-primary-500/20 mb-6">
                  <svg className="h-10 w-10 text-primary-400" fill="none" viewBox="0 0 24 24" strokeWidth={1} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456z" />
                  </svg>
                </div>

                <h2 className="text-2xl font-bold text-dark-100 mb-3">
                  MoA Agent Workspace
                </h2>
                <p className="text-sm text-dark-400 mb-6 leading-relaxed">
                  상단 카테고리에서 도메인을 선택하고, 원하는 서브 에이전트를 클릭하세요.
                  <br />
                  좌측 사이드바에서 LLM 모델과 도구(API)를 선택할 수 있습니다.
                </p>

                {/* Quick domain cards */}
                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                  {[
                    { icon: "shopping-cart", label: "웹/쇼핑", labelEn: "Shopping" },
                    { icon: "calendar", label: "일상/비서", labelEn: "Assistant" },
                    { icon: "code", label: "코딩/개발", labelEn: "Coding" },
                    { icon: "video", label: "비디오", labelEn: "Video" },
                  ].map((item) => (
                    <div
                      key={item.labelEn}
                      className="glass-card rounded-xl p-4 text-center transition-all hover:translate-y-[-2px] cursor-default"
                    >
                      <div className="flex h-10 w-10 mx-auto items-center justify-center rounded-lg bg-primary-500/10 border border-primary-500/20 mb-2 text-primary-400">
                        {item.icon === "shopping-cart" && (
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 3h1.386c.51 0 .955.343 1.087.835l.383 1.437M7.5 14.25a3 3 0 00-3 3h15.75m-12.75-3h11.218c1.121-2.3 2.1-4.684 2.924-7.138a60.114 60.114 0 00-16.536-1.84M7.5 14.25L5.106 5.272M6 20.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm12.75 0a.75.75 0 11-1.5 0 .75.75 0 011.5 0z" />
                          </svg>
                        )}
                        {item.icon === "calendar" && (
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5" />
                          </svg>
                        )}
                        {item.icon === "code" && (
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
                          </svg>
                        )}
                        {item.icon === "video" && (
                          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z" />
                          </svg>
                        )}
                      </div>
                      <div className="text-xs font-medium text-dark-200">{item.label}</div>
                      <div className="text-[10px] text-dark-500">{item.labelEn}</div>
                    </div>
                  ))}
                </div>

                <p className="text-[10px] text-dark-600 mt-6">
                  MoA Orchestrator manages resources, priorities, and schedules across all domain agents.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
