with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Let's inspect from line 7160 (start of subtab-obsidian) to line 7845
depth = 0
for i in range(7160, min(len(lines), 7675)):
    line = lines[i]
    opens = line.count('<div') + line.count('<main') + line.count('<aside')
    closes = line.count('</div') + line.count('</main') + line.count('</aside')
    depth += (opens - closes)
    if any(k in line for k in ['subtab-obsidian', 'subtab-raw', 'settings-content-area', 'settings-layout']):
        print(f"Line {i+1} (depth={depth}): {line.strip()[:80]}")
    if opens != closes:
        # print lines with tag changes near the end
        if i >= 7645:
            print(f"Line {i+1} (opens={opens}, closes={closes}, depth={depth}): {line.strip()[:80]}")

print("Total depth at subtab-raw start:", depth)

