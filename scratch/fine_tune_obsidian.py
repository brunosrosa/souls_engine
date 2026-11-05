with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Card 1 Title & Select
old_card1_item1 = '''              <!-- ITEM 1: CAMINHO DO COFRE -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px; font-size: 0.78rem; font-weight: 600; color: #cbd5e1;">
                    <span>Caminho do Cofre Obsidian (OneDrive / Local)</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Localização Física do Cofre</strong>
                        Pasta sincronizada via OneDrive ou armazenada no disco local do usuário.
                      </span>
                    </span>
                  </div>
                  <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ OneDrive Conectado
                  </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                  <input type="text" id="cfg-obsidian-path" class="setting-input" value="C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
                  <button class="btn btn-secondary" onclick="copyToClipboard(document.getElementById('cfg-obsidian-path').value, 'Caminho do Cofre')" title="Copiar caminho completo" style="padding: 7px 12px; font-size: 0.74rem; white-space: nowrap; flex-shrink: 0; display: flex; align-items: center; gap: 4px;">
                    <span>📋</span> Copiar
                  </button>
                </div>
              </div>

              <!-- ITEM 2: SELETOR DE MODO DE INTEGRAÇÃO -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px;">
                    <span>Modo de Integração com o Cofre</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Estratégia de Sincronização</strong>
                        Define se o cofre é gerenciado através do OneDrive para espelhamento móvel ou reside exclusivamente em diretório local do Windows.
                      </span>
                    </span>
                  </div>
                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 250px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                    <option value="onedrive" selected>OneDrive Sincronizado (Nuvem Pessoal)</option>
                    <option value="local_vault">Cofre Local (%USERPROFILE%\\\\Documents)</option>
                    <option value="custom">Diretório Customizado / Air-Gapped</option>
                  </select>
                </div>'''

new_card1_item1 = '''              <!-- ITEM 1: CAMINHO DO COFRE -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                  <div class="setting-title" style="gap: 6px; font-size: 0.78rem; font-weight: 600; color: #cbd5e1;">
                    <span>Diretório do Cofre de Conhecimentos</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Localização Física do Cofre</strong>
                        Pasta sincronizada via OneDrive ou armazenada no disco local do usuário.
                      </span>
                    </span>
                  </div>
                  <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ OneDrive Conectado
                  </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                  <input type="text" id="cfg-obsidian-path" class="setting-input" value="C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
                  <button class="btn btn-secondary" onclick="copyToClipboard(document.getElementById('cfg-obsidian-path').value, 'Caminho do Cofre')" title="Copiar caminho completo" style="padding: 7px 12px; font-size: 0.74rem; white-space: nowrap; flex-shrink: 0; display: flex; align-items: center; gap: 4px;">
                    <span>📋</span> Copiar
                  </button>
                </div>
              </div>

              <!-- ITEM 2: SELETOR DE MODO DE INTEGRAÇÃO -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                  <div class="setting-title" style="gap: 6px;">
                    <span>Modo de Integração com o Cofre</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Estratégia de Sincronização</strong>
                        Define se o cofre é gerenciado através do OneDrive para espelhamento móvel ou reside exclusivamente em diretório local do Windows.
                      </span>
                    </span>
                  </div>
                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 210px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                    <option value="onedrive" selected>OneDrive Sincronizado</option>
                    <option value="local_vault">Cofre Local (Documentos)</option>
                    <option value="custom">Diretório Customizado</option>
                  </select>
                </div>'''

if old_card1_item1 in content:
    content = content.replace(old_card1_item1, new_card1_item1, 1)
    print("Updated Card 1 Title & Select!")
else:
    print("Warning: old_card1_item1 not found!")

# 2. Card 3 Privacy Mode Select
old_card3_select = '''                  <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 220px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                      <option value="hybrid" selected>Híbrido Anonimizado (Recomendado)</option>
                      <option value="strict_local">Estrito Local (Air-Gapped Silício)</option>
                    </select>'''

new_card3_select = '''                  <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 210px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                      <option value="hybrid" selected>Híbrido Anonimizado (Padrão)</option>
                      <option value="strict_local">Estrito Local (Silício)</option>
                    </select>'''

if old_card3_select in content:
    content = content.replace(old_card3_select, new_card3_select, 1)
    print("Updated Card 3 Privacy Mode Select!")
else:
    print("Warning: old_card3_select not found!")

# 3. Populate default batch size 3
old_pop = "setInput('cfg-obsidian-batch-size', cfg.obsidian.curator_batch_size);"
new_pop = "setInput('cfg-obsidian-batch-size', cfg.obsidian.curator_batch_size || 3);"
if old_pop in content:
    content = content.replace(old_pop, new_pop, 1)
    print("Updated populateSettingsForm batch size default to 3!")
else:
    print("Warning: old_pop not found!")

# Ensure LF
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Saved fine-tuned dashboard_ui.html!")

