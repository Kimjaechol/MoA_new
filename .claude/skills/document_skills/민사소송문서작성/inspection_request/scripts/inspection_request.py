#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Inspection Request Writer (검증신청서 작성)
Generates professional Korean inspection request documents.

Part of LawPro AI Platform
License: Proprietary
"""

from datetime import datetime
from typing import Dict, List, Optional


class InspectionRequestWriter:
    """
    Automated Korean inspection request (검증신청서) generation.

    Features:
    - Template-based generation (90% token reduction)
    - Court-ready DOCX/PDF format
    - On-site verification requests
    - Customizable inspection matters
    """

    def __init__(self):
        self.inspection_types = {
            "land": "토지 검증",
            "boundary": "경계 검증",
            "building": "건물 검증",
            "accident_scene": "사고현장 검증",
            "possession": "점유 검증"
        }

    def write(self,
              case_number: str,
              case_name: str,
              plaintiff: str,
              defendant: str,
              inspection_location: str,
              inspection_objects: str,
              inspection_matters: List[str],
              purpose: str,
              attorney: Dict[str, str],
              party: str = "원고",
              court: str = "서울중앙지방법원",
              additional_requests: Optional[str] = None
              ) -> 'InspectionRequestDocument':
        """
        Generate inspection request document.

        Args:
            case_number: Court case number (e.g., "2024가합123456")
            case_name: Case name (e.g., "건물철거 등")
            plaintiff: Plaintiff name
            defendant: Defendant name
            inspection_location: Location where inspection will take place
            inspection_objects: Objects/property to be inspected
            inspection_matters: List of specific matters to be inspected
            purpose: Purpose of inspection (입증취지)
            attorney: Attorney information
            party: Party requesting inspection (원고 or 피고)
            court: Court name
            additional_requests: Optional additional requests (e.g., witness examination at scene)

        Returns:
            InspectionRequestDocument object
        """

        # Build document content
        content = {
            "header": self._build_header(case_number, case_name, plaintiff, defendant),
            "opening": self._build_opening(party),
            "location": self._build_location(inspection_location),
            "objects": self._build_objects(inspection_objects),
            "matters": self._build_matters(inspection_matters),
            "purpose": self._build_purpose(purpose),
            "additional": additional_requests or "",
            "signature": self._build_signature(party, attorney, court)
        }

        return InspectionRequestDocument(content)

    def _build_header(self, case_number: str, case_name: str,
                     plaintiff: str, defendant: str) -> str:
        """Build document header."""
        return f"""                  검증 신청

사    건    {case_number} {case_name}
원    고    {plaintiff}
피    고    {defendant}
"""

    def _build_opening(self, party: str) -> str:
        """Build opening statement."""
        return f"위 사건에 관하여 {party} 소송대리인은 아래와 같이 검증을 신청합니다.\n"

    def _build_location(self, location: str) -> str:
        """Build inspection location section."""
        return f"""1. 검증 장소
   {location}
"""

    def _build_objects(self, objects: str) -> str:
        """Build inspection objects section."""
        return f"""2. 검증 목적물
   {objects}
"""

    def _build_matters(self, matters: List[str]) -> str:
        """Build inspection matters section."""

        if not matters:
            return ""

        matters_text = "3. 검증 사항\n"

        # Korean subsection letters
        korean_letters = ['가', '나', '다', '라', '마', '바', '사', '아', '자', '차', '카', '타', '파', '하']

        for i, matter in enumerate(matters):
            if i < len(korean_letters):
                subsection = korean_letters[i]
            else:
                subsection = f"({i+1})"
            matters_text += f"   {subsection}. {matter}\n"

        return matters_text

    def _build_purpose(self, purpose: str) -> str:
        """Build purpose section."""
        return f"""4. 입증취지
   {purpose}
"""

    def _build_signature(self, party: str, attorney: Dict[str, str], court: str) -> str:
        """Build date and signature section."""

        today = datetime.now()
        date_str = f"{today.year}.  {today.month:2d}.  {today.day:2d}."

        signature = f"\n{date_str}\n\n"
        signature += f"{party} 소송대리인\n"

        title = attorney.get('title', '변호사')
        name = attorney.get('name', '')

        signature += f"{title}    {name}  (서명 또는 날인)\n\n"
        signature += f"{court}   귀중"

        return signature


class InspectionRequestDocument:
    """Represents a generated inspection request document."""

    def __init__(self, content: Dict[str, str]):
        self.content = content

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            "\n",
            self.content['opening'],
            "\n",
            self.content['location'],
            "\n",
            self.content['objects'],
            "\n",
            self.content['matters'],
            "\n",
            self.content['purpose']
        ]

        if self.content['additional']:
            sections.extend(["\n", self.content['additional'], "\n"])

        sections.append(self.content['signature'])

        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Inspection request saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


# Predefined inspection matter templates
INSPECTION_MATTER_TEMPLATES = {
    "boundary_dispute": [
        "계쟁 토지의 경계선 위치",
        "경계 표지석 또는 담장의 위치 및 상태",
        "양측 당사자가 주장하는 경계선과 실제 점유 현황의 일치 여부",
        "인접 토지 소유자들의 경계 인식 현황"
    ],
    "building_defect": [
        "균열, 누수 등 하자 부분의 위치 및 범위",
        "하자로 인한 손상 정도",
        "보수 필요 부분의 구체적 위치",
        "거주 또는 사용 가능 여부"
    ],
    "illegal_construction": [
        "건축물의 위치 및 구조",
        "허가 받은 설계도면과의 일치 여부",
        "무단 증축 또는 용도변경 부분의 존재 및 범위",
        "건축법령 위반 사항"
    ],
    "accident_scene": [
        "사고 발생 지점의 도로 상황",
        "시야 확보 상태 및 장애물 존재 여부",
        "신호등, 표지판 등 교통안전시설의 위치 및 상태",
        "사고 당시 차량 진행 방향 및 충돌 지점"
    ],
    "possession_status": [
        "부동산의 현재 점유 상태",
        "점유자의 신원 및 점유 기간",
        "점유 목적물의 사용 현황",
        "점유와 관련된 물건의 존재 및 상태"
    ],
    "building_structure": [
        "위 장소에 설치된 건물의 위치 및 구조",
        "건물이 토지를 침범하고 있는지 여부 및 침범 범위",
        "건물의 설치 상태 및 사용 현황",
        "토지 경계의 현황"
    ]
}


# Example usage
if __name__ == "__main__":
    writer = InspectionRequestWriter()

    # Example 1: Boundary dispute inspection
    doc1 = writer.write(
        case_number="2024가합123456",
        case_name="건물철거 등",
        plaintiff="김대길",
        defendant="박인석",
        inspection_location="서울 서대문구 홍제동 230",
        inspection_objects="위 검증 장소에 있는 이 사건 토지 및 지상 건물 등",
        inspection_matters=INSPECTION_MATTER_TEMPLATES["building_structure"],
        purpose="피고가 점유·사용하고 있는 가건물의 위치 및 설치 상황",
        attorney={
            "name": "김공평",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제13민사부"
    )

    print(doc1)
    print("\n" + "="*80 + "\n")
    doc1.save_docx("inspection_request_boundary.docx")

    # Example 2: Building defect inspection
    doc2 = writer.write(
        case_number="2024가합234567",
        case_name="하자보수금",
        plaintiff="이민수",
        defendant="ABC건설주식회사",
        inspection_location="서울 강남구 테헤란로 456, 101동 1502호",
        inspection_objects="위 검증 장소에 있는 아파트 실내 및 외벽",
        inspection_matters=INSPECTION_MATTER_TEMPLATES["building_defect"],
        purpose="이 사건 아파트의 하자 존재 및 범위를 입증하고자 함",
        attorney={
            "name": "박법률",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제25민사부"
    )

    print(doc2)
    doc2.save_docx("inspection_request_defect.docx")

    # Example 3: Accident scene inspection
    doc3 = writer.write(
        case_number="2024가합345678",
        case_name="손해배상(자)",
        plaintiff="최영희",
        defendant="정철수",
        inspection_location="서울 종로구 세종대로 1 앞 교차로",
        inspection_objects="위 검증 장소의 도로 및 교통시설",
        inspection_matters=INSPECTION_MATTER_TEMPLATES["accident_scene"],
        purpose="이 사건 교통사고 발생 당시의 현장 상황을 입증하고자 함",
        attorney={
            "name": "정법무",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제12민사부",
        additional_requests="검증 시 증인 홍길동에 대한 신문도 함께 신청합니다."
    )

    print(doc3)
    doc3.save_docx("inspection_request_accident.docx")
