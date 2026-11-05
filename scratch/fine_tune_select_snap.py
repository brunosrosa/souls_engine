with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Update Card 3 select options to never clip
old_opt = '''                    <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 210px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                      <option value="hybrid" selected>Híbrido Anonimizado (Padrão)</option>
                      <option value="strict_local">Estrito Local (Silício)</option>
                    </select>'''

new_opt = '''                    <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 195px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                      <option value="hybrid" selected>Híbrido Anonimizado</option>
                      <option value="strict_local">Estrito no Silício</option>
                    </select>'''

if old_opt in content:
    content = content.replace(old_opt, new_opt, 1)
    print("Updated privacy mode select options!")
else:
    print("Warning: old_opt not found!")

# 2. Update initSlidersMagneticSnap to toggle .default-indicator visibility
old_snap = '''        const updateTickStyle = (val) => {
          if (tick) {
            if (Math.abs(val - defVal) <= 0.001) {
              tick.classList.add('at-default');
            } else {
              tick.classList.remove('at-default');
            }
          }
        };'''

new_snap = '''        const updateTickStyle = (val) => {
          const isDef = Math.abs(val - defVal) <= 0.001;
          if (tick) {
            if (isDef) {
              tick.classList.add('at-default');
            } else {
              tick.classList.remove('at-default');
            }
          }
          const badge = slider.closest('.setting-item')?.querySelector('.range-val-badge .default-indicator');
          if (badge) {
            badge.style.display = isDef ? 'inline' : 'none';
          }
        };'''

if old_snap in content:
    content = content.replace(old_snap, new_snap, 1)
    print("Updated initSlidersMagneticSnap to toggle default-indicator!")
else:
    print("Warning: old_snap not found!")

# Ensure LF
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Saved fine-tuning!")

