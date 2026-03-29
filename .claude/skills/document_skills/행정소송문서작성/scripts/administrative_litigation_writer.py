"""
행정소송문서작성 API

행정소송 소장을 자동으로 작성하는 Python API입니다.
세무소송(취득세·양도소득세·상속세·법인세·증여세 부과처분취소)과
각종 행정소송(운전면허취소·징계처분취소·건축허가거부·토지수용재결취소 등)의
소장 작성을 지원합니다.

참조: 행정소송 소장 (백영사, 19,363줄)
"""

from datetime import datetime
from typing import Dict, List, Optional


class AdministrativeLitigationWriter:
    """행정소송 소장 작성 클래스"""

    def __init__(self):
        self.reference_data = "행정소송 소장 (백영사, 19,363줄)"
        self.version = "1.0.0"

    def write_acquisition_tax_complaint(
        self,
        plaintiff: Dict,
        defendant: Dict,
        land_info: Dict,
        tax_amount: int,
        acquisition_purpose: str,
        use_plan: str,
        legal_grounds: str,
        preliminary_procedure: Dict,
        evidence: List[str],
        court: str
    ) -> str:
        """
        취득세부과처분취소 청구의 소장 작성

        Args:
            plaintiff: 원고 정보 (type, name, address, representative, attorney)
            defendant: 피고 정보 (name, address)
            land_info: 토지 정보 (location, area, acquisition_date)
            tax_amount: 부과된 취득세 금액
            acquisition_purpose: 취득 목적
            use_plan: 사용 계획 및 경위
            legal_grounds: 정당한 사유
            preliminary_procedure: 전치절차 정보
            evidence: 입증방법 목록
            court: 관할 법원

        Returns:
            작성된 소장 문서
        """
        doc = f"""소    장

원      고  {plaintiff['name']}
            {plaintiff['address']}
"""

        if plaintiff['type'] == '법인':
            doc += f"""            대표이사  {plaintiff['representative']}
"""

        if plaintiff.get('attorney'):
            doc += f"""            소송대리인 {plaintiff['attorney']}

피      고  {defendant['name']}
            {defendant.get('address', '')}

취득세부과처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 {preliminary_procedure['disposition_date']}한 취득세
금 {tax_amount:,}원의 부과처분은 이를 취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 {acquisition_purpose}을(를) 위하여 다음 토지를 취득하였습니다.

   {land_info['location']} {land_info['area']}
   (취득일: {land_info['acquisition_date']})

2. 원고가 위 토지를 취득한 날로부터 1년 이내에 고유 목적에 직접 사용하지
   못한 것은 다음과 같은 정당한 사유가 있었습니다.

   {legal_grounds}

3. {use_plan}

4. 따라서 원고는 이 사건 토지를 취득한 이래 지금까지 당초의 계획대로
   {acquisition_purpose}을(를) 위한 부지로 사용하고 있을 뿐 다른 용도로
   사용하려는 계획이나 사용한 사실이 전혀 없습니다.

5. 그런데 피고는 원고가 이 사건 토지를 취득한 날로부터 1년 이내에 정당한
   사유없이 그 고유목적에 직접 사용하지 아니하였다는 이유로 지방세법
   제112조 제2항, 같은 법 시행령 제84조의 4 제1항에 의하여
   {preliminary_procedure['disposition_date']}원고에게 취득세를
   부과고지하였습니다.

6. 그러나 관계법령의 규정을 보건대, 지방세법 제112조 제2항, 같은법 시행령
   제84조의 4 제1항은 "법인의 비업무용 토지의 범위에 관하여 법인이 토지를
   취득한 날로부터 1년 이내에 정당한 사유없이 그 법인의 고유업무에 직접
   사용하지 아니하는 토지를 말한다"고 규정하고 있고, 위 법 시행령
   제84조의 4 제1항 소정의 정당한 사유라는 것은 법령에 의한 금지, 제한 등
   법인이 마음대로 할 수 없는 외부적인 사유를 뜻하는 것이 원칙입니다
   (대법원 1992. 6. 23. 선고 92누 1773 판결 참조).

7. 앞에서 본 이 사건의 사실관계를 법령에 비추어 보면, 원고가 이 사건 토지를
   취득한 날로부터 1년 이내에 그 고유업무에 직접 사용하지 못한 것은
   법령에 의한 제한으로 원고가 마음대로 할 수 없었던 외부적인 사유 때문이었
   으므로, 이는 지방세법 시행령 제84조의 4 제1항 소정의 "정당한 사유"에
   해당한다 할 것입니다.

8. 따라서 피고의 이 사건 부과처분은 위법·부당하여 마땅히 취소되어야 할
   것이므로, 원고는 다음과 같은 전치절차를 거쳐 이 사건 청구에 이릅니다.

전 치 절 차

가. {preliminary_procedure.get('disposition_date', '')}     처분일
나. {preliminary_procedure.get('receipt_date', '')}          고지서 수령일
다. {preliminary_procedure.get('review_request_date', '')}   심사청구일
라. {preliminary_procedure.get('review_decision_date', '')}  심사청구 기각일
마. {preliminary_procedure.get('appeal_request_date', '')}   심판청구일
바. {preliminary_procedure.get('appeal_decision_date', '')}  심판청구 기각일

입 증 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 입증방법            각 1통
"""

        if plaintiff['type'] == '법인':
            doc += "1. 법인등기부등본      1통\n"

        doc += """1. 납부서              1통
1. 위임장              1통
1. 소장부본            1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

원고 {plaintiff.get('attorney', plaintiff['name'])}

{court} 귀중
"""
        return doc

    def write_capital_gains_tax_complaint(
        self,
        plaintiff: Dict,
        defendant: str,
        land_info: Dict,
        acquisition_price: int,
        transfer_price: int,
        taxed_amount: int,
        dispute_reason: str,
        land_use_status: str,
        preliminary_procedure: Dict,
        evidence: List[str],
        court: str
    ) -> str:
        """
        양도소득세부과처분취소 청구의 소장 작성

        Args:
            plaintiff: 원고 정보
            defendant: 피고 (세무서장)
            land_info: 토지 정보
            acquisition_price: 실제 취득가액
            transfer_price: 실제 양도가액
            taxed_amount: 부과된 양도소득세
            dispute_reason: 다툼 사유
            land_use_status: 토지 이용 상황
            preliminary_procedure: 전치절차
            evidence: 입증방법
            court: 관할 법원

        Returns:
            작성된 소장 문서
        """
        doc = f"""소    장

원      고  {plaintiff['name']}
            {plaintiff['address']}
            주민등록번호: {plaintiff.get('resident_number', 'XXXXXX-XXXXXXX')}

피      고  {defendant}

양도소득세부과처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 {preliminary_procedure['disposition_date']}한
양도소득세 금 {taxed_amount:,}원의 부과처분은 이를 취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 다음 토지를 매수하였다가 이를 매도하였습니다.

   가. 소재지: {land_info['location']}
   나. 지목: {land_info.get('land_type', '답')}
   다. 면적: {land_info['area']}
   라. 취득일: {land_info['acquisition_date']}
   마. 취득가액: 금 {acquisition_price:,}원
   바. 양도일: {land_info.get('transfer_date', '')}
   사. 양도가액: 금 {transfer_price:,}원

2. {dispute_reason}

3. {land_use_status}

4. 따라서 {dispute_reason}을(를) 전제로 한 피고의 이 사건 부과처분은
   위법·부당하므로 원고는 그 취소를 구하고자 다음과 같은 전치절차를 거쳐
   이 사건 청구에 이릅니다.

전 치 절 차

가. {preliminary_procedure.get('disposition_date', '')}     처분일
나. {preliminary_procedure.get('receipt_date', '')}          고지서 송달일
다. {preliminary_procedure.get('review_request_date', '')}   심사청구일
라. {preliminary_procedure.get('review_decision_date', '')}  심사청구 기각일
마. {preliminary_procedure.get('appeal_request_date', '')}   심판청구일
바. {preliminary_procedure.get('appeal_decision_date', '')}  심판청구 기각일

입 증 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 입증방법            각 1통
1. 납부서              1통
1. 위임장              1통
1. 소장부본            1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

원고 {plaintiff.get('attorney', plaintiff['name'])}

{court} 귀중
"""
        return doc

    def write_license_revocation_complaint(
        self,
        plaintiff: Dict,
        defendant: str,
        license_info: Dict,
        revocation_date: str,
        violation_facts: str,
        mitigation_grounds: str,
        preliminary_procedure: Dict,
        evidence: List[str],
        court: str
    ) -> str:
        """
        자동차운전면허취소처분취소 청구의 소장 작성

        Args:
            plaintiff: 원고 정보
            defendant: 피고 (경찰서장, 지방경찰청장)
            license_info: 면허 정보
            revocation_date: 취소 처분일
            violation_facts: 위반 사실
            mitigation_grounds: 참작 사유
            preliminary_procedure: 전치절차
            evidence: 입증방법
            court: 관할 법원

        Returns:
            작성된 소장 문서
        """
        doc = f"""소    장

원      고  {plaintiff['name']}
            {plaintiff['address']}
            주민등록번호: {plaintiff.get('resident_number', 'XXXXXX-XXXXXXX')}
            전화: {plaintiff.get('phone', '')}

피      고  {defendant}

자동차운전면허취소처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 {revocation_date}한 자동차운전면허 취소처분을
취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 {license_info['acquisition_date']}제{license_info['type']} 운전면허를
   취득한 이래 운전을 하여 왔습니다.

2. 원고는 {violation_facts}(으)로 단속되어 피고로부터 {revocation_date}
   운전면허 취소처분을 받았습니다.

3. 그러나 다음과 같은 사정을 고려할 때 이 사건 처분은 재량권을 일탈·남용한
   것으로 위법·부당합니다.

{mitigation_grounds}

4. 이 사건의 경우 면허 취소보다는 면허 정지 처분이 적절하며, 취소 처분은
   비례의 원칙에 위반됩니다.

5. 따라서 피고의 이 사건 처분은 재량권을 일탈·남용한 위법·부당한 처분이므로
   원고는 다음과 같은 전치절차를 거쳐 그 취소를 구하고자 이 사건 청구에
   이릅니다.

전 치 절 차

가. {preliminary_procedure.get('disposition_date', revocation_date)}     처분일
나. {preliminary_procedure.get('appeal_request_date', '')}               행정심판 청구일
다. {preliminary_procedure.get('appeal_decision_date', '')}              행정심판 기각일

입 증 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 입증방법            각 1통
1. 납부서              1통
1. 위임장              1통
1. 소장부본            1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

원고 {plaintiff.get('attorney', plaintiff['name'])}

{court} 귀중
"""
        return doc

    def write_disciplinary_action_complaint(
        self,
        plaintiff: Dict,
        defendant: str,
        employment_info: Dict,
        disciplinary_type: str,
        disciplinary_date: str,
        alleged_facts: str,
        defense_grounds: str,
        mitigation_grounds: str,
        preliminary_procedure: Dict,
        evidence: List[str],
        court: str
    ) -> str:
        """
        징계처분취소 청구의 소장 작성

        Args:
            plaintiff: 원고 (공무원)
            defendant: 피고 (임용권자)
            employment_info: 임용 정보
            disciplinary_type: 징계 종류 (파면, 해임, 강등, 정직, 감봉, 견책)
            disciplinary_date: 징계 처분일
            alleged_facts: 징계 사유로 된 사실
            defense_grounds: 방어 사유
            mitigation_grounds: 참작 사유
            preliminary_procedure: 소청심사 결정
            evidence: 입증방법
            court: 관할 법원

        Returns:
            작성된 소장 문서
        """
        doc = f"""소    장

원      고  {plaintiff['name']}
            {plaintiff['address']}
            전화: {plaintiff.get('phone', '')}

피      고  {defendant}

{disciplinary_type}처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 {disciplinary_date}한 {disciplinary_type}처분을
취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 {employment_info['appointment_date']}
   {employment_info['position']}(으)로 임용되어 근무하여 왔습니다.

2. 피고는 {disciplinary_date}원고가 다음과 같은 사유로 {employment_info.get('relevant_law', '국가공무원법')}
   제78조의 징계사유에 해당한다는 이유로 {disciplinary_type}처분을 하였습니다.

   {alleged_facts}

3. 그러나 이 사건 {disciplinary_type}처분은 다음과 같은 이유로 위법·부당합니다.

   가. {defense_grounds}

   나. 참작사유
   {mitigation_grounds}

4. 따라서 피고의 이 사건 {disciplinary_type}처분은 징계사유가 없거나,
   징계양정이 지나치게 가혹하여 재량권을 일탈·남용한 위법·부당한 처분이므로
   원고는 다음과 같은 전치절차를 거쳐 그 취소를 구하고자 이 사건 청구에
   이릅니다.

전 치 절 차

가. {preliminary_procedure.get('disposition_date', disciplinary_date)}     처분일
나. {preliminary_procedure.get('appeal_request_date', '')}                 소청심사 청구일
다. {preliminary_procedure.get('appeal_decision_date', '')}                소청심사 기각일

입 증 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 입증방법            각 1통
1. 납부서              1통
1. 위임장              1통
1. 소장부본            1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

원고 {plaintiff.get('attorney', plaintiff['name'])}

{court} 귀중
"""
        return doc

    def write_building_permit_denial_complaint(
        self,
        plaintiff: Dict,
        defendant: str,
        land_info: Dict,
        building_plan: Dict,
        permit_application_date: str,
        denial_date: str,
        denial_reason: str,
        legal_compliance: str,
        evidence: List[str],
        court: str
    ) -> str:
        """
        건축허가거부처분취소 청구의 소장 작성

        Args:
            plaintiff: 원고 (건축주)
            defendant: 피고 (구청장, 시장 등)
            land_info: 대지 정보
            building_plan: 건축 계획
            permit_application_date: 허가 신청일
            denial_date: 거부 처분일
            denial_reason: 거부 사유
            legal_compliance: 법령 준수 사항
            evidence: 입증방법
            court: 관할 법원

        Returns:
            작성된 소장 문서
        """
        doc = f"""소    장

원      고  {plaintiff['name']}
            {plaintiff['address']}

피      고  {defendant}

건축허가거부처분취소 청구의 소

청 구 취 지

피고가 원고에 대하여 {denial_date}한 건축허가 거부처분을 취소한다.
소송비용은 피고의 부담으로 한다.
라는 판결을 구합니다.

청 구 원 인

1. 원고는 다음 대지에 건물을 신축하고자 {permit_application_date}피고에게
   건축허가를 신청하였습니다.

   가. 대지 소재지: {land_info['location']}
   나. 대지 면적: {land_info['area']}
   다. 용도지역: {land_info.get('zone', '')}

2. 건축계획은 다음과 같습니다.

   가. 건물 용도: {building_plan['usage']}
   나. 건물 구조: {building_plan.get('structure', '')}
   다. 건물 규모: {building_plan.get('scale', '')}
   라. 건폐율: {building_plan.get('building_coverage_ratio', '')}
   마. 용적률: {building_plan.get('floor_area_ratio', '')}

3. 그런데 피고는 {denial_date}다음과 같은 이유로 원고의 건축허가 신청을
   거부하였습니다.

   {denial_reason}

4. 그러나 피고의 이 사건 거부처분은 다음과 같은 이유로 위법·부당합니다.

   {legal_compliance}

5. 따라서 피고의 이 사건 거부처분은 법령을 오해하거나 재량권을 일탈·남용한
   위법·부당한 처분이므로 원고는 그 취소를 구하고자 이 사건 청구에 이릅니다.

입 증 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 입증방법            각 1통
1. 토지등기부등본      1통
1. 납부서              1통
1. 위임장              1통
1. 소장부본            1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

원고 {plaintiff.get('attorney', plaintiff['name'])}

{court} 귀중
"""
        return doc

    def write_suspension_of_execution_application(
        self,
        applicant: Dict,
        respondent: str,
        main_case: str,
        disposition: str,
        irreparable_damage: str,
        urgent_necessity: str,
        evidence: List[str],
        court: str
    ) -> str:
        """
        행정처분 효력정지 가처분 신청서 작성

        Args:
            applicant: 신청인
            respondent: 피신청인
            main_case: 본안 사건 (사건번호)
            disposition: 처분 내용
            irreparable_damage: 회복하기 어려운 손해
            urgent_necessity: 긴급한 필요
            evidence: 입증방법
            court: 관할 법원

        Returns:
            작성된 신청서
        """
        doc = f"""집행정지신청서

신  청  인  {applicant['name']}
            {applicant['address']}

피신청인  {respondent}

본안사건  {court} {main_case}

집 행 정 지 신 청

신 청 취 지

피신청인이 신청인에 대하여 한 {disposition}은(는) {court} {main_case}
사건의 판결이 확정될 때까지 그 효력을 정지한다.
신청비용은 피신청인의 부담으로 한다.
라는 결정을 구합니다.

신 청 이 유

1. 신청인은 피신청인을 상대로 위 본안사건을 제기하여 현재 계속 중입니다.

2. 피신청인은 신청인에 대하여 {disposition}을(를) 하였습니다.

3. 그러나 이 사건 처분으로 인하여 신청인은 다음과 같은 회복하기 어려운
   손해를 입게 됩니다.

   {irreparable_damage}

4. 또한 다음과 같은 긴급한 필요가 있습니다.

   {urgent_necessity}

5. 이 사건 처분의 효력을 정지하더라도 공공복리에 중대한 영향을 미칠
   우려가 없습니다.

6. 따라서 행정소송법 제23조에 따라 이 사건 처분의 효력을 정지할 필요가
   있으므로 이 사건 신청에 이릅니다.

소 명 방 법

"""

        for idx, ev in enumerate(evidence, 1):
            doc += f"갑 제{idx}호증     {ev}\n"

        doc += f"""
첨 부 서 류

1. 소명방법            각 1통
1. 신청서부본          1통

{datetime.now().year}.  {datetime.now().month}.  {datetime.now().day}.

신청인 {applicant.get('attorney', applicant['name'])}

{court} 귀중
"""
        return doc

    def format_money(self, amount: int) -> str:
        """금액을 한글 형식으로 변환"""
        units = ['', '만', '억', '조']
        result = []

        for i, unit in enumerate(units):
            part = amount % 10000
            if part > 0:
                result.append(f"금 {part:,}{unit}원")
            amount //= 10000
            if amount == 0:
                break

        return ' '.join(reversed(result)) if result else '금 0원'

    def generate_evidence_list(self, evidence: List[str]) -> str:
        """입증방법 목록 생성"""
        result = "입 증 방 법\n\n"
        for idx, ev in enumerate(evidence, 1):
            result += f"갑 제{idx}호증     {ev}\n"
        return result

    def generate_attachments(self, plaintiff_type: str = '개인', has_attorney: bool = True) -> str:
        """첨부서류 목록 생성"""
        result = "첨 부 서 류\n\n1. 입증방법            각 1통\n"

        if plaintiff_type == '법인':
            result += "1. 법인등기부등본      1통\n"

        result += "1. 납부서              1통\n"

        if has_attorney:
            result += "1. 위임장              1통\n"

        result += "1. 소장부본            1통\n"

        return result


# 사용 예시
if __name__ == "__main__":
    writer = AdministrativeLitigationWriter()

    # 취득세부과처분취소 소장 작성 예시
    complaint = writer.write_acquisition_tax_complaint(
        plaintiff={
            "type": "법인",
            "name": "주식회사 ○○",
            "address": "서울시 종로구 ○○동 1234",
            "representative": "김○○",
            "attorney": "변호사 이○○"
        },
        defendant={
            "name": "서울시 종로구청장",
            "address": "서울시 종로구 ○○동 5678"
        },
        land_info={
            "location": "서울시 종로구 ○○동 1284의 2",
            "area": "2,449㎡",
            "acquisition_date": "2020. 6. 10."
        },
        tax_amount=174159420,
        acquisition_purpose="업무용 건물 신축 부지",
        use_plan="백화점 건물 신축 계획으로 설계 용역 계약 체결 및 건축허가 취득",
        legal_grounds="건설부장관의 건축허가 제한조치 (2020.5.15~2020.12.31)",
        preliminary_procedure={
            "disposition_date": "2020. 2. 10.",
            "receipt_date": "2020. 2. 15.",
            "review_request_date": "2020. 3. 15.",
            "review_decision_date": "2020. 4. 10.",
            "appeal_request_date": "2020. 5. 10.",
            "appeal_decision_date": "2020. 6. 15."
        },
        evidence=[
            "납세고지서",
            "심사청구 결정서",
            "국세심판결정통지",
            "토지등기부등본",
            "건축허가신청서"
        ],
        court="서울행정법원"
    )

    print(complaint)
