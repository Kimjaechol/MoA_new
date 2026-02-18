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

아래에서 자신의 운영체제에 맞는 탭을 따라하세요.

#### Windows에서 설치 (초보자 가이드)

윈도우 노트북에서 앱을 빌드하려면 아래 순서대로 설치하세요.
모든 설치 프로그램은 "다음 → 다음 → 완료" 방식으로 진행하면 됩니다.

**1단계: Git 설치 (코드 다운로드용)**

1. https://git-scm.com/download/win 접속
2. **"Click here to download"** 클릭하여 설치 파일 다운로드
3. 설치 프로그램 실행 → 모든 옵션 기본값 그대로 → **Install** 클릭
4. 설치 완료 후 시작 메뉴에서 **"Git Bash"** 또는 **"PowerShell"** 을 열어 확인:
   ```
   git --version
   ```
   `git version 2.xx.x` 같은 결과가 나오면 성공

**2단계: Visual Studio Build Tools 설치 (C++ 컴파일러)**

Rust와 Tauri 앱을 빌드하려면 C++ 컴파일러가 필요합니다.

1. https://visualstudio.microsoft.com/ko/visual-cpp-build-tools/ 접속
2. **"Build Tools 다운로드"** 클릭
3. 설치 프로그램 실행
4. 워크로드 선택 화면에서 **"C++를 사용한 데스크톱 개발"** 체크
5. **설치** 클릭 (약 2~5GB 다운로드, 시간이 걸릴 수 있습니다)
6. 설치 완료 후 **PC 재시작**

**3단계: Rust 설치 (백엔드 빌드용)**

1. https://rustup.rs 접속
2. **"RUSTUP-INIT.EXE (64-BIT)"** 클릭하여 다운로드
3. 다운로드된 `rustup-init.exe` 실행
4. 검은 터미널 창이 뜨면 **1** 을 입력하고 Enter (기본 설치 선택)
5. 설치가 끝나면 터미널 창을 닫고, **새 PowerShell 창**을 열어서 확인:
   ```
   rustc --version
   ```
   `rustc 1.xx.x` 같은 결과가 나오면 성공

**4단계: Node.js 설치 (프론트엔드 빌드용)**

1. https://nodejs.org 접속
2. **LTS** (왼쪽 초록색 버튼) 클릭하여 다운로드
3. 설치 프로그램 실행 → 모든 옵션 기본값 그대로 → **Install** 클릭
4. 새 PowerShell 창을 열어서 확인:
   ```
   node --version
   npm --version
   ```
   각각 버전 번호가 나오면 성공

**5단계: 저장소 복제 (코드 다운로드)**

PowerShell을 열고 아래 명령어 실행:
```powershell
# 원하는 폴더로 이동 (예: 바탕화면)
cd ~/Desktop

# 코드 다운로드
git clone https://github.com/Kimjaechol/MoA_new.git

# 다운로드된 폴더로 이동
cd MoA_new
```

**6단계 (선택): 배포용 CLI 도구 설치**

앱 빌드만 할 거라면 이 단계는 건너뛰어도 됩니다.
서버 배포(Railway, Vercel)까지 하려면 설치하세요.

```powershell
npm install -g @railway/cli
npm install -g vercel
```

AWS CLI (R2 업로드용)는 https://aws.amazon.com/cli/ 에서 별도 다운로드합니다.

#### macOS에서 설치

```bash
# 1. Xcode 커맨드라인 도구 설치 (C++ 컴파일러 포함)
xcode-select --install

# 2. Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Node.js 설치 (Homebrew 사용)
brew install node

# 4. 배포 CLI 도구 (선택)
npm install -g @railway/cli
npm install -g vercel
```

#### Linux (Ubuntu/Debian)에서 설치

```bash
# 1. 시스템 패키지 설치 (C++ 컴파일러 + Tauri 의존성)
sudo apt update
sudo apt install -y build-essential curl wget file \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev

# 2. Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Node.js 18+ 설치
sudo apt install -y nodejs npm

# 4. 배포 CLI 도구 (선택)
npm install -g @railway/cli
npm install -g vercel
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

#### Windows에서 빌드하기

사전 준비: [2. 사전 준비물](#2-사전-준비물)의 Windows 설치 가이드를 모두 완료한 상태여야 합니다.
(Git, Visual Studio Build Tools, Rust, Node.js 설치 완료)

**PowerShell**을 열고 아래 명령어를 **한 줄씩** 실행하세요:

```powershell
# 1. 프로젝트 폴더로 이동 (코드를 다운로드한 위치)
cd ~/Desktop/MoA_new

# 2. Tauri 앱 폴더로 이동
cd clients/tauri

# 3. 프론트엔드 패키지 설치 (처음 한 번만)
npm install

# 4-A. 개발 모드로 실행 (테스트용, 코드 수정하면 바로 반영됨)
npm run tauri dev

# 4-B. 또는, 배포용 설치파일(.msi) 만들기
npm run tauri build
```

> **참고**: 첫 빌드에서는 Rust 패키지를 다운로드하고 컴파일하므로 시간이 오래 걸립니다.
> 두 번째 빌드부터는 훨씬 빠릅니다.

빌드가 완료되면 설치 파일이 아래 경로에 생성됩니다:
```
clients/tauri/src-tauri/target/release/bundle/msi/MoA_*.msi
```
이 `.msi` 파일을 더블클릭하면 MoA 앱이 내 PC에 설치됩니다.

#### macOS / Linux에서 빌드하기

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

### Q: Windows에서 빌드가 안돼요
- **"MSVC not found"** 오류 → Visual Studio Build Tools에서 **"C++를 사용한 데스크톱 개발"** 워크로드가 설치되어 있는지 확인하세요.
- **"rustc not found"** 오류 → PowerShell 창을 닫고 새로 열어보세요. 그래도 안 되면 `rustup-init.exe`를 다시 실행하세요.
- **"npm not found"** 오류 → Node.js 설치 후 PowerShell 창을 새로 열어야 합니다.
- 빌드 중 멈춘 것처럼 보일 때 → Rust 첫 빌드는 오래 걸립니다. 인내심을 갖고 기다려주세요.

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
