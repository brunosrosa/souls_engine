with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print("Total lines:", len(lines))
# find subtab-sandbox closing tag
subtab_start = -1
subtab_end = -1
for i, l in enumerate(lines):
    if 'id="subtab-sandbox"' in l:
        subtab_start = i
    if subtab_start != -1 and i > subtab_start and '<!-- FIM SUBTAB-SANDBOX -->' in l or ('id="subtab-vault"' in l or 'id="subtab-toml"' in l):
        print(f"subtab ended or next subtab at line {i+1}: {l.strip()}")

# check lines around 8000 to end
for i in range(len(lines)-30, len(lines)):
    print(f"{i+1}: {lines[i]}", end='')

