"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import Header from "@/components/Header";
import Footer from "@/components/Footer";

const R2_BASE =
  process.env.NEXT_PUBLIC_R2_BASE_URL || "https://downloads.moa.example.com";

interface Platform {
  id: string;
  name: string;
  icon: string;
  arch: string;
  files: { label: string; path: string; size: string }[];
  instructions: string[];
}

const platforms: Platform[] = [
  {
    id: "windows",
    name: "Windows",
    icon: "🪟",
    arch: "x64 (64-bit)",
    files: [
      {
        label: "MSI Installer",
        path: "/releases/latest/MoA-windows-x64.msi",
        size: "~25 MB",
      },
      {
        label: "EXE Installer",
        path: "/releases/latest/MoA-windows-x64-setup.exe",
        size: "~25 MB",
      },
    ],
    instructions: [
      "MSI 또는 EXE 파일을 다운로드합니다.",
      "다운로드한 파일을 실행합니다.",
      "설치 마법사의 안내를 따릅니다.",
      "설치 완료 후 시작 메뉴에서 MoA를 찾아 실행합니다.",
    ],
  },
  {
    id: "macos",
    name: "macOS",
    icon: "🍎",
    arch: "Universal (Intel + Apple Silicon)",
    files: [
      {
        label: "DMG (Universal)",
        path: "/releases/latest/MoA-macos-universal.dmg",
        size: "~30 MB",
      },
    ],
    instructions: [
      "DMG 파일을 다운로드합니다.",
      "DMG를 열고 MoA 아이콘을 Applications 폴더로 드래그합니다.",
      "처음 실행시 '개발자를 확인할 수 없습니다' 메시지가 뜨면:",
      "시스템 설정 → 개인정보 보호 및 보안 → '확인 없이 열기' 클릭",
    ],
  },
  {
    id: "linux",
    name: "Linux",
    icon: "🐧",
    arch: "x86_64",
    files: [
      {
        label: "AppImage",
        path: "/releases/latest/MoA-linux-x86_64.AppImage",
        size: "~30 MB",
      },
      {
        label: "DEB (Ubuntu/Debian)",
        path: "/releases/latest/moa_amd64.deb",
        size: "~20 MB",
      },
    ],
    instructions: [
      "AppImage: 다운로드 후 chmod +x MoA*.AppImage && ./MoA*.AppImage",
      "DEB: sudo dpkg -i moa_amd64.deb",
      "AppImage는 설치 없이 바로 실행 가능합니다.",
    ],
  },
  {
    id: "android",
    name: "Android",
    icon: "🤖",
    arch: "ARM64 / x86_64",
    files: [
      {
        label: "APK Direct Download",
        path: "/releases/latest/MoA-android.apk",
        size: "~20 MB",
      },
    ],
    instructions: [
      "APK 파일을 다운로드합니다.",
      "설정 → 보안 → 알 수 없는 출처 허용을 활성화합니다.",
      "다운로드한 APK를 탭하여 설치합니다.",
      "Google Play 출시 예정",
    ],
  },
  {
    id: "ios",
    name: "iOS",
    icon: "📱",
    arch: "ARM64",
    files: [],
    instructions: [
      "App Store에서 'MoA AI' 검색 (출시 예정)",
      "TestFlight 베타 참여: 아래 링크 클릭",
    ],
  },
];

function detectPlatform(): string {
  if (typeof navigator === "undefined") return "windows";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("android")) return "android";
  if (ua.includes("iphone") || ua.includes("ipad")) return "ios";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux")) return "linux";
  return "windows";
}

export default function DownloadPage() {
  const [currentPlatform, setCurrentPlatform] = useState("windows");
  const [selected, setSelected] = useState<string | null>(null);

  useEffect(() => {
    const detected = detectPlatform();
    setCurrentPlatform(detected);
    setSelected(detected);
  }, []);

  const activePlatform = platforms.find((p) => p.id === selected) || platforms[0];

  return (
    <div className="min-h-screen bg-gray-950 text-white">
      <Header />

      <section className="pt-28 pb-12 px-4">
        <div className="max-w-4xl mx-auto text-center">
          <h1 className="text-4xl font-bold mb-3">앱 다운로드</h1>
          <p className="text-gray-400 text-lg mb-2">
            모든 기기에서 MoA를 사용하세요
          </p>
          <p className="text-gray-500 text-sm">
            Download MoA for your platform
          </p>
        </div>
      </section>

      {/* Platform selector */}
      <section className="px-4 pb-16">
        <div className="max-w-4xl mx-auto">
          <div className="flex flex-wrap justify-center gap-3 mb-10">
            {platforms.map((p) => (
              <button
                key={p.id}
                onClick={() => setSelected(p.id)}
                className={`flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-medium transition-all ${
                  selected === p.id
                    ? "bg-indigo-600 text-white shadow-lg shadow-indigo-500/20"
                    : "bg-gray-900 border border-gray-800 text-gray-400 hover:text-white hover:border-gray-600"
                } ${
                  p.id === currentPlatform && selected !== p.id
                    ? "ring-1 ring-indigo-500/30"
                    : ""
                }`}
              >
                <span>{p.icon}</span>
                {p.name}
                {p.id === currentPlatform && (
                  <span className="text-[10px] opacity-60">(현재 OS)</span>
                )}
              </button>
            ))}
          </div>

          {/* Download card */}
          <div className="bg-gray-900/50 border border-gray-800 rounded-2xl p-8">
            <div className="flex items-center gap-4 mb-6">
              <span className="text-4xl">{activePlatform.icon}</span>
              <div>
                <h2 className="text-xl font-semibold">
                  {activePlatform.name}
                </h2>
                <p className="text-sm text-gray-500">{activePlatform.arch}</p>
              </div>
            </div>

            {activePlatform.files.length > 0 ? (
              <div className="space-y-3 mb-8">
                {activePlatform.files.map((file, i) => (
                  <a
                    key={i}
                    href={`${R2_BASE}${file.path}`}
                    className="flex items-center justify-between p-4 bg-gray-800/50 border border-gray-700 rounded-xl hover:border-indigo-500/40 transition-colors group"
                    download
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-indigo-400 text-lg">⬇</span>
                      <div>
                        <p className="font-medium text-sm group-hover:text-indigo-400 transition-colors">
                          {file.label}
                        </p>
                        <p className="text-xs text-gray-500">{file.size}</p>
                      </div>
                    </div>
                    <span className="text-xs px-3 py-1.5 bg-indigo-600 rounded-lg font-medium">
                      다운로드
                    </span>
                  </a>
                ))}
              </div>
            ) : (
              <div className="p-6 bg-gray-800/30 rounded-xl text-center mb-8">
                <p className="text-gray-400 text-sm">곧 출시 예정입니다</p>
                <p className="text-gray-500 text-xs mt-1">Coming soon</p>
              </div>
            )}

            {/* Installation instructions */}
            <div>
              <h3 className="text-sm font-semibold text-gray-300 mb-3">
                설치 방법
              </h3>
              <ol className="space-y-2">
                {activePlatform.instructions.map((step, i) => (
                  <li
                    key={i}
                    className="flex gap-3 text-sm text-gray-400"
                  >
                    <span className="text-indigo-400 font-mono text-xs mt-0.5 shrink-0">
                      {i + 1}.
                    </span>
                    {step}
                  </li>
                ))}
              </ol>
            </div>
          </div>

          {/* System Requirements */}
          <div className="mt-10 p-6 bg-gray-900/30 border border-gray-800 rounded-2xl">
            <h3 className="text-sm font-semibold mb-4">
              시스템 요구사항
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm text-gray-400">
              <div>
                <p className="text-gray-300 font-medium mb-1">데스크탑</p>
                <ul className="space-y-1 text-xs">
                  <li>Windows 10+ / macOS 11+ / Ubuntu 20.04+</li>
                  <li>RAM 4GB 이상</li>
                  <li>저장공간 100MB</li>
                </ul>
              </div>
              <div>
                <p className="text-gray-300 font-medium mb-1">모바일</p>
                <ul className="space-y-1 text-xs">
                  <li>Android 8.0+ / iOS 15+</li>
                  <li>RAM 2GB 이상</li>
                  <li>저장공간 50MB</li>
                </ul>
              </div>
              <div>
                <p className="text-gray-300 font-medium mb-1">네트워크</p>
                <ul className="space-y-1 text-xs">
                  <li>인터넷 연결 필수</li>
                  <li>HTTPS 지원</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
