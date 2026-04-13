import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, cleanup } from "@testing-library/react";
import { LiveKitVoiceChat } from "../components/LiveKitVoiceChat";

// ── Mock livekit-client (factory is hoisted — all refs must be inline) ──

const mockConnect = vi.fn().mockResolvedValue(undefined);
const mockDisconnect = vi.fn();
const mockSetMicEnabled = vi.fn().mockResolvedValue(undefined);
const mockOn = vi.fn().mockReturnThis();

vi.mock("livekit-client", () => {
  return {
    Room: class MockRoom {
      connect = mockConnect;
      disconnect = mockDisconnect;
      localParticipant = { setMicrophoneEnabled: mockSetMicEnabled };
      on = mockOn;
      off = vi.fn();
    },
    RoomEvent: {
      Connected: "connected",
      Disconnected: "disconnected",
      TrackSubscribed: "trackSubscribed",
      TrackUnsubscribed: "trackUnsubscribed",
      DataReceived: "dataReceived",
      ParticipantMetadataChanged: "participantMetadataChanged",
      ConnectionStateChanged: "connectionStateChanged",
    },
    Track: { Kind: { Audio: "audio", Video: "video" } },
    ConnectionState: { Reconnecting: "reconnecting" },
  };
});

// ── Mock api client ──

let mockGetToken: () => string | null = () => "test-jwt-token";

vi.mock("../lib/api", () => ({
  apiClient: {
    getServerUrl: () => "http://localhost:3000",
    getToken: () => mockGetToken(),
  },
}));

// ── Mock fetch ──

const mockFetch = vi.fn();

beforeEach(() => {
  mockGetToken = () => "test-jwt-token";
  vi.stubGlobal("fetch", mockFetch);
  mockFetch.mockResolvedValue({
    ok: true,
    json: () =>
      Promise.resolve({
        token: "lk-test-token",
        url: "wss://lk.example.com",
      }),
  });
});

afterEach(() => {
  cleanup();
  mockConnect.mockClear();
  mockDisconnect.mockClear();
  mockSetMicEnabled.mockClear();
  mockOn.mockClear();
  mockFetch.mockClear();
});

// ── Tests ──

describe("LiveKitVoiceChat", () => {
  const defaultProps = {
    locale: "en" as const,
    onClose: vi.fn(),
  };

  it("renders idle state with start button", () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    expect(screen.getByText(/Start Voice Chat/i)).toBeInTheDocument();
    expect(screen.getByText("Idle")).toBeInTheDocument();
  });

  it("renders Korean locale correctly", () => {
    render(<LiveKitVoiceChat {...defaultProps} locale="ko" />);
    expect(screen.getByText(/음성 대화 시작/)).toBeInTheDocument();
    expect(screen.getByText("대기")).toBeInTheDocument();
  });

  it("requests LiveKit token on connect", async () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:3000/api/livekit/token",
        expect.objectContaining({
          method: "POST",
          headers: expect.objectContaining({
            Authorization: "Bearer test-jwt-token",
          }),
        })
      );
    });
  });

  it("connects to LiveKit room after receiving token", async () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      expect(mockConnect).toHaveBeenCalledWith(
        "wss://lk.example.com",
        "lk-test-token"
      );
    });
  });

  it("enables microphone after room connection", async () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      expect(mockSetMicEnabled).toHaveBeenCalledWith(true);
    });
  });

  it("sends voice_mode in token request metadata", async () => {
    render(<LiveKitVoiceChat {...defaultProps} initialMode="s2s" />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      const call = mockFetch.mock.calls[0];
      const body = JSON.parse(call[1].body);
      const metadata = JSON.parse(body.metadata);
      expect(metadata.voice_mode).toBe("s2s");
    });
  });

  it("shows error when auth token is missing", async () => {
    mockGetToken = () => null;

    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      const matches = screen.getAllByText(/Not authenticated/i);
      expect(matches.length).toBeGreaterThan(0);
    });
  });

  it("shows error on token fetch failure", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 403,
      json: () => Promise.resolve({ error: "Forbidden" }),
    });

    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      const matches = screen.getAllByText(/Forbidden/i);
      expect(matches.length).toBeGreaterThan(0);
    });
  });

  it("registers required LiveKit room event handlers", async () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      const eventNames = mockOn.mock.calls.map((c: any[]) => c[0]);
      expect(eventNames).toContain("connected");
      expect(eventNames).toContain("disconnected");
      expect(eventNames).toContain("trackSubscribed");
      expect(eventNames).toContain("dataReceived");
    });
  });

  it("displays mode selector with pipeline and s2s options", () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    expect(screen.getByText(/Mode A: Pipeline/)).toBeInTheDocument();
    expect(screen.getByText(/Mode B: Gemini Live/)).toBeInTheDocument();
  });

  it("disables mode selector during active session", async () => {
    render(<LiveKitVoiceChat {...defaultProps} />);
    fireEvent.click(screen.getByText(/Start Voice Chat/i));

    await waitFor(() => {
      const modeBtn = screen.getByText(/Mode A: Pipeline/);
      expect(modeBtn).toBeDisabled();
    });
  });
});
