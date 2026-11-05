with open('crates/souls_server/src/routes/dashboard.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'get_config_handler' in l or 'post_config_handler' in l:
        print(f"at line {i+1}:")
        for j in range(i, min(len(lines), i + 40)):
            print(f"{j+1}: {lines[j]}", end='')
        break

