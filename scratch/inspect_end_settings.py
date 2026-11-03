with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('id="subtab-raw"')
end_sec = text.find('</section>', idx)
print('subtab-raw from', idx, 'to', end_sec)
print(text[idx+1000:end_sec+15])

