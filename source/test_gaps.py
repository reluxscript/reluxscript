#!/usr/bin/env python3
"""Test all gap files to identify parser issues"""

import subprocess
import os
from pathlib import Path

def test_file(file_path):
    """Test a single .rsc file and return (passed, error_msg)"""
    try:
        result = subprocess.run(
            ["cargo", "run", "--no-default-features", "--bin", "reluxscript", "--", "parse", file_path],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Check if there's a parse error in the output
        if "Parse error" in result.stderr or "Parse error" in result.stdout:
            # Extract the parse error line
            for line in result.stderr.split('\n') + result.stdout.split('\n'):
                if "Parse error" in line:
                    return False, line.strip()
            return False, "Parse error (unknown)"

        # Check if the process failed
        if result.returncode != 0:
            # Look for compilation errors
            if "error:" in result.stderr:
                for line in result.stderr.split('\n'):
                    if "error:" in line and "Compiling" not in line:
                        return False, line.strip()
            return False, f"Exit code: {result.returncode}"

        return True, None

    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, f"Error: {str(e)}"

def main():
    # Already in reluxscript directory

    # Find all .rsc files in tests/gaps
    gap_files = sorted(Path("tests/gaps").glob("*.rsc"))

    print("Testing parser on gap files...")
    print("=" * 80)
    print()

    passed = []
    failed = []

    for i, file_path in enumerate(gap_files, 1):
        print(f"[{i}/{len(gap_files)}] Testing: {file_path} ... ", end="", flush=True)

        success, error = test_file(str(file_path))

        if success:
            print("PASSED")
            passed.append(str(file_path))
        else:
            print("FAILED")
            print(f"    {error}")
            failed.append((str(file_path), error))

    print()
    print("=" * 80)
    print(f"Results: {len(passed)} passed, {len(failed)} failed out of {len(gap_files)} total")
    print(f"Success rate: {len(passed) * 100 // len(gap_files)}%")
    print()

    if failed:
        print("Failed files:")
        for file_path, error in failed:
            file_name = Path(file_path).name
            print(f"  - {file_name}: {error}")

if __name__ == "__main__":
    main()
