import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

for i, l in enumerate(head_lines):
    if '<script>' in l:
        print(f"HEAD script tag at line {i+1}")
        break

