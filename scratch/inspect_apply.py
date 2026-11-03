with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('id="modal-quick-apply"')
print(text[idx+1800:idx+3500])
