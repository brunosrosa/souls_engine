with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

tag_stack = []
for idx in range(7167, 7664):
    line = lines[idx]
    import re
    tokens = re.finditer(r'<(div)[^>]*>|</(div)>', line, re.IGNORECASE)
    for m in tokens:
        tag_str = m.group(0)
        if tag_str.startswith('</'):
            if tag_stack:
                last_line, last_snippet = tag_stack.pop()
            else:
                print(f"Line {idx+1}: EXTRA </div> with no matching <div")
        else:
            tag_stack.append((idx + 1, line.strip()[:60]))

print(f"At line 7664, unclosed divs in subtab-obsidian: {len(tag_stack)}")
for item in tag_stack:
    print(f"  Line {item[0]}: {item[1]}")

