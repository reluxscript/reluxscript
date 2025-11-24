#!/usr/bin/env python3
"""Fix minimact_full_refactored.rsc by moving everything inside writer block"""

input_file = "minimact_full_refactored_v3.rsc"
output_file = "minimact_full_refactored_fixed.rsc"

with open(input_file, 'r', encoding='utf-8') as f:
    lines = f.readlines()

output_lines = []
for i, line in enumerate(lines):
    line_num = i + 1

    # Remove #[derive(...)] lines
    if line.strip().startswith('#[derive'):
        continue

    # Skip the closing brace at line 187
    if line_num == 187 and line.strip() == '}':
        continue

    # Indent lines from 189 onwards (but not blank lines)
    if line_num >= 189:
        if line.strip():  # Only indent non-blank lines
            output_lines.append('    ' + line)
        else:
            output_lines.append('\n')  # Keep blank lines as-is
    else:
        output_lines.append(line)

# Add closing brace at the end
output_lines.append('}\n')

with open(output_file, 'w', encoding='utf-8') as f:
    f.writelines(output_lines)

print(f"Fixed file written to {output_file}")
print(f"Original: {len(lines)} lines")
print(f"Fixed: {len(output_lines)} lines")
