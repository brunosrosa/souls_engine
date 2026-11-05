with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Let's inspect line by line from 7168 to 7665
stack = []
for idx in range(7167, 7665):
    line = lines[idx]
    # Simple tag tokenizer for div, main, aside, section
    i = 0
    while i < len(line):
        if line[i:i+4] == '<div':
            stack.append((idx + 1, 'div', line.strip()[:40]))
            i += 4
        elif line[i:i+6] == '</div>':
            if stack:
                popped = stack.pop()
            else:
                print(f"Excess </div> at line {idx+1}")
            i += 6
        else:
            i += 1

print(f"Remaining open divs in subtab-obsidian: {len(stack)}")
for item in stack:
    print(f"  Line {item[0]}: {item[1]} -> {item[2]}")

