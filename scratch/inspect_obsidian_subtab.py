with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

sub_start = -1
sub_end = -1
for i, l in enumerate(lines):
    if 'id="subtab-obsidian"' in l:
        sub_start = i
    if 'id="subtab-raw"' in l:
        sub_end = i
        break

print(f"subtab-obsidian starts at line {sub_start+1}, subtab-raw at line {sub_end+1}")
print("Content currently in subtab-obsidian:")
for j in range(sub_start, sub_end):
    print(f"{j+1}: {lines[j]}", end='')

