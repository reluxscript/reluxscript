#!/usr/bin/env python3
"""
Test all analyzer .rsc files to verify they parse correctly
"""

import subprocess
import sys
from pathlib import Path

def test_file(file_path):
    """Test a single .rsc file"""
    print(f"\n{'='*60}")
    print(f"Testing: {file_path.name}")
    print('='*60)

    try:
        result = subprocess.run(
            ["cargo", "run", "--", "check", str(file_path)],
            cwd=file_path.parent.parent.parent,
            capture_output=True,
            text=True,
            timeout=30
        )

        output = result.stdout + result.stderr

        # Check for parse errors
        if "Parse error" in output:
            print(f"[FAIL] PARSE ERROR")
            # Print the parse error line
            for line in output.split('\n'):
                if "Parse error" in line:
                    print(f"   {line}")
            return False

        # Check for check passed/failed
        if "Check passed" in output:
            print(f"[PASS] Parsed and validated successfully")
            return True
        elif "Check failed" in output:
            print(f"[PASS] Parsed successfully (has semantic errors)")
            # Count errors
            error_count = output.count("error[")
            print(f"   {error_count} semantic error(s)")
            return True  # Still counts as success since it parsed
        else:
            print(f"[UNKNOWN] Unexpected result")
            print(output[-200:])  # Print last 200 chars
            return False

    except subprocess.TimeoutExpired:
        print(f"[FAIL] TIMEOUT")
        return False
    except Exception as e:
        print(f"[FAIL] ERROR: {e}")
        return False

def main():
    # Find all .rsc files in analyzers directory
    analyzers_dir = Path(__file__).parent / "analyzers"

    if not analyzers_dir.exists():
        print(f"Error: Directory not found: {analyzers_dir}")
        sys.exit(1)

    rsc_files = sorted(analyzers_dir.glob("*.rsc"))

    if not rsc_files:
        print(f"No .rsc files found in {analyzers_dir}")
        sys.exit(1)

    print(f"Found {len(rsc_files)} analyzer files to test")

    results = {}
    for rsc_file in rsc_files:
        results[rsc_file.name] = test_file(rsc_file)

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print('='*60)

    passed = sum(1 for v in results.values() if v)
    failed = sum(1 for v in results.values() if not v)

    print(f"Passed: {passed}/{len(results)}")
    print(f"Failed: {failed}/{len(results)}")

    if failed > 0:
        print("\nFailed files:")
        for name, success in results.items():
            if not success:
                print(f"  - {name}")

    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
