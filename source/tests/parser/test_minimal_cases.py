#!/usr/bin/env python3
"""
Test minimal reproduction cases for parser issues
"""

import subprocess
import sys
from pathlib import Path

def test_file(file_path):
    """Test a single .rsc file"""
    print(f"\nTesting: {file_path.name}")
    print('-' * 60)

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
            print("[FAIL] Parse error:")
            for line in output.split('\n'):
                if "Parse error" in line:
                    print(f"  {line}")
            return False
        elif "Check passed" in output:
            print("[PASS] Parsed and validated successfully")
            return True
        elif "Check failed" in output:
            print("[PASS] Parsed successfully (semantic errors)")
            return True
        else:
            print("[UNKNOWN]")
            return False

    except Exception as e:
        print(f"[ERROR] {e}")
        return False

def main():
    tests = [
        "multiline_conditions.rsc",
        "match_arm_reference.rsc",
        "multiline_function_params.rsc",
        "multiline_match_patterns.rsc",
    ]

    parser_dir = Path(__file__).parent
    results = {}

    for test_name in tests:
        test_file_path = parser_dir / test_name
        if test_file_path.exists():
            results[test_name] = test_file(test_file_path)
        else:
            print(f"\n[SKIP] {test_name} not found")
            results[test_name] = None

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print('='*60)

    passed = sum(1 for v in results.values() if v is True)
    failed = sum(1 for v in results.values() if v is False)
    skipped = sum(1 for v in results.values() if v is None)

    print(f"Passed: {passed}/{len(tests)}")
    print(f"Failed: {failed}/{len(tests)}")
    if skipped:
        print(f"Skipped: {skipped}/{len(tests)}")

    if failed > 0:
        print("\nFailed tests:")
        for name, result in results.items():
            if result is False:
                print(f"  - {name}")

    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
