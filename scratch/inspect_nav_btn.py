with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'subtab-sandbox' in l and 'button' in l:
        print(f"Line {i+1}: {l.strip()}")
        for j in range(max(0, i-2), min(i+15, len(lines))):
            print(f"  {j+1}: {lines[j].strip()}")
        break

