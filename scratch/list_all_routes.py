with open('crates/souls_server/src/routes/mod.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if '.route(' in l:
        print(l.strip())

