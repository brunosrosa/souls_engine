with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'id="subtab-vault"' in l:
        print(f"subtab-vault at line {i+1}")
    if 'id="subtab-toml"' in l:
        print(f"subtab-toml at line {i+1}")
    if 'tab-models' in l:
        print(f"tab-models at line {i+1}")
    if '<script>' in l or '<script ' in l:
        print(f"script tag at line {i+1}")

