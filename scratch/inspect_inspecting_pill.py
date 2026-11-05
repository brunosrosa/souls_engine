with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if '.inspecting-pill' in l:
        print(f"inspecting-pill at line {i+1}:")
        for j in range(max(0, i-2), min(len(lines), i + 25)):
            print(f"{j+1}: {lines[j]}", end='')
        break

