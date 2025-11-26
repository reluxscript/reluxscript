#!/usr/bin/env python3
"""
Extract methods from swc_original.rs into modular files based on line ranges.
"""

import re
import subprocess

# File paths
SWC_ORIGINAL = "src/codegen/swc_original.rs"
SWC_DIR = "src/codegen/swc"

# Method definitions with their line ranges (start, end)
# Format: (method_name, start_line, end_line, target_module)
METHODS = []

def find_method_ranges():
    """Find all method line numbers using grep."""
    # Find all 'fn ' declarations
    result = subprocess.run(
        ['grep', '-n', r'^\s*fn ', SWC_ORIGINAL],
        capture_output=True,
        text=True
    )

    lines = result.stdout.strip().split('\n')
    method_starts = []

    for line in lines:
        if not line:
            continue
        match = re.match(r'(\d+):\s*fn\s+(\w+)', line)
        if match:
            line_num = int(match.group(1))
            method_name = match.group(2)
            method_starts.append((line_num, method_name))

    # Find method end by looking for next method start
    method_ranges = []
    for i, (start, name) in enumerate(method_starts):
        if i + 1 < len(method_starts):
            end = method_starts[i + 1][0] - 1
        else:
            # Last method goes to end of impl block (roughly line 3370)
            end = 3370
        method_ranges.append((name, start, end))

    return method_ranges

def read_lines(file_path, start, end):
    """Read lines from start to end (inclusive)."""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    # Convert to 0-indexed
    return ''.join(lines[start-1:end])

def classify_method(method_name):
    """Determine which module a method belongs to."""

    # Emit methods
    if method_name in ['emit', 'emit_indent', 'emit_line']:
        return 'emit'

    # Detection methods
    if 'detect' in method_name:
        return 'detection'

    # Type mapping methods
    if any(x in method_name for x in ['to_swc', 'to_rust', '_type', '_op', 'type_to', 'reluxscript_']):
        return 'type_mapping'

    # Type inference (deprecated)
    if method_name in ['infer_type', 'type_from_ast', 'get_element_type', 'extract_matches_pattern']:
        return 'type_inference'

    # Pattern generation
    if 'pattern' in method_name.lower():
        return 'patterns'

    # Expression generation
    if method_name in ['gen_expr', 'gen_literal', 'gen_matches_macro', 'gen_decorated_expr']:
        return 'expressions'

    # Statement generation
    if method_name in ['gen_stmt', 'gen_block', 'gen_if_let_stmt', 'gen_traverse_stmt',
                       'gen_stmt_with_context', 'gen_decorated_stmt', 'gen_decorated_if_let_stmt',
                       'gen_decorated_block']:
        return 'statements'

    # Structure generation
    if method_name in ['gen_struct', 'gen_enum', 'gen_helper_function',
                       'gen_parser_module_helpers', 'gen_codegen_module_helpers',
                       'gen_codegen_call', 'gen_codegen_config']:
        return 'structures'

    # Visitor generation
    if 'visitor' in method_name or 'visit_method' in method_name or method_name == 'is_visitor_method':
        return 'visitors'

    # Top-level generation
    if method_name in ['gen_plugin', 'gen_writer', 'gen_module', 'gen_decorated_plugin',
                       'gen_decorated_writer', 'gen_decorated_visitor_method']:
        return 'top_level'

    # Default to top_level for anything else
    return 'top_level'

def main():
    print("Finding method ranges...")
    method_ranges = find_method_ranges()

    print(f"Found {len(method_ranges)} methods")

    # Group methods by module
    modules = {
        'emit': [],
        'detection': [],
        'type_mapping': [],
        'type_inference': [],
        'patterns': [],
        'expressions': [],
        'statements': [],
        'structures': [],
        'visitors': [],
        'top_level': []
    }

    for name, start, end in method_ranges:
        module = classify_method(name)
        modules[module].append((name, start, end))
        print(f"  {name:40} -> {module:20} (lines {start}-{end})")

    # Write each module
    for module_name, methods in modules.items():
        if not methods:
            continue

        output_file = f"{SWC_DIR}/{module_name}.rs"
        print(f"\nWriting {output_file} ({len(methods)} methods)...")

        # Read existing header
        with open(output_file, 'r', encoding='utf-8') as f:
            content = f.read()

        # Keep only the header (up to impl block)
        header_end = content.find('impl SwcGenerator {')
        if header_end == -1:
            header = content
        else:
            header = content[:header_end]

        # Build new content
        new_content = header + 'impl SwcGenerator {\n'

        for name, start, end in methods:
            method_code = read_lines(SWC_ORIGINAL, start, end)
            # Change fn to pub(super) fn if not already
            if method_code.strip().startswith('fn '):
                method_code = '    pub(super) ' + method_code.strip()
            else:
                method_code = '    ' + method_code.strip()

            new_content += method_code + '\n'

        new_content += '}\n'

        # Write file
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(new_content)

        print(f"  Wrote {len(methods)} methods to {output_file}")

if __name__ == '__main__':
    main()
