# Gemini Live Binary Frames Fix

> PR #29 — `fix/gemini-live-binary-frames` (2026-02-26)

## Problem

Voice interpretation (Korean↔Japanese) via Gemini Live API was failing with:

```
Gemini Live setup timed out or failed
```

The WebSocket connection opened successfully (HTTP 101), but `setupComplete`
was never received, causing the gateway's 10-second timeout to fire every time.

Node.js `ws` library worked with the same URL and JSON. Rust `tokio-tungstenite`
did not. This made it appear to be a Rust library issue.

## Root Cause

**Two bugs working together:**

### Bug 1: Binary Frame Mismatch

Google Gemini Live API sends **ALL messages as WebSocket Binary frames**,
including JSON responses like `{"setupComplete": {}}`.

Our `inbound_loop()` only parsed JSON from **Text frames**. Binary frames
were unconditionally treated as raw audio data. So `setupComplete` arrived
as a Binary frame, was ignored, and the session was never marked as ready.

**Evidence:**

```
Got first message (direct read) msg=Binary([123, 10, 32, 32, 34, 115, 101,
116, 117, 112, 67, 111, 109, 112, 108, 101, 116, 101, 34, ...])
```

Decodes to: `{"setupComplete": {}}`

### Bug 2: Duplicate setupComplete Wait

`GeminiLiveSession::connect()` consumed the `setupComplete` message internally
(before splitting the stream into sender/receiver). But the gateway handler
(`handle_voice_ws_connection`) had a **separate** 10-second wait for a
`SetupComplete` event via the `event_rx` channel.

Since `connect()` already consumed the message before the inbound loop was
spawned, the gateway's wait could never succeed.

## Fix

### `src/voice/gemini_live.rs`

1. **`connect()`**: Read `setupComplete` directly on the unsplit stream
   before splitting into sender/receiver halves. Checks both Binary and Text
   frames for the `setupComplete` JSON string.

2. **`inbound_loop()`**: Binary frames starting with `{` are now parsed as
   JSON first. Only if UTF-8 decode or JSON parse fails do we treat the data
   as raw audio.

### `src/gateway/mod.rs`

3. **Removed duplicate wait**: The gateway no longer waits for `SetupComplete`
   via event channel. If `connect()` returns `Ok`, the session is ready.
   (28 lines removed)

## What Was NOT the Cause

- **TLS backend** (rustls vs native-tls): Both work with `connect_async()`.
  Tested both explicitly — same behavior. Kept rustls (no OS dependency).

- **Manual WebSocket headers**: Adding explicit `Host`, `Connection`, `Upgrade`,
  `Sec-WebSocket-Version`, `Sec-WebSocket-Key` headers via
  `connect_async_tls_with_config` caused **duplicate headers** (tungstenite adds
  them automatically), which made Google's server silently drop all messages.
  Fix: use simple `connect_async(&url)`.

## Files Changed

| File | Change |
|------|--------|
| `src/voice/gemini_live.rs` | +96 lines: Binary JSON parsing, setupComplete in connect() |
| `src/gateway/mod.rs` | -28 lines: Removed duplicate setupComplete wait |

## Verification

1. Server log: `Gemini Live setup complete — ready to stream` (confirmed)
2. Without `GEMINI_API_KEY`: returns proper error message instead of timeout
3. WebSocket upgrade succeeds (HTTP 101) and voice session marked "ready"

## Key Takeaway

When integrating with Google's streaming APIs over WebSocket, **do not assume
JSON will arrive in Text frames**. Google Gemini Live sends everything
(including JSON control messages) as Binary frames. Always check Binary frame
content before treating it as raw binary data.
