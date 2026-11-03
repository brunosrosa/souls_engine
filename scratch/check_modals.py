import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
lines = out.splitlines()

start = -1
for i, l in enumerate(lines):
    if 'id="modal-quick-apply"' in l:
        start = i
        break

for i in range(start, min(len(lines), start+70)):
    print(f'{i+1}: {lines[i]}')
