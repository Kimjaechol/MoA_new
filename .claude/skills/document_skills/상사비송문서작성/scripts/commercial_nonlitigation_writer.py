"""
상사비송문서작성 API

상사비송 신청서를 자동으로 작성하는 Python API입니다.
회사 비송사건(검사인 선임·주식매수가액 결정·회사 해산명령·직무대행자 선임),
사채 비송사건, 회사 청산, 법인·신탁 비송사건 신청서 작성을 지원합니다.

참조: 상사비송사건실무 (김종호 편저, 백영사, 45,923줄)
"""

from datetime import datetime
from typing import Dict, List, Optional


class CommercialNonlitigationWriter:
    """상사비송 신청서 작성 클래스"""

    def __init__(self):
        self.reference_data = "상사비송사건실무 (김종호 편저, 백영사, 45,923줄)"
        self.version = "1.0.0"

    def write_inspector_appointment_application(
        self,
        applicant: Dict,
        company: Dict,
        inspection_purpose: str,
        inspection_target: Dict,
        inspection_reason: str,
        urgency: str,
        court: str
    ) -> str:
        """
        검사인 선임 신청서 작성

        Args:
            applicant: 신청인 정보
            company: 회사 정보
            inspection_purpose: 검사 목적
            inspection_target: 검사 대상
            inspection_reason: 검사 사유
            urgency: 긴급성
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        doc = f"""검사인 선임 신청서

신  청  인  {applicant['name']}
            {applicant['address']}
            ({applicant.get('position', '발기인')})

피 신 청 인  {company['name']}
            {company['address']}
            대표이사  {company['representative']}

{inspection_purpose} 검사인 선임 신청

신 청 취 지

피신청인의 {inspection_purpose}에 대하여 검사인을 선임하여 주시기 바랍니다.

신 청 이 유

1. 피신청인은 {company.get('establishment_date', '20○○. ○. ○.')}
   설립등기를 마친 주식회사입니다.

2. 신청인은 피신청인의 {applicant.get('position', '발기인')}으로서
   다음과 같이 {inspection_purpose}을(를) 하였습니다.

   가. {inspection_purpose} 목적물: {inspection_target['description']}
   나. 가액: 금 {inspection_target['claimed_value']:,}원
   다. 배정주식수: {inspection_target.get('shares', '')}

3. {inspection_reason}

4. {urgency}

소 명 방 법

갑 제1호증     정관
갑 제2호증     {inspection_purpose} 계약서
갑 제3호증     등기부등본
갑 제4호증     감정평가서

첨 부 서 류

1. 소명방법            각 1통
1. 신청서부본          1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

신청인  {applicant['name']}

{court} 귀중
"""
        return doc

    def write_share_purchase_price_decision_application(
        self,
        applicant: Dict,
        company: Dict,
        shares_info: Dict,
        claimed_price: int,
        company_offered_price: int,
        basis_of_price: str,
        case_type: str,
        court: str
    ) -> str:
        """
        주식매수가격 결정 신청서 작성

        Args:
            applicant: 신청인 (주주)
            company: 회사 정보
            shares_info: 주식 정보
            claimed_price: 주장 가액 (1주당)
            company_offered_price: 회사 제시 가액
            basis_of_price: 가액 산정 근거
            case_type: 사건 유형
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        total_claimed = shares_info['quantity'] * claimed_price
        total_offered = shares_info['quantity'] * company_offered_price

        doc = f"""주식매수가격 결정 신청서

신  청  인  {applicant['name']}
            {applicant['address']}
            (주주, {shares_info['type']} {shares_info['quantity']:,}주 보유)

피 신 청 인  {company['name']}
            {company['address']}
            대표이사  {company['representative']}

주식매수가격 결정 신청

신 청 취 지

신청인이 보유한 피신청인 발행 주식 {shares_info['quantity']:,}주의 매수가격을
결정하여 주시기 바랍니다.

신 청 이 유

1. 신청인은 피신청인 회사의 주주로서 {shares_info['type']} {shares_info['quantity']:,}주
   (발행주식 총수의 {applicant.get('ownership_ratio', '')} %)를 보유하고 있습니다.

2. {case_type} 사건의 경위

   {applicant.get('background', '')}

3. 신청인은 주식매수를 청구하였으나, 다음과 같이 매수가격에 대한
   협의가 이루어지지 않았습니다.

   가. 신청인 주장 가격:
       - 1주당 금 {claimed_price:,}원
       - 총액 금 {total_claimed:,}원
       - 산정 근거: {basis_of_price}

   나. 피신청인 제시 가격:
       - 1주당 금 {company_offered_price:,}원
       - 총액 금 {total_offered:,}원

4. 상법 제374조의2 제4항에 따라 법원에 매수가격의 결정을 구합니다.

소 명 방 법

갑 제1호증     주주명부 등본
갑 제2호증     주주총회 의사록
갑 제3호증     주식매수청구서
갑 제4호증     감정평가서
갑 제5호증     재무제표 (최근 3년)

첨 부 서 류

1. 소명방법            각 1통
1. 법인등기부등본      1통
1. 신청서부본          1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

신청인  {applicant['name']}

{court} 귀중
"""
        return doc

    def write_acting_officer_appointment_application(
        self,
        applicant: Dict,
        company: Dict,
        officer_type: str,
        vacancy_reason: str,
        candidate: Optional[Dict],
        urgency: str,
        court: str
    ) -> str:
        """
        직무대행자 선임 신청서 작성

        Args:
            applicant: 신청인
            company: 회사 정보
            officer_type: 직무대행자 유형
            vacancy_reason: 결원 사유
            candidate: 직무대행자 후보
            urgency: 긴급성
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        candidate_info = ""
        if candidate:
            candidate_info = f"{candidate['name']}(주민등록번호: {candidate.get('resident_number', '○○○○○○-○○○○○○○')})를"
        else:
            candidate_info = "적합한 자를"

        doc = f"""일시 {officer_type} 직무대행자 선임 신청서

신  청  인  {applicant['name']}
            {applicant['address']}
            ({applicant.get('position', '이사')})

피 신 청 인  {company['name']}
            {company['address']}

일시 {officer_type} 직무대행자 선임 신청

신 청 취 지

피신청인 회사의 일시 {officer_type} 직무대행자로 {candidate_info}
선임하여 주시기 바랍니다.

신 청 이 유

1. 피신청인의 {officer_type} 결원 경위

   {vacancy_reason}

2. 긴급한 업무 처리의 필요성

   {urgency}

3. 상법 제386조(제408조의2)에 따라 법원에 일시 {officer_type}
   직무대행자 선임을 신청합니다.

"""

        if candidate:
            doc += f"""4. {candidate['name']}은(는) {candidate.get('qualification', '업무에 정통하고 이해관계자들의 신뢰를 받고 있어')}
   직무대행자로 적합합니다.

"""

        doc += f"""소 명 방 법

갑 제1호증     법인등기부등본
갑 제2호증     {officer_type} 사임서 (또는 결원 증빙)
갑 제3호증     정관
갑 제4호증     긴급 업무 소명자료

첨 부 서 류

1. 소명방법            각 1통
1. 신청서부본          1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

신청인  {applicant['name']}

{court} 귀중
"""
        return doc

    def write_liquidator_appointment_application(
        self,
        applicant: Dict,
        company: Dict,
        dissolution_reason: str,
        candidate: Optional[Dict],
        company_assets: Dict,
        urgency: str,
        court: str
    ) -> str:
        """
        청산인 선임 신청서 작성

        Args:
            applicant: 신청인
            company: 회사 정보 (청산 중)
            dissolution_reason: 해산 사유
            candidate: 청산인 후보
            company_assets: 회사 재산 현황
            urgency: 긴급성
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        candidate_info = ""
        if candidate:
            candidate_info = f"{candidate.get('title', '변호사')} {candidate['name']}({candidate['address']})을"
        else:
            candidate_info = "적합한 자를"

        doc = f"""청산인 선임 신청서

신  청  인  {applicant['name']}
            {applicant['address']}
            ({applicant.get('position', '주주')})

피 신 청 인  {company['name']}
            {company['address']}
            (해산등기 완료)

청산인 선임 신청

신 청 취 지

피신청인의 청산인으로 {candidate_info} 선임하여 주시기 바랍니다.

신 청 이 유

1. 피신청인은 {dissolution_reason}

2. 청산인 부재 사유

   피신청인의 정관에는 청산인에 관한 규정이 없고, 주주총회에서도
   청산인을 선임하지 못하였습니다. 또한 법정 청산인(이사)도 없는
   상태입니다.

3. 피신청인의 재산 현황

   가. 자산
       {company_assets.get('assets', '')}

   나. 부채
       {company_assets.get('liabilities', '')}

4. 청산의 긴급성

   {urgency}

"""

        if candidate:
            doc += f"""5. {candidate.get('title', '변호사')} {candidate['name']}은(는) {candidate.get('qualification', '청산 업무에 경험이 있고 이해관계자들의 동의를 얻어')}
   청산인 후보로 적합합니다.

"""

        doc += f"""소 명 방 법

갑 제1호증     법인등기부등본
갑 제2호증     해산 주주총회 의사록
갑 제3호증     재산목록
갑 제4호증     대차대조표
갑 제5호증     후보자 동의서

첨 부 서 류

1. 소명방법            각 1통
1. 신청서부본          1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

신청인  {applicant['name']}

{court} 귀중
"""
        return doc

    def write_director_suspension_provisional_disposition(
        self,
        applicant: Dict,
        respondent: Dict,
        company: Dict,
        suspension_grounds: str,
        damage_description: str,
        acting_director_candidate: Dict,
        court: str
    ) -> str:
        """
        이사 직무집행정지 및 직무대행자 선임 가처분 신청서 작성

        Args:
            applicant: 신청인 (주주)
            respondent: 피신청인 (이사)
            company: 회사 (당사자)
            suspension_grounds: 정지 사유
            damage_description: 손해 발생 우려
            acting_director_candidate: 직무대행자 후보
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        doc = f"""대표이사 직무집행정지 가처분 및 직무대행자 선임 신청서

채  권  자  {applicant['name']}
            {applicant['address']}
            (주주, {applicant.get('shares_info', '')})

채  무  자  1. {respondent['name']}
               {respondent['address']}
               ({company['name']} {respondent.get('position', '대표이사')})

            2. {company['name']}
               {company['address']}
               대표이사  {respondent['name']}

{respondent.get('position', '대표이사')} 직무집행정지 가처분 및 직무대행자 선임 신청

신 청 취 지

1. 채무자 {respondent['name']}의 채무자 {company['name']}
   {respondent.get('position', '대표이사')}로서의 직무집행을 본안 판결
   확정시까지 정지한다.

2. 채무자 {company['name']}의 {respondent.get('position', '대표이사')}
   직무대행자로 {acting_director_candidate['name']}(주민등록번호:
   {acting_director_candidate.get('resident_number', '○○○○○○-○○○○○○○')})를 선임한다.

3. 신청비용은 채무자들의 부담으로 한다.

라는 결정을 구합니다.

신 청 이 유

제1. 피보전권리

1. 채권자는 채무자 회사의 주주로서 {applicant.get('ownership_info', '상당한 지분을')}
   보유하고 있습니다.

2. 채무자 {respondent['name']}의 위법·부당한 업무집행

   {suspension_grounds}

3. 채권자는 채무자 {respondent['name']}의 이사 해임을 구하는 본안소송을
   제기할 예정입니다.

제2. 보전의 필요성

1. 회복할 수 없는 손해 발생 우려

   {damage_description}

2. 따라서 본안 판결 확정시까지 채무자 {respondent['name']}의 직무집행을
   정지하고 직무대행자를 선임할 긴급한 필요가 있습니다.

제3. 직무대행자 선임의 필요성

1. 채무자 회사는 현재 긴급한 업무가 있습니다.

2. {acting_director_candidate['name']}은(는) {acting_director_candidate.get('qualification', '업무에 정통하고 주주들의 신뢰를 받고 있어')}
   직무대행자로 적합합니다.

소 명 방 법

갑 제1호증     주주명부 등본
갑 제2호증     위법행위 증빙자료
갑 제3호증     회계장부
갑 제4호증     직무대행자 후보 동의서

첨 부 서 류

1. 소명방법            각 1통
1. 법인등기부등본      1통
1. 신청서부본          2통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

채권자  {applicant['name']}

{court} 귀중
"""
        return doc


# 사용 예시
if __name__ == "__main__":
    writer = CommercialNonlitigationWriter()

    # 검사인 선임 신청서 작성 예시
    inspector_app = writer.write_inspector_appointment_application(
        applicant={
            "name": "김○○",
            "address": "서울시 강남구 ○○동 1234",
            "position": "발기인"
        },
        company={
            "name": "주식회사 ○○",
            "address": "서울시 강남구 ○○동 5678",
            "representative": "이○○",
            "establishment_date": "2024. 1. 15."
        },
        inspection_purpose="현물출자",
        inspection_target={
            "description": "서울시 강남구 ○○동 123 대지 500㎡",
            "claimed_value": 500000000,
            "shares": "보통주 50,000주 (1주당 금 10,000원)"
        },
        inspection_reason="상법 제299조에 의하여 현물출자에 대한 검사인의 조사를 받아야 합니다.",
        urgency="회사 설립등기를 위해 긴급히 검사가 필요합니다.",
        court="서울중앙지방법원"
    )

    print(inspector_app)
