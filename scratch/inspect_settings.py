with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('.settings-subtabs {')
print('CSS around .settings-subtabs:')
print(text[idx:idx+1500])
