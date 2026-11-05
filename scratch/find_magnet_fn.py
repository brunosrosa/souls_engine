import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

for i, l in enumerate(head_lines):
    if 'magnet' in l.lower() or 'snap' in l.lower():
        if 'function ' in l or 'init' in l:
            print(f"HEAD Line {i+1}: {l.strip()}")

