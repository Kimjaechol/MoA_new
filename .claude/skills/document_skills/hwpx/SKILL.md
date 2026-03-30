---
name: hwpx
description: "Korean HWPX document creation, editing, and analysis. HWPX is the XML-based successor to HWP (Hangul Word Processor) and the standard document format in Korean government, law, and business. When Claude needs to: (1) Create new .hwpx documents, (2) Edit existing .hwpx files, (3) Extract text from .hwpx files, (4) Convert between .hwpx and other formats"
license: Proprietary. LICENSE.txt has complete terms
version: 1.0.0
---

# HWPX Creation, Editing, and Analysis

## Overview

A user may ask you to create, edit, or analyze the contents of a .hwpx file. HWPX is the Korean standard document format based on OWPML (Open Word Processing Markup Language), standardized as KS X 6101. Like .docx, a .hwpx file is a ZIP archive containing XML files and resources. HWPX is the mandatory format for Korean government submissions, legal filings, and many business documents.

## Key Differences from DOCX

| Aspect | DOCX (OOXML) | HWPX (OWPML) |
|--------|-------------|--------------|
| Root namespace | `w:` (wordprocessingML) | `hp:` (hwpml) |
| Main content | `word/document.xml` | `Contents/section0.xml` |
| Styles | `word/styles.xml` | `Contents/header.xml` (document settings + styles) |
| Sections | In document.xml | Separate files: `section0.xml`, `section1.xml`, ... |
| Page layout | `w:sectPr` | `hp:pagedef` in header.xml |
| Paragraphs | `w:p` | `hp:p` |
| Runs (text) | `w:r` / `w:t` | `hp:run` / `hp:t` |
| Tables | `w:tbl` / `w:tr` / `w:tc` | `hp:tbl` / `hp:tr` / `hp:tc` |
| Binary data | `word/media/` | `BinData/` |
| Manifest | `[Content_Types].xml` | `META-INF/manifest.xml` |

## Workflow Decision Tree

### Reading/Analyzing Content
Use "Text extraction" section below

### Creating New HWPX Document
Use "Creating a new HWPX document" workflow

### Editing Existing HWPX Document
Use "Editing an existing HWPX document" workflow

## Reading and analyzing content

### Text extraction

HWPX is a ZIP containing XML. Extract text by parsing the section XML files:

```bash
# Unpack HWPX (it's a ZIP archive)
python scripts/unpack.py document.hwpx unpacked/

# Read the main content
cat unpacked/Contents/section0.xml
```

Alternatively, use `hwp5txt` from the `pyhwp` package if available:
```bash
pip install pyhwp
hwp5txt document.hwpx
```

### Raw XML access

Unpack the HWPX file and inspect the XML structure:

```bash
python scripts/unpack.py document.hwpx unpacked/
```

**Key file structure of an HWPX archive:**
```
document.hwpx (ZIP)
├── META-INF/
│   ├── manifest.xml          # File manifest (like [Content_Types].xml)
│   └── container.xml         # Container metadata
├── Contents/
│   ├── header.xml            # Document settings, page layout, styles, fonts
│   ├── section0.xml          # Main content (first section)
│   ├── section1.xml          # Second section (if multi-section)
│   └── ...
├── BinData/                  # Embedded images and binary data
│   ├── image1.png
│   └── ...
├── Preview/
│   └── PrvImage.png          # Thumbnail preview
└── version.xml               # HWPX version info
```

## Creating a new HWPX document

Since HWPX is a ZIP of XML files, you create documents by generating the correct XML structure and packaging it.

### Workflow

1. **MANDATORY - READ ENTIRE FILE**: Read [`owpml.md`](owpml.md) completely for the OWPML XML schema reference.
2. Create the directory structure with required XML files.
3. Use the helper script to generate a properly formatted HWPX.
4. Pack the directory into a .hwpx file.

### Quick creation using the helper script

```bash
# Set PYTHONPATH to include the scripts directory
export PYTHONPATH="$(pwd)/scripts:$PYTHONPATH"

# Create a simple document
python3 - <<'EOF'
from hwpx_document import HwpxDocument

doc = HwpxDocument()

# Set page layout (A4, Korean standard margins)
doc.set_page(width_mm=210, height_mm=297,
             margin_top_mm=20, margin_bottom_mm=15,
             margin_left_mm=20, margin_right_mm=20)

# Add content
doc.add_paragraph("제목", font_size=18, bold=True, align="center")
doc.add_paragraph("")  # empty line
doc.add_paragraph("본문 내용을 여기에 작성합니다.", font_size=12)
doc.add_paragraph("한글 HWPX 문서를 프로그래밍 방식으로 생성합니다.", font_size=12)

# Add a table
doc.add_table([
    ["항목", "내용", "비고"],
    ["성명", "홍길동", ""],
    ["주소", "서울특별시 강남구", ""],
    ["연락처", "010-1234-5678", ""],
])

# Save
doc.save("output.hwpx")
EOF
```

### Manual creation (for complex documents)

1. Create the directory structure:
```bash
mkdir -p hwpx_dir/META-INF hwpx_dir/Contents hwpx_dir/BinData hwpx_dir/Preview
```

2. Generate `META-INF/manifest.xml`, `Contents/header.xml`, `Contents/section0.xml`, and `version.xml` following the OWPML spec in [`owpml.md`](owpml.md).

3. Pack into HWPX:
```bash
python scripts/pack.py hwpx_dir output.hwpx
```

## Editing an existing HWPX document

### Workflow

1. **MANDATORY - READ ENTIRE FILE**: Read [`owpml.md`](owpml.md) completely.
2. Unpack: `python scripts/unpack.py document.hwpx unpacked/`
3. Edit the XML files in `unpacked/Contents/` using the HwpxDocument library or direct XML manipulation.
4. Pack: `python scripts/pack.py unpacked/ edited.hwpx`

### Example: Replace text in a document

```python
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'scripts'))
from hwpx_document import HwpxDocument

doc = HwpxDocument.open("unpacked/")

# Find and replace text
doc.find_replace("원고", "피고")
doc.find_replace("2025년", "2026년")

doc.save_unpacked("unpacked/")
```

Then pack: `python scripts/pack.py unpacked/ result.hwpx`

## Converting HWPX to other formats

### HWPX → PDF
```bash
# Using LibreOffice (supports HWPX natively on Linux with Korean locale)
soffice --headless --convert-to pdf document.hwpx

# If LibreOffice doesn't support HWPX, convert via DOCX first
# HWPX → DOCX → PDF
```

### HWPX → DOCX
```bash
# Direct conversion if LibreOffice supports it
soffice --headless --convert-to docx document.hwpx
```

### HWPX → Images (for visual analysis)
```bash
soffice --headless --convert-to pdf document.hwpx
pdftoppm -jpeg -r 150 document.pdf page
```

## Korean Legal Document Templates

HWPX is the mandatory format for Korean legal documents. Common templates:

| Template | Korean | Use Case |
|----------|--------|----------|
| 소장 | Complaint | Civil lawsuit filing |
| 답변서 | Answer | Defendant's response |
| 준비서면 | Preparatory brief | Pre-trial arguments |
| 내용증명 | Certified content mail | Legal notice with proof |
| 계약서 | Contract | General agreements |
| 진정서 | Petition | Administrative petition |

Use the `HwpxDocument` helper with Korean legal formatting presets:

```python
doc = HwpxDocument()
doc.set_legal_format()  # Sets court-standard margins and fonts
doc.add_heading("소    장", level=1)  # Korean legal title spacing
doc.add_legal_parties(plaintiff="원고 홍길동", defendant="피고 김철수")
# ... legal document body
doc.save("complaint.hwpx")
```

## Code Style Guidelines

**IMPORTANT**: When generating code for HWPX operations:
- Write concise code
- Use Korean comments for Korean legal documents
- Follow OWPML XML namespace conventions (`hp:`, `hc:`, `ha:`)
- Preserve original document encoding (UTF-8)

## Dependencies

Required dependencies (install if not available):

- **Python 3**: For XML generation and manipulation
- **defusedxml**: `pip install defusedxml` (for secure XML parsing)
- **LibreOffice**: `sudo apt-get install libreoffice` (for format conversion)
- **Poppler**: `sudo apt-get install poppler-utils` (for PDF to image conversion)
- **pyhwp** (optional): `pip install pyhwp` (for legacy HWP support)
