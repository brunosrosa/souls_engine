with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'initSandboxSubtab' in l or 'provisionDataSpace' in l or 'setSandboxScenario' in l:
        print(f"Working copy line {i+1}: {l.strip()[:60]}")

