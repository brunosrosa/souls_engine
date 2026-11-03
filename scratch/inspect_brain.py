with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('function renderProfileCards()')
print(text[idx+1000:idx+3000])
