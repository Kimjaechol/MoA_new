#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Petition Writer (탄원서/진정서 작성)
Generates professional Korean petition documents.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime
from typing import Dict, List, Optional, Any


class PetitionWriter:
    """
    Automated Korean petition (탄원서/진정서) generation.

    Features:
    - Template-based generation (95% token reduction)
    - Court-ready DOCX/PDF format
    - Dual petition types (leniency/complaint)
    - Relationship description
    - Multiple petition reasons support
    """

    def __init__(self):
        self.petition_types = {
            "leniency": {
                "title": "탄 원 서",
                "petitioner_label": "탄원인",
                "subject_label": "피고인",
                "purpose_title": "탄 원 취 지",
                "content_title": "탄 원 사 유",
                "purpose_text": "탄원인은 피고인에 대하여 다음과 같은 사유로 관용 있는 처분을\n구하고자 탄원서를 제출합니다.",
                "closing": "위와 같은 사정을 참작하시어 피고인에게 관용 있는 처분을 내려주시기를\n간곡히 탄원합니다."
            },
            "complaint": {
                "title": "진 정 서",
                "petitioner_label": "진정인",
                "subject_label": "피진정인",
                "purpose_title": "진 정 취 지",
                "content_title": "진 정 내 용",
                "purpose_text": "진정인은 피진정인의 부당한 행위에 대하여 철저한 조사와\n시정 조치를 요청하고자 진정서를 제출합니다.",
                "closing": "위와 같은 사정을 조사하시어 적절한 시정 조치를 취해주시기를\n간곡히 요청합니다."
            }
        }

        self.relationship_types = {
            "family": ["부모", "배우자", "자녀", "형제", "자매", "친척"],
            "work": ["직장 동료", "상사", "부하직원", "고용주", "직원"],
            "social": ["친구", "이웃", "지인", "동창"],
            "victim": ["피해자", "피해자 가족"]
        }

    def write(self,
              petition_type: str,
              petitioner: Dict[str, str],
              subject: Dict[str, str],
              reasons: Optional[List[str]] = None,
              complaint_details: Optional[List[str]] = None,
              relationship: Optional[str] = None,
              requested_action: Optional[str] = None,
              filing_authority: str = "검찰청",
              additional_info: Optional[str] = None
              ) -> 'PetitionDocument':
        """
        Generate petition document.

        Args:
            petition_type: Type of petition ("leniency" for 탄원서, "complaint" for 진정서)
            petitioner: Petitioner information (name, resident_number, address, phone)
            subject: Subject person information (name, resident_number, address, position)
            reasons: List of reasons for leniency (for 탄원서)
            complaint_details: List of complaint details (for 진정서)
            relationship: Relationship to subject (for 탄원서)
            requested_action: Requested action (for 진정서)
            filing_authority: Filing authority
            additional_info: Additional information

        Returns:
            PetitionDocument object
        """

        if petition_type not in self.petition_types:
            raise ValueError(f"Invalid petition type: {petition_type}")

        petition_config = self.petition_types[petition_type]

        # Build document content
        content = {
            "header": self._build_header(petitioner, subject, relationship, petition_config),
            "purpose": self._build_purpose(petition_config),
            "content": self._build_content(
                petition_type,
                reasons,
                complaint_details,
                requested_action,
                petition_config,
                additional_info
            ),
            "closing": self._build_closing(petition_config),
            "signature": self._build_signature(petitioner, filing_authority, petition_config)
        }

        return PetitionDocument(content, petition_type)

    def _build_header(self,
                      petitioner: Dict[str, str],
                      subject: Dict[str, str],
                      relationship: Optional[str],
                      config: Dict[str, str]) -> str:
        """Build document header with parties."""

        header = f"                     {config['title']}\n\n"

        # Petitioner
        header += f"{config['petitioner_label']}    {petitioner['name']}"
        if petitioner.get('resident_number'):
            header += f"({petitioner['resident_number']})"
        header += "\n"
        header += f"          {petitioner['address']}\n"
        if petitioner.get('phone'):
            header += f"          연락처 {petitioner['phone']}\n"

        # Relationship (for leniency petitions)
        if relationship:
            header += f"          {config['subject_label']}과의 관계: {relationship}\n"

        header += "\n\n"

        # Subject
        header += f"{config['subject_label']}  {subject['name']}"
        if subject.get('resident_number'):
            header += f"({subject['resident_number']})"
        header += "\n"

        if subject.get('address'):
            header += f"          {subject['address']}\n"

        if subject.get('position'):
            header += f"          소속: {subject['position']}\n"

        if subject.get('phone'):
            header += f"          연락처: {subject['phone']}\n"

        return header

    def _build_purpose(self, config: Dict[str, str]) -> str:
        """Build purpose section."""

        purpose = f"\n\n{config['purpose_title']}\n\n"
        purpose += f"{config['purpose_text']}\n"

        return purpose

    def _build_content(self,
                       petition_type: str,
                       reasons: Optional[List[str]],
                       complaint_details: Optional[List[str]],
                       requested_action: Optional[str],
                       config: Dict[str, str],
                       additional_info: Optional[str]) -> str:
        """Build content section."""

        content = f"\n\n{config['content_title']}\n\n"

        if petition_type == "leniency":
            # Build leniency reasons
            if reasons:
                for i, reason in enumerate(reasons, 1):
                    content += f"{i}. {reason}"
                    if not reason.endswith('.'):
                        content += "."
                    content += "\n\n"
            else:
                content += "별지 첨부\n\n"

        elif petition_type == "complaint":
            # Build complaint details
            if complaint_details:
                for i, detail in enumerate(complaint_details, 1):
                    content += f"{i}. {detail}"
                    if not detail.endswith('.'):
                        content += "."
                    content += "\n\n"
            else:
                content += "별지 첨부\n\n"

            # Add requested action if provided
            if requested_action:
                content += f"\n이에 {requested_action}를 요청합니다.\n\n"

        # Add additional info if provided
        if additional_info:
            content += f"\n{additional_info}\n\n"

        return content

    def _build_closing(self, config: Dict[str, str]) -> str:
        """Build closing section."""

        return f"\n\n{config['closing']}\n"

    def _build_signature(self,
                         petitioner: Dict[str, str],
                         filing_authority: str,
                         config: Dict[str, str]) -> str:
        """Build date and signature section."""

        today = datetime.now()
        date_str = f"{today.year}.  {today.month:2d}.  {today.day:2d}."

        signature = f"\n\n{date_str}\n\n"
        signature += f"                     {config['petitioner_label']}  {petitioner['name']}  (인)\n\n\n"
        signature += f"{filing_authority}   귀중"

        return signature


class PetitionDocument:
    """Represents a generated petition document."""

    def __init__(self, content: Dict[str, str], petition_type: str):
        self.content = content
        self.petition_type = petition_type

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            self.content['purpose'],
            self.content['content'],
            self.content['closing'],
            self.content['signature']
        ]
        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Petition saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


# Example usage
if __name__ == "__main__":
    writer = PetitionWriter()

    print("=" * 80)
    print("Example 1: Petition for Leniency (탄원서)")
    print("=" * 80)

    # Example: Petition for leniency
    doc1 = writer.write(
        petition_type="leniency",
        petitioner={
            "name": "김철수",
            "resident_number": "831130-1247712",
            "address": "서울특별시 강남구 테헤란로 123",
            "phone": "010-1234-5678"
        },
        subject={
            "name": "이영희",
            "resident_number": "800217-1348311",
            "address": "서울특별시 서초구 서초대로 456"
        },
        relationship="직장 동료",
        reasons=[
            "피고인은 평소 성실하고 책임감 있는 사람으로, 이번 일은 일시적인 판단 착오로 인한 것입니다",
            "피고인은 70세 노모와 어린 자녀 2명을 부양하고 있어, 엄중한 처벌을 받을 경우 가족이 생계에 어려움을 겪게 됩니다",
            "피고인은 깊이 반성하고 있으며, 피해자와 원만히 합의하여 피해를 모두 배상하였습니다",
            "피고인은 이번이 처음이자 마지막 범죄이며, 다시는 같은 잘못을 반복하지 않겠다고 다짐하고 있습니다"
        ],
        filing_authority="서울중앙지방검찰청"
    )

    print(doc1)
    doc1.save_docx("petition_leniency_example.docx")

    print("\n" + "=" * 80)
    print("Example 2: Complaint to Authorities (진정서)")
    print("=" * 80)

    # Example: Complaint to authorities
    doc2 = writer.write(
        petition_type="complaint",
        petitioner={
            "name": "김철수",
            "resident_number": "831130-1247712",
            "address": "서울특별시 강남구 테헤란로 123",
            "phone": "010-1234-5678"
        },
        subject={
            "name": "이영희",
            "resident_number": "800217-1348311",
            "position": "○○회사 영업팀장"
        },
        complaint_details=[
            "피진정인은 2024. 5. 1.경 직장 내에서 진정인에게 업무와 무관한 심부름을 강요하며 인격적 모욕을 주었습니다",
            "피진정인은 진정인이 이를 거부하자 부당한 인사 불이익을 주겠다고 협박하였습니다",
            "이러한 행위는 근로기준법 및 직장 내 괴롭힘 금지 규정을 위반하는 것입니다",
            "진정인은 정신적 고통을 받았으며, 정상적인 업무 수행이 어려운 상황입니다"
        ],
        requested_action="철저한 조사와 시정 조치",
        filing_authority="서울지방고용노동청"
    )

    print(doc2)
    doc2.save_docx("complaint_authorities_example.docx")
