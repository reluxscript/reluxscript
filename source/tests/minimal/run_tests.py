#!/usr/bin/env python3
"""
TDD Test Runner for ReluxScript Minimal Tests

Runs each .lux file through the compiler and checks for:
1. Successful compilation
2. Expected output patterns in generated code
3. No compilation errors
"""

import subprocess
import os
import sys
from pathlib import Path
from typing import List, Tuple, Optional

# ANSI color codes
GREEN = '\033[92m'
RED = '\033[91m'
YELLOW = '\033[93m'
RESET = '\033[0m'
BOLD = '\033[1m'

class TestCase:
    def __init__(self, name: str, file: str, checks: List[Tuple[str, str]]):
        self.name = name
        self.file = file
        self.checks = checks  # List of (description, pattern_to_find)

def run_test(test: TestCase, relux_path: str) -> Tuple[bool, List[str]]:
    """Run a single test case and return (passed, errors)"""
    errors = []

    print(f"\n{BOLD}Testing: {test.name}{RESET}")
    print(f"  File: {test.file}")

    # Run the compiler
    try:
        result = subprocess.run(
            [relux_path, 'build', test.file, '--target', 'swc'],
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            errors.append(f"Compilation failed with code {result.returncode}")
            if result.stderr:
                errors.append(f"Error output: {result.stderr[:500]}")
            return False, errors

    except subprocess.TimeoutExpired:
        errors.append("Compilation timed out")
        return False, errors
    except Exception as e:
        errors.append(f"Failed to run compiler: {e}")
        return False, errors

    # Read generated output
    output_file = Path('dist/lib.rs')
    if not output_file.exists():
        errors.append("No output file generated")
        return False, errors

    try:
        with open(output_file, 'r', encoding='utf-8') as f:
            output = f.read()
    except Exception as e:
        errors.append(f"Failed to read output: {e}")
        return False, errors

    # Run checks
    all_passed = True
    for description, pattern in test.checks:
        if pattern in output:
            print(f"  {GREEN}PASS{RESET} {description}")
        else:
            print(f"  {RED}FAIL{RESET} {description}")
            errors.append(f"Expected pattern not found: {pattern}")
            all_passed = False

    return all_passed, errors

def main():
    script_dir = Path(__file__).parent
    relux_path = script_dir / '../../target/release/relux.exe'

    if not relux_path.exists():
        print(f"{RED}Error: Compiler not found at {relux_path}{RESET}")
        print("Please build the compiler first: cargo build --release")
        sys.exit(1)

    # Change to test directory
    os.chdir(script_dir)

    # Define tests with expected patterns
    tests = [
        TestCase(
            name="Path Expressions",
            file="test_path_expressions.lux",
            checks=[
                ("codegen::generate uses ::", "codegen::generate"),
                ("fs::read_to_string uses ::", "fs::read_to_string"),
            ]
        ),
        TestCase(
            name="Macro Calls",
            file="test_macro_calls.lux",
            checks=[
                ("format! has suffix", "format!("),
                ("println! has suffix", "println!("),
                ("vec! has suffix", "vec!["),
                ("panic! has suffix", "panic!("),
            ]
        ),
        TestCase(
            name="Pattern Matching",
            file="test_pattern_matching.lux",
            checks=[
                ("Pat::Array qualification", "Pat::Array"),
                ("Pat::Object qualification", "Pat::Object"),
                ("Pat::Ident qualification", "Pat::Ident"),
                ("Expr::Call qualification", "Expr::Call"),
                ("Expr::Member qualification", "Expr::Member"),
                ("Stmt::Return qualification", "Stmt::Return"),
                ("Stmt::If qualification", "Stmt::If"),
            ]
        ),
        TestCase(
            name="Named Imports",
            file="test_named_imports.lux",
            checks=[
                ("mod helpers declaration", "mod helpers;"),
                ("use helpers with named imports", "use helpers::{escape_string, uppercase};"),
                ("Can call imported function", "escape_string(input)"),
            ]
        ),
        TestCase(
            name="Built-in Module Imports",
            file="test_builtin_imports.lux",
            checks=[
                ("fs module imported", "use std::fs;"),
                ("json/serde imported", "use serde_json;"),
                ("codegen uses ::", "codegen::generate"),
                ("fs uses ::", "fs::read_to_string"),
            ]
        ),
        TestCase(
            name="Minimact Patterns",
            file="test_minimact_patterns.lux",
            checks=[
                ("Pat::Array for array destructuring", "Pat::Array"),
                ("Pat::Ident for identifier patterns", "Pat::Ident"),
                ("Stmt::VariableDeclaration", "Stmt::Var"),
                ("Nested if-let compiles", "if let"),
            ]
        ),
    ]

    # Run all tests
    total = len(tests)
    passed = 0
    failed = 0

    print(f"\n{BOLD}Running {total} test cases...{RESET}\n")

    for test in tests:
        success, errors = run_test(test, str(relux_path))

        if success:
            passed += 1
            print(f"{GREEN}PASS{RESET}")
        else:
            failed += 1
            print(f"{RED}FAIL{RESET}")
            if errors:
                for error in errors[:3]:  # Show first 3 errors
                    print(f"  {RED}Error:{RESET} {error}")

    # Summary
    print(f"\n{BOLD}{'='*60}{RESET}")
    print(f"{BOLD}Test Summary{RESET}")
    print(f"{'='*60}")
    print(f"Total:  {total}")
    print(f"{GREEN}Passed: {passed}{RESET}")
    if failed > 0:
        print(f"{RED}Failed: {failed}{RESET}")
    else:
        print(f"Failed: {failed}")

    if failed == 0:
        print(f"\n{GREEN}{BOLD}All tests passed!{RESET}")
        sys.exit(0)
    else:
        print(f"\n{RED}{BOLD}Some tests failed!{RESET}")
        sys.exit(1)

if __name__ == '__main__':
    main()
