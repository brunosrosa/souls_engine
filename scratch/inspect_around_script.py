with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print("Lines 7150 to 7330:")
for i in range(7150, min(7330, len(lines))):
    print(f"{i+1}: {lines[i]}", end='')

