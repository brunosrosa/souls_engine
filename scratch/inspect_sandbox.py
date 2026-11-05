with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'id="subtab-sandbox"' in l:
        print(f'subtab-sandbox at line {i+1}')
        for j in range(i, min(i+100, len(lines))):
            print(f'{j+1}: {lines[j]}', end='')
        break

