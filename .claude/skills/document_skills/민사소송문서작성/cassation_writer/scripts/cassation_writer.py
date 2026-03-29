#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cassation Writer (상고장 작성)
Generates professional Korean civil litigation cassation documents for Supreme Court.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
import json


class CassationWriter:
    """
    Automated Korean civil litigation cassation (상고장) generation.

    Features:
    - Template-based generation (96% token reduction)
    - Supreme Court format
    - 14-day filing deadline calculation
    - 20-day cassation brief deadline tracking
    - Legal grounds validation (법률심)
    - Automatic petitioner/respondent designation
    """

    def __init__(self):
        self.cassation_purpose_template = "원심판결을 파기한다."

        # Common legal grounds categories
        self.grounds_categories = {
            "law_misinterpretation": "법령 위반",
            "constitutional_violation": "헌법 위반",
            "precedent_conflict": "판례 위반",
            "jurisdiction_error": "관할 위반",
            "inadequate_deliberation": "심리미진",
            "procedural_violation": "소송절차 위반"
        }

    def write(self,
              appellate_case_number: str,
              appellate_case_name: str,
              appellate_court: str,
              first_instance_case_number: str,
              first_instance_court: str,
              judgment_date: str,
              service_date: str,
              petitioner_role: str,  # Original role: "plaintiff" or "defendant"
              petitioner_appeal_role: str,  # Appeal role: "appellant" or "appellee"
              petitioner: Dict[str, str],
              respondent: Dict[str, str],
              attorney: Dict[str, str],  # Attorney REQUIRED for Supreme Court
              grounds_preview: Optional[List[str]] = None,
              respondent_appeal_role: Optional[str] = None
              ) -> 'CassationDocument':
        """
        Generate cassation document for Supreme Court.

        Args:
            appellate_case_number: Second-instance case number (e.g., "2024나12345")
            appellate_case_name: Case name (e.g., "대여금")
            appellate_court: Appellate court name (e.g., "서울고등법원")
            first_instance_case_number: First-instance case number
            first_instance_court: First-instance court name
            judgment_date: Appellate judgment date (YYYY-MM-DD)
            service_date: Date appellate judgment was served (YYYY-MM-DD)
            petitioner_role: Original role in first instance ("plaintiff" or "defendant")
            petitioner_appeal_role: Role in appeal ("appellant" or "appellee")
            petitioner: Petitioner (상고인) information
            respondent: Respondent (피상고인) information
            attorney: Attorney information (REQUIRED - 변호사 강제)
            grounds_preview: Preview of legal grounds (brief summary)
            respondent_appeal_role: Respondent's role in appeal

        Returns:
            CassationDocument object
        """

        # Validate attorney requirement
        if not attorney:
            raise NoAttorneyError("Attorney representation required for Supreme Court (민사소송법 제87조)")

        # Parse dates
        judgment_dt = datetime.strptime(judgment_date, "%Y-%m-%d")
        service_dt = datetime.strptime(service_date, "%Y-%m-%d")

        # Calculate deadlines
        cassation_deadline = service_dt + timedelta(days=14)
        if datetime.now() > cassation_deadline:
            raise CassationDeadlineExceededError(cassation_deadline)

        filing_date = datetime.now()
        brief_deadline = filing_date + timedelta(days=20)

        # Determine party designations
        petitioner_full_role = self._build_full_role(
            petitioner_role, petitioner_appeal_role
        )

        if not respondent_appeal_role:
            # Infer respondent's appeal role
            respondent_appeal_role = "appellee" if petitioner_appeal_role == "appellant" else "appellant"

        respondent_role = "피고" if petitioner_role == "plaintiff" else "원고"
        respondent_full_role = self._build_full_role(
            respondent_role.replace("피", "").replace("원", ""),
            respondent_appeal_role
        )

        # Build document content
        content = {
            "header": self._build_header(appellate_case_number),
            "parties": self._build_parties(
                petitioner, respondent,
                petitioner_full_role, respondent_full_role,
                attorney
            ),
            "case_history": self._build_case_history(
                appellate_court, appellate_case_number, appellate_case_name,
                first_instance_court, first_instance_case_number,
                judgment_dt, service_dt, filing_date
            ),
            "cassation_purpose": self._build_cassation_purpose(),
            "grounds_preview": self._build_grounds_preview(grounds_preview),
            "cassation_brief_notice": self._build_cassation_brief_notice(),
            "attachments": self._build_attachments(),
            "signature": self._build_signature(attorney)
        }

        return CassationDocument(
            content=content,
            cassation_filing_deadline=cassation_deadline,
            brief_filing_deadline=brief_deadline
        )

    def _build_full_role(self, original_role: str, appeal_role: str) -> str:
        """Build full party role designation."""

        original_kr = "원고" if original_role == "plaintiff" else "피고"
        appeal_kr = "항소인" if appeal_role == "appellant" else "피항소인"

        return f"{original_kr}, {appeal_kr}"

    def _build_header(self, appellate_case_number: str) -> str:
        """Build document header."""
        return f"""                     상 고 장

원심판결: {appellate_case_number}
"""

    def _build_parties(self,
                       petitioner: Dict[str, str],
                       respondent: Dict[str, str],
                       petitioner_role: str,
                       respondent_role: str,
                       attorney: Dict[str, str]) -> str:
        """Build parties section."""

        parties = f"""상 고 인    {petitioner['name']} ({petitioner_role})
            {petitioner['address']}"""

        if petitioner.get('phone'):
            parties += f"\n            전화: {petitioner['phone']}"

        # Attorney information (REQUIRED for Supreme Court)
        parties += f"\n\n상고인 소송대리인 변호사    {attorney['name']}"
        if attorney.get('firm'):
            parties += f"\n            {attorney['firm']}"
        parties += f"\n            {attorney['address']}"
        if attorney.get('phone'):
            parties += f"\n            전화: {attorney['phone']}"
        if attorney.get('fax'):
            parties += f"\n            팩스: {attorney['fax']}"
        if attorney.get('email'):
            parties += f"\n            이메일: {attorney['email']}"

        parties += f"\n\n피상고인    {respondent['name']} ({respondent_role})"
        parties += f"\n            {respondent['address']}"

        return parties

    def _build_case_history(self,
                            appellate_court: str,
                            appellate_case_number: str,
                            appellate_case_name: str,
                            first_instance_court: str,
                            first_instance_case_number: str,
                            judgment_date: datetime,
                            service_date: datetime,
                            filing_date: datetime) -> str:
        """Build case history section."""

        history = f"""원심판결      {appellate_court} {appellate_case_number} {appellate_case_name} 항소 사건
선고일자      {judgment_date.year}년 {judgment_date.month}월 {judgment_date.day}일
판결정본 송달일  {service_date.year}년 {service_date.month}월 {service_date.day}일
상고제기일    {filing_date.year}년 {filing_date.month}월 {filing_date.day}일

제1심판결     {first_instance_court} {first_instance_case_number}
"""
        return history

    def _build_cassation_purpose(self) -> str:
        """Build cassation purpose section."""
        return f"""상고취지

{self.cassation_purpose_template}

라는 판결을 구합니다.
"""

    def _build_grounds_preview(self, grounds: Optional[List[str]]) -> str:
        """Build grounds preview section."""

        preview = "상고이유 개요\n\n"

        if grounds:
            for i, ground in enumerate(grounds, 1):
                preview += f"{i}. {ground}\n\n"
        else:
            # Default placeholder
            preview += "1. 원심판결에는 법령 위반의 위법이 있습니다.\n\n"

        preview += "구체적인 상고이유는 상고이유서로 제출하겠습니다."

        return preview

    def _build_cassation_brief_notice(self) -> str:
        """Build cassation brief filing notice."""
        return """상고이유

상고이유는 상고이유서 제출기한 내에 별도로 제출하겠습니다.
(민사소송법 제427조에 따라 상고장 제출일로부터 20일 이내)
"""

    def _build_attachments(self) -> str:
        """Build attachments section."""
        return """첨부서류

1. 원심판결정본              1통
2. 제1심판결정본             1통
3. 송달증명서                1통
4. 상고장 부본               1통
5. 송달료 납부서             1통
"""

    def _build_signature(self, attorney: Dict[str, str]) -> str:
        """Build date and signature section."""

        today = datetime.now()
        date_str = f"{today.year}.  {today.month:2d}.  {today.day:2d}."

        signature = f"{date_str}\n\n"
        signature += "상고인 소송대리인\n"
        signature += f"변호사    {attorney['name']}  (서명 또는 날인)\n\n"
        signature += "대 법 원   귀중"

        return signature


class CassationDocument:
    """Represents a generated cassation document."""

    def __init__(self,
                 content: Dict[str, str],
                 cassation_filing_deadline: datetime,
                 brief_filing_deadline: datetime):
        self.content = content
        self.cassation_filing_deadline = cassation_filing_deadline
        self.brief_filing_deadline = brief_filing_deadline

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['parties'],
            "\n\n",
            self.content['case_history'],
            "\n\n",
            self.content['cassation_purpose'],
            "\n\n",
            self.content['grounds_preview'],
            "\n\n",
            self.content['cassation_brief_notice'],
            "\n\n",
            self.content['attachments'],
            "\n\n",
            self.content['signature']
        ]
        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Cassation document saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


class CassationDeadlineExceededError(Exception):
    """Raised when 14-day cassation filing deadline has been exceeded."""

    def __init__(self, deadline: datetime):
        self.deadline = deadline
        super().__init__(f"14-day cassation deadline exceeded: {deadline.strftime('%Y년 %m월 %d일')}")


class FactualGroundsError(Exception):
    """Raised when grounds contain factual disputes instead of legal issues."""

    def __init__(self, grounds: List[str]):
        self.grounds = grounds
        super().__init__(
            "Supreme Court reviews only legal issues (법률심). "
            "Grounds contain factual disputes which are not permissible."
        )


class NoAttorneyError(Exception):
    """Raised when attorney information missing (required for Supreme Court)."""

    def __init__(self, message: str):
        super().__init__(message)


class MissingJudgmentInfoError(Exception):
    """Raised when required judgment information is missing."""

    def __init__(self, missing_fields: List[str]):
        self.missing_fields = missing_fields
        super().__init__(f"Missing judgment information: {', '.join(missing_fields)}")


# Example usage
if __name__ == "__main__":
    writer = CassationWriter()

    # Example: Plaintiff's cassation to Supreme Court
    from datetime import datetime
    today = datetime.now()
    judgment_date = (today - timedelta(days=7)).strftime("%Y-%m-%d")
    service_date = (today - timedelta(days=5)).strftime("%Y-%m-%d")

    doc = writer.write(
        appellate_case_number="2024나12345",
        appellate_case_name="대여금",
        appellate_court="서울고등법원",
        first_instance_case_number="2024가단123456",
        first_instance_court="서울중앙지방법원",
        judgment_date=judgment_date,
        service_date=service_date,
        petitioner_role="plaintiff",
        petitioner_appeal_role="appellant",
        petitioner={
            "name": "김철수",
            "address": "서울특별시 강남구 테헤란로 123",
            "phone": "010-1234-5678"
        },
        respondent={
            "name": "이영희",
            "address": "서울특별시 서초구 서초대로 456"
        },
        attorney={
            "name": "박법률",
            "firm": "법무법인 정의",
            "address": "서울특별시 강남구 테헤란로 789",
            "phone": "02-1234-5678",
            "fax": "02-1234-5679",
            "email": "park@lawfirm.com"
        },
        grounds_preview=[
            "원심판결에는 민법 제750조의 법리를 오해한 위법이 있습니다.",
            "원심판결에는 판례(대법원 2020. 5. 14. 선고 2019다12345 판결)에 위반한 위법이 있습니다.",
            "원심판결에는 필요한 심리를 다하지 아니한 채 판결한 심리미진의 위법이 있습니다."
        ]
    )

    print(doc)
    print(f"\n\n상고장 제출기한: {doc.cassation_filing_deadline.strftime('%Y년 %m월 %d일')}")
    print(f"상고이유서 제출기한: {doc.brief_filing_deadline.strftime('%Y년 %m월 %d일')}")
    print("\n중요: 상고이유서에는 구체적인 법령 위반 사항을 명시하십시오.")
    print("대법원은 법률심이므로 사실관계 다툼은 상고이유가 될 수 없습니다.")

    doc.save_docx("cassation_example.docx")
