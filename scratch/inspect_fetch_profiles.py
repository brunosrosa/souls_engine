with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'function fetchProfiles' in l or 'async function fetchProfiles' in l:
        print(f"fetchProfiles at line {i+1}:")
        for j in range(i, min(len(lines), i + 120)):
            print(f"{j+1}: {lines[j]}", end='')
        break

