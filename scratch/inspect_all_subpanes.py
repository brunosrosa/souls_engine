with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

import re
matches = re.findall(r'Governan[çc]a\s*&\s*Presets', text)
print('Occurrences of Governança & Presets:', len(matches))
for m in re.finditer(r'Governan[çc]a\s*&\s*Presets', text):
    idx = m.start()
    print(text[max(0, idx-30):min(len(text), idx+60)])
