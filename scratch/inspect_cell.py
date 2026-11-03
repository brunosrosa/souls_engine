with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('${brandBadge}\n                    ${capTags.join(\'\')}')
print('Cell target:', idx)
print(text[idx-120:idx+350])

