"""HWPX Document creation and manipulation library.

Provides a high-level API for creating and editing HWPX (Korean Hangul Word
Processor XML) documents. HWPX is a ZIP archive of OWPML XML files.

Usage:
    from hwpx_document import HwpxDocument

    doc = HwpxDocument()
    doc.add_paragraph("안녕하세요", font_size=12)
    doc.save("output.hwpx")
"""

import os
import zipfile
from xml.etree import ElementTree as ET

# OWPML Namespaces
NS = {
    "hp": "http://www.hancom.co.kr/hwpml/2011/paragraph",
    "ha": "http://www.hancom.co.kr/hwpml/2011/app",
    "hc": "http://www.hancom.co.kr/hwpml/2011/common",
    "hs": "http://www.hancom.co.kr/hwpml/2011/section",
    "odf": "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0",
}

# Register namespaces for clean XML output
for prefix, uri in NS.items():
    ET.register_namespace(prefix, uri)


def mm_to_hwpunit(mm):
    """Convert millimeters to HWPUNIT (1/7200 inch)."""
    return round(mm * 7200 / 25.4)


def pt_to_charsize(pt):
    """Convert point size to OWPML char height (1/100 pt)."""
    return round(pt * 100)


class HwpxDocument:
    """High-level HWPX document builder."""

    def __init__(self):
        self._paragraphs = []
        self._tables = []
        self._content_items = []  # ordered list of (type, data)
        self._char_props = []
        self._para_props = []
        self._styles = []
        self._fonts = ["함초롬바탕"]
        self._page = {
            "width": mm_to_hwpunit(210),
            "height": mm_to_hwpunit(297),
            "margin_top": mm_to_hwpunit(20),
            "margin_bottom": mm_to_hwpunit(15),
            "margin_left": mm_to_hwpunit(20),
            "margin_right": mm_to_hwpunit(20),
            "header": mm_to_hwpunit(15),
            "footer": mm_to_hwpunit(15),
        }
        self._setup_defaults()

    def _setup_defaults(self):
        # Default body style: 12pt, justify, 160% line spacing
        self._char_props.append({
            "id": 0, "height": pt_to_charsize(12),
            "color": "#000000", "bold": False,
        })
        self._para_props.append({
            "id": 0, "align": "JUSTIFY",
            "line_spacing": 160, "before": 0, "after": 0,
        })
        self._styles.append({
            "id": 0, "name": "본문", "eng": "Body",
            "para_ref": 0, "char_ref": 0,
        })

        # Title style: 18pt bold, center, spacing
        self._char_props.append({
            "id": 1, "height": pt_to_charsize(18),
            "color": "#000000", "bold": True,
        })
        self._para_props.append({
            "id": 1, "align": "CENTER",
            "line_spacing": 160, "before": 400, "after": 400,
        })
        self._styles.append({
            "id": 1, "name": "제목", "eng": "Title",
            "para_ref": 1, "char_ref": 1,
        })

    def set_page(self, width_mm=210, height_mm=297,
                 margin_top_mm=20, margin_bottom_mm=15,
                 margin_left_mm=20, margin_right_mm=20):
        self._page.update({
            "width": mm_to_hwpunit(width_mm),
            "height": mm_to_hwpunit(height_mm),
            "margin_top": mm_to_hwpunit(margin_top_mm),
            "margin_bottom": mm_to_hwpunit(margin_bottom_mm),
            "margin_left": mm_to_hwpunit(margin_left_mm),
            "margin_right": mm_to_hwpunit(margin_right_mm),
        })

    def set_legal_format(self):
        """Set court-standard Korean legal document formatting."""
        self.set_page(margin_top_mm=20, margin_bottom_mm=15,
                      margin_left_mm=20, margin_right_mm=20)

    def _get_or_create_char_prop(self, font_size=12, bold=False, color="#000000"):
        height = pt_to_charsize(font_size)
        for cp in self._char_props:
            if cp["height"] == height and cp["bold"] == bold and cp["color"] == color:
                return cp["id"]
        new_id = len(self._char_props)
        self._char_props.append({
            "id": new_id, "height": height,
            "color": color, "bold": bold,
        })
        return new_id

    def _get_or_create_para_prop(self, align="JUSTIFY", line_spacing=160):
        for pp in self._para_props:
            if pp["align"] == align and pp["line_spacing"] == line_spacing:
                return pp["id"]
        new_id = len(self._para_props)
        self._para_props.append({
            "id": new_id, "align": align,
            "line_spacing": line_spacing, "before": 0, "after": 0,
        })
        return new_id

    def add_paragraph(self, text, font_size=12, bold=False,
                      align="JUSTIFY", color="#000000"):
        char_id = self._get_or_create_char_prop(font_size, bold, color)
        align_upper = align.upper()
        para_id = self._get_or_create_para_prop(align_upper)
        self._content_items.append(("para", {
            "text": text,
            "char_ref": char_id,
            "para_ref": para_id,
            "style_ref": 0,
        }))

    def add_heading(self, text, level=1):
        sizes = {1: 18, 2: 16, 3: 14}
        self.add_paragraph(text, font_size=sizes.get(level, 14),
                           bold=True, align="CENTER")

    def add_legal_parties(self, plaintiff, defendant):
        self.add_paragraph("")
        self.add_paragraph(f"원  고    {plaintiff}", font_size=12)
        self.add_paragraph(f"피  고    {defendant}", font_size=12)
        self.add_paragraph("")

    def add_table(self, rows):
        """Add a table. rows is a list of lists of strings."""
        if not rows:
            return
        self._content_items.append(("table", rows))

    def _build_header_xml(self):
        root = ET.Element(f"{{{NS['ha']}}}HWPMLHead", attrib={"version": "1.1"})
        ref_list = ET.SubElement(root, f"{{{NS['ha']}}}refList")

        # Fonts
        fontfaces = ET.SubElement(ref_list, f"{{{NS['ha']}}}fontfaces")
        for lang in ["HANGUL", "LATIN", "HANJA"]:
            ff = ET.SubElement(fontfaces, f"{{{NS['ha']}}}fontface", lang=lang)
            for i, face in enumerate(self._fonts):
                ET.SubElement(ff, f"{{{NS['ha']}}}font", face=face, type="TTF", id=str(i))

        # Border fills
        bfs = ET.SubElement(ref_list, f"{{{NS['ha']}}}borderFills")
        # Empty border (id=1)
        bf1 = ET.SubElement(bfs, f"{{{NS['hc']}}}borderFill", id="1")
        for side in ["leftBorder", "rightBorder", "topBorder", "bottomBorder"]:
            ET.SubElement(bf1, f"{{{NS['hc']}}}{side}", type="NONE", width="0.1mm", color="#000000")
        # Solid border for tables (id=2)
        bf2 = ET.SubElement(bfs, f"{{{NS['hc']}}}borderFill", id="2")
        for side in ["leftBorder", "rightBorder", "topBorder", "bottomBorder"]:
            ET.SubElement(bf2, f"{{{NS['hc']}}}{side}", type="SOLID", width="0.12mm", color="#000000")

        # Char properties
        cps = ET.SubElement(ref_list, f"{{{NS['ha']}}}charProperties")
        for cp in self._char_props:
            attrs = {"id": str(cp["id"]), "height": str(cp["height"]), "textColor": cp["color"]}
            if cp["bold"]:
                attrs["bold"] = "true"
            el = ET.SubElement(cps, f"{{{NS['hc']}}}charPr", **attrs)
            ET.SubElement(el, f"{{{NS['hc']}}}fontRef", hangul="0", latin="0", hanja="0")

        # Para properties
        pps = ET.SubElement(ref_list, f"{{{NS['ha']}}}paraProperties")
        for pp in self._para_props:
            el = ET.SubElement(pps, f"{{{NS['hc']}}}paraPr",
                               id=str(pp["id"]), align=pp["align"])
            ET.SubElement(el, f"{{{NS['hc']}}}margin", indent="0", left="0", right="0")
            ET.SubElement(el, f"{{{NS['hc']}}}lineSpacing",
                          type="PERCENT", value=str(pp["line_spacing"]))
            ET.SubElement(el, f"{{{NS['hc']}}}spacing",
                          before=str(pp["before"]), after=str(pp["after"]))

        # Styles
        styles = ET.SubElement(ref_list, f"{{{NS['ha']}}}styles")
        for st in self._styles:
            ET.SubElement(styles, f"{{{NS['hc']}}}style",
                          id=str(st["id"]), type="PARA",
                          name=st["name"], engName=st["eng"],
                          paraPrIDRef=str(st["para_ref"]),
                          charPrIDRef=str(st["char_ref"]))

        return root

    def _build_section_xml(self):
        root = ET.Element(f"{{{NS['hs']}}}sec")
        p = self._page

        page_def = ET.SubElement(root, f"{{{NS['hs']}}}pageDef",
                                 orientation="PORTRAIT",
                                 width=str(p["width"]),
                                 height=str(p["height"]),
                                 gutterType="LEFT_ONLY")
        ET.SubElement(page_def, f"{{{NS['hs']}}}margin",
                       header=str(p["header"]), footer=str(p["footer"]),
                       top=str(p["margin_top"]), bottom=str(p["margin_bottom"]),
                       left=str(p["margin_left"]), right=str(p["margin_right"]),
                       gutter="0")

        for item_type, data in self._content_items:
            if item_type == "para":
                para = ET.SubElement(root, f"{{{NS['hp']}}}p",
                                     paraPrIDRef=str(data["para_ref"]),
                                     styleIDRef=str(data["style_ref"]))
                if data["text"]:
                    run = ET.SubElement(para, f"{{{NS['hp']}}}run",
                                        charPrIDRef=str(data["char_ref"]))
                    t = ET.SubElement(run, f"{{{NS['hp']}}}t")
                    t.text = data["text"]
            elif item_type == "table":
                rows = data
                col_count = max(len(r) for r in rows) if rows else 0
                row_count = len(rows)
                para = ET.SubElement(root, f"{{{NS['hp']}}}p",
                                     paraPrIDRef="0", styleIDRef="0")
                run = ET.SubElement(para, f"{{{NS['hp']}}}run", charPrIDRef="0")
                tbl = ET.SubElement(run, f"{{{NS['hp']}}}tbl",
                                    colCount=str(col_count),
                                    rowCount=str(row_count),
                                    cellSpacing="0",
                                    borderFillIDRef="2")
                for row in rows:
                    tr = ET.SubElement(tbl, f"{{{NS['hp']}}}tr")
                    for cell_text in row:
                        tc = ET.SubElement(tr, f"{{{NS['hp']}}}tc",
                                           colSpan="1", rowSpan="1",
                                           borderFillIDRef="2")
                        tcp = ET.SubElement(tc, f"{{{NS['hp']}}}tcPr")
                        ET.SubElement(tcp, f"{{{NS['hp']}}}cellMargin",
                                       left="170", right="170", top="28", bottom="28")
                        cp = ET.SubElement(tc, f"{{{NS['hp']}}}p",
                                           paraPrIDRef="0", styleIDRef="0")
                        cr = ET.SubElement(cp, f"{{{NS['hp']}}}run", charPrIDRef="0")
                        ct = ET.SubElement(cr, f"{{{NS['hp']}}}t")
                        ct.text = str(cell_text)

        return root

    def save(self, filepath):
        """Save as .hwpx file (ZIP archive)."""
        with zipfile.ZipFile(filepath, "w", zipfile.ZIP_DEFLATED) as zf:
            # version.xml
            zf.writestr("version.xml", self._xml_str(self._build_version_xml()))

            # META-INF/manifest.xml
            zf.writestr("META-INF/manifest.xml", self._xml_str(self._build_manifest_xml()))

            # META-INF/container.xml
            zf.writestr("META-INF/container.xml", self._xml_str(self._build_container_xml()))

            # Contents/header.xml
            zf.writestr("Contents/header.xml", self._xml_str(self._build_header_xml()))

            # Contents/section0.xml
            zf.writestr("Contents/section0.xml", self._xml_str(self._build_section_xml()))

    def save_unpacked(self, dirpath):
        """Save as unpacked directory structure."""
        os.makedirs(os.path.join(dirpath, "META-INF"), exist_ok=True)
        os.makedirs(os.path.join(dirpath, "Contents"), exist_ok=True)
        os.makedirs(os.path.join(dirpath, "BinData"), exist_ok=True)

        self._write_xml(os.path.join(dirpath, "version.xml"), self._build_version_xml())
        self._write_xml(os.path.join(dirpath, "META-INF", "manifest.xml"), self._build_manifest_xml())
        self._write_xml(os.path.join(dirpath, "META-INF", "container.xml"), self._build_container_xml())
        self._write_xml(os.path.join(dirpath, "Contents", "header.xml"), self._build_header_xml())
        self._write_xml(os.path.join(dirpath, "Contents", "section0.xml"), self._build_section_xml())

    @staticmethod
    def open(dirpath):
        """Open an unpacked HWPX directory for editing."""
        doc = HwpxDocument()
        doc._unpacked_dir = dirpath
        # Parse existing section0.xml if present
        section_path = os.path.join(dirpath, "Contents", "section0.xml")
        if os.path.exists(section_path):
            doc._existing_section = ET.parse(section_path)
        return doc

    def find_replace(self, old_text, new_text):
        """Find and replace text in all section XML files."""
        if not hasattr(self, "_unpacked_dir"):
            return
        contents_dir = os.path.join(self._unpacked_dir, "Contents")
        for fname in sorted(os.listdir(contents_dir)):
            if fname.startswith("section") and fname.endswith(".xml"):
                fpath = os.path.join(contents_dir, fname)
                content = open(fpath, "r", encoding="utf-8").read()
                if old_text in content:
                    content = content.replace(old_text, new_text)
                    open(fpath, "w", encoding="utf-8").write(content)

    def _build_version_xml(self):
        return ET.Element(f"{{{NS['ha']}}}HWPMLPackMainVersion", version="1.1")

    def _build_manifest_xml(self):
        root = ET.Element(f"{{{NS['odf']}}}manifest")
        entries = [
            ("/", "application/hwp+zip"),
            ("version.xml", "application/xml"),
            ("Contents/header.xml", "application/xml"),
            ("Contents/section0.xml", "application/xml"),
        ]
        for path, media in entries:
            ET.SubElement(root, f"{{{NS['odf']}}}file-entry",
                          attrib={f"{{{NS['odf']}}}full-path": path,
                                  f"{{{NS['odf']}}}media-type": media})
        return root

    def _build_container_xml(self):
        root = ET.Element("container")
        rfs = ET.SubElement(root, "rootfiles")
        ET.SubElement(rfs, "rootfile", attrib={
            "full-path": "Contents/header.xml",
            "media-type": "application/xml"
        })
        return root

    @staticmethod
    def _xml_str(element):
        return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' + \
               ET.tostring(element, encoding="unicode")

    @staticmethod
    def _write_xml(filepath, element):
        tree = ET.ElementTree(element)
        ET.indent(tree, space="  ")
        tree.write(filepath, encoding="UTF-8", xml_declaration=True)
