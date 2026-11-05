with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'subtab-brain' in l and 'settings-subpane' in l:
        print(f"subtab-brain at line {i+1}:")
        for j in range(i, min(len(lines), i + 100)):
            print(f"{j+1}: {lines[j]}", end='')
        break

