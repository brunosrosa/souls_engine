with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(7320, 7370):
    print(f'{i+1}: {lines[i]}', end='')

