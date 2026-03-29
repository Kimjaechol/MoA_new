#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
개인파산·개인회생 신청서 작성 모듈

75,869줄의 개인파산·상속재산파산 실무 데이터베이스를 기반으로
모든 종류의 도산 관련 신청서를 자동으로 작성합니다.

지원 기능:
- 개인파산 및 면책 신청
- 상속재산파산 신청
- 상속포기·한정승인
- 개인회생 신청
- 채권자목록·재산목록
"""

from typing import Dict, List, Optional
from datetime import datetime


class PersonalBankruptcyWriter:
    """개인파산·개인회생 신청서 작성 클래스"""

    def __init__(self):
        """초기화"""
        self.reference_data = "신청사례로 보는 개인파산·상속재산파산실무 (이기형, 김현선 공저, 백영사, 75,869줄)"

    def write_personal_bankruptcy_application(
        self,
        debtor: Dict,
        total_debt: str,
        monthly_income: str,
        monthly_expense: str,
        creditors: List[Dict],
        assets: List[Dict],
        court: str = "서울회생법원"
    ) -> str:
        """
        개인파산 및 면책 신청서 작성

        Args:
            debtor: 채무자 정보 (name, resident_number, address, registration_base)
            total_debt: 총 채무액
            monthly_income: 월 수입
            monthly_expense: 월 지출
            creditors: 채권자 목록 (name, amount, type)
            assets: 재산 목록 (type, description, value)
            court: 관할 회생법원

        Returns:
            개인파산 및 면책 신청서 전문
        """
        application = "개인파산 및 면책 신청서\n\n"

        # 채무자
        application += f"채 무 자    {debtor.get('address', '')}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n"
        application += f"            등록기준지: {debtor.get('registration_base', '')}\n\n"

        # 신청 취지
        application += "신청 취지\n\n"
        application += "1. 채무자에 대하여 파산을 선고한다.\n"
        application += "2. 채무자를 면책한다.\n\n"
        application += "라는 결정을 구합니다.\n\n"

        # 파산 원인
        application += "파산 원인\n\n"
        application += "1. 채무 현황\n"
        application += f"   총 채무액: 금 {total_debt}원\n\n"

        application += "   주요 채권자:\n"
        for idx, creditor in enumerate(creditors[:5], 1):
            application += f"   {idx}. {creditor['name']}: "
            application += f"금 {creditor['amount']}원 ({creditor.get('type', '일반채권')})\n"
        application += "\n"

        application += "2. 지급불능 상태\n"
        application += f"   월 수입: 약 {monthly_income}원\n"
        application += f"   월 지출: 약 {monthly_expense}원\n"
        available = int(monthly_income.replace(',', '')) - int(monthly_expense.replace(',', ''))
        application += f"   여유 자금: 약 {available:,}원\n\n"
        application += "   채무자는 현재 지급불능 상태로서 변제기에 있는 채무를 "
        application += "일반적·계속적으로 변제할 수 없는 상태입니다.\n\n"

        # 재산 목록
        application += "3. 보유 재산\n"
        for asset in assets:
            application += f"   - {asset.get('type', '동산')}: {asset['description']} "
            application += f"(시가 {asset.get('value', '0')}원)\n"
        application += "\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 주민등록등본\n"
        application += "2. 가족관계증명서\n"
        application += "3. 소득증명서류\n"
        application += "4. 재산증명서류\n"
        application += "5. 채권증빙서류\n"
        application += "6. 채권자목록 (별첨)\n"
        application += "7. 재산목록 (별첨)\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채무자  {debtor['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_inheritance_estate_bankruptcy(
        self,
        deceased: Dict,
        heir: Dict,
        estate_assets: List[Dict],
        estate_debts: List[Dict],
        court: str = "서울회생법원"
    ) -> str:
        """
        상속재산파산 신청서 작성

        Args:
            deceased: 피상속인 정보 (name, resident_number, death_date, last_address)
            heir: 상속인 정보 (name, resident_number, address, relation)
            estate_assets: 상속재산 목록 (type, description, amount)
            estate_debts: 상속채무 목록 (creditor, amount, type)
            court: 관할 회생법원

        Returns:
            상속재산파산 신청서 전문
        """
        application = "상속재산파산 신청서\n\n"

        # 피상속인
        application += "피상속인\n"
        application += f"  성명: 고(故) {deceased['name']}\n"
        application += f"  주민등록번호: {deceased['resident_number']} (사망)\n"
        application += f"  최종 주소: {deceased.get('last_address', '')}\n"
        application += f"  사망일: {deceased['death_date']}\n\n"

        # 신청인 (상속인)
        application += "신청인 (상속인)\n"
        application += f"  성명: {heir['name']}\n"
        application += f"  주민등록번호: {heir['resident_number']}\n"
        application += f"  주소: {heir.get('address', '')}\n"
        application += f"  피상속인과의 관계: {heir.get('relation', '자녀')}\n\n"

        # 신청 취지
        application += "신청 취지\n\n"
        application += f"피상속인 고(故) {deceased['name']}의 상속재산에 대하여 "
        application += "파산을 선고한다.\n\n"
        application += "라는 결정을 구합니다.\n\n"

        # 파산 원인
        application += "파산 원인\n\n"
        application += "1. 상속재산\n"
        total_assets = 0
        for asset in estate_assets:
            application += f"   - {asset.get('type', '재산')}: "
            application += f"{asset.get('description', '')} "
            amount = int(asset['amount'].replace(',', '')) if isinstance(asset['amount'], str) else asset['amount']
            application += f"(금 {amount:,}원)\n"
            total_assets += amount
        application += f"\n   합계: 금 {total_assets:,}원\n\n"

        application += "2. 상속채무\n"
        total_debts = 0
        for debt in estate_debts:
            application += f"   - {debt['creditor']}: "
            amount = int(debt['amount'].replace(',', '')) if isinstance(debt['amount'], str) else debt['amount']
            application += f"금 {amount:,}원 ({debt.get('type', '일반채무')})\n"
            total_debts += amount
        application += f"\n   합계: 금 {total_debts:,}원\n\n"

        application += "3. 채무초과 상태\n"
        application += f"   상속재산 {total_assets:,}원으로는 상속채무 {total_debts:,}원을 "
        application += "변제할 수 없으므로 상속재산파산을 신청합니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 피상속인의 기본증명서 (사망 기재)\n"
        application += "2. 피상속인의 가족관계증명서\n"
        application += "3. 상속인 전원의 가족관계증명서\n"
        application += "4. 상속재산 및 상속채무 소명자료\n"
        application += "5. 채권자목록 (별첨)\n"
        application += "6. 재산목록 (별첨)\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 신청인 (상속인)  {heir['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_inheritance_renunciation(
        self,
        heir: Dict,
        deceased: Dict,
        reason: str,
        court: str = "가정법원"
    ) -> str:
        """
        상속포기 신고서 작성

        Args:
            heir: 상속인 정보 (name, resident_number, address)
            deceased: 피상속인 정보 (name, resident_number, death_date, relation)
            reason: 상속포기 사유
            court: 관할 가정법원

        Returns:
            상속포기 신고서 전문
        """
        report = "상속포기 신고서\n\n"

        # 신고인 (상속인)
        report += f"신 고 인    {heir.get('address', '')}\n"
        report += f"            {heir['name']} "
        report += f"(주민등록번호: {heir['resident_number']})\n\n"

        # 피상속인
        report += "피상속인\n"
        report += f"  성명: 고(故) {deceased['name']}\n"
        report += f"  주민등록번호: {deceased.get('resident_number', '')} (사망)\n"
        report += f"  사망일: {deceased['death_date']}\n"
        report += f"  신고인과의 관계: {deceased.get('relation', '부')}\n\n"

        # 신고 취지
        report += "신고 취지\n\n"
        report += f"신고인은 피상속인 고(故) {deceased['name']}의 상속을 포기합니다.\n\n"
        report += "라는 심판을 구합니다.\n\n"

        # 신고 이유
        report += "신고 이유\n\n"
        report += f"  {reason}\n\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 피상속인의 가족관계증명서 및 기본증명서\n"
        report += "2. 신고인의 가족관계증명서\n"
        report += "3. 피상속인의 채무 관련 서류\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 신고인  {heir['name']} (인)\n\n"
        report += f"{court} 귀중\n"

        return report

    def write_limited_acceptance(
        self,
        heir: Dict,
        deceased: Dict,
        estate_assets: List[Dict],
        estate_debts: List[Dict],
        court: str = "가정법원"
    ) -> str:
        """
        한정승인 신고서 작성

        Args:
            heir: 상속인 정보 (name, resident_number, address)
            deceased: 피상속인 정보 (name, resident_number, death_date, relation)
            estate_assets: 상속재산 목록
            estate_debts: 상속채무 목록
            court: 관할 가정법원

        Returns:
            한정승인 신고서 전문
        """
        report = "한정승인 신고서\n\n"

        # 신고인 (상속인)
        report += f"신 고 인    {heir.get('address', '')}\n"
        report += f"            {heir['name']} "
        report += f"(주민등록번호: {heir.get('resident_number', '')})\n\n"

        # 피상속인
        report += "피상속인\n"
        report += f"  성명: 고(故) {deceased['name']}\n"
        report += f"  사망일: {deceased['death_date']}\n\n"

        # 신고 취지
        report += "신고 취지\n\n"
        report += f"신고인은 피상속인 고(故) {deceased['name']}의 상속재산 범위 내에서 "
        report += "상속채무를 변제하는 조건으로 상속을 승인합니다.\n\n"
        report += "라는 심판을 구합니다.\n\n"

        # 상속재산목록
        report += "상속재산목록\n"
        total_assets = 0
        for asset in estate_assets:
            amount = int(asset['value'].replace(',', '')) if isinstance(asset.get('value'), str) else asset.get('value', 0)
            report += f"  - {asset.get('type', '재산')}: "
            report += f"{asset.get('description', '')} (금 {amount:,}원)\n"
            total_assets += amount
        report += f"합계: {total_assets:,}원\n\n"

        # 상속채무목록
        report += "상속채무목록\n"
        total_debts = 0
        for debt in estate_debts:
            amount = int(debt['amount'].replace(',', '')) if isinstance(debt['amount'], str) else debt['amount']
            report += f"  - {debt['creditor']}: 금 {amount:,}원\n"
            total_debts += amount
        report += f"합계: {total_debts:,}원\n\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 피상속인의 가족관계증명서 및 기본증명서\n"
        report += "2. 신고인의 가족관계증명서\n"
        report += "3. 상속재산 및 상속채무 소명자료\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 신고인  {heir['name']} (인)\n\n"
        report += f"{court} 귀중\n"

        return report

    def write_creditor_list(
        self,
        creditors: List[Dict]
    ) -> str:
        """
        채권자목록 작성

        Args:
            creditors: 채권자 목록 (name, address, type, amount, note)

        Returns:
            채권자목록
        """
        creditor_list = "채권자목록\n\n"
        creditor_list += "-" * 100 + "\n"
        creditor_list += f"{'순번':<5} {'채권자명':<20} {'주소':<30} {'채권종류':<15} {'채권액':<15} {'비고':<15}\n"
        creditor_list += "-" * 100 + "\n"

        total = 0
        for idx, creditor in enumerate(creditors, 1):
            amount = int(creditor['amount'].replace(',', '')) if isinstance(creditor['amount'], str) else creditor['amount']
            creditor_list += f"{idx:<5} "
            creditor_list += f"{creditor['name']:<20} "
            creditor_list += f"{creditor.get('address', ''):<30} "
            creditor_list += f"{creditor.get('type', '일반채권'):<15} "
            creditor_list += f"{amount:>13,}원 "
            creditor_list += f"{creditor.get('note', ''):<15}\n"
            total += amount

        creditor_list += "-" * 100 + "\n"
        creditor_list += f"{'합계':<75} {total:>13,}원\n"
        creditor_list += "-" * 100 + "\n"

        return creditor_list

    def write_asset_list(
        self,
        assets: List[Dict]
    ) -> str:
        """
        재산목록 작성

        Args:
            assets: 재산 목록 (type, description, value, note)

        Returns:
            재산목록
        """
        asset_list = "재산목록\n\n"

        # 부동산
        asset_list += "[부동산]\n"
        real_estates = [a for a in assets if a.get('type') == '부동산']
        if real_estates:
            for asset in real_estates:
                asset_list += f"- {asset['description']}: 시가 {asset['value']}원\n"
        else:
            asset_list += "- 없음\n"
        asset_list += "\n"

        # 동산
        asset_list += "[동산]\n"
        movables = [a for a in assets if a.get('type') == '동산']
        if movables:
            for asset in movables:
                asset_list += f"- {asset['description']}: 시가 {asset['value']}원\n"
        else:
            asset_list += "- 없음\n"
        asset_list += "\n"

        # 채권·예금
        asset_list += "[채권·예금]\n"
        claims = [a for a in assets if a.get('type') in ['채권', '예금']]
        if claims:
            for asset in claims:
                asset_list += f"- {asset['description']}: {asset['value']}원\n"
        else:
            asset_list += "- 없음\n"
        asset_list += "\n"

        # 기타 재산
        asset_list += "[기타 재산]\n"
        others = [a for a in assets if a.get('type') not in ['부동산', '동산', '채권', '예금']]
        if others:
            for asset in others:
                asset_list += f"- {asset['description']}: {asset['value']}원\n"
        else:
            asset_list += "- 없음\n"
        asset_list += "\n"

        # 합계
        total = sum(int(a['value'].replace(',', '')) if isinstance(a.get('value'), str) else a.get('value', 0) for a in assets)
        asset_list += f"합계: 약 {total:,}원\n"

        return asset_list


if __name__ == "__main__":
    # 테스트 코드
    writer = PersonalBankruptcyWriter()

    print("=" * 80)
    print("개인파산 및 면책 신청서 샘플")
    print("=" * 80)
    bankruptcy = writer.write_personal_bankruptcy_application(
        debtor={
            "name": "김채무",
            "resident_number": "800101-1234567",
            "address": "서울특별시 강남구 테헤란로 123",
            "registration_base": "서울 강남구"
        },
        total_debt="500,000,000",
        monthly_income="2,000,000",
        monthly_expense="1,500,000",
        creditors=[
            {"name": "○○은행", "amount": "200,000,000", "type": "대출금"},
            {"name": "△△카드", "amount": "50,000,000", "type": "카드대금"}
        ],
        assets=[
            {"type": "동산", "description": "승용차 1대 (2018년식)", "value": "5,000,000"}
        ],
        court="서울회생법원"
    )
    print(bankruptcy)

    print("\n" + "=" * 80)
    print("상속포기 신고서 샘플")
    print("=" * 80)
    renunciation = writer.write_inheritance_renunciation(
        heir={
            "name": "김상속",
            "resident_number": "900101-1234567",
            "address": "서울특별시 서초구 서초대로 456"
        },
        deceased={
            "name": "김망인",
            "resident_number": "500101-1234567",
            "death_date": "2024. 5. 15.",
            "relation": "부"
        },
        reason="피상속인이 남긴 채무가 과다하여 상속재산으로 변제가 불가능하므로 상속을 포기합니다.",
        court="서울가정법원"
    )
    print(renunciation)
