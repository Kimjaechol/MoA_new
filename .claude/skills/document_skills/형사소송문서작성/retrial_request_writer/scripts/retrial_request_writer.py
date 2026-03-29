"""
형사 재심청구서 작성 스킬
Criminal Retrial Request Writer Skill

이 모듈은 형사사건에서 확정된 유죄판결에 대하여 재심을 청구하는
재심청구서를 자동으로 작성합니다.
"""

from typing import Dict, List, Optional
from datetime import datetime


class RetrialRequestWriter:
    """형사 재심청구서 작성 클래스"""

    def __init__(self):
        self.version = "1.0.0"

    def write_law_amendment_retrial(
        self,
        requester: Dict,
        original_judgment: Dict,
        retrial_grounds: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        법령 개폐에 따른 재심청구서 작성 (형사소송법 제420조 제10호)

        Args:
            requester: 재심청구인 정보
                - name: 성명
                - residence: 주거
                - registration: 본적 (등록기준지)
                - relation: 관계 (피고인 본인, 법정대리인 등, optional)
            original_judgment: 원판결 정보
                - court: 원판결 법원
                - case_number: 원판결 사건번호
                - crime: 죄명
                - sentence: 형량
                - judgment_date: 판결일자
                - finalized_date: 확정일자 (optional)
                - crime_facts: 범죄사실 요지 (optional)
            retrial_grounds: 재심 사유
                - law_before: 개정 전 법령
                - law_after: 개정 후 법령
                - amendment_date: 법령 개정일
                - crime_description: 범죄 행위 설명
                - no_longer_crime: 범죄 불성립 이유
            additional_info: 추가 정보 (optional)
                - attorney: 변호인 (optional)

        Returns:
            작성된 재심청구서 (문자열)
        """
        doc = "재 심 청 구 서\n\n"

        # 재심청구인 정보
        doc += f"재심청구인  {requester['name']}\n"
        if 'residence' in requester:
            doc += f"주    거  {requester['residence']}\n"
        if 'registration' in requester:
            doc += f"본    적  {requester['registration']}\n"

        doc += "\n"

        # 원판결 표시
        doc += "원판결의 표시  "
        doc += f"{original_judgment['court']}\n"
        doc += f"             {original_judgment['case_number']} {original_judgment['crime']}\n"

        doc += "\n"

        # 서두
        if 'crime_facts' in original_judgment:
            doc += f"위 판결사건의 재심청구인은 {retrial_grounds['crime_description']}하였다는\n"
            doc += "이유로 귀원에 기소되어 "
        else:
            doc += "위 판결사건의 재심청구인은 귀원에 "

        if 'judgment_date' in original_judgment:
            doc += f"공판 중 {original_judgment['judgment_date']} 판결을 선고받고 "
        else:
            doc += "판결을 선고받고 "

        doc += "그 판결이 확정되었으나\n"

        if 'finalized_date' in original_judgment:
            doc += f"그 확정 전인 "
        else:
            doc += "확정 전인 "

        doc += f"{retrial_grounds['amendment_date']} 법률의 개정으로 인하여 범죄되지 않도록\n"
        doc += "되어 있으므로 아래와 같이 재심사유가 있어 재심청구를 하오니 재심개시 결정을\n"
        doc += "하여 주시기 바랍니다.\n\n"

        # 재심 사유
        doc += "아    래\n\n"

        doc += f"1. 원 판결 {original_judgment['case_number']} {original_judgment['crime']} 사건은\n"
        doc += "   형사소송법 제420조 제10호에 의하여 인정한 법령의 개폐로 인하여 죄가 되지\n"
        doc += "   아니한 때에 해당하므로 당연히 원판결을 취소하고 면소의 판결을 하여야 할\n"
        doc += "   것입니다.\n\n"

        # 법령 개정 상세 설명 (optional)
        if 'law_before' in retrial_grounds or 'law_after' in retrial_grounds:
            doc += "2. 법령 개정 내용\n\n"
            if 'law_before' in retrial_grounds:
                doc += f"   개정 전: {retrial_grounds['law_before']}\n"
            if 'law_after' in retrial_grounds:
                doc += f"   개정 후: {retrial_grounds['law_after']}\n"
            if 'no_longer_crime' in retrial_grounds:
                doc += f"\n   {retrial_grounds['no_longer_crime']}\n"
            doc += "\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 판결문  1통\n"

        if additional_info and 'attachments' in additional_info:
            for idx, attachment in enumerate(additional_info['attachments'], 2):
                doc += f"{idx}. {attachment}\n"

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재심청구인  {requester['name']}  (인)\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변 호 인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{original_judgment['court']} 귀중\n"

        return doc

    def write_new_evidence_retrial(
        self,
        requester: Dict,
        original_judgment: Dict,
        new_evidence: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        새로운 증거 발견에 따른 재심청구서 작성 (형사소송법 제420조 제5호)

        Args:
            requester: 재심청구인 정보
            original_judgment: 원판결 정보
                - court: 원판결 법원
                - case_number: 사건번호
                - crime: 죄명
                - sentence: 형량
                - judgment_date: 판결일자
                - crime_facts: 범죄사실 (optional)
            new_evidence: 새로운 증거
                - type: 증거 유형 (CCTV, DNA, 진술서 등)
                - description: 증거 설명
                - proves: 무엇을 입증하는지
                - why_not_discovered: 원판결 당시 미발견 이유
                - attached_documents: 첨부 서류 (list, optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 재심청구서
        """
        doc = "재 심 청 구 서\n\n"

        # 재심청구인 정보
        doc += f"재심청구인  {requester['name']}\n"
        if 'residence' in requester:
            doc += f"주    거  {requester['residence']}\n"
        if 'registration' in requester:
            doc += f"본    적  {requester['registration']}\n"

        doc += "\n"

        # 원판결 표시
        doc += "원판결의 표시  "
        doc += f"{original_judgment['court']}\n"
        doc += f"             {original_judgment['case_number']} {original_judgment['crime']}\n"

        doc += "\n"

        # 서두
        doc += "위 판결사건의 재심청구인은 다음과 같은 이유로 재심을 청구합니다.\n\n"

        # 1. 원 판결의 내용
        doc += "1. 원 판결의 내용\n\n"
        doc += "   재심청구인은 "

        if 'crime_facts' in original_judgment:
            doc += f"{original_judgment['crime_facts']}하였다는\n"
            doc += "   혐의로 기소되어 "
        else:
            doc += f"{original_judgment['crime']} 혐의로 기소되어 "

        if 'judgment_date' in original_judgment:
            doc += f"{original_judgment['judgment_date']} "

        doc += f"{original_judgment['sentence']}을 선고받고 그 판결이 확정되었습니다.\n\n"

        # 2. 새로운 증거의 발견
        doc += "2. 새로운 증거의 발견\n\n"
        doc += f"   원 판결 이후 {new_evidence['description']}이(가) 발견되었습니다.\n"
        doc += f"   이 {new_evidence['type']}은(는) {new_evidence['proves']}을(를) 명백히\n"
        doc += "   보여주는 증거입니다.\n\n"

        # 3. 원판결 당시 미발견 이유
        doc += "3. 원판결 당시 미발견 이유\n\n"
        doc += f"   {new_evidence['why_not_discovered']}\n\n"

        # 4. 재심 사유
        doc += "4. 재심 사유\n\n"
        doc += f"   위 {new_evidence['type']}은(는) 재심청구인이 "
        if "무죄" in new_evidence.get('proves', '') or "알리바이" in new_evidence.get('proves', ''):
            doc += "무죄임을"
        else:
            doc += f"{new_evidence['proves']}을(를)"
        doc += " 명백히 입증하는\n"
        doc += "   증거로서, 형사소송법 제420조 제5호에 해당하는 재심 사유입니다.\n\n"
        doc += "   따라서 재심개시 결정을 하여 주시고, 재심에서 무죄 판결을 선고하여\n"
        doc += "   주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 원 판결문  1통\n"

        attachment_idx = 2
        if 'attached_documents' in new_evidence:
            for attachment in new_evidence['attached_documents']:
                doc += f"{attachment_idx}. {attachment}\n"
                attachment_idx += 1

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재심청구인  {requester['name']}  (인)\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변 호 인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{original_judgment['court']} 귀중\n"

        return doc

    def write_false_evidence_retrial(
        self,
        requester: Dict,
        original_judgment: Dict,
        false_evidence: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        허위 증거에 따른 재심청구서 작성 (형사소송법 제420조 제1호, 제2호)

        Args:
            requester: 재심청구인 정보
            original_judgment: 원판결 정보
            false_evidence: 허위/위조 증거 정보
                - type: 증거 유형 (서류, 증거물, 증언, 감정 등)
                - description: 증거 설명
                - false_confirmation: 허위/위조 확정판결 정보
                - impact_on_judgment: 원판결에 미친 영향
                - article: 해당 조문 (제1호 또는 제2호)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 재심청구서
        """
        doc = "재 심 청 구 서\n\n"

        # 재심청구인 정보
        doc += f"재심청구인  {requester['name']}\n"
        if 'residence' in requester:
            doc += f"주    거  {requester['residence']}\n"
        if 'registration' in requester:
            doc += f"본    적  {requester['registration']}\n"

        doc += "\n"

        # 원판결 표시
        doc += "원판결의 표시  "
        doc += f"{original_judgment['court']}\n"
        doc += f"             {original_judgment['case_number']} {original_judgment['crime']}\n"

        doc += "\n"

        # 서두
        doc += "위 판결사건의 재심청구인은 다음과 같은 이유로 재심을 청구합니다.\n\n"

        # 1. 원 판결의 내용
        doc += "1. 원 판결의 내용\n\n"
        doc += f"   재심청구인은 {original_judgment['judgment_date']} "
        doc += f"{original_judgment['sentence']}을 선고받고 그 판결이 확정되었습니다.\n\n"

        # 2. 원판결의 증거
        doc += "2. 원판결의 증거\n\n"
        doc += f"   원 판결은 {false_evidence['description']}을(를) 주요 증거로 채택하여\n"
        doc += "   유죄를 인정하였습니다.\n\n"

        # 3. 허위/위조 사실의 확정
        doc += "3. 허위/위조 사실의 확정\n\n"
        doc += f"   그러나 위 {false_evidence['type']}은(는) "
        doc += f"{false_evidence['false_confirmation']}에 의하여\n"

        if "증언" in false_evidence['type'] or "감정" in false_evidence['type']:
            doc += "   허위임이 확정판결로 증명되었습니다.\n\n"
        else:
            doc += "   위조 또는 변조임이 확정판결로 증명되었습니다.\n\n"

        # 4. 원판결에 미친 영향
        doc += "4. 원판결에 미친 영향\n\n"
        doc += f"   {false_evidence['impact_on_judgment']}\n\n"

        # 5. 재심 사유
        doc += "5. 재심 사유\n\n"
        doc += f"   이는 형사소송법 제420조 제{false_evidence['article']}호에 해당하는 재심 사유로서,\n"
        doc += "   원판결을 취소하고 무죄 판결을 선고하여야 합니다.\n\n"
        doc += "   따라서 재심개시 결정을 하여 주시고, 재심에서 무죄 판결을 선고하여\n"
        doc += "   주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 원 판결문  1통\n"
        doc += f"2. {false_evidence['type']} 허위/위조 확정판결문  1통\n"

        if additional_info and 'attachments' in additional_info:
            for idx, attachment in enumerate(additional_info['attachments'], 3):
                doc += f"{idx}. {attachment}\n"

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재심청구인  {requester['name']}  (인)\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변 호 인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{original_judgment['court']} 귀중\n"

        return doc

    def write_official_misconduct_retrial(
        self,
        requester: Dict,
        original_judgment: Dict,
        misconduct: Dict,
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        공무원 직무범죄에 따른 재심청구서 작성 (형사소송법 제420조 제4호)

        Args:
            requester: 재심청구인 정보
            original_judgment: 원판결 정보
            misconduct: 직무범죄 정보
                - official_type: 공무원 유형 (법관, 검사, 사법경찰관)
                - official_name: 공무원 성명
                - crime: 직무범죄 내용
                - conviction: 확정판결 정보
                - impact: 원판결에 미친 영향
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 재심청구서
        """
        doc = "재 심 청 구 서\n\n"

        # 재심청구인 정보
        doc += f"재심청구인  {requester['name']}\n"
        if 'residence' in requester:
            doc += f"주    거  {requester['residence']}\n"
        if 'registration' in requester:
            doc += f"본    적  {requester['registration']}\n"

        doc += "\n"

        # 원판결 표시
        doc += "원판결의 표시  "
        doc += f"{original_judgment['court']}\n"
        doc += f"             {original_judgment['case_number']} {original_judgment['crime']}\n"

        doc += "\n"

        # 서두
        doc += "위 판결사건의 재심청구인은 다음과 같은 이유로 재심을 청구합니다.\n\n"

        # 1. 원 판결의 내용
        doc += "1. 원 판결의 내용\n\n"
        doc += f"   재심청구인은 {original_judgment['judgment_date']} "
        doc += f"{original_judgment['sentence']}을 선고받고 그 판결이 확정되었습니다.\n\n"

        # 2. 공무원의 직무범죄
        doc += "2. 공무원의 직무범죄\n\n"
        doc += f"   원 판결에 {misconduct['official_type']}으로 관여한 {misconduct['official_name']}은(는)\n"
        doc += f"   {misconduct['crime']}하였고, 이는 {misconduct['conviction']}에 의하여\n"
        doc += "   확정판결로 증명되었습니다.\n\n"

        # 3. 원판결에 미친 영향
        doc += "3. 원판결에 미친 영향\n\n"
        doc += f"   {misconduct['impact']}\n\n"

        # 4. 재심 사유
        doc += "4. 재심 사유\n\n"
        doc += "   이는 형사소송법 제420조 제4호에 해당하는 재심 사유로서, 원판결을 취소하고\n"
        doc += "   무죄 판결을 선고하여야 합니다.\n\n"
        doc += "   따라서 재심개시 결정을 하여 주시고, 재심에서 무죄 판결을 선고하여\n"
        doc += "   주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 원 판결문  1통\n"
        doc += f"2. {misconduct['official_name']} 직무범죄 확정판결문  1통\n"

        if additional_info and 'attachments' in additional_info:
            for idx, attachment in enumerate(additional_info['attachments'], 3):
                doc += f"{idx}. {attachment}\n"

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재심청구인  {requester['name']}  (인)\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변 호 인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{original_judgment['court']} 귀중\n"

        return doc

    def write_comprehensive_retrial(
        self,
        requester: Dict,
        original_judgment: Dict,
        retrial_grounds: List[Dict],
        additional_info: Optional[Dict] = None
    ) -> str:
        """
        종합 재심청구서 작성 (여러 재심 사유 포함 가능)

        Args:
            requester: 재심청구인 정보
            original_judgment: 원판결 정보
            retrial_grounds: 재심 사유 목록 (list of dict)
                각 dict는 다음을 포함:
                - article: 해당 조문 (제1호, 제2호 등)
                - reason: 재심 사유 설명
                - evidence: 증거 (optional)
            additional_info: 추가 정보 (optional)

        Returns:
            작성된 종합 재심청구서
        """
        doc = "재 심 청 구 서\n\n"

        # 재심청구인 정보
        doc += f"재심청구인  {requester['name']}\n"
        if 'residence' in requester:
            doc += f"주    거  {requester['residence']}\n"
        if 'registration' in requester:
            doc += f"본    적  {requester['registration']}\n"

        doc += "\n"

        # 원판결 표시
        doc += "원판결의 표시  "
        doc += f"{original_judgment['court']}\n"
        doc += f"             {original_judgment['case_number']} {original_judgment['crime']}\n"

        doc += "\n"

        # 서두
        doc += "위 판결사건의 재심청구인은 다음과 같은 이유로 재심을 청구합니다.\n\n"

        # 1. 원 판결의 내용
        doc += "1. 원 판결의 내용\n\n"
        doc += f"   재심청구인은 {original_judgment['judgment_date']} "
        doc += f"{original_judgment['sentence']}을 선고받고 그 판결이 확정되었습니다.\n\n"

        # 2. 재심 사유
        doc += "2. 재심 사유\n\n"

        for idx, ground in enumerate(retrial_grounds):
            sub_idx = chr(ord('가') + idx)
            doc += f"   {sub_idx}. 형사소송법 제420조 제{ground['article']}호\n\n"
            doc += f"      {ground['reason']}\n\n"

        # 3. 결론
        doc += "3. 결론\n\n"
        doc += "   이상과 같은 재심 사유가 존재하므로, 재심개시 결정을 하여 주시고,\n"
        doc += "   재심에서 무죄 판결을 선고하여 주시기 바랍니다.\n\n"

        # 첨부 서류
        doc += "첨부 서류\n\n"
        doc += "1. 원 판결문  1통\n"

        attachment_idx = 2
        if additional_info and 'attachments' in additional_info:
            for attachment in additional_info['attachments']:
                doc += f"{attachment_idx}. {attachment}\n"
                attachment_idx += 1

        doc += "\n"

        # 날짜 및 서명
        doc += f"{datetime.now().year}.     .     .\n\n"
        doc += f"재심청구인  {requester['name']}  (인)\n"

        if additional_info and 'attorney' in additional_info:
            doc += f"변 호 인  {additional_info['attorney']}  (인)\n"

        doc += "\n"
        doc += f"{original_judgment['court']} 귀중\n"

        return doc


# 사용 예시
if __name__ == "__main__":
    writer = RetrialRequestWriter()

    # 법령 개폐에 따른 재심청구서 예시
    print("=" * 80)
    print("법령 개폐에 따른 재심청구서 예시")
    print("=" * 80)

    law_amendment_retrial = writer.write_law_amendment_retrial(
        requester={
            "name": "이유미",
            "residence": "서울 강남구 청담동 1828-11",
            "registration": "서울 강남구 청담동 1291"
        },
        original_judgment={
            "court": "서울북부지방법원",
            "case_number": "2020고단2419",
            "crime": "건축법위반",
            "sentence": "벌금 200만원",
            "judgment_date": "2020. 8. 19.",
            "crime_facts": "근린생활시설을 노래연습장으로 용도변경하면서 허가를 받지 않음"
        },
        retrial_grounds={
            "law_before": "건축법 제14조 (용도변경 허가 필요)",
            "law_after": "건축법 제14조 개정 (노래연습장 용도변경 허가 제외)",
            "amendment_date": "2020. 8. 9.",
            "crime_description": "근린생활시설을 노래연습장으로 용도변경하면서 허가를 받지 않",
            "no_longer_crime": "개정된 법에 따르면 노래연습장 용도변경은 허가 대상이 아니므로 범죄가 성립하지 않습니다."
        }
    )

    print(law_amendment_retrial)
    print("\n")

    # 새로운 증거에 따른 재심청구서 예시
    print("=" * 80)
    print("새로운 증거 발견에 따른 재심청구서 예시")
    print("=" * 80)

    new_evidence_retrial = writer.write_new_evidence_retrial(
        requester={
            "name": "김무죄",
            "residence": "서울 서초구 서초대로 123",
            "registration": "서울 서초구 서초동 456"
        },
        original_judgment={
            "court": "서울중앙지방법원",
            "case_number": "2020고단1234",
            "crime": "절도",
            "sentence": "징역 1년",
            "judgment_date": "2020. 8. 20.",
            "crime_facts": "2020. 3. 15. 14:00경 서울 강남구 소재 편의점에서 물품을 절취"
        },
        new_evidence={
            "type": "CCTV 영상",
            "description": "범행 시각인 2020. 3. 15. 14:00경 재심청구인이 서울 송파구 소재\n   은행에서 업무를 처리하고 있었음을 입증하는 CCTV 영상",
            "proves": "알리바이 (범행 불가능)",
            "why_not_discovered": "원판결 당시 재심청구인과 변호인은 해당 은행에 CCTV가 있는지 알지\n   못하였고, 은행 측에서도 자발적으로 제공하지 않았습니다. 최근 재심청구인의\n   가족이 은행을 방문하여 CCTV 영상을 확인한 결과 해당 영상이 보관되어 있음을\n   발견하였습니다.",
            "attached_documents": [
                "은행 CCTV 영상 (USB)  1개",
                "은행 영상 제공 확인서  1통"
            ]
        }
    )

    print(new_evidence_retrial)
    print("\n")

    # 허위 증거에 따른 재심청구서 예시
    print("=" * 80)
    print("허위 증거에 따른 재심청구서 예시")
    print("=" * 80)

    false_evidence_retrial = writer.write_false_evidence_retrial(
        requester={
            "name": "박억울",
            "residence": "서울 용산구 이태원로 789",
            "registration": "서울 용산구 용산동 123"
        },
        original_judgment={
            "court": "서울남부지방법원",
            "case_number": "2019고단5678",
            "crime": "폭행",
            "sentence": "징역 6월",
            "judgment_date": "2019. 12. 15."
        },
        false_evidence={
            "type": "증인 이허위의 증언",
            "description": "증인 이허위의 '피고인이 피해자를 주먹으로 폭행하는 것을 목격했다'는 증언",
            "false_confirmation": "서울남부지방법원 2023고단9012 위증 사건 판결 (2024. 3. 10. 확정)",
            "impact_on_judgment": "위 증언은 원판결의 핵심 증거였으며, 만약 이 증언이 없었다면\n   재심청구인에게 유죄를 인정할 증거가 없어 무죄가 선고되었을 것입니다.",
            "article": "2"
        }
    )

    print(false_evidence_retrial)
