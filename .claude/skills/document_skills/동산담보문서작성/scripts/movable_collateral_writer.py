"""
동산담보문서작성 (Movable Property Collateral Document Writer)

동산·채권·지적재산권 담보 관련 각종 문서를 자동으로 작성하는 모듈입니다.

주요 기능:
1. 동산담보 문서 작성
   - 동산담보권 설정 계약서
   - 동산담보등기 신청서
   - 동산 경매신청서
   - 동산 사적실행 통지서

2. 채권담보 문서 작성
   - 채권담보권 설정 계약서
   - 채권담보등기 신청서
   - 채권 추심명령 신청서
   - 채권 전부명령 신청서

3. 지적재산권담보 문서 작성
   - 지적재산권담보권 설정 계약서
   - 지적재산권담보등기 신청서
   - 지적재산권 경매신청서

참고자료: 동산·채권·지적재산권 담보제도 실무 (김현선 저, 백영사, 33,174줄)
Version: 1.0.0
"""

from typing import Dict, List
from datetime import datetime


class MovableCollateralWriter:
    """동산담보문서 작성 클래스"""

    def __init__(self):
        self.reference_data = "동산·채권·지적재산권 담보제도 실무 (김현선 저, 백영사, 33,174줄)"
        self.version = "1.0.0"

    # ============================================================================
    # 1. 동산담보 문서 작성
    # ============================================================================

    def write_movable_collateral_contract(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_property: Dict,
        secured_claim: Dict,
        contract_terms: Dict,
        contract_date: str
    ) -> str:
        """
        동산담보권 설정 계약서 작성

        Args:
            creditor: 채권자(담보권자) 정보
                - name: 상호/성명
                - registration_number: 사업자등록번호/주민등록번호
                - address: 주소
                - representative: 대표자
                - position: 직위
            debtor: 채무자(담보권설정자) 정보
                - name: 상호/성명
                - registration_number: 사업자등록번호/주민등록번호
                - address: 주소
                - representative: 대표자
                - position: 직위
            collateral_property: 담보목적물 정보
                - type: 동산의 종류 (재고자산, 기계장비, 차량 등)
                - description: 상세 설명
                - quantity: 수량
                - location: 보관장소
                - estimated_value: 평가액
                - identification: 특정 방법
            secured_claim: 피담보채권 정보
                - principal: 원금
                - interest_rate: 이자율
                - max_amount: 채권최고액
                - loan_date: 대출일
                - maturity_date: 만기일
                - purpose: 대출목적
            contract_terms: 계약 조건
                - registration_deadline: 등기 신청 기한
                - insurance_required: 보험가입 의무 여부
                - insurance_beneficiary: 보험수익자
                - periodic_report: 정기보고 의무
                - private_execution_allowed: 사적실행 허용 여부
                - notice_period_days: 사적실행 통지기간
            contract_date: 계약일

        Returns:
            str: 작성된 동산담보권 설정 계약서
        """
        document = f"""
                    동산담보권 설정 계약서


채권자(이하 "갑"이라 한다)와 채무자 겸 담보권설정자(이하 "을"이라 한다)는 다음과 같이 동산담보권 설정 계약을 체결한다.


제1조(목적)
본 계약은 을이 갑에 대하여 부담하는 채무를 담보하기 위하여 「동산·채권 등의 담보에 관한 법률」에 따라 을 소유의 동산에 담보권을 설정함을 목적으로 한다.


제2조(당사자)

1. 채권자(담보권자) - 갑
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   주소: {creditor['address']}
   대표자: {creditor['representative']} ({creditor.get('position', '대표이사')})

2. 채무자 겸 담보권설정자 - 을
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   주소: {debtor['address']}
   대표자: {debtor['representative']} ({debtor.get('position', '대표이사')})


제3조(피담보채권)

① 본 계약에 의하여 담보되는 채권(이하 "피담보채권"이라 한다)은 다음과 같다.

   1. 원금: {secured_claim['principal']:,}원
   2. 이자: 연 {secured_claim['interest_rate']}%
   3. 지연손해금: 연 {secured_claim.get('delay_damages_rate', secured_claim['interest_rate'] * 1.5)}%
   4. 대출일: {secured_claim['loan_date']}
   5. 만기일: {secured_claim['maturity_date']}
   6. 대출목적: {secured_claim['purpose']}

② 피담보채권의 범위는 원금, 이자, 지연손해금 및 담보권 실행비용을 포함한다.

③ 채권최고액: {secured_claim['max_amount']:,}원


제4조(담보목적물)

① 담보목적물은 다음과 같다.

   1. 동산의 종류: {collateral_property['type']}
   2. 상세 설명: {collateral_property['description']}
   3. 수량: {collateral_property.get('quantity', '별지 목록 기재')}
   4. 보관장소: {collateral_property['location']}
   5. 평가액: 금 {collateral_property['estimated_value']:,}원
   6. 특정 방법: {collateral_property['identification']}

② 담보목적물에는 그 종물, 부합물, 과실을 포함한다.


제5조(담보권의 설정 및 등기)

① 을은 제4조 기재 동산에 대하여 갑을 위하여 채권최고액 {secured_claim['max_amount']:,}원의 동산담보권을 설정한다.

② 을은 {contract_terms['registration_deadline']}에 동산담보권 설정등기를 신청하여야 한다.

③ 등기신청에 필요한 비용은 을이 부담한다.


제6조(담보목적물의 보존·관리 의무)

① 을은 담보목적물을 선량한 관리자의 주의로 보존·이용·개량하여야 한다.

② 을은 담보목적물을 임의로 처분, 양도, 훼손, 멸실하게 하거나 담보목적물의 가치를 감소시키는 행위를 하여서는 아니 된다.

③ 을은 담보목적물의 멸실·훼손 또는 가치감소의 우려가 있을 때에는 지체 없이 갑에게 통지하여야 한다.
"""

        # 보험가입 조항 추가
        if contract_terms.get('insurance_required'):
            document += f"""

제7조(보험)

① 을은 담보목적물에 대하여 화재보험, 도난보험 등 적절한 보험에 가입하여야 한다.

② 보험증권의 보험금 수령인은 {contract_terms['insurance_beneficiary']}로 지정한다.

③ 을은 보험증권 사본을 갑에게 교부하여야 한다.
"""

        # 정기보고 조항 추가
        if contract_terms.get('periodic_report'):
            document += f"""

제8조(정기보고 의무)

을은 {contract_terms['periodic_report']}에 담보목적물의 현황(수량, 위치, 상태 등)을 갑에게 서면으로 보고하여야 한다.
"""

        # 사적실행 조항 추가
        article_number = 9
        if contract_terms.get('private_execution_allowed'):
            document += f"""

제{article_number}조(사적실행)

① 갑은 을이 피담보채권을 변제기에 변제하지 아니하는 경우 담보목적물을 처분하거나 갑에게 귀속시켜 피담보채권의 변제에 충당할 수 있다.

② 갑이 사적실행을 하고자 하는 경우 을에게 그 뜻을 통지하여야 하며, 통지 후 {contract_terms['notice_period_days']}일이 경과하면 사적실행을 할 수 있다.

③ 갑은 상당한 가격으로 담보목적물을 처분하여야 한다.

④ 처분대금이 피담보채권액을 초과하는 경우 그 잔액은 을에게 반환하고, 부족한 경우 을은 그 부족액을 변제할 책임이 있다.
"""
            article_number += 1

        document += f"""

제{article_number}조(경매에 의한 실행)

① 갑은 을이 피담보채권을 변제기에 변제하지 아니하는 경우 「동산·채권 등의 담보에 관한 법률」 및 「민사집행법」에 따라 담보목적물에 대한 경매를 신청할 수 있다.

② 갑은 경매 절차에서 다른 채권자에 우선하여 배당받을 권리가 있다.


제{article_number + 1}조(기한의 이익 상실)

을에게 다음 각 호의 사유가 발생한 경우 을은 당연히 기한의 이익을 상실하고, 갑은 피담보채권 전액을 즉시 청구할 수 있다.

   1. 원금 또는 이자의 지급을 1회라도 지체한 때
   2. 다른 채권자로부터 가압류·가처분·강제집행·경매 등을 받은 때
   3. 파산·회생절차개시 신청을 받거나 스스로 신청한 때
   4. 담보목적물을 갑의 동의 없이 처분·훼손하거나 담보가치를 현저히 감소시킨 때
   5. 제5조 제2항의 등기 의무를 이행하지 아니한 때
   6. 제6조의 보존·관리 의무를 위반한 때
   7. 기타 본 계약의 주요 조항을 위반한 때


제{article_number + 2}조(계약의 해지)

① 갑은 을이 본 계약상의 의무를 위반한 경우 상당한 기간을 정하여 시정을 최고하고, 을이 이를 이행하지 아니하면 본 계약을 해지할 수 있다.

② 을이 피담보채권을 전액 변제한 경우 본 계약은 종료된다.


제{article_number + 3}조(담보권의 말소)

① 을이 피담보채권을 전액 변제한 경우 갑은 지체 없이 담보권 말소등기에 필요한 서류를 을에게 교부하여야 한다.

② 담보권 말소등기에 필요한 비용은 을이 부담한다.


제{article_number + 4}조(통지)

본 계약에 따른 모든 통지는 본 계약서에 기재된 주소로 하되, 내용증명우편으로 발송한다.


제{article_number + 5}조(관할법원)

본 계약에 관한 분쟁은 갑의 본점 소재지를 관할하는 법원을 제1심 관할법원으로 한다.


본 계약의 성립을 증명하기 위하여 계약서 2통을 작성하여 갑, 을이 기명날인한 후 각 1통씩 보관한다.


                            {contract_date}


      채권자(담보권자) - 갑
          주소: {creditor['address']}
          상호: {creditor['name']}
          대표자: {creditor['representative']}  (인)


      채무자 겸 담보권설정자 - 을
          주소: {debtor['address']}
          상호: {debtor['name']}
          대표자: {debtor['representative']}  (인)
"""

        return document

    def write_movable_collateral_registration_application(
        self,
        registration_info: Dict,
        creditor: Dict,
        debtor: Dict,
        collateral_property: Dict,
        registration_details: Dict
    ) -> str:
        """
        동산담보권 설정등기 신청서 작성

        Args:
            registration_info: 등기신청 정보
                - registry_office: 등기소
                - application_date: 신청일
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_property: 담보목적물 정보
            registration_details: 등기사항
                - secured_claim_max_amount: 채권최고액
                - claim_description: 채권의 표시
                - contract_date: 계약일
                - private_execution: 사적실행 가능 여부
                - notice_period: 통지기간

        Returns:
            str: 작성된 동산담보권 설정등기 신청서
        """
        document = f"""
                동산담보권 설정등기 신청서


{registration_info['registry_office']}  귀중


1. 등기의 목적
   동산담보권 설정등기


2. 등기원인 및 그 연월일
   {registration_details['contract_date']} 담보권설정계약


3. 등기권리자(담보권자)
   주소: {creditor['address']}
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   대표자: {creditor['representative']}


4. 등기의무자(담보권설정자)
   주소: {debtor['address']}
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   대표자: {debtor['representative']}


5. 담보목적물의 표시
   가. 동산의 종류: {collateral_property['type']}
   나. 동산의 상세 설명: {collateral_property['description']}
   다. 동산의 보관장소: {collateral_property['location']}
   라. 동산의 특정 방법: {collateral_property['identification']}


6. 피담보채권의 표시
   가. 채권최고액: 금 {registration_details['secured_claim_max_amount']:,}원
   나. 채권의 표시: {registration_details['claim_description']}


7. 사적실행
   가. 사적실행 가능 여부: {registration_details.get('private_execution', '가능')}
"""

        if registration_details.get('private_execution') == '가능':
            document += f"""   나. 사적실행 통지기간: {registration_details.get('notice_period', '14일')}
"""

        document += f"""

8. 첨부서류
   가. 담보권설정계약서  1통
   나. 등기권리자(담보권자)의 법인 등기사항증명서  1통
   다. 등기의무자(담보권설정자)의 법인 등기사항증명서  1통
   라. 등기권리자(담보권자)의 인감증명서  1통
   마. 등기의무자(담보권설정자)의 인감증명서  1통
   바. 위임장(대리인이 신청하는 경우)  각 1통


                            {registration_info['application_date']}


          신청인

          등기권리자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)

          등기의무자(담보권설정자)
              주소: {debtor['address']}
              상호: {debtor['name']}
              대표자: {debtor['representative']}  (인)
"""

        return document

    def write_movable_auction_application(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_property: Dict,
        secured_claim: Dict,
        collateral_registration: Dict,
        default_facts: Dict,
        court: str
    ) -> str:
        """
        동산 경매신청서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_property: 담보목적물 정보
            secured_claim: 피담보채권 정보
            collateral_registration: 담보등기 정보
            default_facts: 채무불이행 사실
            court: 관할법원

        Returns:
            str: 작성된 동산 경매신청서
        """
        document = f"""
                      경 매 신 청 서


채  권  자    {creditor['name']}
채  무  자    {debtor['name']}


{court}  귀중


청 구 채 권

1. 원금: {secured_claim['principal']:,}원
2. 이자: {secured_claim['interest']}
3. 지연손해금: {secured_claim['delay_damages']}

합계: {secured_claim['total_claim']:,}원


경매목적물의 표시

1. 동산의 종류: {collateral_property['type']}
2. 동산의 상세 설명: {collateral_property['description']}
3. 동산의 소재지: {collateral_property['location']}
4. 동산의 평가액: 금 {collateral_property['estimated_value']:,}원


청 구 원 인

1. 피담보채권의 발생

   채권자는 {secured_claim['loan_date']}에 채무자에게 금 {secured_claim['principal']:,}원을 변제기 {secured_claim['maturity_date']}, 이자 연 {secured_claim['interest'].split(',')[0].split('연 ')[1]}의 약정으로 대여하였습니다.


2. 담보권의 설정

   채권자는 위 대여금채권을 담보하기 위하여 채무자 소유의 동산에 대하여 채권최고액 {collateral_registration['max_secured_amount']:,}원의 동산담보권을 설정받고, {collateral_registration['registration_date']}에 동산담보권 설정등기(등기번호: {collateral_registration['registration_number']})를 마쳤습니다.


3. 채무불이행

   채무자는 {default_facts['reason']}

   채권자는 {default_facts['notice_date']}에 {default_facts['notice_method']}으로 채무이행을 최고하였으나, 채무자는 이를 이행하지 않고 있습니다.


4. 경매신청

   채권자는 「동산·채권 등의 담보에 관한 법률」 제41조 및 「민사집행법」에 따라 위 동산에 대한 경매를 신청합니다.


소 명 방 법

1. 금전소비대차계약서  1통
2. 동산담보권설정계약서  1통
3. 동산담보권설정등기증명서  1통
4. 채무불이행 증명자료(내용증명우편 등)  1통
5. 담보목적물 사진  각 1매


첨 부 서 류

1. 위 소명자료  각 1통
2. 법인 등기사항증명서(채권자, 채무자)  각 1통
3. 경매신청수수료 납부서  1통


                            {datetime.now().strftime('%Y년 %m월 %d일')}


          채권자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)
              전화번호: {creditor.get('phone', '')}
"""

        return document

    def write_private_execution_notice(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_property: Dict,
        secured_claim: Dict,
        execution_plan: Dict,
        contract_info: Dict,
        notice_date: str
    ) -> str:
        """
        동산 사적실행 통지서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_property: 담보목적물 정보
            secured_claim: 피담보채권 정보
            execution_plan: 사적실행 계획
            contract_info: 담보약정 정보
            notice_date: 통지일

        Returns:
            str: 작성된 동산 사적실행 통지서
        """
        document = f"""
                    사적실행 통지서


수신: {debtor['name']} {debtor['representative']} 대표이사
발신: {creditor['name']} {creditor['representative']} 대표이사


{debtor['name']} (이하 "귀사")가 당사에 대하여 부담하는 채무를 변제하지 않고 있으므로, 「동산·채권 등의 담보에 관한 법률」 제42조 및 {contract_info['contract_date']} 체결한 동산담보권설정계약 {contract_info['private_execution_clause']}에 따라 다음과 같이 담보목적물을 사적으로 실행할 것을 통지합니다.


1. 피담보채권의 내역

   가. 원금: {secured_claim['principal']:,}원
   나. 이자: {secured_claim['interest']}
   다. 지연손해금: {secured_claim['delay_damages']}

   합계: {secured_claim['total_claim']:,}원


2. 담보목적물의 표시

   가. 동산의 종류: {collateral_property['type']}
   나. 동산의 상세 설명: {collateral_property['description']}
   다. 동산의 소재지: {collateral_property['location']}


3. 사적실행의 방법

   가. 실행방법: {execution_plan['method']}
"""

        if execution_plan['method'] == '제3자 매각':
            document += f"""   나. 매수인: {execution_plan['buyer']}
   다. 매각가격: {execution_plan['sale_price']:,}원
   라. 매각예정일: {execution_plan['sale_date']}
"""

        if execution_plan.get('appraisal'):
            document += f"""   마. 평가자료: {execution_plan['appraisal']}
"""

        document += f"""

4. 사적실행의 시기

   본 통지서를 발송한 날로부터 {contract_info['notice_period']}일이 경과한 후 사적실행을 진행할 예정입니다.


5. 귀사의 권리

   가. 귀사는 본 통지를 받은 날로부터 {contract_info['notice_period']}일 이내에 피담보채권을 전액 변제하여 사적실행을 중지시킬 수 있습니다.

   나. 귀사는 사적실행 방법 또는 가격에 이의가 있는 경우 법원에 이의신청을 할 수 있습니다.


6. 정산

   사적실행으로 얻은 금액이 피담보채권액을 초과하는 경우 그 잔액은 귀사에게 반환하고, 부족한 경우 귀사는 그 부족액을 변제할 책임이 있습니다.


위와 같이 통지합니다.


                            {notice_date}


          채권자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              사업자등록번호: {creditor['registration_number']}
              대표자: {creditor['representative']}  (인)
"""

        return document

    # ============================================================================
    # 2. 채권담보 문서 작성
    # ============================================================================

    def write_claim_collateral_contract(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_claim: Dict,
        secured_claim: Dict,
        contract_terms: Dict,
        contract_date: str
    ) -> str:
        """
        채권담보권 설정 계약서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_claim: 담보채권 정보
                - type: 채권의 종류 (매출채권, 대여금채권 등)
                - debtor_of_claim: 제3채무자
                - claim_amount: 채권금액
                - claim_description: 채권의 표시
                - maturity_date: 변제기
                - collateral_ratio: 담보인정비율
                - scope: 담보채권의 범위
            secured_claim: 피담보채권 정보
            contract_terms: 계약 조건
            contract_date: 계약일

        Returns:
            str: 작성된 채권담보권 설정 계약서
        """
        document = f"""
                    채권담보권 설정 계약서


채권자(이하 "갑"이라 한다)와 채무자 겸 담보권설정자(이하 "을"이라 한다)는 다음과 같이 채권담보권 설정 계약을 체결한다.


제1조(목적)
본 계약은 을이 갑에 대하여 부담하는 채무를 담보하기 위하여 「동산·채권 등의 담보에 관한 법률」에 따라 을이 제3채무자에 대하여 가지는 채권에 담보권을 설정함을 목적으로 한다.


제2조(당사자)

1. 채권자(담보권자) - 갑
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   주소: {creditor['address']}
   대표자: {creditor['representative']} ({creditor.get('position', '대표이사')})

2. 채무자 겸 담보권설정자 - 을
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   주소: {debtor['address']}
   대표자: {debtor['representative']} ({debtor.get('position', '대표이사')})


제3조(피담보채권)

① 본 계약에 의하여 담보되는 채권(이하 "피담보채권"이라 한다)은 다음과 같다.

   1. 원금: {secured_claim['principal']:,}원
   2. 이자: 연 {secured_claim['interest_rate']}%
   3. 지연손해금: 연 {secured_claim.get('delay_damages_rate', secured_claim['interest_rate'] * 2)}%
   4. 대출일: {secured_claim['loan_date']}
   5. 만기일: {secured_claim['maturity_date']}
   6. 대출목적: {secured_claim['purpose']}

② 피담보채권의 범위는 원금, 이자, 지연손해금 및 담보권 실행비용을 포함한다.

③ 채권최고액: {secured_claim['max_amount']:,}원


제4조(담보목적 채권)

① 담보목적 채권은 다음과 같다.

   1. 채권의 종류: {collateral_claim['type']}
   2. 제3채무자: {collateral_claim['debtor_of_claim']}
   3. 채권금액: {collateral_claim['claim_amount']:,}원
   4. 채권의 표시: {collateral_claim['claim_description']}
   5. 변제기: {collateral_claim['maturity_date']}
   6. 담보채권의 범위: {collateral_claim['scope']}
   7. 담보인정비율: {collateral_claim.get('collateral_ratio', 100)}%

② 담보목적 채권에는 그로부터 발생하는 이자 기타 부수채권을 포함한다.


제5조(담보권의 설정 및 등기)

① 을은 제4조 기재 채권에 대하여 갑을 위하여 채권최고액 {secured_claim['max_amount']:,}원의 채권담보권을 설정한다.

② 을은 {contract_terms['registration_deadline']}에 채권담보권 설정등기를 신청하여야 한다.

③ 등기신청에 필요한 비용은 을이 부담한다.


제6조(채권양도 통지)

① 을은 제3채무자에게 담보목적 채권의 담보권설정 사실을 통지하여야 한다.

② 을은 제3채무자로부터 받은 승낙서 또는 통지서의 사본을 갑에게 교부하여야 한다.
"""

        # 추심계좌 지정 조항 추가
        if contract_terms.get('collection_account'):
            document += f"""

제7조(추심계좌)

① 을은 제3채무자가 담보목적 채권을 변제할 때 다음 계좌로 입금하도록 조치하여야 한다.

   계좌번호: {contract_terms['collection_account']}

② 제3채무자로부터 입금된 금액은 피담보채권의 변제에 우선 충당한다.
"""

        # 정기보고 조항 추가
        article_num = 8
        if contract_terms.get('periodic_report'):
            document += f"""

제{article_num}조(정기보고 의무)

을은 {contract_terms['periodic_report']}에 담보목적 채권의 현황(잔액, 변제내역 등)을 갑에게 서면으로 보고하여야 한다.
"""
            article_num += 1

        # 사적실행 조항 추가
        if contract_terms.get('private_execution_allowed'):
            document += f"""

제{article_num}조(사적실행)

① 갑은 을이 피담보채권을 변제기에 변제하지 아니하는 경우 담보목적 채권을 추심하거나 제3자에게 양도하여 피담보채권의 변제에 충당할 수 있다.

② 갑이 사적실행을 하고자 하는 경우 을에게 그 뜻을 통지하여야 하며, 통지 후 {contract_terms.get('notice_period_days', 14)}일이 경과하면 사적실행을 할 수 있다.

③ 추심금액 또는 양도대금이 피담보채권액을 초과하는 경우 그 잔액은 을에게 반환하고, 부족한 경우 을은 그 부족액을 변제할 책임이 있다.
"""
            article_num += 1

        document += f"""

제{article_num}조(채권의 추심 및 처분 제한)

① 을은 갑의 사전 서면 동의 없이 담보목적 채권을 추심, 양도, 면제하거나 기타 처분행위를 할 수 없다.

② 을이 제1항을 위반하여 추심 또는 처분한 경우 을은 당연히 기한의 이익을 상실한다.


제{article_num + 1}조(기한의 이익 상실)

을에게 다음 각 호의 사유가 발생한 경우 을은 당연히 기한의 이익을 상실하고, 갑은 피담보채권 전액을 즉시 청구할 수 있다.

   1. 원금 또는 이자의 지급을 1회라도 지체한 때
   2. 다른 채권자로부터 가압류·가처분·강제집행·경매 등을 받은 때
   3. 파산·회생절차개시 신청을 받거나 스스로 신청한 때
   4. 담보목적 채권을 갑의 동의 없이 추심하거나 처분한 때
   5. 제5조 제2항의 등기 의무를 이행하지 아니한 때
   6. 제6조의 통지 의무를 이행하지 아니한 때
   7. 기타 본 계약의 주요 조항을 위반한 때


제{article_num + 2}조(담보권의 말소)

① 을이 피담보채권을 전액 변제한 경우 갑은 지체 없이 담보권 말소등기에 필요한 서류를 을에게 교부하여야 한다.

② 담보권 말소등기에 필요한 비용은 을이 부담한다.


제{article_num + 3}조(관할법원)

본 계약에 관한 분쟁은 갑의 본점 소재지를 관할하는 법원을 제1심 관할법원으로 한다.


본 계약의 성립을 증명하기 위하여 계약서 2통을 작성하여 갑, 을이 기명날인한 후 각 1통씩 보관한다.


                            {contract_date}


      채권자(담보권자) - 갑
          주소: {creditor['address']}
          상호: {creditor['name']}
          대표자: {creditor['representative']}  (인)


      채무자 겸 담보권설정자 - 을
          주소: {debtor['address']}
          상호: {debtor['name']}
          대표자: {debtor['representative']}  (인)
"""

        return document

    def write_claim_collateral_registration_application(
        self,
        registration_info: Dict,
        creditor: Dict,
        debtor: Dict,
        collateral_claim: Dict,
        registration_details: Dict
    ) -> str:
        """
        채권담보권 설정등기 신청서 작성

        Args:
            registration_info: 등기신청 정보
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_claim: 담보채권 정보
            registration_details: 등기사항

        Returns:
            str: 작성된 채권담보권 설정등기 신청서
        """
        document = f"""
                채권담보권 설정등기 신청서


{registration_info['registry_office']}  귀중


1. 등기의 목적
   채권담보권 설정등기


2. 등기원인 및 그 연월일
   {registration_details['contract_date']} 담보권설정계약


3. 등기권리자(담보권자)
   주소: {creditor['address']}
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   대표자: {creditor['representative']}


4. 등기의무자(담보권설정자)
   주소: {debtor['address']}
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   대표자: {debtor['representative']}


5. 담보목적 채권의 표시
   가. 채권의 종류: {collateral_claim['type']}
   나. 제3채무자: {collateral_claim['debtor_of_claim']}
   다. 채권금액: 금 {collateral_claim['claim_amount']:,}원
   라. 채권의 표시: {collateral_claim['claim_description']}
   마. 변제기: {collateral_claim.get('maturity_date', '')}
   바. 담보채권의 범위: {collateral_claim['scope']}


6. 피담보채권의 표시
   가. 채권최고액: 금 {registration_details['secured_claim_max_amount']:,}원
   나. 채권의 표시: {registration_details['claim_description']}


7. 사적실행
   가. 사적실행 가능 여부: {registration_details.get('private_execution', '가능')}
"""

        if registration_details.get('private_execution') == '가능':
            document += f"""   나. 사적실행 통지기간: {registration_details.get('notice_period', '14일')}
"""

        document += f"""

8. 첨부서류
   가. 담보권설정계약서  1통
   나. 등기권리자(담보권자)의 법인 등기사항증명서  1통
   다. 등기의무자(담보권설정자)의 법인 등기사항증명서  1통
   라. 등기권리자(담보권자)의 인감증명서  1통
   마. 등기의무자(담보권설정자)의 인감증명서  1통
   바. 채권양도통지서 및 승낙서(또는 확정일자 있는 통지서)  1통
   사. 위임장(대리인이 신청하는 경우)  각 1통


                            {registration_info['application_date']}


          신청인

          등기권리자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)

          등기의무자(담보권설정자)
              주소: {debtor['address']}
              상호: {debtor['name']}
              대표자: {debtor['representative']}  (인)
"""

        return document

    def write_claim_collection_order_application(
        self,
        creditor: Dict,
        debtor: Dict,
        third_party_debtor: Dict,
        collateral_claim: Dict,
        secured_claim: Dict,
        collateral_registration: Dict,
        collection_amount: Dict,
        court: str
    ) -> str:
        """
        채권 추심명령 신청서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            third_party_debtor: 제3채무자 정보
            collateral_claim: 담보채권 정보
            secured_claim: 피담보채권 정보
            collateral_registration: 담보등기 정보
            collection_amount: 추심금액
            court: 관할법원

        Returns:
            str: 작성된 채권 추심명령 신청서
        """
        document = f"""
                    추 심 명 령 신 청 서


채  권  자    {creditor['name']}
채  무  자    {debtor['name']}
제3채무자    {third_party_debtor['name']}


{court}  귀중


청 구 채 권

1. 원금: {secured_claim['principal']:,}원
2. 이자: {secured_claim['interest']}
3. 지연손해금: {secured_claim['delay_damages']}

합계: {secured_claim['total_claim']:,}원


추 심 대 상 채 권

1. 채권의 종류: {collateral_claim['description']}
2. 제3채무자: {third_party_debtor['name']}
3. 채권금액: {collateral_claim['amount']:,}원
4. 변제기: {collateral_claim['maturity_date']}
5. 채권의 발생원인: {collateral_claim['claim_basis']}
6. 채권의 소재지: {collateral_claim['location']}


추 심 금 액

금 {collection_amount['requested_amount']:,}원
(추심금액 산정 근거: {collection_amount['basis']})


신 청 취 지

채권자가 채무자의 제3채무자에 대한 위 채권을 추심할 수 있도록 허가한다는 결정을 구합니다.


신 청 원 인

1. 피담보채권의 발생

   채권자는 {secured_claim.get('loan_date', '')} 채무자에게 금 {secured_claim['principal']:,}원을 대여하였습니다.


2. 담보권의 설정

   채권자는 위 대여금채권을 담보하기 위하여 채무자가 제3채무자에 대하여 가지는 위 채권에 대하여 채권최고액 {collateral_registration['max_secured_amount']:,}원의 채권담보권을 설정받고, {collateral_registration['registration_date']}에 채권담보권 설정등기(등기번호: {collateral_registration['registration_number']})를 마쳤습니다.


3. 채무불이행

   채무자는 위 대여금의 변제기가 도래하였음에도 이를 변제하지 않고 있습니다.


4. 추심명령 신청

   채권자는 「동산·채권 등의 담보에 관한 법률」 제41조 및 「민사집행법」 제229조에 따라 위 채권에 대한 추심명령을 신청합니다.


소 명 방 법

1. 금전소비대차계약서  1통
2. 채권담보권설정계약서  1통
3. 채권담보권설정등기증명서  1통
4. 채권양도통지서 및 확정일자 증명  1통
5. 제3채무자에 대한 채권 증명자료(거래명세서, 세금계산서 등)  각 1통
6. 채무불이행 증명자료  1통


첨 부 서 류

1. 위 소명자료  각 1통
2. 법인 등기사항증명서(채권자, 채무자, 제3채무자)  각 1통


                            {datetime.now().strftime('%Y년 %m월 %d일')}


          채권자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)
"""

        return document

    def write_claim_transfer_order_application(
        self,
        creditor: Dict,
        debtor: Dict,
        third_party_debtor: Dict,
        collateral_claim: Dict,
        secured_claim: Dict,
        collateral_registration: Dict,
        transfer_amount: Dict,
        court: str
    ) -> str:
        """
        채권 전부명령 신청서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            third_party_debtor: 제3채무자 정보
            collateral_claim: 담보채권 정보
            secured_claim: 피담보채권 정보
            collateral_registration: 담보등기 정보
            transfer_amount: 전부금액
            court: 관할법원

        Returns:
            str: 작성된 채권 전부명령 신청서
        """
        document = f"""
                    전 부 명 령 신 청 서


채  권  자    {creditor['name']}
채  무  자    {debtor['name']}
제3채무자    {third_party_debtor['name']}


{court}  귀중


청 구 채 권

1. 원금: {secured_claim['principal']:,}원
2. 이자: {secured_claim['interest']}
3. 지연손해금: {secured_claim['delay_damages']}

합계: {secured_claim['total_claim']:,}원


전 부 대 상 채 권

1. 채권의 종류: {collateral_claim['description']}
2. 제3채무자: {third_party_debtor['name']}
3. 채권금액: {collateral_claim['amount']:,}원
4. 변제기: {collateral_claim['maturity_date']}
5. 채권의 발생원인: {collateral_claim['claim_basis']}


전 부 금 액

금 {transfer_amount['requested_amount']:,}원
(전부금액 산정 근거: {transfer_amount['basis']})


신 청 취 지

채무자의 제3채무자에 대한 위 채권 중 금 {transfer_amount['requested_amount']:,}원을 채권자에게 전부한다는 결정을 구합니다.


신 청 원 인

1. 피담보채권의 발생

   채권자는 채무자에게 금 {secured_claim['principal']:,}원을 대여하였습니다.


2. 담보권의 설정

   채권자는 위 대여금채권을 담보하기 위하여 채무자가 제3채무자에 대하여 가지는 위 채권에 대하여 채권최고액 {collateral_registration['max_secured_amount']:,}원의 채권담보권을 설정받고, {collateral_registration['registration_date']}에 채권담보권 설정등기(등기번호: {collateral_registration['registration_number']})를 마쳤습니다.


3. 채무불이행

   채무자는 위 대여금의 변제기가 도래하였음에도 이를 변제하지 않고 있습니다.


4. 전부명령 신청

   채권자는 「동산·채권 등의 담보에 관한 법률」 제41조 및 「민사집행법」 제229조의2에 따라 위 채권에 대한 전부명령을 신청합니다.


소 명 방 법

1. 금전소비대차계약서  1통
2. 채권담보권설정계약서  1통
3. 채권담보권설정등기증명서  1통
4. 채권양도통지서 및 확정일자 증명  1통
5. 제3채무자에 대한 채권 증명자료  각 1통
6. 채무불이행 증명자료  1통


첨 부 서 류

1. 위 소명자료  각 1통
2. 법인 등기사항증명서(채권자, 채무자, 제3채무자)  각 1통


                            {datetime.now().strftime('%Y년 %m월 %d일')}


          채권자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)
"""

        return document

    # ============================================================================
    # 3. 지적재산권담보 문서 작성
    # ============================================================================

    def write_ip_collateral_contract(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_ip: Dict,
        secured_claim: Dict,
        contract_terms: Dict,
        contract_date: str
    ) -> str:
        """
        지적재산권담보권 설정 계약서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_ip: 담보 지적재산권 정보
                - type: 지적재산권 종류 (특허권, 상표권, 저작권, 디자인권)
                - title: 권리의 명칭
                - registration_number: 등록번호
                - registration_date: 등록일
                - holder: 권리자
                - description: 권리의 내용
                - estimated_value: 평가액
                - term_expiry: 권리 만료일
            secured_claim: 피담보채권 정보
            contract_terms: 계약 조건
            contract_date: 계약일

        Returns:
            str: 작성된 지적재산권담보권 설정 계약서
        """
        document = f"""
              지적재산권담보권 설정 계약서


채권자(이하 "갑"이라 한다)와 채무자 겸 담보권설정자(이하 "을"이라 한다)는 다음과 같이 지적재산권담보권 설정 계약을 체결한다.


제1조(목적)
본 계약은 을이 갑에 대하여 부담하는 채무를 담보하기 위하여 「동산·채권 등의 담보에 관한 법률」에 따라 을 소유의 지적재산권에 담보권을 설정함을 목적으로 한다.


제2조(당사자)

1. 채권자(담보권자) - 갑
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   주소: {creditor['address']}
   대표자: {creditor['representative']} ({creditor.get('position', '대표이사')})

2. 채무자 겸 담보권설정자 - 을
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   주소: {debtor['address']}
   대표자: {debtor['representative']} ({debtor.get('position', '대표이사')})


제3조(피담보채권)

① 본 계약에 의하여 담보되는 채권(이하 "피담보채권"이라 한다)은 다음과 같다.

   1. 원금: {secured_claim['principal']:,}원
   2. 이자: 연 {secured_claim['interest_rate']}%
   3. 지연손해금: 연 {secured_claim.get('delay_damages_rate', secured_claim['interest_rate'] * 2)}%
   4. 대출일: {secured_claim['loan_date']}
   5. 만기일: {secured_claim['maturity_date']}
   6. 대출목적: {secured_claim['purpose']}

② 피담보채권의 범위는 원금, 이자, 지연손해금 및 담보권 실행비용을 포함한다.

③ 채권최고액: {secured_claim['max_amount']:,}원


제4조(담보목적 지적재산권)

① 담보목적 지적재산권은 다음과 같다.

   1. 권리의 종류: {collateral_ip['type']}
   2. 권리의 명칭: {collateral_ip['title']}
   3. 등록번호: {collateral_ip['registration_number']}
   4. 등록일: {collateral_ip['registration_date']}
   5. 권리자: {collateral_ip['holder']}
   6. 권리의 내용: {collateral_ip['description']}
   7. 평가액: 금 {collateral_ip['estimated_value']:,}원
   8. 권리 존속기간 만료일: {collateral_ip.get('term_expiry', '')}

② 담보목적 지적재산권에는 그로부터 발생하는 사용료, 로열티 등 수익을 포함한다.


제5조(담보권의 설정 및 등기)

① 을은 제4조 기재 지적재산권에 대하여 갑을 위하여 채권최고액 {secured_claim['max_amount']:,}원의 지적재산권담보권을 설정한다.

② 을은 {contract_terms['registration_deadline']}에 지적재산권담보권 설정등기를 신청하여야 한다.

③ 등기신청에 필요한 비용은 을이 부담한다.


제6조(지적재산권의 유지·관리 의무)

① 을은 담보목적 지적재산권을 선량한 관리자의 주의로 유지·관리하여야 한다.

② 을은 권리 유지에 필요한 등록료, 연차료 등을 기한 내에 납부하여야 한다.

③ 을은 담보목적 지적재산권에 대한 무효심판, 취소심판 등이 청구된 경우 지체 없이 갑에게 통지하고 적절한 방어조치를 취하여야 한다.
"""

        # 전용실시권 설정 제한 조항
        if contract_terms.get('license_restriction'):
            document += f"""

제7조(실시권 설정 제한)

을은 {contract_terms['license_restriction']}
"""

        # 로열티 계좌 지정 조항
        article_num = 8
        if contract_terms.get('royalty_account'):
            document += f"""

제{article_num}조(사용료 관리)

① 을이 담보목적 지적재산권의 실시를 허락하여 사용료(로열티)를 받는 경우 {contract_terms['royalty_account']}

② 사용료는 피담보채권의 변제에 우선 충당할 수 있다.
"""
            article_num += 1

        # 정기보고 조항
        if contract_terms.get('periodic_report'):
            document += f"""

제{article_num}조(정기보고 의무)

을은 {contract_terms['periodic_report']}
"""
            article_num += 1

        # 사적실행 조항
        if contract_terms.get('private_execution_allowed'):
            document += f"""

제{article_num}조(사적실행)

① 갑은 을이 피담보채권을 변제기에 변제하지 아니하는 경우 담보목적 지적재산권을 처분하거나 갑에게 귀속시켜 피담보채권의 변제에 충당할 수 있다.

② 갑이 사적실행을 하고자 하는 경우 을에게 그 뜻을 통지하여야 하며, 통지 후 {contract_terms.get('notice_period_days', 14)}일이 경과하면 사적실행을 할 수 있다.

③ 갑은 상당한 가격으로 담보목적 지적재산권을 처분하여야 한다.

④ 처분대금이 피담보채권액을 초과하는 경우 그 잔액은 을에게 반환하고, 부족한 경우 을은 그 부족액을 변제할 책임이 있다.
"""
            article_num += 1

        # 권리 유지 의무
        if contract_terms.get('maintenance_obligation'):
            document += f"""

제{article_num}조(권리 유지 의무)

을은 {contract_terms['maintenance_obligation']}
"""
            article_num += 1

        document += f"""

제{article_num}조(경매에 의한 실행)

① 갑은 을이 피담보채권을 변제기에 변제하지 아니하는 경우 「동산·채권 등의 담보에 관한 법률」 및 「민사집행법」에 따라 담보목적 지적재산권에 대한 경매를 신청할 수 있다.

② 갑은 경매 절차에서 다른 채권자에 우선하여 배당받을 권리가 있다.


제{article_num + 1}조(기한의 이익 상실)

을에게 다음 각 호의 사유가 발생한 경우 을은 당연히 기한의 이익을 상실하고, 갑은 피담보채권 전액을 즉시 청구할 수 있다.

   1. 원금 또는 이자의 지급을 1회라도 지체한 때
   2. 다른 채권자로부터 가압류·가처분·강제집행·경매 등을 받은 때
   3. 파산·회생절차개시 신청을 받거나 스스로 신청한 때
   4. 담보목적 지적재산권을 갑의 동의 없이 처분하거나 담보가치를 현저히 감소시킨 때
   5. 제5조 제2항의 등기 의무를 이행하지 아니한 때
   6. 제6조의 유지·관리 의무를 위반한 때
   7. 담보목적 지적재산권이 무효 또는 취소된 때
   8. 기타 본 계약의 주요 조항을 위반한 때


제{article_num + 2}조(담보권의 말소)

① 을이 피담보채권을 전액 변제한 경우 갑은 지체 없이 담보권 말소등기에 필요한 서류를 을에게 교부하여야 한다.

② 담보권 말소등기에 필요한 비용은 을이 부담한다.


제{article_num + 3}조(관할법원)

본 계약에 관한 분쟁은 갑의 본점 소재지를 관할하는 법원을 제1심 관할법원으로 한다.


본 계약의 성립을 증명하기 위하여 계약서 2통을 작성하여 갑, 을이 기명날인한 후 각 1통씩 보관한다.


                            {contract_date}


      채권자(담보권자) - 갑
          주소: {creditor['address']}
          상호: {creditor['name']}
          대표자: {creditor['representative']}  (인)


      채무자 겸 담보권설정자 - 을
          주소: {debtor['address']}
          상호: {debtor['name']}
          대표자: {debtor['representative']}  (인)
"""

        return document

    def write_ip_collateral_registration_application(
        self,
        registration_info: Dict,
        creditor: Dict,
        debtor: Dict,
        collateral_ip: Dict,
        registration_details: Dict
    ) -> str:
        """
        지적재산권담보권 설정등기 신청서 작성

        Args:
            registration_info: 등기신청 정보
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_ip: 담보 지적재산권 정보
            registration_details: 등기사항

        Returns:
            str: 작성된 지적재산권담보권 설정등기 신청서
        """
        document = f"""
            지적재산권담보권 설정등기 신청서


{registration_info['registry_office']}  귀중


1. 등기의 목적
   지적재산권담보권 설정등기


2. 등기원인 및 그 연월일
   {registration_details['contract_date']} 담보권설정계약


3. 등기권리자(담보권자)
   주소: {creditor['address']}
   상호(성명): {creditor['name']}
   사업자등록번호: {creditor['registration_number']}
   대표자: {creditor['representative']}


4. 등기의무자(담보권설정자)
   주소: {debtor['address']}
   상호(성명): {debtor['name']}
   사업자등록번호: {debtor['registration_number']}
   대표자: {debtor['representative']}


5. 담보목적 지적재산권의 표시
   가. 권리의 종류: {collateral_ip['type']}
   나. 권리의 명칭: {collateral_ip['title']}
   다. 등록번호: {collateral_ip['registration_number']}
   라. 등록일: {collateral_ip['registration_date']}
   마. 권리의 내용: {collateral_ip['description']}


6. 피담보채권의 표시
   가. 채권최고액: 금 {registration_details['secured_claim_max_amount']:,}원
   나. 채권의 표시: {registration_details['claim_description']}


7. 사적실행
   가. 사적실행 가능 여부: {registration_details.get('private_execution', '가능')}
"""

        if registration_details.get('private_execution') == '가능':
            document += f"""   나. 사적실행 통지기간: {registration_details.get('notice_period', '14일')}
"""

        document += f"""

8. 첨부서류
   가. 담보권설정계약서  1통
   나. 등기권리자(담보권자)의 법인 등기사항증명서  1통
   다. 등기의무자(담보권설정자)의 법인 등기사항증명서  1통
   라. 지적재산권 등록증  1통
   마. 등기권리자(담보권자)의 인감증명서  1통
   바. 등기의무자(담보권설정자)의 인감증명서  1통
   사. 위임장(대리인이 신청하는 경우)  각 1통


                            {registration_info['application_date']}


          신청인

          등기권리자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)

          등기의무자(담보권설정자)
              주소: {debtor['address']}
              상호: {debtor['name']}
              대표자: {debtor['representative']}  (인)
"""

        return document

    def write_ip_auction_application(
        self,
        creditor: Dict,
        debtor: Dict,
        collateral_ip: Dict,
        secured_claim: Dict,
        collateral_registration: Dict,
        default_facts: Dict,
        court: str
    ) -> str:
        """
        지적재산권 경매신청서 작성

        Args:
            creditor: 채권자(담보권자) 정보
            debtor: 채무자(담보권설정자) 정보
            collateral_ip: 담보 지적재산권 정보
            secured_claim: 피담보채권 정보
            collateral_registration: 담보등기 정보
            default_facts: 채무불이행 사실
            court: 관할법원

        Returns:
            str: 작성된 지적재산권 경매신청서
        """
        document = f"""
                      경 매 신 청 서


채  권  자    {creditor['name']}
채  무  자    {debtor['name']}


{court}  귀중


청 구 채 권

1. 원금: {secured_claim['principal']:,}원
2. 이자: {secured_claim['interest']}
3. 지연손해금: {secured_claim['delay_damages']}

합계: {secured_claim['total_claim']:,}원


경매목적물의 표시

1. 권리의 종류: {collateral_ip['type']}
2. 권리의 명칭: {collateral_ip['title']}
3. 등록번호: {collateral_ip['registration_number']}
4. 등록일: {collateral_ip['registration_date']}
5. 권리의 내용: {collateral_ip['description']}
6. 평가액: 금 {collateral_ip['estimated_value']:,}원


청 구 원 인

1. 피담보채권의 발생

   채권자는 {secured_claim['loan_date']}에 채무자에게 금 {secured_claim['principal']:,}원을 변제기 {secured_claim['maturity_date']}, 이자 연 {secured_claim['interest'].split(',')[0].split('연 ')[1]}의 약정으로 대여하였습니다.


2. 담보권의 설정

   채권자는 위 대여금채권을 담보하기 위하여 채무자 소유의 위 지적재산권에 대하여 채권최고액 {collateral_registration['max_secured_amount']:,}원의 지적재산권담보권을 설정받고, {collateral_registration['registration_date']}에 지적재산권담보권 설정등기(등기번호: {collateral_registration['registration_number']})를 마쳤습니다.


3. 채무불이행

   채무자는 {default_facts['reason']}

   채권자는 {default_facts['notice_date']}에 {default_facts['notice_method']}으로 채무이행을 최고하였으나, 채무자는 이를 이행하지 않고 있습니다.


4. 경매신청

   채권자는 「동산·채권 등의 담보에 관한 법률」 제41조 및 「민사집행법」에 따라 위 지적재산권에 대한 경매를 신청합니다.


소 명 방 법

1. 금전소비대차계약서  1통
2. 지적재산권담보권설정계약서  1통
3. 지적재산권담보권설정등기증명서  1통
4. 지적재산권 등록증  1통
5. 채무불이행 증명자료(내용증명우편 등)  1통


첨 부 서 류

1. 위 소명자료  각 1통
2. 법인 등기사항증명서(채권자, 채무자)  각 1통
3. 경매신청수수료 납부서  1통


                            {datetime.now().strftime('%Y년 %m월 %d일')}


          채권자(담보권자)
              주소: {creditor['address']}
              상호: {creditor['name']}
              대표자: {creditor['representative']}  (인)
              전화번호: {creditor.get('phone', '')}
"""

        return document


# 모듈 정보
__version__ = "1.0.0"
__author__ = "LawPro AI Team"
__reference__ = "동산·채권·지적재산권 담보제도 실무 (김현선 저, 백영사)"
