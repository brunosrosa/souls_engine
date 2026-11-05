import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

match_line = -1
for i, l in enumerate(head_lines):
    if '${m.author ||' in l:
        print(f"Found author at line {i+1}: {l}")
        match_line = i

if match_line != -1:
    print("Following lines in HEAD:")
    for j in range(match_line, min(match_line + 30, len(head_lines))):
        print(f"{j+1}: {head_lines[j]}")
    print("Total HEAD lines:", len(head_lines))

