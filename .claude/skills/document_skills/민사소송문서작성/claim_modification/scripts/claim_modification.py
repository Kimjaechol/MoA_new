"""
Claim Modification Request Writer - Korean Civil Litigation Claim Modification Generator
Generates 청구변경신청서 for modifying claims during Korean civil litigation.
"""

from datetime import datetime
from typing import Dict, List, Optional


class ClaimModificationWriter:
    """
    Generates Korean civil litigation claim modification request documents
    (청구변경신청서 또는 청구취지 및 청구원인 변경신청서).

    Supports various modification types:
    - Add preliminary claim (예비적 청구 추가)
    - Change claim amount (청구금액 변경)
    - Change legal basis (청구원인 변경)
    - Exchange claim type (청구 교환)
    - Add alternative claim (선택적 청구 추가)
    """

    MODIFICATION_TYPE_NAMES = {
        "add_preliminary": "예비적 청구 추가",
        "change_amount": "청구금액 변경",
        "change_grounds": "청구원인 변경",
        "exchange_claim": "청구 교환",
        "add_alternative": "선택적 청구 추가",
        "custom": "청구 변경"
    }

    def __init__(self):
        """Initialize ClaimModificationWriter."""
        self.template_cache = {}

    def write(
        self,
        case_number: str,
        case_name: str,
        plaintiff: Dict[str, str],
        defendant: Dict[str, str],
        attorney: Optional[Dict[str, str]] = None,
        modification_type: str = "custom",
        modified_claims: Dict[str, Dict[str, any]] = None,
        reason_for_modification: str = None,
        evidence: List[Dict[str, str]] = None,
        court: str = "서울중앙지방법원",
        division: Optional[str] = None,
        filing_date: Optional[datetime] = None
    ) -> 'ClaimModificationDocument':
        """
        Generate a claim modification request document.

        Args:
            case_number: Case number (e.g., "2024가합12345")
            case_name: Case name (e.g., "소유권이전등기")
            plaintiff: Plaintiff information
                - name: Party name
                - address: Full address
            defendant: Defendant information (same structure)
            attorney: Attorney information (optional)
                - name: Attorney name
                - firm: Law firm name
                - address: Office address
                - phone: Contact phone
                - fax: Fax number (optional)
                - email: Email address (optional)
            modification_type: Type of modification
            modified_claims: Modified claim structure
                For primary/preliminary:
                {
                    "primary": {
                        "purpose": [list of claim items],
                        "grounds": "claim grounds text"
                    },
                    "preliminary": {
                        "purpose": [list of claim items],
                        "grounds": "claim grounds text"
                    }
                }
                For simple modification:
                {
                    "primary": {
                        "purpose": [list of claim items],
                        "grounds": "claim grounds text"
                    }
                }
            reason_for_modification: Explanation for modification
            evidence: List of evidence
                - type: Evidence designation (e.g., "갑제4호증")
                - description: Evidence description
            court: Court name
            division: Court division (e.g., "제3민사부")
            filing_date: Date of filing (default: today)

        Returns:
            ClaimModificationDocument object
        """
        # Validate inputs
        self._validate_inputs(
            modification_type,
            modified_claims,
            reason_for_modification
        )

        # Set defaults
        if filing_date is None:
            filing_date = datetime.now()
        if evidence is None:
            evidence = []

        # Create document
        document = ClaimModificationDocument(
            case_number=case_number,
            case_name=case_name,
            plaintiff=plaintiff,
            defendant=defendant,
            attorney=attorney,
            modification_type=modification_type,
            modification_name=self.MODIFICATION_TYPE_NAMES.get(
                modification_type,
                "청구 변경"
            ),
            modified_claims=modified_claims,
            reason_for_modification=reason_for_modification,
            evidence=evidence,
            court=court,
            division=division,
            filing_date=filing_date
        )

        return document

    def _validate_inputs(
        self,
        modification_type: str,
        modified_claims: Optional[Dict[str, Dict[str, any]]],
        reason_for_modification: Optional[str]
    ) -> None:
        """Validate claim modification inputs."""
        if modification_type not in self.MODIFICATION_TYPE_NAMES:
            raise ValueError(
                f"Invalid modification_type: {modification_type}. "
                f"Must be one of: {list(self.MODIFICATION_TYPE_NAMES.keys())}"
            )

        if not modified_claims:
            raise ValueError("Modified claims must be provided")

        if "primary" not in modified_claims:
            raise ValueError("Modified claims must include 'primary' claim")

        primary = modified_claims["primary"]
        if "purpose" not in primary or not primary["purpose"]:
            raise ValueError("Primary claim must include 'purpose' list")
        if "grounds" not in primary or not primary["grounds"]:
            raise ValueError("Primary claim must include 'grounds' text")

        # Check preliminary claim structure if present
        if "preliminary" in modified_claims:
            preliminary = modified_claims["preliminary"]
            if "purpose" not in preliminary or not preliminary["purpose"]:
                raise ValueError("Preliminary claim must include 'purpose' list")
            if "grounds" not in preliminary or not preliminary["grounds"]:
                raise ValueError("Preliminary claim must include 'grounds' text")

        if not reason_for_modification:
            raise ValueError("Reason for modification must be provided")


class ClaimModificationDocument:
    """
    Represents a generated claim modification request document.
    """

    def __init__(
        self,
        case_number: str,
        case_name: str,
        plaintiff: Dict[str, str],
        defendant: Dict[str, str],
        attorney: Optional[Dict[str, str]],
        modification_type: str,
        modification_name: str,
        modified_claims: Dict[str, Dict[str, any]],
        reason_for_modification: str,
        evidence: List[Dict[str, str]],
        court: str,
        division: Optional[str],
        filing_date: datetime
    ):
        """Initialize ClaimModificationDocument."""
        self.case_number = case_number
        self.case_name = case_name
        self.plaintiff = plaintiff
        self.defendant = defendant
        self.attorney = attorney
        self.modification_type = modification_type
        self.modification_name = modification_name
        self.modified_claims = modified_claims
        self.reason_for_modification = reason_for_modification
        self.evidence = evidence
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
        lines.append("     청구취지 및 청구원인 변경(추가)신청서")
        lines.append("")
        lines.append(f"사건: {self.case_number} {self.case_name}")
        lines.append("")

        # Parties
        lines.append(f"원      고    {self.plaintiff['name']}")
        lines.append(f"              {self.plaintiff.get('address', '')}")
        lines.append("")

        lines.append(f"피      고    {self.defendant['name']}")
        lines.append(f"              {self.defendant.get('address', '')}")
        lines.append("")

        # Attorney (if present)
        if self.attorney:
            lines.append(f"원고 소송대리인 변호사    {self.attorney['name']}")
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
        lines.append("위 사건에 관하여 원고 소송대리인은 아래와 같이 청구취지 및 원인을 변경(추가)합니다.")
        lines.append("")

        # Modified claim purpose section
        lines.append("변경한 청구취지")
        lines.append("")

        # Primary claim
        if "primary" in self.modified_claims:
            if "preliminary" in self.modified_claims or "alternative" in self.modified_claims:
                lines.append("주위적으로")

            primary = self.modified_claims["primary"]
            for i, item in enumerate(primary["purpose"], 1):
                lines.append(f"{i}. {item}")
            lines.append("")
            lines.append("라는 판결을,")
            lines.append("")

        # Preliminary claim (if present)
        if "preliminary" in self.modified_claims:
            lines.append("예비적으로")
            preliminary = self.modified_claims["preliminary"]
            for i, item in enumerate(preliminary["purpose"], 1):
                lines.append(f"{i}. {item}")
            lines.append("")
            lines.append("라는 판결을 구합니다.")
            lines.append("")

        # Alternative claim (if present)
        elif "alternative" in self.modified_claims:
            lines.append("선택적으로")
            alternative = self.modified_claims["alternative"]
            for i, item in enumerate(alternative["purpose"], 1):
                lines.append(f"{i}. {item}")
            lines.append("")
            lines.append("라는 판결을 구합니다.")
            lines.append("")

        # If only primary claim
        elif "primary" in self.modified_claims:
            lines[-2] = "라는 판결을 구합니다."  # Replace last "라는 판결을,"
            lines.append("")

        # Modified claim grounds section
        lines.append("변경한 청구원인")
        lines.append("")

        # Primary claim grounds
        if "primary" in self.modified_claims:
            lines.append("1. 주위적 청구원인")
            lines.append("")
            for line in self.modified_claims["primary"]["grounds"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        # Preliminary claim grounds (if present)
        if "preliminary" in self.modified_claims:
            lines.append("2. 예비적 청구원인")
            lines.append("")
            for line in self.modified_claims["preliminary"]["grounds"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        # Alternative claim grounds (if present)
        elif "alternative" in self.modified_claims:
            lines.append("2. 선택적 청구원인")
            lines.append("")
            for line in self.modified_claims["alternative"]["grounds"].split("\n"):
                lines.append(f"  {line}")
            lines.append("")

        # Conclusion
        conclusion_num = 2 if "preliminary" in self.modified_claims or "alternative" in self.modified_claims else 2
        if "preliminary" in self.modified_claims or "alternative" in self.modified_claims:
            conclusion_num = 3

        lines.append(f"{conclusion_num}. 결론")
        lines.append("")
        lines.append(f"  {self.reason_for_modification}")
        lines.append("")

        # Evidence section
        if self.evidence:
            lines.append("증명방법")
            lines.append("")
            for i, ev in enumerate(self.evidence, 1):
                desc = ev.get("description", "")
                lines.append(f"{i}. {ev['type']:12s}  {desc}")
            lines.append("")

        # Attachments
        lines.append("첨부서류")
        lines.append("")
        attachment_num = 1
        if self.evidence:
            lines.append(f"{attachment_num}. 위 증명방법                           2통")
            attachment_num += 1

        lines.append(f"{attachment_num}. 청구취지 및 청구원인 변경신청서 부본    1통")
        lines.append("")

        # Date and signature
        date_str = self.filing_date.strftime("%Y. %m. %d.")
        lines.append(f"{date_str}")
        lines.append("")

        if self.attorney:
            lines.append("원고 소송대리인")
            lines.append(f"변호사    {self.attorney['name']}  (서명 또는 날인)")
        else:
            lines.append(f"원고    {self.plaintiff['name']}  (서명 또는 날인)")
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
    writer = ClaimModificationWriter()

    # Example: Add preliminary claim
    document = writer.write(
        case_number="2024가합12345",
        case_name="소유권이전등기",
        plaintiff={
            "name": "김을동",
            "address": "서울 강남구 테헤란로 123"
        },
        defendant={
            "name": "이경자",
            "address": "서울 서초구 서초대로 456"
        },
        attorney={
            "name": "연수희",
            "firm": "법무법인 정의",
            "address": "서울 강남구 논현로 456",
            "phone": "02-1234-5678"
        },
        modification_type="add_preliminary",
        modified_claims={
            "primary": {
                "purpose": [
                    "피고는 원고로부터 1억 원을 지급받음과 동시에 원고에게 별지 목록 기재 부동산에 관하여 2024. 4. 24. 매매를 원인으로 한 소유권이전등기절차를 이행하라.",
                    "소송비용은 피고가 부담한다."
                ],
                "grounds": "원고는 2024. 4. 24. 피고로부터 경기 포천군 일동면 길명리 120-1 잡종지 12,358㎡를 대금 2억 원에 매수하며, 같은 날 계약금으로 2,000만 원을 지급하고, 중도금 8,000만 원은 같은 달 30.에, 나머지 잔금 1억 원은 같은 해 5. 31.까지 위 토지에 설정된 근저당권설정등기를 말소한 소유권이전등기에 필요한 서류의 교부와 상환으로 각 지급하기로 약정하였고, 2024. 4. 30. 위 중도금을 지급하였습니다.\n\n따라서 피고는 원고로부터 위 잔금을 지급받음과 동시에 원고에게 위 부동산에 관하여 위 매매를 원인으로 한 소유권이전등기절차를 이행할 의무가 있습니다."
            },
            "preliminary": {
                "purpose": [
                    "피고는 원고에게 1억 2,000만 원 및 그 중 1억 원에 대하여는 2024. 4. 30.부터 이 사건 청구취지 및 청구원인 변경신청서 부본 송달일까지 연 5%의, 1억 2,000만 원에 대하여는 그 다음날부터 다 갚는 날까지 연 15%의 각 비율에 의한 금원을 지급하라.",
                    "소송비용은 피고가 부담한다.",
                    "제1항은 가집행할 수 있다."
                ],
                "grounds": "그럼에도 피고는 원고의 소유권이전등기청구에 응하지 않으면서 오히려 위 매매계약 당시 계약금의 배액을 배상함으로써 동 계약을 해제할 수 있기로 약정하였으므로 동 매매계약을 해제하였다고 주장합니다.\n\n만약 피고의 위 주장이 인정된다면, 피고는 피고가 이미 지급받은 위 매매 대금을 부당이득으로 반환하고 아울러 위 약정에 따른 배상을 하여야 할 것인바, 그렇다면 피고는 원고에게 원상회복금 1억 원과 약정 배상금 2,000만 원을 합한 1억 2,000만 원 및 그에 대한 지연손해금을 지급할 의무가 있습니다."
            }
        },
        reason_for_modification="그러므로 원고는 종래의 소유권이전등기청구를 주위적으로 구하고, 위 부당이득금 등 반환청구를 예비적으로 구하는 것으로 청구취지 및 청구원인을 변경(추가)합니다.",
        evidence=[
            {"type": "갑제4호증", "description": "통지서"},
            {"type": "갑제5호증", "description": "계약서 사본"}
        ],
        court="서울중앙지방법원",
        division="제3민사부"
    )

    print(document.to_text())
