with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

target = '''              <!-- CAMINHO ATIVO & SELETOR DE ESTRATÉGIA -->
              <div style="margin-bottom: 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                  <span style="font-size: 0.72rem; font-weight: 600; color: #cbd5e1;">Diretório Raiz de Persistência:</span>
                  <span class="mono" style="font-size: 0.68rem; color: var(--green);">Permissão I/O: Leitura/Escrita</span>
                </div>
                <div style="display: flex; gap: 6px; align-items: center;">
                  <input type="text" id="cfg-storage-path" class="setting-input" value="Z:\\souls_engine\\.souls_data" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 10px; flex: 1;">
                  <button class="btn btn-micro" onclick="copyToClipboard(document.getElementById('cfg-storage-path').value, 'Caminho de dados')" title="Copiar caminho completo" style="padding: 7px 10px; font-size: 0.76rem;">
                    📋 Copiar
                  </button>
                </div>
              </div>'''

replacement = '''              <!-- ITEM 1: CAMINHO ATIVO & PERMISSÕES I/O -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px; font-size: 0.78rem; font-weight: 600; color: #cbd5e1;">
                    <span>Diretório Raiz de Persistência</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Diretório Canônico de Dados</strong>
                        Pasta base onde residem todos os bancos de dados SQLite STRICT, vetores LanceDB, grafo Ladybug e backups com confinamento estrito.
                      </span>
                    </span>
                  </div>
                  <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ I/O Leitura & Escrita
                  </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                  <input type="text" id="cfg-storage-path" class="setting-input" value="Z:\\souls_engine\\.souls_data" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
                  <button class="btn btn-secondary" onclick="copyToClipboard(document.getElementById('cfg-storage-path').value, 'Caminho de dados')" title="Copiar caminho completo" style="padding: 7px 12px; font-size: 0.74rem; white-space: nowrap; flex-shrink: 0; display: flex; align-items: center; gap: 4px;">
                    <span>📋</span> Copiar
                  </button>
                </div>
              </div>'''

if target in content:
    content = content.replace(target, replacement, 1)
    print("Replaced storage path row successfully!")
else:
    print("Warning: target not found!")

# Also ensure LF
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Saved dashboard_ui.html")

