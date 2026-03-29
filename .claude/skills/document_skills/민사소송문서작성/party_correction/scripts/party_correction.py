"""
Party Correction Request Writer - Korean Civil Litigation Party Correction Generator
Generates 당사자표시정정신청서 for correcting party information in Korean civil litigation.
"""

from datetime import datetime
from typing import Dict, List, Optional


class PartyCorrectionWriter:
    """
    Generates Korean civil litigation party correction request documents (당사자표시정정신청서).

    Supports various correction types:
    - Deceased to heirs (사망자 → 상속인)
    - Address correction (주소 정정)
    - Name correction (성명 정정)
    - Registration number correction (주민등록번호 정정)
    """

    CORRECTION_TYPE_NAMES = {
        "deceased_to_heirs": "사망자를 상속인으로 정정",
        "address_correction": "주소 정정",
        "name_correction": "성명 정정",
        "registration_correction": "주민등록번호 정정",
        "custom": "당사자 표시정정"
    }

    def __init__(self):
        """Initialize PartyCorrectionWriter."""
        self.template_cache = {}

    def write(
        self,
        case_number: str,
        case_name: str,
        applicant: Dict[str, str],
        attorney: Optional[Dict[str, str]] = None,
        correction_type: str = "custom",
        current_party: Dict[str, str] = None,
        corrected_parties: List[Dict[str, str]] = None,
        reason: Dict[str, str] = None,
        supporting_documents: List[Dict[str, str]] = None,
        court: str = "서울중앙지방법원",
        division: Optional[str] = None,
        filing_date: Optional[datetime] = None
    ) -> 'PartyCorrectionDocument':
        """
        Generate a party correction request document.

        Args:
            case_number: Case number (e.g., "2024가합1234")
            case_name: Case name (e.g., "소유권이전등기")
            applicant: Applicant information
                - role: "원고" or "피고"
                - name: Party name
                - address: Full address
            attorney: Attorney information (optional)
                - name: Attorney name
                - firm: Law firm name
                - address: Office address
                - phone: Contact phone
                - fax: Fax number (optional)
                - email: Email address (optional)
            correction_type: Type of correction
            current_party: Current (incorrect) party information
                - role: "원고" or "피고"
                - name: Current name
                - registration_number: Current registration number (optional)
                - address: Current address (optional)
            corrected_parties: List of corrected party information
                - name: Corrected name
                - registration_number: Registration number (optional)
                - address: Corrected address
                - relationship: Relationship to original party (for heirs)
            reason: Reason for correction
                - type: "deceased", "clerical_error", etc.
                - death_date: Date of death (for deceased cases)
                - explanation: Detailed explanation
            supporting_documents: List of supporting documents
            court: Court name
            division: Court division (e.g., "제5민사부")
            filing_date: Date of filing (default: today)

        Returns:
            PartyCorrectionDocument object
        """
        # Validate inputs
        self._validate_inputs(
            correction_type,
            current_party,
            corrected_parties,
            reason
        )

        # Set defaults
        if filing_date is None:
            filing_date = datetime.now()
        if supporting_documents is None:
            supporting_documents = []

        # Create document
        document = PartyCorrectionDocument(
            case_number=case_number,
            case_name=case_name,
            applicant=applicant,
            attorney=attorney,
            correction_type=correction_type,
            correction_name=self.CORRECTION_TYPE_NAMES.get(
                correction_type,
                "당사자 표시정정"
            ),
            current_party=current_party,
            corrected_parties=corrected_parties,
            reason=reason,
            supporting_documents=supporting_documents,
            court=court,
            division=division,
            filing_date=filing_date
        )

        return document

    def _validate_inputs(
        self,
        correction_type: str,
        current_party: Optional[Dict[str, str]],
        corrected_parties: Optional[List[Dict[str, str]]],
        reason: Optional[Dict[str, str]]
    ) -> None:
        """Validate correction request inputs."""
        if correction_type not in self.CORRECTION_TYPE_NAMES:
            raise ValueError(
                f"Invalid correction_type: {correction_type}. "
                f"Must be one of: {list(self.CORRECTION_TYPE_NAMES.keys())}"
            )

        if not current_party:
            raise ValueError("Current party information must be provided")

        if not corrected_parties:
            raise ValueError("Corrected party information must be provided")

        if not reason:
            raise ValueError("Reason for correction must be provided")

        # Check required reason fields
        if "explanation" not in reason:
            raise ValueError("Reason must include 'explanation' field")


class PartyCorrectionDocument:
    """
    Represents a generated party correction request document.
    """

    def __init__(
        self,
        case_number: str,
        case_name: str,
        applicant: Dict[str, str],
        attorney: Optional[Dict[str, str]],
        correction_type: str,
        correction_name: str,
        current_party: Dict[str, str],
        corrected_parties: List[Dict[str, str]],
        reason: Dict[str, str],
        supporting_documents: List[Dict[str, str]],
        court: str,
        division: Optional[str],
        filing_date: datetime
    ):
        """Initialize PartyCorrectionDocument."""
        self.case_number = case_number
        self.case_name = case_name
        self.applicant = applicant
        self.attorney = attorney
        self.correction_type = correction_type
        self.correction_name = correction_name
        self.current_party = current_party
        self.corrected_parties = corrected_parties
        self.reason = reason
        self.supporting_documents = supporting_documents
        self.court = court
        self.division = division
        self.filing_date = filing_date

    def to_text(self) -> str:
        """
        Convert document to plain text format.

        Returns:
            Formatted text document
        """
        lines = []

        # Header
        lines.append("")
        lines.append("         당사자 표시정정 신청서")
        lines.append("")
        lines.append(f"사건: {self.case_number} {self.case_name}")
        lines.append("")

        # Parties
        lines.append(f"{self.applicant['role']:8s}  {self.applicant['name']}")
        lines.append(f"              {self.applicant.get('address', '')}")
        lines.append("")

        lines.append(f"{self.current_party['role']:8s}  {self.current_party['name']}")
        if "address" in self.current_party:
            lines.append(f"              {self.current_party['address']}")
        lines.append("")

        # Attorney (if present)
        if self.attorney:
            lines.append(f"{self.applicant['role']} 소송대리인 변호사    {self.attorney['name']}")
            lines.append(f"              {self.attorney['address']}")
            if "firm" in self.attorney:
                lines.append(f"              {self.attorney['firm']}")
            if "phone" in self.attorney:
                lines.append(f"              전화: {self.attorney['phone']}")
            if "fax" in self.attorney:
                lines.append(f"              팩스: {self.attorney['fax']}")
            if "email" in self.attorney:
                lines.append(f"              이메일: {self.attorney['email']}")
            lines.append("")

        # Statement of intent
        lines.append(f"위 사건에 관하여 {self.applicant['role']} 소송대리인은 당사자({self.current_party['role']}) 표시정정을 신청합니다.")
        lines.append("")

        # Purpose section
        lines.append("신청취지")
        lines.append("")

        # Build current party description
        current_desc = f"{self.current_party['role']} {self.current_party['name']}"
        if "registration_number" in self.current_party:
            current_desc += f" ({self.current_party['registration_number']})"
        if "address" in self.current_party:
            current_desc += f" {self.current_party['address']}"

        lines.append(f'위 사건에 관하여 "{current_desc}"을')
        lines.append("별지 명부 기재와 같이 정정한다.")
        lines.append("")
        lines.append("라는 재판을 구합니다.")
        lines.append("")

        # Reason section
        lines.append("신청원인")
        lines.append("")

        explanation = self.reason.get("explanation", "")
        for line in explanation.split("\n"):
            lines.append(line)
        lines.append("")

        # List of corrected parties (명부)
        if len(self.corrected_parties) > 1 or self.correction_type == "deceased_to_heirs":
            lines.append("명 부")
            lines.append("")

            for i, party in enumerate(self.corrected_parties, 1):
                party_line = f"{self.current_party['role']} {i}. {party['name']}"
                if "registration_number" in party:
                    party_line += f" ({party['registration_number']})"
                lines.append(party_line)

                if "address" in party:
                    lines.append(f"        {party['address']}")
                lines.append("")

            lines.append("                                        끝.")
            lines.append("")

        # Attachments
        if self.supporting_documents:
            lines.append("첨부서류")
            for i, doc in enumerate(self.supporting_documents, 1):
                doc_type = doc.get("type", "")
                count = doc.get("count", 1)
                lines.append(f"{i}. {doc_type:24s}  {count}통")

            # Always add copy of application
            lines.append(f"{len(self.supporting_documents) + 1}. 신청서 부본                  {len(self.corrected_parties)}통")
            lines.append("")

        # Date and signature
        date_str = self.filing_date.strftime("%Y. %m. %d.")
        lines.append(f"{date_str}")
        lines.append("")

        if self.attorney:
            lines.append(f"{self.applicant['role']} 소송대리인")
            lines.append(f"변호사    {self.attorney['name']}  (서명 또는 날인)")
        else:
            lines.append(f"{self.applicant['role']}    {self.applicant['name']}  (서명 또는 날인)")
        lines.append("")

        court_line = self.court
        if self.division:
            court_line += f" {self.division}"
        court_line += "   귀중"
        lines.append(court_line)
        lines.append("")

        return "\n".join(lines)

    def save_text(self, filepath: str) -> None:
        """Save document as plain text file."""
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(self.to_text())

    def save_docx(self, filepath: str) -> None:
        """
        Save document as DOCX file.
        Uses docx skill for professional formatting.
        """
        # TODO: Integrate with docx skill
        # For now, save as text
        self.save_text(filepath.replace(".docx", ".txt"))

    def save_pdf(self, filepath: str) -> None:
        """
        Save document as PDF file.
        Uses pdf skill for professional formatting.
        """
        # TODO: Integrate with pdf skill
        # For now, save as text
        self.save_text(filepath.replace(".pdf", ".txt"))


# Example usage
if __name__ == "__main__":
    writer = PartyCorrectionWriter()

    # Example: Deceased party to heirs correction
    document = writer.write(
        case_number="2024가합1234",
        case_name="소유권이전등기",
        applicant={
            "role": "원고",
            "name": "김선학",
            "address": "서울 강남구 테헤란로 123"
        },
        attorney={
            "name": "연수희",
            "firm": "법무법인 정의",
            "address": "서울 강남구 논현로 456",
            "phone": "02-1234-5678"
        },
        correction_type="deceased_to_heirs",
        current_party={
            "role": "피고",
            "name": "김춘수",
            "registration_number": "641230-1023576",
            "address": "서울 중구 서소문동 123"
        },
        corrected_parties=[
            {
                "name": "최영순",
                "registration_number": "642017-2215361",
                "address": "서울 마포구 난지도길 123",
                "relationship": "배우자"
            },
            {
                "name": "김병학",
                "registration_number": "881108-1023546",
                "address": "서울 서대문구 수색로74길 34",
                "relationship": "자"
            },
            {
                "name": "김병순",
                "registration_number": "900205-2038362",
                "address": "서울 구로구 경인로25길 33",
                "relationship": "자"
            }
        ],
        reason={
            "type": "deceased",
            "death_date": "2024-05-10",
            "explanation": "피고 김춘수는 이 사건 소 제기 전인 2024. 5. 10.에 이미 사망하였으나 사망신고가 되어 있지 않은 관계로 원고는 이를 모르고 피고를 김춘수로 표시하였는바, 이는 명백한 잘못이므로 신청취지와 같이 그 상속인들로 표시를 정정합니다."
        },
        supporting_documents=[
            {"type": "제적 등본", "count": 1},
            {"type": "가족관계증명서", "count": 3},
            {"type": "기본증명서", "count": 1},
            {"type": "친양자입양관계증명서", "count": 1}
        ],
        court="서울중앙지방법원",
        division="제5민사부"
    )

    print(document.to_text())
