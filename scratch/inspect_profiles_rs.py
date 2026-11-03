with open('crates/souls_server/src/profiles.rs', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('deep_swe')
print(text[idx-50:idx+2000])
