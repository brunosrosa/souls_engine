with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(1630, min(len(lines), 1850)):
    if any(k in lines[i] for k in ['.settings-layout', '.settings-sidebar', '.settings-content-area', '.settings-subpane']):
        for j in range(max(0, i-2), min(len(lines), i + 25)):
            print(f"{j+1}: {lines[j]}", end='')
        print("---")

