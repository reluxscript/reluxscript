#!/usr/bin/env python3
"""
Test codegen by comparing generated output with manually-written expected output.
"""
import subprocess
from pathlib import Path
import difflib
import sys

def normalize_output(content):
    """Normalize output for comparison (ignore whitespace differences)"""
    lines = content.strip().splitlines()
    # Strip leading/trailing whitespace from each line and filter empty lines
    normalized_lines = [line.strip() for line in lines if line.strip()]
    return '\n'.join(normalized_lines)

def test_codegen(test_name):
    """Test a single codegen case"""
    source = Path(f"tests/codegen/{test_name}.rsc")
    expected_babel = Path(f"tests/codegen/expected/{test_name}.babel.js")
    expected_swc = Path(f"tests/codegen/expected/{test_name}.swc.rs")

    if not source.exists():
        return False, f"Source file not found: {source}"
    if not expected_babel.exists():
        return False, f"Expected Babel output not found: {expected_babel}"
    if not expected_swc.exists():
        return False, f"Expected SWC output not found: {expected_swc}"

    # Build the test
    temp_dir = Path("dist_temp")
    temp_dir.mkdir(exist_ok=True)

    try:
        result = subprocess.run(
            ["cargo", "run", "--bin", "reluxscript", "--",
             "build", str(source), "--output", str(temp_dir)],
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            return False, f"Build failed:\n{result.stderr}"

        # Read generated outputs
        babel_out = temp_dir / "index.js"
        swc_out = temp_dir / "lib.rs"

        if not babel_out.exists() or not swc_out.exists():
            return False, "Output files not generated"

        actual_babel = normalize_output(babel_out.read_text())
        actual_swc = normalize_output(swc_out.read_text())

        expected_babel_content = normalize_output(expected_babel.read_text())
        expected_swc_content = normalize_output(expected_swc.read_text())

        # Compare
        babel_match = actual_babel == expected_babel_content
        swc_match = actual_swc == expected_swc_content

        if not babel_match or not swc_match:
            errors = []
            if not babel_match:
                diff = list(difflib.unified_diff(
                    expected_babel_content.splitlines(keepends=True),
                    actual_babel.splitlines(keepends=True),
                    fromfile="expected (Babel)",
                    tofile="actual (Babel)",
                    lineterm=""
                ))
                errors.append("Babel output mismatch:\n" + "".join(diff))

            if not swc_match:
                diff = list(difflib.unified_diff(
                    expected_swc_content.splitlines(keepends=True),
                    actual_swc.splitlines(keepends=True),
                    fromfile="expected (SWC)",
                    tofile="actual (SWC)",
                    lineterm=""
                ))
                errors.append("SWC output mismatch:\n" + "".join(diff))

            return False, "\n\n".join(errors)

        return True, None

    except Exception as e:
        return False, f"Error: {str(e)}"
    finally:
        # Cleanup
        if temp_dir.exists():
            for f in temp_dir.glob("*"):
                f.unlink()
            temp_dir.rmdir()

def main():
    # List of test cases
    tests = [
        "unit_literal",
        "try_operator",
        "block_expr",
        "tuple_destruct",
        "closure_block",
        "parser_module",
        "test_hooks",
    ]

    print("Testing codegen output...")
    print("=" * 80)

    passed = 0
    failed = 0

    for i, test in enumerate(tests, 1):
        print(f"\n[{i}/{len(tests)}] {test} ... ", end="", flush=True)

        success, error = test_codegen(test)

        if success:
            print("PASSED")
            passed += 1
        else:
            print("FAILED")
            if error:
                print(f"\n{error}\n")
            failed += 1

    # Summary
    print("\n" + "=" * 80)
    print(f"Results: {passed} passed, {failed} failed out of {len(tests)} total")

    if failed == 0:
        print("\nAll codegen tests passed!")
        return 0
    else:
        print(f"\n{failed} test(s) failed")
        return 1

if __name__ == "__main__":
    exit(main())
