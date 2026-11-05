with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

in_content_area = False
for i, line in enumerate(lines):
    if '<main class="settings-content-area">' in line:
        in_content_area = True
        print(f"Line {i+1}: ENTER settings-content-area")
    elif '</main>' in line and in_content_area:
        in_content_area = False
        print(f"Line {i+1}: EXIT settings-content-area")
    
    if 'class="settings-subpane' in line:
        print(f"Line {i+1} (inside_content_area={in_content_area}): {line.strip()[:60]}")

