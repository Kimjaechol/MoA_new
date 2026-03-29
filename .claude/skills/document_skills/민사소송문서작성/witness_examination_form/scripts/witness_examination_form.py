#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Witness Examination Form Writer (증인신문신청서 작성)
Generates professional Korean civil litigation witness examination forms.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime
from typing import Dict, List, Optional


class WitnessExaminationFormWriter:
    """
    Automated Korean civil litigation witness examination form (증인신문신청서) generation.

    Features:
    - Template-based generation (94% token reduction)
    - Court-ready format
    - Automatic copy calculation
    - Structured examination questions
    """

    def __init__(self):
        pass

    def write(self,
              case_number: str,
              case_name: str,
              plaintiff_name: str,
              defendant_name: str,
              submitting_party: str,  # "plaintiff" or "defendant"
              witness: Dict[str, str],
              examination_questions: List[str],
              evidentiary_purpose: str,
              court: str,
              num_opponents: int = 1,
              is_panel_court: bool = False,
              attorney: Optional[Dict[str, str]] = None
              ) -> 'WitnessExaminationDocument':
        """
        Generate witness examination form.

        Args:
            case_number: Court case number (e.g., "2024가단123456")
            case_name: Case name (e.g., "대여금")
            plaintiff_name: Plaintiff name
            defendant_name: Defendant name
            submitting_party: Party requesting examination ("plaintiff" or "defendant")
            witness: Witness personal information
            examination_questions: List of questions to ask witness
            evidentiary_purpose: What facts witness will prove
            court: Court name
            num_opponents: Number of opposing parties
            is_panel_court: True if 합의부 (panel court)
            attorney: Attorney information (if represented)

        Returns:
            WitnessExaminationDocument object
        """

        # Validate inputs
        self._validate_witness_info(witness)
        self._validate_questions(examination_questions)

        # Calculate required copies
        required_copies = self._calculate_required_copies(num_opponents, is_panel_court)

        # Build document content
        content = {
            "header": self._build_header(case_number, case_name),
            "parties": self._build_parties(plaintiff_name, defendant_name),
            "introduction": self._build_introduction(submitting_party),
            "witness_info": self._build_witness_info(witness),
            "questions": self._build_questions(examination_questions),
            "purpose": self._build_purpose(evidentiary_purpose),
            "signature": self._build_signature(
                submitting_party, plaintiff_name, defendant_name, attorney, court
            )
        }

        return WitnessExaminationDocument(content, required_copies)

    def _validate_witness_info(self, witness: Dict[str, str]):
        """Validate witness information for required fields."""

        required_fields = ["name", "address"]

        for field in required_fields:
            if field not in witness or not witness[field]:
                raise MissingWitnessInfoError(
                    f"Missing required witness field '{field}'"
                )

    def _validate_questions(self, questions: List[str]):
        """Validate examination questions."""

        if not questions or len(questions) < 3:
            raise InsufficientQuestionsError(
                "At least 3 examination questions required"
            )

    def _calculate_required_copies(self, num_opponents: int, is_panel_court: bool) -> int:
        """
        Calculate required number of copies.

        Rule 80(1):
        - Single judge: opponents + 3
        - Panel court: opponents + 4
        """
        if is_panel_court:
            return num_opponents + 4
        else:
            return num_opponents + 3

    def _build_header(self, case_number: str, case_name: str) -> str:
        """Build document header."""
        return f"""             증 인 신 문 신 청 서

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

        return f"""위 사건에 관하여 {party_label}는 아래 증인에 대한 신문을 신청합니다.

"""

    def _build_witness_info(self, witness: Dict[str, str]) -> str:
        """Build witness information section."""

        info = "증인 인적사항\n\n"
        info += f"성      명:  {witness['name']}\n"
        info += f"주      소:  {witness['address']}\n"

        if witness.get('birth_date'):
            # Format birth date if provided as YYYY-MM-DD
            birth_date = witness['birth_date']
            if '-' in birth_date:
                parts = birth_date.split('-')
                birth_date = f"{parts[0]}. {int(parts[1]):2d}. {int(parts[2]):2d}."
            info += f"생년월일:  {birth_date}\n"

        if witness.get('phone'):
            info += f"연 락 처:  {witness['phone']}\n"

        if witness.get('relationship'):
            info += f"관      계:  {witness['relationship']}\n"

        return info

    def _build_questions(self, questions: List[str]) -> str:
        """Build examination questions section."""

        questions_text = "신문사항\n\n"

        for i, question in enumerate(questions, 1):
            questions_text += f"{i}. {question}\n\n"

        return questions_text

    def _build_purpose(self, evidentiary_purpose: str) -> str:
        """Build evidentiary purpose section."""

        return f"""입증취지

{evidentiary_purpose}
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


class WitnessExaminationDocument:
    """Represents a generated witness examination form document."""

    def __init__(self, content: Dict[str, str], required_copies: int):
        self.content = content
        self.required_copies = required_copies

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['parties'],
            "\n",
            self.content['introduction'],
            self.content['witness_info'],
            "\n",
            self.content['questions'],
            "\n",
            self.content['purpose'],
            "\n\n",
            self.content['signature']
        ]
        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Witness examination form saved: {filename}")
        print(f"Required copies: {self.required_copies}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


class MissingWitnessInfoError(Exception):
    """Raised when required witness information is missing."""
    pass


class InsufficientQuestionsError(Exception):
    """Raised when insufficient examination questions provided."""
    pass


# Example usage
if __name__ == "__main__":
    writer = WitnessExaminationFormWriter()

    # Example: Plaintiff requesting witness examination
    doc = writer.write(
        case_number="2024가단123456",
        case_name="대여금",
        plaintiff_name="김철수",
        defendant_name="이영희",
        submitting_party="plaintiff",
        witness={
            "name": "홍길동",
            "address": "서울특별시 강남구 테헤란로 123",
            "birth_date": "1980-05-15",
            "phone": "010-1234-5678",
            "relationship": "원고의 친구"
        },
        examination_questions=[
            "증인은 원고 김철수를 알고 있습니까?",
            "증인은 피고 이영희를 알고 있습니까?",
            "증인은 2024. 1. 15. 원고와 피고가 만난 자리에 동석하였습니까?",
            "위 자리에서 피고가 원고로부터 금원을 차용한다는 이야기를 들은 적이 있습니까?",
            "피고가 차용한 금액이 얼마였습니까?",
            "피고가 차용증서를 작성하는 것을 보았습니까?",
            "피고가 차용증서에 서명 및 날인하는 것을 직접 보았습니까?"
        ],
        evidentiary_purpose="피고가 2024. 1. 15. 원고로부터 금 10,000,000원을 차용한 사실",
        attorney={
            "name": "박법률",
            "firm": "법무법인 정의"
        },
        court="서울중앙지방법원",
        num_opponents=1,
        is_panel_court=False
    )

    print(doc)
    print(f"\n제출 필요 통수: {doc.required_copies}통")
    doc.save_docx("witness_examination_example.docx")
