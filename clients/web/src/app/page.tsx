import Link from "next/link";
import Footer from "@/components/Footer";

const features = [
  {
    icon: "\uD83E\uDD16",
    title: "\uB2E4\uC911 AI \uBAA8\uB378 \uC9C0\uC6D0",
    titleEn: "Multi-Model Support",
    description:
      "OpenAI, Anthropic, Google Gemini, Ollama \uB4F1 \uB2E4\uC591\uD55C AI \uBAA8\uB378\uC744 \uD558\uB098\uC758 \uC778\uD130\uD398\uC774\uC2A4\uC5D0\uC11C \uC790\uC720\uB86D\uAC8C \uC804\uD658\uD558\uC5EC \uC0AC\uC6A9\uD560 \uC218 \uC788\uC2B5\uB2C8\uB2E4.",
    descriptionEn:
      "Switch between OpenAI, Anthropic, Gemini, Ollama, and more from a unified interface.",
  },
  {
    icon: "\uD83D\uDD12",
    title: "\uC5D4\uB4DC\uD22C\uC5D4\uB4DC \uC554\uD638\uD654",
    titleEn: "End-to-End Encryption",
    description:
      "\uD398\uC5B4\uB9C1 \uAE30\uBC18 \uC778\uC99D\uACFC \uBCF4\uC548 \uD1B5\uC2E0\uC73C\uB85C \uB300\uD654 \uB0B4\uC6A9\uC744 \uC548\uC804\uD558\uAC8C \uBCF4\uD638\uD569\uB2C8\uB2E4.",
    descriptionEn:
      "Pairing-based authentication and secure communication protect your conversations.",
  },
  {
    icon: "\uD83D\uDCAC",
    title: "\uBA40\uD2F0\uCC44\uB110 \uD1B5\uD569",
    titleEn: "Multi-Channel Integration",
    description:
      "Telegram, Discord, Slack, \uC6F9 \uB4F1 \uB2E4\uC591\uD55C \uCC44\uB110\uC5D0\uC11C \uB3D9\uC2DC\uC5D0 AI \uC5D0\uC774\uC804\uD2B8\uC640 \uB300\uD654\uD560 \uC218 \uC788\uC2B5\uB2C8\uB2E4.",
    descriptionEn:
      "Chat with your AI agent across Telegram, Discord, Slack, Web, and more simultaneously.",
  },
  {
    icon: "\uD83E\uDDE0",
    title: "\uD559\uC2B5\uD615 \uBA54\uBAA8\uB9AC",
    titleEn: "Adaptive Memory",
    description:
      "\uB300\uD654 \uB9E5\uB77D\uACFC \uC120\uD638\uB3C4\uB97C \uD559\uC2B5\uD558\uC5EC \uC810\uC810 \uB354 \uB611\uB611\uD558\uACE0 \uAC1C\uC778\uD654\uB41C \uC751\uB2F5\uC744 \uC81C\uACF5\uD569\uB2C8\uB2E4.",
    descriptionEn:
      "Learns context and preferences to deliver increasingly personalized responses.",
  },
  {
    icon: "\uD83D\uDD27",
    title: "\uB3C4\uAD6C \uC2E4\uD589",
    titleEn: "Tool Execution",
    description:
      "\uD30C\uC77C \uAD00\uB9AC, \uC258 \uBA85\uB839\uC5B4, \uC6F9 \uBE0C\uB77C\uC6B0\uC9D5, \uBA54\uBAA8\uB9AC \uAD00\uB9AC \uB4F1 \uB2E4\uC591\uD55C \uB3C4\uAD6C\uB97C \uC790\uC728\uC801\uC73C\uB85C \uC2E4\uD589\uD569\uB2C8\uB2E4.",
    descriptionEn:
      "Autonomously execute file operations, shell commands, web browsing, and memory tools.",
  },
  {
    icon: "\uD83D\uDCF1",
    title: "\uD06C\uB85C\uC2A4 \uD50C\uB7AB\uD3FC",
    titleEn: "Cross-Platform",
    description:
      "Windows, macOS, Linux, Android, iOS, \uC6F9 \uBE0C\uB77C\uC6B0\uC800 \uB4F1 \uBAA8\uB4E0 \uD50C\uB7AB\uD3FC\uC5D0\uC11C \uC0AC\uC6A9 \uAC00\uB2A5\uD569\uB2C8\uB2E4.",
    descriptionEn:
      "Available on Windows, macOS, Linux, Android, iOS, and Web browsers.",
  },
];

const pricingTiers = [
  {
    name: "Free",
    nameKo: "\uBB34\uB8CC",
    price: "\u20A90",
    priceNote: "\uC601\uC6D0\uD788 \uBB34\uB8CC forever free",
    features: [
      "\uAE30\uBCF8 AI \uBAA8\uB378 1\uAC1C Basic model (1)",
      "\uC6F9 \uCC44\uD305 \uBB34\uC81C\uD55C Unlimited web chat",
      "\uBA54\uBAA8\uB9AC 100MB Memory 100MB",
      "\uCEE4\uBBA4\uB2C8\uD2F0 \uC9C0\uC6D0 Community support",
    ],
    cta: "\uBB34\uB8CC\uB85C \uC2DC\uC791",
    ctaEn: "Start Free",
    href: "/chat",
    highlighted: false,
  },
  {
    name: "Pro",
    nameKo: "\uD504\uB85C",
    price: "\u20A99,900",
    priceNote: "/ \uC6D4 per month",
    features: [
      "\uBAA8\uB4E0 AI \uBAA8\uB378 All AI models",
      "\uBA40\uD2F0\uCC44\uB110 \uD1B5\uD569 Multi-channel",
      "\uBA54\uBAA8\uB9AC 10GB Memory 10GB",
      "\uB3C4\uAD6C \uC2E4\uD589 Tool execution",
      "\uC6B0\uC120 \uC9C0\uC6D0 Priority support",
    ],
    cta: "\uD504\uB85C \uC2DC\uC791",
    ctaEn: "Start Pro",
    href: "/chat",
    highlighted: true,
  },
  {
    name: "Enterprise",
    nameKo: "\uC5D4\uD130\uD504\uB77C\uC774\uC988",
    price: "\uBB38\uC758",
    priceNote: "Contact us",
    features: [
      "\uBAA8\uB4E0 Pro \uAE30\uB2A5 All Pro features",
      "\uC804\uC6A9 \uC11C\uBC84 Dedicated server",
      "\uBB34\uC81C\uD55C \uBA54\uBAA8\uB9AC Unlimited memory",
      "SLA \uBCF4\uC7A5 SLA guarantee",
      "\uC804\uB2F4 \uB9E4\uB2C8\uC800 Dedicated manager",
      "\uCEE4\uC2A4\uD140 \uD1B5\uD569 Custom integration",
    ],
    cta: "\uC601\uC5C5\uD300 \uBB38\uC758",
    ctaEn: "Contact Sales",
    href: "mailto:enterprise@moa-agent.com",
    highlighted: false,
  },
];

export default function HomePage() {
  return (
    <>
      {/* Hero Section */}
      <section className="relative overflow-hidden pt-32 pb-20 sm:pt-40 sm:pb-28">
        {/* Background effects */}
        <div className="absolute inset-0 bg-hero-gradient" />
        <div className="absolute inset-0 bg-mesh-gradient" />
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[600px] bg-primary-500/5 rounded-full blur-3xl" />

        <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="text-center">
            {/* Badge */}
            <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary-500/20 bg-primary-500/5 px-4 py-1.5 animate-fade-in">
              <div className="h-1.5 w-1.5 rounded-full bg-primary-400 animate-pulse" />
              <span className="text-xs font-medium text-primary-300">
                Powered by ZeroClaw Runtime
              </span>
            </div>

            {/* Title */}
            <h1 className="text-4xl font-extrabold tracking-tight sm:text-6xl lg:text-7xl animate-fade-in-up">
              <span className="text-gradient">MoA</span>
              <span className="text-dark-50"> - Master of AI</span>
            </h1>

            {/* Subtitle */}
            <p className="mx-auto mt-6 max-w-2xl text-lg text-dark-300 leading-relaxed animate-fade-in-up sm:text-xl">
              {"\uB2F9\uC2E0\uC758 \uC790\uC728 AI \uC5D0\uC774\uC804\uD2B8."}
              <br />
              <span className="text-dark-400">
                {"\uB2E4\uC911 \uBAA8\uB378, \uBA40\uD2F0\uCC44\uB110, \uD559\uC2B5\uD615 \uBA54\uBAA8\uB9AC, \uB3C4\uAD6C \uC2E4\uD589\uC744 \uD558\uB098\uC758 \uC5D0\uC774\uC804\uD2B8\uC5D0\uC11C."}
              </span>
            </p>
            <p className="mx-auto mt-3 max-w-xl text-sm text-dark-500 animate-fade-in-up">
              Your autonomous AI agent. Multi-model, multi-channel, adaptive
              memory, and tool execution in one powerful agent.
            </p>

            {/* CTA Buttons */}
            <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row animate-fade-in-up">
              <Link href="/chat" className="btn-primary px-8 py-3.5 text-base glow-primary">
                {"\uC6F9\uC5D0\uC11C \uBC14\uB85C \uC2DC\uC791"}
                <svg
                  className="ml-2 h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  strokeWidth={2}
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
                  />
                </svg>
              </Link>
              <Link href="/download" className="btn-secondary px-8 py-3.5 text-base">
                {"\uC571 \uB2E4\uC6B4\uB85C\uB4DC"}
                <svg
                  className="ml-2 h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  strokeWidth={2}
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"
                  />
                </svg>
              </Link>
            </div>

            {/* Stats */}
            <div className="mt-16 grid grid-cols-2 gap-6 sm:grid-cols-4 animate-fade-in-up">
              {[
                { value: "6+", label: "AI \uBAA8\uB378", labelEn: "AI Models" },
                { value: "5+", label: "\uCC44\uB110", labelEn: "Channels" },
                { value: "6+", label: "\uD50C\uB7AB\uD3FC", labelEn: "Platforms" },
                { value: "100%", label: "\uC624\uD508\uC18C\uC2A4", labelEn: "Open Source" },
              ].map((stat) => (
                <div key={stat.labelEn} className="text-center">
                  <div className="text-2xl font-bold text-gradient sm:text-3xl">
                    {stat.value}
                  </div>
                  <div className="mt-1 text-xs text-dark-400">
                    {stat.label}{" "}
                    <span className="text-dark-600">{stat.labelEn}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="relative py-20 sm:py-28" id="features">
        <div className="absolute inset-0 bg-gradient-radial" />
        <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-dark-50 sm:text-4xl">
              {"\uAC15\uB825\uD55C \uAE30\uB2A5"}
              <span className="text-dark-500 ml-3 text-xl font-normal">
                Features
              </span>
            </h2>
            <p className="mt-4 text-dark-400 max-w-xl mx-auto">
              {"\uC790\uC728 AI \uC5D0\uC774\uC804\uD2B8\uC5D0 \uD544\uC694\uD55C \uBAA8\uB4E0 \uAE30\uB2A5\uC744 \uC81C\uACF5\uD569\uB2C8\uB2E4."}
            </p>
          </div>

          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
            {features.map((feature, index) => (
              <div
                key={feature.titleEn}
                className="glass-card rounded-2xl p-6 transition-all duration-300 hover:translate-y-[-2px]"
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-primary-500/10 border border-primary-500/20 text-2xl">
                  {feature.icon}
                </div>
                <h3 className="text-lg font-semibold text-dark-50 mb-1">
                  {feature.title}
                </h3>
                <p className="text-xs text-primary-400/60 font-medium mb-3">
                  {feature.titleEn}
                </p>
                <p className="text-sm text-dark-400 leading-relaxed">
                  {feature.description}
                </p>
                <p className="text-xs text-dark-500 mt-2 leading-relaxed">
                  {feature.descriptionEn}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Architecture Section */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 gap-12 lg:grid-cols-2 items-center">
            <div>
              <h2 className="text-3xl font-bold text-dark-50 sm:text-4xl">
                ZeroClaw Runtime
              </h2>
              <p className="mt-4 text-dark-400 leading-relaxed">
                {"\uACE0\uC131\uB2A5 Rust \uB7F0\uD0C0\uC784 \uC704\uC5D0 \uAD6C\uCD95\uB41C MoA\uB294 \uBE60\uB978 \uC18D\uB3C4, \uB0AE\uC740 \uBA54\uBAA8\uB9AC \uC0AC\uC6A9\uB7C9, \uADF8\uB9AC\uACE0 \uB192\uC740 \uBCF4\uC548\uC131\uC744 \uB3D9\uC2DC\uC5D0 \uC81C\uACF5\uD569\uB2C8\uB2E4."}
              </p>
              <p className="mt-2 text-sm text-dark-500">
                Built on a high-performance Rust runtime, MoA delivers speed,
                low memory footprint, and strong security guarantees.
              </p>
              <ul className="mt-8 space-y-4">
                {[
                  {
                    title: "Trait-driven \uBAA8\uB4C8\uB7EC \uC544\uD0A4\uD14D\uCC98",
                    titleEn: "Trait-driven modular architecture",
                    desc: "\uD504\uB85C\uBC14\uC774\uB354, \uCC44\uB110, \uB3C4\uAD6C \uB4F1\uC744 \uD2B8\uB808\uC787 \uAE30\uBC18\uC73C\uB85C \uC790\uC720\uB86D\uAC8C \uD655\uC7A5",
                  },
                  {
                    title: "\uBCF4\uC548 \uC6B0\uC120 \uC124\uACC4",
                    titleEn: "Security-first design",
                    desc: "\uD398\uC5B4\uB9C1 \uC778\uC99D, \uBE44\uBC00 \uAD00\uB9AC, \uCD5C\uC18C \uAD8C\uD55C \uC6D0\uCE59",
                  },
                  {
                    title: "\uACB0\uC815\uC801 \uBE4C\uB4DC",
                    titleEn: "Deterministic builds",
                    desc: "\uC7A0\uAE08\uB41C \uC758\uC874\uC131\uACFC \uC7AC\uD604 \uAC00\uB2A5\uD55C CI/CD",
                  },
                ].map((item) => (
                  <li key={item.titleEn} className="flex gap-3">
                    <div className="mt-1 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full bg-accent-500/10">
                      <svg
                        className="h-3 w-3 text-accent-400"
                        fill="none"
                        viewBox="0 0 24 24"
                        strokeWidth={3}
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          d="M4.5 12.75l6 6 9-13.5"
                        />
                      </svg>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-dark-200">
                        {item.title}
                      </span>
                      <span className="text-xs text-dark-500 ml-2">
                        {item.titleEn}
                      </span>
                      <p className="text-xs text-dark-400 mt-0.5">
                        {item.desc}
                      </p>
                    </div>
                  </li>
                ))}
              </ul>
            </div>

            {/* Architecture diagram */}
            <div className="glass-card rounded-2xl p-8">
              <div className="space-y-4">
                {[
                  {
                    label: "\uD504\uB85C\uBC14\uC774\uB354 Providers",
                    items: ["OpenAI", "Anthropic", "Gemini", "Ollama"],
                    color: "primary",
                  },
                  {
                    label: "\uCC44\uB110 Channels",
                    items: ["Telegram", "Discord", "Slack", "Web"],
                    color: "secondary",
                  },
                  {
                    label: "\uB3C4\uAD6C Tools",
                    items: ["Shell", "File", "Browser", "Memory"],
                    color: "accent",
                  },
                ].map((layer) => (
                  <div key={layer.label}>
                    <div className="text-xs font-medium text-dark-400 mb-2">
                      {layer.label}
                    </div>
                    <div className="grid grid-cols-4 gap-2">
                      {layer.items.map((item) => (
                        <div
                          key={item}
                          className={`rounded-lg border px-3 py-2 text-center text-xs font-medium transition-all hover:scale-[1.02] ${
                            layer.color === "primary"
                              ? "border-primary-500/20 bg-primary-500/5 text-primary-300"
                              : layer.color === "secondary"
                                ? "border-secondary-500/20 bg-secondary-500/5 text-secondary-300"
                                : "border-accent-500/20 bg-accent-500/5 text-accent-300"
                          }`}
                        >
                          {item}
                        </div>
                      ))}
                    </div>
                  </div>
                ))}

                {/* Center core */}
                <div className="flex items-center justify-center py-4">
                  <div className="rounded-xl border border-dark-600 bg-dark-800 px-6 py-3 text-center">
                    <div className="text-sm font-bold text-gradient">
                      ZeroClaw Core
                    </div>
                    <div className="text-[10px] text-dark-500 mt-0.5">
                      Agent Orchestration | Security | Memory
                    </div>
                  </div>
                </div>

                <div>
                  <div className="text-xs font-medium text-dark-400 mb-2">
                    {"\uBA54\uBAA8\uB9AC Memory"}
                  </div>
                  <div className="grid grid-cols-3 gap-2">
                    {["Markdown", "SQLite", "Vector"].map((item) => (
                      <div
                        key={item}
                        className="rounded-lg border border-dark-600 bg-dark-800/50 px-3 py-2 text-center text-xs font-medium text-dark-300"
                      >
                        {item}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section className="relative py-20 sm:py-28" id="pricing">
        <div className="absolute inset-0 bg-gradient-radial" />
        <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-dark-50 sm:text-4xl">
              {"\uC694\uAE08\uC81C"}
              <span className="text-dark-500 ml-3 text-xl font-normal">
                Pricing
              </span>
            </h2>
            <p className="mt-4 text-dark-400 max-w-xl mx-auto">
              {"\uBB34\uB8CC\uB85C \uC2DC\uC791\uD558\uACE0, \uD544\uC694\uC5D0 \uB530\uB77C \uC5C5\uADF8\uB808\uC774\uB4DC\uD558\uC138\uC694."}
            </p>
            <p className="mt-1 text-sm text-dark-500">
              Start free, upgrade as you grow.
            </p>
          </div>

          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 max-w-5xl mx-auto">
            {pricingTiers.map((tier) => (
              <div
                key={tier.name}
                className={`relative rounded-2xl p-6 transition-all duration-300 ${
                  tier.highlighted
                    ? "glass-card border-primary-500/30 bg-primary-500/5 scale-[1.02]"
                    : "glass-card"
                }`}
              >
                {tier.highlighted && (
                  <div className="absolute -top-3 left-1/2 -translate-x-1/2 rounded-full bg-primary-500 px-4 py-1 text-xs font-semibold text-white">
                    {"\uC778\uAE30"} Popular
                  </div>
                )}

                <div className="mb-6">
                  <h3 className="text-lg font-semibold text-dark-50">
                    {tier.nameKo}{" "}
                    <span className="text-dark-500 text-sm font-normal">
                      {tier.name}
                    </span>
                  </h3>
                  <div className="mt-3 flex items-baseline gap-1">
                    <span className="text-3xl font-bold text-dark-50">
                      {tier.price}
                    </span>
                    <span className="text-sm text-dark-500">
                      {tier.priceNote}
                    </span>
                  </div>
                </div>

                <ul className="space-y-3 mb-8">
                  {tier.features.map((feature) => (
                    <li key={feature} className="flex items-start gap-2.5">
                      <svg
                        className={`mt-0.5 h-4 w-4 flex-shrink-0 ${
                          tier.highlighted
                            ? "text-primary-400"
                            : "text-dark-500"
                        }`}
                        fill="none"
                        viewBox="0 0 24 24"
                        strokeWidth={2}
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          d="M4.5 12.75l6 6 9-13.5"
                        />
                      </svg>
                      <span className="text-sm text-dark-300">{feature}</span>
                    </li>
                  ))}
                </ul>

                <Link
                  href={tier.href}
                  className={`block w-full text-center rounded-lg px-6 py-3 text-sm font-semibold transition-all ${
                    tier.highlighted
                      ? "bg-primary-500 text-white hover:bg-primary-600"
                      : "border border-dark-600 bg-dark-800 text-dark-200 hover:border-dark-500 hover:bg-dark-700"
                  }`}
                >
                  {tier.cta}{" "}
                  <span className="text-xs opacity-60">{tier.ctaEn}</span>
                </Link>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="relative overflow-hidden rounded-3xl bg-gradient-to-br from-primary-500/10 via-secondary-500/5 to-accent-500/10 border border-primary-500/10 p-10 sm:p-16 text-center">
            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[500px] h-[500px] bg-primary-500/5 rounded-full blur-3xl" />
            <div className="relative">
              <h2 className="text-3xl font-bold text-dark-50 sm:text-4xl">
                {"\uC9C0\uAE08 \uBC14\uB85C \uC2DC\uC791\uD558\uC138\uC694"}
              </h2>
              <p className="mt-4 text-dark-400 max-w-lg mx-auto">
                {"\uBCC4\uB3C4\uC758 \uC124\uCE58 \uC5C6\uC774 \uC6F9 \uBE0C\uB77C\uC6B0\uC800\uC5D0\uC11C \uBC14\uB85C MoA\uB97C \uCCB4\uD5D8\uD574\uBCF4\uC138\uC694."}
              </p>
              <p className="mt-1 text-sm text-dark-500">
                Try MoA right in your browser. No installation required.
              </p>
              <div className="mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row">
                <Link href="/chat" className="btn-primary px-8 py-3.5 text-base glow-primary">
                  {"\uC6F9\uC5D0\uC11C \uCC44\uD305 \uC2DC\uC791"}
                  <svg
                    className="ml-2 h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth={2}
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
                    />
                  </svg>
                </Link>
                <a
                  href="https://github.com/AiFlowTools/MoA"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="btn-secondary px-8 py-3.5 text-base"
                >
                  GitHub
                  <svg
                    className="ml-2 h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    strokeWidth={2}
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
                    />
                  </svg>
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </>
  );
}
