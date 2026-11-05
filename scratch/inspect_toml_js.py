with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'function fetchConfigToml' in l or 'function saveRawToml' in l:
        print(f"at line {i+1}:")
        for j in range(i, min(len(lines), i + 40)):
            print(f"{j+1}: {lines[j]}", end='')

