#!/usr/bin/env python3
"""Unpack an HWPX file (ZIP archive) into a directory for inspection/editing."""
import os
import sys
import zipfile


def unpack(hwpx_path, output_dir):
    if not os.path.isfile(hwpx_path):
        print(f"Error: file not found: {hwpx_path}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(output_dir, exist_ok=True)

    with zipfile.ZipFile(hwpx_path, "r") as zf:
        zf.extractall(output_dir)

    print(f"Unpacked {hwpx_path} → {output_dir}/")
    print("Key files:")
    for root, dirs, files in os.walk(output_dir):
        for f in sorted(files):
            rel = os.path.relpath(os.path.join(root, f), output_dir)
            print(f"  {rel}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <file.hwpx> <output_dir>")
        sys.exit(1)
    unpack(sys.argv[1], sys.argv[2])
