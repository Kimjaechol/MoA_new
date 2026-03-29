"""법인등기 신청서 작성 스킬"""
from typing import Dict, List

class CorporateRegistrationWriter:
    """법인등기 신청서 작성 클래스"""

    def write_incorporation(self, corp_type: str, name: str, office: str,
                           purposes: List[str], approval: Dict, assets: str,
                           directors: List[Dict], representative: str,
                           auditors: List[Dict] = None) -> str:
        """법인 설립등기 신청서 작성"""
        app = "등 기 신 청 서\n\n"
        app += f"법인의 표시\n"
        app += f"  명    칭     {name}\n"
        app += f"  주사무소     {office}\n\n"
        app += "등기의 사유\n  설립등기\n\n"
        app += "등기할 사항\n"
        app += "  목    적   " + "\n             ".join([f"{i+1}. {p}" for i, p in enumerate(purposes)]) + "\n\n"
        app += f"  설립허가   {approval['date']} {approval['authority']} 허가\n\n"
        app += f"  자산총액   금 {assets}원\n\n"

        for d in directors:
            app += f"  이    사   {d['address']}\n"
            app += f"             {d['name']} (주민등록번호: {d['resident_number']})\n\n"

        app += f"  대표이사   {representative}\n\n"

        if auditors:
            for a in auditors:
                app += f"  감    사   {a['address']}\n"
                app += f"             {a['name']} (주민등록번호: {a['resident_number']})\n\n"

        app += "첨부서류\n  설립허가서                     1통\n  정관                          1통\n"
        app += "  이사 및 감사 선임결의서        1통\n  이사 및 감사 취임승낙서        각 1통\n"
        app += "  이사 및 감사 인감증명서        각 1통\n  재산목록                      1통\n\n"
        app += "등록면허세   금 112,500원\n수 수 료     금  34,200원\n\n"
        app += "위 신청인     " + name + "\n              대표이사 " + representative + " (인)\n\n"
        app += "서울중앙지방법원 등기국 법인등기과 귀중\n"
        return app
