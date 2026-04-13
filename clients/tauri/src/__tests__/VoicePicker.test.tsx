import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import VoicePicker from "../components/VoicePicker";

// ── Mock modules (factory is hoisted — no external refs allowed) ──

vi.mock("../lib/tauri-bridge", () => ({
  gatewayFetch: vi.fn().mockResolvedValue({
    status: 200,
    body: JSON.stringify({
      voices: [
        { voice_id: "v1", voice_name: "Miran Choi", gender: "female", age: "young_adult", use_cases: ["conversational"], models: [{ version: "v1", emotions: ["happy"] }] },
        { voice_id: "v2", voice_name: "Hans", gender: "male", age: "middle_age", use_cases: ["voice_assistant"], models: [{ version: "v1", emotions: ["neutral"] }] },
        { voice_id: "v3", voice_name: "Charlotte", gender: "female", age: "teenager", use_cases: ["conversational"], models: [{ version: "v1", emotions: ["sad"] }] },
      ],
      categories: {
        gender: { female: { en: "Female", ko: "여성" }, male: { en: "Male", ko: "남성" } },
        age: { young_adult: { en: "Young Adult", ko: "청년" }, middle_age: { en: "Middle Age", ko: "중년" }, teenager: { en: "Teenager", ko: "10대" } },
        use_cases: { conversational: { en: "Conversational", ko: "일상대화" }, voice_assistant: { en: "Voice Assistant", ko: "음성비서" } },
        mood: { formal: { en: "Formal", ko: "격식체" }, casual: { en: "Casual", ko: "반말" } },
        language: {},
      },
      mood_to_use_cases: { formal: ["voice_assistant"], casual: ["conversational"] },
      smart_emotion: true,
      emotions: ["happy", "neutral", "sad"],
      languages_count: 9,
    }),
  }),
}));

vi.mock("../lib/api", () => ({
  apiClient: { getServerUrl: () => "http://localhost:3000" },
}));

describe("VoicePicker", () => {
  const defaultProps = {
    locale: "en",
    onSelect: vi.fn(),
    onClose: vi.fn(),
  };

  beforeEach(() => {
    defaultProps.onSelect.mockClear();
    defaultProps.onClose.mockClear();
  });

  it("renders loading state initially", () => {
    render(<VoicePicker {...defaultProps} />);
    expect(screen.getByText(/Loading voices/i)).toBeInTheDocument();
  });

  it("renders voice cards after loading", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => {
      expect(screen.getByText("Miran Choi")).toBeInTheDocument();
      expect(screen.getByText("Hans")).toBeInTheDocument();
      expect(screen.getByText("Charlotte")).toBeInTheDocument();
    });
  });

  it("shows correct voice count in subtitle", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => {
      expect(screen.getByText(/3 assistants/)).toBeInTheDocument();
    });
  });

  it("filters by gender", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    fireEvent.click(screen.getByText("Female"));

    expect(screen.getByText("Miran Choi")).toBeInTheDocument();
    expect(screen.getByText("Charlotte")).toBeInTheDocument();
    expect(screen.queryByText("Hans")).not.toBeInTheDocument();
  });

  it("filters by search text", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    const searchInput = screen.getByPlaceholderText(/Search assistant/i);
    fireEvent.change(searchInput, { target: { value: "hans" } });

    expect(screen.getByText("Hans")).toBeInTheDocument();
    expect(screen.queryByText("Miran Choi")).not.toBeInTheDocument();
  });

  it("shows reset button when filters are active", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    expect(screen.queryByText(/Reset/)).not.toBeInTheDocument();

    fireEvent.click(screen.getByText("Female"));
    expect(screen.getByText(/Reset \(1\)/)).toBeInTheDocument();
  });

  it("resets all filters on reset button click", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    fireEvent.click(screen.getByText("Female"));
    expect(screen.queryByText("Hans")).not.toBeInTheDocument();

    fireEvent.click(screen.getByText(/Reset/));
    expect(screen.getByText("Hans")).toBeInTheDocument();
    expect(screen.getByText("Miran Choi")).toBeInTheDocument();
  });

  it("collapses and expands filter sections", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    // Gender pills should be visible
    expect(screen.getByText("Female")).toBeInTheDocument();

    // Click Gender label to collapse
    fireEvent.click(screen.getByText("Gender"));

    // Female pill should be hidden
    expect(screen.queryByText("Female")).not.toBeInTheDocument();

    // Click again to expand
    fireEvent.click(screen.getByText("Gender"));
    expect(screen.getByText("Female")).toBeInTheDocument();
  });

  it("calls onSelect when a voice card is clicked", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Hans"));

    fireEvent.click(screen.getByText("Hans"));

    expect(defaultProps.onSelect).toHaveBeenCalledWith("v2", "Hans", "male");
  });

  it("calls onClose when overlay is clicked", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    // Click the overlay (background)
    const overlay = document.querySelector(".voice-picker-overlay");
    if (overlay) fireEvent.click(overlay);

    expect(defaultProps.onClose).toHaveBeenCalled();
  });

  it("shows empty message when no voices match filters", async () => {
    render(<VoicePicker {...defaultProps} />);
    await waitFor(() => screen.getByText("Miran Choi"));

    const searchInput = screen.getByPlaceholderText(/Search assistant/i);
    fireEvent.change(searchInput, { target: { value: "nonexistent" } });

    expect(screen.getByText(/No assistants match your filters/)).toBeInTheDocument();
  });

  it("renders Korean locale", async () => {
    render(<VoicePicker {...defaultProps} locale="ko" />);
    await waitFor(() => {
      expect(screen.getByText(/비서 선택/)).toBeInTheDocument();
    });
  });
});
