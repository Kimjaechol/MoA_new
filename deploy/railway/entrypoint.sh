#!/bin/sh
# ZeroClaw Railway Entrypoint
#
# Fixes volume mount permissions before dropping to non-root user.
# Railway mounts volumes as root:root, but ZeroClaw runs as non-root
# user 'zeroclaw'. This script runs as root to fix ownership, then
# exec's the main binary as the zeroclaw user via gosu.

set -e

ZEROCLAW_HOME="/app"
ZEROCLAW_DIR="${ZEROCLAW_HOME}/.zeroclaw"
WORKSPACE_DIR="${ZEROCLAW_DIR}/workspace"

# Fix ownership on volume-mounted directories if running as root.
if [ "$(id -u)" = "0" ]; then
    # Ensure directories exist inside the volume mount.
    mkdir -p "${ZEROCLAW_DIR}" "${WORKSPACE_DIR}"

    # Fix ownership so the zeroclaw user can write.
    chown -R zeroclaw:zeroclaw "${ZEROCLAW_DIR}"

    # Seed default config if none exists (first deploy with fresh volume).
    if [ ! -f "${ZEROCLAW_DIR}/config.toml" ]; then
        printf 'default_temperature = 0.7\n\n[gateway]\nallow_public_bind = true\n' \
            > "${ZEROCLAW_DIR}/config.toml"
        chown zeroclaw:zeroclaw "${ZEROCLAW_DIR}/config.toml"
        chmod 600 "${ZEROCLAW_DIR}/config.toml"
    fi

    # Drop to zeroclaw user and exec the command.
    exec gosu zeroclaw "$@"
fi

# If already running as non-root (shouldn't happen with this Dockerfile),
# just exec the command directly.
exec "$@"
