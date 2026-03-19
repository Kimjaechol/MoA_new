import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Locale } from "../lib/i18n";

interface ChannelConnectProps {
  locale: Locale;
  onClose: () => void;
}

interface ChannelInfo {
  name: string;
  displayName: string;
  displayNameKo: string;
  icon: string;
  description: string;
  descriptionKo: string;
}

const CHANNELS: ChannelInfo[] = [
  {
    name: "kakao",
    displayName: "KakaoTalk",
    displayNameKo: "\uCE74\uCE74\uC624\uD1A1",
    icon: "\uD83D\uDCAC",
    description: "Connect KakaoTalk channel to chat with MoA",
    descriptionKo: "\uCE74\uCE74\uC624\uD1A1 \uCC44\uB110\uC744 \uC5F0\uACB0\uD558\uC5EC MoA\uC640 \uB300\uD654\uD558\uC138\uC694",
  },
  {
    name: "whatsapp",
    displayName: "WhatsApp",
    displayNameKo: "WhatsApp",
    icon: "\uD83D\uDCF1",
    description: "Connect WhatsApp to chat with MoA",
    descriptionKo: "WhatsApp\uC744 \uC5F0\uACB0\uD558\uC5EC MoA\uC640 \uB300\uD654\uD558\uC138\uC694",
  },
  {
    name: "telegram",
    displayName: "Telegram",
    displayNameKo: "\uD154\uB808\uADF8\uB7A8",
    icon: "\u2708\uFE0F",
    description: "Connect Telegram bot to chat with MoA",
    descriptionKo: "\uD154\uB808\uADF8\uB7A8 \uBD07\uC744 \uC5F0\uACB0\uD558\uC5EC MoA\uC640 \uB300\uD654\uD558\uC138\uC694",
  },
  {
    name: "discord",
    displayName: "Discord",
    displayNameKo: "\uB514\uC2A4\uCF54\uB4DC",
    icon: "\uD83C\uDFAE",
    description: "Connect Discord bot to chat with MoA",
    descriptionKo: "\uB514\uC2A4\uCF54\uB4DC \uBD07\uC744 \uC5F0\uACB0\uD558\uC5EC MoA\uC640 \uB300\uD654\uD558\uC138\uC694",
  },
];

type Step = "select" | "pairing" | "done";

export default function ChannelConnect({ locale, onClose }: ChannelConnectProps) {
  const [step, setStep] = useState<Step>("select");
  const [selectedChannel, setSelectedChannel] = useState<ChannelInfo | null>(null);
  const [pairingCode, setPairingCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const isKo = locale === "ko";

  const handleSelectChannel = async (channel: ChannelInfo) => {
    setSelectedChannel(channel);
    setLoading(true);
    setError("");

    try {
      const result = await invoke<{ code: string; instructions: string }>(
        "generate_channel_pairing_code",
        { channel: channel.name }
      );
      setPairingCode(result.code);
      setStep("pairing");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const renderChannelSelect = () => (
    <div style={{ padding: "20px" }}>
      <h2 style={{ marginBottom: "16px" }}>
        {isKo ? "\uCC44\uB110 \uC5F0\uACB0" : "Connect Channel"}
      </h2>
      <p style={{ color: "#888", marginBottom: "20px", fontSize: "14px" }}>
        {isKo
          ? "\uBA54\uC2E0\uC800 \uCC44\uB110\uC744 \uC5F0\uACB0\uD558\uBA74, \uD574\uB2F9 \uCC44\uB110\uC5D0\uC11C \uBCF4\uB0B8 \uBA54\uC2DC\uC9C0\uAC00 \uC774 MoA \uC571\uC73C\uB85C \uC790\uB3D9 \uC804\uB2EC\uB429\uB2C8\uB2E4."
          : "Connect a messaging channel so messages sent there are automatically processed by this MoA app."}
      </p>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {CHANNELS.map((ch) => (
          <button
            key={ch.name}
            onClick={() => handleSelectChannel(ch)}
            disabled={loading}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "12px",
              padding: "16px",
              border: "1px solid #333",
              borderRadius: "8px",
              background: "#1a1a1a",
              color: "#fff",
              cursor: "pointer",
              textAlign: "left",
            }}
          >
            <span style={{ fontSize: "24px" }}>{ch.icon}</span>
            <div>
              <div style={{ fontWeight: "bold" }}>
                {isKo ? ch.displayNameKo : ch.displayName}
              </div>
              <div style={{ fontSize: "12px", color: "#888" }}>
                {isKo ? ch.descriptionKo : ch.description}
              </div>
            </div>
          </button>
        ))}
      </div>
      {error && (
        <p style={{ color: "#ff4444", marginTop: "12px", fontSize: "13px" }}>
          {error}
        </p>
      )}
    </div>
  );

  const renderPairing = () => (
    <div style={{ padding: "20px", textAlign: "center" }}>
      <h2 style={{ marginBottom: "8px" }}>
        {selectedChannel?.icon}{" "}
        {isKo
          ? `${selectedChannel?.displayNameKo} \uC5F0\uACB0`
          : `Connect ${selectedChannel?.displayName}`}
      </h2>
      <p style={{ color: "#888", marginBottom: "24px", fontSize: "14px" }}>
        {isKo
          ? `${selectedChannel?.displayNameKo}\uC5D0\uC11C MoA \uCC44\uB110\uC5D0 \uC544\uB798 \uCF54\uB4DC\uB97C \uBCF4\uB0B4\uC138\uC694`
          : `Send this code to the MoA channel on ${selectedChannel?.displayName}`}
      </p>

      <div
        style={{
          fontSize: "32px",
          fontFamily: "monospace",
          fontWeight: "bold",
          letterSpacing: "4px",
          padding: "20px",
          background: "#0a0a0a",
          borderRadius: "12px",
          border: "2px solid #333",
          marginBottom: "20px",
          userSelect: "all",
          cursor: "pointer",
        }}
        title={isKo ? "\uD074\uB9AD\uD558\uC5EC \uBCF5\uC0AC" : "Click to copy"}
        onClick={() => navigator.clipboard?.writeText(pairingCode)}
      >
        {pairingCode}
      </div>

      <div
        style={{
          background: "#1a1a2e",
          borderRadius: "8px",
          padding: "16px",
          textAlign: "left",
          fontSize: "13px",
          lineHeight: "1.8",
          marginBottom: "20px",
        }}
      >
        <p style={{ fontWeight: "bold", marginBottom: "8px" }}>
          {isKo ? "\uBC29\uBC95:" : "Steps:"}
        </p>
        <ol style={{ margin: 0, paddingLeft: "20px" }}>
          <li>
            {isKo
              ? `${selectedChannel?.displayNameKo}\uC5D0\uC11C "MoA" \uCC44\uB110\uC744 \uCC3E\uC544 \uCE5C\uAD6C \uCD94\uAC00\uD558\uC138\uC694`
              : `Find and add the "MoA" channel on ${selectedChannel?.displayName}`}
          </li>
          <li>
            {isKo
              ? "\uCC44\uD305\uCC3D\uC5D0 \uC704\uC758 \uCF54\uB4DC\uB97C \uBCF5\uC0AC\uD574\uC11C \uBCF4\uB0B4\uC138\uC694"
              : "Copy the code above and send it in the chat"}
          </li>
          <li>
            {isKo
              ? '"\uD398\uC5B4\uB9C1 \uC644\uB8CC" \uBA54\uC2DC\uC9C0\uAC00 \uC624\uBA74 \uC131\uACF5\uC785\uB2C8\uB2E4!'
              : 'You\'ll see a "Pairing complete" message when done!'}
          </li>
        </ol>
      </div>

      <p style={{ color: "#666", fontSize: "12px" }}>
        {isKo
          ? "\uCF54\uB4DC\uB294 10\uBD84\uAC04 \uC720\uD6A8\uD569\uB2C8\uB2E4. \uB9CC\uB8CC \uC2DC \uB2E4\uC2DC \uC0DD\uC131\uD574 \uC8FC\uC138\uC694."
          : "Code expires in 10 minutes. Generate a new one if expired."}
      </p>

      <div style={{ display: "flex", gap: "12px", justifyContent: "center", marginTop: "20px" }}>
        <button
          onClick={() => {
            setStep("select");
            setPairingCode("");
          }}
          style={{
            padding: "10px 20px",
            borderRadius: "6px",
            border: "1px solid #555",
            background: "transparent",
            color: "#aaa",
            cursor: "pointer",
          }}
        >
          {isKo ? "\uB4A4\uB85C" : "Back"}
        </button>
        <button
          onClick={onClose}
          style={{
            padding: "10px 20px",
            borderRadius: "6px",
            border: "none",
            background: "#4a9eff",
            color: "#fff",
            cursor: "pointer",
          }}
        >
          {isKo ? "\uC644\uB8CC" : "Done"}
        </button>
      </div>
    </div>
  );

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: "rgba(0,0,0,0.7)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        style={{
          background: "#111",
          borderRadius: "12px",
          maxWidth: "480px",
          width: "90%",
          maxHeight: "80vh",
          overflow: "auto",
          position: "relative",
        }}
      >
        <button
          onClick={onClose}
          style={{
            position: "absolute",
            top: "12px",
            right: "12px",
            background: "none",
            border: "none",
            color: "#888",
            fontSize: "20px",
            cursor: "pointer",
          }}
        >
          X
        </button>
        {step === "select" && renderChannelSelect()}
        {step === "pairing" && renderPairing()}
      </div>
    </div>
  );
}
