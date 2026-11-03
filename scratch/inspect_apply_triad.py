with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('applyTriadModel')
while idx != -1:
    print('applyTriadModel at', idx)
    print(text[idx-20:idx+200])
    idx = text.find('applyTriadModel', idx+1)
