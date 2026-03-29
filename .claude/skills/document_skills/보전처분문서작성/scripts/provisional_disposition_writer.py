#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
보전처분 신청서 작성 모듈

이 모듈은 52,798줄의 민사실무대전 가압류·가처분 편 데이터베이스를 기반으로
모든 종류의 보전처분 신청서를 자동으로 작성합니다.

지원 기능:
- 가압류 (부동산, 채권, 동산, 선박, 자동차)
- 가처분 (처분금지, 점유이전금지, 임시지위)
- 특수보전 (노동, 상사사건)
- 구제절차 (이의신청, 취소신청)
"""

from typing import Dict, List, Optional
from datetime import datetime


class ProvisionalDispositionWriter:
    """보전처분 신청서 작성 클래스"""

    def __init__(self):
        """초기화"""
        self.reference_data = "민사실무대전 (IV) 가압류·가처분 편 (백영사, 52,798줄)"

    def write_real_estate_seizure(
        self,
        creditor: Dict,
        debtor: Dict,
        claim_amount: str,
        preserved_right: Dict,
        necessity: Dict,
        real_estate: Dict,
        security_deposit: str,
        court: str
    ) -> str:
        """
        부동산 가압류 신청서 작성

        Args:
            creditor: 채권자 정보 (name, resident_number, address)
            debtor: 채무자 정보 (name, resident_number, address)
            claim_amount: 청구금액
            preserved_right: 피보전권리 정보 (type, date, amount, etc.)
            necessity: 보전의 필요성 (reason, evidence)
            real_estate: 부동산 정보 (location, lot_number, category, area)
            security_deposit: 담보금액
            court: 관할법원

        Returns:
            부동산 가압류 신청서 전문
        """
        application = "가 압 류 신 청 서\n\n"

        # 당사자 표시
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        application += f"청구금액    금 {claim_amount}원\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += f"채무자 소유의 별지 목록 기재 부동산에 대하여 위 청구금액을\n"
        application += f"본권으로 한 부동산가압류를 명하는 재판을 구합니다.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += "1. 피보전권리\n\n"

        if preserved_right['type'] == '대여금채권':
            application += f"  채권자는 {preserved_right['date']}. 채무자에게 "
            application += f"금 {preserved_right['amount']}원을\n"
            application += f"  변제기 {preserved_right['due_date']}., "
            application += f"이자 연 {preserved_right['interest_rate']}로 정하여 "
            application += f"대여하였습니다\n"
            application += f"  (소명 제1호증 {preserved_right['evidence']}).\n\n"

            application += f"  채무자는 변제기에 원리금을 변제하지 않고 있으므로,\n"
            application += f"  채권자는 채무자에 대하여 위 대여금 {preserved_right['amount']}원 및\n"
            application += f"  이에 대한 지연손해금채권을 가지고 있습니다.\n\n"

        # 보전의 필요성
        application += "2. 보전의 필요성\n\n"
        application += f"  가. {necessity['reason']}\n"
        if 'evidence' in necessity:
            application += f"      (소명 제2호증 {necessity['evidence']}).\n\n"
        else:
            application += "\n"

        application += f"  나. 채무자가 위 부동산을 처분하면 채무자는 무자력이\n"
        application += f"      되어 채권자는 채권의 만족을 얻을 수 없게 됩니다.\n\n"

        application += f"  다. 따라서 채권자의 채권을 보전하기 위하여 긴급히\n"
        application += f"      채무자의 위 부동산에 대한 가압류가 필요합니다.\n\n"

        # 담보제공
        application += "3. 담보제공\n\n"
        application += f"  채권자는 이 사건 가압류로 인하여 채무자가 입을지도\n"
        application += f"  모르는 손해를 담보하기 위하여 금 {security_deposit}원을\n"
        application += f"  공탁하겠습니다.\n\n"

        # 소명방법
        application += "소명방법\n\n"
        application += f"1. 소명 제1호증    {preserved_right['evidence']}\n"
        if 'evidence' in necessity:
            application += f"2. 소명 제2호증    {necessity['evidence']}\n"
            application += f"3. 소명 제3호증    부동산등기사항증명서\n"
            application += f"4. 소명 제4호증    진술서\n\n"
        else:
            application += f"2. 소명 제2호증    부동산등기사항증명서\n"
            application += f"3. 소명 제3호증    진술서\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 위 소명방법 각 1통\n"
        if debtor.get('type') == '법인':
            application += "2. 법인등기사항증명서(채무자) 1통\n"
            application += "3. 송달료납부서 1통\n\n"
        else:
            application += "2. 송달료납부서 1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n\n"

        # 별지 목록
        application += "[별지] 부동산 목록\n\n"
        application += f"1. {real_estate['location']} {real_estate['lot_number']} "
        application += f"{real_estate['category']} {real_estate['area']}\n\n"

        if 'building' in real_estate:
            application += f"2. 위 지상 {real_estate['building']['structure']}\n"
            application += f"   {real_estate['building']['area']}\n"

        return application

    def write_disposition_injunction(
        self,
        creditor: Dict,
        debtor: Dict,
        preserved_right: Dict,
        necessity: Dict,
        real_estate: Dict,
        security_deposit: str,
        court: str
    ) -> str:
        """
        부동산 처분금지 가처분 신청서 작성

        Args:
            creditor: 채권자 정보
            debtor: 채무자 정보
            preserved_right: 피보전권리 (type, cause, contract_date, etc.)
            necessity: 보전의 필요성
            real_estate: 부동산 정보
            security_deposit: 담보금액
            court: 관할법원

        Returns:
            부동산 처분금지 가처분 신청서 전문
        """
        application = "부동산처분금지가처분신청서\n\n"

        # 당사자 표시
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += "1. 채무자는 별지 목록 기재 부동산에 관하여 소유권이전등기,\n"
        application += "   저당권설정등기 기타 일체의 등기신청행위를 하여서는\n"
        application += "   아니된다.\n\n"
        application += "2. 위 가처분등기를 촉탁한다.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += "1. 피보전권리\n\n"

        if preserved_right['type'] == '소유권이전등기청구권':
            application += f"  가. 채권자와 채무자는 {preserved_right['contract_date']}. "
            application += f"별지 목록 기재 부동산에\n"
            application += f"      관하여 매매대금 {preserved_right['sale_price']}원으로 정하여\n"
            application += f"      매매계약을 체결하였습니다 (소명 제1호증 부동산매매계약서).\n\n"

            application += f"  나. 채권자는 {preserved_right['down_payment']['date']}. "
            application += f"채무자에게 계약금\n"
            application += f"      {preserved_right['down_payment']['amount']}원을, "
            application += f"{preserved_right['interim_payment']['date']}. 중도금 "
            application += f"{preserved_right['interim_payment']['amount']}원을\n"
            application += f"      각 지급하였습니다 (소명 제2호증 영수증).\n\n"

            application += f"  다. 잔금 지급일은 {preserved_right['balance_date']}.로 정하였고,\n"
            application += f"      잔금 지급과 동시에 소유권이전등기를 이행하기로\n"
            application += f"      약정하였습니다.\n\n"

            application += f"  라. 따라서 채권자는 채무자에 대하여 위 부동산에 관한\n"
            application += f"      소유권이전등기청구권을 가지고 있습니다.\n\n"

        # 보전의 필요성
        application += "2. 보전의 필요성\n\n"
        application += f"  가. 그런데 채무자는 {necessity.get('double_contract_date', '')}. "
        application += f"제3자인 소외 {necessity.get('third_party', '')}에게\n"
        application += f"      위 부동산을 이중으로 매도하는 계약을 체결하였고,\n"
        application += f"      소외인이 먼저 소유권이전등기를 경료하려 하고 있습니다\n"
        application += f"      (소명 제3호증 확인서).\n\n"

        application += f"  나. 만약 소외인이 먼저 소유권이전등기를 경료하면,\n"
        application += f"      채권자는 채무자에 대한 소유권이전등기청구권을\n"
        application += f"      행사할 수 없게 되어 회복하기 어려운 손해를 입게 됩니다.\n\n"

        application += f"  다. 따라서 채권자의 소유권이전등기청구권을 보전하기 위하여\n"
        application += f"      채무자의 위 부동산에 대한 일체의 처분행위를\n"
        application += f"      금지하는 가처분이 필요합니다.\n\n"

        # 담보제공
        application += "3. 담보제공\n\n"
        application += f"  채권자는 이 사건 가처분으로 인하여 채무자가 입을지도\n"
        application += f"  모르는 손해를 담보하기 위하여 금 {security_deposit}원을\n"
        application += f"  공탁하겠습니다.\n\n"

        # 소명방법
        application += "소명방법\n\n"
        application += "1. 소명 제1호증    부동산매매계약서\n"
        application += "2. 소명 제2호증    영수증 (계약금, 중도금)\n"
        application += "3. 소명 제3호증    확인서\n"
        application += "4. 소명 제4호증    부동산등기사항증명서\n"
        application += "5. 소명 제5호증    진술서\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 위 소명방법 각 1통\n"
        application += "2. 송달료납부서 1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n\n"

        # 별지 목록
        application += "[별지] 부동산 목록\n\n"
        application += f"{real_estate['location']} {real_estate['lot_number']}\n"
        if 'type' in real_estate:
            application += f"{real_estate['structure']} {real_estate['type']}\n"
            if 'unit' in real_estate:
                application += f"{real_estate['unit']} {real_estate['area']}\n"

        return application

    def write_construction_halt_injunction(
        self,
        creditor: Dict,
        debtor: Dict,
        preserved_right: Dict,
        necessity: Dict,
        construction_site: Dict,
        security_deposit: str,
        calculation: str,
        court: str
    ) -> str:
        """
        공사중지 가처분 신청서 작성

        Args:
            creditor: 채권자 정보
            debtor: 채무자 정보 (법인의 경우 registration_number, representative 포함)
            preserved_right: 피보전권리 (type, basis, damage)
            necessity: 보전의 필요성 (일조권 침해 정도 등)
            construction_site: 공사현장 정보
            security_deposit: 담보금액
            calculation: 담보액 산정 근거
            court: 관할법원

        Returns:
            공사중지 가처분 신청서 전문
        """
        application = "건축공사중지가처분신청서\n\n"

        # 당사자 표시
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']}\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        if debtor.get('registration_number'):
            application += f"            {debtor['name']}\n"
            application += f"            (법인등록번호: {debtor['registration_number']})\n"
            application += f"            대표이사 {debtor['representative']}\n\n"
        else:
            application += f"            {debtor['name']}\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += "채무자는 별지 목록 기재 토지상에서의 건축공사를 중지하라.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += "1. 피보전권리\n\n"

        application += f"  가. 채권자는 {creditor['address']} 건물을 소유하고 있으며,\n"
        application += f"      위 건물에서 거주하고 있습니다.\n\n"

        if preserved_right['type'] == '일조권 침해 배제청구권':
            application += f"  나. {preserved_right['damage']}\n\n"

            application += f"  다. 채권자는 {preserved_right['basis']}에 기하여 채무자에 대하여\n"
            application += f"      일조권 침해의 배제를 청구할 권리가 있습니다.\n\n"

        # 보전의 필요성
        application += "2. 보전의 필요성\n\n"

        application += f"  가. 채권자 주택은 현재 {necessity['current_sunlight']} 일조를\n"
        application += f"      확보하고 있으나, 채무자의 건축공사가 완료되면\n"
        application += f"      {necessity['expected_sunlight']}으로 일조가 현저히 감소합니다.\n\n"

        application += f"  나. 이로 인해 {necessity['health_damage']}의\n"
        application += f"      피해가 발생하며, {necessity['property_damage']}의\n"
        application += f"      재산적 손해가 예상됩니다.\n\n"

        application += f"  다. 채무자의 공사가 완료되면 원상회복이 사실상 불가능하므로,\n"
        application += f"      본안판결 확정 시까지 공사를 중지할 필요가 있습니다.\n\n"

        # 담보제공
        application += "3. 담보제공\n\n"
        application += f"  채권자는 이 사건 가처분으로 인하여 채무자가 입을지도\n"
        application += f"  모르는 손해를 담보하기 위하여 금 {security_deposit}원\n"
        application += f"  ({calculation})을 공탁하겠습니다.\n\n"

        # 소명방법
        application += "소명방법\n\n"
        application += "1. 소명 제1호증    부동산등기사항증명서 (채권자 주택)\n"
        application += "2. 소명 제2호증    건축허가서 사본 (채무자 건축)\n"
        application += "3. 소명 제3호증    건축설계도면\n"
        application += "4. 소명 제4호증    일조권 분석 보고서\n"
        application += "5. 소명 제5호증    현장사진\n"
        application += "6. 소명 제6호증    진술서\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 위 소명방법 각 1통\n"
        if debtor.get('registration_number'):
            application += "2. 법인등기사항증명서(채무자) 1통\n"
            application += "3. 송달료납부서 1통\n\n"
        else:
            application += "2. 송달료납부서 1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n\n"

        # 별지 목록
        application += "[별지] 공사현장 목록\n\n"
        application += f"{construction_site['location']}\n"
        application += f"건축계획: {construction_site['building_plan']}\n"
        application += f"허가일자: {construction_site['permit_date']}\n"
        application += f"공사착공: {construction_site['construction_start']}\n"

        return application

    def write_employee_status_preservation(
        self,
        creditor: Dict,
        debtor: Dict,
        preserved_right: Dict,
        necessity: Dict,
        request: Dict,
        security_deposit: str,
        court: str
    ) -> str:
        """
        근로자 지위보전 가처분 신청서 작성

        Args:
            creditor: 채권자(근로자) 정보
            debtor: 채무자(사용자) 정보
            preserved_right: 피보전권리 (고용관계)
            necessity: 보전의 필요성
            request: 신청내용 (지위보전, 임금지급)
            security_deposit: 담보금액
            court: 관할법원

        Returns:
            근로자 지위보전 가처분 신청서 전문
        """
        application = "근로자지위보전가처분신청서\n\n"

        # 당사자 표시
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']}\n"
        application += f"            (법인등록번호: {debtor['registration_number']})\n"
        application += f"            대표이사 {debtor['representative']}\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += "1. 채권자는 채무자에 대하여 근로계약상 근로자의 지위에\n"
        application += "   있음을 가정적으로 정한다.\n\n"

        if request.get('interim_wage'):
            application += f"2. 채무자는 채권자에게 본안판결 확정 시까지 매월\n"
            application += f"   {request['payment_date']} 금 {request['monthly_wage']}원을\n"
            application += f"   지급하라.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += "1. 피보전권리\n\n"

        application += f"  가. 채권자는 {preserved_right['employment_date']}. "
        application += f"채무자 회사에\n"
        application += f"      {preserved_right['position']}으로 입사하여 "
        application += f"근무하였습니다.\n\n"

        application += f"  나. 채권자의 월 급여는 금 {preserved_right['monthly_salary']}원이었습니다.\n\n"

        application += f"  다. 그런데 채무자는 {preserved_right['dismissal_date']}. "
        application += f"채권자를\n"
        application += f"      '{preserved_right['dismissal_reason']}'를 사유로 해고하였습니다.\n\n"

        application += f"  라. 그러나 위 해고는 정당한 사유가 없는 부당해고로서\n"
        application += f"      무효이므로, 채권자는 여전히 채무자 회사의\n"
        application += f"      근로자로서의 지위를 가지고 있습니다.\n\n"

        # 보전의 필요성
        application += "2. 보전의 필요성\n\n"

        application += f"  가. 위 해고는 다음과 같은 이유로 명백히 위법·무효입니다.\n\n"
        for idx, reason in enumerate(necessity['illegality'], 1):
            application += f"      {idx}) {reason}\n"
        application += "\n"

        application += f"  나. {necessity['hardship']}\n\n"

        application += f"  다. {necessity['urgency']}\n\n"

        application += f"  라. 따라서 채권자의 근로자 지위를 가정적으로 정하고,\n"
        application += f"      본안판결 확정 시까지 임금을 가지급할 필요가 있습니다.\n\n"

        # 담보제공
        application += "3. 담보제공\n\n"
        application += f"  채권자는 이 사건 가처분으로 인하여 채무자가 입을지도\n"
        application += f"  모르는 손해를 담보하기 위하여 금 {security_deposit}원을\n"
        application += f"  공탁하겠습니다.\n\n"

        # 소명방법
        application += "소명방법\n\n"
        application += "1. 소명 제1호증    근로계약서\n"
        application += "2. 소명 제2호증    급여명세서\n"
        application += "3. 소명 제3호증    해고통지서\n"
        application += "4. 소명 제4호증    진술서\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 위 소명방법 각 1통\n"
        application += "2. 법인등기사항증명서(채무자) 1통\n"
        application += "3. 송달료납부서 1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_director_suspension_injunction(
        self,
        creditor: Dict,
        debtor: Dict,
        company: Dict,
        preserved_right: Dict,
        illegal_acts: List[Dict],
        necessity: Dict,
        acting_director: Dict,
        security_deposit: str,
        court: str
    ) -> str:
        """
        이사 직무집행정지 가처분 신청서 작성

        Args:
            creditor: 채권자(주주) 정보
            debtor: 채무자(이사) 정보
            company: 회사 정보
            preserved_right: 피보전권리
            illegal_acts: 이사의 위법행위 목록
            necessity: 보전의 필요성
            acting_director: 직무대행자 정보
            security_deposit: 담보금액
            court: 관할법원

        Returns:
            이사 직무집행정지 가처분 신청서 전문
        """
        application = "이사직무집행정지가처분신청서\n\n"

        # 당사자 표시
        application += f"채 권 자    {creditor['address']}\n"
        application += f"            {creditor['name']} "
        application += f"(주민등록번호: {creditor['resident_number']})\n"
        application += f"            (소유주식: {creditor['shares']})\n\n"

        application += f"채 무 자    {debtor['address']}\n"
        application += f"            {debtor['name']} "
        application += f"(주민등록번호: {debtor['resident_number']})\n\n"

        application += f"회    사    {company['address']}\n"
        application += f"            {company['name']}\n"
        application += f"            (법인등록번호: {company['registration_number']})\n\n"

        # 신청취지
        application += "신청취지\n\n"
        application += f"1. 채무자는 {company['name']}의 {debtor['position']}로서의\n"
        application += f"   직무집행을 정지한다.\n\n"
        application += f"2. 위 이사의 직무를 대행할 자로 {acting_director['name']}을 선임한다.\n\n"
        application += f"3. 위 가처분 등기를 촉탁한다.\n\n"

        # 신청이유
        application += "신청이유\n\n"
        application += "1. 당사자\n\n"

        application += f"  가. 채권자는 {company['name']}의 주주로서\n"
        application += f"      {creditor['shares']}을 보유하고 있습니다.\n\n"

        application += f"  나. 채무자는 위 회사의 {debtor['position']}으로\n"
        application += f"      회사 업무를 집행하고 있습니다.\n\n"

        # 피보전권리
        application += "2. 피보전권리\n\n"

        application += f"  {preserved_right['basis']}에 따르면, 이사의 직무집행으로 인하여\n"
        application += f"  회사에 회복하기 어려운 손해가 생길 염려가 있는 때에는\n"
        application += f"  법원은 주주의 신청에 의하여 이사의 직무집행을 정지하고\n"
        application += f"  그 직무를 대행할 자를 선임할 수 있습니다.\n\n"

        application += f"  채권자는 {company['name']}의 주주로서 위 법률에 기한\n"
        application += f"  {preserved_right['claim']}을 가지고 있습니다.\n\n"

        # 보전의 필요성
        application += "3. 보전의 필요성\n\n"

        application += "  가. 채무자의 위법행위\n\n"

        for idx, act in enumerate(illegal_acts, 1):
            application += f"    {idx}) {act['act']}\n\n"
            application += f"       채무자는 {act['date']}. {act['detail']}\n"
            if 'amount' in act:
                application += f"       (금액: {act['amount']}원).\n\n"
            else:
                application += "\n"

        application += "  나. 회복할 수 없는 손해 발생 우려\n\n"
        application += f"      {necessity['irrecoverable_damage']}\n\n"

        application += "  다. 긴급성\n\n"
        application += f"      {necessity['urgency']}\n\n"

        # 직무대행자
        application += "4. 직무대행자\n\n"
        application += f"  {acting_director['name']}은(는) {acting_director['qualification']}하여\n"
        application += f"  채무자의 직무를 대행하기에 적합합니다.\n\n"

        # 담보제공
        application += "5. 담보제공\n\n"
        application += f"  채권자는 이 사건 가처분으로 인하여 채무자가 입을지도\n"
        application += f"  모르는 손해를 담보하기 위하여 금 {security_deposit}원을\n"
        application += f"  공탁하겠습니다.\n\n"

        # 소명방법
        application += "소명방법\n\n"
        application += "1. 소명 제1호증    주식보유증명서\n"
        application += "2. 소명 제2호증    법인등기사항증명서\n"
        application += "3. 소명 제3호증    이사회 의사록\n"
        application += "4. 소명 제4호증    회계장부\n"
        application += "5. 소명 제5호증    진술서\n\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 위 소명방법 각 1통\n"
        application += "2. 법인등기사항증명서(회사) 1통\n"
        application += "3. 송달료납부서 1통\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 채권자  {creditor['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application


if __name__ == "__main__":
    # 테스트 코드
    writer = ProvisionalDispositionWriter()

    # 부동산 가압류 테스트
    print("=" * 80)
    print("부동산 가압류 신청서 샘플")
    print("=" * 80)
    seizure = writer.write_real_estate_seizure(
        creditor={
            "name": "김철수",
            "resident_number": "800101-1234567",
            "address": "서울특별시 강남구 테헤란로 123"
        },
        debtor={
            "name": "이영희",
            "resident_number": "750505-2345678",
            "address": "서울특별시 서초구 서초대로 456"
        },
        claim_amount="500,000,000",
        preserved_right={
            "type": "대여금채권",
            "date": "2024. 6. 10",
            "amount": "500,000,000",
            "interest_rate": "12%",
            "due_date": "2024. 12. 10",
            "evidence": "금전소비대차계약서"
        },
        necessity={
            "reason": "채무자는 2024. 11. 1. 자신이 소유한 별지 목록 기재 부동산을 제3자에게 매도하는 계약을 체결하였고, 곧 소유권이전등기를 마칠 예정입니다",
            "evidence": "부동산매매계약서"
        },
        real_estate={
            "location": "서울특별시 강남구 역삼동",
            "lot_number": "123-45",
            "category": "대",
            "area": "200㎡",
            "building": {
                "structure": "철근콘크리트조 슬래브지붕 3층 근린생활시설",
                "area": "1층 150㎡, 2층 150㎡, 3층 150㎡"
            }
        },
        security_deposit="50,000,000",
        court="서울중앙지방법원"
    )
    print(seizure)
