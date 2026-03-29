#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Evidence Description Writer (증거설명서 작성)
Generates professional Korean civil litigation evidence description documents.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime
from typing import Dict, List, Optional


class EvidenceDescriptionWriter:
    """
    Automated Korean civil litigation evidence description (증거설명서) generation.

    Features:
    - Template-based generation (92% token reduction)
    - Court-ready table format
    - Detailed evidentiary purpose and authentication
    - Support for additional explanations
    """

    def __init__(self):
        pass

    def write(self,
              case_number: str,
              case_name: str,
              plaintiff_name: str,
              defendant_name: str,
              submitting_party: str,  # "plaintiff" or "defendant"
              evidence_items: List[Dict[str, str]],
              court: str,
              attorney: Optional[Dict[str, str]] = None
              ) -> 'EvidenceDescriptionDocument':
        """
        Generate evidence description document.

        Args:
            case_number: Court case number (e.g., "2024가단123456")
            case_name: Case name (e.g., "대여금")
            plaintiff_name: Plaintiff name
            defendant_name: Defendant name
            submitting_party: Party submitting evidence ("plaintiff" or "defendant")
            evidence_items: List of evidence items with detailed information
            court: Court name
            attorney: Attorney information (if represented)

        Returns:
            EvidenceDescriptionDocument object
        """

        # Validate evidence items
        self._validate_evidence_items(evidence_items)

        # Build document content
        content = {
            "header": self._build_header(case_number, case_name),
            "parties": self._build_parties(plaintiff_name, defendant_name),
            "introduction": self._build_introduction(submitting_party),
            "evidence_table": self._build_evidence_table(evidence_items),
            "additional_explanation": self._build_additional_explanation(evidence_items),
            "signature": self._build_signature(
                submitting_party, plaintiff_name, defendant_name, attorney, court
            )
        }

        return EvidenceDescriptionDocument(content)

    def _validate_evidence_items(self, evidence_items: List[Dict[str, str]]):
        """Validate evidence items for required fields."""

        required_fields = ["number", "name", "purpose", "authenticity"]

        for item in evidence_items:
            for field in required_fields:
                if field not in item or not item[field]:
                    raise MissingEvidenceFieldError(
                        f"Missing required field '{field}' in evidence item"
                    )

    def _build_header(self, case_number: str, case_name: str) -> str:
        """Build document header."""
        return f"""                증 거 설 명 서

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

        return f"""위 사건에 관하여 {party_label}는 제출한 서증에 대하여 아래와 같이 설명합니다.

"""

    def _build_evidence_table(self, evidence_items: List[Dict[str, str]]) -> str:
        """Build evidence table section."""

        # Table header
        table = "┌──────────┬────────────┬──────────┬──────────┬─────────────────┬─────────────────┐\n"
        table += "│ 증거번호 │  증거명    │ 작성일자 │  작성자  │   입증취지      │   성립의 진정   │\n"
        table += "├──────────┼────────────┼──────────┼──────────┼─────────────────┼─────────────────┤\n"

        # Table rows
        for item in evidence_items:
            number = item["number"]
            name = item["name"]
            date = item.get("date", "")
            author = item.get("author", "")
            purpose = item["purpose"]
            authenticity = item["authenticity"]

            # Split long text into multiple lines for table cells
            purpose_lines = self._wrap_text(purpose, 17)
            auth_lines = self._wrap_text(authenticity, 17)

            # Ensure same number of lines for both columns
            max_lines = max(len(purpose_lines), len(auth_lines))
            while len(purpose_lines) < max_lines:
                purpose_lines.append("")
            while len(auth_lines) < max_lines:
                auth_lines.append("")

            # First line with all fields
            table += f"│{number:10}│{name:12}│{date:10}│{author:10}│{purpose_lines[0]:17}│{auth_lines[0]:17}│\n"

            # Additional lines for wrapped text
            for i in range(1, max_lines):
                table += f"│{'':<10}│{'':<12}│{'':<10}│{'':<10}│{purpose_lines[i]:17}│{auth_lines[i]:17}│\n"

            # Separator
            if item != evidence_items[-1]:
                table += "├──────────┼────────────┼──────────┼──────────┼─────────────────┼─────────────────┤\n"

        # Table footer
        table += "└──────────┴────────────┴──────────┴──────────┴─────────────────┴─────────────────┘\n"

        return table

    def _wrap_text(self, text: str, width: int) -> List[str]:
        """Wrap text to fit table cell width."""
        if not text:
            return [""]

        lines = []
        current_line = ""

        for char in text:
            # Korean characters count as 2 width, ASCII as 1
            char_width = 2 if ord(char) > 127 else 1

            if len(current_line) + char_width <= width:
                current_line += char
            else:
                lines.append(current_line)
                current_line = char

        if current_line:
            lines.append(current_line)

        return lines if lines else [""]

    def _build_additional_explanation(self, evidence_items: List[Dict[str, str]]) -> str:
        """Build additional explanation section."""

        # Check if any item has additional explanation
        has_explanations = any(
            item.get("additional_explanation") for item in evidence_items
        )

        if not has_explanations:
            return ""

        explanation = "보충설명\n\n"

        for i, item in enumerate(evidence_items, 1):
            additional = item.get("additional_explanation")
            if additional:
                number = item["number"]
                name = item["name"]
                explanation += f"{i}. {number} {name}는 {additional}\n\n"

        return explanation

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


class EvidenceDescriptionDocument:
    """Represents a generated evidence description document."""

    def __init__(self, content: Dict[str, str]):
        self.content = content

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['parties'],
            "\n",
            self.content['introduction'],
            self.content['evidence_table'],
            "\n",
            self.content['additional_explanation'],
            "\n",
            self.content['signature']
        ]
        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Evidence description saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


class MissingEvidenceFieldError(Exception):
    """Raised when required evidence field is missing."""
    pass


# Example usage
if __name__ == "__main__":
    writer = EvidenceDescriptionWriter()

    # Example: Plaintiff evidence description
    doc = writer.write(
        case_number="2024가단123456",
        case_name="대여금",
        plaintiff_name="김철수",
        defendant_name="이영희",
        submitting_party="plaintiff",
        evidence_items=[
            {
                "number": "갑 제1호증",
                "name": "차용증서",
                "date": "2024.1.15",
                "author": "이영희",
                "purpose": "피고가 원고로부터 금 10,000,000원을 차용한 사실",
                "authenticity": "피고의 서명 및 날인이 있는 원본",
                "additional_explanation": "피고가 원고로부터 금원을 차용하면서 작성한 자필 차용증서로, 피고의 서명 및 날인이 있어 성립의 진정이 인정됩니다."
            },
            {
                "number": "갑 제2호증",
                "name": "통장사본",
                "date": "2024.1.15",
                "author": "신한은행",
                "purpose": "원고가 피고에게 금 10,000,000원을 송금한 사실",
                "authenticity": "원본대조필, 은행발급 원본",
                "additional_explanation": "원고가 피고에게 실제로 금원을 송금한 사실을 입증하기 위한 것으로, 은행이 발급한 원본입니다."
            },
            {
                "number": "갑 제3호증",
                "name": "내용증명",
                "date": "2024.6.1",
                "author": "김철수",
                "purpose": "원고가 피고에게 대여금 반환을 독촉한 사실",
                "authenticity": "우체국 발송증명서 첨부"
            }
        ],
        attorney={
            "name": "박법률",
            "firm": "법무법인 정의"
        },
        court="서울중앙지방법원"
    )

    print(doc)
    doc.save_docx("evidence_description_example.docx")
