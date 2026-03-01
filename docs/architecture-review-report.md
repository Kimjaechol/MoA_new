# MoA Architecture Review Report

Date: 2026-03-01

## Overview

Comprehensive review of the MoA codebase against the intended architecture
and patent specification for the 3-tier memory synchronization system.

## Critical Findings

### C1: Tauri App is a Thin Client (Not Local Agent)

The Tauri app sends all chat messages to the Railway server's `/webhook`
endpoint. ZeroClaw agent runtime is NOT embedded in the Tauri app. Each
device should run ZeroClaw locally with its own SQLite, using Railway
only as a memory sync relay.

**Files**: `clients/tauri/src-tauri/src/lib.rs:69`, `clients/tauri/src/lib/api.ts:329`

### C2: No API Key Management UI

Settings page has no API key input fields for Claude/Gemini/OpenAI.
No distinction between user's own key vs operator's fallback key.
No 2x credit deduction logic for operator key usage.

**Files**: `clients/tauri/src/components/Settings.tsx`, `src/providers/mod.rs`

### C3: Encryption Algorithm Mismatch

Sync uses ChaCha20-Poly1305 instead of patent-specified AES-256-GCM.
No PBKDF2 key derivation from user passphrase (uses random key file).

**Files**: `src/memory/sync.rs`, `src/security/encryption.rs`

### C4: No Credit Purchase UI

Backend Kakao Pay integration exists but no frontend purchase/balance UI.

**Files**: `src/billing/payment.rs`

## Well-Implemented Components

1. 3-tier sync protocol data structures and message types
2. WebSocket broadcast (no DB storage, echo prevention)
3. TTL-based relay (5 min, in-memory, auto-delete)
4. Version vectors + order buffer (sequence guarantees)
5. FTS5 full-text search with sync triggers
6. Multi-device auth with SQLite-backed device management
7. Gemini Live voice interpretation (2.5 Flash)
8. Mobile platform configuration (Android/iOS)
9. Agent loop with provider/memory/tools integration

## Priority Matrix

- **P0**: Embed ZeroClaw in Tauri, API key management, 2x credit logic,
  AES-256-GCM, PBKDF2 key derivation
- **P1**: Credit UI, model auto-selection, sqlite-vec, key exchange,
  Tier 2-to-3 escalation
- **P2**: Conversation/settings sync, DeltaAck processing, cost tracker
  consolidation, Interpreter default provider
