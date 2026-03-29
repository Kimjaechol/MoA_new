"""
형사 항고장 · 재항고장 작성 스킬
Criminal Appeal to Prosecutor Writer Skill

이 모듈은 형사사건에서 검사의 불기소처분에 불복하는 항고장 및 재항고장을
자동으로 작성합니다.
"""

from typing import Dict, List, Optional
from datetime import datetime


class AppealToProsecutorWriter:
    """형사 항고장 · 재항고장 작성 클래스"""

    def __init__(self):
        self.version = "1.0.0"

    def write_non_prosecution_appeal(
        self,
        case: Dict,
        suspect: Dict,
        complainant: Dict,
        appeal_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        무혐의 불기소처분 항고장 작성

        Args:
            case: 사건 정보
                - case_number: 사건번호
                - crime: 죄명
                - prosecutor_office: 처분 검찰청
                - prosecutor_name: 담당 검사 (optional)
                - disposition: 처분 내용 (예: 무혐의 불기소, 기소유예 등)
                - disposition_date: 처분일자 (optional)
                - disposition_reason: 처분 이유 요지 (optional)
            suspect: 피의자 정보
                - name: 성명
                - birth_date: 생년월일 (optional)
                - address: 주소 (optional)
            complainant: 고소인/고발인 정보
                - name: 성명
                - address: 주소
                - phone: 연락처 (optional)
                - registration: 등록기준지 (optional)
            appeal_grounds: 항고 이유
                - summary: 항고 이유 요약
                - reasons: 구체적 항고 이유 (list)
                - evidence: 제출 증거 (list, optional)
                - procedural_errors: 수사상 하자 (list, optional)
            additional_info: 추가 정보 (optional)
                - attorney: 변호인 (optional)

        Returns:
            작성된 항고장 (문자열)
        """
        doc = "항 고 장\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']}"
        if 'crime' in case:
            doc += f" {case['crime']}"
        doc += "\n"

        doc += f"피의자    {suspect['name']}"
        if 'birth_date' in suspect:
            doc += f" ({suspect['birth_date']} 생)"
        doc += "\n"

        if 'address' in suspect:
            doc += f"주    소  {suspect['address']}\n"

        doc += "\n"

        # 서두
        doc += f"위 자에 대한 귀청 {case['case_number']}호 "
        if 'crime' in case:
            doc += f"{case['crime']} "
        doc += "고소사건에 관하여 "
        doc += f"{case['prosecutor_office']} "
        if 'prosecutor_name' in case:
            doc += f"{case['prosecutor_name']} "
        doc += "검사는\n"
        doc += "이 사건에 대하여 "
        if 'disposition_date' in case:
            doc += f"{case['disposition_date']} 자로 "
        doc += f"{case['disposition']} 처분을 하였는바 다음과 같은\n"
        doc += "이유로 항고를 제기하는 바입니다.\n\n"

        # 항고 이유
        doc += "항고 이유\n\n"

        # 처분 이유 요지 (있는 경우)
        if 'disposition_reason' in case:
            doc += "검사의 불기소 이유의 요지는,\n\n"
            doc += f"{case['disposition_reason']}\n\n"

        # 항고 이유 요약
        if 'summary' in appeal_grounds:
            doc += f"{appeal_grounds['summary']}\n\n"

        # 구체적 항고 이유
        if 'reasons' in appeal_grounds and appeal_grounds['reasons']:
            for idx, reason in enumerate(appeal_grounds['reasons'], 1):
                doc += f"{idx}. {reason}\n\n"

        # 수사상 하자 (있는 경우)
        if 'procedural_errors' in appeal_grounds and appeal_grounds['procedural_errors']:
            doc += "수사상 하자:\n\n"
            for idx, error in enumerate(appeal_grounds['procedural_errors'], 1):
                doc += f"   {idx}) {error}\n"
            doc += "\n"

        # 결론
        doc += "제반사항을 종합검토하여 보면 본건 고소사실에 대한 증거가 충분하여 그 증명이\n"
        doc += "명백함으로 검사의 "
        if '무혐의' in case.get('disposition', ''):
            doc += "증거불충분하다는 이유로 "
        doc += "불기소처분한 것은\n"
        doc += "부당한 처분이므로 이에 항고장을 제출하오니 철저히 조사하여 공소제기명령을\n"
        doc += "하여 주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 불기소처분 통지서  1통\n"
        doc += "2. 불기소 이유 고지서  1통\n"

        if 'evidence' in appeal_grounds and appeal_grounds['evidence']:
            for idx, evidence in enumerate(appeal_grounds['evidence'], 3):
                doc += f"{idx}. {evidence}\n"

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"항고인 (고소인) {complainant['name']} (인)\n"
        if 'address' in complainant:
            doc += f"주소: {complainant['address']}\n"
        if 'phone' in complainant:
            doc += f"연락처: {complainant['phone']}\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변호인  {additional_info['attorney']}  (인)\n"

        doc += "\n"

        # 제출처 (상급 검찰청)
        higher_office = self._get_higher_prosecutor_office(case['prosecutor_office'])
        doc += f"{higher_office} 귀중\n"

        return doc

    def write_dismissal_appeal(
        self,
        case: Dict,
        suspect: Dict,
        complainant: Dict,
        appeal_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        기소유예처분 항고장 작성

        Args:
            case: 사건 정보
            suspect: 피의자 정보
            complainant: 고소인 정보
            appeal_grounds: 항고 이유
                - crime_severity: 범죄의 중대성
                - victim_damage: 피해의 심각성
                - public_interest: 공익상 필요성
                - reasons: 기타 사유 (list)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 항고장
        """
        doc = "항 고 장\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피의자    {suspect['name']}"
        if 'birth_date' in suspect:
            doc += f" ({suspect['birth_date']} 생)"
        doc += "\n\n"

        # 서두
        doc += f"위 자에 대한 귀청 {case['case_number']}호 {case['crime']} 고소사건에 관하여\n"
        doc += f"{case['prosecutor_office']} 검사는 이 사건에 대하여 기소유예 처분을 하였는바\n"
        doc += "다음과 같은 이유로 항고를 제기하는 바입니다.\n\n"

        # 항고 이유
        doc += "항고 이유\n\n"

        doc += "검사는 피의자의 범죄사실을 인정하면서도 정상을 참작하여 기소유예 처분을\n"
        doc += "하였으나, 다음과 같은 이유로 피의자를 반드시 처벌해야 합니다.\n\n"

        reason_idx = 1

        if 'crime_severity' in appeal_grounds:
            doc += f"{reason_idx}. 범죄의 중대성\n\n"
            doc += f"   {appeal_grounds['crime_severity']}\n\n"
            reason_idx += 1

        if 'victim_damage' in appeal_grounds:
            doc += f"{reason_idx}. 피해의 심각성\n\n"
            doc += f"   {appeal_grounds['victim_damage']}\n\n"
            reason_idx += 1

        if 'public_interest' in appeal_grounds:
            doc += f"{reason_idx}. 공익상 필요성\n\n"
            doc += f"   {appeal_grounds['public_interest']}\n\n"
            reason_idx += 1

        if 'reasons' in appeal_grounds and appeal_grounds['reasons']:
            for reason in appeal_grounds['reasons']:
                doc += f"{reason_idx}. {reason}\n\n"
                reason_idx += 1

        # 결론
        doc += "따라서 피의자에 대한 기소유예 처분은 부당하므로 이에 항고장을 제출하오니\n"
        doc += "재수사하여 공소를 제기하여 주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 불기소처분 통지서  1통\n"
        doc += "2. 불기소 이유 고지서  1통\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"항고인 (고소인) {complainant['name']} (인)\n\n"

        # 제출처
        higher_office = self._get_higher_prosecutor_office(case['prosecutor_office'])
        doc += f"{higher_office} 귀중\n"

        return doc

    def write_re_appeal(
        self,
        case: Dict,
        suspect: Dict,
        complainant: Dict,
        re_appeal_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        재항고장 작성 (항고 기각 후)

        Args:
            case: 사건 정보
                - case_number: 항고 사건번호
                - crime: 죄명
                - high_prosecutor_office: 항고심 검찰청
                - appeal_rejection_date: 항고기각 일자
                - original_case_number: 원 사건번호 (optional)
            suspect: 피의자 정보
            complainant: 재항고인 (고소인) 정보
            re_appeal_grounds: 재항고 이유
                - summary: 재항고 이유 요약
                - original_decision_summary: 원결정 요지
                - reasons: 구체적 재항고 이유 (list)
                - procedural_errors: 절차상 하자 (list, optional)
                - fact_finding_errors: 사실 오인 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 재항고장
        """
        doc = "재 항 고 장\n\n"

        # 사건 표시
        doc += f"재항고인 (고소인)  {complainant['name']}\n"
        if 'address' in complainant:
            doc += f"주        소  {complainant['address']}\n"

        doc += "\n"

        doc += f"피의자 (피항고인, 피재항고인)  {suspect['name']}\n"
        if 'birth_date' in suspect:
            doc += f"                              ({suspect['birth_date']} 생)\n"

        doc += "\n"

        # 서두
        doc += f"위 피의자에 대한 {case['high_prosecutor_office']} {case['case_number']}호 "
        doc += f"{case['crime']}\n"
        doc += "항고 사건에 관하여 "
        if 'appeal_rejection_date' in case:
            doc += f"{case['appeal_rejection_date']} "
        doc += "동청에서 항고 기각의 결정을\n"
        doc += "한바 동결정에 불복이므로 재항고 합니다.\n\n"

        # 재항고 취지
        doc += "재항고 취지\n\n"
        doc += "원심 결정을 파기하고 다시 상당한 조치를 바랍니다.\n\n"

        # 재항고 이유
        doc += "재항고 이유\n\n"

        # 원결정 요지
        if 'original_decision_summary' in re_appeal_grounds:
            doc += "1. 원 결정의 요지\n\n"
            doc += f"   {re_appeal_grounds['original_decision_summary']}\n\n"
            reason_start = 2
        else:
            reason_start = 1

        # 재항고 이유 요약
        if 'summary' in re_appeal_grounds:
            doc += f"{reason_start}. {re_appeal_grounds['summary']}\n\n"
            reason_start += 1

        # 구체적 재항고 이유
        if 'reasons' in re_appeal_grounds and re_appeal_grounds['reasons']:
            sub_idx = ord('가')
            for reason in re_appeal_grounds['reasons']:
                doc += f"   {chr(sub_idx)}. {reason}\n\n"
                sub_idx += 1

        # 사실 오인 (있는 경우)
        if 'fact_finding_errors' in re_appeal_grounds and re_appeal_grounds['fact_finding_errors']:
            doc += f"{reason_start}. 원결정의 사실 오인 및 채증법칙 위배\n\n"
            for idx, error in enumerate(re_appeal_grounds['fact_finding_errors'], 1):
                doc += f"   {idx}) {error}\n"
            doc += "\n"
            reason_start += 1

        # 절차상 하자 (있는 경우)
        if 'procedural_errors' in re_appeal_grounds and re_appeal_grounds['procedural_errors']:
            doc += f"{reason_start}. 수사 미진 및 절차상 하자\n\n"
            for idx, error in enumerate(re_appeal_grounds['procedural_errors'], 1):
                doc += f"   {idx}) {error}\n"
            doc += "\n"

        # 결론
        doc += "위와 같은 이유로 원결정은 채증법칙을 위배하여 사실을 오인하고 수사가\n"
        doc += "미진되었음을 이유로 재항고 하오니 고소인의 고소 사실에 대하여 엄정하게\n"
        doc += "수사하시와 재기 수사 명령을 내려주시기 바랍니다.\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재항고인 (고소인) {complainant['name']} (인)\n"
        if additional_info and 'attorney' in additional_info:
            doc += f"변호인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += "대 검 찰 청 귀중\n"

        return doc

    def write_detailed_appeal(
        self,
        case: Dict,
        suspect: Dict,
        complainant: Dict,
        complaint_summary: Dict,
        appeal_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        상세 항고장 작성 (고소 요지 포함)

        Args:
            case: 사건 정보
            suspect: 피의자 정보
            complainant: 고소인 정보
            complaint_summary: 고소 요지
                - incident_date: 사건 발생일
                - incident_location: 사건 장소
                - incident_description: 사건 경위
                - damages: 피해 내용
            appeal_grounds: 항고 이유
                - disposition_summary: 검사 처분 요지
                - counter_arguments: 반박 주장 (list)
                - evidence_analysis: 증거 분석 (list, optional)
                - legal_arguments: 법리적 주장 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 상세 항고장
        """
        doc = "항 고 장\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피의자 (피항고인)  {suspect['name']}"
        if 'birth_date' in suspect:
            doc += f" ({suspect['birth_date']} 생)"
        doc += "\n"
        if 'address' in suspect:
            doc += f"주    소  {suspect['address']}\n"

        doc += "\n"

        doc += f"항고인 (고소인)  {complainant['name']}\n"
        if 'address' in complainant:
            doc += f"주    소  {complainant['address']}\n"
        if 'phone' in complainant:
            doc += f"연락처  {complainant['phone']}\n"

        doc += "\n"

        # 서두
        doc += f"위 사건에 관하여 {case['prosecutor_office']} "
        if 'prosecutor_name' in case:
            doc += f"{case['prosecutor_name']} "
        doc += "검사는 "
        if 'disposition_date' in case:
            doc += f"{case['disposition_date']} 자로 "
        doc += f"{case['disposition']} 처분을 하였는바 다음과 같은 이유로 항고를 제기합니다.\n\n"

        # 항고 이유
        doc += "항고 이유\n\n"

        # 검사의 처분 이유
        if 'disposition_summary' in appeal_grounds:
            doc += "1. 검사의 불기소 이유의 요지는,\n\n"
            doc += f"{appeal_grounds['disposition_summary']}\n\n"

        # 고소 요지
        doc += "2. 고소인의 고소 요지는,\n\n"

        if 'incident_description' in complaint_summary:
            doc += f"{complaint_summary['incident_description']}\n\n"

        if 'damages' in complaint_summary:
            doc += f"피해 내용: {complaint_summary['damages']}\n\n"

        # 반박 주장
        doc += "3. 항고인의 주장\n\n"

        if 'counter_arguments' in appeal_grounds and appeal_grounds['counter_arguments']:
            sub_idx = ord('가')
            for argument in appeal_grounds['counter_arguments']:
                doc += f"   {chr(sub_idx)}. {argument}\n\n"
                sub_idx += 1

        # 증거 분석 (있는 경우)
        if 'evidence_analysis' in appeal_grounds and appeal_grounds['evidence_analysis']:
            doc += "4. 증거 분석\n\n"
            for idx, analysis in enumerate(appeal_grounds['evidence_analysis'], 1):
                doc += f"   {idx}) {analysis}\n"
            doc += "\n"

        # 법리적 주장 (있는 경우)
        if 'legal_arguments' in appeal_grounds and appeal_grounds['legal_arguments']:
            doc += "5. 법리적 주장\n\n"
            for idx, argument in enumerate(appeal_grounds['legal_arguments'], 1):
                doc += f"   {idx}) {argument}\n"
            doc += "\n"

        # 결론
        doc += "위와 같은 제반사정을 종합검토하여 보면 본건 고소사실에 대하여 피의자는\n"
        doc += "명백히 범죄를 범하였음에도 검사가 피의자의 주장만을 토대로 "
        doc += f"{case['disposition']} 처분을 한 것은\n"
        doc += "부당하므로 이에 불복 항고를 하오니 철저히 조사하여 공소제기명령을 하여\n"
        doc += "주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 불기소처분 통지서  1통\n"
        doc += "2. 불기소 이유 고지서  1통\n"
        if 'evidence' in appeal_grounds and appeal_grounds['evidence']:
            for idx, evidence in enumerate(appeal_grounds['evidence'], 3):
                doc += f"{idx}. {evidence}\n"
        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"항고인 (고소인) {complainant['name']} (인)\n"

        doc += "\n"

        # 제출처
        higher_office = self._get_higher_prosecutor_office(case['prosecutor_office'])
        doc += f"{higher_office} 귀중\n"

        return doc

    def _get_higher_prosecutor_office(self, prosecutor_office: str) -> str:
        """
        상급 검찰청 결정

        Args:
            prosecutor_office: 원처분 검찰청

        Returns:
            상급 검찰청명
        """
        # 지방검찰청 -> 고등검찰청
        if "지방검찰청" in prosecutor_office:
            # 서울/인천/수원 등 지방검찰청은 각각 고등검찰청 관할
            if "서울" in prosecutor_office:
                return "서울고등검찰청"
            elif "인천" in prosecutor_office or "수원" in prosecutor_office:
                return "서울고등검찰청"
            elif "대전" in prosecutor_office or "청주" in prosecutor_office:
                return "대전고등검찰청"
            elif "대구" in prosecutor_office:
                return "대구고등검찰청"
            elif "부산" in prosecutor_office or "울산" in prosecutor_office or "창원" in prosecutor_office:
                return "부산고등검찰청"
            elif "광주" in prosecutor_office or "전주" in prosecutor_office:
                return "광주고등검찰청"
            else:
                return "고등검찰청"
        # 지청 -> 지방검찰청
        elif "지청" in prosecutor_office:
            return prosecutor_office.replace("지청", "지방검찰청").split()[0] + " 지방검찰청"
        else:
            return "고등검찰청"


# 사용 예시
if __name__ == "__main__":
    writer = AppealToProsecutorWriter()

    # 무혐의 불기소처분 항고장 예시
    print("=" * 80)
    print("무혐의 불기소처분 항고장 예시")
    print("=" * 80)

    non_prosecution_appeal = writer.write_non_prosecution_appeal(
        case={
            "case_number": "2024형제1234",
            "crime": "폭력행위",
            "prosecutor_office": "서울북부지방검찰청",
            "disposition": "무혐의 불기소",
            "disposition_date": "2024. 5. 15.",
            "disposition_reason": "피의자는 고소인의 어깨부위를 잡고 가게 밖으로 끌어낸 사실은 있으나\n"
                                 "가슴을 쥐어박고 넘어뜨린 사실은 없다고 범행을 부인하며\n"
                                 "달리 피의사실을 입증할 증거가 없어 범죄혐의 없다는 것입니다."
        },
        suspect={
            "name": "이정근",
            "birth_date": "1985. 8. 7."
        },
        complainant={
            "name": "홍길동",
            "address": "서울 강남구 테헤란로 123",
            "phone": "010-1234-5678"
        },
        appeal_grounds={
            "summary": "검사의 증거불충분 판단은 부당합니다.",
            "reasons": [
                "피의자의 폭행 사실은 상해진단서(전치 3주) 및 목격자 2명의 진술로 명백히 입증됨",
                "검사가 중요 목격자인 현장 상인들을 충분히 조사하지 않는 등 수사 미진",
                "현장 CCTV 영상이 존재함에도 불구하고 이를 확보·분석하지 않음"
            ],
            "evidence": [
                "상해진단서 1통",
                "목격자 진술서 2통",
                "현장 사진 3매"
            ],
            "procedural_errors": [
                "중요 목격자 미조사",
                "객관적 증거(CCTV) 미확보"
            ]
        }
    )

    print(non_prosecution_appeal)
    print("\n")

    # 재항고장 예시
    print("=" * 80)
    print("재항고장 예시")
    print("=" * 80)

    re_appeal = writer.write_re_appeal(
        case={
            "case_number": "2024불항1125",
            "crime": "특정경제범죄가중처벌등에관한법률위반",
            "high_prosecutor_office": "서울고등검찰청",
            "appeal_rejection_date": "2024. 7. 31."
        },
        suspect={
            "name": "이철수",
            "birth_date": "1970. 3. 15."
        },
        complainant={
            "name": "김이정",
            "address": "서울 서초구 서초대로 1234"
        },
        re_appeal_grounds={
            "summary": "원결정은 채증법칙을 위배하여 사실을 오인하였습니다.",
            "original_decision_summary": "항고 기각 (증거 불충분으로 범죄 혐의 없음)",
            "reasons": [
                "피의자의 경제 능력으로 보아 거액의 자금을 조달할 능력이 전혀 없었음",
                "고소인이 직접 금융기관을 통해 자금을 송금한 사실이 입증됨",
                "피의자가 제출한 경리장부가 위조되었음이 생산월보와의 대조로 명백히 입증됨"
            ],
            "fact_finding_errors": [
                "피의자가 당시 극도로 어려운 경제 사정으로 차용 능력이 없었다는 사실 간과",
                "고소인이 직접 자금을 송금했다는 참고인 진술 배척의 부당함",
                "경리장부와 생산월보의 모순(노임 지급액 불일치)을 간과"
            ],
            "procedural_errors": [
                "중요 참고인인 조춘화에 대한 조사 소홀",
                "고소인이 제출한 녹취록에 대한 진실성 검토 미실시",
                "자금 출처에 대한 구체적 수사 미진"
            ]
        }
    )

    print(re_appeal)
    print("\n")

    # 상세 항고장 예시
    print("=" * 80)
    print("상세 항고장 예시")
    print("=" * 80)

    detailed_appeal = writer.write_detailed_appeal(
        case={
            "case_number": "2024형제5678",
            "crime": "사기",
            "prosecutor_office": "서울중앙지방검찰청",
            "prosecutor_name": "박정의",
            "disposition": "무혐의 불기소",
            "disposition_date": "2024. 6. 20."
        },
        suspect={
            "name": "김사기",
            "birth_date": "1975. 5. 10.",
            "address": "서울 송파구 올림픽로 456"
        },
        complainant={
            "name": "이피해",
            "address": "서울 강남구 테헤란로 789",
            "phone": "010-9876-5432"
        },
        complaint_summary={
            "incident_date": "2024. 3. 15.",
            "incident_location": "서울 강남구 소재 피의자 사무실",
            "incident_description": "피의자는 고소인에게 투자금의 2배를 보장하는 사업이라고 허위 사실을 말하며\n"
                                   "고소인으로부터 5,000만원을 편취하였습니다. 피의자는 실제로는 사업 계획도\n"
                                   "없었고 고소인의 투자금을 개인 용도로 사용하였습니다.",
            "damages": "투자금 5,000만원 편취 및 정신적 피해"
        },
        appeal_grounds={
            "disposition_summary": "피의자가 투자 사업을 추진했으나 시장 상황 악화로 실패했을 뿐\n"
                                 "처음부터 기망의 고의가 없었다고 판단하여 무혐의 처분",
            "counter_arguments": [
                "피의자는 고소인을 만난 시점에 이미 다른 투자자들로부터 사기 피해 고소를 당한 상태였음",
                "피의자가 제시한 사업계획서는 다른 회사의 자료를 무단 도용한 것으로 확인됨",
                "투자금은 사업과 무관하게 피의자의 개인 채무 변제와 유흥비로 사용됨",
                "피의자는 고소인 외에도 동일한 수법으로 10여 명을 기망하여 총 3억원을 편취"
            ],
            "evidence_analysis": [
                "피의자의 은행 거래내역: 투자금 수령 직후 개인 채무 변제 및 유흥업소 결제 확인",
                "다른 피해자들의 진술: 동일한 허위 사업계획서로 기망당했다는 일치된 진술",
                "사업계획서 출처 확인: 타사의 IR 자료를 무단 복제한 것으로 확인"
            ],
            "legal_arguments": [
                "기망행위의 고의: 처음부터 사업 의사 없이 투자금을 편취할 목적이었음",
                "대법원 2018도12345 판결: 유사 사례에서 사기죄 인정",
                "피해자 다수: 반복적·계획적 범행으로 엄정한 처벌 필요"
            ],
            "evidence": [
                "피의자 은행 거래내역서 1부",
                "다른 피해자 진술서 5부",
                "사업계획서 원본 대조 자료 1부"
            ]
        }
    )

    print(detailed_appeal)
