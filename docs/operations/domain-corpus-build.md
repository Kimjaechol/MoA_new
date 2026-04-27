# Domain corpus build — 6-category Korean legal corpus

`vault domain build`로 사용자 코퍼스를 swappable `domain.db`로 베이크하는 운영 가이드.

## 지원 디렉토리 카테고리 (v8.1)

| 디렉토리 | `SourceCategory` | 분류 | 처리 방식 |
|---|---|---|---|
| `현행법령/<YYYYMMDD>/...` | `Current` | 법령 (statute) | canonical + versioned 슬러그 모두 기록 |
| `연혁법령/<YYYYMMDD>/...` | `Historical` | 법령 (statute) | versioned 슬러그만 (canonical 보존) |
| `자치법규/...` | `LocalOrdinance` | 법령 (statute) | canonical 슬러그만 |
| `행정규칙/...` | `AdminRule` | 법령 (statute) | canonical 슬러그만 |
| `판례/<YYYYMMDD>/...` | `Case` | 판례 (case) | 사건번호 슬러그 1개 |
| `헌재결정례/<YYYYMMDD>/...` | `ConstitutionalCase` | 판례 (case) | 사건번호 슬러그 1개 |

루트 디렉토리(예: `C:\Users\kjccj\세컨드브레인_로컬DB`) 하부 어디에 위 6개 폴더 이름이 path에 등장하든 자동으로 인식됩니다. 인식되지 않은 경로는 `Unknown` 카테고리로 폴백되어 — 통과는 되지만 versioning 없이 canonical만 기록됩니다.

## 자동 추출되는 핵심 키워드

판례 ingest 시 다음 출처에서 인용 법조문을 추출해 **`cites` 엣지 + 태그**로 변환합니다 (우선순위 순):

1. **`## 적용법조` / `## 적용법령` / `## 적용법률`** — 법원이 실제 적용한 법조문 (사용자 명시: 최고 우선)
2. **`## 참조조문`** — 보조 인용
3. **`## 판례내용` / `## 결정문`** — 본문 내 인라인 인용

판례 ingest는 다음을 자동 alias/tag로 등록:
- **사건번호** — `2024노3424` (alias)
- **사건명 (제목)** — `근로기준법위반` (alias + `title:근로기준법위반` 태그)
- **법원명+사건번호** — `수원지법 2024노3424` (alias)
- **사건번호+사건명** — `2024노3424 근로기준법위반` (alias)

법령 ingest는 다음을 자동 alias/tag로 등록:
- **조문 슬러그** — `statute::근로기준법::36`
- **사람용 별칭** — `근로기준법 제36조`, `근기법 제36조`, ...
- **조문 소제목** — `금품 청산` (alias) + `근로기준법 금품 청산` (alias) + `금품 청산` (`title_kw` 태그)
  - 사용자 명시: 조항의 소제목에 해당하는 명사가 핵심 키워드 → bare alias로 노출되어 `[[금품 청산]]` 같은 위키링크로 곧장 resolve됨

검색 흐름 (`graph_query::find_nodes`):
1. 정확한 슬러그 매칭
2. 정확한 alias 매칭 ← **여기서 사건명/조문 소제목 hit**
3. 인용 정규식 파싱 (`근로기준법 제36조` → 슬러그 빌드)
4. 사건번호 파싱
5. FTS5 폴백 (3-gram trigram, main schema only)

## 빌드 명령

### Windows (PowerShell, 실제 사용자 환경)

```powershell
# 1) 빌드된 zeroclaw 바이너리 위치까지 이동
cd C:\Users\kjccj\dev\moa\target\release  # 또는 본인 빌드 경로

# 2) domain.db 베이크 — 기존 build 위치를 새 버전 폴더로 분리
.\zeroclaw.exe vault domain build `
  C:\Users\kjccj\세컨드브레인_로컬DB `
  --out C:\Users\kjccj\세컨드브레인_로컬DB\.build\korean-legal-2026.04.db
```

### Linux/macOS (대체 환경)

```bash
zeroclaw vault domain build \
  ~/세컨드브레인_로컬DB \
  --out ~/세컨드브레인_로컬DB/.build/korean-legal-2026.04.db
```

빌드 진행 중 표시되는 정보:
```
vault domain build: <corpus> → <out>
  progress: statutes=100 cases=0 (errors=0)
  progress: statutes=200 cases=0 (errors=0)
  ...
  ✓ statute files:    1500
  ✓ statute articles: 25000
  ✓ case files:       8000
  · skipped:          12
  · errors:           3
  ✓ edges resolved:   45000

  · corpus mix by source-path category:
    -              current : 800
    -           historical : 700
    -      local_ordinance : 400
    -           admin_rule : 200
    -                 case : 7500
    - constitutional_case  : 500
    -              unknown : 12
```

`--out` 경로가 이미 존재하면 build는 거부됩니다 (immutable bundle 원칙). 새 버전이면 파일명에 날짜/버전을 포함해 새로 만듭니다.

## 빌드 후 단계

### 로컬에서 직접 사용 (배포 없이)

```powershell
# 사용자 워크스페이스의 memory/domain.db 위치로 swap
.\zeroclaw.exe vault domain swap --from `
  C:\Users\kjccj\세컨드브레인_로컬DB\.build\korean-legal-2026.04.manifest.json
```

⚠️ Swap은 manifest를 통해 가야 sha256 검증이 됩니다. 로컬 직접 swap을 원하면 먼저 `publish`로 manifest를 생성하세요:

```powershell
.\zeroclaw.exe vault domain publish `
  C:\Users\kjccj\세컨드브레인_로컬DB\.build\korean-legal-2026.04.db `
  --url file:///C:/Users/kjccj/세컨드브레인_로컬DB/.build/korean-legal-2026.04.db `
  --name korean-legal `
  --version 2026.04
```

### R2/S3로 배포

```powershell
# 1) manifest 생성 (R2 URL을 미리 결정)
.\zeroclaw.exe vault domain publish `
  C:\Users\kjccj\세컨드브레인_로컬DB\.build\korean-legal-2026.04.db `
  --url https://r2.example.com/moa/domain/korean-legal-2026.04.db `
  --name korean-legal `
  --version 2026.04

# 2) bundle + manifest를 R2에 업로드
aws s3 cp .\korean-legal-2026.04.db s3://moa/domain/korean-legal-2026.04.db
aws s3 cp .\korean-legal-2026.04.manifest.json s3://moa/domain/korean-legal-2026.04.manifest.json

# 3) 클라이언트는 manifest URL로 install
.\zeroclaw.exe vault domain install --from `
  https://r2.example.com/moa/domain/korean-legal-2026.04.manifest.json
```

## 검증 체크리스트

빌드 후 통합 검색이 의도대로 동작하는지 확인:

```powershell
.\zeroclaw.exe vault domain info
# installed: true
# vault_documents: 33500
# vault_links:     45000

# 조문 소제목으로 검색 — 자동으로 [[금품 청산]] alias로 resolve되어야
.\zeroclaw.exe vault legal stats   # graph 노드/엣지 카운트 확인
```

## 알려진 제약

- **자치법규 / 행정규칙 시점 메타데이터**: 폴더명에 `YYYYMMDD`가 없으면 `publish_date` 가 None이 됩니다. 이 경우 versioned 슬러그가 만들어지지 않고 canonical 슬러그만 기록됩니다 — 검색은 정상이지만 "이 시점의 자치법규" 같은 시간축 질의는 불가합니다.
- **FTS5 cross-schema 한계**: SQLite FTS5의 `MATCH` 연산자는 schema-qualified table을 거부합니다. domain.db에 ATTACH된 상태에서 사용자 워크스페이스의 `find_nodes` 5단계 FTS 폴백은 `main` 스키마(brain.db)만 조회합니다. 도메인 corpus에 대한 본문 substring 검색은 별도 connection 기반 도구로 분리될 예정 (현재는 alias/citation 경로로 도달).
- **사건명 alias 충돌**: 같은 제목의 판례가 여러 건이면 (예: "근로기준법위반"), 첫 INSERT만 alias 등록되고 나머지는 INSERT OR IGNORE로 스킵됩니다. 판례는 `사건번호`로 항상 고유하게 도달 가능하므로 검색 자체는 영향 없음.

## 코드 위치

- 카테고리 인식: `src/vault/legal/source_path.rs::detect_category`
- 적용법조 추출: `src/vault/legal/case_extractor.rs::extract_case` (3-tier 우선순위)
- 조문 소제목 alias: `src/vault/legal/ingest.rs::upsert_statute_article`
- 사건명 alias/태그: `src/vault/legal/ingest.rs::ingest_case_to`
- 빌드 진행/리포트: `src/vault/domain_cli.rs::build`
