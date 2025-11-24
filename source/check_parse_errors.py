#!/usr/bin/env python3
"""
Check parse errors in analyzer files and categorize them
"""

import subprocess
import sys
from pathlib import Path
import re

def get_parse_error(file_path):
    """Get the first parse error from a file"""
    try:
        result = subprocess.run(
            ["cargo", "run", "--", "check", str(file_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        output = result.stdout + result.stderr

        # Find parse error line
        for line in output.split('\n'):
            if "Parse error at" in line:
                return line.strip()

        return None

    except Exception as e:
        return f"Exception: {e}"

def extract_error_location(error_line):
    """Extract line:col from parse error"""
    match = re.search(r'at (\d+):(\d+)', error_line)
    if match:
        return int(match.group(1)), int(match.group(2))
    return None, None

def get_code_snippet(file_path, line_num, col_num):
    """Get code snippet around error"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()

        if line_num <= 0 or line_num > len(lines):
            return None

        # Get context lines
        start = max(0, line_num - 2)
        end = min(len(lines), line_num + 1)

        snippet = []
        for i in range(start, end):
            prefix = ">>> " if i == line_num - 1 else "    "
            snippet.append(f"{prefix}{i+1:4d}: {lines[i].rstrip()}")

        # Add pointer to error column
        if col_num:
            pointer_line = " " * (col_num + 10) + "^"
            snippet.append(pointer_line)

        return '\n'.join(snippet)

    except Exception:
        return None

def main():
    analyzers_dir = Path("tests/codegen/analyzers")

    if not analyzers_dir.exists():
        print(f"Error: Directory {analyzers_dir} does not exist")
        sys.exit(1)

    rsc_files = sorted(analyzers_dir.glob("*.rsc"))

    print(f"Checking {len(rsc_files)} analyzer files for parse errors...\n")
    print("=" * 80)

    errors_found = {}

    for file_path in rsc_files:
        error = get_parse_error(file_path)

        if error:
            line_num, col_num = extract_error_location(error)
            snippet = get_code_snippet(file_path, line_num, col_num) if line_num else None

            errors_found[file_path.name] = {
                'error': error,
                'snippet': snippet
            }

    if not errors_found:
        print("\nNo parse errors found! All files parse successfully.")
        sys.exit(0)

    print(f"\nFound parse errors in {len(errors_found)} files:\n")

    for filename, info in sorted(errors_found.items()):
        print(f"\n{'='*80}")
        print(f"File: {filename}")
        print(f"Error: {info['error']}")

        if info['snippet']:
            print("\nCode snippet:")
            print(info['snippet'])

    print(f"\n{'='*80}")
    print(f"\nSummary: {len(errors_found)} files with parse errors")

if __name__ == "__main__":
    main()
