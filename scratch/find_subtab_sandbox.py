with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'id="subtab-sandbox"' in l:
        print(f"Line {i+1}: {l.strip()}")

