"use client";

import Link from "next/link";

const plans = [
  {
    id: "free",
    name: "Free",
    nameKo: "무료",
    price: "₩0",
    period: "영원히 무료",
    features: [
      { text: "기본 LLM 1개 (Claude Haiku / Gemini Flash / GPT-4.1 Mini)", included: true },
      { text: "웹 채팅 무제한", included: true },
      { text: "메모리 100MB", included: true },
      { text: "브라우저 도구 (월 100회)", included: true },
      { text: "이미지 생성 (월 20회)", included: true },
      { text: "음악 생성", included: false },
      { text: "비디오 생성", included: false },
      { text: "멀티채널 연동", included: false },
    ],
    current: true,
  },
  {
    id: "pro",
    name: "Pro",
    nameKo: "프로",
    price: "₩9,900",
    period: "/ 월",
    features: [
      { text: "모든 LLM 모델 (15개+)", included: true },
      { text: "모든 채널 연동 (18개+)", included: true },
      { text: "메모리 10GB", included: true },
      { text: "브라우저 도구 무제한", included: true },
      { text: "이미지 생성 (월 500회)", included: true },
      { text: "음악 생성 (월 100곡)", included: true },
      { text: "비디오 생성 (월 50편)", included: true },
      { text: "실시간 통역", included: true },
      { text: "API 크레딧 ₩50,000 포함", included: true },
      { text: "우선 지원", included: true },
    ],
    highlighted: true,
    current: false,
  },
  {
    id: "enterprise",
    name: "Enterprise",
    nameKo: "엔터프라이즈",
    price: "문의",
    period: "",
    features: [
      { text: "모든 Pro 기능", included: true },
      { text: "전용 서버 / GPU 할당", included: true },
      { text: "무제한 메모리 & 생성", included: true },
      { text: "SLA 보장 (99.9%)", included: true },
      { text: "전담 매니저", included: true },
      { text: "커스텀 LLM 파인튜닝", included: true },
      { text: "온프레미스 배포 지원", included: true },
      { text: "감사 로그 & 규정 준수", included: true },
    ],
    current: false,
  },
];

const creditPacks = [
  { name: "API 크레딧 5만", price: "₩5,000", credits: "₩50,000", description: "LLM API 호출, 이미지/음악/비디오 생성에 사용" },
  { name: "API 크레딧 20만", price: "₩18,000", credits: "₩200,000", description: "10% 할인, 헤비 유저용", badge: "인기" },
  { name: "API 크레딧 100만", price: "₩80,000", credits: "₩1,000,000", description: "20% 할인, 팀/기업용", badge: "최고 할인" },
];

export default function PaymentPage() {
  return (
    <div className="min-h-screen pt-[65px]">
      <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6 lg:px-8">
        {/* Back link */}
        <Link
          href="/workspace"
          className="inline-flex items-center gap-1.5 text-sm text-dark-400 hover:text-dark-200 transition-all mb-8"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
          </svg>
          워크스페이스로 돌아가기
        </Link>

        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-3xl font-bold text-dark-50 mb-3">
            결제 <span className="text-dark-500 text-xl font-normal">Payment</span>
          </h1>
          <p className="text-dark-400 max-w-lg mx-auto">
            무료로 시작하고, 필요에 따라 업그레이드하세요. 모든 도메인 에이전트와 도구를 자유롭게 사용하세요.
          </p>
        </div>

        {/* Plans */}
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 mb-16">
          {plans.map((plan) => (
            <div
              key={plan.id}
              className={`relative rounded-2xl p-6 transition-all duration-300 ${
                plan.highlighted
                  ? "glass-card border-primary-500/30 bg-primary-500/5 scale-[1.02]"
                  : "glass-card"
              }`}
            >
              {plan.highlighted && (
                <div className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-primary-500 px-4 py-1 text-xs font-semibold text-white">
                  인기 Popular
                </div>
              )}

              <div className="mb-6">
                <h3 className="text-lg font-semibold text-dark-50">
                  {plan.nameKo} <span className="text-dark-500 text-sm font-normal">{plan.name}</span>
                </h3>
                <div className="mt-3 flex items-baseline gap-1">
                  <span className="text-3xl font-bold text-dark-50">{plan.price}</span>
                  <span className="text-sm text-dark-500">{plan.period}</span>
                </div>
              </div>

              <ul className="space-y-2.5 mb-8">
                {plan.features.map((feature) => (
                  <li key={feature.text} className="flex items-start gap-2">
                    {feature.included ? (
                      <svg className="mt-0.5 h-4 w-4 flex-shrink-0 text-green-400" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                      </svg>
                    ) : (
                      <svg className="mt-0.5 h-4 w-4 flex-shrink-0 text-dark-600" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    )}
                    <span className={`text-sm ${feature.included ? "text-dark-300" : "text-dark-600"}`}>
                      {feature.text}
                    </span>
                  </li>
                ))}
              </ul>

              <button
                className={`block w-full text-center rounded-lg px-6 py-3 text-sm font-semibold transition-all ${
                  plan.current
                    ? "border border-dark-600 bg-dark-800 text-dark-400 cursor-default"
                    : plan.highlighted
                      ? "bg-primary-500 text-white hover:bg-primary-600"
                      : "border border-dark-600 bg-dark-800 text-dark-200 hover:border-dark-500 hover:bg-dark-700"
                }`}
                disabled={plan.current}
              >
                {plan.current ? "현재 플랜 Current Plan" : plan.id === "enterprise" ? "영업팀 문의 Contact Sales" : "업그레이드 Upgrade"}
              </button>
            </div>
          ))}
        </div>

        {/* Credit Packs */}
        <div className="mb-12">
          <div className="text-center mb-8">
            <h2 className="text-2xl font-bold text-dark-50 mb-2">
              API 크레딧 충전 <span className="text-dark-500 text-lg font-normal">Credit Packs</span>
            </h2>
            <p className="text-sm text-dark-400">
              LLM 호출, Seedance, Suno, Freepik 등 외부 API 사용량을 크레딧으로 관리합니다.
            </p>
          </div>

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3 max-w-3xl mx-auto">
            {creditPacks.map((pack) => (
              <div key={pack.name} className="relative glass-card rounded-xl p-5">
                {pack.badge && (
                  <div className="absolute -top-2.5 right-4 rounded-full bg-accent-500 px-3 py-0.5 text-[10px] font-semibold text-white">
                    {pack.badge}
                  </div>
                )}
                <h4 className="text-sm font-semibold text-dark-100 mb-1">{pack.name}</h4>
                <div className="flex items-baseline gap-1 mb-2">
                  <span className="text-xl font-bold text-dark-50">{pack.price}</span>
                  <span className="text-xs text-dark-500">({pack.credits} 크레딧)</span>
                </div>
                <p className="text-[11px] text-dark-500 mb-4">{pack.description}</p>
                <button className="w-full rounded-lg border border-dark-600 bg-dark-800 px-4 py-2 text-xs font-semibold text-dark-200 hover:border-dark-500 hover:bg-dark-700 transition-all">
                  구매하기 Purchase
                </button>
              </div>
            ))}
          </div>
        </div>

        {/* Resource Usage */}
        <div className="glass-card rounded-2xl p-6 max-w-3xl mx-auto">
          <h3 className="text-lg font-semibold text-dark-100 mb-4">
            리소스 사용량 <span className="text-dark-500 text-sm font-normal">Resource Usage</span>
          </h3>
          <div className="space-y-4">
            {[
              { label: "API 크레딧", used: 12500, total: 50000, unit: "₩", color: "primary" },
              { label: "이미지 생성", used: 8, total: 20, unit: "회", color: "secondary" },
              { label: "메모리 사용", used: 23, total: 100, unit: "MB", color: "accent" },
            ].map((res) => (
              <div key={res.label}>
                <div className="flex items-center justify-between mb-1.5">
                  <span className="text-xs text-dark-300">{res.label}</span>
                  <span className="text-xs text-dark-400">
                    {res.used.toLocaleString()}{res.unit} / {res.total.toLocaleString()}{res.unit}
                  </span>
                </div>
                <div className="h-2 rounded-full bg-dark-800 overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-500 ${
                      res.color === "primary" ? "bg-primary-500"
                        : res.color === "secondary" ? "bg-secondary-500"
                          : "bg-accent-500"
                    }`}
                    style={{ width: `${(res.used / res.total) * 100}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
