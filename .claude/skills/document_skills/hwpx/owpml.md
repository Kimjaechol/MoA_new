# OWPML (Open Word Processing Markup Language) Reference for HWPX

## Overview

OWPML is the XML schema used inside HWPX files, standardized as KS X 6101.
This document covers the key XML elements needed to create and edit HWPX documents.

## Namespaces

```xml
xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app"
xmlns:hc="http://www.hancom.co.kr/hwpml/2011/common"
xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0"
```

## File Structure

### version.xml
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ha:HWPMLPackMainVersion version="1.1"
    xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app"/>
```

### META-INF/manifest.xml
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<odf:manifest xmlns:odf="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">
  <odf:file-entry odf:full-path="/" odf:media-type="application/hwp+zip"/>
  <odf:file-entry odf:full-path="version.xml" odf:media-type="application/xml"/>
  <odf:file-entry odf:full-path="Contents/header.xml" odf:media-type="application/xml"/>
  <odf:file-entry odf:full-path="Contents/section0.xml" odf:media-type="application/xml"/>
  <odf:file-entry odf:full-path="Preview/PrvImage.png" odf:media-type="image/png"/>
</odf:manifest>
```

### META-INF/container.xml
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<container>
  <rootfiles>
    <rootfile full-path="Contents/header.xml" media-type="application/xml"/>
  </rootfiles>
</container>
```

### Contents/header.xml — Document Settings & Styles

This is the main document settings file containing page layout, fonts, styles,
and numbering definitions.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ha:HWPMLHead version="1.1"
    xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app"
    xmlns:hc="http://www.hancom.co.kr/hwpml/2011/common">

  <ha:refList>
    <!-- Font faces -->
    <ha:fontfaces>
      <ha:fontface lang="HANGUL">
        <ha:font face="함초롬바탕" type="TTF"/>
      </ha:fontface>
      <ha:fontface lang="LATIN">
        <ha:font face="함초롬바탕" type="TTF"/>
      </ha:fontface>
      <ha:fontface lang="HANJA">
        <ha:font face="함초롬바탕" type="TTF"/>
      </ha:fontface>
    </ha:fontfaces>

    <!-- Border/Fill styles -->
    <ha:borderFills>
      <hc:borderFill id="1">
        <hc:slash type="NONE"/>
        <hc:backSlash type="NONE"/>
        <hc:leftBorder type="NONE" width="0.1mm" color="#000000"/>
        <hc:rightBorder type="NONE" width="0.1mm" color="#000000"/>
        <hc:topBorder type="NONE" width="0.1mm" color="#000000"/>
        <hc:bottomBorder type="NONE" width="0.1mm" color="#000000"/>
      </hc:borderFill>
      <hc:borderFill id="2">
        <hc:leftBorder type="SOLID" width="0.12mm" color="#000000"/>
        <hc:rightBorder type="SOLID" width="0.12mm" color="#000000"/>
        <hc:topBorder type="SOLID" width="0.12mm" color="#000000"/>
        <hc:bottomBorder type="SOLID" width="0.12mm" color="#000000"/>
      </hc:borderFill>
    </ha:borderFills>

    <!-- Character properties (charPrs) -->
    <ha:charProperties>
      <hc:charPr id="0" height="1000" textColor="#000000">
        <hc:fontRef hangul="0" latin="0" hanja="0"/>
      </hc:charPr>
      <hc:charPr id="1" height="1800" textColor="#000000" bold="true">
        <hc:fontRef hangul="0" latin="0" hanja="0"/>
      </hc:charPr>
    </ha:charProperties>

    <!-- Paragraph properties (paraPrs) -->
    <ha:paraProperties>
      <hc:paraPr id="0" align="JUSTIFY">
        <hc:margin indent="0" left="0" right="0"/>
        <hc:lineSpacing type="PERCENT" value="160"/>
        <hc:spacing before="0" after="0"/>
      </hc:paraPr>
      <hc:paraPr id="1" align="CENTER">
        <hc:margin indent="0" left="0" right="0"/>
        <hc:lineSpacing type="PERCENT" value="160"/>
        <hc:spacing before="400" after="400"/>
      </hc:paraPr>
    </ha:paraProperties>

    <!-- Named styles -->
    <ha:styles>
      <hc:style id="0" type="PARA" name="본문" engName="Body"
                paraPrIDRef="0" charPrIDRef="0"/>
      <hc:style id="1" type="PARA" name="제목" engName="Title"
                paraPrIDRef="1" charPrIDRef="1"/>
    </ha:styles>
  </ha:refList>

  <!-- Document-wide settings -->
  <ha:docOption>
    <ha:linkinfo path="" pageInLink="false" externLink="false"/>
  </ha:docOption>

</ha:HWPMLHead>
```

### Contents/section0.xml — Main Content

This file contains the actual document content: paragraphs, tables, images.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/common"
        xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app">

  <!-- Page definition (용지 설정) -->
  <hs:pageDef orientation="PORTRAIT"
              width="59528" height="84188"
              gutterType="LEFT_ONLY">
    <hs:margin header="4252" footer="4252"
               top="5668" bottom="4252"
               left="5668" right="5668"
               gutter="0"/>
  </hs:pageDef>

  <!-- Paragraph (문단) -->
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>본문 텍스트를 여기에 작성합니다.</hp:t>
    </hp:run>
  </hp:p>

  <!-- Bold title paragraph -->
  <hp:p paraPrIDRef="1" styleIDRef="1">
    <hp:run charPrIDRef="1">
      <hp:t>제목</hp:t>
    </hp:run>
  </hp:p>

</hs:sec>
```

## Key XML Elements

### Paragraphs (`hp:p`)

Attributes:
- `paraPrIDRef` — reference to paragraph property ID in header.xml
- `styleIDRef` — reference to style ID in header.xml

Children:
- `hp:run` — a text run with character formatting
- `hp:lineseg` — line segment info (auto-generated)

### Text Runs (`hp:run`)

Attributes:
- `charPrIDRef` — reference to character property ID in header.xml

Children:
- `hp:t` — the actual text content
- `hp:tab` — tab character
- `hp:lineBreak` — line break within paragraph

### Tables (`hp:tbl`)

```xml
<hp:p>
  <hp:run>
    <hp:tbl colCount="3" rowCount="2" cellSpacing="0"
            borderFillIDRef="2">
      <hp:tr>
        <hp:tc colSpan="1" rowSpan="1"
               borderFillIDRef="2">
          <hp:tcPr>
            <hp:cellMargin left="170" right="170"
                          top="28" bottom="28"/>
          </hp:tcPr>
          <hp:p paraPrIDRef="0" styleIDRef="0">
            <hp:run charPrIDRef="0">
              <hp:t>셀 내용</hp:t>
            </hp:run>
          </hp:p>
        </hp:tc>
        <!-- more cells -->
      </hp:tr>
      <!-- more rows -->
    </hp:tbl>
  </hp:run>
</hp:p>
```

### Images (`hp:pic`)

```xml
<hp:p>
  <hp:run>
    <hp:pic>
      <hp:picRect x="0" y="0"/>
      <hc:img binaryItemIDRef="image1.png"/>
      <hp:shapeSize width="14000" height="10000"/>
    </hp:pic>
  </hp:run>
</hp:p>
```

Binary data is stored in `BinData/image1.png` and referenced by filename.

## Units

OWPML uses **HWPUNIT** (1/7200 inch = ~0.00353mm):
- A4 width: 59528 hwpunit = 210mm
- A4 height: 84188 hwpunit = 297mm
- 1mm = ~283.46 hwpunit
- Font size: in 1/100 pt (e.g., 1000 = 10pt, 1200 = 12pt)

Conversion helpers:
```python
def mm_to_hwpunit(mm): return round(mm * 7200 / 25.4)
def hwpunit_to_mm(hu): return round(hu * 25.4 / 7200, 2)
def pt_to_charsize(pt): return round(pt * 100)
```

## Page Layouts for Korean Legal Documents

### 법원 제출 문서 (Court Filing)
```
용지: A4 (210×297mm)
위 여백: 20mm, 아래 여백: 15mm
왼쪽 여백: 20mm, 오른쪽 여백: 20mm
머리말: 15mm, 꼬리말: 15mm
글꼴: 함초롬바탕 12pt, 줄간격 160%
```

### 행정기관 제출 문서 (Government Filing)
```
용지: A4 (210×297mm)
위 여백: 30mm, 아래 여백: 15mm
왼쪽 여백: 20mm, 오른쪽 여백: 15mm
글꼴: 함초롬바탕 또는 맑은고딕 11pt, 줄간격 160%
```

## Alignment Values

| Value | Korean | Description |
|-------|--------|-------------|
| `JUSTIFY` | 양쪽정렬 | Both sides aligned (default for body) |
| `LEFT` | 왼쪽정렬 | Left aligned |
| `CENTER` | 가운데정렬 | Center aligned (for titles) |
| `RIGHT` | 오른쪽정렬 | Right aligned |
| `DISTRIBUTE` | 배분정렬 | Distributed alignment |
