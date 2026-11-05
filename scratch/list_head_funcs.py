import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

for i in range(8057, len(head_lines)):
    l = head_lines[i]
    if 'function ' in l or 'async function ' in l:
        print(f"HEAD {i+1}: {l.strip()[:70]}")

