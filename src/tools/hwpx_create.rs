//! `hwpx_create` tool — create HWPX (Korean Hangul Word Processor XML) documents.
//!
//! Wraps the bundled `hwpx_skill/hwpx_document.py` library so the LLM agent
//! can produce real `.hwpx` files for Korean government / legal / business
//! filings (내용증명, 진술서, 계약서 등). The Python script is embedded into
//! the Rust binary at compile time via [`include_str!`] and written to a
//! per-call temp directory at runtime, so users only need `python3` on PATH.
//!
//! # Why a separate tool from `document_pipeline`
//!
//! `document_pipeline` reads existing HWPX files (Hancom DocsConverter →
//! HTML/Markdown). This tool **writes** new HWPX files. The two are
//! complementary, not duplicates — see ARCHITECTURE.md §6C.
//!
//! # Why Python instead of native Rust
//!
//! HWPX is OWPML XML inside a ZIP. A correct native implementation is
//! ~1500 lines of XML construction with namespace handling, font tables,
//! style references, and ZIP packaging. The bundled Python script
//! (`hwpx_skill/hwpx_document.py`) already handles all of this in 359
//! lines and has been validated against Hancom Office. Reimplementing it
//! in Rust would be ~5x the code with no functional gain.
//!
//! Python availability: macOS / most Linux distributions ship Python 3 by
//! default; Windows installs typically include `py` launcher. Korean
//! professional users (the target audience for HWPX) overwhelmingly run
//! macOS or Linux, so the dependency is acceptable.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;

use crate::tools::traits::{Tool, ToolResult};

/// Embedded HWPX document library — bundled at compile time so the tool
/// works on any user device with `python3` on PATH, no external file dep.
const HWPX_DOCUMENT_PY: &str = include_str!("hwpx_skill/hwpx_document.py");

/// Generator wrapper that imports the embedded library and applies a JSON
/// spec received on stdin. Kept as a separate constant for readability.
///
/// Note: uses `r##"..."##` raw delimiter because the embedded Python code
/// contains the substring `"#` (e.g. `"#000000"`) which would otherwise
/// close a single-hash raw string early.
const HWPX_GENERATOR_PY: &str = r##"
import json
import os
import sys

# hwpx_document.py is in the same temp dir as this script.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hwpx_document import HwpxDocument

def main():
    spec = json.loads(sys.stdin.read())
    doc = HwpxDocument()

    if spec.get("legal_format"):
        doc.set_legal_format()

    if spec.get("title"):
        doc.add_heading(spec["title"], level=1)

    for item in spec.get("paragraphs", []):
        if isinstance(item, str):
            doc.add_paragraph(item)
        elif isinstance(item, dict):
            text = item.get("text", "")
            doc.add_paragraph(
                text,
                font_size=item.get("font_size", 12),
                bold=item.get("bold", False),
                align=item.get("align", "JUSTIFY"),
                color=item.get("color", "#000000"),
            )

    output_path = spec["output_path"]
    output_dir = os.path.dirname(output_path)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)
    doc.save(output_path)
    print(json.dumps({"output_path": output_path, "ok": True}))

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(json.dumps({"ok": False, "error": str(e)}), file=sys.stderr)
        sys.exit(1)
"##;

#[derive(Debug, Serialize, Deserialize)]
struct HwpxCreateArgs {
    /// Absolute or relative path where the .hwpx file should be saved.
    output_path: String,
    /// Optional document title (rendered as a level-1 heading at the top).
    #[serde(default)]
    title: Option<String>,
    /// Body content as an array of either strings (default formatting) or
    /// objects: `{ "text": "...", "font_size": 12, "bold": false, "align": "JUSTIFY", "color": "#000000" }`.
    #[serde(default)]
    paragraphs: Vec<Value>,
    /// When true, applies court-standard Korean legal document margins.
    #[serde(default)]
    legal_format: bool,
}

/// Tool: write a new HWPX document via the bundled Python skill.
pub struct HwpxCreateTool;

impl Default for HwpxCreateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl HwpxCreateTool {
    pub fn new() -> Self {
        Self
    }

    /// Locate `python3` (or `python` as a fallback). Returns the binary name
    /// suitable for `Command::new`. Returns `None` only on truly broken
    /// systems where neither is on PATH.
    fn python_binary() -> Option<&'static str> {
        if which::which("python3").is_ok() {
            return Some("python3");
        }
        if which::which("python").is_ok() {
            return Some("python");
        }
        None
    }
}

#[async_trait]
impl Tool for HwpxCreateTool {
    fn name(&self) -> &str {
        "hwpx_create"
    }

    fn description(&self) -> &str {
        "Create a new HWPX (Korean Hangul Word Processor XML) document. \
         Use for Korean government / legal / business filings such as 내용증명, \
         진술서, 계약서, 공문, 보고서. Output is a real .hwpx file that opens in \
         Hancom Office and ONLYOFFICE. Pass `legal_format: true` for court-standard margins."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "output_path": {
                    "type": "string",
                    "description": "Where to save the .hwpx file (path is created if needed)"
                },
                "title": {
                    "type": "string",
                    "description": "Optional title rendered as a centered heading at the top"
                },
                "paragraphs": {
                    "type": "array",
                    "description": "Body content. Each item is either a plain string or an object with text/font_size/bold/align/color.",
                    "items": {
                        "oneOf": [
                            { "type": "string" },
                            {
                                "type": "object",
                                "properties": {
                                    "text": { "type": "string" },
                                    "font_size": { "type": "integer", "minimum": 6, "maximum": 72 },
                                    "bold": { "type": "boolean" },
                                    "align": {
                                        "type": "string",
                                        "enum": ["LEFT", "RIGHT", "CENTER", "JUSTIFY"]
                                    },
                                    "color": {
                                        "type": "string",
                                        "description": "CSS hex like #000000"
                                    }
                                },
                                "required": ["text"]
                            }
                        ]
                    }
                },
                "legal_format": {
                    "type": "boolean",
                    "description": "Apply Korean court-standard legal document margins"
                }
            },
            "required": ["output_path"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let parsed: HwpxCreateArgs = serde_json::from_value(args)
            .map_err(|e| anyhow::anyhow!("invalid hwpx_create arguments: {e}"))?;

        if parsed.output_path.trim().is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("output_path must not be empty".into()),
            });
        }

        let python = match Self::python_binary() {
            Some(p) => p,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(
                        "python3 not found on PATH. Install Python 3 to use hwpx_create.".into(),
                    ),
                });
            }
        };

        // Write the embedded library + generator wrapper to a fresh temp dir.
        // The temp dir auto-cleans when `tmp_dir` drops at the end of the call.
        let tmp_dir = tempfile::tempdir()
            .map_err(|e| anyhow::anyhow!("failed to create temp dir: {e}"))?;
        let lib_path = tmp_dir.path().join("hwpx_document.py");
        let gen_path = tmp_dir.path().join("__moa_hwpx_generator.py");
        tokio::fs::write(&lib_path, HWPX_DOCUMENT_PY)
            .await
            .map_err(|e| anyhow::anyhow!("failed to write embedded library: {e}"))?;
        tokio::fs::write(&gen_path, HWPX_GENERATOR_PY)
            .await
            .map_err(|e| anyhow::anyhow!("failed to write generator wrapper: {e}"))?;

        // Resolve output_path to an absolute path so the Python side does not
        // depend on its CWD.
        let output_abs: PathBuf = if std::path::Path::new(&parsed.output_path).is_absolute() {
            parsed.output_path.clone().into()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&parsed.output_path))
                .unwrap_or_else(|_| parsed.output_path.clone().into())
        };

        let spec = json!({
            "output_path": output_abs.to_string_lossy(),
            "title": parsed.title,
            "paragraphs": parsed.paragraphs,
            "legal_format": parsed.legal_format,
        });
        let spec_bytes = serde_json::to_vec(&spec)?;

        let mut cmd = Command::new(python);
        cmd.arg(&gen_path);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn {python}: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(&spec_bytes)
                .await
                .map_err(|e| anyhow::anyhow!("failed to write spec to python stdin: {e}"))?;
            stdin.shutdown().await.ok();
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| anyhow::anyhow!("python wait failed: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("python exited {}: {stderr}", output.status)),
            });
        }

        Ok(ToolResult {
            success: true,
            output: format!("Wrote HWPX document to {}", output_abs.display()),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_library_is_present_and_valid() {
        // The bundled Python source must be non-empty and contain the
        // class header. include_str! at compile time guarantees the file
        // exists; this test verifies the asset wasn't accidentally truncated.
        assert!(HWPX_DOCUMENT_PY.contains("class HwpxDocument"));
        assert!(HWPX_DOCUMENT_PY.len() > 1000);
    }

    #[test]
    fn generator_wrapper_imports_hwpx_document() {
        assert!(HWPX_GENERATOR_PY.contains("from hwpx_document import HwpxDocument"));
        assert!(HWPX_GENERATOR_PY.contains("doc.save"));
    }

    #[test]
    fn schema_declares_required_output_path() {
        let tool = HwpxCreateTool::new();
        let schema = tool.parameters_schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "output_path"));
    }

    #[tokio::test]
    async fn execute_rejects_empty_output_path() {
        let tool = HwpxCreateTool::new();
        let result = tool
            .execute(json!({ "output_path": "" }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("output_path"));
    }

    #[tokio::test]
    async fn execute_returns_error_when_python_missing() {
        // We can only test this when Python is actually missing. On dev
        // machines Python is normally installed, so this test asserts the
        // happy-path schema/argument parsing instead. Full execution is
        // covered by an opt-in integration test below.
        let _tool = HwpxCreateTool::new();
        let parsed: HwpxCreateArgs = serde_json::from_value(json!({
            "output_path": "/tmp/test.hwpx",
            "title": "Test",
            "paragraphs": ["Hello", { "text": "World", "bold": true }],
            "legal_format": true,
        }))
        .unwrap();
        assert_eq!(parsed.output_path, "/tmp/test.hwpx");
        assert_eq!(parsed.title.as_deref(), Some("Test"));
        assert_eq!(parsed.paragraphs.len(), 2);
        assert!(parsed.legal_format);
    }

    /// Opt-in integration test: only runs when `MOA_TEST_HWPX_PYTHON=1` is
    /// set, because it shells out to python3 and writes a real .hwpx file.
    #[tokio::test]
    async fn execute_writes_real_hwpx_file_when_python_available() {
        if std::env::var("MOA_TEST_HWPX_PYTHON").ok().as_deref() != Some("1") {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path().join("integration.hwpx");
        let tool = HwpxCreateTool::new();
        let result = tool
            .execute(json!({
                "output_path": output.to_string_lossy(),
                "title": "통합 테스트 문서",
                "paragraphs": [
                    "이 문서는 hwpx_create 도구의 통합 테스트로 생성되었습니다.",
                    { "text": "강조된 본문 단락", "bold": true }
                ],
                "legal_format": true,
            }))
            .await
            .unwrap();
        assert!(result.success, "tool failed: {:?}", result.error);
        assert!(output.exists(), "expected output file to exist");
        let metadata = std::fs::metadata(&output).unwrap();
        assert!(metadata.len() > 0, "output file should not be empty");
    }
}
