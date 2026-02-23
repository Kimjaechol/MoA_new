"use client";

import { useState } from "react";
import {
  type SubAgent,
  type Domain,
  getLLMById,
  getToolById,
} from "@/lib/domains";

interface AgentInfoPanelProps {
  domain: Domain;
  subAgent: SubAgent;
  selectedLLM: string | null;
  selectedTool: string | null;
  onSelectLLM: (llmId: string) => void;
  onSelectTool: (toolId: string) => void;
  onClose: () => void;
}

export default function AgentInfoPanel({
  domain,
  subAgent,
  selectedLLM,
  selectedTool,
  onSelectLLM,
  onSelectTool,
  onClose,
}: AgentInfoPanelProps) {
  const [activeTab, setActiveTab] = useState<"llm" | "tool">("llm");

  const recommendedLLMs = subAgent.recommendedLLMs.map(getLLMById).filter(Boolean);
  const alternativeLLMs = subAgent.alternativeLLMs.map(getLLMById).filter(Boolean);
  const recommendedTools = subAgent.recommendedTools.map(getToolById).filter(Boolean);
  const alternativeTools = subAgent.alternativeTools.map(getToolById).filter(Boolean);

  return (
    <div className="border-l border-dark-800/50 w-80 flex flex-col bg-dark-950/95 animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-dark-800/50">
        <div>
          <h3 className="text-sm font-semibold text-dark-100">{subAgent.nameKo}</h3>
          <p className="text-[10px] text-dark-500">{domain.nameKo} &middot; {subAgent.name}</p>
        </div>
        <button
          onClick={onClose}
          className="flex h-7 w-7 items-center justify-center rounded-md text-dark-500 hover:bg-dark-800 hover:text-dark-300 transition-all"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Description */}
      <div className="px-4 py-3 border-b border-dark-800/50">
        <p className="text-xs text-dark-300 leading-relaxed">{subAgent.descriptionKo}</p>
        <p className="text-[10px] text-dark-500 mt-1">{subAgent.description}</p>
      </div>

      {/* Tab switcher */}
      <div className="flex border-b border-dark-800/50">
        <button
          onClick={() => setActiveTab("llm")}
          className={`flex-1 px-4 py-2 text-xs font-medium transition-all ${
            activeTab === "llm"
              ? "text-primary-400 border-b-2 border-primary-400 bg-primary-500/5"
              : "text-dark-500 hover:text-dark-300"
          }`}
        >
          LLM ({recommendedLLMs.length + alternativeLLMs.length})
        </button>
        <button
          onClick={() => setActiveTab("tool")}
          className={`flex-1 px-4 py-2 text-xs font-medium transition-all ${
            activeTab === "tool"
              ? "text-accent-400 border-b-2 border-accent-400 bg-accent-500/5"
              : "text-dark-500 hover:text-dark-300"
          }`}
        >
          Tool ({recommendedTools.length + alternativeTools.length})
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto custom-scrollbar p-3 space-y-3">
        {activeTab === "llm" && (
          <>
            {/* Recommended LLMs */}
            {recommendedLLMs.length > 0 && (
              <div>
                <div className="flex items-center gap-1.5 mb-2">
                  <div className="h-1.5 w-1.5 rounded-full bg-green-400" />
                  <span className="text-[10px] font-semibold text-dark-400 uppercase tracking-wider">
                    Recommended
                  </span>
                </div>
                <div className="space-y-1.5">
                  {recommendedLLMs.map((model) => model && (
                    <button
                      key={model.id}
                      onClick={() => onSelectLLM(model.id)}
                      className={`w-full text-left rounded-lg p-3 transition-all border ${
                        selectedLLM === model.id
                          ? "bg-primary-500/10 border-primary-500/30"
                          : "bg-dark-800/30 border-dark-700/50 hover:bg-dark-800/60"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={`text-xs font-medium ${selectedLLM === model.id ? "text-primary-300" : "text-dark-200"}`}>
                          {model.name}
                        </span>
                        <span className={`text-[9px] px-1.5 py-0.5 rounded-full ${
                          model.tier === "free" ? "bg-green-500/10 text-green-400"
                            : model.tier === "pro" ? "bg-primary-500/10 text-primary-400"
                              : "bg-secondary-500/10 text-secondary-400"
                        }`}>{model.tier}</span>
                      </div>
                      <p className="text-[10px] text-dark-500">{model.provider}</p>
                      <p className="text-[10px] text-dark-400 mt-1">{model.descriptionKo}</p>
                      <div className="flex flex-wrap gap-1 mt-1.5">
                        {model.strengths.slice(0, 3).map((s) => (
                          <span key={s} className="text-[9px] px-1.5 py-0.5 rounded bg-dark-700/50 text-dark-400">
                            {s}
                          </span>
                        ))}
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Alternative LLMs */}
            {alternativeLLMs.length > 0 && (
              <div>
                <div className="flex items-center gap-1.5 mb-2">
                  <div className="h-1.5 w-1.5 rounded-full bg-dark-500" />
                  <span className="text-[10px] font-semibold text-dark-500 uppercase tracking-wider">
                    Alternative
                  </span>
                </div>
                <div className="space-y-1.5">
                  {alternativeLLMs.map((model) => model && (
                    <button
                      key={model.id}
                      onClick={() => onSelectLLM(model.id)}
                      className={`w-full text-left rounded-lg p-3 transition-all border ${
                        selectedLLM === model.id
                          ? "bg-primary-500/10 border-primary-500/30"
                          : "bg-dark-800/20 border-dark-800/50 hover:bg-dark-800/40"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={`text-xs font-medium ${selectedLLM === model.id ? "text-primary-300" : "text-dark-300"}`}>
                          {model.name}
                        </span>
                        <span className={`text-[9px] px-1.5 py-0.5 rounded-full ${
                          model.tier === "free" ? "bg-green-500/10 text-green-400"
                            : model.tier === "pro" ? "bg-primary-500/10 text-primary-400"
                              : "bg-secondary-500/10 text-secondary-400"
                        }`}>{model.tier}</span>
                      </div>
                      <p className="text-[10px] text-dark-500">{model.provider} - {model.descriptionKo}</p>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </>
        )}

        {activeTab === "tool" && (
          <>
            {/* Recommended Tools */}
            {recommendedTools.length > 0 && (
              <div>
                <div className="flex items-center gap-1.5 mb-2">
                  <div className="h-1.5 w-1.5 rounded-full bg-green-400" />
                  <span className="text-[10px] font-semibold text-dark-400 uppercase tracking-wider">
                    Recommended
                  </span>
                </div>
                <div className="space-y-1.5">
                  {recommendedTools.map((tool) => tool && (
                    <button
                      key={tool.id}
                      onClick={() => onSelectTool(tool.id)}
                      className={`w-full text-left rounded-lg p-3 transition-all border ${
                        selectedTool === tool.id
                          ? "bg-accent-500/10 border-accent-500/30"
                          : "bg-dark-800/30 border-dark-700/50 hover:bg-dark-800/60"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={`text-xs font-medium ${selectedTool === tool.id ? "text-accent-300" : "text-dark-200"}`}>
                          {tool.name}
                        </span>
                        <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-dark-700 text-dark-400">
                          {tool.apiType}
                        </span>
                      </div>
                      <p className="text-[10px] text-dark-400 mt-0.5">{tool.descriptionKo}</p>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Alternative Tools */}
            {alternativeTools.length > 0 && (
              <div>
                <div className="flex items-center gap-1.5 mb-2">
                  <div className="h-1.5 w-1.5 rounded-full bg-dark-500" />
                  <span className="text-[10px] font-semibold text-dark-500 uppercase tracking-wider">
                    Alternative
                  </span>
                </div>
                <div className="space-y-1.5">
                  {alternativeTools.map((tool) => tool && (
                    <button
                      key={tool.id}
                      onClick={() => onSelectTool(tool.id)}
                      className={`w-full text-left rounded-lg p-3 transition-all border ${
                        selectedTool === tool.id
                          ? "bg-accent-500/10 border-accent-500/30"
                          : "bg-dark-800/20 border-dark-800/50 hover:bg-dark-800/40"
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={`text-xs font-medium ${selectedTool === tool.id ? "text-accent-300" : "text-dark-300"}`}>
                          {tool.name}
                        </span>
                        <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-dark-700 text-dark-400">
                          {tool.apiType}
                        </span>
                      </div>
                      <p className="text-[10px] text-dark-500">{tool.descriptionKo}</p>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
