with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(7570, 7663):
    line = lines[i]
    if '<div' in line or '</div' in line:
        print(f"{i+1}: {line.strip()[:60]}")

