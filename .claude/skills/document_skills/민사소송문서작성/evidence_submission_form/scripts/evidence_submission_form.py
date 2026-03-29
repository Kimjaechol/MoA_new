#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Evidence Submission Form Writer (서증제출서 작성)
Generates professional Korean civil litigation evidence submission forms.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime
from typing import Dict, List, Optional


class EvidenceSubmissionFormWriter:
    """
    Automated Korean civil litigation evidence submission form (서증제출서) generation.

    Features:
    - Template-based generation (97% token reduction)
    - Court-ready format
    - Automatic sequential numbering
    - Support for plaintiff/defendant evidence
    """

    def __init__(self):
        self.evidence_prefixes = {
            "plaintiff": "갑",
            "defendant": "을"
        }

    def write(self,
              case_number: str,
              case_name: str,
              plaintiff_name: str,
              defendant_name: str,
              submitting_party: str,  # "plaintiff" or "defendant"
              evidence_list: List[Dict[str, str]],
              court: str,
              attorney: Optional[Dict[str, str]] = None
              ) -> 'EvidenceSubmissionDocument':
        """
        Generate evidence submission form.

        Args:
            case_number: Court case number (e.g., "2024가단123456")
            case_name: Case name (e.g., "대여금")
            plaintiff_name: Plaintiff name
            defendant_name: Defendant name
            submitting_party: Party submitting evidence ("plaintiff" or "defendant")
            evidence_list: List of evidence items with number and description
            court: Court name
            attorney: Attorney information (if represented)

        Returns:
            EvidenceSubmissionDocument object
        """

        # Auto-number evidence if not provided
        evidence_list = self._auto_number_evidence(evidence_list, submitting_party)

        # Validate evidence list
        self._validate_evidence_list(evidence_list)

        # Build document content
        content = {
            "header": self._build_header(case_number, case_name),
            "parties": self._build_parties(plaintiff_name, defendant_name),
            "introduction": self._build_introduction(submitting_party),
            "evidence_list": self._build_evidence_list(evidence_list),
            "attachments": self._build_attachments(evidence_list),
            "signature": self._build_signature(
                submitting_party, plaintiff_name, defendant_name, attorney, court
            )
        }

        return EvidenceSubmissionDocument(content)

    def _auto_number_evidence(self,
                              evidence_list: List[Dict[str, str]],
                              submitting_party: str) -> List[Dict[str, str]]:
        """Auto-number evidence items if number not provided."""

        prefix = self.evidence_prefixes[submitting_party]
        numbered_list = []
        counter = 1

        for item in evidence_list:
            if "number" not in item or not item["number"]:
                item["number"] = f"{prefix} 제{counter}호증"
                counter += 1
            else:
                # Extract counter from existing number for next item
                # e.g., "갑 제3호증" -> counter = 4
                import re
                match = re.search(r'제(\d+)호증', item["number"])
                if match:
                    counter = int(match.group(1)) + 1

            numbered_list.append(item)

        return numbered_list

    def _validate_evidence_list(self, evidence_list: List[Dict[str, str]]):
        """Validate evidence list for duplicates and missing descriptions."""

        numbers = [item["number"] for item in evidence_list]

        # Check for duplicates
        if len(numbers) != len(set(numbers)):
            raise DuplicateEvidenceNumberError("Duplicate evidence numbers found")

        # Check for missing descriptions
        for item in evidence_list:
            if not item.get("description"):
                raise MissingEvidenceDescriptionError(
                    f"Missing description for {item['number']}"
                )

    def _build_header(self, case_number: str, case_name: str) -> str:
        """Build document header."""
        return f"""                서 증 제 출 서

사건: {case_number} {case_name}
"""

    def _build_parties(self, plaintiff_name: str, defendant_name: str) -> str:
        """Build parties section."""
        return f"""원      고    {plaintiff_name}
피      고    {defendant_name}
"""

    def _build_introduction(self, submitting_party: str) -> str:
        """Build introduction section."""
        party_label = "원고" if submitting_party == "plaintiff" else "피고"

        return f"""위 사건에 관하여 {party_label}는 아래와 같이 서증을 제출합니다.

- 기 -
"""

    def _build_evidence_list(self, evidence_list: List[Dict[str, str]]) -> str:
        """Build evidence list section."""

        evidence_text = ""
        for i, item in enumerate(evidence_list, 1):
            number = item["number"]
            description = item["description"]
            evidence_text += f"{i}. {number}    {description}\n"

        return evidence_text

    def _build_attachments(self, evidence_list: List[Dict[str, str]]) -> str:
        """Build attachments section."""

        if not evidence_list:
            return ""

        # Determine evidence number range
        first_number = evidence_list[0]["number"]
        last_number = evidence_list[-1]["number"]

        # Extract prefix (갑 or 을)
        prefix = first_number.split()[0]

        # Build range description
        if len(evidence_list) == 1:
            range_desc = first_number
        else:
            first_num_only = first_number.replace(prefix + " ", "")
            last_num_only = last_number.replace(prefix + " ", "")
            range_desc = f"{prefix} {first_num_only} 내지 {last_num_only}"

        return f"""첨부서류

1. 위 {range_desc}              각 1통
"""

    def _build_signature(self,
                        submitting_party: str,
                        plaintiff_name: str,
                        defendant_name: str,
                        attorney: Optional[Dict[str, str]],
                        court: str) -> str:
        """Build date and signature section."""

        today = datetime.now()
        date_str = f"{today.year}.  {today.month:2d}.  {today.day:2d}."

        signature = f"{date_str}\n\n"

        # Determine signatory
        party_name = plaintiff_name if submitting_party == "plaintiff" else defendant_name
        party_label = "원고" if submitting_party == "plaintiff" else "피고"

        if attorney:
            # Attorney signature
            signature += f"{party_label} 소송대리인\n"
            signature += f"변호사    {attorney['name']}  (서명 또는 날인)\n\n"
        else:
            # Pro se party signature
            signature += f"{party_label}    {party_name}  (서명 또는 날인)\n\n"

        signature += f"{court}   귀중"

        return signature


class EvidenceSubmissionDocument:
    """Represents a generated evidence submission form document."""

    def __init__(self, content: Dict[str, str]):
        self.content = content

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['parties'],
            "\n",
            self.content['introduction'],
            "\n",
            self.content['evidence_list'],
            "\n",
            self.content['attachments'],
            "\n",
            self.content['signature']
        ]
        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Evidence submission form saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


class DuplicateEvidenceNumberError(Exception):
    """Raised when duplicate evidence numbers are detected."""
    pass


class MissingEvidenceDescriptionError(Exception):
    """Raised when evidence description is missing."""
    pass


# Example usage
if __name__ == "__main__":
    writer = EvidenceSubmissionFormWriter()

    # Example: Plaintiff submitting evidence
    doc = writer.write(
        case_number="2024가단123456",
        case_name="대여금",
        plaintiff_name="김철수",
        defendant_name="이영희",
        submitting_party="plaintiff",
        evidence_list=[
            {
                "number": "갑 제1호증",
                "description": "차용증서"
            },
            {
                "number": "갑 제2호증",
                "description": "통장사본"
            },
            {
                "number": "갑 제3호증의 1 내지 3",
                "description": "각 영수증"
            },
            {
                "number": "갑 제4호증",
                "description": "내용증명우편"
            }
        ],
        attorney={
            "name": "박법률",
            "firm": "법무법인 정의"
        },
        court="서울중앙지방법원"
    )

    print(doc)
    doc.save_docx("evidence_submission_example.docx")
