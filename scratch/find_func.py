with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

for i in range(9900, 9953):
    print(f"{i+1}: {lines[i]}", end='')

# Search backwards from 9900 to find the function name
for i in range(9900, 7313, -1):
    if 'function ' in lines[i]:
        print(f"\nFunction defined at line {i+1}: {lines[i].strip()}")
        break

