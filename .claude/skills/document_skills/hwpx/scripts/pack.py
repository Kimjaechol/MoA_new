#!/usr/bin/env python3
"""Pack a directory into an HWPX file (ZIP archive)."""
import os
import sys
import zipfile


def pack(input_dir, output_path):
    if not os.path.isdir(input_dir):
        print(f"Error: directory not found: {input_dir}", file=sys.stderr)
        sys.exit(1)

    with zipfile.ZipFile(output_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, dirs, files in os.walk(input_dir):
            for f in sorted(files):
                filepath = os.path.join(root, f)
                arcname = os.path.relpath(filepath, input_dir)
                zf.write(filepath, arcname)

    print(f"Packed {input_dir}/ → {output_path}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input_dir> <output.hwpx>")
        sys.exit(1)
    pack(sys.argv[1], sys.argv[2])
