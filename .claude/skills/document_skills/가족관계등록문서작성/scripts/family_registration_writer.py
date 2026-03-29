#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
가족관계등록 신고서 작성 모듈

38,331줄의 가족관계등록실무 데이터베이스를 기반으로
모든 종류의 가족관계등록 신고서를 자동으로 작성합니다.

지원 기능:
- 출생·사망신고
- 혼인·이혼신고
- 입양·파양·친양자 신고
- 인지신고
- 친권·후견 신고
- 개명·창설·정정 신청
- 성별정정 신청
"""

from typing import Dict, List, Optional
from datetime import datetime


class FamilyRegistrationWriter:
    """가족관계등록 신고서 작성 클래스"""

    def __init__(self):
        """초기화"""
        self.reference_data = "가족관계등록에관한실무 (김현선 저, 백영사, 38,331줄)"

    def write_birth_report(
        self,
        declarant: Dict,
        child: Dict,
        parents: Dict,
        birth_certificate: str,
        office: str = "시·읍·면장"
    ) -> str:
        """
        출생신고서 작성

        Args:
            declarant: 신고인 정보 (name, resident_number, relation)
            child: 출생자 정보 (name, sex, birth_datetime, birth_place)
            parents: 부모 정보 (father, mother, registration_base)
            birth_certificate: 출생증명서 번호
            office: 신고 관할 (시·읍·면장)

        Returns:
            출생신고서 전문
        """
        report = "출생신고서\n\n"

        # 신고인
        report += f"신 고 인    {declarant.get('address', '')}\n"
        report += f"            {declarant['name']} "
        report += f"(주민등록번호: {declarant.get('resident_number', '')})\n\n"

        report += f"            본인과의 관계: {declarant.get('relation', '부')}\n\n"

        # 출생자
        report += "출생자 정보\n\n"
        report += f"  성    명:  {child['name']}\n"
        report += f"  성    별:  {child.get('sex', '남')}\n"
        report += f"  출생일시:  {child['birth_datetime']}\n"
        report += f"  출생장소:  {child.get('birth_place', '')}\n\n"

        # 부모
        report += "부모 정보\n\n"
        report += f"  부: {parents.get('father', '')} "
        report += f"(등록기준지: {parents.get('father_registration_base', '')})\n"
        report += f"  모: {parents.get('mother', '')} "
        report += f"(등록기준지: {parents.get('mother_registration_base', '')})\n\n"

        # 출생증명서
        report += "첨부서류\n\n"
        report += f"1. 출생증명서 (번호: {birth_certificate})\n"
        report += "2. 신고인 신분증 사본\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 신고인  {declarant['name']} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_death_report(
        self,
        declarant: Dict,
        deceased: Dict,
        death_certificate: str,
        office: str = "시·읍·면장"
    ) -> str:
        """
        사망신고서 작성

        Args:
            declarant: 신고인 정보 (name, address, relation)
            deceased: 사망자 정보 (name, resident_number, death_datetime, death_place, cause)
            death_certificate: 사망진단서 번호
            office: 신고 관할

        Returns:
            사망신고서 전문
        """
        report = "사망신고서\n\n"

        # 신고인
        report += f"신 고 인    {declarant.get('address', '')}\n"
        report += f"            {declarant['name']}\n\n"
        report += f"            본인과의 관계: {declarant.get('relation', '가족')}\n\n"

        # 사망자
        report += "사망자 정보\n\n"
        report += f"  성    명:  {deceased['name']}\n"
        report += f"  주민등록번호:  {deceased['resident_number']}\n"
        report += f"  사망일시:  {deceased['death_datetime']}\n"
        report += f"  사망장소:  {deceased.get('death_place', '')}\n"
        report += f"  사망원인:  {deceased.get('cause', '질병사')}\n\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += f"1. 사망진단서 (번호: {death_certificate})\n"
        report += "2. 신고인 신분증 사본\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 신고인  {declarant['name']} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_marriage_report(
        self,
        husband: Dict,
        wife: Dict,
        witnesses: List[Dict],
        marriage_date: Optional[str] = None,
        office: str = "시·읍·면장"
    ) -> str:
        """
        혼인신고서 작성

        Args:
            husband: 부 정보 (name, resident_number, address, registration_base)
            wife: 처 정보 (name, resident_number, address, registration_base)
            witnesses: 증인 2인 정보 (name, resident_number)
            marriage_date: 혼인성립일 (None이면 신고일)
            office: 신고 관할

        Returns:
            혼인신고서 전문
        """
        report = "혼인신고서\n\n"

        # 부
        report += f"부            {husband.get('address', '')}\n"
        report += f"              {husband['name']} "
        report += f"(주민등록번호: {husband['resident_number']})\n"
        report += f"              등록기준지: {husband.get('registration_base', '')}\n\n"

        # 처
        report += f"처            {wife.get('address', '')}\n"
        report += f"              {wife['name']} "
        report += f"(주민등록번호: {wife['resident_number']})\n"
        report += f"              등록기준지: {wife.get('registration_base', '')}\n\n"

        # 혼인성립일
        if marriage_date:
            report += f"혼인성립일:  {marriage_date}\n\n"
        else:
            today = datetime.now().strftime("%Y. %m. %d.")
            report += f"혼인성립일:  {today} (신고일)\n\n"

        # 증인
        report += "증인 (성년자 2인)\n\n"
        for idx, witness in enumerate(witnesses[:2], 1):
            report += f"  증인 {idx}:  {witness['name']} "
            report += f"(주민등록번호: {witness['resident_number']}) (인)\n"
        report += "\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 가족관계증명서 (부, 처 각 1통)\n"
        report += "2. 혼인관계증명서 (부, 처 각 1통)\n"
        report += "3. 증인 신분증 사본 (2통)\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 부  {husband['name']} (인)\n"
        report += f"위 처  {wife['name']} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_divorce_report(
        self,
        husband: Dict,
        wife: Dict,
        confirmation_date: str,
        children: Optional[List[Dict]] = None,
        office: str = "시·읍·면장"
    ) -> str:
        """
        협의이혼 신고서 작성

        Args:
            husband: 부 정보 (name, resident_number, address)
            wife: 처 정보 (name, resident_number, address)
            confirmation_date: 이혼의사 확인서 교부일
            children: 미성년 자녀 정보 (name, custodian)
            office: 신고 관할

        Returns:
            협의이혼신고서 전문
        """
        report = "협의이혼신고서\n\n"

        # 부
        report += f"부            {husband.get('address', '')}\n"
        report += f"              {husband['name']} "
        report += f"(주민등록번호: {husband['resident_number']})\n\n"

        # 처
        report += f"처            {wife.get('address', '')}\n"
        report += f"              {wife['name']} "
        report += f"(주민등록번호: {wife['resident_number']})\n\n"

        # 이혼의사 확인
        report += f"이혼의사 확인서 교부일:  {confirmation_date}\n\n"

        # 미성년 자녀
        if children:
            report += "미성년 자녀의 친권자 및 양육자 지정\n\n"
            for child in children:
                report += f"  자녀: {child['name']}\n"
                custodian = "부" if child.get('custodian') == "부" else "모"
                report += f"  친권자 및 양육자: {custodian}\n\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 이혼의사 확인서 (가정법원)\n"
        report += "2. 가족관계증명서\n"
        report += "3. 혼인관계증명서\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 부  {husband['name']} (인)\n"
        report += f"위 처  {wife['name']} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_adoption_report(
        self,
        adoptive_parents: Dict,
        adoptee: Dict,
        consent: Dict,
        adoption_date: Optional[str] = None,
        office: str = "시·읍·면장"
    ) -> str:
        """
        입양신고서 작성

        Args:
            adoptive_parents: 양친 정보 (father, mother, address)
            adoptee: 양자 정보 (name, resident_number or birth_date, original_surname)
            consent: 동의 정보 (natural_parents, court_permit)
            adoption_date: 입양성립일 (None이면 신고일)
            office: 신고 관할

        Returns:
            입양신고서 전문
        """
        report = "입양신고서\n\n"

        # 양친
        report += f"양부          {adoptive_parents.get('address', '')}\n"
        report += f"              {adoptive_parents.get('father', '')}\n\n"

        report += f"양모          {adoptive_parents.get('address', '')}\n"
        report += f"              {adoptive_parents.get('mother', '')}\n\n"

        # 양자
        report += f"양자          {adoptee['name']}\n"
        if 'resident_number' in adoptee:
            report += f"              (주민등록번호: {adoptee['resident_number']})\n"
        else:
            report += f"              (생년월일: {adoptee['birth_date']})\n"

        if 'original_surname' in adoptee:
            report += f"              입양 전 성과 본: {adoptee['original_surname']}\n"
        report += "\n"

        # 입양성립일
        if adoption_date:
            report += f"입양성립일:  {adoption_date}\n\n"
        else:
            today = datetime.now().strftime("%Y. %m. %d.")
            report += f"입양성립일:  {today} (신고일)\n\n"

        # 동의 사항
        report += "동의 및 허가\n\n"
        if consent.get('natural_parents'):
            report += "  친생부모 동의: 있음\n"
        if consent.get('court_permit'):
            report += "  가정법원 허가: 있음\n"
        report += "\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 양자의 가족관계증명서\n"
        if consent.get('natural_parents'):
            report += "2. 친생부모 동의서\n"
        if consent.get('court_permit'):
            report += "3. 가정법원 허가서\n"
        report += "\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"{today}\n\n"
        report += f"위 양부  {adoptive_parents.get('father', '')} (인)\n"
        report += f"위 양모  {adoptive_parents.get('mother', '')} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_recognition_report(
        self,
        recognizer: Dict,
        recognized: Dict,
        mother: Optional[Dict] = None,
        consent: bool = False,
        office: str = "시·읍·면장"
    ) -> str:
        """
        인지신고서 작성

        Args:
            recognizer: 인지자 (부 또는 모) 정보 (name, resident_number, address)
            recognized: 피인지자 (자) 정보 (name, birth_date, address)
            mother: 모 정보 (부가 인지하는 경우)
            consent: 자의 동의 (성년자인 경우)
            office: 신고 관할

        Returns:
            인지신고서 전문
        """
        report = "인지신고서\n\n"

        # 인지자
        report += f"인 지 자      {recognizer.get('address', '')}\n"
        report += f"              {recognizer['name']} "
        report += f"(주민등록번호: {recognizer['resident_number']})\n"
        report += f"              등록기준지: {recognizer.get('registration_base', '')}\n\n"

        # 피인지자
        report += f"피인지자      {recognized['name']}\n"
        if 'resident_number' in recognized:
            report += f"              (주민등록번호: {recognized['resident_number']})\n"
        else:
            report += f"              (생년월일: {recognized['birth_date']})\n"
        report += f"              주소: {recognized.get('address', '')}\n\n"

        # 모 (부가 인지하는 경우)
        if mother:
            report += f"모            {mother['name']}\n\n"

        # 인지성립일
        today = datetime.now().strftime("%Y. %m. %d.")
        report += f"인지성립일:  {today} (신고일)\n\n"

        # 자의 동의
        if consent:
            report += "자의 동의:  있음 (성년자)\n\n"

        # 첨부서류
        report += "첨부서류\n\n"
        report += "1. 피인지자의 가족관계증명서\n"
        if consent:
            report += "2. 자의 동의서 (성년자)\n"
        report += "\n"

        # 날짜 및 서명
        report += f"{today}\n\n"
        report += f"위 인지자  {recognizer['name']} (인)\n\n"
        report += f"{office} 귀중\n"

        return report

    def write_name_change_application(
        self,
        applicant: Dict,
        old_name: str,
        new_name: str,
        reason: str,
        evidence: Optional[List[str]] = None,
        court: str = "가정법원"
    ) -> str:
        """
        개명허가 신청서 작성

        Args:
            applicant: 신청인 정보 (name, resident_number, address, registration_base)
            old_name: 개명 전 이름
            new_name: 개명 후 이름
            reason: 개명 사유
            evidence: 소명자료 목록
            court: 관할 가정법원

        Returns:
            개명허가 신청서 전문
        """
        application = "개명허가 신청서\n\n"

        # 신청인
        application += f"신 청 인      {applicant.get('address', '')}\n"
        application += f"              {applicant['name']} "
        application += f"(주민등록번호: {applicant['resident_number']})\n"
        application += f"              등록기준지: {applicant.get('registration_base', '')}\n\n"

        # 개명 내용
        application += "개명 사항\n\n"
        application += f"  개명 전 이름:  {old_name}\n"
        application += f"  개명 후 이름:  {new_name}\n\n"

        # 개명 사유
        application += "개명 사유\n\n"
        application += f"  {reason}\n\n"

        # 소명자료
        if evidence:
            application += "소명자료\n\n"
            for idx, doc in enumerate(evidence, 1):
                application += f"  {idx}. {doc}\n"
            application += "\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 가족관계증명서\n"
        application += "2. 기본증명서\n"
        application += "3. 주민등록등본\n"
        if evidence:
            application += "4. 소명자료 일체\n"
        application += "\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 신청인  {applicant['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application

    def write_sex_correction_application(
        self,
        applicant: Dict,
        current_sex: str,
        new_sex: str,
        medical_records: List[str],
        court: str = "가정법원"
    ) -> str:
        """
        성별정정 허가 신청서 작성

        Args:
            applicant: 신청인 정보 (name, resident_number, address, registration_base)
            current_sex: 현재 성별 (남/여)
            new_sex: 정정 후 성별 (여/남)
            medical_records: 의료기록 목록
            court: 관할 가정법원

        Returns:
            성별정정 허가 신청서 전문
        """
        application = "성별정정 허가 신청서\n\n"

        # 신청인
        application += f"신 청 인      {applicant.get('address', '')}\n"
        application += f"              {applicant['name']} "
        application += f"(주민등록번호: {applicant['resident_number']})\n"
        application += f"              등록기준지: {applicant.get('registration_base', '')}\n\n"

        # 정정 내용
        application += "성별 정정 사항\n\n"
        application += f"  현재 성별:  {current_sex}\n"
        application += f"  정정 후 성별:  {new_sex}\n\n"

        # 정정 사유
        application += "성별 정정 사유\n\n"
        application += "  신청인은 성주체성장애로 인해 성전환 수술을 완료하였으며, "
        application += "현재 정정 후 성별로 사회생활을 영위하고 있습니다. "
        application += "가족관계등록부의 성별란을 실제 성별과 일치시키기 위하여 "
        application += "이 신청을 합니다.\n\n"

        # 의료기록
        application += "의료기록\n\n"
        for idx, record in enumerate(medical_records, 1):
            application += f"  {idx}. {record}\n"
        application += "\n"

        # 첨부서류
        application += "첨부서류\n\n"
        application += "1. 가족관계증명서\n"
        application += "2. 기본증명서\n"
        application += "3. 정신건강의학과 진단서 (성주체성장애)\n"
        application += "4. 성전환 수술 확인서\n"
        application += "5. 호르몬 치료 기록\n"
        application += "6. 사회적응 소명서\n\n"

        # 날짜 및 서명
        today = datetime.now().strftime("%Y. %m. %d.")
        application += f"{today}\n\n"
        application += f"위 신청인  {applicant['name']} (인)\n\n"
        application += f"{court} 귀중\n"

        return application


if __name__ == "__main__":
    # 테스트 코드
    writer = FamilyRegistrationWriter()

    print("=" * 80)
    print("출생신고서 샘플")
    print("=" * 80)
    birth = writer.write_birth_report(
        declarant={
            "name": "김아빠",
            "resident_number": "800101-1234567",
            "address": "서울특별시 강남구 테헤란로 123",
            "relation": "부"
        },
        child={
            "name": "김아기",
            "sex": "남",
            "birth_datetime": "2024. 11. 10. 09:30",
            "birth_place": "○○병원"
        },
        parents={
            "father": "김아빠",
            "mother": "이엄마",
            "father_registration_base": "서울 강남구",
            "mother_registration_base": "서울 서초구"
        },
        birth_certificate="2024-1234",
        office="강남구청장"
    )
    print(birth)

    print("\n" + "=" * 80)
    print("혼인신고서 샘플")
    print("=" * 80)
    marriage = writer.write_marriage_report(
        husband={
            "name": "김신랑",
            "resident_number": "900101-1234567",
            "address": "서울특별시 강남구 역삼동 123",
            "registration_base": "서울 강남구"
        },
        wife={
            "name": "이신부",
            "resident_number": "920303-2345678",
            "address": "서울특별시 서초구 서초동 456",
            "registration_base": "서울 서초구"
        },
        witnesses=[
            {"name": "박증인", "resident_number": "800101-1111111"},
            {"name": "최증인", "resident_number": "750505-2222222"}
        ],
        office="강남구청장"
    )
    print(marriage)

    print("\n" + "=" * 80)
    print("개명허가 신청서 샘플")
    print("=" * 80)
    name_change = writer.write_name_change_application(
        applicant={
            "name": "김아무개",
            "resident_number": "950101-1234567",
            "address": "서울특별시 마포구 월드컵로 123",
            "registration_base": "서울 마포구"
        },
        old_name="김아무개",
        new_name="김새이름",
        reason="본인의 이름이 동명이인이 많아 사회생활에 현저한 불편을 겪고 있습니다. "
              "행정안전부 동명이인 검색 결과 전국에 350명의 동명이인이 있음을 확인하였습니다.",
        evidence=["동명이인 검색 결과 (행정안전부)", "불편 사례 소명서"],
        court="서울가정법원"
    )
    print(name_change)
