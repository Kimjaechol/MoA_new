---
name: administrative_litigation_writer
description: "행정소송 소장 자동 작성 스킬. 세무소송(취득세·양도소득세·상속세·법인세·증여세 부과처분취소), 각종 행정소송(운전면허취소·징계처분취소·건축허가거부·토지수용재결취소·개발부담금부과처분취소 등) 소장을 자동으로 작성합니다."
version: 1.0.0
reference_data: "행정소송 소장 (백영사, 19,363줄)"
---

# 행정소송문서작성 스킬

## 개요

행정소송은 행정청의 위법한 처분이나 부작위로 인하여 권리 또는 이익을 침해받은 국민이 그 처분의 취소 또는 변경이나 무효확인 등을 구하는 소송입니다. 본 스킬은 세무소송과 각종 행정소송의 소장 작성을 지원합니다.

## 참조 데이터베이스

- **출처**: 행정소송 소장 (백영사)
- **분량**: 19,363줄
- **내용**: 세무소송 소장, 각종 행정소송 소장, 청구취지 사례

## 주요 기능

### 1. 세무소송 소장 작성

#### 1.1 취득세부과처분취소 청구의 소

**법적 근거**
- 지방세법 제112조 제2항 (법인의 비업무용 토지)
- 지방세법 시행령 제84조의 4 제1항

**작성 요소**
```python
def write_acquisition_tax_complaint(
    plaintiff: dict,          # 원고 정보 (법인명, 대표자, 주소, 소송대리인)
    defendant: dict,          # 피고 정보 (구청장, 주소)
    land_info: dict,          # 토지 정보 (소재지, 면적, 취득일)
    tax_amount: int,          # 부과된 취득세 금액
    acquisition_purpose: str, # 취득 목적 (업무용 건물 부지 등)
    use_plan: str,           # 사용 계획 및 경위
    legal_grounds: str,      # 정당한 사유 (법령에 의한 제한 등)
    preliminary_procedure: dict, # 전치절차 (심사청구, 심판청구)
    evidence: list,          # 입증방법
    court: str               # 관할 법원
) -> str:
    """
    취득세부과처분취소 청구의 소장 작성

    주요 내용:
    1. 청구취지: "피고가 원고에 대하여 한 취득세 부과처분을 취소한다"
    2. 청구원인:
       - 토지 취득 경위 및 목적
       - 업무용 건물 신축을 위한 정당한 사용 계획
       - 법령에 의한 제한으로 1년 이내 사용 불가 사유
       - 지방세법 제112조 제2항의 "정당한 사유" 해당성
    3. 전치절차: 심사청구 → 심판청구 → 소 제기
    4. 입증방법: 납세고지서, 결정서, 국세심판결정통지 등
    """
    pass
```

**작성 예시**
```
소    장

원      고  주식회사 ○○
            서울시 종로구 ○○동 1234
            대표이사  김 ○ ○
            소송대리인 변호사 이 ○ ○

피      고  서울시 종로구청장

취득세부과처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 20○○. 2. 10. 한 20○○년 2수시분 취득세
금 174,159,420원의 부과처분은 이를 취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 ○○업을 영위하는 법인으로서, 업무용 건물 신축을 위하여
   다음과 같은 토지를 취득하였습니다.

   가. 서울시 종로구 ○○동 1284의 2 대 2,449㎡ (20○○. 6. 10. 취득)
   나. 서울시 종로구 ○○동 1284의 3 대 330.9㎡ (20○○. 7. 22. 취득)

2. 원고가 위 토지를 취득한 날로부터 1년 이내에 업무용 건물 신축에
   직접 사용하지 못한 것은 다음과 같은 정당한 사유가 있었습니다.

   가. 건설부장관의 건축허가 제한조치 (20○○. 5. 15. ~ 20○○. 12. 31.)
   나. 서울시의 도시계획 도로확장으로 인한 토지 일부 수용

3. 건축허가제한조치가 해제된 후 원고는 즉시 업무용 건물 신축을
   추진하였습니다.

   가. 20○○. 3. 22. 유통연구소와 기획업무 계약 체결
   나. 20○○. 7. 12. 건축사사무소와 설계용역 계약 체결
   다. 20○○. 10. 29. 건축허가 신청
   라. 20○○. 12. 7. 건축허가 취득
   마. 20○○. 2. 21. 착공
```

#### 1.2 양도소득세부과처분취소 청구의 소

**법적 근거**
- 소득세법 제94조 (양도소득의 범위)
- 소득세법 시행령 제170조 (양도가액 및 취득가액의 산정)

**작성 요소**
```python
def write_capital_gains_tax_complaint(
    plaintiff: dict,          # 원고 정보
    defendant: str,           # 피고 (세무서장)
    land_info: dict,          # 토지 정보
    acquisition_price: int,   # 실제 취득가액
    transfer_price: int,      # 실제 양도가액
    taxed_amount: int,        # 부과된 양도소득세
    dispute_reason: str,      # 다툼 사유 (나대지 vs 농지 등)
    land_use_status: str,     # 토지 이용 상황
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """
    양도소득세부과처분취소 청구의 소장 작성

    주요 쟁점:
    1. 토지의 성질 (농지 vs 나대지)
    2. 장기보유특별공제 대상 여부
    3. 실거래가액 확인 여부
    4. 환산가액 적용의 적법성
    """
    pass
```

#### 1.3 상속세부과처분취소 청구의 소

**법적 근거**
- 상속세 및 증여세법 제8조의2 제2항 제2호
- 민법 제1008조의3 (분묘 등의 승계)

**작성 요소**
```python
def write_inheritance_tax_complaint(
    plaintiff: list,          # 원고들 (상속인)
    defendant: str,           # 피고 (세무서장)
    deceased: dict,           # 피상속인 정보
    property_info: dict,      # 재산 정보 (분묘, 묘토 등)
    tax_amount: int,          # 부과된 상속세
    exemption_grounds: str,   # 과세가액 불산입 사유
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """
    상속세부과처분취소 청구의 소장 작성

    주요 쟁점:
    1. 분묘에 속하는 묘토인 농지 해당 여부 (600평 이내)
    2. 상속세 과세가액 불산입 대상 여부
    3. 제사주재자의 승계 여부
    """
    pass
```

#### 1.4 법인세부과처분취소 청구의 소

**법적 근거**
- 법인세법
- 법인세법 시행령

**작성 요소**
```python
def write_corporate_tax_complaint(
    plaintiff: dict,          # 원고 (법인)
    defendant: str,           # 피고 (세무서장)
    tax_period: str,          # 과세 기간
    tax_amount: int,          # 부과된 법인세
    dispute_item: str,        # 다툼 항목 (손금불산입, 익금산입 등)
    legal_interpretation: str, # 법령 해석
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """법인세부과처분취소 청구의 소장 작성"""
    pass
```

#### 1.5 증여세부과처분취소 청구의 소

**법적 근거**
- 상속세 및 증여세법 제2조 (증여세 과세대상)
- 상속세 및 증여세법 제35조 (증여재산공제)

**작성 요소**
```python
def write_gift_tax_complaint(
    plaintiff: dict,          # 원고 (수증자)
    defendant: str,           # 피고 (세무서장)
    gift_property: dict,      # 증여재산 정보
    gift_date: str,           # 증여일
    tax_amount: int,          # 부과된 증여세
    dispute_reason: str,      # 다툼 사유
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """증여세부과처분취소 청구의 소장 작성"""
    pass
```

### 2. 각종 행정소송 소장 작성

#### 2.1 자동차운전면허취소처분취소 청구의 소

**법적 근거**
- 도로교통법 제93조 (운전면허의 취소·정지)
- 도로교통법 시행규칙 [별표 28] (운전면허 취소·정지 처분기준)

**작성 요소**
```python
def write_license_revocation_complaint(
    plaintiff: dict,          # 원고 (운전자)
    defendant: str,           # 피고 (경찰서장, 지방경찰청장)
    license_info: dict,       # 면허 정보
    revocation_date: str,     # 취소 처분일
    violation_facts: str,     # 위반 사실
    mitigation_grounds: str,  # 참작 사유 (생계, 무사고 경력 등)
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """
    자동차운전면허취소처분취소 청구의 소장 작성

    주요 내용:
    1. 청구취지: "피고가 원고에 대하여 한 운전면허 취소처분을 취소한다"
    2. 청구원인:
       - 운전면허 취득 경위 및 이력
       - 취소 사유로 된 위반 사실의 경위
       - 참작 사유 (생계 수단, 무사고 운전 경력, 가족 부양 등)
       - 비례의 원칙 위반 (취소보다 정지가 적절)
    3. 재량권 일탈·남용 주장
    """
    pass
```

**작성 예시**
```
소    장

원      고  김 ○ ○
            서울시 ○○구 ○○동 1234

피      고  서울○○경찰서장

자동차운전면허취소처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 20○○. 6. 15. 한 자동차운전면허 취소처분을
취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 19○○년 제1종 보통 운전면허를 취득한 이래 약 30년간
   무사고로 운전하여 왔으며, 운전을 생업으로 하고 있습니다.

2. 원고는 20○○. 5. 20. 음주운전(혈중알코올농도 0.05%)으로
   단속되어 피고로부터 운전면허 취소처분을 받았습니다.

3. 그러나 다음과 같은 사정을 고려할 때 이 사건 처분은 재량권을
   일탈·남용한 것입니다.

   가. 원고는 약 30년간 무사고 운전 경력을 가지고 있습니다.
   나. 원고는 택시 운전을 생업으로 하고 있어, 운전면허가 취소되면
       생계 수단을 상실하게 됩니다.
   다. 원고는 노모와 미성년 자녀를 부양하고 있습니다.
   라. 원고의 혈중알코올농도는 0.05%로 취소 기준에 근접한 수치입니다.
   마. 원고는 이 사건 이후 금주를 다짐하고 있습니다.

4. 따라서 이 사건의 경우 면허 취소보다는 면허 정지 처분이 적절하며,
   취소 처분은 비례의 원칙에 위반됩니다.
```

#### 2.2 징계처분취소 청구의 소

**법적 근거**
- 국가공무원법 제78조 (징계 사유)
- 국가공무원법 제79조 (징계의 종류)
- 지방공무원법 제69조, 제70조

**작성 요소**
```python
def write_disciplinary_action_complaint(
    plaintiff: dict,          # 원고 (공무원)
    defendant: str,           # 피고 (임용권자)
    employment_info: dict,    # 임용 정보
    disciplinary_type: str,   # 징계 종류 (파면, 해임, 강등, 정직, 감봉, 견책)
    disciplinary_date: str,   # 징계 처분일
    alleged_facts: str,       # 징계 사유로 된 사실
    defense_grounds: str,     # 방어 사유 (사실 부인, 위법성 조각 등)
    mitigation_grounds: str,  # 참작 사유
    preliminary_procedure: dict, # 소청심사위원회 결정
    evidence: list,
    court: str
) -> str:
    """
    징계처분취소 청구의 소장 작성

    주요 쟁점:
    1. 징계 사유의 존재 여부
    2. 징계 양정의 적정성
    3. 징계절차의 적법성
    4. 재량권 일탈·남용 여부
    """
    pass
```

#### 2.3 건축허가거부처분취소 청구의 소

**법적 근거**
- 건축법 제11조 (건축허가)
- 건축법 제40조 (대지의 조경)
- 국토의 계획 및 이용에 관한 법률

**작성 요소**
```python
def write_building_permit_denial_complaint(
    plaintiff: dict,          # 원고 (건축주)
    defendant: str,           # 피고 (구청장, 시장 등)
    land_info: dict,          # 대지 정보
    building_plan: dict,      # 건축 계획
    permit_application_date: str, # 허가 신청일
    denial_date: str,         # 거부 처분일
    denial_reason: str,       # 거부 사유
    legal_compliance: str,    # 법령 준수 사항
    evidence: list,
    court: str
) -> str:
    """
    건축허가거부처분취소 청구의 소장 작성

    주요 쟁점:
    1. 건축법령상 허가 요건 충족 여부
    2. 거부 사유의 적법성
    3. 재량권 일탈·남용 여부
    """
    pass
```

#### 2.4 토지수용재결처분취소 청구의 소

**법적 근거**
- 공익사업을 위한 토지 등의 취득 및 보상에 관한 법률 제23조 (협의)
- 토지보상법 제28조 (재결 신청)
- 토지보상법 제83조 (이의신청)

**작성 요소**
```python
def write_land_expropriation_complaint(
    plaintiff: dict,          # 원고 (토지소유자)
    defendant: str,           # 피고 (지방토지수용위원회)
    land_info: dict,          # 토지 정보
    expropriation_purpose: str, # 수용 목적 (도로, 공원 등)
    adjudication_date: str,   # 재결일
    compensation_amount: int, # 재결 보상액
    claimed_amount: int,      # 주장 보상액
    appraisal_report: dict,   # 감정평가서
    dispute_reason: str,      # 다툼 사유
    evidence: list,
    court: str
) -> str:
    """
    토지수용재결처분취소 청구의 소장 작성

    주요 쟁점:
    1. 공익사업의 적법성
    2. 보상액 산정의 적정성
    3. 재결 절차의 적법성
    """
    pass
```

#### 2.5 개발부담금부과처분취소 청구의 소

**법적 근거**
- 개발이익 환수에 관한 법률 제5조 (개발부담금의 부과·징수)
- 개발이익 환수에 관한 법률 제10조 (개발이익의 산정)

**작성 요소**
```python
def write_development_charge_complaint(
    plaintiff: dict,          # 원고 (개발사업자)
    defendant: str,           # 피고 (시장, 군수, 구청장)
    development_project: dict, # 개발사업 정보
    charge_amount: int,       # 부과된 개발부담금
    land_value_increase: int, # 지가 상승분
    dispute_reason: str,      # 다툼 사유 (산정 방법 등)
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """개발부담금부과처분취소 청구의 소장 작성"""
    pass
```

#### 2.6 과밀부담금부과처분취소 청구의 소

**법적 근거**
- 수도권정비계획법 제12조 (과밀부담금)

**작성 요소**
```python
def write_congestion_charge_complaint(
    plaintiff: dict,          # 원고
    defendant: str,           # 피고
    facility_info: dict,      # 인구집중유발시설 정보
    charge_amount: int,       # 부과된 과밀부담금
    dispute_reason: str,      # 다툼 사유
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """과밀부담금부과처분취소 청구의 소장 작성"""
    pass
```

#### 2.7 영업정지처분취소 청구의 소

**법적 근거**
- 개별 업종별 법령 (식품위생법, 공중위생관리법 등)

**작성 요소**
```python
def write_business_suspension_complaint(
    plaintiff: dict,          # 원고 (영업자)
    defendant: str,           # 피고 (구청장 등)
    business_info: dict,      # 영업 정보
    suspension_period: str,   # 영업정지 기간
    violation_facts: str,     # 위반 사실
    defense_grounds: str,     # 방어 사유
    mitigation_grounds: str,  # 참작 사유
    preliminary_procedure: dict,
    evidence: list,
    court: str
) -> str:
    """영업정지처분취소 청구의 소장 작성"""
    pass
```

### 3. 행정처분 효력정지 가처분 신청

**법적 근거**
- 행정소송법 제23조 (집행정지)

**작성 요소**
```python
def write_suspension_of_execution_application(
    applicant: dict,          # 신청인
    respondent: str,          # 피신청인
    main_case: str,           # 본안 사건 (사건번호)
    disposition: str,         # 처분 내용
    irreparable_damage: str,  # 회복하기 어려운 손해
    urgent_necessity: str,    # 긴급한 필요
    evidence: list,
    court: str
) -> str:
    """
    행정처분 효력정지 가처분 신청서 작성

    요건:
    1. 본안 소송이 제기되었거나 제기될 것
    2. 처분의 집행으로 인하여 회복하기 어려운 손해 발생 우려
    3. 긴급한 필요가 있을 것
    4. 공공복리에 중대한 영향을 미칠 우려가 없을 것

    청구취지 예시:
    "피신청인이 신청인에 대하여 한 ○○처분은 이 사건 본안 사건의
    판결 확정시까지 그 효력을 정지한다."
    """
    pass
```

### 4. 행정처분 무효확인 청구

**법적 근거**
- 행정소송법 제4조 (무효등확인소송)

**작성 요소**
```python
def write_nullity_confirmation_complaint(
    plaintiff: dict,          # 원고
    defendant: str,           # 피고
    disposition: str,         # 처분 내용
    nullity_grounds: str,     # 무효 사유 (중대·명백한 하자)
    legal_interest: str,      # 확인의 이익
    evidence: list,
    court: str
) -> str:
    """
    행정처분무효확인 청구의 소장 작성

    무효 사유:
    1. 처분의 내용이 법률상 실현 불가능한 것
    2. 처분의 내용이 극히 부당하여 공서양속에 위반되는 것
    3. 처분이 그 상대방이 아닌 제3자에 대한 것인 경우
    4. 처분의 근거 법률이 위헌무효인 경우
    5. 기타 중대하고 명백한 하자가 있는 경우

    청구취지 예시:
    "피고가 원고에 대하여 한 ○○처분은 무효임을 확인한다."
    """
    pass
```

## 청구취지 작성 기준

### 1. 취소소송의 청구취지

```
1. 피고가 원고에 대하여 20○○. ○. ○. 한 ○○처분을 취소한다.
2. 소송비용은 피고의 부담으로 한다.
```

### 2. 무효확인소송의 청구취지

```
1. 피고가 원고에 대하여 20○○. ○. ○. 한 ○○처분은 무효임을 확인한다.
2. 소송비용은 피고의 부담으로 한다.
```

### 3. 부작위위법확인소송의 청구취지

```
1. 피고가 원고의 20○○. ○. ○.자 ○○신청에 대하여 응답하지 아니한
   것은 위법함을 확인한다.
2. 소송비용은 피고의 부담으로 한다.
```

### 4. 의무이행소송의 청구취지

```
1. 피고는 원고에게 ○○처분을 하라.
2. 소송비용은 피고의 부담으로 한다.
```

## 전치절차 작성

행정소송을 제기하기 전에 행정심판을 거쳐야 하는 경우 전치절차를 기재합니다.

```
전 치 절 차

가. 20○○. ○. ○.  처분일
나. 20○○. ○. ○.  고지서 수령일
다. 20○○. ○. ○.  심사청구일
라. 20○○. ○. ○.  심사청구 기각 결정일
마. 20○○. ○. ○.  심사청구 기각 결정서 송달일
바. 20○○. ○. ○.  심판청구일
사. 20○○. ○. ○.  심판청구 기각 결정일
아. 20○○. ○. ○.  심판청구 기각 결정서 송달일
```

**주의사항**
- 조세소송: 국세기본법 제56조에 따라 심사청구 또는 심판청구를 거쳐야 함
- 일반 행정소송: 행정심판을 거치지 않고 바로 소송 제기 가능 (임의적 전치주의)
- 필요적 전치주의가 적용되는 경우: 특별법에 규정된 경우

## 입증방법 작성

```
입 증 방 법

갑 제1호증     납세고지서 (또는 처분서)
갑 제2호증     심사청구 결정서
갑 제3호증의 1 국세심판결정통지
         의 2 심판청구 결정서
갑 제4호증     등기부등본
갑 제5호증     토지대장
갑 제6호증     건축허가서
갑 제7호증     계약서
```

## 첨부서류 작성

```
첨 부 서 류

1. 입증방법            각 1통
1. 법인등기부등본      1통 (법인인 경우)
1. 납부서              1통
1. 위임장              1통 (소송대리인이 있는 경우)
1. 소장부본            1통
```

## 재량행위와 기속행위

### 재량행위에 대한 소송

재량권 일탈·남용 주장:
```
1. 사회통념상 현저히 타당성을 잃은 경우
2. 비례의 원칙 위반
3. 평등의 원칙 위반
4. 신뢰보호의 원칙 위반
5. 부당결부금지의 원칙 위반
```

### 기속행위에 대한 소송

법령 위반 주장:
```
1. 법령상 요건 불충족
2. 법령 해석의 오류
3. 사실관계 인정의 잘못
```

## 행정소송의 종류별 특징

### 1. 취소소송
- 가장 일반적인 행정소송
- 처분의 위법을 이유로 취소를 구함
- 제소기간: 처분이 있음을 안 날로부터 90일, 처분이 있은 날로부터 1년

### 2. 무효등확인소송
- 처분의 무효 확인을 구함
- 중대하고 명백한 하자가 있어야 함
- 제소기간 제한 없음

### 3. 부작위위법확인소송
- 행정청의 부작위가 위법함을 확인
- 의무이행소송의 전제

### 4. 의무이행소송
- 행정청에게 일정한 처분을 할 의무 이행을 구함
- 부작위위법확인판결을 받은 후 제기 가능

## 주요 법리

### 1. 처분성
- 행정청이 행하는 구체적 사실에 관한 법집행으로서의 공권력의 행사
- 국민의 권리의무에 직접 영향을 미치는 행위

### 2. 원고적격
- 처분의 취소를 구할 법률상 이익이 있는 자
- 법률상 보호되는 이익

### 3. 협의의 소의 이익
- 처분의 효과가 소멸한 경우에도 소의 이익이 인정되는 경우:
  - 회복할 법률상 이익이 있는 경우
  - 반복 위험이 있는 경우

### 4. 제소기간
- 주관적 기간: 처분이 있음을 안 날로부터 90일
- 객관적 기간: 처분이 있은 날로부터 1년
- 정당한 사유가 있으면 연장 가능

## 사용 예시

```python
from administrative_litigation_writer import AdministrativeLitigationWriter

writer = AdministrativeLitigationWriter()

# 취득세부과처분취소 소장 작성
acquisition_tax_complaint = writer.write_acquisition_tax_complaint(
    plaintiff={
        "type": "법인",
        "name": "주식회사 ○○",
        "address": "서울시 종로구 ○○동 1234",
        "representative": "김○○",
        "attorney": "변호사 이○○"
    },
    defendant={
        "name": "서울시 종로구청장",
        "address": "서울시 종로구 ○○동 5678"
    },
    land_info={
        "location": "서울시 종로구 ○○동 1284의 2",
        "area": "2,449㎡",
        "acquisition_date": "20○○. 6. 10."
    },
    tax_amount=174159420,
    acquisition_purpose="업무용 건물 신축 부지",
    use_plan="백화점 건물 신축 계획으로 토지 취득",
    legal_grounds="건설부장관의 건축허가 제한조치 (20○○.5.15~12.31)",
    preliminary_procedure={
        "disposition_date": "20○○. 2. 10.",
        "receipt_date": "20○○. 2. 15.",
        "review_request_date": "20○○. 3. 15.",
        "review_decision_date": "20○○. 4. 10.",
        "appeal_request_date": "20○○. 5. 10.",
        "appeal_decision_date": "20○○. 6. 15."
    },
    evidence=[
        "납세고지서",
        "심사청구 결정서",
        "국세심판결정통지",
        "토지등기부등본",
        "건축허가 신청서"
    ],
    court="서울행정법원"
)

# 운전면허취소처분취소 소장 작성
license_complaint = writer.write_license_revocation_complaint(
    plaintiff={
        "name": "김○○",
        "address": "서울시 ○○구 ○○동 1234",
        "resident_number": "XXXXXX-XXXXXXX"
    },
    defendant="서울○○경찰서장",
    license_info={
        "type": "제1종 보통",
        "acquisition_date": "19○○. ○. ○.",
        "license_number": "11-○○-○○○○○○-○○"
    },
    revocation_date="20○○. 6. 15.",
    violation_facts="음주운전 (혈중알코올농도 0.05%)",
    mitigation_grounds="""
    1. 약 30년간 무사고 운전 경력
    2. 택시 운전을 생업으로 함 (면허 취소시 생계 곤란)
    3. 노모 및 미성년 자녀 부양
    4. 혈중알코올농도가 취소 기준에 근접한 수치
    5. 이 사건 이후 금주 다짐
    """,
    preliminary_procedure={
        "appeal_request_date": "20○○. 7. 1.",
        "appeal_decision_date": "20○○. 8. 15."
    },
    evidence=[
        "운전면허증",
        "운전경력증명서",
        "무사고 경력 확인서",
        "가족관계증명서",
        "소득금액증명원"
    ],
    court="서울행정법원"
)
```

## 참고사항

1. **관할법원**: 피고의 소재지를 관할하는 행정법원 (서울, 부산 등)
2. **인지액**: 소송목적의 값에 따라 산정
3. **송달료**: 1회 송달료 × 5회분 예납
4. **변호사 강제주의**: 행정소송은 변호사를 선임해야 함 (본인 소송 불가)

## 관련 법령

- 행정소송법
- 행정심판법
- 행정절차법
- 국세기본법 (조세소송)
- 지방세기본법 (지방세 소송)
- 개별 행정작용법 (건축법, 도로교통법 등)

---

**버전 정보**
- 버전: 1.0.0
- 최종 수정일: 2025-11-11
- 참조 데이터: 행정소송 소장 (백영사, 19,363줄)
