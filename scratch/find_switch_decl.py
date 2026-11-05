with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'function switchSettingsSubtab' in l:
        print(f"Line {i+1}: {l.strip()}")
        for j in range(i, min(i+25, len(lines))):
            print(f"  {j+1}: {lines[j].strip()}")
        break

