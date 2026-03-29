"""
형사 항소이유서 작성 스킬
Criminal Appeal Reason Statement Writer Skill

이 모듈은 형사사건에서 1심 판결에 불복하여 항소심에 제출하는
항소이유서를 자동으로 작성합니다.
"""

from typing import Dict, List, Optional
from datetime import datetime


class AppealReasonWriter:
    """형사 항소이유서 작성 클래스"""

    def __init__(self):
        self.version = "1.0.0"

    def write_sentencing_appeal(
        self,
        case: Dict,
        defendant: Dict,
        appeal_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        양형부당 항소이유서 작성

        Args:
            case: 사건 정보
                - court: 원심 법원명
                - case_number: 원심 사건번호
                - crime: 죄명
                - original_sentence: 원심 선고 형량
                - sentence_date: 선고일자 (optional)
            defendant: 피고인 정보
                - name: 성명
                - birth_date: 생년월일
                - residence: 주소 (optional)
                - registration: 등록기준지 (optional)
            appeal_grounds: 항소 이유
                - first_offense: 초범 여부 (bool)
                - remorse: 반성 내용
                - victim_settlement: 피해자 합의 여부 (bool)
                - settlement_amount: 합의금 또는 배상액 (optional)
                - family_situation: 가정환경 (optional)
                - social_contribution: 사회적 기여 (optional)
                - health_condition: 건강 상태 (optional)
                - pretrial_detention_days: 미결구금 일수 (optional)
                - employment: 직업 (optional)
                - other_circumstances: 기타 참작사항 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 항소이유서 (문자열)
        """
        doc = "항 소 이 유 서\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피 고 인  {defendant['name']}"
        if 'birth_date' in defendant:
            doc += f" ({defendant['birth_date']} 생)"
        doc += "\n"

        if 'residence' in defendant:
            doc += f"주    소  {defendant['residence']}\n"
        if 'registration' in defendant:
            doc += f"등록기준지  {defendant['registration']}\n"

        doc += "\n"

        # 서두
        doc += f"위 피고인에 대한 귀원 {case['case_number']}호 {case['crime']} "
        doc += "피고사건에 관하여 "
        if 'sentence_date' in case:
            doc += f"원심에서 {case['sentence_date']} "
        doc += f"{case['original_sentence']}형을 선고받았으나 이에 불복으로 항소를 제기하였는바 "
        doc += "다음과 같이 항소이유를 개진합니다.\n\n"

        doc += "다    음\n\n"

        # 항소 이유
        doc += "1. 원심 판결의 양형 부당\n\n"
        doc += f"   원심은 피고인에게 {case['original_sentence']}의 "
        if "징역" in case['original_sentence'] and "집행유예" not in case['original_sentence']:
            doc += "실형을 "
        else:
            doc += "형을 "
        doc += "선고하였으나, 이는 다음과 같은 이유로\n"
        doc += "   지나치게 무거워 부당합니다.\n\n"

        # 구체적 양형 이유
        reason_index = ord('가')

        if appeal_grounds.get('first_offense', False):
            doc += f"   {chr(reason_index)}. 피고인은 이 사건이 초범입니다.\n\n"
            reason_index += 1

        if 'remorse' in appeal_grounds:
            doc += f"   {chr(reason_index)}. 피고인은 {appeal_grounds['remorse']}으며, "
            doc += "재범하지 않을 것을 다짐하고 있습니다.\n\n"
            reason_index += 1

        if appeal_grounds.get('victim_settlement', False):
            doc += f"   {chr(reason_index)}. 피고인은 피해자와 합의하여 "
            if 'settlement_amount' in appeal_grounds:
                doc += f"피해금 {appeal_grounds['settlement_amount']} 전액을 변제하였고, "
            doc += "피해자는 피고인에 대한 선처를 탄원하고 있습니다.\n\n"
            reason_index += 1

        if 'family_situation' in appeal_grounds:
            doc += f"   {chr(reason_index)}. 피고인은 {appeal_grounds['family_situation']}로서, "
            if "징역" in case['original_sentence'] and "집행유예" not in case['original_sentence']:
                doc += "실형을\n"
                doc += "       선고받을 경우 가족의 생계에 심각한 어려움이 발생합니다.\n\n"
            else:
                doc += "가족에 대한 책임을 다하고 있습니다.\n\n"
            reason_index += 1

        if 'employment' in appeal_grounds:
            doc += f"   {chr(reason_index)}. 피고인은 {appeal_grounds['employment']}으로 "
            doc += "성실히 근무하고 있으며, 실형을 선고받을 경우 직장을 잃게 되어\n"
            doc += "       생활 기반이 무너지게 됩니다.\n\n"
            reason_index += 1

        if 'social_contribution' in appeal_grounds:
            doc += f"   {chr(reason_index)}. 피고인은 {appeal_grounds['social_contribution']} "
            doc += "등 사회에 기여해 왔습니다.\n\n"
            reason_index += 1

        if 'health_condition' in appeal_grounds:
            doc += f"   {chr(reason_index)}. 피고인은 {appeal_grounds['health_condition']}으로 "
            doc += "건강이 좋지 않아\n"
            doc += "       수형생활에 어려움이 예상됩니다.\n\n"
            reason_index += 1

        if 'pretrial_detention_days' in appeal_grounds and appeal_grounds['pretrial_detention_days'] > 0:
            doc += f"   {chr(reason_index)}. 피고인은 구속 후 {appeal_grounds['pretrial_detention_days']}일간 "
            doc += "미결구금되어 이미 상당한 불이익을 받았습니다.\n\n"
            reason_index += 1

        if 'other_circumstances' in appeal_grounds:
            for circumstance in appeal_grounds['other_circumstances']:
                doc += f"   {chr(reason_index)}. {circumstance}\n\n"
                reason_index += 1

        # 결론
        doc += "2. 결론\n\n"
        doc += "   이상과 같은 정상을 참작하여 원심보다 관대한 판결을 바라면서 "
        doc += "항소이유서를\n"
        doc += "   제출합니다.\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"피고인  {defendant['name']}  (인)\n"
        if additional_info and 'attorney' in additional_info:
            doc += f"변호인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{case['court']} 귀중\n"

        return doc

    def write_factual_error_appeal(
        self,
        case: Dict,
        defendant: Dict,
        factual_errors: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        사실오인 항소이유서 작성

        Args:
            case: 사건 정보
            defendant: 피고인 정보
            factual_errors: 사실오인 내용
                - evidence_credibility: 증거 신빙성 문제 (list)
                - favorable_evidence_ignored: 유리한 증거 간과 (list)
                - logical_contradictions: 논리적 모순 (list)
                - witness_credibility_issues: 증인 신빙성 문제 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 항소이유서
        """
        doc = "항 소 이 유 서\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피 고 인  {defendant['name']}"
        if 'birth_date' in defendant:
            doc += f" ({defendant['birth_date']} 생)"
        doc += "\n\n"

        # 서두
        doc += f"위 피고인에 대한 귀원 {case['case_number']}호 {case['crime']} "
        doc += "피고사건에 관하여 "
        doc += f"원심에서 {case['original_sentence']}형을 선고받았으나 이에 불복으로 항소를 제기하였는바 "
        doc += "다음과 같이 항소이유를 개진합니다.\n\n"

        doc += "다    음\n\n"

        # 항소 이유
        doc += "1. 원심 판결의 사실오인 및 채증법칙 위배\n\n"
        doc += "   원심은 증거를 잘못 평가하여 사실을 오인하였고, 이는 채증법칙에 위배되어\n"
        doc += "   부당합니다.\n\n"

        reason_index = ord('가')

        # 증거 신빙성 문제
        if 'evidence_credibility' in factual_errors and factual_errors['evidence_credibility']:
            doc += f"   {chr(reason_index)}. 원심이 채택한 증거의 신빙성 부족\n\n"
            for idx, issue in enumerate(factual_errors['evidence_credibility'], 1):
                doc += f"      {idx}) {issue}\n"
            doc += "\n"
            reason_index += 1

        # 유리한 증거 간과
        if 'favorable_evidence_ignored' in factual_errors and factual_errors['favorable_evidence_ignored']:
            doc += f"   {chr(reason_index)}. 피고인에게 유리한 증거를 간과한 잘못\n\n"
            for idx, evidence in enumerate(factual_errors['favorable_evidence_ignored'], 1):
                doc += f"      {idx}) {evidence}\n"
            doc += "\n"
            reason_index += 1

        # 논리적 모순
        if 'logical_contradictions' in factual_errors and factual_errors['logical_contradictions']:
            doc += f"   {chr(reason_index)}. 경험칙 및 논리법칙 위배\n\n"
            for idx, contradiction in enumerate(factual_errors['logical_contradictions'], 1):
                doc += f"      {idx}) {contradiction}\n"
            doc += "\n"
            reason_index += 1

        # 증인 신빙성 문제
        if 'witness_credibility_issues' in factual_errors and factual_errors['witness_credibility_issues']:
            doc += f"   {chr(reason_index)}. 증인 진술의 신빙성 문제\n\n"
            for idx, issue in enumerate(factual_errors['witness_credibility_issues'], 1):
                doc += f"      {idx}) {issue}\n"
            doc += "\n"
            reason_index += 1

        # 결론
        doc += "2. 결론\n\n"
        doc += "   이상과 같이 원심은 증거를 잘못 평가하여 사실을 오인하였으므로, 원심을\n"
        doc += "   파기하고 피고인에게 무죄를 선고하여 주시기 바라면서 항소이유서를\n"
        doc += "   제출합니다.\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"피고인  {defendant['name']}  (인)\n"
        if additional_info and 'attorney' in additional_info:
            doc += f"변호인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{case['court']} 귀중\n"

        return doc

    def write_legal_error_appeal(
        self,
        case: Dict,
        defendant: Dict,
        legal_errors: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        법리오해 항소이유서 작성

        Args:
            case: 사건 정보
            defendant: 피고인 정보
            legal_errors: 법리오해 내용
                - misinterpretation: 법률 해석 착오 (list)
                - justification: 위법성 조각사유 (list, optional)
                - precedent_violation: 판례 위반 (list, optional)
                - procedural_errors: 절차상 하자 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 항소이유서
        """
        doc = "항 소 이 유 서\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피 고 인  {defendant['name']}"
        if 'birth_date' in defendant:
            doc += f" ({defendant['birth_date']} 생)"
        doc += "\n\n"

        # 서두
        doc += f"위 피고인에 대한 귀원 {case['case_number']}호 {case['crime']} "
        doc += "피고사건에 관하여 "
        if 'attorney' in (additional_info or {}):
            doc += "피고인의 변호인은 "
        doc += "다음과 같이 항소이유를 개진합니다.\n\n"

        doc += "다    음\n\n"

        # 항소 이유
        doc += "1. 원심의 법리오해\n\n"
        doc += "   원심은 관련 법률의 해석 및 적용을 잘못하여 판결에 영향을 미친 위법이\n"
        doc += "   있습니다.\n\n"

        reason_index = ord('가')

        # 법률 해석 착오
        if 'misinterpretation' in legal_errors and legal_errors['misinterpretation']:
            doc += f"   {chr(reason_index)}. 법률 해석 및 적용의 착오\n\n"
            for idx, error in enumerate(legal_errors['misinterpretation'], 1):
                doc += f"      {idx}) {error}\n"
            doc += "\n"
            reason_index += 1

        # 위법성 조각사유
        if 'justification' in legal_errors and legal_errors['justification']:
            doc += f"   {chr(reason_index)}. 위법성 조각사유의 존재\n\n"
            for idx, justification in enumerate(legal_errors['justification'], 1):
                doc += f"      {idx}) {justification}\n"
            doc += "\n"
            reason_index += 1

        # 판례 위반
        if 'precedent_violation' in legal_errors and legal_errors['precedent_violation']:
            doc += f"   {chr(reason_index)}. 대법원 판례 위반\n\n"
            for idx, precedent in enumerate(legal_errors['precedent_violation'], 1):
                doc += f"      {idx}) {precedent}\n"
            doc += "\n"
            reason_index += 1

        # 절차상 하자
        if 'procedural_errors' in legal_errors and legal_errors['procedural_errors']:
            doc += f"   {chr(reason_index)}. 소송절차상 하자\n\n"
            for idx, error in enumerate(legal_errors['procedural_errors'], 1):
                doc += f"      {idx}) {error}\n"
            doc += "\n"
            reason_index += 1

        # 결론
        doc += "2. 결론\n\n"
        doc += "   이상과 같이 원심은 법률의 해석·적용을 잘못하였으므로, 원심을 파기하고\n"
        doc += "   피고인에게 정당한 판결을 하여 주시기 바라면서 항소이유서를 제출합니다.\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        if additional_info and 'attorney' in additional_info:
            doc += f"위 피고인의 변호인\n"
            doc += f"변 호 사  {additional_info['attorney']}  (인)\n"
        else:
            doc += f"피고인  {defendant['name']}  (인)\n"

        doc += "\n"
        doc += f"{case['court']} 귀중\n"

        return doc

    def write_comprehensive_appeal(
        self,
        case: Dict,
        defendant: Dict,
        factual_errors: Optional[Dict] = None,
        legal_errors: Optional[Dict] = None,
        sentencing_grounds: Optional[Dict] = None,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        종합 항소이유서 작성 (사실오인, 법리오해, 양형부당 모두 포함)

        Args:
            case: 사건 정보
            defendant: 피고인 정보
            factual_errors: 사실오인 내용 (optional)
            legal_errors: 법리오해 내용 (optional)
            sentencing_grounds: 양형 이유 (optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 종합 항소이유서
        """
        doc = "항 소 이 유 서\n\n"

        # 사건 표시
        doc += f"사    건  {case['case_number']} {case['crime']}\n"
        doc += f"피 고 인  {defendant['name']}"
        if 'birth_date' in defendant:
            doc += f" ({defendant['birth_date']} 생)"
        doc += "\n"

        if 'residence' in defendant:
            doc += f"주    소  {defendant['residence']}\n"

        doc += "\n"

        # 서두
        doc += f"위 피고인에 대한 귀원 {case['case_number']}호 {case['crime']} "
        doc += "피고사건에 관하여 "
        if 'attorney' in (additional_info or {}):
            doc += "피고인의 변호인은 "
        doc += "다음과 같이 항소이유를 개진합니다.\n\n"

        doc += "다    음\n\n"

        main_index = 1

        # 사실오인
        if factual_errors:
            doc += f"{main_index}. 원심 판결의 사실오인 및 채증법칙 위배\n\n"
            doc += "   원심은 증거를 잘못 평가하여 사실을 오인하였습니다.\n\n"

            sub_index = ord('가')

            if 'evidence_credibility' in factual_errors and factual_errors['evidence_credibility']:
                doc += f"   {chr(sub_index)}. 원심이 채택한 증거의 신빙성 부족\n\n"
                for idx, issue in enumerate(factual_errors['evidence_credibility'], 1):
                    doc += f"      {idx}) {issue}\n"
                doc += "\n"
                sub_index += 1

            if 'favorable_evidence_ignored' in factual_errors and factual_errors['favorable_evidence_ignored']:
                doc += f"   {chr(sub_index)}. 피고인에게 유리한 증거를 간과한 잘못\n\n"
                for idx, evidence in enumerate(factual_errors['favorable_evidence_ignored'], 1):
                    doc += f"      {idx}) {evidence}\n"
                doc += "\n"
                sub_index += 1

            main_index += 1

        # 법리오해
        if legal_errors:
            doc += f"{main_index}. 원심의 법리오해\n\n"
            doc += "   원심은 관련 법률의 해석 및 적용을 잘못하였습니다.\n\n"

            sub_index = ord('가')

            if 'misinterpretation' in legal_errors and legal_errors['misinterpretation']:
                doc += f"   {chr(sub_index)}. 법률 해석 및 적용의 착오\n\n"
                for idx, error in enumerate(legal_errors['misinterpretation'], 1):
                    doc += f"      {idx}) {error}\n"
                doc += "\n"
                sub_index += 1

            if 'precedent_violation' in legal_errors and legal_errors['precedent_violation']:
                doc += f"   {chr(sub_index)}. 대법원 판례 위반\n\n"
                for idx, precedent in enumerate(legal_errors['precedent_violation'], 1):
                    doc += f"      {idx}) {precedent}\n"
                doc += "\n"
                sub_index += 1

            main_index += 1

        # 양형부당
        if sentencing_grounds:
            doc += f"{main_index}. 원심의 양형 부당\n\n"
            doc += "   원심이 선고한 형량은 다음과 같은 이유로 지나치게 무거워 부당합니다.\n\n"

            sub_index = ord('가')

            if sentencing_grounds.get('first_offense', False):
                doc += f"   {chr(sub_index)}. 피고인은 이 사건이 초범입니다.\n\n"
                sub_index += 1

            if 'remorse' in sentencing_grounds:
                doc += f"   {chr(sub_index)}. 피고인은 {sentencing_grounds['remorse']}.\n\n"
                sub_index += 1

            if sentencing_grounds.get('victim_settlement', False):
                doc += f"   {chr(sub_index)}. 피고인은 피해자와 합의하여 피해를 배상하였습니다.\n\n"
                sub_index += 1

            main_index += 1

        # 결론
        doc += f"{main_index}. 결론\n\n"
        doc += "   이상과 같은 이유로 원심을 파기하고 피고인에게 정당한 판결을 하여 주시기\n"
        doc += "   바라면서 항소이유서를 제출합니다.\n\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        if additional_info and 'attorney' in additional_info:
            doc += f"위 피고인의 변호인\n"
            doc += f"변 호 사  {additional_info['attorney']}  (인)\n"
        else:
            doc += f"피고인  {defendant['name']}  (인)\n"

        doc += "\n"
        doc += f"{case['court']} 귀중\n"

        return doc


# 사용 예시
if __name__ == "__main__":
    writer = AppealReasonWriter()

    # 양형부당 항소이유서 예시
    print("=" * 80)
    print("양형부당 항소이유서 예시")
    print("=" * 80)

    sentencing_appeal = writer.write_sentencing_appeal(
        case={
            "court": "서울중앙지방법원",
            "case_number": "2024고단1234",
            "crime": "특정범죄가중처벌등에관한법률위반(절도)",
            "original_sentence": "징역 1년",
            "sentence_date": "2024. 5. 15."
        },
        defendant={
            "name": "홍길동",
            "birth_date": "1990. 1. 15.",
            "residence": "서울 서초구 서초대로 1234"
        },
        appeal_grounds={
            "first_offense": True,
            "remorse": "범행 후 깊이 반성하고 있",
            "victim_settlement": True,
            "settlement_amount": "500만원",
            "family_situation": "80세 노모와 어린 자녀 2명을 부양해야 하는 가장",
            "social_contribution": "5년간 지역 봉사활동에 참여하는",
            "pretrial_detention_days": 90
        }
    )

    print(sentencing_appeal)
    print("\n")

    # 사실오인 항소이유서 예시
    print("=" * 80)
    print("사실오인 항소이유서 예시")
    print("=" * 80)

    factual_appeal = writer.write_factual_error_appeal(
        case={
            "court": "서울북부지방법원",
            "case_number": "2024고단5678",
            "crime": "폭행",
            "original_sentence": "벌금 300만원"
        },
        defendant={
            "name": "이철수",
            "birth_date": "1985. 3. 20."
        },
        factual_errors={
            "evidence_credibility": [
                "피해자 진술의 모순: 사건 발생 시각과 장소에 대한 진술이 수차례 변경됨",
                "목격자 부재: 현장에 제3자 목격자가 전혀 없음"
            ],
            "favorable_evidence_ignored": [
                "피고인의 알리바이: CCTV 영상으로 사건 발생 시각에 다른 장소에 있었음이 입증됨",
                "진단서 미제출: 피해자가 상해 진단서를 제출하지 못함"
            ],
            "logical_contradictions": [
                "피해자 진술과 물증의 불일치",
                "사건 전후 피해자의 행동 패턴이 진술 내용과 모순됨"
            ]
        }
    )

    print(factual_appeal)
    print("\n")

    # 법리오해 항소이유서 예시
    print("=" * 80)
    print("법리오해 항소이유서 예시")
    print("=" * 80)

    legal_appeal = writer.write_legal_error_appeal(
        case={
            "court": "인천지방법원",
            "case_number": "2024고단9012",
            "crime": "업무상횡령",
            "original_sentence": "징역 2년"
        },
        defendant={
            "name": "김영희",
            "birth_date": "1982. 7. 10."
        },
        legal_errors={
            "misinterpretation": [
                "횡령의 고의 부존재: 회사 자금을 일시 차용한 것으로 횡령 고의가 없음",
                "불법영득의사 부존재: 반환 의사와 능력이 있었음"
            ],
            "justification": [
                "경영상 긴급한 필요에 의한 자금 집행",
                "이사회의 사후 승인 가능성"
            ],
            "precedent_violation": [
                "대법원 2020도12345 판결: 일시 차용과 횡령의 구별 기준",
                "대법원 2019도67890 판결: 불법영득의사 판단 기준"
            ]
        },
        additional_info={
            "attorney": "법무법인 정의 변호사 박정의"
        }
    )

    print(legal_appeal)
