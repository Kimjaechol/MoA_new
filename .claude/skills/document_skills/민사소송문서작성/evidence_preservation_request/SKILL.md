---
name: evidence_preservation_request
description: "대한민국 민사소송법에 따른 증거보전신청서 자동 작성 스킬. 의료소송 특화. 진료기록 위·변조 방지를 위한 기습 검증 신청. 의료과오 사건에 필수적이며 상세한 진료기록 목록 포함. 법원 제출용 DOCX/PDF 문서 생성."
license: Proprietary. For LawPro AI Platform use only.
version: 2.0.0
---

# 증거보전신청서 작성 스킬 (Evidence Preservation Request Writer Skill)

## Overview

Generates professional evidence preservation requests (증거보전신청서) for securing medical records before they can be altered, destroyed, or concealed. **Essential for medical malpractice litigation** where medical records are concentrated with defendants and risk of tampering is high.

**Key Features:**
- **Surprise inspection (기습성)**: Coordinates service timing to prevent record alteration
- **Comprehensive medical record lists**: 27+ types of medical documents
- **Dual methods**: Document inspection (서증) and physical inspection (검증)
- **Pre-litigation filing**: Can be filed before lawsuit
- **Court jurisdiction**: Files with location court, not defendant's domicile

## Document Purpose

증거보전 (Evidence Preservation) is a critical pre-litigation procedure that allows parties to:

1. **Secure evidence early** (조기 확보): Obtain medical records before filing lawsuit
2. **Prevent alteration** (위·변조 방지): Surprise inspection prevents tampering
3. **Clarify facts** (사실관계 파악): Understand case before committing to litigation
4. **Avoid frivolous suits** (무익한 소송 방지): Assess merits before filing
5. **Establish tampering** (입증방해 주장): If altered, claim evidence spoliation

###

 Necessity in Medical Litigation

**Information Asymmetry** (정보 편중):
- Medical treatment occurs in closed rooms (밀실성)
- Patient unconscious during surgery (마취 상태)
- All records held by medical institution
- Patient has only fragments (진단서, 투약봉지 등)

**Alteration Risk** (위·변조 위험):
- Medical institutions control records
- High incentive to modify after accident
- Court decisions recognize this risk (서울고법 1994. 6. 22.선고 92나67782)
- Once altered, proving alteration nearly impossible

## Document Structure

### 1. Caption (표제부)
```
                증거보전신청서

신 청 인  김철수
         서울 서초구 서초동 123
         전화 010-1234-5678

피신청인  의료법인 ○○의료재단
         서울 강남구 테헤란로 456
         대표자 이사장 박영희
```

### 2. Request (신청취지)
```
위 사건에 관하여 신청인은 증거보전을 위하여 다음과 같이
검증 및 제출명령을 구합니다.

다    음

1. 이 건에 대하여 피신청인의 주소지 소재 ○○병원 의무기록실에
   임하여 피신청인이 소지하는 아래 기재 문서를 검증한다.

2. 피신청인은 위 문서를 이 건 증거조사기일에 현장에서 제시하라.

3. 위 증거조사기일은 2025년 ○월 ○일 ○○:○○경 실시한다.

라는 결정을 구합니다.
```

### 3. Facts to Be Proven (증명하여야 할 사실)
```
피신청인의 피용자인 ○○과 의사 ○○○가 신청인에 대한 치료에 임함에
있어서 의사로서 다해야 할 업무상의 주의의무를 위반하여 위 신청인으로
하여금 ○○○ 등의 신체적 상해를 입게 한 사실
```

### 4. Evidence Subject (감정 목적물)
```
피신청인 병원 의무기록실에서 보관하고 있는 신청인 ○○○
(주민등록번호 000000-0000000, 서울시 ○○구 ○○동 100번지)에 대한

1. 진단요약색인기록지 (Diagnostic Summary Index)
2. 입퇴원기록지 (Admission and Discharge Record)
3. 퇴원요약지 (Discharge Summary)
[... 27 types total - see below ...]
27. 기타 위 환자와 관련된 진료기록 일체
```

### 5. Preservation Grounds (증거보전을 필요로 하는 사유)
```
가. 정보 편중
   의료행위는 진료실이나 수술실 등 밀실에서 행해지고,
   진료정보를 대개 의사측에서 가지고 있을 만큼
   진료정보의 편중성이 심합니다.

나. 위·변조 우려
   진료기록이나 각종 검사지 등을 의사가 가지고 있어
   의료사고가 발생하면 언제든지 기록의 위·변조나
   폐기, 은닉의 개연성이 있습니다.

다. 소명자료
   [진단서, 진찰권, 환자 진술서, 의학문헌 첨부]
```

## Complete Medical Record List (27 Types)

For medical malpractice cases, request all types:

```python
MEDICAL_RECORDS = [
    "1. 진단요약색인기록지 (Diagnostic Summary Index)",
    "2. 입퇴원기록지 (Admission and Discharge Record)",
    "3. 퇴원요약지 (Discharge Summary)",
    "4. 병력기록지 (History)",
    "5. 신체검진기록지 (Physical Examination)",
    "6. 경과기록지 (Progress Note)",
    "7. 수술기록지 (Operation Record)",
    "8. 의사지시전 (Doctor's Orders)",
    "9. 협의진료기록지 (Consultation Record)",
    "10. 임상병리검사보고서 (Laboratory Reports)",
    "11. 조직병리검사보고서 (Pathology Report)",
    "12. 그라프기록지 (Graphic Record)",
    "13. 약물투입배출표 (Intake and Output Chart)",
    "14. 실측정치기록지 (Vital Sign Record)",
    "15. 검사결과기록지 (Flow Sheet)",
    "16. 간호기록지 (Nurses' Record)",
    "17. 방사선기록지 (X-ray Report)",
    "18. 방사선필름 (X-ray Film, CT Film, MRI Film 등)",
    "19. 심전도검사보고서 (Electrocardiographic Report)",
    "20. 뇌파검사보고서 (Electroencephalographic Report)",
    "21. 마취기록지 (Anesthesia Record)",
    "22. 회복실기록지 (Recovery Room Record)",
    "23. 수혈기록지 (Transfusion Record)",
    "24. 응급실기록지 (Emergency Room Record)",
    "25. 수술, 마취청약서 (Operation Anesthesia Consent Form)",
    "26. 자퇴서약서 (Discharge Consent Form)",
    "27. 기타 위 환자와 관련된 진료기록 일체"
]
```

## Jurisdictional Rules

### Pre-Litigation (제소 전)
```python
jurisdiction = {
    "court": "지방법원 단독판사",
    "location": "진료기록 소재지 또는 검증 목적물 소재지"
}

# Example:
# Hospital location: Seoul Central District
# Corporation HQ: Daejeon District
# → File with Seoul Central (hospital location)
```

### Post-Litigation (소송계속 중)
```python
jurisdiction = {
    "primary": "본안소송 계속 법원",
    "emergency": "진료기록 소재지 법원 (급할 경우)"
}
```

## Inspection Methods

### Method 1: Document Inspection (서증 조사) - Preferred by Courts
```
증거보전의 방법으로 서증 조사를 신청합니다.

피신청인이 소지하는 아래 기재 문서를 제출하게 하여
이를 조사할 것을 구합니다.
```

**Advantages**:
- Simpler procedure for court
- Less formal inspection report needed
- Documents can be copied easily

### Method 2: Physical Inspection (검증) - Better for Tampering Risk
```
증거보전의 방법으로 검증을 신청합니다.

피신청인의 주소지 소재 ○○병원 의무기록실에 임하여
피신청인이 소지하는 문서를 검증할 것을 구합니다.
```

**Advantages**:
- Surprise element preserved
- Detect physical alteration (correction fluid, different ink)
- Public duty to cooperate (검증의무)
- Better for photographs of tampering

### Method 3: Combined (서증 + 검증)
```
검증과 제출명령을 병합하여 신청합니다.
```

## Service Timing Strategy (Critical)

**Goal**: Serve decision as close to inspection date as possible to prevent tampering.

### Service Methods (송달방법)

**① Simultaneous Service (동시송달)** - Best:
```
Court clerk serves decision document directly to hospital
representative at inspection time (민사소송법 제177조 제1항)
```

**② Special Delivery by Court Marshal (집행관 송달)** - Good:
```
Court marshal serves 1-2 hours before inspection
(민사소송법 제176조 제1항)
```

**③ Postal Service (우편송달)** - ❌ Avoid:
```
Postal service delivers 3-7 days before inspection
→ Allows time for record alteration
```

**Strategy**: Coordinate with court to use Method ① or ② to maintain surprise element.

## Day-of-Inspection Procedures

### Attorney Checklist

**Before Inspection**:
- [ ] Bring photographer (if alteration suspected)
- [ ] Bring copying equipment (if hospital lacks facilities)
- [ ] Confirm hospital operating hours
- [ ] Prepare 2 copies of all documents (1 for court record, 1 for attorney)

**During Inspection**:
- [ ] Photograph visible alterations (correction fluid, different ink, added notes)
- [ ] Note missing records in inspection report
- [ ] Copy X-rays, CT, MRI films
- [ ] For electronic records, request creation/modification timestamps
- [ ] Request signed certification if hospital claims records lost

**Special Considerations**:
- **Large volumes** (심전도, 뇌파): Note existence without copying if not probative
- **Electronic records**: Require certified electronic signature (공인전자서명)
- **Film copies**: Use hospital equipment or take to another facility
- **Tissue samples**: Photograph (cannot copy)

## Post-Inspection Procedures

### 1. Translation (번역)
```python
# Medical records contain:
- Latin terms
- English medical abbreviations
- German terminology
- Medical shorthand

# AI translation service needed
```

### 2. Analysis (분석)
```python
# Work with:
- Medical literature
- Case law
- Expert consultants
- AI medical record analyzer
```

### 3. Follow-up Actions
- File lawsuit if malpractice confirmed
- Request expert examination (감정신청)
- Claim evidence spoliation if tampering found
- Negotiate settlement with evidence in hand

## Electronic Medical Records (전자의무기록)

### Special Requirements

**Public Key Infrastructure (공인전자서명)**:
```python
requirements = {
    "certification": "공인인증기관 발급 공인인증서",
    "standards": "전자서명법 준수",
    "audit_trail": "작성/수정 일시 기록"
}

# Major hospitals compliant:
# - Seoul National University Hospital
# - Asan Medical Center
# - Samsung Medical Center
# - Severance Hospital
```

**Non-Compliant Records**:
```
If electronic records lack certified signature:
1. Violates 의료법
2. Request printed version with signature/seal
3. Request creation/modification logs
4. Use as evidence of poor record-keeping
```

## Appeal Rights

**No Appeal** (민사소송법 제380조):
```
Whether granted or denied, evidence preservation
decisions cannot be appealed.
```

**Strategy**: Get it right the first time. Carefully draft application.

## Success Factors

### High Grant Rate Factors:
1. ✅ Abstract alteration risk claim sufficient ("위·변조 우려 있음")
2. ✅ Medical records subject to disclosure right (의료법 제20조)
3. ✅ No disadvantage to医生院 from disclosure
4. ✅ Serves truth-finding function
5. ✅ Patient difficulty proving specific alteration risk

### Abuse Prevention:
- Circumstantial evidence helpful (but not required):
  - Prior alteration history
  - Evasive explanations
  - Contradictory statements
  - Uncooperative attitude

## Integration with Evidence Examination

```python
# Workflow:
preservation_request = EvidencePreservationWriter().write(
    applicant="김철수",
    respondent="의료법인 ○○재단",
    hospital_location="서울 강남구 테헤란로 456",
    patient_info={...},
    incident_date="2024. 4. 3.",
    injury="하반신 마비"
)

# After inspection:
medical_records = preservation_request.inspect()

# Analyze with AI:
analysis = MedicalRecordAnalyzer().analyze(
    medical_records,
    web_search=True
)

# If malpractice found:
if analysis.malpractice_likely:
    # File expert examination request
    examination = ExpertExaminationWriter().write(
        examination_type="medical_record",
        medical_records=medical_records,
        examination_matters=analysis.suggested_matters
    )

    # File complaint
    complaint = ComplaintWriter().write(
        case_type="medical_malpractice",
        facts=analysis.timeline,
        legal_claims=analysis.negligence_points
    )
```

## Reference Materials

### Source Document
- **File**: `/document_skills/reference_materials/민사소송/의료소송_서류_및_작성법`
- **Section**: 제4절 증거보전에 관하여 (pp. 129-138)
- **Content**: 9 sections covering all aspects of evidence preservation

### Key Court Decisions

**서울고법 1994. 6. 22.선고 92나67782판결**:
```
"의료과오를 원인으로 하는 소송사건에 있어서는
그 증거가 모두 병원 또는 의사측에 편중되어 있고,
환자로서는 그 의료행위의 과정도 알 수 없는 것..."

"이 사건 소제기 후 원고에 대한 의사기록 중 원고의
진단명 중 일부가 피고병원측 사람의 소행으로 흑색볼펜으로
가필되어 원래의 진단명을 식별할 수 없게 변조된 점..."
```

## 라이선스

Proprietary. For LawPro AI Platform use only.

---

**최종 업데이트**: 2025-11-15
**상태**: ✅ 한국 민사소송법 법률용어 완전 적용

**참고**: 이 스킬은 LawPro Claude Skills 패키지에 포함된 표준 docx 및 pdf 스킬과 통합되어 전문적인 문서 서식과 생성을 지원합니다. 모든 증거보전신청서는 민사소송법 제375조를 준수합니다.

**중요**: 증거보전은 의료과오소송에서 종종 **가장 중요한 절차**입니다. 기습 검증을 통해 적절히 확보된 진료기록 없이는, 정보 비대칭성 및 기록 위·변조 위험이 높아 의료과오 입증이 거의 불가능해집니다.
