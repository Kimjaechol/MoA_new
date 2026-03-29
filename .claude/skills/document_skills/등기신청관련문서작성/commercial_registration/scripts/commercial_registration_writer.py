"""상업등기 신청서 작성 스킬"""
from typing import Dict, List

class CommercialRegistrationWriter:
    """상업등기 신청서 작성 클래스"""

    def write_corporation_incorporation(self, company_name: str, office: str,
                                       purposes: List[str], capital: str,
                                       total_shares: str, par_value: str,
                                       directors: List[Dict], representative: str) -> str:
        """주식회사 설립등기 신청서 작성"""
        app = "등 기 신 청 서\n\n"
        app += f"회사의 표시\n"
        app += f"  상    호     {company_name}\n"
        app += f"  본    점     {office}\n\n"
        app += "등기의 사유\n  설립등기\n\n"
        app += "등기할 사항\n"
        app += "  목    적   " + "\n             ".join([f"{i+1}. {p}" for i, p in enumerate(purposes)]) + "\n\n"
        app += "  공고방법   관보에 게재한다\n\n"
        app += f"  자본금     금 {capital}원\n\n"
        app += f"  발행주식의 총수     {total_shares}주\n\n"
        app += f"  1주의 금액         금 {par_value}원\n\n"

        for d in directors:
            app += f"  이    사   {d['address']}\n"
            app += f"             {d['name']} (주민등록번호: {d['resident_number']})\n\n"

        rep_address = next((d['address'] for d in directors if d['name'] == representative), "")
        app += f"  대표이사   {representative}\n"
        app += f"            ({rep_address})\n\n"

        app += "첨부서류\n  정관                          1통\n  발기인 결의서                  1통\n"
        app += "  창립총회의사록                 1통\n  이사 선임결의서                1통\n"
        app += "  이사 취임승낙서                각 1통\n  대표이사 선임결의서            1통\n"
        app += "  대표이사 취임승낙서            1통\n  이사 인감증명서                각 1통\n"
        app += "  자본금 납입증명서              1통\n  본점 사용승낙서                1통\n\n"
        app += "등록면허세   금 150,000원\n수 수 료     금  37,200원\n\n"
        app += "위 신청인     " + company_name + "\n              설립시 대표이사 " + representative + " (인)\n\n"
        app += "서울중앙지방법원 등기국 상업등기과 귀중\n"
        return app
