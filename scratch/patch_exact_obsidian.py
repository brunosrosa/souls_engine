with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Update title
old_t = '<span>Caminho do Cofre Obsidian (OneDrive / Local)</span>'
new_t = '<span>Diretório do Cofre de Conhecimentos</span>'
content = content.replace(old_t, new_t, 1)

# 2. Update select width and options
old_sel = '''                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 250px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                    <option value="onedrive" selected>OneDrive Sincronizado (Nuvem Pessoal)</option>
                    <option value="local_vault">Cofre Local (%USERPROFILE%\\Documents)</option>
                    <option value="custom">Diretório Customizado / Air-Gapped</option>
                  </select>'''

new_sel = '''                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 200px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                    <option value="onedrive" selected>OneDrive Sincronizado</option>
                    <option value="local_vault">Cofre Local (Documentos)</option>
                    <option value="custom">Diretório Customizado</option>
                  </select>'''

content = content.replace(old_sel, new_sel, 1)

# Ensure LF
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Applied exact string replacements!")

