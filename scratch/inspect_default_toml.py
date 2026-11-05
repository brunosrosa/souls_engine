with open('crates/souls_server/src/routes/dashboard.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'get_default_config_toml' in l:
        print(f"at line {i+1}:")
        for j in range(i, min(len(lines), i + 60)):
            print(f"{j+1}: {lines[j]}", end='')
        break

