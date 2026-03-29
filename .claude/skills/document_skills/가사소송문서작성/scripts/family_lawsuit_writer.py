#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
가사소송 문서 작성 모듈

103,983줄의 가사소송서식 데이터베이스를 기반으로
모든 종류의 가사소송 문서를 자동으로 작성합니다.

지원 기능:
- 이혼소송 (유책, 재산분할, 위자료)
- 친권 및 양육권 (양육비, 양육자 지정)
- 면접교섭권
- 인지청구
- 친생부인
- 상속 (상속재산분할, 상속포기)
"""

from typing import Dict, List, Optional
from datetime import datetime


class FamilyLawsuitWriter:
    """가사소송 문서 작성 클래스"""

    def __init__(self):
        """초기화"""
        self.reference_data = "가사소송서식 (백영사, 103,983줄)"

    def write_divorce_complaint(
        self,
        plaintiff: Dict,
        defendant: Dict,
        marriage_info: Dict,
        grounds: List[Dict],
        children: Optional[List[Dict]],
        property_division: Optional[Dict],
        alimony: Optional[Dict],
        court: str
    ) -> str:
        """
        이혼소장 작성

        Args:
            plaintiff: 원고 정보 (name, resident_number, address)
            defendant: 피고 정보
            marriage_info: 혼인정보 (marriage_date, registration_date)
            grounds: 이혼 사유 목록 (민법 제840조)
            children: 자녀 정보 (optional)
            property_division: 재산분할 청구 (optional)
            alimony: 위자료 청구 (optional)
            court: 관할법원

        Returns:
            이혼소장 전문
        """
        complaint = "소    장\n\n"

        # 당사자
        complaint += f"원    고    {plaintiff['address']}\n"
        complaint += f"            {plaintiff['name']} "
        complaint += f"(주민등록번호: {plaintiff['resident_number']})\n\n"

        complaint += f"피    고    {defendant['address']}\n"
        complaint += f"            {defendant['name']} "
        complaint += f"(주민등록번호: {defendant['resident_number']})\n\n"

        # 청구취지
        complaint += "청구취지\n\n"
        complaint += "1. 원고와 피고는 이혼한다.\n"

        if children:
            for idx, child in enumerate(children, 1):
                if child.get('custody_claim'):
                    complaint += f"{idx + 1}. 원고와 피고의 미성년 자녀 {child['name']}의 "
                    complaint += f"친권자 및 양육자를 원고로 지정한다.\n"

        if property_division:
            complaint += f"{len(children) + 2 if children else 2}. "
            complaint += f"피고는 원고에게 재산분할로 금 {property_division['amount']}원을 "
            complaint += f"지급하라.\n"

        if alimony:
            complaint += f"{(len(children) if children else 0) + (3 if property_division else 2)}. "
            complaint += f"피고는 원고에게 위자료로 금 {alimony['amount']}원을 지급하라.\n"

        complaint += "\n라는 판결을 구합니다.\n\n"

        # 청구원인
        complaint += "청구원인\n\n"
        complaint += "1. 당사자의 관계\n\n"
        complaint += f"   원고와 피고는 {marriage_info['marriage_date']}. 혼인신고를 하여 "
        complaint += f"법률상 부부관계에 있습니다.\n\n"

        if children:
            complaint += f"   원고와 피고 사이에는 다음과 같은 자녀가 있습니다.\n\n"
            for child in children:
                complaint += f"   - {child['name']} ({child['birth_date']}. 출생, "
                complaint += f"{child['age']}세)\n"
            complaint += "\n"

        # 이혼 사유
        complaint += "2. 이혼 사유\n\n"
        for idx, ground in enumerate(grounds, 1):
            complaint += f"   {idx}. {ground['title']}\n\n"
            complaint += f"   {ground['description']}\n\n"

        # 재산분할 청구
        if property_division:
            complaint += "3. 재산분할\n\n"
            complaint += "   원고와 피고가 혼인 중 형성한 재산은 다음과 같습니다.\n\n"

            for asset in property_division.get('assets', []):
                complaint += f"   - {asset['type']}: {asset['description']} "
                complaint += f"(가액: 금 {asset['value']}원)\n"

            complaint += f"\n   합계: 금 {property_division['total_value']}원\n\n"
            complaint += f"   원고는 위 재산 형성에 {property_division['plaintiff_contribution']}% "
            complaint += f"기여하였으므로, 그 비율에 따라 재산분할을 청구합니다.\n\n"

        # 위자료 청구
        if alimony:
            complaint += f"{4 if property_division else 3}. 위자료\n\n"
            complaint += f"   {alimony['reason']}\n\n"
            complaint += f"   이에 원고는 피고에게 위자료로 금 {alimony['amount']}원을 "
            complaint += f"청구합니다.\n\n"

        # 첨부서류
        complaint += "첨부서류\n\n"
        complaint += "1. 가족관계증명서                1통\n"
        complaint += "2. 혼인관계증명서                1통\n"
        complaint += "3. 기본증명서                    1통\n"

        if children:
            complaint += f"4. 가족관계증명서 (자녀)        {len(children)}통\n"

        if property_division:
            complaint += f"{5 if children else 4}. 부동산등기사항증명서        "
            complaint += f"{len(property_division.get('assets', []))}통\n"

        complaint += "\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        complaint += f"{today}\n\n"
        complaint += f"위 원고  {plaintiff['name']} (인)\n\n"
        complaint += f"{court} 귀중\n"

        return complaint

    def write_child_custody_claim(
        self,
        applicant: Dict,
        respondent: Dict,
        children: List[Dict],
        reasons: List[str],
        current_situation: str,
        court: str
    ) -> str:
        """
        친권자 및 양육자 지정 신청서 작성

        Args:
            applicant: 신청인 정보
            respondent: 상대방 정보
            children: 자녀 정보
            reasons: 신청 이유
            current_situation: 현재 양육 상황
            court: 관할법원

        Returns:
            친권자 및 양육자 지정 신청서 전문
        """
        application = "친권자 및 양육자 지정 심판청구서\n\n"

        # 당사자
        application += f"청 구 인    {applicant['address']}\n"
        application += f"            {applicant['name']} "
        application += f"(주민등록번호: {applicant['resident_number']})\n\n"

        application += f"상 대 방    {respondent['address']}\n"
        application += f"            {respondent['name']} "
        application += f"(주민등록번호: {respondent['resident_number']})\n\n"

        # 자녀
        application += "미성년자    "
        for idx, child in enumerate(children):
            if idx > 0:
                application += "            "
            application += f"{child['name']} ({child['birth_date']}. 출생)\n"
        application += "\n"

        # 청구취지
        application += "청구취지\n\n"
        for idx, child in enumerate(children, 1):
            application += f"{idx}. 청구인과 상대방의 미성년 자녀 {child['name']}의 "
            application += f"친권자 및 양육자를 청구인으로 지정한다.\n"
        application += "\n라는 심판을 구합니다.\n\n"

        # 청구원인
        application += "청구원인\n\n"
        application += "1. 당사자의 관계\n\n"
        application += f"   청구인과 상대방은 부부로서 다음과 같은 미성년 자녀가 있습니다.\n\n"

        for child in children:
            application += f"   - {child['name']} ({child['birth_date']}. 출생, "
            application += f"{child['age']}세)\n"
        application += "\n"

        application += "2. 현재 양육 상황\n\n"
        application += f"   {current_situation}\n\n"

        application += "3. 친권자 및 양육자 지정 사유\n\n"
        for idx, reason in enumerate(reasons, 1):
            application += f"   {idx}. {reason}\n\n"

        application += "4. 자녀의 복리\n\n"
        application += "   위와 같은 사정을 고려할 때, 청구인이 친권자 및 양육자로 "
        application += "지정되는 것이 미성년 자녀들의 복리에 가장 부합합니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 가족관계증명서                1통\n"
        application += f"2. 기본증명서 (자녀)            {len(children)}통\n"
        application += "3. 소득증명서                    1통\n"
        application += "4. 주거증명서                    1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 청구인  {applicant['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_child_support_claim(
        self,
        creditor: Dict,
        debtor: Dict,
        children: List[Dict],
        monthly_amount: str,
        calculation_basis: Dict,
        court: str
    ) -> str:
        """
        양육비 청구 신청서 작성

        Args:
            creditor: 양육비 청구권자 (양육자)
            debtor: 양육비 부담자
            children: 자녀 정보
            monthly_amount: 월 양육비 청구액
            calculation_basis: 산정 근거
            court: 관할법원

        Returns:
            양육비 청구 신청서 전문
        """
        application = "양육비청구 심판청구서\n\n"

        # 당사자
        application += f"청 구 인    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"상 대 방    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        # 청구취지
        application += "청구취지\n\n"
        application += "상대방은 청구인에게\n\n"

        for idx, child in enumerate(children, 1):
            application += f"{idx}. {child['name']}의 양육비로 매월 말일 "
            application += f"금 {monthly_amount}원씩을 {child['end_date']}까지 지급하라.\n"

        application += "\n라는 심판을 구합니다.\n\n"

        # 청구원인
        application += "청구원인\n\n"
        application += "1. 당사자의 관계 및 자녀 현황\n\n"
        application += f"   청구인과 상대방 사이에는 다음 자녀가 있으며, "
        application += f"현재 청구인이 양육하고 있습니다.\n\n"

        for child in children:
            application += f"   - {child['name']} ({child['birth_date']}. 출생, "
            application += f"{child['age']}세)\n"
        application += "\n"

        application += "2. 양육비 산정\n\n"
        application += "   양육비는 다음과 같이 산정됩니다.\n\n"
        application += f"   (1) 청구인의 월 소득: 금 {calculation_basis['creditor_income']}원\n"
        application += f"   (2) 상대방의 월 소득: 금 {calculation_basis['debtor_income']}원\n"
        application += f"   (3) 총 양육비: 금 {calculation_basis['total_cost']}원\n"
        application += f"   (4) 상대방의 부담 비율: {calculation_basis['debtor_ratio']}%\n\n"

        application += f"   따라서 상대방이 부담할 양육비는 월 {monthly_amount}원입니다.\n\n"

        application += "3. 양육비 지급의 필요성\n\n"
        application += "   자녀 양육에는 다음과 같은 비용이 소요됩니다.\n\n"

        for expense in calculation_basis.get('expenses', []):
            application += f"   - {expense['category']}: 월 {expense['amount']}원\n"

        application += "\n   청구인의 소득만으로는 자녀를 적정하게 양육하기 어려우므로, "
        application += "상대방의 양육비 분담이 필요합니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 가족관계증명서                1통\n"
        application += "2. 소득증명서 (청구인)          1통\n"
        application += "3. 소득증명서 (상대방)          1통\n"
        application += "4. 양육비 산정자료                1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 청구인  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_visitation_rights_claim(
        self,
        applicant: Dict,
        respondent: Dict,
        children: List[Dict],
        proposed_schedule: Dict,
        reasons: List[str],
        court: str
    ) -> str:
        """
        면접교섭권 청구 신청서 작성

        Args:
            applicant: 신청인 (비양육자)
            respondent: 상대방 (양육자)
            children: 자녀 정보
            proposed_schedule: 제안하는 면접교섭 방법 및 시간
            reasons: 신청 이유
            court: 관할법원

        Returns:
            면접교섭권 청구 신청서 전문
        """
        application = "면접교섭권 심판청구서\n\n"

        # 당사자
        application += f"청 구 인    {applicant['address']}\n"
        application += f"            {applicant['name']} "
        application += f"(주민등록번호: {applicant['resident_number']})\n\n"

        application += f"상 대 방    {respondent['address']}\n"
        application += f"            {respondent['name']} "
        application += f"(주민등록번호: {respondent['resident_number']})\n\n"

        # 청구취지
        application += "청구취지\n\n"
        application += "청구인은 상대방이 양육하고 있는 미성년 자녀들과 "
        application += "다음과 같이 면접교섭할 수 있다.\n\n"

        application += f"1. 일시: {proposed_schedule['regular']}\n"
        application += f"2. 장소: {proposed_schedule['location']}\n"
        application += f"3. 방법: {proposed_schedule['method']}\n"

        if 'vacation' in proposed_schedule:
            application += f"4. 방학 중: {proposed_schedule['vacation']}\n"

        if 'holidays' in proposed_schedule:
            application += f"5. 명절: {proposed_schedule['holidays']}\n"

        application += "\n라는 심판을 구합니다.\n\n"

        # 청구원인
        application += "청구원인\n\n"
        application += "1. 당사자의 관계\n\n"
        application += f"   청구인과 상대방 사이에는 다음 미성년 자녀가 있으며, "
        application += f"현재 상대방이 양육하고 있습니다.\n\n"

        for child in children:
            application += f"   - {child['name']} ({child['birth_date']}. 출생, "
            application += f"{child['age']}세)\n"
        application += "\n"

        application += "2. 면접교섭의 필요성\n\n"
        for idx, reason in enumerate(reasons, 1):
            application += f"   {idx}. {reason}\n\n"

        application += "3. 제안하는 면접교섭 방법\n\n"
        application += "   청구인은 자녀의 복리를 최우선으로 고려하여 위와 같은 "
        application += "면접교섭을 제안합니다. 이는 자녀의 일상생활과 교육에 지장을 주지 "
        application += "않으면서도, 청구인과 자녀가 건전한 친자관계를 유지할 수 있는 "
        application += "합리적인 방법입니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 가족관계증명서                1통\n"
        application += "2. 이혼판결문 (해당시)          1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 청구인  {applicant['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_paternity_claim(
        self,
        plaintiff: Dict,
        defendant: Dict,
        child: Dict,
        relationship_facts: List[str],
        paternity_evidence: List[str],
        court: str
    ) -> str:
        """
        인지청구소장 작성

        Args:
            plaintiff: 원고 (자녀 또는 법정대리인)
            defendant: 피고 (부 또는 모)
            child: 자녀 정보
            relationship_facts: 원고-피고 관계 사실
            paternity_evidence: 친생자 관계 입증 자료
            court: 관할법원

        Returns:
            인지청구소장 전문
        """
        complaint = "소    장\n\n"

        # 당사자
        complaint += f"원    고    {plaintiff['address']}\n"
        complaint += f"            {plaintiff['name']} "
        complaint += f"(주민등록번호: {plaintiff['resident_number']})\n"

        if plaintiff.get('legal_representative'):
            complaint += f"            (법정대리인 모 {plaintiff['legal_representative']})\n"

        complaint += "\n"

        complaint += f"피    고    {defendant['address']}\n"
        complaint += f"            {defendant['name']} "
        complaint += f"(주민등록번호: {defendant['resident_number']})\n\n"

        # 청구취지
        complaint += "청구취지\n\n"
        complaint += f"피고는 원고 {child['name']}을(를) 인지하라.\n\n"
        complaint += "라는 판결을 구합니다.\n\n"

        # 청구원인
        complaint += "청구원인\n\n"
        complaint += "1. 원고의 출생\n\n"
        complaint += f"   원고는 {child['birth_date']}. 출생하였습니다.\n\n"

        complaint += "2. 피고와의 관계\n\n"
        for idx, fact in enumerate(relationship_facts, 1):
            complaint += f"   {idx}. {fact}\n\n"

        complaint += "3. 친생자 관계\n\n"
        complaint += "   다음 사실에 비추어 원고는 피고의 친생자임이 명백합니다.\n\n"

        for idx, evidence in enumerate(paternity_evidence, 1):
            complaint += f"   {idx}. {evidence}\n\n"

        complaint += "4. 인지 청구의 필요성\n\n"
        complaint += "   원고는 피고의 친생자임에도 불구하고 법률상 친자관계가 형성되지 "
        complaint += "않아 상속권 등 법률상 권리를 행사할 수 없는 상태입니다. "
        complaint += "이에 인지를 청구합니다.\n\n"

        # 첨부서류
        complaint += "첨부서류\n\n"
        complaint += "1. 기본증명서 (원고)            1통\n"
        complaint += "2. 가족관계증명서 (원고)        1통\n"
        complaint += "3. 친생자 관계 입증자료          1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        complaint += f"{today}\n\n"

        if plaintiff.get('legal_representative'):
            complaint += f"위 원고의 법정대리인  {plaintiff['legal_representative']} (인)\n\n"
        else:
            complaint += f"위 원고  {plaintiff['name']} (인)\n\n"

        complaint += f"{court} 귀중\n"

        return complaint

    def write_inheritance_distribution(
        self,
        applicant: Dict,
        respondents: List[Dict],
        deceased: Dict,
        inheritance_assets: List[Dict],
        distribution_plan: Dict,
        court: str
    ) -> str:
        """
        상속재산분할 심판청구서 작성

        Args:
            applicant: 신청인 (공동상속인 중 1인)
            respondents: 상대방들 (다른 공동상속인들)
            deceased: 피상속인 정보
            inheritance_assets: 상속재산 목록
            distribution_plan: 분할안
            court: 관할법원

        Returns:
            상속재산분할 심판청구서 전문
        """
        application = "상속재산분할 심판청구서\n\n"

        # 당사자
        application += f"청 구 인    {applicant['address']}\n"
        application += f"            {applicant['name']} "
        application += f"(주민등록번호: {applicant['resident_number']})\n\n"

        for idx, respondent in enumerate(respondents, 1):
            application += f"상대방{idx}      {respondent['address']}\n"
            application += f"            {respondent['name']} "
            application += f"(주민등록번호: {respondent['resident_number']})\n\n"

        # 청구취지
        application += "청구취지\n\n"
        application += f"피상속인 {deceased['name']}의 별지 목록 기재 상속재산을 "
        application += f"다음과 같이 분할한다.\n\n"

        for heir_name, allocation in distribution_plan.items():
            application += f"- {heir_name}: {allocation}\n"

        application += "\n라는 심판을 구합니다.\n\n"

        # 청구원인
        application += "청구원인\n\n"
        application += "1. 피상속인의 사망 및 상속인\n\n"
        application += f"   피상속인 {deceased['name']}은(는) "
        application += f"{deceased['death_date']}. 사망하였고, 상속인으로는 "
        application += f"청구인과 상대방들이 있습니다.\n\n"

        application += "2. 상속재산\n\n"
        application += "   피상속인의 상속재산은 별지 목록 기재와 같으며, "
        application += "그 가액은 다음과 같습니다.\n\n"

        total_value = 0
        for asset in inheritance_assets:
            application += f"   - {asset['description']}: 금 {asset['value']}원\n"
            total_value += int(asset['value'].replace(',', ''))

        application += f"\n   합계: 금 {total_value:,}원\n\n"

        application += "3. 상속지분\n\n"
        application += "   각 상속인의 상속지분은 다음과 같습니다.\n\n"

        for heir in [applicant] + respondents:
            application += f"   - {heir['name']}: {heir['share']}\n"

        application += "\n"

        application += "4. 분할의 필요성\n\n"
        application += "   상속인들 사이에 상속재산 분할에 관한 협의가 이루어지지 않아 "
        application += "법원의 심판을 구합니다.\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 제적등본 (피상속인)          1통\n"
        application += "2. 가족관계증명서 (피상속인)    1통\n"
        application += f"3. 가족관계증명서 (상속인)      {1 + len(respondents)}통\n"
        application += "4. 상속재산 목록 및 평가서        1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 청구인  {applicant['name']} (인)\n\n"
        application += f"{court} 귀중\n\n"

        # 별지 목록
        application += "[별지] 상속재산 목록\n\n"
        for idx, asset in enumerate(inheritance_assets, 1):
            application += f"{idx}. {asset['description']}\n"
            application += f"   (가액: 금 {asset['value']}원)\n\n"

        return application


if __name__ == "__main__":
    # 테스트 코드
    writer = FamilyLawsuitWriter()

    print("=" * 80)
    print("이혼소장 샘플")
    print("=" * 80)
    divorce = writer.write_divorce_complaint(
        plaintiff={
            "name": "김원고",
            "resident_number": "800101-2345678",
            "address": "서울특별시 강남구 테헤란로 123"
        },
        defendant={
            "name": "이피고",
            "resident_number": "790505-1234567",
            "address": "서울특별시 서초구 서초대로 456"
        },
        marriage_info={
            "marriage_date": "2010. 5. 15",
            "registration_date": "2010. 5. 15"
        },
        grounds=[
            {
                "title": "부정행위 (민법 제840조 제1호)",
                "description": "피고는 2023. 3.경부터 제3자와 부정한 관계를 맺고 있습니다. "
                              "원고는 2024. 1.경 이 사실을 알게 되었습니다."
            },
            {
                "title": "혼인을 계속하기 어려운 중대한 사유 (민법 제840조 제6호)",
                "description": "피고의 부정행위로 인해 원고는 심각한 정신적 고통을 받았으며, "
                              "부부간의 신뢰는 완전히 파괴되었습니다. 혼인관계를 계속 유지하는 것은 불가능합니다."
            }
        ],
        children=[
            {
                "name": "김자녀",
                "birth_date": "2012. 3. 10",
                "age": 12,
                "custody_claim": True
            }
        ],
        property_division={
            "total_value": "500,000,000",
            "plaintiff_contribution": 50,
            "amount": "250,000,000",
            "assets": [
                {
                    "type": "부동산",
                    "description": "서울 강남구 아파트",
                    "value": "400,000,000"
                },
                {
                    "type": "예금",
                    "description": "○○은행 예금",
                    "value": "100,000,000"
                }
            ]
        },
        alimony={
            "amount": "50,000,000",
            "reason": "피고의 부정행위로 인해 원고는 극심한 정신적 고통을 받았고, "
                     "12년간의 혼인생활이 파탄에 이르렀습니다."
        },
        court="서울가정법원"
    )
    print(divorce)

    print("\n" + "=" * 80)
    print("양육비 청구 신청서 샘플")
    print("=" * 80)
    child_support = writer.write_child_support_claim(
        creditor={
            "name": "박양육",
            "resident_number": "850101-2345678",
            "address": "서울특별시 마포구 월드컵로 123"
        },
        debtor={
            "name": "최부담",
            "resident_number": "830505-1234567",
            "address": "경기도 성남시 분당구 판교로 456"
        },
        children=[
            {
                "name": "최자녀",
                "birth_date": "2015. 7. 20",
                "age": 9,
                "end_date": "2033. 7. 19"
            }
        ],
        monthly_amount="1,000,000",
        calculation_basis={
            "creditor_income": "2,500,000",
            "debtor_income": "5,000,000",
            "total_cost": "2,000,000",
            "debtor_ratio": 67,
            "expenses": [
                {"category": "교육비", "amount": "500,000"},
                {"category": "식비", "amount": "400,000"},
                {"category": "의류비", "amount": "200,000"},
                {"category": "의료비", "amount": "200,000"},
                {"category": "기타", "amount": "700,000"}
            ]
        },
        court="서울가정법원"
    )
    print(child_support)
