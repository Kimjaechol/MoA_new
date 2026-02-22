import type { Metadata, Viewport } from "next";
import "./globals.css";
import Header from "@/components/Header";

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: "#0f172a",
};

export const metadata: Metadata = {
  title: "MoA - Master of AI | \uB2F9\uC2E0\uC758 AI \uC5D0\uC774\uC804\uD2B8",
  description:
    "MoA\uB294 ZeroClaw \uB7F0\uD0C0\uC784 \uAE30\uBC18\uC758 \uC790\uC728 AI \uC5D0\uC774\uC804\uD2B8\uC785\uB2C8\uB2E4. \uB2E4\uC911 AI \uBAA8\uB378, \uC5D4\uB4DC\uD22C\uC5D4\uB4DC \uC554\uD638\uD654, \uBA40\uD2F0\uCC44\uB110 \uD1B5\uD569\uC744 \uC9C0\uC6D0\uD569\uB2C8\uB2E4. MoA is an autonomous AI agent powered by ZeroClaw runtime. Multi-model, encrypted, cross-platform.",
  keywords: [
    "MoA",
    "Master of AI",
    "AI agent",
    "ZeroClaw",
    "chatbot",
    "AI \uC5D0\uC774\uC804\uD2B8",
    "\uC790\uC728 AI",
    "\uBA40\uD2F0\uBAA8\uB378",
    "cross-platform",
  ],
  authors: [{ name: "ZeroClaw Team" }],
  openGraph: {
    type: "website",
    locale: "ko_KR",
    alternateLocale: "en_US",
    url: "https://moa-agent.com",
    siteName: "MoA - Master of AI",
    title: "MoA - Master of AI | \uB2F9\uC2E0\uC758 AI \uC5D0\uC774\uC804\uD2B8",
    description:
      "ZeroClaw \uB7F0\uD0C0\uC784 \uAE30\uBC18\uC758 \uC790\uC728 AI \uC5D0\uC774\uC804\uD2B8. \uB2E4\uC911 AI \uBAA8\uB378, \uC5D4\uB4DC\uD22C\uC5D4\uB4DC \uC554\uD638\uD654, \uBA40\uD2F0\uCC44\uB110 \uD1B5\uD569.",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "MoA - Master of AI",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "MoA - Master of AI",
    description:
      "ZeroClaw \uB7F0\uD0C0\uC784 \uAE30\uBC18\uC758 \uC790\uC728 AI \uC5D0\uC774\uC804\uD2B8",
    images: ["/og-image.png"],
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ko" className="dark">
      <body
        className="font-sans bg-dark-950 text-dark-100 antialiased"
      >
        <Header />
        <main className="min-h-screen">{children}</main>
      </body>
    </html>
  );
}
