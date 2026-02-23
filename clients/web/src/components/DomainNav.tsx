"use client";

import { useState, useRef, useEffect } from "react";
import { domains, type Domain, type SubAgent } from "@/lib/domains";

interface DomainNavProps {
  selectedDomain: string | null;
  selectedSubAgent: string | null;
  onSelectSubAgent: (domainId: string, subAgentId: string) => void;
}

const domainIcons: Record<string, JSX.Element> = {
  "shopping-cart": (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 3h1.386c.51 0 .955.343 1.087.835l.383 1.437M7.5 14.25a3 3 0 00-3 3h15.75m-12.75-3h11.218c1.121-2.3 2.1-4.684 2.924-7.138a60.114 60.114 0 00-16.536-1.84M7.5 14.25L5.106 5.272M6 20.25a.75.75 0 11-1.5 0 .75.75 0 011.5 0zm12.75 0a.75.75 0 11-1.5 0 .75.75 0 011.5 0z" />
    </svg>
  ),
  calendar: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5" />
    </svg>
  ),
  "file-text": (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
    </svg>
  ),
  code: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
    </svg>
  ),
  languages: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 21l5.25-11.25L21 21m-9-3h7.5M3 5.621a48.474 48.474 0 016-.371m0 0c1.12 0 2.233.038 3.334.114M9 5.25V3m3.334 2.364C11.176 10.658 7.69 15.08 3 17.502m9.334-12.138c.896.061 1.785.147 2.666.257m-4.589 8.495a18.023 18.023 0 01-3.827-5.802" />
    </svg>
  ),
  music: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
    </svg>
  ),
  image: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5a2.25 2.25 0 002.25-2.25V5.25a2.25 2.25 0 00-2.25-2.25H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
    </svg>
  ),
  video: (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z" />
    </svg>
  ),
};

const colorClasses: Record<string, { bg: string; text: string; border: string; hoverBg: string }> = {
  primary: {
    bg: "bg-primary-500/10",
    text: "text-primary-400",
    border: "border-primary-500/20",
    hoverBg: "hover:bg-primary-500/20",
  },
  secondary: {
    bg: "bg-secondary-500/10",
    text: "text-secondary-400",
    border: "border-secondary-500/20",
    hoverBg: "hover:bg-secondary-500/20",
  },
  accent: {
    bg: "bg-accent-500/10",
    text: "text-accent-400",
    border: "border-accent-500/20",
    hoverBg: "hover:bg-accent-500/20",
  },
};

export default function DomainNav({
  selectedDomain,
  selectedSubAgent,
  onSelectSubAgent,
}: DomainNavProps) {
  const [openDomain, setOpenDomain] = useState<string | null>(null);
  const navRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (navRef.current && !navRef.current.contains(e.target as Node)) {
        setOpenDomain(null);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  function handleDomainClick(domainId: string) {
    setOpenDomain(openDomain === domainId ? null : domainId);
  }

  function handleSubAgentClick(domain: Domain, subAgent: SubAgent) {
    onSelectSubAgent(domain.id, subAgent.id);
    setOpenDomain(null);
  }

  return (
    <div ref={navRef} className="relative border-b border-dark-800/50 bg-dark-950/90 backdrop-blur-sm">
      <div className="flex items-center gap-1 px-3 py-2 overflow-x-auto custom-scrollbar">
        {domains.map((domain) => {
          const colors = colorClasses[domain.color] || colorClasses.primary;
          const isSelected = selectedDomain === domain.id;
          const isOpen = openDomain === domain.id;

          return (
            <div key={domain.id} className="relative flex-shrink-0">
              <button
                onClick={() => handleDomainClick(domain.id)}
                className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all duration-200 ${
                  isSelected
                    ? `${colors.bg} ${colors.text} ${colors.border} border`
                    : isOpen
                      ? `bg-dark-800/80 text-dark-200 border border-dark-600`
                      : `text-dark-400 hover:text-dark-200 hover:bg-dark-800/50 border border-transparent`
                }`}
              >
                <span className={isSelected ? colors.text : "text-dark-500"}>
                  {domainIcons[domain.icon]}
                </span>
                <span>{domain.nameKo}</span>
                <svg
                  className={`h-3 w-3 transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
                  fill="none"
                  viewBox="0 0 24 24"
                  strokeWidth={2}
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
                </svg>
              </button>

              {/* Dropdown mega-menu */}
              {isOpen && (
                <div className="absolute left-0 top-full z-50 mt-1 w-80 rounded-xl border border-dark-700 bg-dark-900 shadow-2xl animate-fade-in">
                  <div className="p-3">
                    <div className="flex items-center gap-2 mb-3 pb-2 border-b border-dark-800">
                      <span className={colors.text}>
                        {domainIcons[domain.icon]}
                      </span>
                      <span className="text-sm font-semibold text-dark-100">{domain.nameKo}</span>
                      <span className="text-xs text-dark-500">{domain.name}</span>
                    </div>
                    <div className="space-y-1">
                      {domain.subAgents.map((sub) => {
                        const isSubSelected = selectedDomain === domain.id && selectedSubAgent === sub.id;
                        return (
                          <button
                            key={sub.id}
                            onClick={() => handleSubAgentClick(domain, sub)}
                            className={`w-full text-left rounded-lg px-3 py-2.5 transition-all duration-150 ${
                              isSubSelected
                                ? `${colors.bg} ${colors.border} border`
                                : `hover:bg-dark-800/80 border border-transparent`
                            }`}
                          >
                            <div className="flex items-center justify-between">
                              <span className={`text-sm font-medium ${isSubSelected ? colors.text : "text-dark-200"}`}>
                                {sub.nameKo}
                              </span>
                              {isSubSelected && (
                                <svg className={`h-3.5 w-3.5 ${colors.text}`} fill="none" viewBox="0 0 24 24" strokeWidth={2.5} stroke="currentColor">
                                  <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                                </svg>
                              )}
                            </div>
                            <p className="text-[11px] text-dark-500 mt-0.5 leading-relaxed">{sub.descriptionKo}</p>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
