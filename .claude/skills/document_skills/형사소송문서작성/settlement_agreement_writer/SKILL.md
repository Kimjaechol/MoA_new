---
name: settlement_agreement_writer
description: "한국 형사소송법에 따른 형사 합의서 자동 작성 스킬. 형사 사건의 법적으로 유효한 합의서 작성. 피해자-가해자 합의, 보상 조건 및 처벌 불원 표시 포함."
license: Proprietary. For LawPro AI Platform use only.
version: 2.0.0
---

# Settlement Agreement Writer Skill (형사 합의서 작성)

## Overview

Generates professional Korean criminal settlement agreements (형사 합의서) that document victim-perpetrator settlements, including compensation, non-prosecution agreements, and leniency requests.

**Key Features:**
- **Complete settlement structure**: Parties + Incident + Compensation + Agreement terms
- **Multiple case types**: Assault, theft, traffic accidents, fraud, etc.
- **Victim protection**: True consent verification, no coercion clauses
- **Legal validity**: Meets prosecution and court standards
- **Leniency clauses**: Optional prosecutor/judge leniency requests

## Document Purpose

형사 합의서 (Criminal Settlement Agreement) is a document where victim and perpetrator mutually agree on:

1. **Compensation** (손해배상): Financial compensation for damages
2. **Non-prosecution** (형사처벌 불원): Victim's statement of no desire for punishment
3. **Civil release** (민사청구권 포기): Waiver of all civil claims
4. **Settlement finality** (최종 합의): Irrevocable settlement

**Use Cases**:
- **사건 종결**: Close criminal case through settlement
- **처벌 감경**: Reduce perpetrator's punishment
- **피해 배상**: Compensate victim for damages
- **관계 회복**: Restore relationship between parties

## 합의서의 법적 효과

### 1. 형사사건에서의 효과

**반의사불벌죄** (폭행, 협박, 명예훼손, 업무방해 등):
- 피해자가 처벌을 원하지 않으면 **공소제기 불가** (형사소송법 제232조)
- 합의서 제출 시 **사건 종결**

**일반 범죄** (사기, 절도, 횡령 등):
- 공소제기 가능하나 **양형 참작** (형법 제51조)
- 실형 → 집행유예, 벌금형 감경 가능

**친고죄** (모욕, 사자 명예훼손, 비밀침해 등):
- 고소 취소 시 **공소기각** (형사소송법 제232조)
- 합의서와 함께 고소취소서 제출

### 2. 민사사건에서의 효과

- **손해배상청구권 소멸**: 동일 사건으로 민사소송 불가
- **기판력**: 합의 내용에 대한 재소송 제한
- **강제집행 불가**: 합의서만으로는 강제집행 불가 (공정증서 필요)

### 3. 교통사고의 경우

**형사합의 vs 민사합의 구분 필요**:
- **형사합의**: 가해자 개인이 지급, 형사처벌 감경 목적
- **민사합의**: 보험회사가 지급, 손해배상 목적

**보험금 공제 방지 조항** (채권양도 조항):
- 보험회사가 합의금을 공제할 경우 가해자의 부당이득반환청구권을 피해자에게 양도
- 보험회사에 채권양도 통지

## Document Structure

### 1. Header (표제부)
```
                     합 의 서

피해자    김철수(831130-1247712)
          서울특별시 강남구 테헤란로 123
          전화 010-1234-5678

가해자    이영희(800217-1348311)
          서울특별시 서초구 서초대로 456
          전화 010-9876-5432
```

### 2. 사건 개요 (Incident Summary)
```
20○○년 ○월 ○일 22시경 서울시 강남구 ○○동 소재 ○○빌딩 앞 도로에서
발생한 폭행사건(이하 "이 사건"이라 합니다)과 관련하여 위 당사자는 다음과 같이
합의합니다.
```

### 3. 합의 내용 (Agreement Terms)

#### 가. 합의금액
```
1. 합의금액: 금 5,000,000원(오백만원)
2. 지급일자: 20○○년 ○월 ○일
3. 지급방법: 은행계좌 입금 (○○은행 123-456-789012, 예금주: 김철수)
```

#### 나. 형사처벌 불원 조항
```
피해자는 위 사건으로 인한 일체 민사상 손해배상을 가해자에게 청구하지 않으며,
또한 가해자가 형사상 처벌받는 것을 원하지 않습니다.

(검사님 그리고 판사님! 가해자는 피해자에게 미안해하고 있고, 깊이 뉘우치고 있으므로
최대한의 선처를 하여 주시기 바랍니다.)
```

#### 다. 최종 조항
```
본 합의는 피해자와 가해자의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자 등으로부터
사기와 강박 등이 전혀 없이 평온·공연하게 합의한 것이므로 향후 어떠한 사유로도
피해자는 가해자에게, 가해자는 피해자에게 민·형사상 책임을 묻지 아니하며
모든 청구권을 포기합니다.
```

### 4. 서명날인
```
첨부서류: 인감증명서 각 1통

20○○년 ○월 ○일

위 합의인(피해자)  김철수  (인)
서울특별시 강남구 테헤란로 123

위 합의인(가해자)  이영희  (인)
서울특별시 서초구 서초대로 456
```

## 사건 유형별 합의서

### 1. 폭행 합의서

**주요 내용**:
- 사건 발생 일시·장소
- 상해 진단명 및 치료기간 (전치 ○주)
- 합의금액 (치료비 + 위자료 + 일실수입)
- 형사처벌 불원 및 선처 요청

**예시**:
```
피해자 최○○을 "갑"이라 칭하고, 가해자 조○○을 "을"이라 칭하며
다음과 같은 조건으로 민·형사상 합의를 한다.

1. (갑)과 (을)이 200○. 4. 22. 02:00경 서울 도봉구 방학동 690-79 앞 노상에서
   서로 다툼으로 인해 "갑"은 상악좌측, 우측 중측절치의 치관균열, 출혈성 뇌좌상,
   두개골골절, 두피좌상, 안면부좌상, 요추 염좌 등 전치 6주의 상해를 입은 사실이
   있는바, (을)은 (갑)에게 민·형사상 합의금조로 금 5,000,000원을 200○. 7. 7. 자
   지급하기로 한다.

2. (갑)은 위 사건으로 인한 일체 민사상 손해배상을 "을"에게 청구하지 않으며,
   또한 "을"이 형사상처벌 받는 것을 원하지 않는다.
```

### 2. 절도 합의서

**주요 내용**:
- 절도 물품 및 가액
- 손해배상금액
- 형사처벌 불원
- 대리인 서명 (미성년자인 경우)

**예시**:
```
1. 장○○(821020-1234567)은 200○. 5. 20. 00:20경 서울 서초구 ○○로 100길 노상 앞에
   세워 둔 피해자 박○○의 소유 75아1234호 소나타 내부에 장착된 카 오디오를 훔쳐
   이웃 주민의 신고로 서울서초경찰서에 현재 구속된바, 그 구속 사건에 대하여,

2. 장○○의 아버지 장한국은 손해배상금 및 위자료조로 200○. 5. 21. 금 1,000,000원을
   피해자의 은행통장계좌(○○은행, 12-458-678900)로 입금한다.

3. 피해자는 위 사건에 대하여 가해자의 형사상 처벌을 원하지 않으며,
   일체 어떠한 민사상 책임도 묻지 않는다.
```

### 3. 교통사고 형사합의서

**주요 특징**:
- 보험회사 민사합의금과 **별개**로 가해자 개인이 지급
- 형사처벌 감경 목적
- 채권양도 조항 포함 (보험금 공제 방지)

**예시**:
```
형 사 합 의 서

1. 사고 내용
   가. 사고일시: 200○. 5. 25.
   나. 사고장소: 서울 서초구 ○○로 1000길 교대사거리 노상 앞
   다. 사고유형: 교통사고처리특례법 12개 항목 중 신호위반 사고
   라. 가해자: 정○○
   마. 가해차량: 37라1234
   바. 피해자: 홍○○

2. 합의내용
   가. 합의금액: 금오백만 원(5,000,000원)
   나. 합의사항
       (1) 이 합의금은 가해자가 형사처벌을 감경할 목적으로 가해자 개인이 지급하는 금액이다.
       (2) 피해자는 위 금원을 지급받고 가해자의 처벌을 원치 않는다.

   다. 채권양도
       (1) 만일 보험회사의 보상금에서 위 합의금의 일부라도 공제될 경우 그에 대하여
           가해자가 보험회사를 상대로 갖게 될 부당이득반환청구권(또는 보험금 청구권)을
           피해자 홍○○에게 양도한다.
       (2) 이와 같은 채권양도의 효력을 확실히 하기 위해 가해자는 즉시 가해차량의
           보험회사인 ○○화재해상보험(주)에 채권양도통지를 한다.
       (3) 이 합의로써 채권양도 되었기에 나중에 가해자가 보험회사에 대한
           부당이득반환청구권을 포기하더라도 그 효력은 인정되지 못한다.
       (4) 만일 가해자가 보험회사에 대한 부당이득반환청구권을 포기할 경우에는
           보상금액에서 공제된 합의금 액수만큼 피해자에게 다시 지급한다.
```

### 4. 보험회사 대인배상보험금 합의서

**주요 특징**:
- 보험회사가 작성한 표준양식
- **민사 합의**만 해당 (형사합의 아님)
- 모든 권리 포기 확약

**주의사항**:
- 형사처벌 불원 조항 없음
- 보험금 지급 항목 명시 (부상보험금, 후유장애보험금, 사망보험금 등)
- 서명 후 보험금 지급 종결 시 추가 안내

## 합의서 작성 시 주의사항

### 1. 진정한 의사표시 확인

**필수 기재사항**:
```
본 합의는 피해자와 가해자의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자 등으로부터
사기와 강박 등이 전혀 없이 평온·공연하게 합의한 것입니다.
```

**목적**:
- 사기·강박에 의한 합의 주장 방지
- 나중에 합의 무효 주장 차단

### 2. 선처 요청 문구 (선택사항)

**탄원 효과 있음**:
```
(검사님, 판사님 가해자께서는 피해자에 대하여 미안해하고, 안타까워하고,
마음아파하고 있으니 넓으신 마음으로 혜량하여 주시기 바랍니다.)
```

**주의**:
- 선처 요청 문구를 넣으면 탄원의 효과도 포함
- 피해자가 원하지 않으면 생략 가능

### 3. 인감증명서 첨부

**첨부 이유**:
- 본인 확인
- 진정한 의사표시 증명
- 나중에 합의 부인 방지

**첨부서류**:
```
첨부서류: 인감증명서 각 1통
```

### 4. 합의서 제출 방법

**형사사건**:
- 검찰청 또는 법원에 제출
- 3부 작성 (법원/검찰 제출용 1부, 피해자 보관용 1부, 가해자 보관용 1부)

**고소취소가 필요한 경우**:
- 친고죄: 합의서 + 고소취소서 함께 제출
- 반의사불벌죄: 합의서만 제출 (처벌 불원 의사 표시로 충분)

### 5. 교통사고의 경우 주의사항

**형사합의 vs 민사합의 혼동 주의**:
- **형사합의**: 가해자 개인 지급 → 형사처벌 감경
- **민사합의**: 보험회사 지급 → 손해배상

**형사합의 시 채권양도 조항 필수**:
- 보험회사가 형사합의금을 민사합의금에서 공제하는 것 방지
- 공제 시 가해자의 부당이득반환청구권을 피해자에게 양도

## 사용 예시

### 예시 1: 폭행사건 합의서

```python
from settlement_agreement_writer import SettlementAgreementWriter

writer = SettlementAgreementWriter()

# 폭행사건 합의서 작성
settlement = writer.write_assault_settlement(
    victim={
        "name": "김철수",
        "resident_number": "831130-1247712",
        "address": "서울특별시 강남구 테헤란로 123",
        "phone": "010-1234-5678"
    },
    perpetrator={
        "name": "이영희",
        "resident_number": "800217-1348311",
        "address": "서울특별시 서초구 서초대로 456",
        "phone": "010-9876-5432"
    },
    incident={
        "date": "2024년 10월 15일",
        "time": "22시경",
        "location": "서울시 강남구 역삼동 소재 ○○빌딩 앞 도로",
        "injury": "전치 2주의 안면부 타박상",
        "description": "양 당사자 간 말다툼으로 인한 폭행"
    },
    compensation={
        "amount": 3000000,
        "payment_date": "2024년 11월 1일",
        "payment_method": "은행계좌 입금",
        "bank_info": {
            "bank": "국민은행",
            "account": "123-456-789012",
            "holder": "김철수"
        }
    },
    leniency_request=True
)

print(settlement)
```

### 예시 2: 교통사고 형사합의서

```python
# 교통사고 형사합의서 작성
traffic_settlement = writer.write_traffic_accident_settlement(
    victim={
        "name": "홍길동",
        "resident_number": "900515-1234567",
        "address": "서울시 송파구 올림픽로 123",
        "phone": "010-1111-2222"
    },
    perpetrator={
        "name": "정민수",
        "resident_number": "850820-1234567",
        "address": "서울시 강남구 테헤란로 456",
        "phone": "010-3333-4444"
    },
    accident={
        "date": "2024년 9월 20일",
        "time": "14:30경",
        "location": "서울 서초구 반포대로 교대사거리",
        "type": "신호위반",
        "vehicle": "37라1234 (소나타)",
        "injury": "전치 4주의 경추 염좌"
    },
    compensation={
        "amount": 5000000,
        "payment_date": "2024년 10월 1일",
        "payment_method": "은행계좌 입금",
        "bank_info": {
            "bank": "우리은행",
            "account": "987-654-321098",
            "holder": "홍길동"
        }
    },
    insurance_company="삼성화재해상보험주식회사",
    credit_assignment=True,  # 채권양도 조항 포함
    leniency_request=True
)

print(traffic_settlement)
```

### 예시 3: 절도사건 합의서 (미성년자 가해자)

```python
# 절도사건 합의서 작성 (가해자 대리인)
theft_settlement = writer.write_theft_settlement(
    victim={
        "name": "박영희",
        "resident_number": "750312-2234567",
        "address": "서울시 마포구 월드컵로 789",
        "phone": "010-5555-6666"
    },
    perpetrator={
        "name": "장철민",
        "resident_number": "080920-3234567",
        "address": "서울시 은평구 통일로 321",
        "phone": "010-7777-8888",
        "legal_representative": {
            "name": "장한국",
            "resident_number": "470724-1234567",
            "relationship": "부"
        }
    },
    incident={
        "date": "2024년 10월 5일",
        "time": "02:00경",
        "location": "서울 서초구 강남대로 100길 노상",
        "stolen_item": "카 오디오",
        "value": 500000,
        "description": "피해자 소유 차량(75아1234) 내부 카 오디오 절취"
    },
    compensation={
        "amount": 1000000,
        "payment_date": "2024년 10월 10일",
        "payment_method": "은행계좌 입금",
        "bank_info": {
            "bank": "신한은행",
            "account": "111-222-333444",
            "holder": "박영희"
        }
    },
    leniency_request=True
)

print(theft_settlement)
```

## Python API Reference

### Class: `SettlementAgreementWriter`

#### Methods:

##### 1. `write_assault_settlement()`
폭행사건 합의서 작성

**Parameters**:
- `victim` (dict): 피해자 정보
- `perpetrator` (dict): 가해자 정보
- `incident` (dict): 사건 정보
- `compensation` (dict): 합의금 정보
- `leniency_request` (bool): 선처 요청 여부

##### 2. `write_traffic_accident_settlement()`
교통사고 형사합의서 작성

**Parameters**:
- `victim` (dict): 피해자 정보
- `perpetrator` (dict): 가해자 정보
- `accident` (dict): 사고 정보
- `compensation` (dict): 합의금 정보
- `insurance_company` (str): 보험회사명
- `credit_assignment` (bool): 채권양도 조항 포함 여부
- `leniency_request` (bool): 선처 요청 여부

##### 3. `write_theft_settlement()`
절도사건 합의서 작성

**Parameters**:
- `victim` (dict): 피해자 정보
- `perpetrator` (dict): 가해자 정보 (legal_representative 포함 가능)
- `incident` (dict): 사건 정보
- `compensation` (dict): 합의금 정보
- `leniency_request` (bool): 선처 요청 여부

##### 4. `write_general_settlement()`
일반 형사사건 합의서 작성

**Parameters**:
- `victim` (dict): 피해자 정보
- `perpetrator` (dict): 가해자 정보
- `incident` (dict): 사건 정보
- `compensation` (dict): 합의금 정보
- `leniency_request` (bool): 선처 요청 여부

## 법률 참고사항

### 형사소송법

**제232조 (피해자의 의사와 처벌)**
반의사불벌죄와 친고죄는 피해자의 명시한 의사에 반하여 공소를 제기할 수 없다.

### 형법

**제51조 (양형의 조건)**
형을 정함에 있어서는 다음 사항을 참작하여야 한다:
1. 범인의 연령, 성행, 지능과 환경
2. 피해자에 대한 관계
3. **범행의 동기, 수단과 결과**
4. **범행후의 정황** ← 합의 포함

**제328조 (친족간의 범행과 고소)**
직계혈족, 배우자, 동거친족, 동거가족 또는 그 배우자간의 절도, 사기, 횡령, 배임죄는 그 형을 면제한다.

### 교통사고처리특례법

**제3조 (처벌의 특례)**
제1항 단서 12개 항목 위반 시 피해자의 명시적 의사에 반하여 공소제기 불가

**12개 항목**:
1. 신호위반
2. 중앙선 침범
3. 제한속도 20km/h 초과
4. 앞지르기 방법 위반
5. 철길건널목 통과방법 위반
6. 횡단보도 보행자 보호의무 위반
7. 무면허운전
8. 음주운전
9. 보도 침범
10. 승객 추락방지의무 위반
11. 어린이 보호구역 안전운전 의무 위반
12. 화물 고정조치 위반

## License

Proprietary. For LawPro AI Platform use only.

## 버전 이력

- **2.0.0** (2025-12-02): 한국어 법률용어 완전 적용 및 사법연수원 교재 기준 준수
  - 한국어 설명 완전 적용
  - 형사소송법 법률용어 표준화

- **1.0.0** (2025-11-11): Initial release
  - 폭행사건 합의서 작성
  - 교통사고 형사합의서 작성 (채권양도 조항 포함)
  - 절도사건 합의서 작성 (미성년자 대리인 지원)
  - 일반 형사사건 합의서 작성
