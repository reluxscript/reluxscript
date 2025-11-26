#!/usr/bin/env python3
"""
Extract methods from swc_original.rs using grep to find line numbers.
Much simpler and more reliable than brace counting!
"""

import subprocess
import re
from pathlib import Path

# Module configurations
modules = {
    'emit.rs': [
        'emit',
        'emit_indent',
        'emit_line',
    ],
    'detection.rs': [
        'detect_std_collections',
        'detect_collections_in_item',
        'detect_collections_in_type',
        'detect_collections_in_block',
        'detect_collections_in_stmt',
        'detect_collections_in_expr',
    ],
    'type_mapping.rs': [
        'visitor_name_to_swc',
        'visitor_name_to_swc_type',
        'to_swc_node_name',
        'reluxscript_to_swc_type',
        'type_to_rust',
        'get_default_value_for_type',
        'visitor_method_to_swc',
        'reluxscript_type_to_swc',
        'binary_op_to_rust',
        'compound_op_to_rust',
    ],
    'type_inference.rs': [
        'infer_type',
        'type_from_ast',
        'get_element_type',
        'extract_matches_pattern',
    ],
    'patterns.rs': [
        'gen_pattern',
        'gen_swc_pattern_check',
        'gen_decorated_pattern',
    ],
    'expressions.rs': [
        'gen_expr',
        'gen_literal',
        'gen_matches_macro',
        'gen_decorated_expr',
    ],
    'statements.rs': [
        'gen_block',
        'gen_stmt',
        'gen_stmt_with_context',
        'gen_if_let_stmt',
        'gen_traverse_stmt',
        'gen_decorated_stmt',
        'gen_decorated_if_let_stmt',
        'gen_decorated_block',
    ],
    'structures.rs': [
        'gen_struct',
        'gen_enum',
        'gen_helper_function',
        'gen_parser_module_helpers',
        'gen_codegen_module_helpers',
        'gen_codegen_call',
        'gen_codegen_config',
    ],
    'visitors.rs': [
        'gen_visitor_method',
        'gen_visit_method',
        'is_visitor_method',
    ],
    'top_level.rs': [
        'gen_plugin',
        'gen_writer',
        'gen_module',
        'gen_decorated_plugin',
        'gen_decorated_writer',
        'gen_decorated_visitor_method',
    ],
}

# Read the source file
source_file = Path("src/codegen/swc_original.rs")
with open(source_file, 'r', encoding='utf-8') as f:
    lines = f.readlines()

def find_method_lines(method_name):
    """Use grep to find the line number where a method starts"""
    try:
        # Find line with fn method_name(
        result = subprocess.run(
            ['grep', '-n', f'fn {method_name}(', str(source_file)],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            return None

        # Parse output: "123:    fn method_name("
        match = re.match(r'(\d+):', result.stdout)
        if match:
            return int(match.group(1))
    except Exception as e:
        print(f"Error finding {method_name}: {e}")

    return None

def extract_method(start_line):
    """Extract method starting at start_line by counting braces"""
    if start_line is None or start_line > len(lines):
        return None

    # Find the opening brace on the start line or next line
    current = start_line - 1  # Convert to 0-indexed

    # Skip until we find the opening brace
    while current < len(lines) and '{' not in lines[current]:
        current += 1

    if current >= len(lines):
        return None

    # Now count braces from this line
    method_lines = []
    brace_count = 0
    in_string = False

    for i in range(current, len(lines)):
        line = lines[i]
        method_lines.append(line)

        # Simple brace counting (ignore strings)
        for char in line:
            if char == '"':
                in_string = not in_string
            elif not in_string:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1

        # Stop when braces are balanced
        if brace_count == 0 and '{' in ''.join(method_lines):
            return ''.join(method_lines)

    return None

# Extract methods for each module
for module_name, method_names in modules.items():
    module_path = Path(f"src/codegen/swc/{module_name}")

    # Read existing template
    existing = module_path.read_text(encoding='utf-8')

    # Find the impl block
    impl_start = existing.find('impl SwcGenerator {')
    if impl_start == -1:
        print(f"Warning: No impl block found in {module_name}")
        continue

    # Extract methods
    methods = []
    for method_name in method_names:
        start_line = find_method_lines(method_name)
        if start_line:
            method_content = extract_method(start_line)
            if method_content:
                # Change fn to pub(super) fn
                method_content = re.sub(
                    rf'(\s+)fn {re.escape(method_name)}\(',
                    rf'\1pub(super) fn {method_name}(',
                    method_content,
                    count=1
                )
                methods.append(method_content)
                print(f"[OK] Extracted {method_name} -> {module_name} (line {start_line})")
            else:
                print(f"[FAIL] Could not extract {method_name} (found at line {start_line})")
        else:
            print(f"[FAIL] Could not find {method_name}")

    # Build new content
    header = existing[:impl_start + len('impl SwcGenerator {') + 1]
    footer = '\n}\n'

    new_content = header + '\n' + '\n'.join(methods) + footer

    # Write back
    module_path.write_text(new_content, encoding='utf-8')
    print(f"[OK] Updated {module_name} with {len(methods)} methods\n")

print("Extraction complete!")
