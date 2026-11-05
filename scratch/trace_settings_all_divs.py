with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

start_line = None
end_line = None
for i, l in enumerate(lines):
    if 'id="tab-settings"' in l:
        start_line = i
    if start_line is not None and '</section>' in l and i > start_line:
        end_line = i
        break

print(f"tab-settings from line {start_line+1} to {end_line+1}")

tag_stack = []
for idx in range(start_line, end_line + 1):
    line = lines[idx]
    # Simple scanner for opening and closing tags
    import re
    tokens = re.findall(r'<(div|main|aside|section)[^>]*>|</(div|main|aside|section)>', line, re.IGNORECASE)
    for op, cl in tokens:
        if op:
            tag_name = op.lower()
            tag_stack.append((idx + 1, tag_name, line.strip()[:60]))
        elif cl:
            tag_name = cl.lower()
            if not tag_stack:
                print(f"ERROR: Unmatched closing </{tag_name}> at line {idx+1}")
            else:
                last_line, last_tag, last_snippet = tag_stack.pop()
                if last_tag != tag_name:
                    print(f"MISMATCH at line {idx+1}: closed </{tag_name}> but expected </{last_tag}> (opened at line {last_line}: {last_snippet})")

print(f"\nRemaining unclosed tags in tab-settings: {len(tag_stack)}")
for item in tag_stack:
    print(f"  Line {item[0]}: <{item[1]}> -> {item[2]}")

