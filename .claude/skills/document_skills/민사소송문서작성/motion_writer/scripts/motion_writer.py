"""
Motion Writer - Korean Civil Litigation Motion/Application Generator
Generates 신청서 for various procedural requests in Korean civil litigation.
"""

from datetime import datetime
from typing import Dict, List, Optional


class MotionWriter:
    """
    Generates Korean civil litigation motion/application documents (신청서).

    Supports various motion types:
    - Trial date designation (변론기일지정신청)
    - Trial reopening (변론재개신청)
    - Evidence preservation (증거보전신청)
    - Extension of time (기간연장신청)
    - Custom motions
    """

    MOTION_TYPE_TITLES = {
        "trial_date_designation": "변론기일지정 신청서",
        "trial_reopening": "변론재개 신청서",
        "evidence_preservation": "증거보전 신청서",
        "extension_of_time": "기간연장 신청서",
        "document_production": "문서제출명령 신청서",
        "custom": "신청서"
    }

    def __init__(self):
        """Initialize MotionWriter."""
        self.template_cache = {}

    def write(
        self,
        case_number: str,
        case_name: str,
        applicant: Dict[str, str],
        respondent: Dict[str, str],
        attorney: Optional[Dict[str, str]] = None,
        motion_type: str = "custom",
        purpose: List[str] = None,
        reasons: Dict[str, str] = None,
        attachments: List[Dict[str, str]] = None,
        court: str = "서울중앙지방법원",
        filing_date: Optional[datetime] = None
    ) -> 'MotionDocument':
        """
        Generate a motion/application document.

        Args:
            case_number: Case number (e.g., "2024가단123456")
            case_name: Case name (e.g., "대여금")
            applicant: Applicant information
                - role: "원고" or "피고"
                - name: Party name
                - address: Full address
            respondent: Respondent information (same structure as applicant)
            attorney: Attorney information (optional)
                - name: Attorney name
                - firm: Law firm name
                - address: Office address
                - phone: Contact phone
                - fax: Fax number (optional)
                - email: Email address (optional)
            motion_type: Type of motion (see MOTION_TYPE_TITLES)
            purpose: List of requested relief items (신청취지)
            reasons: Reasons for motion
                - facts: Factual background
                - necessity: Why motion is necessary
                - legal_basis: Legal foundation (optional)
                - conclusion: Concluding statement
            attachments: List of attached documents
            court: Court name
            filing_date: Date of filing (default: today)

        Returns:
            MotionDocument object
        """
        # Validate inputs
        self._validate_inputs(motion_type, purpose, reasons)

        # Set defaults
        if filing_date is None:
            filing_date = datetime.now()
        if purpose is None:
            purpose = []
        if reasons is None:
            reasons = {}
        if attachments is None:
            attachments = []

        # Create document
        document = MotionDocument(
            case_number=case_number,
            case_name=case_name,
            applicant=applicant,
            respondent=respondent,
            attorney=attorney,
            motion_type=motion_type,
            motion_title=self.MOTION_TYPE_TITLES.get(motion_type, "신청서"),
            purpose=purpose,
            reasons=reasons,
            attachments=attachments,
            court=court,
            filing_date=filing_date
        )

        return document

    def _validate_inputs(
        self,
        motion_type: str,
        purpose: Optional[List[str]],
        reasons: Optional[Dict[str, str]]
    ) -> None:
        """Validate motion inputs."""
        if motion_type not in self.MOTION_TYPE_TITLES and motion_type != "custom":
            raise ValueError(
                f"Invalid motion_type: {motion_type}. "
                f"Must be one of: {list(self.MOTION_TYPE_TITLES.keys())}"
            )

        if not purpose:
            raise ValueError("Purpose (신청취지) must be provided")

        if not reasons:
            raise ValueError("Reasons (신청이유) must be provided")

        # Check required reason fields
        if "facts" not in reasons:
            raise ValueError("Reasons must include 'facts' field")
        if "necessity" not in reasons:
            raise ValueError("Reasons must include 'necessity' field")


class MotionDocument:
    """
    Represents a generated motion/application document.
    """

    def __init__(
        self,
        case_number: str,
        case_name: str,
        applicant: Dict[str, str],
        respondent: Dict[str, str],
        attorney: Optional[Dict[str, str]],
        motion_type: str,
        motion_title: str,
        purpose: List[str],
        reasons: Dict[str, str],
        attachments: List[Dict[str, str]],
        court: str,
        filing_date: datetime
    ):
        """Initialize MotionDocument."""
        self.case_number = case_number
        self.case_name = case_name
        self.applicant = applicant
        self.respondent = respondent
        self.attorney = attorney
        self.motion_type = motion_type
        self.motion_title = motion_title
        self.purpose = purpose
        self.reasons = reasons
        self.attachments = attachments
        self.court = court
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
        lines.append(f"                     {self.motion_title}")
        lines.append("")
        lines.append(f"사건: {self.case_number} {self.case_name}")
        lines.append("")

        # Parties
        lines.append(f"{self.applicant['role']:8s}  {self.applicant['name']}")
        lines.append(f"              {self.applicant['address']}")
        lines.append("")

        lines.append(f"{self.respondent['role']:8s}  {self.respondent['name']}")
        lines.append(f"              {self.respondent['address']}")
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

        # Purpose section
        lines.append(f"위 사건에 관하여 {self.applicant['role']} 소송대리인은 다음과 같은 재판을 구합니다.")
        lines.append("")
        lines.append("신청취지")
        for i, item in enumerate(self.purpose, 1):
            lines.append(f"{i}. {item}")
        lines.append("")
        lines.append("라는 재판을 구합니다.")
        lines.append("")

        # Reasons section
        lines.append("신청이유")
        lines.append("")

        if "facts" in self.reasons:
            lines.append("1. 사실관계")
            lines.append("")
            for line in self.reasons["facts"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        if "necessity" in self.reasons:
            lines.append("2. 신청의 필요성")
            lines.append("")
            for line in self.reasons["necessity"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        if "legal_basis" in self.reasons:
            lines.append("3. 법적 근거")
            lines.append("")
            for line in self.reasons["legal_basis"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        if "conclusion" in self.reasons:
            lines.append("4. 결론")
            lines.append("")
            lines.append(f"  {self.reasons['conclusion']}")
            lines.append("")

        # Attachments
        if self.attachments:
            lines.append("첨부서류")
            for i, att in enumerate(self.attachments, 1):
                desc = att.get("description", "")
                count = att.get("count", "1통")
                if desc:
                    lines.append(f"{i}. {att['type']:20s}  {desc}")
                else:
                    lines.append(f"{i}. {att['type']:20s}  {count}")
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

        lines.append(f"{self.court}   귀중")
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
    writer = MotionWriter()

    # Example: Trial date designation motion
    document = writer.write(
        case_number="2024가단123456",
        case_name="대여금",
        applicant={
            "role": "원고",
            "name": "김철수",
            "address": "서울특별시 강남구 테헤란로 123"
        },
        respondent={
            "role": "피고",
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
        motion_type="trial_date_designation",
        purpose=[
            "이 사건의 변론기일을 지정한다."
        ],
        reasons={
            "facts": "2024. 6. 15.자 변론기일에 원고와 피고 쌍방이 불출석하여 소 취하간주 결정이 있었으나, 원고는 부득이한 사정으로 불출석하였습니다.",
            "necessity": "원고는 본 소송을 계속 진행할 의사가 있으므로, 민사소송법 제268조에 따라 1개월 내에 변론기일 지정을 신청합니다.",
            "conclusion": "그러므로 신청취지와 같은 재판을 구합니다."
        },
        attachments=[
            {"type": "소명자료", "description": "불출석 사유서"},
            {"type": "신청서 부본", "description": "1통"}
        ],
        court="서울중앙지방법원"
    )

    print(document.to_text())
