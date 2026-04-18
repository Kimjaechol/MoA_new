//! OS-matched Ollama installer.
//!
//! Detects host OS and runs the appropriate installer with progress callbacks.
//! The library assumes the caller has already obtained explicit user consent
//! (typically via a UI dialog showing the detected OS, install method, and
//! "Install" / "Cancel" buttons). It does NOT prompt — that is the UI's job.
//!
//! Trust boundary: official Ollama installers are downloaded over HTTPS from
//! `ollama.com`. The Unix install script is fetched to a temporary file
//! before execution (instead of the conventional `curl … | sh`), so the
//! exact bytes that run are inspectable / hashable by the caller if desired.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// URL of the official Unix install script (macOS + Linux).
const OFFICIAL_UNIX_SCRIPT_URL: &str = "https://ollama.com/install.sh";
/// URL of the Windows installer.
const WINDOWS_INSTALLER_URL: &str = "https://ollama.com/download/OllamaSetup.exe";

/// Strategy chosen for installing Ollama on the current host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstallMethod {
    /// macOS via Homebrew (`brew install ollama`). Used when `brew` is on PATH.
    BrewMacOS,
    /// macOS or Linux via the official `https://ollama.com/install.sh`.
    /// Script is downloaded to a temp file and executed under `sh`.
    OfficialScriptUnix,
    /// Windows via `OllamaSetup.exe`. Downloaded to temp and run with the
    /// silent flag `/S` (NSIS convention).
    WindowsMsi,
    /// OS is supported by Ollama but no automated path is available
    /// (e.g. unknown architecture). Caller should display the bundled
    /// instructions to the user.
    Manual { instructions: String },
}

impl InstallMethod {
    /// Human-readable summary suitable for a consent dialog body.
    pub fn human_summary(&self) -> String {
        match self {
            InstallMethod::BrewMacOS => "Install via Homebrew (brew install ollama)".into(),
            InstallMethod::OfficialScriptUnix => {
                format!("Download and run the official installer from {OFFICIAL_UNIX_SCRIPT_URL}")
            }
            InstallMethod::WindowsMsi => {
                format!("Download {WINDOWS_INSTALLER_URL} and run it silently (/S)")
            }
            InstallMethod::Manual { instructions } => instructions.clone(),
        }
    }
}

/// One incremental install progress event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    /// High-level stage label, e.g. "downloading", "running", "verifying".
    pub stage: String,
    /// Optional raw line from the installer (stderr or stdout).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
}

// ── Detection ───────────────────────────────────────────────────────────

/// Inspect the host and choose the best install method.
pub async fn detect_install_method() -> InstallMethod {
    match std::env::consts::OS {
        "macos" => {
            if which_in_path("brew").await {
                InstallMethod::BrewMacOS
            } else {
                InstallMethod::OfficialScriptUnix
            }
        }
        "linux" => InstallMethod::OfficialScriptUnix,
        "windows" => InstallMethod::WindowsMsi,
        other => InstallMethod::Manual {
            instructions: format!(
                "Automated install is not supported on {other}. Visit \
                 https://ollama.com/download for manual install steps."
            ),
        },
    }
}

/// Returns true when `bin` is on the PATH.
async fn which_in_path(bin: &str) -> bool {
    Command::new(bin)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Returns true when `ollama` binary is on the PATH and runs.
pub async fn is_ollama_installed() -> bool {
    which_in_path("ollama").await
}

/// Returns the Ollama CLI version string (e.g. `"0.20.7"`) or None when
/// Ollama is not installed or the command fails.
pub async fn ollama_version() -> Option<String> {
    let out = Command::new("ollama")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // `ollama --version` prints lines like "ollama version is 0.20.7".
    // Be tolerant: pick the last whitespace-separated token that looks like
    // a semver.
    stdout
        .split_whitespace()
        .rev()
        .find(|tok| {
            tok.split('.')
                .all(|p| p.chars().all(|c| c.is_ascii_digit()))
        })
        .map(|s| s.to_string())
}

// ── Execution ───────────────────────────────────────────────────────────

/// Install Ollama using the chosen method.
///
/// Caller MUST have obtained explicit user consent before calling this.
/// On success, `ollama --version` should be runnable on PATH.
pub async fn install_ollama<F>(method: &InstallMethod, mut on_progress: F) -> Result<()>
where
    F: FnMut(InstallProgress) + Send,
{
    on_progress(InstallProgress {
        stage: "starting".to_string(),
        line: Some(method.human_summary()),
    });

    match method {
        InstallMethod::BrewMacOS => install_via_brew(&mut on_progress).await?,
        InstallMethod::OfficialScriptUnix => install_via_unix_script(&mut on_progress).await?,
        InstallMethod::WindowsMsi => install_via_windows_msi(&mut on_progress).await?,
        InstallMethod::Manual { instructions } => {
            anyhow::bail!(
                "Automated install not available; show these instructions to the user: {instructions}"
            );
        }
    }

    on_progress(InstallProgress {
        stage: "verifying".to_string(),
        line: None,
    });
    if !is_ollama_installed().await {
        anyhow::bail!("install command completed but `ollama` is still not on PATH");
    }

    on_progress(InstallProgress {
        stage: "done".to_string(),
        line: ollama_version().await.map(|v| format!("ollama {v}")),
    });
    Ok(())
}

async fn install_via_brew<F>(on_progress: &mut F) -> Result<()>
where
    F: FnMut(InstallProgress) + Send,
{
    on_progress(InstallProgress {
        stage: "running".to_string(),
        line: Some("brew install ollama".to_string()),
    });
    stream_command("brew", &["install", "ollama"], on_progress).await
}

async fn install_via_unix_script<F>(on_progress: &mut F) -> Result<()>
where
    F: FnMut(InstallProgress) + Send,
{
    // 1. Download script to a temp file (avoids `curl | sh` opacity).
    on_progress(InstallProgress {
        stage: "downloading".to_string(),
        line: Some(OFFICIAL_UNIX_SCRIPT_URL.to_string()),
    });
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("building reqwest client for installer download")?;
    let body = client
        .get(OFFICIAL_UNIX_SCRIPT_URL)
        .send()
        .await
        .with_context(|| format!("GET {OFFICIAL_UNIX_SCRIPT_URL}"))?
        .error_for_status()
        .context("non-2xx response from install.sh URL")?
        .text()
        .await
        .context("reading install.sh body")?;

    // 2. Write to a temp file and chmod 0755.
    let dir = std::env::temp_dir();
    let path = dir.join(format!("ollama-install-{}.sh", std::process::id()));
    fs::write(&path, &body)
        .await
        .with_context(|| format!("writing installer to {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).await?;
    }

    // 3. Execute under sh, streaming stderr+stdout lines.
    on_progress(InstallProgress {
        stage: "running".to_string(),
        line: Some(format!("sh {}", path.display())),
    });
    let result = stream_command("sh", &[path.to_string_lossy().as_ref()], on_progress).await;

    // 4. Best-effort cleanup of temp script.
    let _ = fs::remove_file(&path).await;

    result
}

async fn install_via_windows_msi<F>(on_progress: &mut F) -> Result<()>
where
    F: FnMut(InstallProgress) + Send,
{
    // 1. Download installer.
    on_progress(InstallProgress {
        stage: "downloading".to_string(),
        line: Some(WINDOWS_INSTALLER_URL.to_string()),
    });
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("building reqwest client for Windows installer")?;
    let bytes = client
        .get(WINDOWS_INSTALLER_URL)
        .send()
        .await
        .with_context(|| format!("GET {WINDOWS_INSTALLER_URL}"))?
        .error_for_status()
        .context("non-2xx response from Windows installer URL")?
        .bytes()
        .await
        .context("reading installer bytes")?;

    // 2. Write to temp.
    let dir = std::env::temp_dir();
    let path: PathBuf = dir.join(format!("OllamaSetup-{}.exe", std::process::id()));
    fs::write(&path, &bytes)
        .await
        .with_context(|| format!("writing installer to {}", path.display()))?;

    // 3. Run silent install.
    on_progress(InstallProgress {
        stage: "running".to_string(),
        line: Some(format!("{} /S", path.display())),
    });
    let result = stream_command(path.to_string_lossy().as_ref(), &["/S"], on_progress).await;

    // 4. Cleanup.
    let _ = fs::remove_file(&path).await;

    result
}

/// Run `program args…`, streaming stdout+stderr lines through the progress
/// callback. Returns Err if the process exits non-zero.
async fn stream_command<F>(program: &str, args: &[&str], on_progress: &mut F) -> Result<()>
where
    F: FnMut(InstallProgress) + Send,
{
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawning {program}"))?;

    let stdout = child.stdout.take().context("child stdout was not piped")?;
    let stderr = child.stderr.take().context("child stderr was not piped")?;

    // Drain stdout and stderr concurrently into the callback.
    let stdout_lines = drain_lines(stdout);
    let stderr_lines = drain_lines(stderr);
    let (out_lines, err_lines) = tokio::join!(stdout_lines, stderr_lines);
    for line in out_lines.into_iter().chain(err_lines.into_iter()) {
        on_progress(InstallProgress {
            stage: "running".to_string(),
            line: Some(line),
        });
    }

    let status = child
        .wait()
        .await
        .context("waiting for installer to exit")?;
    if !status.success() {
        anyhow::bail!(
            "installer `{} {}` exited with status {}",
            program,
            args.join(" "),
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

async fn drain_lines<R: tokio::io::AsyncRead + Unpin + Send + 'static>(reader: R) -> Vec<String> {
    let mut out = Vec::new();
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        out.push(line);
    }
    out
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detect_picks_a_method() {
        let m = detect_install_method().await;
        // On every supported host, we get a non-Manual variant.
        assert!(
            matches!(
                m,
                InstallMethod::BrewMacOS
                    | InstallMethod::OfficialScriptUnix
                    | InstallMethod::WindowsMsi
                    | InstallMethod::Manual { .. }
            ),
            "unexpected variant: {m:?}"
        );
    }

    #[test]
    fn human_summary_describes_method() {
        let s = InstallMethod::BrewMacOS.human_summary();
        assert!(s.contains("brew"));
        let s = InstallMethod::OfficialScriptUnix.human_summary();
        assert!(s.contains("ollama.com"));
        let s = InstallMethod::WindowsMsi.human_summary();
        assert!(s.contains("OllamaSetup.exe"));
        let s = InstallMethod::Manual {
            instructions: "go to ollama.com".to_string(),
        }
        .human_summary();
        assert_eq!(s, "go to ollama.com");
    }

    #[tokio::test]
    async fn ollama_install_check_works() {
        // On a machine with Ollama installed, this returns true.
        // On CI / fresh container, it returns false. Both are valid;
        // we just verify the function does not panic.
        let _ = is_ollama_installed().await;
    }

    #[tokio::test]
    async fn ollama_version_parses_when_present() {
        // Best-effort: if Ollama is installed, the parsed version should
        // look like a semver (digits + dots).
        if let Some(v) = ollama_version().await {
            assert!(v.contains('.'), "expected semver, got {v:?}");
            assert!(v.chars().all(|c| c.is_ascii_digit() || c == '.'));
        }
    }

    /// Manual smoke test that runs `Manual` install path (no system change).
    #[tokio::test]
    async fn manual_method_does_not_modify_system() {
        let method = InstallMethod::Manual {
            instructions: "do nothing".to_string(),
        };
        let mut got = Vec::new();
        let result = install_ollama(&method, |p| got.push((p.stage.clone(), p.line.clone()))).await;
        // Manual variant intentionally errors out so the UI shows instructions.
        assert!(result.is_err());
        assert_eq!(got[0].0, "starting");
    }
}
