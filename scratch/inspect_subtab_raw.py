with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'id="subtab-raw"' in l:
        print(f"subtab-raw at line {i+1}:")
        for j in range(i, min(len(lines), i + 100)):
            print(f"{j+1}: {lines[j]}", end='')
        break

