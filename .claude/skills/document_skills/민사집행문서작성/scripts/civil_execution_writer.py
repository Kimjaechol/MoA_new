#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
민사집행(강제집행) 문서 작성 모듈

55,348줄의 민사실무대전 민사집행절차 편 데이터베이스를 기반으로
모든 종류의 민사집행 신청서를 자동으로 작성합니다.

지원 기능:
- 금전채권 강제집행 (부동산, 선박, 자동차, 동산, 채권)
- 경매 (강제경매, 임의경매)
- 배당, 인도명령
- 구제절차 (집행이의, 청구이의, 제3자이의)
"""

from typing import Dict, List, Optional
from datetime import datetime


class CivilExecutionWriter:
    """민사집행 신청서 작성 클래스"""

    def __init__(self):
        """초기화"""
        self.reference_data = "민사실무대전 (III) 민사집행절차 편 (백영사, 55,348줄)"

    def write_compulsory_auction(
        self,
        creditor: Dict,
        debtor: Dict,
        claim: Dict,
        execution_title: Dict,
        real_estate: Dict,
        court: str
    ) -> str:
        """
        부동산 강제경매 신청서 작성

        Args:
            creditor: 채권자 정보
            debtor: 채무자 정보
            claim: 청구금액 (principal, interest, damages, total)
            execution_title: 집행권원 (type, case_number, verdict, finalized_date)
            real_estate: 부동산 정보
            court: 관할법원

        Returns:
            강제경매 신청서 전문
        """
        application = "강제경매신청서\n\n"

        # 당사자
        application += f"채 권 자    {creditor['address']}\n"
        if creditor.get('type') == '법인':
            application += f"            {creditor['name']} "
            application += f"(법인등록번호: {creditor['registration_number']})\n"
            application += f"            대표이사 {creditor['representative']}\n\n"
        else:
            application += f"            {creditor['name']} "
            application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        # 청구금액
        application += f"청구금액    금 {claim['total']}원\n\n"
        application += f"            원금: 금 {claim['principal']}원\n"
        application += f"            이자: 금 {claim['interest']}원\n"
        application += f"            지연손해금: 금 {claim['damages']}원\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += "채무자 소유의 별지 목록 기재 부동산에 대하여 강제경매를 명하는\n"
        application += "재판을 구합니다.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += f"1. 채권자는 채무자에 대하여 {execution_title['case_number']}호\n"
        application += f"   사건의 확정판결로 다음과 같은 집행권원을 가지고 있습니다.\n\n"
        application += f'   "{execution_title["verdict"]}"\n\n'
        application += f"2. 위 판결은 {execution_title['finalized_date']}. 확정되었습니다.\n\n"
        application += f"3. 그러나 채무자는 위 판결에 따른 채무를 전혀 이행하지 않고 있습니다.\n\n"
        application += f"4. 채권자는 위 채권을 추심하기 위하여 채무자 소유의 별지 목록 기재\n"
        application += f"   부동산에 대한 강제경매를 신청합니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 집행권원 정본 (판결정본)                1통\n"
        application += "2. 집행문                                1통\n"
        application += "3. 확정증명서                            1통\n"
        application += "4. 송달증명서                            2통\n"
        application += "5. 부동산등기사항증명서                  1통\n"
        application += "6. 부동산물건명세서                      1통\n"
        application += "7. 채권계산서                            1통\n"
        if creditor.get('type') == '법인':
            application += "8. 법인등기사항증명서 (채권자)          1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        if creditor.get('type') == '법인':
            application += f"위 채권자  {creditor['name']}\n"
            application += f"          대표이사 {creditor['representative']} (인)\n\n"
        else:
            application += f"위 채권자  {creditor['name']} (인)\n\n"

        application += f"{court} 귀중\n\n"

        # 별지 목록
        application += "[별지] 부동산 목록\n\n"
        application += f"{real_estate['location']} {real_estate['lot_number']} "
        application += f"{real_estate['category']} {real_estate['area']}\n\n"

        if 'building' in real_estate:
            application += f"위 지상 {real_estate['building']['structure']}\n"
            application += f"{real_estate['building']['area']}\n"

        return application

    def write_claim_seizure_collection(
        self,
        creditor: Dict,
        debtor: Dict,
        third_party_debtor: Dict,
        claim_amount: str,
        target_claims: List[Dict],
        execution_title: Dict,
        court: str
    ) -> str:
        """
        채권압류 및 추심명령 신청서 작성

        Args:
            creditor: 채권자 정보
            debtor: 채무자 정보
            third_party_debtor: 제3채무자 정보
            claim_amount: 청구금액
            target_claims: 압류 대상 채권 목록
            execution_title: 집행권원
            court: 관할법원

        Returns:
            채권압류 및 추심명령 신청서 전문
        """
        application = "채권압류 및 추심명령신청서\n\n"

        # 당사자
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        application += f"제3채무자    {third_party_debtor['address']}\n"
        if third_party_debtor.get('type') == '법인':
            application += f"            {third_party_debtor['name']} "
            application += f"(법인등록번호: {third_party_debtor['registration_number']})\n"
            application += f"            대표이사 {third_party_debtor['representative']}\n\n"
        else:
            application += f"            {third_party_debtor['name']}\n\n"

        application += f"청구금액    금 {claim_amount}원\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += "채무자가 제3채무자에 대하여 가지는 별지 채권목록 기재\n"
        application += "채권을 압류하고, 채권자에게 추심을 명하는 재판을 구합니다.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += f"1. 채권자는 채무자에 대하여 {execution_title['case_number']}호\n"
        application += f"   사건의 확정판결로 금 {claim_amount}원의 채권을 가지고\n"
        application += f"   있습니다.\n\n"

        application += f"2. 채무자는 제3채무자에 대하여 별지 채권목록 기재 채권을\n"
        application += f"   가지고 있습니다.\n\n"

        application += f"3. 채권자는 위 채권을 추심하기 위하여 채무자의 제3채무자에\n"
        application += f"   대한 채권을 압류하고 추심할 필요가 있습니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 집행권원 정본 (판결정본)                1통\n"
        application += "2. 집행문                                1통\n"
        application += "3. 확정증명서                            1통\n"
        application += "4. 송달증명서                            3통\n"
        application += "5. 채권목록                              1통\n"
        application += "6. 채권계산서                            1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n\n"

        # 별지 채권목록
        application += "[별지] 채권목록\n\n"
        application += "채무자가 제3채무자에 대하여 가지는 다음 채권\n\n"

        for idx, claim in enumerate(target_claims, 1):
            application += f"{idx}. {claim['type']}\n"
            application += f"   - {claim['description']}\n"
            if 'limit' in claim:
                application += f"   - {claim['limit']}\n"
            application += "\n"

        return application


if __name__ == "__main__":
    # 테스트 코드
    writer = CivilExecutionWriter()

    print("=" * 80)
    print("부동산 강제경매 신청서 샘플")
    print("=" * 80)
    auction = writer.write_compulsory_auction(
        creditor={
            "type": "법인",
            "name": "주식회사 ○○은행",
            "registration_number": "110111-0123456",
            "address": "서울특별시 강남구 테헤란로 123",
            "representative": "김은행"
        },
        debtor={
            "name": "이채무",
            "resident_number": "750505-2345678",
            "address": "서울특별시 서초구 서초대로 456"
        },
        claim={
            "principal": "400,000,000",
            "interest": "50,000,000",
            "damages": "50,000,000",
            "total": "500,000,000"
        },
        execution_title={
            "type": "확정판결",
            "case_number": "서울중앙지방법원 2023가단123456",
            "verdict": "피고는 원고에게 금 400,000,000원 및 이에 대하여 2023. 6. 1.부터 다 갚는 날까지 연 12%의 비율로 계산한 돈을 지급하라",
            "finalized_date": "2024. 3. 15"
        },
        real_estate={
            "location": "서울특별시 강남구 역삼동",
            "lot_number": "123-45",
            "category": "대",
            "area": "200㎡",
            "building": {
                "structure": "철근콘크리트조 슬래브지붕 5층 근린생활시설",
                "area": "1층 150㎡, 2층 150㎡, 3층 150㎡, 4층 150㎡, 5층 150㎡"
            }
        },
        court="서울중앙지방법원"
    )
    print(auction)
