with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('.tooltip-box {')
print(text[idx-50:idx+900])
