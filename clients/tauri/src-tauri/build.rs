fn main() {
    // Locate the zeroclaw binary from the workspace cargo build output and copy
    // it to the sidecar binaries directory with the correct target-triple suffix
    // that Tauri expects for bundling.
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let target_triple = std::env::var("TARGET").unwrap_or_default();

    if !target_triple.is_empty() {
        let ext = if target_triple.contains("windows") {
            ".exe"
        } else {
            ""
        };
        let sidecar_name = format!("zeroclaw-{target_triple}{ext}");
        let binaries_dir = std::path::Path::new("binaries");

        // Pick the right workspace output directory based on build profile
        let profile_dir = if profile == "release" {
            "release"
        } else {
            "debug"
        };
        let workspace_binary = std::path::Path::new("../../../target")
            .join(profile_dir)
            .join(format!("zeroclaw{ext}"));

        if workspace_binary.exists() {
            // Ensure binaries/ directory exists
            if !binaries_dir.exists() {
                let _ = std::fs::create_dir_all(binaries_dir);
            }

            let dest = binaries_dir.join(&sidecar_name);
            let should_copy = if !dest.exists() {
                true
            } else {
                // Copy only if source is newer than destination
                std::fs::metadata(&workspace_binary)
                    .and_then(|m| m.modified())
                    .ok()
                    > std::fs::metadata(&dest)
                        .and_then(|m| m.modified())
                        .ok()
            };

            if should_copy {
                if let Err(e) = std::fs::copy(&workspace_binary, &dest) {
                    println!(
                        "cargo:warning=Failed to copy zeroclaw binary to sidecar dir: {e}"
                    );
                } else {
                    println!(
                        "cargo:warning=Copied zeroclaw binary to {dest}",
                        dest = dest.display()
                    );
                }
            }
        } else {
            println!(
                "cargo:warning=ZeroClaw binary not found at {path}. Run 'cargo build --release' from the workspace root first.",
                path = workspace_binary.display()
            );
        }
    }

    tauri_build::build()
}
