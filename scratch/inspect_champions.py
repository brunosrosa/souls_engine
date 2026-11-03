with open('crates/souls_server/src/profiles.rs', 'r', encoding='utf-8') as f:
    text = f.read()

idx = text.find('Frontier Balanced')
if idx != -1:
    print('Found Frontier Balanced at', idx)
    print(text[idx-200:idx+800])
