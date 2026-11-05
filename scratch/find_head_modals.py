import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = out.splitlines()

for i, l in enumerate(head_lines):
    if '</script>' in l:
        print(f"HEAD </script> at line {i+1}")
        for j in range(i+1, min(i+50, len(head_lines))):
            print(f"{j+1}: {head_lines[j]}")
        break

