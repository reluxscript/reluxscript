#!/usr/bin/env python3
"""
Test script for analyzer .rsc files
Checks if all analyzer files parse successfully
"""

import subprocess
import sys
from pathlib import Path

def test_file(file_path):
    """Test a single .rsc file"""
    try:
        result = subprocess.run(
            ["cargo", "run", "--", "check", str(file_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        output = result.stdout + result.stderr

        # Check for parse errors
        if "Parse error" in output:
            return False, "Parse error"

        # Check for check passed/failed
        if "Check passed" in output:
            return True, "Passed"
        elif "Check failed" in output:
            # Count errors
            error_lines = [line for line in output.split('\n') if line.startswith('error[')]
            error_count = len(error_lines)
            return False, f"Failed with {error_count} error(s)"
        else:
            return False, "Unknown result"

    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, f"Exception: {e}"

def main():
    # Find all analyzer .rsc files
    analyzers_dir = Path("tests/codegen/analyzers")

    if not analyzers_dir.exists():
        print(f"Error: Directory {analyzers_dir} does not exist")
        sys.exit(1)

    rsc_files = sorted(analyzers_dir.glob("*.rsc"))

    if not rsc_files:
        print(f"No .rsc files found in {analyzers_dir}")
        sys.exit(1)

    print(f"Testing {len(rsc_files)} analyzer files...\n")

    results = []
    passed = 0
    failed = 0

    for file_path in rsc_files:
        file_name = file_path.name
        print(f"Testing {file_name}...", end=" ", flush=True)

        success, message = test_file(file_path)
        results.append((file_name, success, message))

        if success:
            print(f"PASSED")
            passed += 1
        else:
            print(f"FAILED ({message})")
            failed += 1

    print(f"\n{'='*60}")
    print(f"Results: {passed} passed, {failed} failed out of {len(rsc_files)} total")

    if failed > 0:
        print(f"\nFailed files:")
        for name, success, message in results:
            if not success:
                print(f"  - {name}: {message}")

    if failed == 0:
        print("\nAll analyzer tests passed!")
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
