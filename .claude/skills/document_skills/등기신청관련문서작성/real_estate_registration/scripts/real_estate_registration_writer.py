"""
부동산등기 신청서 작성 스킬
Real Estate Registration Writer
"""

from typing import Dict, Optional
from datetime import datetime

class RealEstateRegistrationWriter:
    """부동산등기 신청서를 작성하는 클래스"""

    def __init__(self):
        self.registration_types = self._load_registration_types()

    def _load_registration_types(self) -> Dict:
        """등기 유형 데이터베이스 로드"""
        return {
            "소유권보존": {"등록면허세율": 0.0008, "수수료": 15000},
            "소유권이전": {"등록면허세율": 0.02, "수수료": 15000},
            "저당권설정": {"등록면허세율": 0.002, "수수료": 6000},
            "전세권설정": {"등록면허세율": 0.002, "수수료": 6000},
            "말소": {"등록면허세율": 0, "수수료": 3000}
        }

    def write_ownership_transfer(self, registration_purpose: str, cause: str,
                                cause_date: str, land: Dict, building: Optional[Dict],
                                right_holder: Dict, duty_holder: Dict) -> str:
        """
        소유권이전등기 신청서 작성

        Args:
            registration_purpose: 등기 목적
            cause: 원인 (매매, 증여, 상속 등)
            cause_date: 원인일자
            land: 토지 정보
            building: 건물 정보 (선택)
            right_holder: 등기권리자 정보
            duty_holder: 등기의무자 정보

        Returns:
            완성된 등기신청서 (텍스트)
        """
        application = "등 기 신 청 서\n\n"
        application += f"등기 목적     {registration_purpose}\n\n"
        application += f"원        인     {cause_date} {cause}\n\n"

        # 부동산의 표시
        application += "부동산의 표시\n"
        if land:
            application += "  [토지]\n"
            application += f"  소    재     {land['location']}\n"
            application += f"  지    번     {land['lot_number']}\n"
            application += f"  지    목     {land['category']}\n"
            application += f"  면    적     {land['area']}\n\n"

        if building:
            application += "  [건물]\n"
            application += f"  소    재     {building['location']}\n"
            application += f"  건물번호     {building['building_number']}\n"
            if 'name' in building:
                application += f"  건물명칭     {building['name']}\n"
            application += f"  구    조     {building['structure']}\n"
            application += f"  면    적     {building['area']}\n\n"

        # 당사자
        application += f"등기권리자     {right_holder['address']}\n"
        application += f"               {right_holder['name']} (주민등록번호: {right_holder['resident_number']})\n\n"

        application += f"등기의무자     {duty_holder['address']}\n"
        application += f"               {duty_holder['name']} (주민등록번호: {duty_holder['resident_number']})\n\n"

        # 첨부서류
        application += "첨부서류\n"
        application += "  등기원인증명정보            1통\n"
        application += "  등기식별정보               1통\n"
        application += "  인감증명서 (등기의무자)     1통\n"
        application += "  주민등록표등본 (등기권리자) 1통\n"
        application += "  취득세납부영수필확인서      1통\n\n"

        # 수수료
        application += "등록면허세     금 ________원\n"
        application += "국민주택채권   금 ________원\n"
        application += "수 수 료       금     15,000원\n\n"

        application += f"{datetime.now().strftime('%Y년 %m월 %d일')}\n\n"

        application += f"위 신청인     {right_holder['name']} (인)\n"
        application += f"              {duty_holder['name']} (인)\n\n"

        application += "○○지방법원 ○○등기소 귀중\n"

        return application

    def write_mortgage_registration(self, mortgage_type: str, cause_date: str,
                                   land: Dict, mortgagee: Dict, mortgagor: Dict,
                                   mortgage_details: Dict) -> str:
        """
        저당권/근저당권 설정등기 신청서 작성

        Args:
            mortgage_type: 저당권 또는 근저당권
            cause_date: 원인일자
            land: 토지 정보
            mortgagee: 저당권자 정보
            mortgagor: 저당권설정자 정보
            mortgage_details: 저당권 상세정보 (채권최고액, 채무자 등)

        Returns:
            완성된 등기신청서
        """
        application = "등 기 신 청 서\n\n"
        application += f"등기 목적     {mortgage_type}설정\n\n"
        application += f"원        인     {cause_date} 설정\n\n"

        # 부동산의 표시
        application += "부동산의 표시\n"
        application += "  [토지]\n"
        application += f"  소    재     {land['location']}\n"
        application += f"  지    번     {land['lot_number']}\n"
        application += f"  지    목     {land['category']}\n"
        application += f"  면    적     {land['area']}\n\n"

        # 당사자
        if mortgagee['type'] == '법인':
            application += f"등기권리자     {mortgagee['address']}\n"
            application += f"               {mortgagee['name']} (법인등록번호: {mortgagee['registration_number']})\n\n"
        else:
            application += f"등기권리자     {mortgagee['address']}\n"
            application += f"               {mortgagee['name']} (주민등록번호: {mortgagee['resident_number']})\n\n"

        application += f"등기의무자     {mortgagor['address']}\n"
        application += f"               {mortgagor['name']} (주민등록번호: {mortgagor['resident_number']})\n\n"

        # 권리자 기타의 사항
        application += "권리자 기타의 사항\n"
        application += f"  채권최고액  금 {mortgage_details['maximum_amount']}원\n"
        application += f"  채  무  자  {mortgagor['address']}\n"
        application += f"              {mortgage_details['debtor']} (주민등록번호: {mortgagor['resident_number']})\n"
        application += f"  채권의범위  {mortgage_details['scope_of_claim']}\n\n"

        # 첨부서류
        application += "첨부서류\n"
        application += "  등기원인증명정보            1통\n"
        application += "  등기식별정보               1통\n"
        application += "  인감증명서 (등기의무자)     1통\n"
        if mortgagee['type'] == '법인':
            application += "  법인등기사항증명서 (등기권리자) 1통\n\n"
        else:
            application += "  주민등록표등본 (등기권리자) 1통\n\n"

        # 수수료
        max_amount = int(mortgage_details['maximum_amount'].replace(',', ''))
        tax = int(max_amount * 0.002)
        application += f"등록면허세     금 {tax:,}원\n"
        application += "수 수 료       금     6,000원\n\n"

        application += f"{datetime.now().strftime('%Y년 %m월 %d일')}\n\n"

        if mortgagee['type'] == '법인':
            application += f"위 신청인     {mortgagee['name']}\n"
            application += f"              대표이사 {mortgagee['representative']} (인)\n\n"
        else:
            application += f"위 신청인     {mortgagee['name']} (인)\n\n"

        application += f"              {mortgagor['name']} (인)\n\n"

        application += "○○지방법원 ○○등기소 귀중\n"

        return application


# 사용 예시
if __name__ == "__main__":
    writer = RealEstateRegistrationWriter()

    # 소유권이전등기 작성
    transfer_app = writer.write_ownership_transfer(
        registration_purpose="소유권이전",
        cause="매매",
        cause_date="2024년 11월 10일",
        land={
            "location": "서울특별시 강남구 역삼동",
            "lot_number": "123-45",
            "category": "대",
            "area": "200㎡"
        },
        building={
            "location": "서울특별시 강남구 역삼동",
            "building_number": "123-45",
            "name": "역삼타워",
            "structure": "철근콘크리트조",
            "area": "1층 150㎡"
        },
        right_holder={
            "name": "김철수",
            "resident_number": "800101-1234567",
            "address": "서울특별시 강남구 테헤란로 123"
        },
        duty_holder={
            "name": "이영희",
            "resident_number": "750505-2345678",
            "address": "서울특별시 강남구 역삼로 456"
        }
    )

    print("=" * 60)
    print("소유권이전등기 신청서")
    print("=" * 60)
    print(transfer_app)
