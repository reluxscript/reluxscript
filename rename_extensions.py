#!/usr/bin/env python3
"""
Script to rename .rsc files to .lux files
"""
import os
from pathlib import Path

def rename_rsc_to_lux(root_dir="."):
    """Rename all .rsc files to .lux in the given directory tree"""

    root_path = Path(root_dir)
    rsc_files = list(root_path.rglob("*.rsc"))

    renamed_count = 0
    errors = []

    print(f"Found {len(rsc_files)} .rsc files to rename")
    print("=" * 60)

    for rsc_file in rsc_files:
        # Create new filename with .lux extension
        lux_file = rsc_file.with_suffix(".lux")

        # Check if target already exists
        if lux_file.exists():
            errors.append(f"Target already exists: {lux_file}")
            continue

        try:
            # Rename the file
            rsc_file.rename(lux_file)
            print(f"[OK] {rsc_file.relative_to(root_path)} -> {lux_file.name}")
            renamed_count += 1
        except Exception as e:
            errors.append(f"Error renaming {rsc_file}: {str(e)}")

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
    print("ReluxScript File Extension Renamer (.rsc -> .lux)")
    print("=" * 60)
    print()

    renamed, errors = rename_rsc_to_lux()

    if errors:
        exit(1)
    else:
        print(f"\n[SUCCESS] Renamed {renamed} files!")
        exit(0)
