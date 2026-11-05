with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    lines = f.readlines()

print("populateSettingsForm (12980-13048):")
for i in range(12980, 13049):
    print(f"  {i+1}: {lines[i].strip()}")

print("\nsaveSettings (13080-13160):")
for i in range(13080, 13160):
    print(f"  {i+1}: {lines[i].strip()}")

