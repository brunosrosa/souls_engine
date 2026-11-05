with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print("Lines 6770 to 6880:")
for i in range(6770, 6880):
    print(f"{i+1}: {lines[i]}", end='')

