with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'cfg-sandbox-timeout' in l or 'cfg-ast-tree-limit' in l:
        print(f"Line {i+1}: {l.strip()[:80]}")
        for j in range(max(0, i-5), min(i+15, len(lines))):
            print(f"  {j+1}: {lines[j].strip()}")
        print("-" * 50)

