---
name: direct_examination_matters
description: "대한민국 민사소송법에 따른 증인신문사항 자동 생성 스킬. 증인(의료인 포함)에 대한 전략적 신문 질문 생성. 의료소송 특화 기능 포함. 증거 기반 신문사항 작성 및 체계적 질문 구성."
license: Proprietary. For LawPro AI Platform use only.
version: 2.0.0
---

# 증인신문사항 작성 스킬 (Witness Examination Matters Writer Skill)

## 개요

민사소송에서 자기 측 증인에 대한 증인신문사항(주신문)을 전략적으로 생성하여, 증거 기반 신문 질문 작성 및 의료소송 특화 기능을 제공하는 스킬입니다.

**주요 기능:**
- **전략적 질문 작성**: 증거 기반 신문 질문 생성
- **템플릿 기반 구조**: 신문 단계별 체계적 구성
- **의료소송 특화**: 의료인 증인에 대한 전문 질문 생성
- **법원 양식 준수**: 규칙 제80조에 따른 표준 양식
- **다양한 질문 유형**: 인적사항, 사실관계, 서증 진정성립, 전문가 의견
- **반대신문 대비**: 상대방 반대신문 예상 질문 준비

## 문서의 목적

증인신문사항은 증인을 신청한 당사자가 법정에서 주신문(증인신문)을 위해 준비하는 질문으로서:

1. **Establish witness credibility** (증인 신빙성 확립): Identity, relationship, basis of knowledge
2. **Elicit favorable testimony** (유리한 증언 도출): Facts supporting party's case
3. **Authenticate evidence** (증거 진정성립): Verify documents and physical evidence
4. **Build chronological narrative** (시간순 서사 구축): Tell story through witness
5. **Prepare for cross-examination** (반대신문 대비): Anticipate and address weaknesses

**Filing Requirement**: Submit examination questions by court-ordered deadline (규칙 제80조 제1항)

## Question Structure

### Phase 1: Identity Questions (신원 확인)
```
1. 증인의 성함은 무엇입니까?
2. 증인의 직업은 무엇입니까?
3. 증인은 원고(피고)와 어떤 관계입니까?
4. 증인은 언제부터 원고(피고)를 알게 되었습니까?
```

### Phase 2: Background & Context (배경 및 맥락)
```
5. 증인은 이 사건과 관련하여 어떤 사실을 알고 있습니까?
6. 증인이 그러한 사실을 알게 된 경위는 무엇입니까?
7. 증인이 사건 당시 어디에 있었습니까?
```

### Phase 3: Key Facts (핵심 사실)
```
8. 2024. 1. 15. 어디에서 무엇을 하고 있었습니까?
9. 그때 누구를 보았습니까?
10. 그 사람이 무슨 말을 하였습니까?
11. 증인은 그 말을 직접 들었습니까?
12. 그 말을 들었을 때 다른 사람도 함께 있었습니까?
```

### Phase 4: Document Authentication (문서 진정성립)
```
13. 갑 제1호증 계약서를 보여드립니다. 이 문서를 본 적이 있습니까?
14. 이 문서를 언제 어디서 보았습니까?
15. 이 문서의 서명이 누구의 것입니까?
16. 증인이 직접 서명하는 것을 보았습니까?
```

### Phase 5: Confirmation & Details (확인 및 세부사항)
```
17. 거래 금액은 얼마였습니까?
18. 금원은 어떤 방법으로 교부되었습니까?
19. 증인이 금원 교부 현장에 있었습니까?
20. 그 외에 기억나는 중요한 사실이 있습니까?
```

## Quick Start

```python
from direct_examination_matters import DirectExaminationWriter

writer = DirectExaminationWriter()

# Generate witness examination questions (주신문 질문 생성)
questions = writer.generate_questions(
    witness_profile={
        "name": "홍길동",
        "occupation": "회사원",
        "relationship_to_party": "원고의 친구",
        "basis_of_knowledge": "거래 현장 목격"
    },
    case_facts={
        "case_type": "금전소비대차",
        "transaction_date": "2024. 1. 15.",
        "transaction_location": "서울 강남구 테헤란로 123 카페",
        "amount": "10,000,000원",
        "key_events": [
            "피고가 원고에게 차용 요청",
            "원고가 현금 교부",
            "피고가 차용증서 작성"
        ]
    },
    evidence_to_authenticate=[
        "갑 제1호증 차용증서",
        "갑 제2호증 통장 사본"
    ],
    strategy="주장사실 입증 및 차용증서 진정성립"
)

# Questions organized by phase
for phase, questions in questions.by_phase.items():
    print(f"\n{phase}")
    for i, q in enumerate(questions, 1):
        print(f"{i}. {q}")
```

## Question Types

### 1. Identity Questions (신원 확인 질문)

**Purpose**: Establish witness identity and credibility

```python
identity_questions = [
    "증인의 성함은 무엇입니까?",
    "증인의 주소는 어디입니까?",
    "증인의 직업은 무엇입니까?",
    "증인은 원고를 알고 있습니까?",
    "증인과 원고는 어떤 관계입니까?",
    "언제부터 원고를 알게 되었습니까?"
]
```

### 2. Fact Witness Questions (사실 확인 질문)

**Purpose**: Elicit testimony about witnessed events

```python
fact_questions = [
    "증인은 2024. 1. 15. 어디에 있었습니까?",
    "그때 누구와 함께 있었습니까?",
    "원고와 피고가 만났습니까?",
    "두 사람이 무슨 이야기를 하였습니까?",
    "증인은 그 대화를 직접 들었습니까?",
    "피고가 금원을 차용한다는 말을 하였습니까?",
    "차용 금액은 얼마라고 하였습니까?",
    "원고가 금원을 교부하는 것을 보았습니까?",
    "금원은 어떤 형태로 교부되었습니까?",
    "피고가 차용증서를 작성하였습니까?",
    "증인이 직접 보았습니까?"
]
```

### 3. Document Authentication Questions (문서 진정성립 질문)

**Purpose**: Authenticate documents for admission into evidence

```python
document_questions = [
    "갑 제1호증 차용증서를 보여드립니다. 이 문서를 본 적이 있습니까?",
    "이 문서를 언제 어디서 보았습니까?",
    "이 문서의 작성 과정을 보았습니까?",
    "이 문서를 누가 작성하였습니까?",
    "이 서명이 누구의 것입니까?",
    "증인이 직접 서명하는 것을 보았습니까?",
    "서명할 당시 피고의 의식이 명료하였습니까?",
    "피고가 자발적으로 서명하였습니까?"
]
```

### 4. Expert Witness Questions (전문가 증언 질문)

**Purpose**: Elicit expert opinion testimony

```python
expert_questions = [
    "증인의 전문분야는 무엇입니까?",
    "증인의 학력 및 경력을 말씀해주십시오.",
    "관련 분야 근무 경력은 얼마나 됩니까?",
    "관련 자격증이나 면허를 가지고 있습니까?",
    "이 사건 부동산을 감정하였습니까?",
    "감정 방법은 무엇입니까?",
    "감정 결과는 어떻게 되었습니까?",
    "감정액은 얼마입니까?",
    "그렇게 평가한 근거는 무엇입니까?"
]
```

## Medical Witness Examination (의료소송 증인신문)

### Overview

Medical witness examination requires specialized preparation and strategic questioning. Medical professionals tend to be defensive and colleague-protective, requiring careful question formulation.

### Medical Witness Types

#### Type 1: Treating Physician (치료 의사)

**Challenge**: Defensive, colleague-protective, uses technical jargon to evade

**Strategy**: Specific factual questions, timeline reconstruction, standard of care comparison

```python
treating_physician_questions = [
    # Phase 1: Timeline (시간순 확인)
    "환자가 내원한 정확한 시간은 몇 시 몇 분입니까?",
    "최초 진찰을 시작한 시간은 언제입니까?",
    "진찰 소요 시간은 얼마나 되었습니까?",

    # Phase 2: Initial Assessment (초기 평가)
    "환자의 주소증상은 무엇이었습니까?",
    "환자가 호소한 통증 부위는 어디입니까?",
    "통증 정도는 어느 정도였습니까?",
    "환자의 바이탈 사인(혈압, 맥박, 체온)은 어떠하였습니까?",

    # Phase 3: Diagnostic Process (진단 과정)
    "어떤 검사를 시행하였습니까?",
    "검사 결과는 언제 확인하였습니까?",
    "검사 결과 수치는 어떠하였습니까?",
    "○○ 검사를 시행하지 않은 이유는 무엇입니까?",
    "다른 질환의 가능성도 고려하였습니까?",

    # Phase 4: Treatment Decision (치료 결정)
    "어떤 진단을 내렸습니까?",
    "진단의 근거는 무엇입니까?",
    "어떤 치료방법을 선택하였습니까?",
    "○○ 치료법을 선택한 이유는 무엇입니까?",
    "다른 치료방법(△△)도 고려하였습니까?",
    "환자나 가족에게 치료방법을 설명하였습니까?",
    "설명한 내용은 무엇입니까?",
    "동의를 받았습니까?",

    # Phase 5: Procedure Details (시술/수술 세부사항)
    "수술을 시행한 시간은 언제입니까?",
    "수술 소요 시간은 얼마나 되었습니까?",
    "수술 전 환자 상태는 어떠하였습니까?",
    "마취는 어떤 방법으로 하였습니까?",
    "수술 중 특이사항이 있었습니까?",
    "출혈량은 얼마나 되었습니까?",
    "합병증이 발생하였습니까?",
    "합병증 발생 시점은 언제입니까?",
    "합병증에 어떻게 대처하였습니까?",

    # Phase 6: Post-Treatment (치료 후 관리)
    "술후 환자를 언제 확인하였습니까?",
    "술후 관찰은 얼마나 자주 하였습니까?",
    "환자가 ○○ 증상을 호소하였습니까?",
    "그 증상 발생 시점은 언제입니까?",
    "그때 어떤 조치를 취하였습니까?",
    "추가 검사를 시행하였습니까?",
    "전원이 필요하다고 판단하였습니까?",
    "전원을 결정한 시점은 언제입니까?",
    "전원 결정 이유는 무엇입니까?"
]
```

**✅ Effective Question Pattern**:
```
"혈압이 80/50mmHg로 측정되었습니까?"  # Specific fact
"그때 승압제를 투여하였습니까?"  # Concrete action
"투여하지 않았다면 그 이유는 무엇입니까?"  # Explanation required
```

**❌ Ineffective Question Pattern**:
```
"적절한 조치를 취하였습니까?"  # Vague, allows "Yes"
"왜 그렇게 하셨습니까?"  # Open-ended excuse invitation
```

#### Type 2: Nurses (간호사)

**Advantage**: Less defensive than physicians, better factual recall

**Strategy**: Focus on observations, vital signs, patient complaints, medication timing

```python
nurse_questions = [
    # Vital Signs (활력징후)
    "환자의 혈압을 몇 시에 측정하였습니까?",
    "측정 결과는 어떠하였습니까?",
    "정상 범위입니까?",
    "이상 소견이 있었습니까?",
    "이상 소견을 의사에게 보고하였습니까?",
    "보고한 시간은 언제입니까?",

    # Patient Complaints (환자 호소)
    "환자가 통증을 호소하였습니까?",
    "통증 호소 시간은 언제입니까?",
    "통증 부위는 어디입니까?",
    "통증 정도는 얼마나 되었습니까? (VAS 점수)",
    "이를 의사에게 보고하였습니까?",

    # Medication (투약)
    "환자에게 어떤 약물을 투여하였습니까?",
    "투여 시간은 몇 시입니까?",
    "투여 용량은 얼마입니까?",
    "투여 경로는 무엇이었습니까? (정맥/근육/경구)",
    "의사의 처방에 따른 것입니까?",
    "투여 후 환자 반응은 어떠하였습니까?",

    # Medical Records (진료기록)
    "간호기록을 작성하였습니까?",
    "기록 작성 시간은 언제입니까?",
    "실시간으로 기록하였습니까?",
    "나중에 추가 기재한 부분이 있습니까?",
    "수정한 부분이 있습니까?",
    "수정 이유는 무엇입니까?"
]
```

#### Type 3: Expert Witness (감정의)

**Purpose**: Establish standard of care and causation

**Strategy**: Qualifications → Standards → Deviation → Causation

```python
expert_witness_questions = [
    # Qualifications (자격)
    "증인의 전문 진료과는 무엇입니까?",
    "전문의 자격을 취득한 시기는 언제입니까?",
    "○○ 질환 치료 경험은 얼마나 됩니까?",
    "○○ 질환 관련 논문을 발표한 적이 있습니까?",
    "관련 학회 활동을 하고 있습니까?",

    # Standard of Care (표준 진료)
    "○○ 질환의 표준 치료 지침이 있습니까?",
    "어떤 학회나 기관의 지침입니까?",
    "그 지침의 권고사항은 무엇입니까?",
    "임상 현장에서 일반적으로 따르는 방법입니까?",

    # Record Review (기록 검토)
    "이 사건 진료기록을 검토하였습니까?",
    "피고 의사가 시행한 검사는 무엇입니까?",
    "피고 의사가 선택한 치료방법은 무엇입니까?",

    # Deviation Analysis (표준 대비 평가)
    "피고 의사의 치료가 표준 지침에 부합합니까?",
    "부합하지 않는다면 어떤 점이 다릅니까?",
    "시행하지 않은 검사가 있습니까?",
    "그 검사가 필요한 상황이었습니까?",

    # Alternative Approach (대체 방법)
    "다른 치료방법이 가능하였습니까?",
    "그 방법이 더 적절하였을 가능성이 있습니까?",
    "왜 그렇게 판단하십니까?",

    # Causation (인과관계)
    "환자의 현재 상태가 피고의 치료와 관련이 있습니까?",
    "적절한 치료가 이루어졌다면 결과가 달라졌을 가능성이 있습니까?",
    "그 가능성은 어느 정도라고 보십니까?",
    "의학적 근거는 무엇입니까?"
]
```

### Question Formulation Principles

#### ✅ Effective Questions

1. **Specific and Concrete** (구체적):
   ```
   "혈압이 몇이었습니까?"
   "몇 시에 측정하였습니까?"
   ```

2. **Yes/No with Follow-up** (예/아니오 + 후속):
   ```
   "승압제를 투여하였습니까?"
   "투여하지 않았다면 그 이유는 무엇입니까?"
   ```

3. **Timeline-Based** (시간순):
   ```
   "환자 내원 시간은 언제입니까?"
   "진찰 시작 시간은 언제입니까?"
   "수술 시작 시간은 언제입니까?"
   ```

4. **Fact-Based** (사실 기반):
   ```
   "검사 결과 수치는 얼마였습니까?"
   "출혈량은 얼마나 되었습니까?"
   ```

#### ❌ Ineffective Questions

1. **Legal Conclusions** (법적 결론):
   ```
   "과실이 있었습니까?"
   "의료사고입니까?"
   ```

2. **Vague Questions** (애매한 질문):
   ```
   "적절하였습니까?"
   "왜 그렇게 하셨습니까?"
   ```

3. **Multiple Questions** (복합 질문):
   ```
   "검사를 했는지, 했다면 결과는 어떠하였고, 조치는 취하였습니까?"
   ```

4. **Leading Questions** (유도 질문) - generally prohibited:
   ```
   "검사를 하지 않았지 않습니까?"
   ```

## AI-Powered Question Generation

```python
from direct_examination_matters import MedicalQuestionGenerator

generator = MedicalQuestionGenerator()

# Generate questions based on medical records analysis
questions = generator.generate_medical_questions(
    witness_type="treating_physician",
    medical_records={
        "emergency_room": "응급실기록.pdf",
        "patient_chart": "환자차트.pdf",
        "surgical_note": "수술기록.pdf"
    },
    case_theory="수술 전 검사 소홀 및 술후 관리 부적절",
    deviation_points=[
        "수술 전 MRI 검사 미실시",
        "수술 중 척수 손상",
        "술후 신경학적 검사 지연",
        "합병증 발생 시 대처 지연"
    ],
    web_search=True  # Search for medical standards
)

# Output: Structured questions by phase
# - Timeline questions
# - Diagnostic process questions
# - Treatment decision questions
# - Post-treatment management questions
```

## Question Organization

### By Phase (단계별)
```python
questions_by_phase = {
    "신원 확인": [identity_questions],
    "배경 설명": [context_questions],
    "핵심 사실": [key_fact_questions],
    "문서 진정성립": [document_questions],
    "확인 및 마무리": [confirmation_questions]
}
```

### By Evidence (증거별)
```python
questions_by_evidence = {
    "갑 제1호증 차용증서": [document_auth_q1, document_auth_q2],
    "갑 제2호증 통장 사본": [document_auth_q3, document_auth_q4]
}
```

### By Issue (쟁점별)
```python
questions_by_issue = {
    "계약 체결 사실": [contract_formation_questions],
    "금원 교부 사실": [payment_questions],
    "변제 약정 사실": [repayment_agreement_questions]
}
```

## Integration with Witness Examination Form

```python
from witness_examination_form import WitnessExaminationFormWriter
from direct_examination_matters import DirectExaminationWriter

# Generate questions
question_writer = DirectExaminationWriter()
questions = question_writer.generate_questions(
    witness_profile=witness_profile,
    case_facts=case_facts,
    strategy="establish_contract_formation"
)

# Use in witness examination form
form_writer = WitnessExaminationFormWriter()
examination_form = form_writer.write(
    case_number="2024가단123456",
    examination_questions=questions.formatted_list,
    witness=witness_profile,
    evidentiary_purpose=questions.purpose
)
```

## Preparation Requirements

Before generating examination questions:

1. **Case Analysis** (사건 분석):
   - Review all case facts
   - Identify disputed issues
   - Determine evidentiary needs

2. **Witness Profiling** (증인 분석):
   - Witness relationship to parties
   - Basis of witness knowledge
   - Potential credibility issues

3. **Evidence Review** (증거 검토):
   - Documents to authenticate
   - Physical evidence to explain
   - Prior statements to confirm

4. **Strategy Planning** (전략 수립):
   - What facts to establish
   - What documents to authenticate
   - How to address weaknesses

5. **Medical Cases - Additional Prep** (의료소송 추가 준비):
   - Medical record translation
   - Medical literature research
   - Standard of care identification
   - Expert consultation

## Performance

| Metric | Value |
|--------|-------|
| Question generation time | 15-30 seconds |
| Average questions per witness | 15-25 |
| Medical witness questions | 30-50 |
| Token usage | ~300-800 tokens |

## Reference Materials

### Medical Litigation Resource
- **File**: `/document_skills/reference_materials/민사소송/의료소송_서류_및_작성법`
- **Section**: 제19절 증인신문 (pp. 1201-1215)
- **Content**: Complete medical witness examination strategies

### Key Principles

**Principle 1**: "의료소송에서 제일 힘든 것이 증인신문이다."
(Medical witness examination is the most difficult aspect of medical litigation.)

**Principle 2**: "준비를 어느 정도 하였는가에 달려 있다고 해도 과언이 아니다."
(Success depends entirely on preparation level.)

**Principle 3**: "준비가 되지 않아 막연히 성과 없는 신문을 하거나 의료행위의 내용이나 용어 등을 묻는 형태의 신문은 오히려 의사들의 주장을 정당화시킬 우려가 있으므로 신문을 하지 아니한 것만도 못하다."
(Unprepared examination that merely asks about medical procedures can backfire, making it worse than no examination at all.)

## 라이선스

Proprietary. For LawPro AI Platform use only.

---

**최종 업데이트**: 2025-11-15
**상태**: ✅ 한국 민사소송법 법률용어 완전 적용

**참고**: 이 스킬은 법정에서 사용하거나 증인신문신청서에 첨부할 증인신문사항(주신문)을 생성합니다. 생성된 질문은 변호사의 검토를 거쳐 각 사건에 맞게 조정되어야 합니다. 의료인 증인 질문은 진료기록 검토, 의학문헌 연구, 전문가 자문을 포함한 철저한 사전 준비가 필요합니다.

**법률용어 정리**:
- **증인신문**: 증인에 대한 신문 전체 과정
- **주신문**: 증인을 신청한 당사자가 하는 신문 (Direct Examination)
- **반대신문**: 상대방 당사자가 하는 신문 (Cross-Examination)
