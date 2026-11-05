with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(7640, min(len(lines), 7668)):
    print(f"{i+1}: {repr(lines[i])}")

