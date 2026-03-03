# Remote Device Access — Architecture & Design

Date: 2026-03-03

## Overview

Remote Device Access enables users to chat with their MoA agent running on a
remote device (typically a mobile phone) from any web browser — including public
computers where nothing can be installed.

This is the **only** way to interact with a personal MoA agent from an uncontrolled
environment. Channel-based access (Telegram, Discord, etc.) requires installing the
channel app first, which defeats the purpose on public/shared machines.

## User Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│  1. User opens MoA homepage in browser (public PC, library, etc.)     │
│  2. Clicks "Remote Access"                                             │
│  3. Enters: username + password                                        │
│  4. Selects target device from their registered devices list           │
│  5. Enters device pairing code (pre-configured on the device)         │
│  6. WebSocket connection established → chat with remote agent         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Architecture

```
┌──────────────────────┐                     ┌──────────────────────────┐
│  Web Browser (public)│                     │  MoA Device (phone)      │
│                      │                     │                          │
│  POST /api/remote/   │  ① Auth + Pairing   │  App starts → registers  │
│       login          │─────────────────────│  via GET /ws/device-link │
│                      │                     │                          │
│  GET /ws/remote      │  ② Messages routed  │  Agent processes message │
│  (WebSocket)         │◄═══════════════════►│  Returns AI response     │
│                      │   via DeviceRouter  │                          │
└──────────────────────┘                     └──────────────────────────┘
                                │
                   ┌────────────┴────────────┐
                   │     Gateway Server      │
                   │                         │
                   │  ┌───────────────────┐  │
                   │  │   DeviceRouter    │  │
                   │  │                   │  │
                   │  │  device_id →      │  │
                   │  │    mpsc::Sender   │  │
                   │  │                   │  │
                   │  │  msg_id →         │  │
                   │  │    response_tx    │  │
                   │  └───────────────────┘  │
                   │                         │
                   │  ┌───────────────────┐  │
                   │  │    AuthStore      │  │
                   │  │  (SQLite-backed)  │  │
                   │  │                   │  │
                   │  │  users            │  │
                   │  │  sessions         │  │
                   │  │  devices          │  │
                   │  │  pairing_codes    │  │
                   │  └───────────────────┘  │
                   └─────────────────────────┘
```

## Components

### 1. DeviceRouter (`src/gateway/remote.rs`)

Central in-memory registry that tracks connected device agents and routes
messages between web clients and devices.

**Data structures:**

| Field | Type | Purpose |
|-------|------|---------|
| `connections` | `HashMap<device_id, DeviceConnection>` | Active device WebSocket connections |
| `login_attempts` | `HashMap<ip, LoginAttemptState>` | Brute-force protection |

**Operations:**
- `register_device(id, name)` → returns `mpsc::Receiver` for inbound messages
- `unregister_device(id)` → cleanup on disconnect
- `send_to_device(id, msg)` → route message to device
- `is_device_online(id)` → check real-time connection status

### 2. AuthStore (`src/auth/store.rs`)

SQLite-backed user authentication with device management and pairing codes.

**Tables:**

| Table | Key Fields | Purpose |
|-------|-----------|---------|
| `users` | id, username, password_hash, salt | User accounts |
| `sessions` | token_hash, user_id, device_id, expires_at | Session tokens (30-day TTL) |
| `devices` | device_id, user_id, device_name, platform, pairing_code_hash | Registered devices |
| `channel_links` | channel, platform_uid, user_id | Channel identity linking |

**Security:**
- Passwords: iterated SHA-256 (100,000 rounds) with per-user salt
- Session tokens: 256-bit random, stored as SHA-256 hash
- Pairing codes: hashed with device_id as salt (constant-time comparison)
- Timing-attack resistant comparisons throughout

### 3. Remote Response Channel (`REMOTE_RESPONSE_CHANNELS`)

Process-global registry mapping message IDs to response channels:

```
msg_id → mpsc::Sender<RoutedMessage>
```

This enables device responses to be routed back to the correct web client
WebSocket when multiple remote sessions are active simultaneously.

## API Endpoints

### REST

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/api/remote/login` | POST | None (creates session) | Authenticate + verify pairing code |
| `/api/remote/devices` | GET | Session token | List devices with online status |
| `/api/remote/logout` | POST | Session token | Revoke session |

### WebSocket

| Endpoint | Auth | Direction | Purpose |
|----------|------|-----------|---------|
| `/ws/remote` | Session token | Web → Device | Remote chat from browser |
| `/ws/device-link` | Session token | Device → Server | Device agent registration |

## Protocol

### Remote Web Client (`/ws/remote`)

```text
Client → Server: {"type":"message","content":"Hello"}
Server → Client: {"type":"device_status","online":true,"device_id":"..."}
Server → Client: {"type":"chunk","content":"partial..."}
Server → Client: {"type":"done","full_response":"complete response","content":"..."}
Server → Client: {"type":"error","message":"..."}
```

### Device Agent Link (`/ws/device-link`)

```text
Server → Device: {"type":"remote_message","id":"msg-uuid","content":"user prompt"}
Device → Server: {"type":"remote_response","id":"msg-uuid","content":"agent response"}
Device → Server: {"type":"remote_chunk","id":"msg-uuid","content":"partial..."}
Device → Server: {"type":"remote_error","id":"msg-uuid","content":"error message"}
Device → Server: {"type":"heartbeat"}
```

## Security Model

### Two-Factor Authentication

Remote access requires both:
1. **Account credentials** — username + password (verified against AuthStore)
2. **Device pairing code** — per-device secret (verified against AuthStore)

This ensures that even if account credentials are compromised, the attacker
cannot access the device without the pairing code that was set locally on the
device itself.

### Rate Limiting

| Parameter | Value | Purpose |
|-----------|-------|---------|
| Max login attempts | 10 per IP | Prevent brute-force |
| Lockout duration | 5 minutes | Cool-down after threshold |
| Tracked IPs | 10,000 max | Memory-bounded |
| Record retention | 15 minutes | Auto-cleanup stale entries |

### Session Management

- Tokens: 256-bit random (64 hex chars)
- Default TTL: 30 days (configurable)
- Storage: SHA-256 hash only (plaintext returned once)
- Revocation: immediate via `/api/remote/logout`

### Device Verification

Before routing any message, the system verifies:
1. Session token is valid and not expired
2. Target device belongs to the authenticated user
3. Device is currently online (connected via `/ws/device-link`)

## Configuration

Enable in `config.toml`:

```toml
[auth]
enabled = true
allow_registration = true
session_ttl_secs = 2592000   # 30 days
max_devices_per_user = 10
```

The device pairing code is set locally on the device via:
- CLI: `zeroclaw device set-pairing-code <code>`
- API: `PUT /api/device/pairing-code`
- Config: `auth.device_pairing_code = "your-code"`

## Data Flow (Message Routing)

```
Web Client                   Gateway Server                    Device Agent
    │                             │                                  │
    │  {"type":"message",         │                                  │
    │   "content":"Hello"}        │                                  │
    │─────────────────────────────►│                                  │
    │                             │  RoutedMessage{                  │
    │                             │    id: "uuid-1",                 │
    │                             │    content: "Hello",             │
    │                             │    direction: "to_device"        │
    │                             │  }                               │
    │                             │──────────────────────────────────►│
    │                             │                                  │
    │                             │  (Agent processes, runs tools)   │
    │                             │                                  │
    │                             │  {"type":"remote_response",      │
    │                             │   "id":"uuid-1",                 │
    │                             │◄──────────────────────────────────│
    │                             │   "content":"Hi! I'm..."}        │
    │                             │                                  │
    │  {"type":"done",            │                                  │
    │◄─────────────────────────────│                                  │
    │   "full_response":"Hi!..."} │                                  │
    │                             │                                  │
```

## Failure Modes

| Scenario | Behavior |
|----------|----------|
| Device goes offline mid-chat | Web client receives `device_status: offline` |
| Web client disconnects | Pending response channels cleaned up |
| Session token expires | WebSocket upgrade returns 401 |
| Invalid pairing code | Login returns 401 |
| Device not found | Login returns 404 |
| Multiple web sessions | Each gets independent message routing |

## Non-Goals (Current Scope)

- **Message persistence**: Remote chat messages are ephemeral, not stored on server
- **Offline message queue**: Device must be online for remote access
- **File transfer**: Only text messages are supported
- **Multi-device fan-out**: One web session talks to one device at a time
- **E2E encryption**: Messages transit through server in cleartext
  (future enhancement: optional E2E with device key exchange)

## Files

| File | Purpose |
|------|---------|
| `src/gateway/remote.rs` | DeviceRouter, REST endpoints, WebSocket handlers |
| `src/auth/store.rs` | SQLite-backed AuthStore (users, sessions, devices, pairing codes) |
| `src/auth/mod.rs` | Auth module exports (added `pub mod store`) |
| `src/gateway/mod.rs` | AppState fields, route registration, gateway init |
| `src/gateway/pair.rs` | Channel auto-pairing web flow (uses AuthStore) |
| `src/config/schema.rs` | AuthConfig schema |
| `docs/remote-device-access.md` | This document |
