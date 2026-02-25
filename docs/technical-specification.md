# MoA (Master of AI) 기술명세서

**문서 버전:** 1.0
**작성일:** 2026-02-25
**기반 프레임워크:** ZeroClaw (Rust 기반 자율 에이전트 런타임)
**저장소:** https://github.com/Kimjaechol/MoA_new

---

## 1. 시스템 개요

MoA(Master of AI)는 ZeroClaw 오픈소스 에이전트 프레임워크를 기반으로 한 **능동형 AI 에이전트 플랫폼**입니다. Rust로 작성된 고성능 백엔드 런타임과 Tauri 기반 크로스 플랫폼 클라이언트, Next.js 웹 클라이언트로 구성됩니다.

### 1.1 아키텍처 요약

```
┌─────────────────────────────────────────────────────────────┐
│                    클라이언트 계층                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────────┐   │
│  │ Tauri App │  │ Web App  │  │ 메시징 채널 (카카오톡 등)   │   │
│  │(Win/Mac/  │  │(Next.js) │  │ (Telegram/Discord/Slack)  │   │
│  │Linux/iOS/ │  │          │  │                           │   │
│  │Android)   │  │          │  │                           │   │
│  └─────┬─────┘  └────┬─────┘  └──────────┬───────────────┘   │
│        │              │                   │                   │
└────────┼──────────────┼───────────────────┼───────────────────┘
         │              │                   │
         ▼              ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                   Gateway (Axum HTTP/WS)                     │
│  /webhook  /health  /pair  /sync  /api/auth/*  /api/voice/* │
├─────────────────────────────────────────────────────────────┤
│                   보안 계층 (Security)                        │
│  페어링 가드 · 정책 엔진 · 암호화 볼트 · 위협 탐지           │
├─────────────────────────────────────────────────────────────┤
│                   에이전트 오케스트레이션                       │
│  Agent Loop · SLM Gatekeeper · Task Routing                  │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│ Provider │ Tools    │ Memory   │ Channels │ Peripherals     │
│ (LLM)   │ (27+)   │ (SQLite) │ (15)     │ (Hardware)      │
└──────────┴──────────┴──────────┴──────────┴─────────────────┘
```

---

## 2. 기술 스택

### 2.1 백엔드 (에이전트 런타임)

| 구분 | 기술 | 버전/상세 |
|------|------|----------|
| **언어** | Rust | 2021 edition |
| **HTTP 프레임워크** | Axum | 비동기 HTTP/WebSocket 서버 |
| **비동기 런타임** | Tokio | 멀티스레드 비동기 I/O |
| **CLI 프레임워크** | Clap | 커맨드 라우팅 |
| **데이터베이스** | SQLite (rusqlite) | 인증, 메모리, 스케줄링 |
| **직렬화** | serde / serde_json / toml | JSON/TOML 처리 |
| **HTTP 클라이언트** | reqwest | rustls-tls 기반 |
| **암호화** | AES-256-GCM, ChaCha20-Poly1305, SHA-256, PBKDF2 | 보안 계층 전반 |
| **빌드 최적화** | LTO, codegen-units=1, opt-level="s", strip | 바이너리 크기 최소화 |

### 2.2 데스크톱/모바일 클라이언트 (Tauri)

| 구분 | 기술 |
|------|------|
| **프레임워크** | Tauri 2 |
| **프론트엔드** | React 18.3 + TypeScript 5.6 |
| **빌드 도구** | Vite 6.0 |
| **지원 플랫폼** | Windows (x64), macOS (Universal), Linux (AppImage/deb/rpm), Android (ARM64), iOS (ARM64) |
| **창 크기** | 1200x800 (최소 480x600) |

### 2.3 웹 클라이언트

| 구분 | 기술 |
|------|------|
| **프레임워크** | Next.js 14 (App Router) |
| **스타일링** | Tailwind CSS 3.4 |
| **호스팅** | Vercel |

### 2.4 인프라

| 구분 | 기술 |
|------|------|
| **서버 호스팅** | Railway (Docker 배포) |
| **웹 호스팅** | Vercel |
| **컨테이너** | Docker (멀티스테이지 빌드, Debian bookworm-slim) |
| **CI/CD** | GitHub Actions |

---

## 3. 모듈 구조 (40+ 모듈)

### 3.1 핵심 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **main** | `src/main.rs` | CLI 진입점, 커맨드 라우팅 |
| **lib** | `src/lib.rs` | 모듈 내보내기, 공유 열거형 |
| **config** | `src/config/` | 설정 스키마, 로딩/병합, TOML 파싱 |
| **agent** | `src/agent/` | 에이전트 오케스트레이션 루프 |
| **gateway** | `src/gateway/` | HTTP/WebSocket 게이트웨이 서버 |

### 3.2 보안 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **security** | `src/security/` | 정책 엔진, 페어링 가드, 암호화 볼트, 위협 탐지 |
| **auth** | `src/auth/` | 다중 사용자 인증, 세션 관리, 디바이스 관리 |
| **sandbox** | `src/sandbox/` | 도구 실행 샌드박스 격리 |
| **approval** | `src/approval/` | 작업 승인 시스템 |

### 3.3 AI/LLM 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **providers** | `src/providers/` | LLM 프로바이더 (8개 구현체 + 신뢰성 래퍼) |
| **gatekeeper** | `src/gatekeeper/` | SLM 기반 의도 분류 게이트키퍼 |
| **rag** | `src/rag/` | RAG(검색 증강 생성) 모듈 |
| **skills** | `src/skills/` | 스킬 정의 및 실행 |
| **skillforge** | `src/skillforge/` | 스킬 생성/관리 |

### 3.4 도구/실행 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **tools** | `src/tools/` | 27개 이상의 자율 도구 |
| **cron** | `src/cron/` | 크론 작업 스케줄러 |
| **voice** | `src/voice/` | 실시간 음성 통역 |

### 3.5 통신/채널 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **channels** | `src/channels/` | 15개 메시징 채널 |
| **tunnel** | `src/tunnel/` | 터널링 (외부 접근) |
| **integrations** | `src/integrations/` | 외부 서비스 통합 |

### 3.6 데이터/메모리 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **memory** | `src/memory/` | 메모리 백엔드 (SQLite, Markdown, Lucid, Vector, 동기화) |
| **sync** | `src/sync/` | 멀티 디바이스 동기화 (3계층) |

### 3.7 인프라/운영 모듈

| 모듈 | 경로 | 설명 |
|------|------|------|
| **runtime** | `src/runtime/` | 런타임 어댑터 (Native, Mobile) |
| **peripherals** | `src/peripherals/` | 하드웨어 주변장치 (STM32, RPi GPIO) |
| **hardware** | `src/hardware/` | 하드웨어 추상화 |
| **observability** | `src/observability/` | 옵저버 인터페이스 |
| **telemetry** | `src/telemetry/` | 텔레메트리 수집 |
| **billing** | `src/billing/` | 결제/과금 시스템 |
| **cost** | `src/cost/` | LLM API 비용 추적 |
| **health** | `src/health/` | 헬스 체크 |
| **heartbeat** | `src/heartbeat/` | 하트비트 모니터링 |
| **daemon** | `src/daemon/` | 데몬 서비스 관리 |
| **service** | `src/service/` | 시스템 서비스 등록 |
| **doctor** | `src/doctor/` | 자가 진단 |
| **onboard** | `src/onboard/` | 온보딩 마법사 |
| **migration** | `src/migration.rs` | 데이터 마이그레이션 |

---

## 4. LLM 프로바이더 (8개 구현체)

| 프로바이더 | 파일 | 지원 모델 | 상태 |
|-----------|------|----------|------|
| **Anthropic** | `anthropic.rs` | Claude 3.5/4 시리즈 | 프로덕션 |
| **OpenAI** | `openai.rs` | GPT-4o/4.1 시리즈 | 프로덕션 |
| **Google Gemini** | `gemini.rs` | Gemini 2.5 Flash/Pro | 프로덕션 |
| **Ollama** | `ollama.rs` | 로컬 모델 (Llama, Mistral 등) | 프로덕션 |
| **OpenRouter** | `openrouter.rs` | 다중 모델 라우팅 | 프로덕션 |
| **GitHub Copilot** | `copilot.rs` | Copilot 모델 | 프로덕션 |
| **OpenAI Compatible** | `compatible.rs` | OpenAI API 호환 서버 | 프로덕션 |
| **Router** | `router.rs` | 서브 프로바이더 라우팅 | 프로덕션 |

**추가 래퍼:**
- **ReliableProvider** (`reliable.rs`): 자동 재시도, 폴백, 타임아웃 처리

---

## 5. 자율 도구 (27개)

### 5.1 파일/시스템 도구

| 도구 | 설명 |
|------|------|
| `shell` | 쉘 명령 실행 (샌드박스 격리) |
| `file_read` | 파일 읽기 |
| `file_write` | 파일 쓰기 |
| `screenshot` | 스크린샷 캡처 |
| `git_operations` | Git 작업 (commit, push, branch 등) |

### 5.2 웹/네트워크 도구

| 도구 | 설명 |
|------|------|
| `browser` | 헤드리스 브라우저 조작 (크롤링, 상호작용) |
| `browser_open` | URL 열기 |
| `http_request` | HTTP 요청 수행 |

### 5.3 메모리 도구

| 도구 | 설명 |
|------|------|
| `memory_store` | 정보 기억 저장 |
| `memory_recall` | 기억 검색/회상 |
| `memory_forget` | 기억 삭제 |

### 5.4 스케줄링 도구

| 도구 | 설명 |
|------|------|
| `schedule` | 일정 예약 |
| `cron_add` | 반복 작업 추가 |
| `cron_list` | 작업 목록 조회 |
| `cron_remove` | 작업 삭제 |
| `cron_update` | 작업 수정 |
| `cron_run` | 작업 즉시 실행 |
| `cron_runs` | 실행 이력 조회 |

### 5.5 미디어/정보 도구

| 도구 | 설명 |
|------|------|
| `image_info` | 이미지 분석/정보 추출 |

### 5.6 통신/알림 도구

| 도구 | 설명 |
|------|------|
| `pushover` | 푸시 알림 전송 |
| `delegate` | 서브 에이전트 위임 |

### 5.7 외부 통합 도구

| 도구 | 설명 |
|------|------|
| `composio` | 외부 서비스 통합 (API 연동) |

### 5.8 하드웨어 도구

| 도구 | 설명 |
|------|------|
| `hardware_board_info` | 보드 정보 조회 |
| `hardware_memory_map` | 메모리 맵 조회 |
| `hardware_memory_read` | 메모리 읽기 |

---

## 6. 메시징 채널 (15개)

### 6.1 프로덕션 채널

| 채널 | 연결 방식 | 인증 | 주요 기능 |
|------|----------|------|----------|
| **KakaoTalk** | Webhook + REST | OAuth2 | 리치 메시지, 알림톡, 페어링 |
| **Telegram** | Long-polling | Bot Token | 미디어 첨부, 메시지 분할 |
| **Discord** | WebSocket Gateway | Bot Token | 멘션 모드, 채널 필터링 |
| **Slack** | HTTP Polling | Bearer Token | 대화 이력 폴링 |
| **WhatsApp** | Webhook + REST | Bearer Token | Meta Cloud API |
| **LINE** | Webhook + REST | Channel Token | HMAC-SHA256 검증 |
| **iMessage** | Native (AppleScript) | 로컬 접근 | macOS 전용, DB 폴링 |
| **Signal** | HTTP SSE + JSON-RPC | 로컬 HTTP | signal-cli 데몬 연동 |
| **Email** | IMAP + SMTP | Username/Password | TLS, MIME 파싱 |
| **IRC** | TLS TCP | SASL/NickServ | 다중 채널 |
| **Matrix** | HTTP Long-polling | Access Token | /sync API |
| **QQ** | WebSocket | OAuth2 | 텐센트 API |
| **DingTalk** | WebSocket Stream | Client ID/Secret | 스트림 모드 |
| **Lark** | WebSocket | OAuth2 | Protobuf 코덱 |

### 6.2 디버그 채널

| 채널 | 설명 |
|------|------|
| **CLI** | stdin/stdout 기반 테스트 채널 |

---

## 7. 메모리 시스템

### 7.1 메모리 백엔드

| 백엔드 | 설명 | 상태 |
|--------|------|------|
| **SQLite** | 기본 영구 메모리 (sqlite.rs) | 프로덕션 |
| **Markdown** | 파일 기반 메모리 (markdown.rs) | 프로덕션 |
| **Lucid** | 경량 메모리 (lucid.rs) | 프로덕션 |
| **None** | 영구 메모리 비활성화 (none.rs) | 프로덕션 |

### 7.2 메모리 부가 기능

| 기능 | 파일 | 설명 |
|------|------|------|
| **Synced** | `synced.rs` | 멀티 디바이스 동기화 래퍼 |
| **Vector** | `vector.rs` | 벡터 임베딩 검색 |
| **Embeddings** | `embeddings.rs` | 임베딩 생성 |
| **Chunker** | `chunker.rs` | 텍스트 청킹 |
| **Snapshot** | `snapshot.rs` | 메모리 스냅샷 |
| **Response Cache** | `response_cache.rs` | 응답 캐싱 |
| **Hygiene** | `hygiene.rs` | 메모리 정리 |

---

## 8. 게이트웨이 API 엔드포인트

### 8.1 공개 엔드포인트

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/health` | 서버 상태 확인 |
| POST | `/webhook` | 메시지 수신 (멱등성 지원) |
| POST | `/pair` | 페어링 코드 교환 |
| GET | `/api/agent/info` | 사용 가능한 채널/도구 목록 |
| GET | `/api/navigation` | 웹 UI 네비게이션 매니페스트 |

### 8.2 인증 엔드포인트

| 메서드 | 경로 | 설명 |
|--------|------|------|
| POST | `/api/auth/register` | 회원가입 |
| POST | `/api/auth/login` | 로그인 (세션 토큰 발급) |
| POST | `/api/auth/logout` | 로그아웃 (세션 취소) |
| GET | `/api/auth/me` | 현재 사용자 정보 |
| GET | `/api/auth/devices` | 디바이스 목록 |
| POST | `/api/auth/devices` | 디바이스 등록 |
| DELETE | `/api/auth/devices/{id}` | 디바이스 삭제 |
| PUT | `/api/auth/devices/{id}/pairing-code` | 디바이스 페어링 코드 설정 |
| POST | `/api/auth/devices/{id}/verify-pairing` | 페어링 코드 검증 |
| POST | `/api/auth/heartbeat` | 디바이스 하트비트 |

### 8.3 동기화 엔드포인트

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/sync` | WebSocket 실시간 동기화 |
| POST | `/api/sync/relay` | 암호화 데이터 업로드 |
| GET | `/api/sync/relay` | 대기 중 데이터 수신 |

### 8.4 음성/통역 엔드포인트

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/api/voice/ui` | 통역 UI 매니페스트 (25개 언어) |
| POST | `/api/voice/sessions` | 음성 세션 생성 |
| GET | `/api/voice/sessions` | 활성 세션 목록 |
| GET | `/api/voice/interpret` | WebSocket 실시간 양방향 통역 |

### 8.5 채널별 웹훅

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET/POST | `/whatsapp` | WhatsApp 웹훅 |
| POST | `/line` | LINE 웹훅 |
| GET/POST | `/pair/auto/{token}` | 원클릭 자동 페어링 |
| GET/POST | `/pair/signup` | 채널 회원가입 |

### 8.6 관리/텔레메트리

| 메서드 | 경로 | 설명 |
|--------|------|------|
| POST | `/api/telemetry/events` | 텔레메트리 이벤트 수집 |
| GET | `/api/admin/telemetry/events` | 관리자 텔레메트리 조회 |
| GET | `/api/admin/telemetry/summary` | 텔레메트리 요약 |
| GET | `/api/admin/telemetry/alerts` | 텔레메트리 알림 |

---

## 9. 보안 아키텍처

### 9.1 인증 (3계층)

| 계층 | 방식 | 설명 |
|------|------|------|
| **1단계** | 페어링 코드 | Bluetooth 스타일 일회용 코드 (6자리) |
| **2단계** | 사용자 계정 | Username/Password 다중 사용자 인증 |
| **3단계** | 디바이스 인증 | 디바이스별 페어링 코드 + 세션 토큰 |

### 9.2 세션 관리

- **토큰 형식**: 32바이트 랜덤 (64자 Hex), 서버 측 SHA-256 해시 저장
- **TTL**: 기본 30일 (설정 가능)
- **저장 방식**: SQLite (`auth.db`), 평문 저장 안 함
- **타이밍 공격 방어**: 상수 시간 비교

### 9.3 비밀번호 보안

- **해싱**: SHA-256 반복 10만회 + 사용자별 랜덤 솔트
- **최소 길이**: 8자
- **사용자명**: 대소문자 무시, 1-64자

### 9.4 정책 엔진

| 기능 | 설명 |
|------|------|
| **자율성 수준** | ReadOnly / Supervised / Full |
| **행동 속도 제한** | 버스트 20/분, 시간당 600/시 |
| **진행적 스트라이크** | 1회 30초, 2회 2분, 3회 10분 잠금 |
| **명령 위험 평가** | Low(읽기) / Medium(쓰기) / High(쉘/삭제) |

### 9.5 네트워크 보안

| 기능 | 설명 |
|------|------|
| **속도 제한** | IP별 슬라이딩 윈도우 (페어링 10/분, 웹훅 30/분) |
| **요청 크기** | 최대 64KB |
| **타임아웃** | 요청당 120초 |
| **CORS** | 모든 오리진 허용 (Tauri/웹 클라이언트용) |
| **웹훅 시크릿** | X-Webhook-Secret 헤더 SHA-256 검증 |
| **WhatsApp 검증** | HMAC-SHA256 서명 (X-Hub-Signature-256) |

### 9.6 암호화 볼트

- **알고리즘**: AES-256-GCM (시크릿별 랜덤 IV)
- **키 파생**: PBKDF2-SHA256 (마스터 비밀번호 기반)
- **저장**: SQLite `secrets` 테이블

---

## 10. 멀티 디바이스 동기화 (3계층)

| 계층 | 이름 | 설명 |
|------|------|------|
| **Layer 1** | Real-Time Relay | TTL 기반 인메모리 저장 (5분), E2E 암호화, 디바이스당 최대 100개 |
| **Layer 2** | Delta Journal + Version Vectors | 오프라인 추적 (증분 델타), 디바이스별 Lamport 타임스탬프 |
| **Layer 3** | Manifest-Based Full Sync | 장기 오프라인/신규 디바이스용 전체 동기화 |

- **프로토콜**: WebSocket (`/sync`) 브로드캐스트 릴레이
- **메시지 타입**: RelayNotify, SyncRequest, SyncResponse, DeltaAck, FullSyncRequest, FullSyncManifestResponse, FullSyncData, FullSyncComplete
- **서버 불투명**: 서버는 암호화 페이로드를 읽을 수 없음 (E2E)

---

## 11. 클라이언트 기능 매트릭스

| 기능 | Tauri App | Web App |
|------|-----------|---------|
| 채팅 인터페이스 | O | O |
| 대화 이력 | O (로컬) | O (localStorage) |
| 다국어 (한/영) | O (120+ 키) | O (부분) |
| 디바이스 관리 | O | X |
| 동기화 상태 표시 | O | X |
| 서버 설정 | O | X |
| 페어링 UI | O | X |
| 마케팅 페이지 | X | O |
| 다운로드 페이지 | X | O |
| 자동 업데이트 | O | 해당 없음 |

---

## 12. 배포 구성

### 12.1 Docker (Railway)

- **빌더**: `rust:1.83-slim-bookworm` (멀티스테이지)
- **런타임**: `debian:bookworm-slim`
- **실행 사용자**: `zeroclaw` (비루트)
- **포트**: 8080
- **헬스체크**: GET `/health`
- **재시작**: ON_FAILURE (최대 5회)

### 12.2 설정 환경변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `ZEROCLAW_HOST` | 바인드 주소 | 0.0.0.0 |
| `ZEROCLAW_PORT` | 게이트웨이 포트 | 8080 |
| `PORT` | Railway 주입 포트 | 8080 |
| `ZEROCLAW_WORKSPACE` | 작업 디렉토리 | ~/.zeroclaw/workspace |

---

## 13. 비용 추적 및 과금

| 모듈 | 설명 |
|------|------|
| `src/cost/` | LLM API 호출별 비용 추적 (SQLite 영구 저장) |
| `src/billing/` | 결제/과금 시스템 (SQLite 기반) |

---

## 14. 추가 시스템

| 시스템 | 경로 | 설명 |
|--------|------|------|
| **RAG** | `src/rag/` | 검색 증강 생성 (문서 기반 응답) |
| **Cron 스케줄러** | `src/cron/` | 반복 작업 스케줄링 (SQLite 저장) |
| **음성 통역** | `src/voice/` | 실시간 양방향 음성 통역 (25개 언어) |
| **텔레메트리** | `src/telemetry/` | 사용 패턴 수집/분석 |
| **온보딩** | `src/onboard/` | 초기 설정 마법사 |
| **데몬** | `src/daemon/` | 시스템 서비스 (자동 시작/재시작) |
| **터널** | `src/tunnel/` | 외부 접근 터널링 |
| **하드웨어** | `src/peripherals/` | STM32, RPi GPIO 하드웨어 연동 |

---

*본 문서는 MoA_new 저장소 코드 분석을 기반으로 자동 생성되었습니다.*
