with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print("Lines 4575 to 4670:")
for i in range(4574, min(len(lines), 4670)):
    if any(k in lines[i] for k in ['settings-', '<aside', '<main', '<div class="settings']):
        print(f"{i+1}: {lines[i]}", end='')

print("\nLines 7650 to 7850:")
for i in range(7645, min(len(lines), 7850)):
    if any(k in lines[i] for k in ['settings-', '</aside', '</main', '</div>', 'id="subtab-raw"', 'SUB-ABA']):
        print(f"{i+1}: {lines[i]}", end='')

