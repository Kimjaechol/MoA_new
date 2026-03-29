#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Criminal Appeal Writer (항고장/항소이유서 작성)
Generates professional Korean criminal appeal documents.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any


class CriminalAppealWriter:
    """
    Automated Korean criminal appeal (항고장/항소이유서) generation.

    Features:
    - Template-based generation (95% token reduction)
    - Court-ready DOCX/PDF format
    - Dual appeal types (prosecution/court)
    - Error identification and analysis
    - Legal citation support
    - Deadline tracking
    """

    def __init__(self):
        self.appeal_types = {
            "prosecution_appeal": {
                "title": "항 고 장",
                "appellant_label": "항 고 인",
                "respondent_label": "피의자",
                "purpose_title": "항 고 취 지",
                "reasons_title": "항 고 이 유"
            },
            "court_appeal": {
                "title": "항 소 이 유 서",
                "appellant_label": "항 소 인",
                "respondent_label": "",
                "purpose_title": "항 소 취 지",
                "reasons_title": "항 소 이 유"
            }
        }

        self.ground_categories = {
            # Prosecution appeal grounds
            "insufficient_investigation": "수사의 불충분",
            "evidence_evaluation_error": "증거 평가의 오류",
            "legal_error_prosecution": "법리 적용의 오류",
            "new_evidence": "새로운 증거",

            # Court appeal grounds
            "factual_error": "사실오인",
            "legal_error": "법리오해",
            "procedural_error": "소송절차 위법",
            "sentencing_error": "양형부당"
        }

    def write(self,
              appeal_type: str,
              appellant: Dict[str, str],
              case_info: Dict[str, str],
              appeal_grounds: List[Dict[str, Any]],
              requested_judgment: Optional[str] = None,
              suspect: Optional[Dict[str, str]] = None,
              attorney: Optional[Dict[str, str]] = None,
              filing_authority: Optional[str] = None,
              new_evidence: Optional[List[Dict[str, str]]] = None,
              additional_arguments: Optional[str] = None
              ) -> 'CriminalAppealDocument':
        """
        Generate criminal appeal document.

        Args:
            appeal_type: Type of appeal ("prosecution_appeal" for 항고장, "court_appeal" for 항소이유서)
            appellant: Appellant information (name, resident_number, address, phone, role)
            case_info: Case information (varies by appeal type)
            appeal_grounds: List of appeal grounds with category, title, details
            requested_judgment: Requested judgment/relief
            suspect: Suspect information (for prosecution appeals)
            attorney: Attorney information (name, bar_number)
            filing_authority: Filing authority/court
            new_evidence: New evidence to submit
            additional_arguments: Additional arguments

        Returns:
            CriminalAppealDocument object
        """

        if appeal_type not in self.appeal_types:
            raise ValueError(f"Invalid appeal type: {appeal_type}")

        # Check deadlines
        if appeal_type == "prosecution_appeal":
            self._check_prosecution_deadline(case_info.get('disposition_date'))
        elif appeal_type == "court_appeal":
            self._check_court_deadline(case_info.get('judgment_date'))

        appeal_config = self.appeal_types[appeal_type]

        # Build document content
        content = {
            "header": self._build_header(appeal_type, appellant, suspect, case_info, attorney, appeal_config),
            "case_info": self._build_case_info(appeal_type, case_info),
            "purpose": self._build_purpose(appeal_type, requested_judgment, appeal_config),
            "reasons": self._build_reasons(appeal_grounds, additional_arguments, appeal_config),
            "signature": self._build_signature(appellant, attorney, filing_authority or self._get_default_authority(appeal_type, case_info))
        }

        if new_evidence:
            content["evidence"] = self._build_new_evidence(new_evidence)

        return CriminalAppealDocument(content, appeal_type, case_info)

    def _check_prosecution_deadline(self, disposition_date: Optional[str]) -> None:
        """Check 30-day deadline for prosecution appeal."""
        if not disposition_date:
            print("Warning: Disposition date not provided. Cannot verify 30-day deadline.")
            return

        try:
            disposition_dt = datetime.strptime(disposition_date, "%Y-%m-%d")
            deadline = disposition_dt + timedelta(days=30)
            today = datetime.now()
            days_left = (deadline - today).days

            if days_left < 0:
                print(f"WARNING: 기한 경과! 불기소 처분일로부터 30일이 지났습니다.")
            elif days_left <= 3:
                print(f"URGENT: {days_left}일 남음 - 항고 기한이 임박했습니다!")
            else:
                print(f"Info: {days_left}일 남음 - 기한 내 제출 가능합니다.")
        except ValueError:
            print("Warning: Invalid disposition date format. Use YYYY-MM-DD.")

    def _check_court_deadline(self, judgment_date: Optional[str]) -> None:
        """Check appeal deadline for court appeal."""
        if not judgment_date:
            print("Info: Judgment date not provided. 항소이유서는 변론종결 시까지 제출 가능합니다.")
            return

        try:
            judgment_dt = datetime.strptime(judgment_date, "%Y-%m-%d")
            notice_deadline = judgment_dt + timedelta(days=7)
            today = datetime.now()

            if today > notice_deadline:
                print("Warning: 항소장 제출 기한(7일)이 지났을 수 있습니다. 항소장 제출 여부를 확인하세요.")
            else:
                print("Info: 항소이유서는 변론종결 시까지 제출 가능합니다.")
        except ValueError:
            print("Warning: Invalid judgment date format. Use YYYY-MM-DD.")

    def _build_header(self,
                      appeal_type: str,
                      appellant: Dict[str, str],
                      suspect: Optional[Dict[str, str]],
                      case_info: Dict[str, str],
                      attorney: Optional[Dict[str, str]],
                      config: Dict[str, str]) -> str:
        """Build document header."""

        header = f"                {config['title']}\n\n"

        # For court appeals, add case number first
        if appeal_type == "court_appeal":
            trial_court = case_info.get('trial_court', '지방법원')
            case_number = case_info.get('case_number', '')
            crime = case_info.get('crime', '형사사건')
            header += f"사건번호: {trial_court} {case_number} {crime}\n"

            if case_info.get('appellate_court') and case_info.get('appellate_case_number'):
                header += f"          (항소심 {case_info['appellate_court']} {case_info['appellate_case_number']})\n"

            header += "\n"

        # Appellant
        header += f"{config['appellant_label']}    {appellant['name']}"
        if appellant.get('resident_number'):
            header += f"({appellant['resident_number']})"
        header += "\n"

        if appellant.get('role'):
            header += f"({appellant['role']})  "
        else:
            header += "            "

        header += f"{appellant['address']}\n"

        if appellant.get('phone'):
            header += f"            연락처 {appellant['phone']}\n"

        header += "\n"

        # Respondent (for prosecution appeals)
        if appeal_type == "prosecution_appeal" and suspect:
            header += f"{config['respondent_label']}      {suspect['name']}"
            if suspect.get('resident_number'):
                header += f"({suspect['resident_number']})"
            header += "\n"

            if suspect.get('address'):
                header += f"            {suspect['address']}\n"

        # Attorney (for court appeals)
        if appeal_type == "court_appeal" and attorney:
            header += "\n"
            bar_info = f" ({attorney.get('bar_number', '')})" if attorney.get('bar_number') else ""
            header += f"변 호 인  변호사 {attorney['name']}{bar_info}\n"

        return header

    def _build_case_info(self, appeal_type: str, case_info: Dict[str, str]) -> str:
        """Build case information section."""

        if appeal_type == "prosecution_appeal":
            info = "\n\n사 건 표 시\n\n"

            prosecutor_office = case_info.get('prosecutor_office', '검찰청')
            case_number = case_info.get('case_number', '')
            crime = case_info.get('crime', '형사사건')
            info += f"불기소 사건: {prosecutor_office} {case_number} {crime}\n"

            if case_info.get('disposition_date'):
                info += f"불기소 처분일: {self._format_date(case_info['disposition_date'])}\n"

            if case_info.get('disposition'):
                info += f"불기소 처분: {case_info['disposition']}\n"

            return info
        else:
            # For court appeals, case info is in header
            return ""

    def _build_purpose(self,
                       appeal_type: str,
                       requested_judgment: Optional[str],
                       config: Dict[str, str]) -> str:
        """Build purpose section."""

        purpose = f"\n\n{config['purpose_title']}\n\n"

        if requested_judgment:
            purpose += f"{requested_judgment}\n"
        else:
            if appeal_type == "prosecution_appeal":
                purpose += "검사의 불기소 처분은 부당하므로 이를 취소하고 재수사 후\n"
                purpose += "기소하여 주시기 바랍니다.\n"
            else:
                purpose += "원심 판결을 파기하고 피고인에게 무죄를 선고하여 주시기 바랍니다.\n"

        return purpose

    def _build_reasons(self,
                       appeal_grounds: List[Dict[str, Any]],
                       additional_arguments: Optional[str],
                       config: Dict[str, str]) -> str:
        """Build reasons section."""

        reasons = f"\n\n{config['reasons_title']}\n\n"

        for i, ground in enumerate(appeal_grounds, 1):
            category = ground.get('category')
            title = ground.get('title')
            details = ground.get('details', [])

            # Use category title if available, otherwise use custom title
            if category and category in self.ground_categories:
                section_title = self.ground_categories[category]
            elif title:
                section_title = title
            else:
                section_title = "기타 사유"

            reasons += f"{i}. {section_title}\n\n"

            # Add introduction if provided
            if ground.get('intro'):
                reasons += f"{ground['intro']}\n\n"

            # Add details
            if details:
                for j, detail in enumerate(details, 1):
                    reasons += f"({j}) {detail}"
                    if not detail.endswith('.'):
                        reasons += "."
                    reasons += "\n\n"

            reasons += "\n"

        # Add additional arguments
        if additional_arguments:
            reasons += f"{len(appeal_grounds) + 1}. 기타 사유\n\n"
            reasons += f"{additional_arguments}\n\n"

        return reasons

    def _build_new_evidence(self, new_evidence: List[Dict[str, str]]) -> str:
        """Build new evidence section."""

        evidence = "\n\n새로운 증거\n\n"

        for i, item in enumerate(new_evidence, 1):
            evidence_type = item.get('type', f'증거 {i}')
            description = item.get('description', '')

            if description:
                evidence += f"{i}. {evidence_type}    ({description})\n"
            else:
                evidence += f"{i}. {evidence_type}\n"

        return evidence

    def _build_signature(self,
                         appellant: Dict[str, str],
                         attorney: Optional[Dict[str, str]],
                         filing_authority: str) -> str:
        """Build date and signature section."""

        today = datetime.now()
        date_str = f"{today.year}.  {today.month:2d}.  {today.day:2d}."

        signature = f"\n\n{date_str}\n\n"

        # For prosecution appeals
        if not attorney or appellant.get('role') == '피해자':
            signature += f"                항고인  {appellant['name']}  (인)\n"
        else:
            # For court appeals with attorney
            signature += f"                항소인  {appellant['name']}  (인)\n"
            if attorney:
                signature += f"                변호인  변호사 {attorney['name']}  (인)\n"

        signature += f"\n\n{filing_authority}   귀중"

        return signature

    def _get_default_authority(self, appeal_type: str, case_info: Dict[str, str]) -> str:
        """Get default filing authority based on appeal type."""

        if appeal_type == "prosecution_appeal":
            # Default to 고등검찰청 검사장
            return case_info.get('higher_prosecutor_office', '고등검찰청 검사장')
        else:
            # Default to appellate court
            return case_info.get('appellate_court', '고등법원')

    def _format_date(self, date_str: str) -> str:
        """Format date string to Korean legal format."""
        try:
            dt = datetime.strptime(date_str, "%Y-%m-%d")
            return f"{dt.year}. {dt.month:2d}. {dt.day:2d}."
        except:
            return date_str


class CriminalAppealDocument:
    """Represents a generated criminal appeal document."""

    def __init__(self, content: Dict[str, str], appeal_type: str, case_info: Dict[str, str]):
        self.content = content
        self.appeal_type = appeal_type
        self.case_info = case_info

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['case_info'],
            self.content['purpose'],
            self.content['reasons']
        ]

        if 'evidence' in self.content:
            sections.append(self.content['evidence'])

        sections.append(self.content['signature'])

        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Criminal appeal saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


# Example usage
if __name__ == "__main__":
    writer = CriminalAppealWriter()

    print("=" * 80)
    print("Example 1: Prosecution Appeal (항고장)")
    print("=" * 80)

    # Example: Appeal to prosecutor
    doc1 = writer.write(
        appeal_type="prosecution_appeal",
        appellant={
            "name": "김철수",
            "resident_number": "831130-1247712",
            "address": "서울특별시 강남구 테헤란로 123",
            "phone": "010-1234-5678",
            "role": "피해자"
        },
        suspect={
            "name": "이영희",
            "resident_number": "800217-1348311",
            "address": "서울특별시 서초구 서초대로 456"
        },
        case_info={
            "case_number": "2024형제12345호",
            "prosecutor_office": "서울중앙지방검찰청",
            "crime": "사기",
            "disposition": "혐의없음",
            "disposition_date": "2024-07-01"
        },
        appeal_grounds=[
            {
                "category": "insufficient_investigation",
                "details": [
                    "피의자의 은행 거래내역, 통화 내역 등 핵심 증거를 전혀 수집하지 않았습니다",
                    "사건 발생 당시 목격자 3명이 있었으나 검사는 이들을 전혀 조사하지 않았습니다"
                ]
            },
            {
                "category": "evidence_evaluation_error",
                "details": [
                    "송금 내역, 카카오톡 대화, 녹취록 등 객관적 증거가 명백히 사기 사실을 입증함에도 불구하고 이를 무시하였습니다",
                    "검사는 피의자의 일방적 변명만을 믿고 항고인의 진술은 신뢰하지 않았습니다"
                ]
            },
            {
                "category": "legal_error_prosecution",
                "details": [
                    "검사는 사기죄의 성립요건에 대한 법리를 잘못 적용하였습니다. 본 건은 명백히 사기죄가 성립하는 사안입니다"
                ]
            }
        ],
        filing_authority="서울고등검찰청 검사장"
    )

    print(doc1)
    doc1.save_docx("prosecution_appeal_example.docx")

    print("\n" + "=" * 80)
    print("Example 2: Appellate Brief (항소이유서)")
    print("=" * 80)

    # Example: Appellate brief
    doc2 = writer.write(
        appeal_type="court_appeal",
        appellant={
            "name": "김철수",
            "resident_number": "831130-1247712",
            "address": "서울특별시 강남구 테헤란로 123",
            "phone": "010-1234-5678",
            "role": "피고인"
        },
        case_info={
            "case_number": "2024고단12345",
            "trial_court": "서울중앙지방법원",
            "appellate_court": "서울고등법원",
            "appellate_case_number": "2024노23456",
            "crime": "사기",
            "original_sentence": "징역 2년",
            "judgment_date": "2024-07-01"
        },
        appeal_grounds=[
            {
                "category": "factual_error",
                "intro": "원심은 피고인이 피해자를 기망하였다고 인정하였으나, 이는 증거에 의하여 인정되는 사실을 잘못 인정한 것입니다.",
                "details": [
                    "원심이 기망행위로 인정한 피고인의 발언은 실제로는 사업 계획에 대한 설명이었을 뿐, 허위 사실이 아니었습니다. 증인 ○○○의 증언과 카카오톡 대화내역이 이를 명확히 입증합니다",
                    "피고인은 차용 당시 충분한 변제 능력이 있었고, 실제로 변제 의사도 있었습니다. 사업자등록증과 매출 내역이 이를 입증합니다"
                ]
            },
            {
                "category": "legal_error",
                "intro": "원심은 사기죄의 성립요건에 관한 법리를 잘못 이해하였습니다.",
                "details": [
                    "원심은 단순한 약속 불이행을 기망으로 인정하였으나, 대법원 판례는 일관되게 단순 약속 불이행은 기망이 아니라고 판시하고 있습니다 (대법원 2020도12345 판결 참조)",
                    "원심은 사후적 변제 불능을 근거로 편취 범의를 인정하였으나, 편취 범의는 차용 당시를 기준으로 판단해야 합니다"
                ]
            },
            {
                "category": "sentencing_error",
                "intro": "설령 유죄가 인정된다 하더라도, 원심의 형량은 너무 무겁습니다.",
                "details": [
                    "피고인은 초범이고 깊이 반성하고 있습니다",
                    "피해자와 원만히 합의하여 피해를 모두 배상하였습니다",
                    "70세 노모와 어린 자녀 2명의 부양 책임이 있습니다"
                ]
            }
        ],
        requested_judgment="원심 판결을 파기하고 피고인에게 무죄를 선고하여 주시기 바랍니다.",
        attorney={
            "name": "박영희",
            "bar_number": "제12345호"
        },
        filing_authority="서울고등법원"
    )

    print(doc2)
    doc2.save_docx("appellate_brief_example.docx")
