with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

stack = []
for idx in range(5826, 6410):
    line = lines[idx]
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
                if opened_line == 5827:
                    print(f"Line 5827 (<div id=subtab-router>) was closed by line {idx+1}")
            i += 6
        else:
            i += 1
