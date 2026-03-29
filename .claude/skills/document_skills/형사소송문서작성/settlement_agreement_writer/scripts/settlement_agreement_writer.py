"""
형사 합의서 작성 (Criminal Settlement Agreement Writer)

형사사건의 피해자와 가해자 간 합의서를 자동으로 작성하는 모듈입니다.

주요 기능:
1. 폭행사건 합의서 작성
2. 교통사고 형사합의서 작성 (채권양도 조항 포함)
3. 절도사건 합의서 작성 (미성년자 대리인 지원)
4. 일반 형사사건 합의서 작성

참고자료: 형사소송문서 서식 (13,571~13,705줄)
Version: 1.0.0
"""

from typing import Dict, Optional
from datetime import datetime


class SettlementAgreementWriter:
    """형사 합의서 작성 클래스"""

    def __init__(self):
        self.version = "1.0.0"

    def _format_amount(self, amount: int) -> str:
        """금액을 한글로 변환"""
        units = ["", "만", "억", "조"]
        result = []

        for i, unit in enumerate(units):
            value = (amount // (10000 ** i)) % 10000
            if value > 0:
                result.insert(0, f"{value}{unit}")

        return "".join(result) if result else "영"

    def write_assault_settlement(
        self,
        victim: Dict,
        perpetrator: Dict,
        incident: Dict,
        compensation: Dict,
        leniency_request: bool = True
    ) -> str:
        """
        폭행사건 합의서 작성

        Args:
            victim: 피해자 정보
                - name: 이름
                - resident_number: 주민등록번호
                - address: 주소
                - phone: 전화번호
            perpetrator: 가해자 정보
                - name: 이름
                - resident_number: 주민등록번호
                - address: 주소
                - phone: 전화번호
            incident: 사건 정보
                - date: 사건 날짜
                - time: 사건 시간
                - location: 사건 장소
                - injury: 상해 진단명
                - description: 사건 설명
            compensation: 합의금 정보
                - amount: 합의금액
                - payment_date: 지급일
                - payment_method: 지급방법
                - bank_info: 은행정보 (선택)
            leniency_request: 선처 요청 여부

        Returns:
            str: 작성된 폭행사건 합의서
        """
        amount_korean = self._format_amount(compensation['amount'])

        document = f"""
                     합 의 서


피해자    {victim['name']}({victim['resident_number']})
          {victim['address']}
          전화 {victim['phone']}

가해자    {perpetrator['name']}({perpetrator['resident_number']})
          {perpetrator['address']}
          전화 {perpetrator['phone']}


위 피해자 {victim['name']}을 "갑"이라 칭하고, 가해자 {perpetrator['name']}을 "을"이라 칭하며
다음과 같은 조건으로 민·형사상 합의를 한다.


                             - 다 음 -


1. (갑)과 (을)이 {incident['date']} {incident['time']} {incident['location']}에서
   {incident['description']}으로 인해 "갑"은 {incident['injury']}의 상해를 입은 사실이
   있는바, (을)은 (갑)에게 민·형사상 합의금조로 금 {compensation['amount']:,}원
   ({amount_korean}원)을 {compensation['payment_date']}에 지급하기로 한다.
"""

        if compensation.get('bank_info'):
            bank_info = compensation['bank_info']
            document += f"""
   지급방법: {bank_info['bank']} {bank_info['account']} (예금주: {bank_info['holder']})
"""

        document += f"""

2. (갑)은 위 사건으로 인한 일체 민사상 손해배상을 "을"에게 청구하지 않으며,
   또한 "을"이 형사상 처벌받는 것을 원하지 않는다.
"""

        if leniency_request:
            document += """
   (검사님 그리고 판사님! 가해자는 피해자에게 미안해하고 있고, 깊이 뉘우치고 있으므로
   최대한의 선처를 하여 주시기 바랍니다.)
"""

        document += f"""

3. 전 항의 합의는 "갑"과 "을"의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자
   등으로부터 사기와 강박 등이 전혀 없이 평온·공연하게 합의한 것이므로 향후
   어떠한 사유로도 "갑"은 "을"에게, "을"은 "갑"에게 민·형사상 책임을 묻지 아니하며
   모든 청구권을 포기한다.


※ 첨부서류: 인감증명서 각 1통


                            {datetime.now().strftime('%Y')}년 {datetime.now().strftime('%m')}월 {datetime.now().strftime('%d')}일


위 합의인(피해자) "갑"    {victim['name']}  (인)
                          {victim['address']}

위 합의인(가해자) "을"    {perpetrator['name']}  (인)
                          {perpetrator['address']}
"""

        return document

    def write_traffic_accident_settlement(
        self,
        victim: Dict,
        perpetrator: Dict,
        accident: Dict,
        compensation: Dict,
        insurance_company: str,
        credit_assignment: bool = True,
        leniency_request: bool = True
    ) -> str:
        """
        교통사고 형사합의서 작성

        Args:
            victim: 피해자 정보
            perpetrator: 가해자 정보
            accident: 사고 정보
                - date: 사고 날짜
                - time: 사고 시간
                - location: 사고 장소
                - type: 사고 유형
                - vehicle: 가해 차량
                - injury: 상해 정도
            compensation: 합의금 정보
            insurance_company: 보험회사명
            credit_assignment: 채권양도 조항 포함 여부
            leniency_request: 선처 요청 여부

        Returns:
            str: 작성된 교통사고 형사합의서
        """
        amount_korean = self._format_amount(compensation['amount'])

        document = f"""
                   형 사 합 의 서


교통사고 피해자    {victim['name']}({victim['resident_number']})
                   {victim['address']}
                   전화 {victim['phone']}

교통사고 가해자    {perpetrator['name']}({perpetrator['resident_number']})
                   {perpetrator['address']}
                   전화 {perpetrator['phone']}


위 피해자와 가해자는 교통사고에 대하여 다음과 같이 형사상의 합의를 한다.


                             - 다 음 -


1. 사고 내용
   가. 사고일시: {accident['date']} {accident['time']}
   나. 사고장소: {accident['location']}
   다. 사고유형: {accident['type']}
   라. 가해자: {perpetrator['name']}
   마. 가해차량: {accident['vehicle']}
   바. 피해자: {victim['name']}
   사. 상해정도: {accident.get('injury', '상세 별첨')}


2. 합의내용
   가. 합의금액: 금 {amount_korean}원({compensation['amount']:,}원)

   나. 합의사항
       (1) 이 합의금은 가해자가 형사처벌을 감경할 목적으로 가해자 개인이 지급하는
           금액이다.
       (2) 피해자는 위 금원을 지급받고 가해자의 형사적인 처벌을 원치 아니한다.
"""

        if leniency_request:
            document += """
           (검사님, 판사님 가해자께서는 피해자에 대하여 미안해하고, 안타까워하고,
           마음아파하고 있으니 넓으신 마음으로 혜량하여 주시기 바랍니다.)
"""

        if credit_assignment:
            document += f"""

   다. 채권양도
       (1) 만일 보험회사의 보상금에서 위 합의금의 일부라도 공제될 경우 그에 대하여
           가해자가 보험회사를 상대로 갖게 될 부당이득반환청구권(또는 보험금 청구권)을
           피해자 {victim['name']}에게 양도한다.
       (2) 이와 같은 채권양도의 효력을 확실히 하기 위해 가해자는 즉시 가해차량의
           보험회사인 {insurance_company}에 채권양도통지를 한다.
       (3) 이 합의로써 채권양도 되었기에 나중에 가해자가 보험회사에 대한
           부당이득반환청구권을 포기하더라도 그 효력은 인정되지 못한다.
       (4) 만일 가해자가 보험회사에 대한 부당이득반환청구권을 포기할 경우에는
           보상금액에서 공제된 합의금 액수만큼 피해자에게 다시 지급한다.
"""

        document += f"""

3. 기타사항
   피해자의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자 등으로부터 사기와
   강박 등이 전혀 없이 평온·공연하게 합의한 것이므로 향후 어떠한 사유로도
   가해자에게 형사상 책임을 묻지 아니하며 모든 청구권을 포기한다.

   위와 같은 합의내용을 확실하게 하기 위해 이 합의서를 3부 작성하여 1부는
   수사기관 또는 법원에 제출하고 나머지는 가해자 측과 피해자 측에서 각 1부씩
   보관한다.


                            {datetime.now().strftime('%Y')}년 {datetime.now().strftime('%m')}월 {datetime.now().strftime('%d')}일


위 피해자    {victim['name']}  (인)
             {victim['address']}

위 가해자    {perpetrator['name']}  (인)
             {perpetrator['address']}
"""

        return document

    def write_theft_settlement(
        self,
        victim: Dict,
        perpetrator: Dict,
        incident: Dict,
        compensation: Dict,
        leniency_request: bool = True
    ) -> str:
        """
        절도사건 합의서 작성

        Args:
            victim: 피해자 정보
            perpetrator: 가해자 정보
                - legal_representative: 법정대리인 정보 (미성년자인 경우)
                    - name: 대리인 이름
                    - resident_number: 대리인 주민등록번호
                    - relationship: 관계 (부, 모 등)
            incident: 사건 정보
                - date: 사건 날짜
                - time: 사건 시간
                - location: 사건 장소
                - stolen_item: 절취 물품
                - value: 피해 금액
                - description: 사건 설명
            compensation: 합의금 정보
            leniency_request: 선처 요청 여부

        Returns:
            str: 작성된 절도사건 합의서
        """
        amount_korean = self._format_amount(compensation['amount'])
        legal_rep = perpetrator.get('legal_representative')

        document = f"""
                     합 의 서


1. {perpetrator['name']}({perpetrator['resident_number']})은 {incident['date']} {incident['time']}
   {incident['location']}에서 {incident['description']}

2. """

        if legal_rep:
            document += f"""{perpetrator['name']}의 {legal_rep['relationship']} {legal_rep['name']}은 손해배상금 및 위자료조로
   {compensation['payment_date']} 금 {compensation['amount']:,}원({amount_korean}원)을
"""
        else:
            document += f"""가해자 {perpetrator['name']}은 손해배상금 및 위자료조로
   {compensation['payment_date']} 금 {compensation['amount']:,}원({amount_korean}원)을
"""

        if compensation.get('bank_info'):
            bank_info = compensation['bank_info']
            document += f"""   피해자의 은행통장계좌({bank_info['bank']}, {bank_info['account']})로 입금한다.
"""
        else:
            document += """   피해자에게 지급한다.
"""

        document += f"""

3. 피해자는 위 사건에 대하여 가해자의 형사상 처벌을 원하지 않으며,
   일체 어떠한 민사상 책임도 묻지 않는다.
"""

        if leniency_request:
            document += """
   (검사님 그리고 판사님! 가해자는 피해자에게 미안해하고 있고, 깊이 뉘우치고 있으므로
   최대한의 선처를 하여 주시기 바랍니다.)
"""

        document += f"""

4. 전 항의 합의는 당사자의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자 등으로부터
   사기와 강박 등이 전혀 없이 평온·공연하게 합의한 것이다.


                            {datetime.now().strftime('%Y')}년 {datetime.now().strftime('%m')}월 {datetime.now().strftime('%d')}일


위 합의인(피해자)    {victim['name']}({victim['resident_number']})  (인)
                     {victim['address']}

위 합의인(가해자)    {perpetrator['name']}
"""

        if legal_rep:
            document += f"""                     {perpetrator['name']}의 대리인 {legal_rep['relationship']}  {legal_rep['name']}({legal_rep['resident_number']})  (인)
"""

        document += f"""                     {perpetrator['address']}
"""

        return document

    def write_general_settlement(
        self,
        victim: Dict,
        perpetrator: Dict,
        incident: Dict,
        compensation: Dict,
        leniency_request: bool = True
    ) -> str:
        """
        일반 형사사건 합의서 작성

        Args:
            victim: 피해자 정보
            perpetrator: 가해자 정보
            incident: 사건 정보
                - case_name: 사건명
                - date: 사건 날짜
                - time: 사건 시간
                - location: 사건 장소
                - description: 사건 설명
            compensation: 합의금 정보
            leniency_request: 선처 요청 여부

        Returns:
            str: 작성된 일반 형사사건 합의서
        """
        amount_korean = self._format_amount(compensation['amount'])

        document = f"""
                     합 의 서


피 해 자    성 명  {victim['name']}({victim['resident_number']})
            주 소  {victim['address']}
            전 화  {victim['phone']}

가 해 자    성 명  {perpetrator['name']}({perpetrator['resident_number']})
            주 소  {perpetrator['address']}
            전 화  {perpetrator['phone']}


{incident['date']} {incident.get('time', '')} {incident['location']}에서 발생한
{incident.get('case_name', '사건')}(이하 "이 사건"이라 합니다)과 관련하여
위 당사자는 다음과 같이 합의합니다.


                             - 다 음 -


1. 이 사건과 관련하여 가해자는 추가로 피해자에게 손해배상 및 위자료로
   금 {compensation['amount']:,}원({amount_korean}원)을 지급하고 위 피해자는 이를
   수령하여 상호 원만히 합의하였으므로 피해자는 이후 이 사건에 대하여
   민·형사상의 소송이나 어떠한 이의도 제기하지 않을 것임을 확약합니다.

2. 가해자는 위 합의금을 {compensation['payment_date']}까지
"""

        if compensation.get('bank_info'):
            bank_info = compensation['bank_info']
            document += f"""   {bank_info['bank']} {bank_info['account']} (예금주: {bank_info['holder']})로 입금한다.
"""
        else:
            document += """   피해자에게 직접 지급한다.
"""

        document += """
3. 피해자는 위 합의금을 수령하고 가해자의 형사처벌을 원하지 않으며,
   일체의 민사상 손해배상 청구권을 포기합니다.
"""

        if leniency_request:
            document += """
   (검사님 그리고 판사님! 가해자는 피해자에게 미안해하고 있고, 깊이 뉘우치고 있으므로
   최대한의 선처를 하여 주시기 바랍니다.)
"""

        document += f"""

4. 본 합의는 피해자와 가해자의 진정한 의사표시이며, 착오 또는 본 건 사고 관계자
   등으로부터 사기와 강박 등이 전혀 없이 평온·공연하게 합의한 것입니다.

5. 이를 증명하기 위하여 본 합의서에 서명 또는 날인합니다.


※ 첨부서류: 인감증명서 각 1통


                            {datetime.now().strftime('%Y')}년 {datetime.now().strftime('%m')}월 {datetime.now().strftime('%d')}일


위 피해자    {victim['name']}  (인)
             {victim['address']}

위 가해자    {perpetrator['name']}  (인)
             {perpetrator['address']}
"""

        return document


# 모듈 정보
__version__ = "1.0.0"
__author__ = "LawPro AI Team"
__reference__ = "형사소송문서 서식 (13,571~13,705줄)"
