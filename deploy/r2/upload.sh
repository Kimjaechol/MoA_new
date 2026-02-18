#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# upload.sh — Upload ZeroClaw release binaries to Cloudflare R2
# ─────────────────────────────────────────────────────────────
#
# Cloudflare R2 is S3-compatible, so we use the AWS CLI with a
# custom endpoint to upload release artifacts.
#
# Required environment variables:
#   AWS_ACCESS_KEY_ID      — R2 API token access key
#   AWS_SECRET_ACCESS_KEY  — R2 API token secret key
#   R2_ENDPOINT            — R2 S3-compatible endpoint URL
#                            (e.g., https://<account-id>.r2.cloudflarestorage.com)
#   R2_BUCKET              — R2 bucket name (e.g., zeroclaw-releases)
#
# Optional:
#   VERSION                — Semantic version tag (e.g., v0.1.0).
#                            If unset, only uploads to /releases/latest/.
#   RELEASE_DIR            — Local directory containing release artifacts.
#                            Defaults to ./release/
#
# Usage:
#   export AWS_ACCESS_KEY_ID=...
#   export AWS_SECRET_ACCESS_KEY=...
#   export R2_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
#   export R2_BUCKET=zeroclaw-releases
#   export VERSION=v0.1.0
#   ./deploy/r2/upload.sh
#
# Directory structure created in R2:
#   /releases/latest/          — Always points to the most recent upload
#   /releases/v0.1.0/          — Versioned archive (when VERSION is set)

set -euo pipefail

# ── Validate required environment variables ───────────────────
missing_vars=()
for var in AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY R2_ENDPOINT R2_BUCKET; do
    if [ -z "${!var:-}" ]; then
        missing_vars+=("$var")
    fi
done

if [ "${#missing_vars[@]}" -gt 0 ]; then
    echo "ERROR: Missing required environment variables: ${missing_vars[*]}" >&2
    echo "See the script header for documentation." >&2
    exit 1
fi

# ── Check for AWS CLI ─────────────────────────────────────────
if ! command -v aws &>/dev/null; then
    echo "ERROR: AWS CLI is not installed." >&2
    echo "Install it: https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html" >&2
    exit 1
fi

# ── Configuration ─────────────────────────────────────────────
RELEASE_DIR="${RELEASE_DIR:-./release}"
VERSION="${VERSION:-}"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "ERROR: Release directory does not exist: $RELEASE_DIR" >&2
    echo "Build release artifacts first, then re-run this script." >&2
    exit 1
fi

# Count files to upload.
file_count=$(find "$RELEASE_DIR" -type f | wc -l)
if [ "$file_count" -eq 0 ]; then
    echo "ERROR: No files found in $RELEASE_DIR" >&2
    exit 1
fi

echo "Found $file_count file(s) in $RELEASE_DIR"

# ── Helper: determine Content-Type from file extension ────────
get_content_type() {
    local filename="$1"
    case "$filename" in
        *.tar.gz) echo "application/gzip" ;;
        *.zip)    echo "application/zip" ;;
        *.exe)    echo "application/vnd.microsoft.portable-executable" ;;
        *.dmg)    echo "application/x-apple-diskimage" ;;
        *.deb)    echo "application/vnd.debian.binary-package" ;;
        *.rpm)    echo "application/x-rpm" ;;
        *.sig)    echo "application/pgp-signature" ;;
        *.pem)    echo "application/x-pem-file" ;;
        *.sha256 | *SHA256SUMS) echo "text/plain" ;;
        *)        echo "application/octet-stream" ;;
    esac
}

# ── Helper: upload a single file to an R2 path ───────────────
upload_file() {
    local local_path="$1"
    local r2_key="$2"
    local content_type
    content_type=$(get_content_type "$local_path")

    echo "  Uploading: $local_path -> s3://$R2_BUCKET/$r2_key ($content_type)"
    aws s3 cp "$local_path" "s3://$R2_BUCKET/$r2_key" \
        --endpoint-url "$R2_ENDPOINT" \
        --content-type "$content_type" \
        --no-progress
}

# ── Upload to /releases/latest/ ──────────────────────────────
echo ""
echo "=== Uploading to /releases/latest/ ==="
while IFS= read -r -d '' file; do
    basename=$(basename "$file")
    upload_file "$file" "releases/latest/$basename"
done < <(find "$RELEASE_DIR" -type f -print0)

# ── Upload to /releases/v{version}/ (if VERSION is set) ──────
if [ -n "$VERSION" ]; then
    # Strip leading 'v' if present for consistent directory naming,
    # then re-add it for the path prefix.
    version_tag="${VERSION#v}"
    echo ""
    echo "=== Uploading to /releases/v${version_tag}/ ==="
    while IFS= read -r -d '' file; do
        basename=$(basename "$file")
        upload_file "$file" "releases/v${version_tag}/$basename"
    done < <(find "$RELEASE_DIR" -type f -print0)
fi

# ── Summary ───────────────────────────────────────────────────
echo ""
echo "Upload complete."
echo "  Bucket:  $R2_BUCKET"
echo "  Latest:  releases/latest/"
if [ -n "$VERSION" ]; then
    echo "  Version: releases/v${version_tag}/"
fi
echo ""
echo "To verify, list the bucket contents:"
echo "  aws s3 ls s3://$R2_BUCKET/releases/ --endpoint-url $R2_ENDPOINT --recursive"
