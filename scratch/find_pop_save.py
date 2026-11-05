with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'function populateSettingsForm' in l:
        print(f"populateSettingsForm at line {i+1}")
        for j in range(i, min(i+35, len(lines))):
            print(f"  {j+1}: {lines[j].strip()}")
    if 'async function saveSettings' in l or 'function saveSettings' in l:
        print(f"saveSettings at line {i+1}")
        for j in range(i, min(i+35, len(lines))):
            print(f"  {j+1}: {lines[j].strip()}")

