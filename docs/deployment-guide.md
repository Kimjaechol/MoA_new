# MoA (Master of AI) 배포 및 설정 가이드

> 이 가이드는 초보자도 따라할 수 있도록 최대한 상세하게 작성되었습니다.

## 목차

1. [전체 아키텍처 개요](#1-전체-아키텍처-개요)
2. [사전 준비물](#2-사전-준비물)
3. [환경변수 총정리](#3-환경변수-총정리)
4. [Railway 백엔드 배포](#4-railway-백엔드-배포)
5. [Vercel 홈페이지 배포](#5-vercel-홈페이지-배포)
6. [Cloudflare R2 설정](#6-cloudflare-r2-설정)
7. [네이티브 앱 빌드](#7-네이티브-앱-빌드)
8. [앱 배포 및 다운로드 링크 연결](#8-앱-배포-및-다운로드-링크-연결)
9. [문제 해결 FAQ](#9-문제-해결-faq)

---

## 1. 전체 아키텍처 개요

```
┌─────────────────────────────────────────────────────────────────┐
│                        사용자 (User)                             │
│                                                                  │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│   │ Windows  │  │  macOS   │  │  Linux   │  │ Android  │       │
│   │ 데스크탑  │  │ 데스크탑  │  │ 데스크탑  │  │   앱     │       │
│   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│        └──────────────┼──────────────┼──────────────┘            │
│                       ▼                                          │
│              ┌─────────────────┐    ┌──────────────────┐        │
│              │  Vercel 홈페이지  │    │  Cloudflare R2   │        │
│              │  (웹 채팅 + 홍보) │    │  (앱 다운로드)    │        │
│              └────────┬────────┘    └──────────────────┘        │
│                       ▼                                          │
│              ┌─────────────────┐                                 │
│              │  Railway 백엔드  │                                 │
│              │  (ZeroClaw API) │                                 │
│              └────────┬────────┘                                 │
│                       ▼                                          │
│              ┌─────────────────┐                                 │
│              │  AI Provider    │                                 │
│              │  (OpenRouter/   │                                 │
│              │   Anthropic 등) │                                 │
│              └─────────────────┘                                 │
└─────────────────────────────────────────────────────────────────┘
```

**데이터 흐름:**
1. 사용자가 네이티브 앱 또는 웹 채팅에서 메시지 입력
2. Railway의 ZeroClaw API 서버로 전송 (`POST /webhook`)
3. ZeroClaw가 AI 모델에 요청 (OpenRouter/Anthropic 등)
4. 응답을 사용자에게 반환

**비용 구조:**
- **Railway**: 월 $5 기본 + 사용량 (소규모 트래픽이면 무료 tier 가능)
- **Vercel**: 무료 tier (월 100GB 대역폭)
- **Cloudflare R2**: 저장 10GB 무료, 전송(egress) 완전 무료!
- **AI API**: 사용량 기반 (OpenRouter가 가장 저렴)

---

## 2. 사전 준비물

### 계정 생성 (모두 무료)

| 서비스 | 가입 URL | 용도 |
|--------|----------|------|
| GitHub | https://github.com | 코드 저장, CI/CD |
| Railway | https://railway.app | 백엔드 서버 |
| Vercel | https://vercel.com | 홈페이지 |
| Cloudflare | https://cloudflare.com | R2 파일 저장 |
| OpenRouter | https://openrouter.ai | AI API 키 |

### 로컬 개발 도구 설치

```bash
# 1. Rust 설치 (백엔드 빌드)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Node.js 18+ 설치 (프론트엔드 빌드)
# macOS: brew install node
# Ubuntu: sudo apt install nodejs npm
# Windows: https://nodejs.org 에서 다운로드

# 3. Railway CLI 설치
npm install -g @railway/cli

# 4. Vercel CLI 설치
npm install -g vercel

# 5. AWS CLI 설치 (R2 업로드용)
# macOS: brew install awscli
# Ubuntu: sudo apt install awscli
# Windows: https://aws.amazon.com/cli/ 에서 다운로드

# 6. Tauri 앱 빌드 도구 (선택)
# macOS: xcode-select --install
# Ubuntu: sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
# Windows: Visual Studio Build Tools 설치 (C++ workload)
cargo install tauri-cli
```

---

## 3. 환경변수 총정리

### 3.1 Railway 백엔드 (ZeroClaw 서버)

| 환경변수 | 필수 | 기본값 | 설명 |
|---------|------|--------|------|
| `ZEROCLAW_API_KEY` | ✅ | - | AI 모델 API 키 (OpenRouter 또는 Anthropic) |
| `ZEROCLAW_DEFAULT_PROVIDER` | ❌ | `openrouter` | AI 제공자 (`openrouter`, `anthropic`, `openai`) |
| `ZEROCLAW_DEFAULT_MODEL` | ❌ | `anthropic/claude-sonnet-4` | 사용할 AI 모델 이름 |
| `ZEROCLAW_HOST` | ❌ | `0.0.0.0` | 서버 바인드 주소 (Railway에서는 반드시 `0.0.0.0`) |
| `ZEROCLAW_PORT` | ❌ | `3000` | 서버 포트 (Railway의 `PORT` 변수와 일치) |
| `PORT` | ✅ | `3000` | Railway가 자동 설정하는 포트 |
| `ZEROCLAW_REQUIRE_PAIRING` | ❌ | `true` | 페어링 인증 필수 여부 |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | ❌ | `false` | 공개 바인드 허용 (Railway에서는 `true`) |
| `ZEROCLAW_WEBHOOK_SECRET` | ❌ | - | 웹훅 시크릿 (추가 보안) |
| `ZEROCLAW_MEMORY_BACKEND` | ❌ | `sqlite` | 메모리 백엔드 (`sqlite`, `markdown`, `none`) |
| `ZEROCLAW_WHATSAPP_APP_SECRET` | ❌ | - | WhatsApp 앱 시크릿 |

### 3.2 Vercel 홈페이지

| 환경변수 | 필수 | 기본값 | 설명 |
|---------|------|--------|------|
| `NEXT_PUBLIC_API_URL` | ✅ | - | Railway 백엔드 URL (예: `https://your-app.railway.app`) |
| `NEXT_PUBLIC_R2_BASE_URL` | ✅ | - | R2 다운로드 URL (예: `https://downloads.moa.ai`) |
| `NEXT_PUBLIC_GA_ID` | ❌ | - | Google Analytics ID |

### 3.3 Cloudflare R2 (업로드용)

| 환경변수 | 필수 | 설명 |
|---------|------|------|
| `R2_ACCOUNT_ID` | ✅ | Cloudflare 계정 ID |
| `R2_ACCESS_KEY_ID` | ✅ | R2 API 토큰 Access Key |
| `R2_SECRET_ACCESS_KEY` | ✅ | R2 API 토큰 Secret Key |
| `R2_BUCKET_NAME` | ✅ | R2 버킷 이름 (예: `moa-downloads`) |
| `R2_ENDPOINT` | ✅ | R2 엔드포인트 (예: `https://<account_id>.r2.cloudflarestorage.com`) |

### 3.4 GitHub Actions (CI/CD)

| 시크릿 이름 | 용도 |
|------------|------|
| `R2_ACCESS_KEY_ID` | R2 업로드용 |
| `R2_SECRET_ACCESS_KEY` | R2 업로드용 |
| `R2_ENDPOINT` | R2 엔드포인트 |
| `R2_BUCKET_NAME` | R2 버킷 이름 |

---

## 4. Railway 백엔드 배포

### 단계별 가이드

#### 4.1 Railway 계정 생성 및 프로젝트 생성

1. https://railway.app 에서 GitHub 계정으로 로그인
2. 대시보드에서 **"New Project"** 클릭
3. **"Deploy from GitHub Repo"** 선택
4. 이 저장소(`MoA_new`)를 선택

#### 4.2 환경변수 설정

Railway 대시보드에서:
1. 프로젝트 → **Variables** 탭 클릭
2. 아래 변수들을 추가:

```
ZEROCLAW_API_KEY=sk-or-v1-xxxxx        # OpenRouter에서 발급받은 키
ZEROCLAW_DEFAULT_PROVIDER=openrouter
ZEROCLAW_DEFAULT_MODEL=anthropic/claude-sonnet-4
ZEROCLAW_HOST=0.0.0.0
ZEROCLAW_ALLOW_PUBLIC_BIND=true
ZEROCLAW_REQUIRE_PAIRING=true
```

#### 4.3 배포 설정

Railway 대시보드 → **Settings** 탭:
1. **Root Directory**: `/` (프로젝트 루트)
2. **Build Command**: (Dockerfile 사용시 자동)
3. **Custom Dockerfile Path**: `deploy/railway/Dockerfile`
4. **Health Check Path**: `/health`

#### 4.4 배포 확인

```bash
# Railway가 할당한 URL로 접속 테스트
curl https://your-app.railway.app/health
# 응답: {"status":"ok","paired":false,"runtime":{...}}
```

#### 4.5 CLI로 배포하기 (대안)

```bash
# Railway CLI 로그인
railway login

# 프로젝트 연결
railway link

# 환경변수 설정
railway variables set ZEROCLAW_API_KEY=sk-or-v1-xxxxx
railway variables set ZEROCLAW_HOST=0.0.0.0
railway variables set ZEROCLAW_ALLOW_PUBLIC_BIND=true

# 배포
railway up
```

---

## 5. Vercel 홈페이지 배포

### 단계별 가이드

#### 5.1 Vercel 프로젝트 생성

```bash
# clients/web 디렉토리로 이동
cd clients/web

# 의존성 설치
npm install

# 로컬 테스트
npm run dev
# http://localhost:3000 에서 확인
```

#### 5.2 Vercel에 배포

**방법 1: CLI 사용 (추천)**
```bash
cd clients/web

# Vercel 로그인
vercel login

# 배포 (첫 배포시 프로젝트 설정 물음)
vercel

# 프로덕션 배포
vercel --prod
```

**방법 2: GitHub 연동 (자동 배포)**
1. https://vercel.com/new 접속
2. GitHub 저장소 가져오기
3. **Root Directory**: `clients/web` 설정
4. **Framework**: Next.js 자동 감지
5. 환경변수 추가 후 **Deploy** 클릭

#### 5.3 환경변수 설정

Vercel 대시보드 → Settings → Environment Variables:

```
NEXT_PUBLIC_API_URL=https://your-app.railway.app
NEXT_PUBLIC_R2_BASE_URL=https://downloads.your-domain.com
```

#### 5.4 커스텀 도메인 (선택)

Vercel 대시보드 → Settings → Domains:
1. 도메인 추가 (예: `moa.ai`)
2. DNS 설정: CNAME `cname.vercel-dns.com`

---

## 6. Cloudflare R2 설정

### R2의 장점
- **저장 용량**: 10GB/월 무료
- **전송(Egress)**: **완전 무료!** (수십만 명이 다운로드해도 추가 비용 없음)
- **S3 호환**: 기존 AWS CLI 도구로 사용 가능

### 단계별 가이드

#### 6.1 R2 버킷 생성

1. https://dash.cloudflare.com 로그인
2. 왼쪽 메뉴 → **R2 Object Storage** 클릭
3. **Create bucket** 클릭
4. 버킷 이름: `moa-downloads`
5. 위치: **APAC** (아시아 사용자 대상) 또는 **Auto**
6. **Create bucket** 클릭

#### 6.2 API 토큰 생성

1. R2 → **Manage R2 API Tokens** 클릭
2. **Create API token** 클릭
3. 권한: **Object Read & Write**
4. 대상 버킷: `moa-downloads`
5. **Create API Token** 클릭
6. **Access Key ID**와 **Secret Access Key**를 안전한 곳에 저장!
   (이 화면을 벗어나면 다시 볼 수 없음)

#### 6.3 퍼블릭 액세스 설정

1. 버킷 → **Settings** 탭
2. **Public Access** → **Allow Access** 활성화
3. **Custom Domain** (선택):
   - `downloads.your-domain.com` 입력
   - Cloudflare DNS에 자동으로 CNAME 추가됨

또는 **R2.dev subdomain** 사용:
   - Settings → R2.dev subdomain → Enable
   - `pub-xxxxx.r2.dev` 형식의 URL 생성됨

#### 6.4 CORS 설정

버킷 → Settings → CORS Policy:
```json
[
  {
    "AllowedOrigins": ["*"],
    "AllowedMethods": ["GET", "HEAD"],
    "AllowedHeaders": ["*"],
    "MaxAgeSeconds": 86400
  }
]
```

#### 6.5 파일 업로드

```bash
# 환경변수 설정
export R2_ACCOUNT_ID="your_account_id"
export R2_ACCESS_KEY_ID="your_access_key"
export R2_SECRET_ACCESS_KEY="your_secret_key"
export R2_BUCKET_NAME="moa-downloads"
export R2_ENDPOINT="https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com"

# AWS CLI로 업로드
aws s3 cp ./MoA-1.0.0-x64.msi \
  s3://${R2_BUCKET_NAME}/releases/latest/MoA-windows-x64.msi \
  --endpoint-url ${R2_ENDPOINT}

aws s3 cp ./MoA-1.0.0-x64.dmg \
  s3://${R2_BUCKET_NAME}/releases/latest/MoA-macos-x64.dmg \
  --endpoint-url ${R2_ENDPOINT}

# 또는 업로드 스크립트 사용
bash deploy/r2/upload.sh
```

#### 6.6 비용 예상

| 항목 | 무료 포함 | 초과시 비용 |
|------|----------|------------|
| 저장용량 | 10 GB/월 | $0.015/GB |
| Class A 작업 (쓰기) | 1백만/월 | $4.50/백만 |
| Class B 작업 (읽기) | 10백만/월 | $0.36/백만 |
| **전송(Egress)** | **무제한** | **$0 (무료!)** |

> 💡 **핵심**: 앱 파일을 R2에 올리면, 수십만 명이 다운로드해도 전송 비용이 $0입니다!
> 앱 파일이 총 1GB 미만이라면, 월 비용은 사실상 $0입니다.

---

## 7. 네이티브 앱 빌드

### 7.1 데스크탑 앱 (Windows, macOS, Linux)

```bash
# Tauri 앱 디렉토리로 이동
cd clients/tauri

# 의존성 설치
npm install

# 개발 모드 (로컬 테스트)
npm run tauri dev

# 프로덕션 빌드
npm run tauri build
```

빌드 결과물 위치:
- **Windows**: `src-tauri/target/release/bundle/msi/MoA_*.msi`
- **macOS**: `src-tauri/target/release/bundle/dmg/MoA_*.dmg`
- **Linux**: `src-tauri/target/release/bundle/deb/moa_*.deb`
- **Linux**: `src-tauri/target/release/bundle/appimage/MoA_*.AppImage`

### 7.2 모바일 앱

```bash
# Android 빌드 준비
# 1. Android Studio 설치
# 2. Android SDK 및 NDK 설치
# 3. ANDROID_HOME 환경변수 설정

npm run tauri android init
npm run tauri android build

# iOS 빌드 준비 (macOS만 가능)
# 1. Xcode 설치
# 2. iOS 시뮬레이터 설정

npm run tauri ios init
npm run tauri ios build
```

### 7.3 GitHub Actions 자동 빌드

태그를 푸시하면 GitHub Actions가 자동으로 모든 플랫폼 빌드:

```bash
# 버전 태그 생성 및 푸시
git tag v1.0.0
git push origin v1.0.0
```

이후 GitHub → Actions 탭에서 빌드 진행 확인 가능.

---

## 8. 앱 배포 및 다운로드 링크 연결

### 전체 흐름

1. **빌드**: GitHub Actions가 모든 플랫폼 앱을 자동 빌드
2. **업로드**: 빌드된 파일이 Cloudflare R2에 자동 업로드
3. **링크**: Vercel 홈페이지의 다운로드 페이지가 R2 URL을 참조
4. **다운로드**: 사용자가 홈페이지에서 자신의 OS에 맞는 앱 다운로드

### Vercel에서 R2 URL 설정

Vercel 환경변수에 R2 퍼블릭 URL 설정:
```
NEXT_PUBLIC_R2_BASE_URL=https://pub-xxxxx.r2.dev
```

또는 커스텀 도메인 사용:
```
NEXT_PUBLIC_R2_BASE_URL=https://downloads.moa.ai
```

---

## 9. 문제 해결 FAQ

### Q: Railway에서 빌드가 실패해요
- Dockerfile의 Rust 버전 확인 (1.83 이상)
- 메모리 부족: Railway 플랜 확인 (Hobby 플랜은 8GB RAM)
- 빌드 로그에서 누락된 시스템 라이브러리 확인

### Q: Vercel에서 API 호출이 안돼요
- CORS 설정 확인 (게이트웨이에 이미 추가됨)
- `NEXT_PUBLIC_API_URL`이 정확한지 확인
- Railway URL에 `https://` 포함 확인

### Q: R2 업로드가 안돼요
- API 토큰의 Access Key가 정확한지 확인
- 엔드포인트 URL 형식: `https://<account_id>.r2.cloudflarestorage.com`
- AWS CLI 프로필에 `--endpoint-url` 필수

### Q: 페어링은 어떻게 하나요?
1. Railway에 서버 배포 후, 서버 로그에서 6자리 페어링 코드 확인
2. 앱 설정에서 서버 URL 입력 후 페어링 코드 입력
3. 발급된 토큰이 자동 저장됨

### Q: 페어링 없이 사용하려면?
Railway 환경변수에 추가:
```
ZEROCLAW_REQUIRE_PAIRING=false
```
> ⚠️ 보안 위험: 누구나 API에 접근 가능해집니다

### Q: AI API 키는 어디서 얻나요?
- **OpenRouter** (추천, 다양한 모델): https://openrouter.ai/keys
- **Anthropic** (Claude 직접): https://console.anthropic.com/settings/keys
- **OpenAI** (GPT): https://platform.openai.com/api-keys

### Q: 비용을 최소화하려면?
1. **Railway**: Hobby 플랜 ($5/월) 사용, 유휴시 자동 슬립
2. **Vercel**: 무료 tier 충분 (월 100GB)
3. **R2**: 전송 무료, 저장 10GB 무료
4. **AI API**: OpenRouter에서 저렴한 모델 선택
   - `meta-llama/llama-3.1-8b-instruct` (매우 저렴)
   - `anthropic/claude-sonnet-4` (성능/비용 밸런스)

---

## 빠른 시작 (5분 배포)

```bash
# 1. Railway 배포
railway login
railway init
railway variables set ZEROCLAW_API_KEY=your_key_here
railway variables set ZEROCLAW_HOST=0.0.0.0
railway variables set ZEROCLAW_ALLOW_PUBLIC_BIND=true
railway up

# 2. Vercel 배포
cd clients/web
npm install
vercel --prod
# 환경변수 설정: NEXT_PUBLIC_API_URL=https://your-app.railway.app

# 3. 완료! 홈페이지에서 웹 채팅 사용 가능
```
