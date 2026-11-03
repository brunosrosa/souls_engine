with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('</script>')
print('</script> found at:', idx)
print('Length of file:', len(text))
print(text[idx:idx+2500])
