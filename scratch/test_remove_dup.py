with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

# Test removing lines 6194 to 6301 (0-indexed: 6193 to 6301)
test_lines = lines[:6193] + lines[6301:]

stack = []
for idx, line in enumerate(test_lines):
    if idx + 1 >= 5825 and idx + 1 <= 6310:
        i = 0
        while i < len(line):
            if line[i:i+4] == "<div":
                end = line.find(">", i)
                if end != -1:
                    tag = line[i:end+1]
                    stack.append((idx + 1, tag))
                    i = end + 1
                else:
                    break
            elif line[i:i+6] == "</div>":
                if stack:
                    opened_line, opened_tag = stack.pop()
                else:
                    print(f"EXTRA CLOSING DIV at line {idx+1}")
                i += 6
            else:
                i += 1

print(f"Remaining open divs in test: {len(stack)}")
for l, t in stack:
    print(f"  Still open from line {l}: {t[:50]}")
