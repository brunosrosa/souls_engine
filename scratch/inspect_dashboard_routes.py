with open('crates/souls_server/src/routes/dashboard.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(1920, min(len(lines), 1995)):
    print(f"{i+1}: {lines[i]}", end='')

