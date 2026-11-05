with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

print("Checking presence of obsidian IDs:")
for eid in [
    'cfg-obsidian-path',
    'cfg-obsidian-storage-type',
    'cfg-obsidian-batch-size',
    'cfg-obsidian-interval-min',
    'cfg-obsidian-privacy-shield',
    'cfg-obsidian-raw-ro',
    'cfg-obsidian-wikilinks',
    'cfg-obsidian-privacy-mode'
]:
    print(f"{eid}: {'Found' if eid in text else 'NOT found'}")

