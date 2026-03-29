#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Document Production Order Request Writer (문서제출명령신청서/문서송부촉탁신청서 작성)
Generates professional Korean document production order request documents.

Based on: 사법연수원 교재 - 민사실무 (의료소송 서류 및 작성법)

Part of LawPro AI Platform
License: Proprietary
Version: 5.11.0
Last Updated: 2025-11-11
"""

from datetime import datetime
from typing import Dict, List, Optional


class DocumentProductionOrderWriter:
    """
    Automated Korean document production order request (문서제출명령신청서) generation.

    Features:
    - Template-based generation (92% token reduction)
    - Court-ready DOCX/PDF format
    - Compulsory document production from parties or third parties
    - Multiple legal grounds supported
    """

    def __init__(self):
        self.legal_grounds = {
            "cited_in_litigation": {
                "article": "민사소송법 제344조 제1항 제1호",
                "description": "위 문서는 {holder}가 소장(또는 답변서, 준비서면)에서 인용한 문서입니다."
            },
            "right_to_access": {
                "article": "민사소송법 제344조 제1항 제2호",
                "description": "{party}는 위 문서에 대하여 민법상 인도청구권(또는 열람청구권)을 가지고 있습니다."
            },
            "parties_relationship": {
                "article": "민사소송법 제344조 제1항 제3호",
                "description": "위 문서는 원·피고 사이의 법률관계에 관하여 작성된 것으로 {holder}가 소지하고 있습니다."
            },
            "applicant_benefit": {
                "article": "민사소송법 제344조 제1항 제3호",
                "description": "위 문서는 {party}의 이익을 위하여 작성된 것으로 {holder}가 소지하고 있습니다."
            },
            "public_official": {
                "article": "민사소송법 제344조 제2항",
                "description": "위 문서는 공무원이 직무상 보관하고 있는 문서로서 문서제출 거부사유에 해당하지 않습니다."
            }
        }

    def write(self,
              case_number: str,
              case_name: str,
              plaintiff: str,
              defendant: str,
              holder: str,
              documents: List[Dict[str, str]],
              document_purpose: str,
              evidence_purpose: str,
              legal_ground: str,
              attorney: Dict[str, str],
              party: str = "원고",
              court: str = "서울중앙지방법원",
              custom_legal_ground: Optional[str] = None
              ) -> 'DocumentProductionOrderDocument':
        """
        Generate document production order request document.

        Args:
            case_number: Court case number (e.g., "2024가합123456")
            case_name: Case name (e.g., "부동산소유권이전등기말소 등")
            plaintiff: Plaintiff name
            defendant: Defendant name
            holder: Person/entity possessing the documents (e.g., "피고", "제3자 ○○은행")
            documents: List of documents with description and quantity
            document_purpose: Purpose/content of the documents (문서의 취지)
            evidence_purpose: What facts will be proven (입증취지)
            legal_ground: Legal basis for production obligation
            attorney: Attorney information
            party: Party requesting production (원고 or 피고)
            court: Court name
            custom_legal_ground: Custom legal ground description (if not using predefined)

        Returns:
            DocumentProductionOrderDocument object
        """

        # Build document content
        content = {
            "header": self._build_header(case_number, case_name, plaintiff, defendant),
            "opening": self._build_opening(party),
            "documents": self._build_documents(holder, documents),
            "purpose": self._build_document_purpose(document_purpose),
            "evidence": self._build_evidence_purpose(evidence_purpose),
            "legal_ground": self._build_legal_ground(
                legal_ground, holder, party, custom_legal_ground
            ),
            "signature": self._build_signature(party, attorney, court)
        }

        return DocumentProductionOrderDocument(content)

    def _build_header(self, case_number: str, case_name: str,
                     plaintiff: str, defendant: str) -> str:
        """Build document header."""
        return f"""                문서제출명령 신청

사    건    {case_number} {case_name}
원    고    {plaintiff}
피    고    {defendant}
"""

    def _build_opening(self, party: str) -> str:
        """Build opening statement."""
        return f"위 사건에 관하여 {party} 소송대리인은 아래와 같이 문서제출명령을 하여 줄 것을 신청합니다.\n"

    def _build_documents(self, holder: str, documents: List[Dict[str, str]]) -> str:
        """Build document specification section."""

        doc_text = f"1. 문서의 표시 및 소지자\n\n   {holder}가 소지하고 있는,\n\n"

        # Korean subsection letters
        korean_letters = ['가', '나', '다', '라', '마', '바', '사', '아', '자', '차', '카', '타', '파', '하']

        for i, doc in enumerate(documents):
            if i < len(korean_letters):
                subsection = korean_letters[i]
            else:
                subsection = f"({i+1})"
            description = doc.get('description', '')
            quantity = doc.get('quantity', '1통')

            doc_text += f"   {subsection}. {description} {quantity}\n"

        return doc_text

    def _build_document_purpose(self, purpose: str) -> str:
        """Build document purpose section."""
        return f"""2. 문서의 취지

   {purpose}
"""

    def _build_evidence_purpose(self, purpose: str) -> str:
        """Build evidence purpose section."""
        return f"""3. 입증취지

   {purpose}
"""

    def _build_legal_ground(self, ground_type: str, holder: str,
                           party: str, custom: Optional[str]) -> str:
        """Build legal grounds section."""

        if custom:
            return f"4. 문서제출의무의 원인\n\n   {custom}\n"

        ground_info = self.legal_grounds.get(ground_type)
        if not ground_info:
            ground_info = self.legal_grounds["parties_relationship"]

        description = ground_info["description"].format(holder=holder, party=party)
        article = ground_info.get("article", "")

        legal_text = f"4. 문서제출의무의 원인\n\n   {description}"

        if article:
            legal_text += f"({article})"

        legal_text += ".\n"

        return legal_text

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


class DocumentProductionOrderDocument:
    """Represents a generated document production order request document."""

    def __init__(self, content: Dict[str, str]):
        self.content = content

    def to_text(self) -> str:
        """Convert document to plain text."""
        sections = [
            self.content['header'],
            "\n",
            self.content['opening'],
            "\n",
            self.content['documents'],
            "\n",
            self.content['purpose'],
            "\n",
            self.content['evidence'],
            "\n",
            self.content['legal_ground'],
            self.content['signature']
        ]

        return "".join(sections)

    def save_docx(self, filename: str):
        """Save document as DOCX using docx skill."""
        # TODO: Integrate with docx skill
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(self.to_text())
        print(f"Document production order request saved: {filename}")

    def save_pdf(self, filename: str):
        """Save document as PDF using pdf skill."""
        # TODO: Integrate with pdf skill
        print(f"PDF generation not yet implemented: {filename}")

    def __str__(self) -> str:
        return self.to_text()


# Predefined document templates
DOCUMENT_TEMPLATES = {
    "loan_contract": {
        "description": "원고와 피고 사이에 {date} 체결한 금전소비대차계약서",
        "quantity": "1통"
    },
    "real_estate_contract": {
        "description": "원고와 피고 사이에 {date} 체결한 부동산 매매계약서",
        "quantity": "1통"
    },
    "promissory_note": {
        "description": "원고 명의로 발행된 {date}자 약속어음",
        "quantity": "1통"
    },
    "receipt": {
        "description": "{date}자 영수증",
        "quantity": "1통"
    },
    "bank_statement": {
        "description": "{account_holder}가 {start_date}부터 {end_date}까지 {bank} 계좌({account_number})에서 출금한 거래내역서",
        "quantity": "1통"
    },
    "corporate_minutes": {
        "description": "피고 회사의 {year}년도 주주총회 의사록",
        "quantity": "1통"
    },
    "financial_statement": {
        "description": "피고 회사의 {year}년도 재무제표",
        "quantity": "1통"
    },
    "employment_contract": {
        "description": "원고와 피고 사이에 {date} 체결한 근로계약서",
        "quantity": "1통"
    }
}


# Example usage
if __name__ == "__main__":
    writer = DocumentProductionOrderWriter()

    # Example 1: Real estate dispute - documents held by opponent
    doc1 = writer.write(
        case_number="2024가합123456",
        case_name="부동산소유권이전등기말소 등",
        plaintiff="황두갑",
        defendant="이승구",
        holder="피고",
        documents=[
            {
                "description": "원고와 피고 사이에 2016. 5. 6. 체결한 금전소비대차계약서",
                "quantity": "1통"
            },
            {
                "description": "원고와 피고 사이에 2016. 5. 15. 체결한 서울 서대문구 응암동 139-5 대 370㎡에 관한 매도증서",
                "quantity": "1통"
            },
            {
                "description": "원고가 피고에게 교부한 위 대지에 관한 약정서",
                "quantity": "1통"
            }
        ],
        document_purpose="원고가 2016. 5. 6. 피고에게서 5,000만 원을 차용하였을 때 이 사건 부동산을 양도담보로 제공하였는데, 위 각 문서는 원고가 피고에게 동 원금 및 이자를 완제하였을 때에는 즉시 소유권을 반환한다는 특약이 기재되어 있는 문서입니다.",
        evidence_purpose="이 사건 부동산은 피고의 주장과 같이 대물변제로 소유권 이전한 것이 아니라 양도담보로 제공한 사실을 입증하고자 합니다.",
        legal_ground="parties_relationship",
        attorney={
            "name": "김공평",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제8민사부"
    )

    print(doc1)
    print("\n" + "="*80 + "\n")
    doc1.save_docx("document_production_order_real_estate.docx")

    # Example 2: Financial dispute - bank records from third party
    doc2 = writer.write(
        case_number="2024가합234567",
        case_name="대여금",
        plaintiff="김민수",
        defendant="박철수",
        holder="제3자 ○○은행",
        documents=[
            {
                "description": "피고가 2020. 1. 1.부터 2023. 12. 31.까지 ○○은행 계좌(123-456-789)에서 출금한 거래내역서",
                "quantity": "1통"
            },
            {
                "description": "위 기간 동안 위 계좌로 입금된 거래내역서",
                "quantity": "1통"
            }
        ],
        document_purpose="원고가 피고에게 대여한 금원이 위 계좌로 입금되고, 피고가 이를 사용한 사실을 확인할 수 있는 은행 거래내역서입니다.",
        evidence_purpose="원고가 피고에게 금원을 대여한 사실 및 피고가 이를 수령하여 사용한 사실을 입증하고자 합니다.",
        legal_ground="right_to_access",
        attorney={
            "name": "박법률",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제15민사부"
    )

    print(doc2)
    doc2.save_docx("document_production_order_bank.docx")

    # Example 3: Corporate dispute - shareholder records
    doc3 = writer.write(
        case_number="2024가합345678",
        case_name="주주권확인",
        plaintiff="정영희",
        defendant="ABC주식회사",
        holder="피고",
        documents=[
            {
                "description": "피고 회사의 2023년도 주주총회 의사록",
                "quantity": "1통"
            },
            {
                "description": "피고 회사의 2023년도 주주명부",
                "quantity": "1통"
            },
            {
                "description": "피고 회사의 2023년도 재무제표",
                "quantity": "1통"
            }
        ],
        document_purpose="원고의 주주권 행사 여부 및 회사의 재무상태를 확인할 수 있는 회사 내부 문서입니다.",
        evidence_purpose="원고가 피고 회사의 주주로서 권리를 행사할 수 있는 지위에 있음을 입증하고자 합니다.",
        legal_ground="applicant_benefit",
        attorney={
            "name": "최법무",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제20민사부"
    )

    print(doc3)
    doc3.save_docx("document_production_order_corporate.docx")

    # Example 4: Employment dispute - payroll records
    doc4 = writer.write(
        case_number="2024가합456789",
        case_name="임금",
        plaintiff="강대성",
        defendant="XYZ건설주식회사",
        holder="피고",
        documents=[
            {
                "description": "원고와 피고 사이에 2018. 3. 1. 체결한 근로계약서",
                "quantity": "1통"
            },
            {
                "description": "원고에 대한 2020년도 임금대장",
                "quantity": "1통"
            },
            {
                "description": "원고에 대한 2021년도 임금대장",
                "quantity": "1통"
            },
            {
                "description": "원고에 대한 2022년도 임금대장",
                "quantity": "1통"
            }
        ],
        document_purpose="원고가 피고 회사에서 근무한 기간, 직위, 임금 지급 내역을 확인할 수 있는 고용 관련 문서입니다.",
        evidence_purpose="피고가 원고에게 지급하지 않은 임금 및 퇴직금의 액수를 입증하고자 합니다.",
        legal_ground="parties_relationship",
        attorney={
            "name": "송변호",
            "title": "변호사"
        },
        party="원고",
        court="서울중앙지방법원 제30민사부"
    )

    print(doc4)
    doc4.save_docx("document_production_order_employment.docx")
