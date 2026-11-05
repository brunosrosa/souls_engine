with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'settings-subtab-btn' in l:
        print(f'{i+1}: {l.strip()[:100]}')

