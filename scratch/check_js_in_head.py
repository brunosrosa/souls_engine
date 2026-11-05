import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

for i, l in enumerate(head_lines):
    if 'initSandboxSubtab' in l or 'provisionDataSpace' in l or 'saveSettings' in l or 'initMemorySubtab' in l:
        print(f"HEAD line {i+1}: {l.strip()[:60]}")

