#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script to rename files from RustScript/rustscript to ReluxScript/reluxscript
"""
import os
import sys
import shutil
from pathlib import Path

# Fix console encoding for Windows
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')

def rename_files():
    """Rename all files containing 'rustscript' or 'RustScript' in their names"""

    # List of files to rename (found via git ls-files | grep -i rustscript)
    files_to_rename = [
        "docs/BABEL_PLUGIN_TO_RUSTSCRIPT_MIGRATION.md",
        "docs/babel-to-rustscript.md",
        "docs/rustscript-codegen-module.md",
        "docs/rustscript-compiler-implementation.md",
        "docs/rustscript-developers-guide.md",
        "docs/rustscript-enhancement-plan-2.md",
        "docs/rustscript-enhancement-plan.md",
        "docs/rustscript-gaps.md",
        "docs/rustscript-guide-for-babelers.md",
        "docs/rustscript-parser-module.md",
        "docs/rustscript-plugin-enhancements.md",
        "docs/rustscript-priority-1-plan.md",
        "docs/rustscript-specification.md",
        "docs/rustscript-type-aware-codegen.md",
    ]

    # Directories to rename
    directories_to_rename = [
        "source/tests/codegen/rustscript-plugin-minimact",
    ]

    renamed_count = 0
    errors = []

    # First rename directories (must be done before files)
    for old_path_str in directories_to_rename:
        old_path = Path(old_path_str)

        # Check if directory exists
        if not old_path.exists():
            errors.append(f"Directory not found: {old_path_str}")
            continue

        # Create new directory name
        new_name = old_path.name.replace("rustscript", "reluxscript").replace("RustScript", "ReluxScript")
        new_path = old_path.parent / new_name

        # Check if target already exists
        if new_path.exists():
            errors.append(f"Target directory already exists: {new_path}")
            continue

        try:
            # Rename the directory
            old_path.rename(new_path)
            print(f"[OK] Renamed directory: {old_path_str}")
            print(f"                    to: {new_path}")
            renamed_count += 1
        except Exception as e:
            errors.append(f"Error renaming directory {old_path_str}: {str(e)}")

    # Then rename files
    for old_path_str in files_to_rename:
        old_path = Path(old_path_str)

        # Check if file exists
        if not old_path.exists():
            errors.append(f"File not found: {old_path_str}")
            continue

        # Create new filename by replacing rustscript/RustScript with reluxscript/ReluxScript
        new_name = old_path.name.replace("rustscript", "reluxscript").replace("RustScript", "ReluxScript")
        new_path = old_path.parent / new_name

        # Check if target already exists
        if new_path.exists():
            errors.append(f"Target already exists: {new_path}")
            continue

        try:
            # Rename the file
            old_path.rename(new_path)
            print(f"[OK] Renamed: {old_path_str}")
            print(f"          to: {new_path}")
            renamed_count += 1
        except Exception as e:
            errors.append(f"Error renaming {old_path_str}: {str(e)}")

    # Print summary
    print(f"\n{'='*60}")
    print(f"Summary:")
    print(f"  Files renamed: {renamed_count}")
    print(f"  Errors: {len(errors)}")

    if errors:
        print(f"\nErrors encountered:")
        for error in errors:
            print(f"  - {error}")

    return renamed_count, errors

if __name__ == "__main__":
    print("RustScript to ReluxScript File Renamer")
    print("=" * 60)
    print()

    renamed, errors = rename_files()

    if errors:
        exit(1)
    else:
        print(f"\n[SUCCESS] Renamed {renamed} items!")
        exit(0)
