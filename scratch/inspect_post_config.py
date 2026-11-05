with open('crates/souls_server/src/routes/dashboard.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if 'pub async fn post_config_handler' in l:
        print(f"at line {i+1}:")
        for j in range(i, min(len(lines), i + 45)):
            print(f"{j+1}: {lines[j]}", end='')
        break

