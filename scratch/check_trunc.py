import subprocess

out = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    curr = f.read()

out = out.replace('\r\n', '\n')
curr = curr.replace('\r\n', '\n')

anchor = 'const id = m.name || m.model_name || m.id;\n'
curr_idx = curr.find(anchor)
head_idx = out.find(anchor)

print('curr_idx:', curr_idx)
print('head_idx:', head_idx)

if curr_idx != -1 and head_idx != -1:
    cut_curr = curr[:curr_idx + len(anchor)]
    tail_head = out[head_idx + len(anchor):]
    full = cut_curr + tail_head
    print('Full length:', len(full))
    print('Has </script>:', '</script>' in full)
    print('Has </html>:', '</html>' in full)
    print('Has modal-quick-apply:', 'modal-quick-apply' in full)
    with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
        f.write(full)
    print('Successfully restored full dashboard_ui.html!')
else:
    print('Failed to find anchor')
