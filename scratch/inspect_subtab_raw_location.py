with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'id="subtab-raw"' in l or 'settings-layout' in l or 'settings-main' in l or 'id="tab-settings"' in l:
        print(f"{i+1}: {l.strip()[:100]}")

