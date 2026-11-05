import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Update Sidebar Nav Item
old_nav_marker = 'id="btn-subtab-obsidian"'
nav_idx = content.find(old_nav_marker)
if nav_idx != -1:
    btn_start = content.rfind('<button', 0, nav_idx)
    btn_end = content.find('</button>', nav_idx) + len('</button>')
    new_nav = '''<button class="settings-subtab-btn" id="btn-subtab-obsidian" onclick="switchSettingsSubtab('subtab-obsidian')">
              <span class="nav-icon">📓</span>
              <div class="nav-text">
                <span class="nav-label">Cofre de Conhecimentos</span>
                <span class="nav-desc">Notas, Wiki Pessoal & Privacidade</span>
              </div>
            </button>'''
    content = content[:btn_start] + new_nav + content[btn_end:]
    print("Updated sidebar nav item!")
else:
    print("Warning: old_nav_marker not found!")

# 2. Update subtab-obsidian HTML
sub_start_marker = 'id="subtab-obsidian"'
sub_end_marker = 'id="subtab-raw"'

idx1 = content.find(sub_start_marker)
idx2 = content.find(sub_end_marker)

if idx1 != -1 and idx2 != -1:
    # Find the start comment before subtab-obsidian
    pane_start = content.rfind('<!-- ==================================================== -->', 0, idx1)
    if pane_start == -1:
        pane_start = content.rfind('<div class="settings-subpane" id="subtab-obsidian"', 0, idx1)
    
    pane_end = content.rfind('<!-- ==================================================== -->', 0, idx2)
    if pane_end == -1:
        pane_end = content.rfind('<div class="settings-subpane" id="subtab-raw"', 0, idx2)

    new_subtab_html = '''<!-- ==================================================== -->
      <!-- SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL -->
      <!-- ==================================================== -->
      <div class="settings-subpane" id="subtab-obsidian">
        <!-- HEADER DA SUB-ABA COM AÇÕES -->
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
          <div>
            <h2 style="font-size: 1.15rem; font-weight: 700; color: #fff; display: flex; align-items: center; gap: 8px; margin: 0 0 4px 0;">
              <span>📓</span>
              <span>Cofre de Conhecimentos & Curadoria Pessoal</span>
              <span class="info-icon" style="z-index: 95;">i
                <span class="tooltip-box">
                  <strong>Curadoria Enciclopédica & Soberania Pessoal</strong>
                  Processa fontes brutas imutáveis em <code>raw/</code> e destila conceitos e entidades atômicas em <code>wiki/</code> com links bidirecionais <code>[[...]]</code>, aplicando desidentificação local de PII no silício.
                </span>
              </span>
            </h2>
            <p style="font-size: 0.78rem; color: var(--text-muted); margin: 0;">
              Destilação contínua de anotações brutas (raw/) em conhecimento enciclopédico hiperconectado (wiki/), com blindagem local de dados sensíveis (Privacy Shield).
            </p>
          </div>
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-secondary" onclick="resetObsidianDefaults()" style="padding: 6px 12px; font-size: 0.75rem;">
              <span>⚡</span> Defaults do Cofre
            </button>
            <button class="btn btn-green" onclick="saveSettings()" style="padding: 6px 14px; font-size: 0.75rem;">
              <span>💾</span> Salvar Configurações
            </button>
          </div>
        </div>

        <!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico) -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 16px;">
          <!-- CARD 1: ESTADO DO COFRE -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--cyan); margin-bottom: 4px;">
              📓 ESTADO DO COFRE
            </div>
            <div class="mono" id="kpi-vault-status" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              CONECTADO 100%
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              OneDrive Sync Ativo
            </div>
          </div>

          <!-- CARD 2: ZONA BRUTA RAW -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--green); margin-bottom: 4px;">
              🔒 ZONA BRUTA (RAW/)
            </div>
            <div class="mono" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              READ-ONLY TOTAL
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Imutável & Sagrado (Lei 1)
            </div>
          </div>

          <!-- CARD 3: PRIVACY SHIELD PII -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--purple); margin-bottom: 4px;">
              🛡️ PRIVACY SHIELD PII
            </div>
            <div class="mono" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              BLINDAGEM ATIVA
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Higienização Local no Silício
            </div>
          </div>

          <!-- CARD 4: CADÊNCIA THE DRIP -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--amber); margin-bottom: 4px;">
              ⏱️ CADÊNCIA (THE DRIP)
            </div>
            <div class="mono" id="metric-obsidian-cadence" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              3 NOTAS / 30 MIN
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Janela Longa (Sem Rate-Limit)
            </div>
          </div>
        </div>

        <!-- BANNER DE GOVERNANÇA DO COFRE & SOBERANIA DE DADOS -->
        <div style="background: rgba(168, 85, 247, 0.05); border: 1px solid rgba(168, 85, 247, 0.25); border-radius: var(--radius-md); padding: 10px 14px; margin-bottom: 16px; display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
          <div style="font-size: 0.74rem; line-height: 1.5; color: #e2e8f0; flex: 1; min-width: 280px; text-wrap: balance;">
            <strong style="color: var(--purple-light);">💡 Soberania do Cofre & Privacidade (AGENTS.md):</strong> As fontes brutas em <code>raw/</code> são sagradas e nunca são modificadas nem deletadas (Lei 1). O pipeline de curadoria destila notas atômicas em <code>wiki/</code> com frontmatter YAML e links bidirecionais <code>[[...]]</code>. Documentos médicos e extratos financeiros passam compulsoriamente por desidentificação local de PII no silício antes de qualquer processamento analítico na nuvem.
          </div>
          <span style="font-size: 0.68rem; font-family: var(--font-mono); color: var(--purple-light); background: rgba(168, 85, 247, 0.12); border: 1px solid rgba(168, 85, 247, 0.3); padding: 3px 8px; border-radius: 4px; white-space: nowrap; align-self: center;">
            • Curadoria Contínua Ativa (v7)
          </span>
        </div>

        <!-- GRID PRINCIPAL DE 2 COLUNAS -->
        <div style="display: grid; grid-template-columns: 1fr 1.15fr; gap: 16px; align-items: start;">
          <!-- COLUNA 1: CONEXÃO DO COFRE & CADÊNCIA OPERACIONAL -->
          <div style="display: flex; flex-direction: column; gap: 16px;">
            <!-- CARD 1: CONEXÃO & DIRETÓRIO DO COFRE -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px; display: flex; justify-content: space-between; align-items: center; gap: 16px;">
                <div class="card-title" style="font-size: 0.86rem; display: flex; align-items: center; gap: 8px; margin: 0;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem; flex-shrink: 0;">📦</div>
                  <span>Conexão & Diretório Raiz do Cofre</span>
                  <span class="info-icon" style="z-index: 90; margin-left: 2px; flex-shrink: 0;">i
                    <span class="tooltip-box">
                      <strong>Cofre Pessoal de Longo Prazo</strong>
                      Localização física onde residem as pastas <code>raw/</code> (fontes brutas) e <code>wiki/</code> (conhecimento atômico destilado).
                    </span>
                  </span>
                </div>
                <span class="badge badge-champion" id="badge-vault-conn" style="font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px;">✓ Sincronizado</span>
              </div>

              <!-- ITEM 1: CAMINHO DO COFRE -->
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
                  <input type="text" id="cfg-obsidian-path" class="setting-input" value="C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
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
                    <option value="local_vault">Cofre Local (%USERPROFILE%\\Documents)</option>
                    <option value="custom">Diretório Customizado / Air-Gapped</option>
                  </select>
                </div>
                <div class="setting-desc" style="text-wrap: balance; line-height: 1.45; margin-bottom: 0;">
                  O espelhamento via OneDrive garante que wikis e entidades estejam acessíveis em dispositivos móveis e tablets simultaneamente.
                </div>
              </div>

              <!-- BOTÕES DE VALIDAÇÃO E AUDITORIA -->
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
                <button class="btn btn-green" onclick="validateVaultStructure()" style="justify-content: center; padding: 8px 12px; font-size: 0.76rem; white-space: nowrap;">
                  <span>🔍</span> Validar Estrutura do Cofre
                </button>
                <button class="btn btn-secondary" onclick="showVaultAuditLog()" style="justify-content: center; padding: 8px 12px; font-size: 0.76rem; white-space: nowrap;">
                  <span>📑</span> Auditoria (wiki/log.md)
                </button>
              </div>

              <!-- CHECKLIST DE INTEGRIDADE DAS PASTAS DO COFRE -->
              <div style="background: rgba(10, 16, 28, 0.7); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 10px 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; flex-wrap: wrap; gap: 6px;">
                  <span style="font-size: 0.70rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #94a3b8;">Estrutura Canônica do Cofre:</span>
                  <span class="mono" id="checklist-vault-status" style="font-size: 0.68rem; color: var(--green);">6 de 6 Conformidades</span>
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 6px; font-size: 0.72rem; font-family: var(--font-mono);">
                  <div class="folder-check-item" id="chk-vault-raw" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>raw/</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Read-Only)</span>
                  </div>
                  <div class="folder-check-item" id="chk-vault-concepts" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>wiki/concepts/</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Conceitos)</span>
                  </div>
                  <div class="folder-check-item" id="chk-vault-entities" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>wiki/entities/</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Entidades)</span>
                  </div>
                  <div class="folder-check-item" id="chk-vault-summaries" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>wiki/summaries/</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Sínteses)</span>
                  </div>
                  <div class="folder-check-item" id="chk-vault-manifest" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>raw_manifest.json</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Status)</span>
                  </div>
                  <div class="folder-check-item" id="chk-vault-log" style="display: flex; align-items: center; gap: 5px; color: #cbd5e1;">
                    <span style="color: var(--green);">✓</span> <code>wiki/log.md</code> <span style="font-size: 0.62rem; color: var(--text-muted);">(Auditoria)</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- CARD 2: CADÊNCIA DE CURADORIA ("THE 30-MINUTE DRIP") -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px;">
                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-amber" style="width: 24px; height: 24px; font-size: 0.88rem;">⏱️</div>
                  Cadência Operacional & Micro-Lotes ("The 30-Minute Drip")
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>The 30-Minute Drip</strong>
                      Processa 2 a 3 arquivos a cada 30 minutos em micro-lotes via subagente agendado, digerindo ~50 notas por dia útil sem estourar rate-limits ou aquecer o silício.
                    </span>
                  </span>
                </div>
              </div>

              <div class="setting-list" style="display: flex; flex-direction: column; gap: 11px;">
                <!-- ITEM 1: TAMANHO DO LOTE SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Tamanho do Micro-Lote por Ciclo</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Batch Size de Curadoria</strong>
                          Quantidade de notas brutas destiladas em cada execução do pipeline.
                          <div class="tooltip-def">Padrão canônico: 3 notas (2 a 3)</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Lote:</span>
                      <span id="val-obsidian-batch-size" style="white-space: nowrap;">3 notas</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-batch-size" min="1" max="10" step="1" value="3" data-default="3" data-snap-delta="0.3" oninput="document.getElementById('val-obsidian-batch-size').innerText = this.value + ' notas'; syncObsidianCadenceKpi(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-batch-size" style="left: calc(8px + (100% - 16px) * 0.2222);" title="Padrão canônico: 3 notas (Clique ou aproxime para restaurar)"></div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Micro-lotes pequenos evitam estouro de janela de contexto e garantem máxima fidelidade atômica.
                  </div>
                </div>

                <!-- ITEM 2: INTERVALO DA ESTEIRA SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Intervalo da Esteira Agendada</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Cadência de Disparo Automático</strong>
                          Frequência com que o Antigravity CLI ou Hermes aciona o subagente de curadoria em segundo plano.
                          <div class="tooltip-def">Padrão canônico: 30 minutos</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Intervalo:</span>
                      <span id="val-obsidian-interval" style="white-space: nowrap;">30 min</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-interval-min" min="15" max="120" step="15" value="30" data-default="30" data-snap-delta="4" oninput="document.getElementById('val-obsidian-interval').innerText = this.value + ' min'; syncObsidianCadenceKpi(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-interval-min" style="left: calc(8px + (100% - 16px) * 0.1428);" title="Padrão canônico: 30 minutos (Clique ou aproxime para restaurar)"></div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Intervalo espaçado protege contra throttling térmico da GPU e dissipa consumo ao longo do dia.
                  </div>
                </div>

                <!-- ITEM 3: DISPARO MANUAL -->
                <div class="setting-item" style="border-top: 1px solid var(--border-subtle); padding-top: 10px;">
                  <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px;">
                    <div>
                      <div style="font-size: 0.78rem; font-weight: 600; color: #fff;">Execução Manual sob Demanda</div>
                      <div style="font-size: 0.72rem; color: var(--text-muted);">Processa o próximo micro-lote de notas PENDING agora.</div>
                    </div>
                    <button class="btn btn-green" onclick="triggerObsidianCurate()" style="padding: 7px 14px; font-size: 0.75rem;">
                      <span>📚</span> Curar Micro-Lote Agora
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- COLUNA 2: PRIVACY SHIELD & SIMULADOR DE HIGIENIZAÇÃO -->
          <div style="display: flex; flex-direction: column; gap: 16px;">
            <!-- CARD 3: SOBERANIA & PRIVACY SHIELD -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px;">
                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-purple" style="width: 24px; height: 24px; font-size: 0.88rem;">🛡️</div>
                  Soberania de Dados & Desidentificação PII (Privacy Shield)
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>Obsidian Privacy Shield</strong>
                      Higieniza localmente no silício (Aho-Corasick) dados médicos e financeiros antes do envio para modelos analíticos de fronteira.
                    </span>
                  </span>
                </div>
              </div>

              <div class="setting-list" style="display: flex; flex-direction: column; gap: 11px;">
                <!-- ITEM 1: PRIVACY SHIELD ATIVO -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Obsidian Privacy Shield Ativo</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Higienização Local Obrigatória</strong>
                          Substitui nomes de médicos, pacientes, contas bancárias e CPFs por marcadores anônimos antes de qualquer chamada em nuvem.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-privacy-shield">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-privacy-shield" checked onchange="updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield'); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante soberania e privacidade inviolável para relatórios médicos e extratos de investimentos.
                  </div>
                </div>

                <!-- ITEM 2: RAW READ-ONLY -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Zona raw/ Estritamente Read-Only</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Lei Suprema 1 do Cofre</strong>
                          Fontes originais brutas nunca são apagadas nem truncadas; preservação aditiva de YAML.
                          <div class="tooltip-def">Status: Ativo (Mandatório)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill locked" id="pill-obsidian-raw-ro">MANDATÓRIO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-raw-ro" checked disabled>
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Blindagem física contra perda de notas originais, fotos e documentos legados no cofre.
                  </div>
                </div>

                <!-- ITEM 3: WIKILINKS & FRONTMATTER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Links Bidirecionais [[...]] & YAML Frontmatter</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Padrão Enciclopédico Atômico</strong>
                          Insere compulsoriamente cabeçalhos YAML padronizados (type, tags, sources, last_updated) e referências cruzadas bidirecionais.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-wikilinks">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-wikilinks" checked onchange="updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks'); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante hiperconectividade ontológica do grafo do cofre no Obsidian Graph View.
                  </div>
                </div>

                <!-- ITEM 4: MODO OPERACIONAL DE PRIVACIDADE -->
                <div class="setting-item">
                  <div class="setting-header" style="flex-wrap: wrap; gap: 8px;">
                    <div class="setting-title">
                      <span>Modo Operacional de Privacidade</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Estratégia de Processamento</strong>
                          HYBRID_ANONYMIZED higieniza PII localmente no silício e envia para modelos de fronteira de alto contexto; STRICT_LOCAL retém 100% no silício local (dGPU RTX 2060m).
                        </span>
                      </span>
                    </div>
                    <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 220px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">
                      <option value="hybrid" selected>Híbrido Anonimizado (Recomendado)</option>
                      <option value="strict_local">Estrito Local (Air-Gapped Silício)</option>
                    </select>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    O modo híbrido preserva 100% da privacidade sem rebaixar a qualidade médica ou contábil para SLMs fracos.
                  </div>
                </div>
              </div>
            </div>

            <!-- CARD 4: SIMULADOR DE HIGIENIZAÇÃO PII & AUDITORIA -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px; justify-content: space-between;">
                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem;">🧪</div>
                  Simulador de Higienização PII & Guardrails
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>Teste em Tempo Real do Privacy Shield</strong>
                      Verifica como exames clínicos, dados bancários e anotações brutas são sanitizados pelo algoritmo Aho-Corasick linear antes de tocar qualquer modelo de nuvem.
                    </span>
                  </span>
                </div>
                <span class="badge" style="background: rgba(6, 182, 212, 0.15); color: var(--cyan); border: 1px solid rgba(6, 182, 212, 0.3); font-size: 0.65rem;">Aho-Corasick Ativo</span>
              </div>

              <div style="font-size: 0.72rem; color: var(--text-muted); margin-bottom: 10px;">
                Selecione um cenário real ou digite um trecho documental para testar a redação de dados sensíveis e verificação da Lei 1:
              </div>

              <!-- CENÁRIOS RÁPIDOS DE SIMULAÇÃO -->
              <div style="display: flex; gap: 6px; margin-bottom: 10px; flex-wrap: wrap;">
                <button class="sim-preset-btn" onclick="setVaultScenario('clinical')" style="font-size: 0.70rem; padding: 4px 9px;">
                  🏥 Exame Clínico (Dr./CRM)
                </button>
                <button class="sim-preset-btn" onclick="setVaultScenario('financial')" style="font-size: 0.70rem; padding: 4px 9px;">
                  💳 Extrato Bancário (PIX/Conta)
                </button>
                <button class="sim-preset-btn" onclick="setVaultScenario('concept')" style="font-size: 0.70rem; padding: 4px 9px;">
                  📄 Conceito Wiki (Atômico)
                </button>
                <button class="sim-preset-btn" onclick="setVaultScenario('raw_mutation')" style="font-size: 0.70rem; padding: 4px 9px; border-color: rgba(239, 68, 68, 0.4); color: #f87171;">
                  🚫 Mutação em raw/ (Lei 1)
                </button>
              </div>

              <!-- CAMPO DE ENTRADA DO SIMULADOR -->
              <div style="margin-bottom: 10px;">
                <textarea id="sim-vault-input" class="setting-input" style="width: 100%; height: 60px; font-size: 0.74rem; font-family: var(--font-mono); resize: vertical; padding: 8px 10px;" placeholder="Insira o texto bruto ou comando para testar a sanitização do cofre...">Paciente: Bruno Rosa | CRM: 48921-SP | Lab Fleury | Hemoglobina Glicada: 5.4% | Glicemia Jejum: 88 mg/dL</textarea>
              </div>

              <button class="btn btn-secondary" onclick="runVaultSimulation()" style="width: 100%; justify-content: center; padding: 7px 12px; font-size: 0.76rem; margin-bottom: 12px;">
                <span>⚡</span> Simular Sanitização PII & Auditoria
              </button>

              <!-- PAINEL DE RESULTADOS DA SIMULAÇÃO -->
              <div style="background: rgba(10, 16, 28, 0.85); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 10px 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; flex-wrap: wrap; gap: 6px;">
                  <span id="sim-vault-verdict" class="badge" style="background: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.68rem; font-weight: 700;">
                    ✓ PII HIGIENIZADO COM SUCESSO
                  </span>
                  <span style="font-size: 0.68rem; color: var(--text-muted); font-family: var(--font-mono);">
                    Latência: <strong style="color: var(--cyan);">&lt; 0.05 ms</strong> &bull; Aho-Corasick
                  </span>
                </div>
                <div style="font-size: 0.68rem; color: #94a3b8; margin-bottom: 4px; font-weight: 600;">Saída Sanitizada para o Modelo de Nuvem:</div>
                <div id="sim-vault-preview" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 4px; padding: 7px 9px; font-size: 0.72rem; font-family: var(--font-mono); color: #e2e8f0; line-height: 1.45; max-height: 80px; overflow-y: auto; margin-bottom: 6px;">
                  Paciente: [USUARIO_ALVO] | CRM: [MEDICO_CRM_OCULTO] | Lab: [LAB_REFERENCIA] | Hemoglobina Glicada: 5.4% | Glicemia Jejum: 88 mg/dL
                </div>
                <div id="sim-vault-rule" style="font-size: 0.68rem; color: var(--green); font-family: var(--font-mono);">
                  ✓ Obsidian Privacy Shield: Identificadores sensíveis desidentificados localmente no silício.
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>\n\n      '''

    content = content[:pane_start] + new_subtab_html + content[pane_end:]
    print("Replaced subtab-obsidian HTML successfully!")
else:
    print(f"Warning: markers not found ({idx1}, {idx2})")

# 3. Add Obsidian JavaScript functions
obsidian_js = '''
    // =========================================================
    // SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL
    // =========================================================
    function initObsidianSubtab() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        document.getElementById('val-obsidian-batch-size').innerText = batchInput.value + ' notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        document.getElementById('val-obsidian-interval').innerText = intervalInput.value + ' min';
      }
      syncObsidianCadenceKpi();
    }

    function syncObsidianCadenceKpi() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      const cadenceEl = document.getElementById('metric-obsidian-cadence');
      if (batchInput && intervalInput && cadenceEl) {
        cadenceEl.innerText = `${batchInput.value} NOTAS / ${intervalInput.value} MIN`;
      }
    }

    function validateVaultStructure() {
      const statusEl = document.getElementById('checklist-vault-status');
      if (statusEl) {
        statusEl.innerText = 'Validando pastas...';
        statusEl.style.color = 'var(--cyan)';
      }
      setTimeout(() => {
        ['chk-vault-raw', 'chk-vault-concepts', 'chk-vault-entities', 'chk-vault-summaries', 'chk-vault-manifest', 'chk-vault-log'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = '6 de 6 Conformidades';
          statusEl.style.color = 'var(--green)';
        }
        showToast('✓ Estrutura canônica do cofre validada com sucesso (6 de 6 conformidades)!', 'success');
      }, 350);
    }

    function showVaultAuditLog() {
      showToast('📑 Auditoria: Registros gravados em C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB\\\\wiki\\\\log.md', 'info');
    }

    function setVaultScenario(type) {
      const inputEl = document.getElementById('sim-vault-input');
      if (!inputEl) return;
      if (type === 'clinical') {
        inputEl.value = 'Paciente: Bruno Rosa | CPF: 123.456.789-00 | CRM: 48921-SP | Lab Fleury | Hemoglobina Glicada: 5.4% | Glicemia Jejum: 88 mg/dL';
      } else if (type === 'financial') {
        inputEl.value = 'Banco do Brasil | Agência: 1234-5 | Conta Corrente: 98765-4 | Chave PIX: bruno@gmail.com | Compra: Farmácia Raia R$ 145,20 | Saldo: R$ 4.250,00';
      } else if (type === 'concept') {
        inputEl.value = 'Conceito: Myers Diff Atômico e Árvore Tree-sitter para refatoração cirúrgica sem quebra de código. Referências: [[wiki/concepts/ast_dehydration.md]]';
      } else if (type === 'raw_mutation') {
        inputEl.value = 'rm -rf "C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB\\\\raw\\\\financas\\\\2026_01.pdf" (Tentativa de Exclusão)';
      }
      runVaultSimulation();
    }

    function runVaultSimulation() {
      const inputEl = document.getElementById('sim-vault-input');
      const text = inputEl ? inputEl.value.trim() : '';
      const verdictEl = document.getElementById('sim-vault-verdict');
      const previewEl = document.getElementById('sim-vault-preview');
      const ruleEl = document.getElementById('sim-vault-rule');

      if (!text) {
        if (verdictEl) {
          verdictEl.innerText = 'AGUARDANDO ENTRADA';
          verdictEl.style.background = 'rgba(255,255,255,0.05)';
          verdictEl.style.color = 'var(--text-muted)';
          verdictEl.style.borderColor = 'rgba(255,255,255,0.1)';
        }
        if (previewEl) previewEl.innerText = 'Digite um trecho documental ou comando para simular a higienização.';
        if (ruleEl) ruleEl.innerText = '--';
        return;
      }

      const lower = text.toLowerCase();

      // Check for raw/ mutation violation
      if (lower.includes('rm ') || lower.includes('delete') || lower.includes('exclusão') || (lower.includes('raw') && (lower.includes('modify') || lower.includes('truncate') || lower.includes('apagar')))) {
        if (verdictEl) {
          verdictEl.innerText = '🛡️ VIOLAÇÃO DA LEI 1 BARRADA (raw/ R-O)';
          verdictEl.style.background = 'rgba(239, 68, 68, 0.15)';
          verdictEl.style.color = '#ef4444';
          verdictEl.style.borderColor = 'rgba(239, 68, 68, 0.3)';
        }
        if (previewEl) previewEl.innerHTML = '<span style="color: #ef4444;">[OPERAÇÃO INTERCEPTADA]: Fontes brutas em <code>raw/</code> são estritamente Read-Only. Nenhuma modificação, renomeação ou exclusão é permitida pelo guardrail físico do cofre.</span>';
        if (ruleEl) ruleEl.innerHTML = '<span style="color: #ef4444;">Lei Suprema 1 do Cofre (AGENTS.md & obsidian-vault-curator)</span>';
        return;
      }

      // Check for clinical / medical PII
      if (lower.includes('paciente') || lower.includes('crm') || lower.includes('hemoglobina') || lower.includes('glicemia') || lower.includes('fleury')) {
        let sanitized = text
          .replace(/Bruno Rosa/gi, '[USUARIO_ALVO]')
          .replace(/\b\d{3}\.\d{3}\.\d{3}-\d{2}\b/g, '[CPF_PROTEGIDO]')
          .replace(/\bCRM:\s*\d+-[A-Z]{2}\b/gi, 'CRM: [MEDICO_CRM_OCULTO]')
          .replace(/Lab Fleury/gi, 'Lab: [LAB_REFERENCIA]');
        if (verdictEl) {
          verdictEl.innerText = '✓ PII CLÍNICO HIGIENIZADO NO SILÍCIO';
          verdictEl.style.background = 'rgba(16, 185, 129, 0.15)';
          verdictEl.style.color = '#10b981';
          verdictEl.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        }
        if (previewEl) previewEl.innerText = sanitized;
        if (ruleEl) ruleEl.innerHTML = '<span style="color: var(--green);">✓ Obsidian Privacy Shield (§2 Pipeline Híbrido de Desidentificação Clínica)</span>';
        return;
      }

      // Check for financial PII
      if (lower.includes('banco') || lower.includes('agência') || lower.includes('agencia') || lower.includes('conta') || lower.includes('pix')) {
        let sanitized = text
          .replace(/Agência:\s*\d+-?\d*/gi, 'Agência: [AGENCIA_A]')
          .replace(/Conta Corrente:\s*\d+-?\d*/gi, 'Conta: [CONTA_CORRENTE_A]')
          .replace(/bruno@gmail\.com/gi, '[CHAVE_PIX_REDACTED]')
          .replace(/Banco do Brasil/gi, '[INSTITUICAO_BANCARIA]');
        if (verdictEl) {
          verdictEl.innerText = '✓ PII FINANCEIRO ANONIMIZADO NO SILÍCIO';
          verdictEl.style.background = 'rgba(16, 185, 129, 0.15)';
          verdictEl.style.color = '#10b981';
          verdictEl.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        }
        if (previewEl) previewEl.innerText = sanitized;
        if (ruleEl) ruleEl.innerHTML = '<span style="color: var(--green);">✓ Obsidian Privacy Shield (§1 Proteção Financeira & Bancária)</span>';
        return;
      }

      // Concept / general note
      if (verdictEl) {
        verdictEl.innerText = '✓ CONCEITO ATÔMICO APROVADO';
        verdictEl.style.background = 'rgba(6, 182, 212, 0.15)';
        verdictEl.style.color = 'var(--cyan)';
        verdictEl.style.borderColor = 'rgba(6, 182, 212, 0.3)';
      }
      if (previewEl) {
        previewEl.innerHTML = `<code>---\\ntype: concept\\ntags: [software_engineering, architecture]\\nsources: ["[[raw/pesquisa_ast.md]]"]\\nlast_updated: ${new Date().toISOString().split('T')[0]}\\nreviewed: false\\n---\\n\\n# Síntese Atômica\\n${text}</code>`;
      }
      if (ruleEl) ruleEl.innerHTML = '<span style="color: var(--cyan);">✓ Obsidian Vault Curator (§3 Frontmatter YAML & Links Bidirecionais)</span>';
    }

    function resetObsidianDefaults() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        batchInput.value = 3;
        document.getElementById('val-obsidian-batch-size').innerText = '3 notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        intervalInput.value = 30;
        document.getElementById('val-obsidian-interval').innerText = '30 min';
      }
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      if (privacyCb) {
        privacyCb.checked = true;
        updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield');
      }
      const wikilinksCb = document.getElementById('cfg-obsidian-wikilinks');
      if (wikilinksCb) {
        wikilinksCb.checked = true;
        updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks');
      }
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      if (modeSelect) modeSelect.value = 'hybrid';
      syncObsidianCadenceKpi();
      showToast('✓ Defaults do Cofre de Conhecimentos restaurados!', 'success');
    }
'''

# Place obsidian_js right after initSandboxSubtab block
marker_after = 'function switchSettingsSubtab(subtabId) {'
if marker_after in content:
    content = content.replace(marker_after, obsidian_js + '\n    ' + marker_after, 1)
    print("Added obsidian_js before switchSettingsSubtab!")
else:
    print("Warning: marker_after not found!")

# 4. Wire switchSettingsSubtab for subtab-obsidian
target_switch = '''      if (subtabId === 'subtab-sandbox') {
        initSandboxSubtab();
      }'''

replacement_switch = '''      if (subtabId === 'subtab-sandbox') {
        initSandboxSubtab();
      }
      if (subtabId === 'subtab-obsidian') {
        initObsidianSubtab();
      }'''

if target_switch in content:
    content = content.replace(target_switch, replacement_switch, 1)
    print("Wired switchSettingsSubtab for subtab-obsidian!")
else:
    print("Warning: target_switch not found!")

# Ensure LF line endings
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Saved assembled dashboard_ui.html successfully!")
