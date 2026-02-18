import Link from "next/link";
import Header from "@/components/Header";
import Footer from "@/components/Footer";

const features = [
  {
    icon: "🤖",
    title: "다중 AI 모델 지원",
    titleEn: "Multi-Model Support",
    desc: "OpenRouter, Anthropic, OpenAI 등 수십 개 모델을 하나의 인터페이스로 사용하세요.",
  },
  {
    icon: "🔒",
    title: "엔드투엔드 암호화",
    titleEn: "End-to-End Encryption",
    desc: "AES-256-GCM과 ChaCha20으로 모든 데이터를 안전하게 보호합니다.",
  },
  {
    icon: "💬",
    title: "멀티채널 통합",
    titleEn: "Multi-Channel Integration",
    desc: "Telegram, Discord, Slack, KakaoTalk, WhatsApp 등을 하나로 연결합니다.",
  },
  {
    icon: "🧠",
    title: "학습형 메모리",
    titleEn: "Adaptive Memory",
    desc: "대화를 기억하고 학습하여 점점 더 나은 응답을 제공합니다.",
  },
  {
    icon: "🔧",
    title: "도구 실행",
    titleEn: "Tool Execution",
    desc: "파일 관리, 웹 검색, 브라우저 자동화 등 강력한 도구를 직접 실행합니다.",
  },
  {
    icon: "📱",
    title: "크로스 플랫폼",
    titleEn: "Cross-Platform",
    desc: "Windows, macOS, Linux, Android, iOS, Web 모든 곳에서 사용 가능합니다.",
  },
];

const pricing = [
  {
    name: "Free",
    nameKo: "무료",
    price: "$0",
    period: "/월",
    features: [
      "하루 50회 메시지",
      "기본 AI 모델",
      "웹 채팅",
      "1개 채널 연결",
    ],
    cta: "무료로 시작",
    highlight: false,
  },
  {
    name: "Pro",
    nameKo: "프로",
    price: "$19",
    period: "/월",
    features: [
      "무제한 메시지",
      "모든 AI 모델",
      "데스크탑 + 모바일 앱",
      "무제한 채널 연결",
      "학습형 메모리",
      "우선 지원",
    ],
    cta: "프로 시작하기",
    highlight: true,
  },
  {
    name: "Enterprise",
    nameKo: "엔터프라이즈",
    price: "문의",
    period: "",
    features: [
      "프로의 모든 기능",
      "셀프 호스팅",
      "커스텀 모델 연동",
      "SLA 보장",
      "전담 기술 지원",
      "온프레미스 배포",
    ],
    cta: "문의하기",
    highlight: false,
  },
];

export default function HomePage() {
  return (
    <div className="min-h-screen bg-gray-950 text-white">
      <Header />

      {/* Hero Section */}
      <section className="relative overflow-hidden pt-32 pb-20 px-4">
        <div className="absolute inset-0 bg-gradient-to-br from-indigo-950/50 via-gray-950 to-purple-950/30" />
        <div className="absolute top-20 left-1/2 -translate-x-1/2 w-[600px] h-[600px] bg-indigo-500/10 rounded-full blur-3xl" />

        <div className="relative max-w-4xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-sm mb-8">
            <span className="w-2 h-2 bg-indigo-400 rounded-full animate-pulse" />
            Powered by ZeroClaw Engine
          </div>

          <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-6">
            <span className="bg-gradient-to-r from-indigo-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
              MoA
            </span>
            <br />
            <span className="text-3xl md:text-4xl text-gray-300 font-medium">
              Master of AI
            </span>
          </h1>

          <p className="text-lg md:text-xl text-gray-400 max-w-2xl mx-auto mb-4">
            당신만의 자율 AI 에이전트. 대화하고, 명령하고, 자동화하세요.
          </p>
          <p className="text-sm text-gray-500 max-w-xl mx-auto mb-10">
            Your autonomous AI agent. Chat, command, and automate — on every
            platform.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/chat"
              className="px-8 py-3.5 bg-indigo-600 hover:bg-indigo-500 rounded-xl font-semibold text-base transition-colors w-full sm:w-auto"
            >
              웹에서 바로 시작 →
            </Link>
            <Link
              href="/download"
              className="px-8 py-3.5 border border-gray-700 hover:border-gray-500 rounded-xl font-semibold text-base text-gray-300 hover:text-white transition-colors w-full sm:w-auto"
            >
              앱 다운로드
            </Link>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 px-4" id="features">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              강력한 기능
            </h2>
            <p className="text-gray-400 text-lg">
              AI 에이전트가 할 수 있는 모든 것
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((f, i) => (
              <div
                key={i}
                className="p-6 rounded-2xl bg-gray-900/50 border border-gray-800 hover:border-indigo-500/30 transition-colors group"
              >
                <div className="text-3xl mb-4">{f.icon}</div>
                <h3 className="text-lg font-semibold mb-1 group-hover:text-indigo-400 transition-colors">
                  {f.title}
                </h3>
                <p className="text-xs text-gray-500 mb-3">{f.titleEn}</p>
                <p className="text-gray-400 text-sm leading-relaxed">
                  {f.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section className="py-20 px-4 bg-gray-900/30" id="pricing">
        <div className="max-w-5xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">요금제</h2>
            <p className="text-gray-400 text-lg">
              누구나 시작할 수 있는 무료 플랜
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {pricing.map((p, i) => (
              <div
                key={i}
                className={`p-6 rounded-2xl border ${
                  p.highlight
                    ? "bg-indigo-950/30 border-indigo-500/40 ring-1 ring-indigo-500/20"
                    : "bg-gray-900/50 border-gray-800"
                }`}
              >
                {p.highlight && (
                  <div className="text-xs font-semibold text-indigo-400 mb-4 uppercase tracking-wider">
                    Most Popular
                  </div>
                )}
                <h3 className="text-lg font-semibold mb-1">{p.nameKo}</h3>
                <p className="text-xs text-gray-500 mb-4">{p.name}</p>
                <div className="flex items-baseline gap-1 mb-6">
                  <span className="text-3xl font-bold">{p.price}</span>
                  <span className="text-gray-500 text-sm">{p.period}</span>
                </div>
                <ul className="space-y-3 mb-8">
                  {p.features.map((feat, j) => (
                    <li
                      key={j}
                      className="flex items-center gap-2 text-sm text-gray-300"
                    >
                      <span className="text-indigo-400 text-xs">✓</span>
                      {feat}
                    </li>
                  ))}
                </ul>
                <button
                  className={`w-full py-2.5 rounded-lg font-medium text-sm transition-colors ${
                    p.highlight
                      ? "bg-indigo-600 hover:bg-indigo-500 text-white"
                      : "border border-gray-700 hover:border-gray-500 text-gray-300 hover:text-white"
                  }`}
                >
                  {p.cta}
                </button>
              </div>
            ))}
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
