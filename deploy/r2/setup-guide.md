# Cloudflare R2 설정 가이드 — ZeroClaw 릴리스 바이너리 호스팅

이 가이드는 ZeroClaw 릴리스 바이너리를 Cloudflare R2에 업로드하고 배포하기 위한
단계별 설정 방법을 설명합니다.

---

## 목차

1. [R2 버킷 생성](#1-r2-버킷-생성)
2. [API 토큰 발급](#2-api-토큰-발급)
3. [커스텀 도메인 설정](#3-커스텀-도메인-설정)
4. [CORS 설정](#4-cors-설정)
5. [비용 예측](#5-비용-예측)
6. [업로드 스크립트 사용법](#6-업로드-스크립트-사용법)

---

## 1. R2 버킷 생성

### 1.1 Cloudflare 대시보드 접속

1. [Cloudflare 대시보드](https://dash.cloudflare.com/)에 로그인합니다.
2. 좌측 메뉴에서 **R2 Object Storage**를 클릭합니다.

### 1.2 버킷 만들기

1. **Create bucket** 버튼을 클릭합니다.
2. 버킷 이름을 입력합니다 (예: `zeroclaw-releases`).
   - 이름은 소문자, 숫자, 하이픈만 사용 가능합니다.
   - 전역적으로 고유할 필요는 없습니다 (계정 내에서만 고유하면 됩니다).
3. **Location**을 선택합니다:
   - **Automatic (추천)**: Cloudflare가 최적의 위치를 자동 선택합니다.
   - **Specific region**: 특정 지역이 필요한 경우 선택합니다 (예: `apac` — 아시아 태평양).
4. **Create bucket**을 클릭하여 완료합니다.

---

## 2. API 토큰 발급

R2에 파일을 업로드하려면 S3 호환 API 토큰이 필요합니다.

### 2.1 토큰 생성

1. Cloudflare 대시보드에서 **R2 Object Storage** > **Manage R2 API Tokens**으로 이동합니다.
2. **Create API token**을 클릭합니다.
3. 토큰 설정:
   - **Token name**: `zeroclaw-release-upload` (식별용)
   - **Permissions**: **Object Read & Write**
   - **Specify bucket(s)**: 위에서 만든 버킷만 선택합니다 (최소 권한 원칙).
   - **TTL**: 필요에 따라 설정합니다 (보안을 위해 유효기간을 설정하는 것을 권장합니다).
4. **Create API Token**을 클릭합니다.

### 2.2 자격 증명 저장

토큰 생성 후 다음 값이 표시됩니다. **이 값은 한 번만 표시되므로** 안전하게 저장하세요:

| 항목 | 설명 | 환경 변수 |
|------|------|-----------|
| **Access Key ID** | S3 호환 액세스 키 | `AWS_ACCESS_KEY_ID` |
| **Secret Access Key** | S3 호환 시크릿 키 | `AWS_SECRET_ACCESS_KEY` |

### 2.3 엔드포인트 URL 확인

R2 S3 호환 엔드포인트는 다음 형식입니다:

```
https://<ACCOUNT_ID>.r2.cloudflarestorage.com
```

**Account ID** 확인 방법:

1. Cloudflare 대시보드 우측 사이드바에서 **Account ID**를 확인합니다.
2. 또는 R2 버킷 설정 페이지에서 S3 API 엔드포인트를 직접 복사합니다.

### 2.4 GitHub Secrets 설정 (CI/CD용)

GitHub Actions에서 사용하려면 리포지토리 Settings > Secrets and variables > Actions에
다음 시크릿을 추가합니다:

| Secret 이름 | 값 |
|---|---|
| `R2_ACCESS_KEY_ID` | 위에서 발급한 Access Key ID |
| `R2_SECRET_ACCESS_KEY` | 위에서 발급한 Secret Access Key |
| `R2_ENDPOINT` | `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` |
| `R2_BUCKET` | 버킷 이름 (예: `zeroclaw-releases`) |

---

## 3. 커스텀 도메인 설정

R2 버킷에 커스텀 도메인을 연결하면 사용자가 직관적인 URL로 다운로드할 수 있습니다.

### 3.1 공개 액세스 활성화

1. R2 버킷 설정 페이지로 이동합니다.
2. **Settings** 탭을 클릭합니다.
3. **Public access** 섹션에서 **Allow Access**를 활성화합니다.

### 3.2 커스텀 도메인 연결

1. **Public access** > **Custom Domains**에서 **Connect Domain**을 클릭합니다.
2. 사용할 도메인을 입력합니다 (예: `releases.zeroclaw.dev`).
   - 해당 도메인이 Cloudflare에서 관리되고 있어야 합니다.
3. **Connect Domain**을 클릭합니다.
4. Cloudflare가 자동으로 DNS CNAME 레코드를 생성합니다.

### 3.3 확인

설정 완료 후 다음 URL로 접속하여 확인합니다:

```
https://releases.zeroclaw.dev/releases/latest/
```

---

## 4. CORS 설정

브라우저에서 직접 다운로드를 허용하려면 CORS (Cross-Origin Resource Sharing) 설정이 필요합니다.

### 4.1 CORS 규칙 추가

1. R2 버킷 **Settings** 탭으로 이동합니다.
2. **CORS Policy** 섹션에서 **Add CORS policy**를 클릭합니다.
3. 다음 JSON 정책을 입력합니다:

```json
[
  {
    "AllowedOrigins": ["*"],
    "AllowedMethods": ["GET", "HEAD"],
    "AllowedHeaders": ["*"],
    "ExposeHeaders": ["Content-Length", "Content-Type", "ETag"],
    "MaxAgeSeconds": 86400
  }
]
```

**설명:**

| 필드 | 값 | 설명 |
|------|-----|------|
| `AllowedOrigins` | `["*"]` | 모든 도메인에서 다운로드 허용. 특정 도메인만 허용하려면 `["https://zeroclaw.dev"]`로 변경합니다. |
| `AllowedMethods` | `["GET", "HEAD"]` | 읽기 전용 접근만 허용 (업로드는 API 토큰으로만 가능). |
| `AllowedHeaders` | `["*"]` | 모든 요청 헤더 허용. |
| `ExposeHeaders` | `[...]` | 클라이언트에 노출할 응답 헤더. 다운로드 진행률 표시에 필요합니다. |
| `MaxAgeSeconds` | `86400` | CORS preflight 캐시 시간 (24시간). |

4. **Save**를 클릭합니다.

### 4.2 보안 참고사항

- 업로드(PUT/POST)는 CORS에서 허용하지 않습니다. 업로드는 API 토큰 인증을 통해서만 가능합니다.
- 특정 도메인만 다운로드를 허용하려면 `AllowedOrigins`를 제한하세요.

---

## 5. 비용 예측

Cloudflare R2는 매우 합리적인 가격 정책을 제공합니다.
**특히 이그레스(데이터 전송) 비용이 무료입니다.**

### 5.1 무료 포함량 (매월)

| 항목 | 무료 포함량 |
|------|------------|
| **스토리지** | 10 GB |
| **Class A 요청** (PUT, POST, LIST 등) | 1,000,000건 |
| **Class B 요청** (GET, HEAD 등) | 10,000,000건 |
| **이그레스 (데이터 전송)** | **무제한 무료** |

### 5.2 초과 시 요금

| 항목 | 요금 |
|------|------|
| 스토리지 | $0.015 / GB / 월 |
| Class A 요청 | $4.50 / 1,000,000건 |
| Class B 요청 | $0.36 / 1,000,000건 |

### 5.3 ZeroClaw 예상 비용

ZeroClaw 릴리스 바이너리 기준 예상:

| 항목 | 예상값 | 비용 |
|------|--------|------|
| 바이너리 크기 (3 플랫폼) | ~15 MB x 3 = ~45 MB/릴리스 | - |
| 월간 릴리스 수 | ~4회 | - |
| 월간 총 스토리지 | ~180 MB (최근 버전 유지) | **무료** (10 GB 이내) |
| 월간 다운로드 | ~1,000건 | **무료** (10M건 이내) |
| 이그레스 | ~45 GB | **무료** (항상 무료) |
| **월간 총 예상 비용** | - | **$0.00** |

대부분의 프로젝트에서 R2 무료 포함량으로 충분합니다.

---

## 6. 업로드 스크립트 사용법

### 6.1 사전 요구사항

- [AWS CLI v2](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) 설치
- 위에서 발급한 API 토큰 (환경 변수 설정)

### 6.2 수동 업로드

```bash
# 환경 변수 설정
export AWS_ACCESS_KEY_ID="your_access_key_id"
export AWS_SECRET_ACCESS_KEY="your_secret_access_key"
export R2_ENDPOINT="https://<account-id>.r2.cloudflarestorage.com"
export R2_BUCKET="zeroclaw-releases"

# 릴리스 빌드
cargo build --release --locked

# 릴리스 디렉토리에 아티팩트 복사
mkdir -p release
cp target/release/zeroclaw release/

# 버전 지정 업로드
VERSION=v0.1.0 ./deploy/r2/upload.sh
```

### 6.3 CI/CD 자동 업로드

GitHub Actions 워크플로우(`release.yml`)에서 자동으로 업로드됩니다.
리포지토리 Secrets에 R2 자격 증명을 설정하면 태그 푸시 시 자동 실행됩니다.

### 6.4 업로드 확인

```bash
# 버킷 내용 확인
aws s3 ls "s3://$R2_BUCKET/releases/" \
    --endpoint-url "$R2_ENDPOINT" \
    --recursive

# 특정 파일 다운로드 테스트
curl -O "https://releases.zeroclaw.dev/releases/latest/zeroclaw-x86_64-unknown-linux-gnu.tar.gz"
```

---

## 참고 링크

- [Cloudflare R2 공식 문서](https://developers.cloudflare.com/r2/)
- [R2 S3 호환 API 문서](https://developers.cloudflare.com/r2/api/s3/)
- [R2 가격 정책](https://developers.cloudflare.com/r2/pricing/)
- [AWS CLI 설치 가이드](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)
