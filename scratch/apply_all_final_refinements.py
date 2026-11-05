# -*- coding: utf-8 -*-
import sys
import re

def main():
    file_path = "crates/souls_server/src/dashboard_ui.html"
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    # =========================================================================
    # PART 1: OBSIDIAN SUBTAB REFINEMENTS (Ponto #1 and Ponto #2)
    # =========================================================================

    # 1.1 Pin #1: Motor de Sanitização PII Header - separate (i) and badge
    old_sanitize_header = '''                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem;">🛡️</div>
                  Motor de Sanitização PII & Auditoria no Silício (Live Engine)
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>Inspeção Bare-Metal em Tempo Real</strong>
                      Executa varredura de PII e verificação de Lei 1 diretamente no daemon Rust (127.0.0.1:9123). Redige dados pessoais e protege notas brutas antes de qualquer tráfego para modelos de nuvem.
                    </span>
                  </span>
                </div>
                <span class="badge" id="badge-vault-engine-status" style="background: rgba(16, 185, 129, 0.15); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem;">Rust Bare-Metal Ativo</span>'''

    new_sanitize_header = '''                <div class="card-title" style="font-size: 0.86rem; display: flex; align-items: center; gap: 8px; margin: 0; flex: 1; min-width: 0;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem; flex-shrink: 0;">🛡️</div>
                  <span style="font-weight: 700;">Motor de Sanitização PII & Auditoria no Silício (Live Engine)</span>
                  <span class="info-icon" style="z-index: 90; margin-left: 4px; flex-shrink: 0;">i
                    <span class="tooltip-box">
                      <strong>Inspeção Bare-Metal em Tempo Real</strong>
                      Executa varredura de PII e verificação de Lei 1 diretamente no daemon Rust (127.0.0.1:9123). Redige dados pessoais e protege notas brutas antes de qualquer tráfego para modelos de nuvem.
                    </span>
                  </span>
                </div>
                <span class="badge" id="badge-vault-engine-status" style="background: rgba(16, 185, 129, 0.15); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; margin-left: 16px; white-space: nowrap; flex-shrink: 0;">Rust Bare-Metal Ativo</span>'''

    assert old_sanitize_header in content, "old_sanitize_header not found"
    content = content.replace(old_sanitize_header, new_sanitize_header, 1)
    print("✓ Refined Sanitize Header (Pin #1: (i) and badge separated)")

    # 1.2 Pin #2: Diretório do Cofre - breathing space and cloud icon
    old_path_row = '''              <!-- ITEM 1: CAMINHO DO COFRE -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px; font-size: 0.78rem; font-weight: 600; color: #cbd5e1;">
                    <span>Diretório do Cofre de Conhecimentos</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Localização Física do Cofre</strong>
                        Pasta sincronizada via OneDrive ou armazenada no disco local do usuário.
                      </span>
                    </span>
                  </div>
                  <span class="badge" id="badge-vault-path-type" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ OneDrive Conectado
                  </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                  <input type="text" id="cfg-obsidian-path" class="setting-input" value="C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
                  <button class="btn btn-secondary" onclick="copyToClipboard(document.getElementById('cfg-obsidian-path').value, 'Caminho do Cofre')" title="Copiar caminho completo" style="padding: 7px 12px; font-size: 0.74rem; white-space: nowrap; flex-shrink: 0; display: flex; align-items: center; gap: 4px;">
                    <span>📋</span> Copiar
                  </button>
                </div>
              </div>'''

    new_path_row = '''              <!-- ITEM 1: CAMINHO DO COFRE (Com respiro generoso e nuvemzinha OneDrive) -->
              <div class="setting-item" style="margin: 12px 0 20px 0; padding: 12px 14px; background: rgba(15, 23, 42, 0.4); border: 1px solid rgba(255, 255, 255, 0.05); border-radius: var(--radius-sm);">
                <div class="setting-header" style="margin-bottom: 8px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px; font-size: 0.78rem; font-weight: 600; color: #cbd5e1;">
                    <span>Diretório do Cofre de Conhecimentos</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Localização Física do Cofre</strong>
                        Pasta sincronizada via OneDrive ou armazenada no disco local do usuário.
                      </span>
                    </span>
                  </div>
                  <span class="badge" id="badge-vault-path-type" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ☁️ OneDrive Conectado
                  </span>
                </div>
                <div style="display: flex; gap: 8px; align-items: center;">
                  <input type="text" id="cfg-obsidian-path" class="setting-input" value="C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB" readonly style="color: #fff; font-size: 0.76rem; padding: 7px 12px; flex: 1; min-width: 0;">
                  <button class="btn btn-secondary" onclick="copyToClipboard(document.getElementById('cfg-obsidian-path').value, 'Caminho do Cofre')" title="Copiar caminho completo" style="padding: 7px 12px; font-size: 0.74rem; white-space: nowrap; flex-shrink: 0; display: flex; align-items: center; gap: 4px;">
                    <span>📋</span> Copiar
                  </button>
                </div>
              </div>'''

    assert old_path_row in content, "old_path_row not found"
    content = content.replace(old_path_row, new_path_row, 1)
    print("✓ Refined Path Row (Pin #2: breathing space and cloud icon added)")

    # 1.3 Sync JavaScript for cloud icon
    old_js_sync = "pathBadge.innerText = '✓ OneDrive Conectado';"
    new_js_sync = "pathBadge.innerText = '☁️ OneDrive Conectado';"
    assert old_js_sync in content, "old_js_sync not found"
    content = content.replace(old_js_sync, new_js_sync, 1)

    old_js_local = "pathBadge.innerText = '✓ Cofre Local Ativo';"
    new_js_local = "pathBadge.innerText = '📁 Cofre Local Ativo';"
    if old_js_local in content:
        content = content.replace(old_js_local, new_js_local, 1)

    # =========================================================================
    # PART 2: MASTER BRAIN & MODELOS (3 cards per row, card order, arrow fix)
    # =========================================================================

    # 2.1 CSS for inspecting-pill (never break arrow)
    old_pill_css = """.inspecting-pill {
      font-size: 0.6rem;
      font-weight: 600;
      color: var(--cyan);
      background: rgba(6, 182, 212, 0.15);
      border: 1px solid rgba(6, 182, 212, 0.35);
      border-radius: var(--radius-full);
      padding: 1px 7px;
      display: inline-block;
      margin-left: 6px;
    }"""

    new_pill_css = """.inspecting-pill {
      font-size: 0.64rem;
      font-weight: 600;
      color: var(--cyan);
      background: rgba(6, 182, 212, 0.15);
      border: 1px solid rgba(6, 182, 212, 0.35);
      border-radius: var(--radius-full);
      padding: 2px 9px;
      display: inline-flex;
      align-items: center;
      gap: 5px;
      white-space: nowrap;
      flex-shrink: 0;
      line-height: 1.2;
    }"""

    assert old_pill_css in content, "old_pill_css not found"
    content = content.replace(old_pill_css, new_pill_css, 1)
    print("✓ Updated .inspecting-pill CSS to prevent wrapping")

    # 2.2 Grid definition in HTML: exactly 3 cards per row
    old_grid_html = '<div class="presets-grid" id="profiles-cards-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(235px, 1fr)); gap: 14px; margin-bottom: 20px;">'
    new_grid_html = '<div class="presets-grid" id="profiles-cards-grid" style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; margin-bottom: 24px;">'

    assert old_grid_html in content, "old_grid_html not found"
    content = content.replace(old_grid_html, new_grid_html, 1)
    print("✓ Changed #profiles-cards-grid to exactly 3 columns")

    # 2.3 renderProfileCards JS: 1st Master Brain, 2nd Novo Perfil, then others
    old_render_fn_target = """    function renderProfileCards() {
      const grid = document.getElementById('profiles-cards-grid');
      if (!grid) return;

      let html = '';
      for (const p of profilesData) {
        const isActive = (p.id === activePreset);
        const isMaster = (p.role === 'orchestrator');
        const roleClass = isMaster ? 'master' : 'subagent';
        const roleLabel = isMaster ? '👑 Master Brain' : '🤖 Subagente';

        const deleteBtn = !p.is_builtin ? `
          <button class="btn-delete-profile" title="Excluir Perfil de Trabalho" onclick="event.stopPropagation(); deleteCustomProfile('${p.id}')">
            &times;
          </button>
        ` : '';

        const activeHermesBadge = p.is_active ? `
          <span class="active-hermes-pill">Ativo no Hermes</span>
        ` : '';

        const inspectingBadge = isActive ? `
          <span class="inspecting-pill"><span>Inspecionando</span> <span style="font-size: 0.70rem;">➔</span></span>
        ` : '';

        html += `
          <div class="preset-card ${isActive ? 'active' : ''}" id="preset-${p.id}" onclick="selectProfilePreset('${p.id}')" style="min-height: 185px; display: flex; flex-direction: column; justify-content: space-between;">
            <div>
              ${deleteBtn}
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                <span class="role-badge ${roleClass}">${roleLabel}</span>
                <span class="category-tag">${p.arena_category}</span>
              </div>
              <div class="preset-title" style="display: flex; justify-content: space-between; align-items: flex-start; gap: 8px; margin-bottom: 6px;">
                <span style="font-weight: 700; font-size: 0.90rem; color: #fff; line-height: 1.3;">${p.name}</span>
                ${inspectingBadge}
              </div>
              <div class="preset-desc" style="font-size: 0.74rem; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">${p.description}</div>
            </div>

            <div class="preset-defined-box" style="margin-top: 10px; background: rgba(0,0,0,0.25); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 8px 10px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 6px; margin-bottom: 4px;">
                <span class="defined-label" style="font-size: 0.65rem; color: var(--text-muted); text-transform: uppercase; font-family: var(--font-mono); white-space: nowrap;">Modelo Definido:</span>
                ${activeHermesBadge}
              </div>
              <span class="defined-model-code" style="font-family: var(--font-mono); font-size: 0.76rem; color: var(--cyan); word-break: break-all; display: block;" title="${p.target_model}">${p.target_model || 'Auto (Best Value)'}</span>
            </div>
          </div>
        `;
      }

      html += `
        <div class="add-preset-card" onclick="openNewProfileModal()">
          <div style="font-size: 1.5rem; color: var(--purple);">➕</div>
          <div style="font-weight: 600; font-size: 0.85rem; color: #fff;">Novo Perfil de Trabalho</div>
          <div style="font-size: 0.7rem; color: var(--text-muted);">Especialista p/ subagentes</div>
        </div>
      `;

      grid.innerHTML = html;
    }"""

    new_render_fn_replacement = """    function renderProfileCards() {
      const grid = document.getElementById('profiles-cards-grid');
      if (!grid) return;

      // Helper para renderizar card individual
      const renderCardHtml = (p) => {
        const isActive = (p.id === activePreset);
        const isMaster = (p.role === 'orchestrator');
        const roleClass = isMaster ? 'master' : 'subagent';
        const roleLabel = isMaster ? '👑 Master Brain' : '🤖 Subagente';

        const deleteBtn = !p.is_builtin ? `
          <button class="btn-delete-profile" title="Excluir Perfil de Trabalho" onclick="event.stopPropagation(); deleteCustomProfile('${p.id}')">
            &times;
          </button>
        ` : '';

        const activeHermesBadge = p.is_active ? `
          <span class="active-hermes-pill" style="white-space: nowrap; flex-shrink: 0;">Ativo no Hermes</span>
        ` : '';

        const inspectingBadge = isActive ? `
          <span class="inspecting-pill"><span>Inspecionando</span> <span style="font-size: 0.72rem;">➔</span></span>
        ` : '';

        return `
          <div class="preset-card ${isActive ? 'active' : ''}" id="preset-${p.id}" onclick="selectProfilePreset('${p.id}')" style="min-height: 195px; padding: 16px; border-radius: var(--radius-md); display: flex; flex-direction: column; justify-content: space-between;">
            <div>
              ${deleteBtn}
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
                <span class="role-badge ${roleClass}">${roleLabel}</span>
                <span class="category-tag">${p.arena_category}</span>
              </div>
              <div class="preset-title" style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 8px;">
                <span style="font-weight: 700; font-size: 0.94rem; color: #fff; line-height: 1.3; flex: 1; min-width: 0;">${p.name}</span>
                ${inspectingBadge}
              </div>
              <div class="preset-desc" style="font-size: 0.75rem; color: var(--text-muted); line-height: 1.45; margin-bottom: 10px;">${p.description}</div>
            </div>

            <div class="preset-defined-box" style="margin-top: 10px; background: rgba(0,0,0,0.3); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 8px 12px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 4px;">
                <span class="defined-label" style="font-size: 0.65rem; color: var(--text-muted); text-transform: uppercase; font-family: var(--font-mono); white-space: nowrap;">Modelo Definido:</span>
                ${activeHermesBadge}
              </div>
              <span class="defined-model-code" style="font-family: var(--font-mono); font-size: 0.78rem; color: var(--cyan); word-break: break-all; display: block;" title="${p.target_model}">${p.target_model || 'Auto (Best Value)'}</span>
            </div>
          </div>
        `;
      };

      // Card 2: Novo Perfil de Trabalho
      const addCardHtml = `
        <div class="add-preset-card" onclick="openNewProfileModal()" style="min-height: 195px; padding: 16px; border-radius: var(--radius-md); display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; border: 2px dashed rgba(168, 85, 247, 0.35); background: rgba(168, 85, 247, 0.04); cursor: pointer; transition: all 0.2s ease;">
          <div style="font-size: 1.8rem; color: var(--purple-light); margin-bottom: 6px;">➕</div>
          <div style="font-weight: 700; font-size: 0.92rem; color: #fff; margin-bottom: 4px;">Novo Perfil de Trabalho</div>
          <div style="font-size: 0.72rem; color: var(--text-muted); line-height: 1.35;">Especialista customizado p/ subagentes</div>
        </div>
      `;

      // Ordem obrigatória solicitada pelo Arquiteto:
      // 1. Master Brain
      // 2. Novo Perfil de Trabalho
      // 3. Perfis já criados (Subagentes)
      const masterProfile = profilesData.find(p => p.role === 'orchestrator' || p.id === 'master_brain') || profilesData[0];
      const subagentProfiles = profilesData.filter(p => p !== masterProfile);

      let finalHtml = '';
      if (masterProfile) {
        finalHtml += renderCardHtml(masterProfile);
      }
      finalHtml += addCardHtml;
      for (const p of subagentProfiles) {
        finalHtml += renderCardHtml(p);
      }

      grid.innerHTML = finalHtml;
    }"""

    assert old_render_fn_target in content, "old_render_fn_target not found"
    content = content.replace(old_render_fn_target, new_render_fn_replacement, 1)
    print("✓ Updated renderProfileCards (Master Brain 1st, Novo Perfil 2nd, Subagentes 3rd+)")

    # =========================================================================
    # PART 3: COMPLETE OVERHAUL OF SUBTAB-RAW (CONFIGURAÇÃO AVANÇADA - TOML)
    # =========================================================================

    old_raw_subtab = '''      <div class="settings-subpane" id="subtab-raw">
        <div class="card">
          <div class="card-header">
            <div class="card-title">
              <div class="card-icon icon-cyan">📄</div>
              Editor de Texto Puro: souls_config.toml (SSOT ReFS)
              <span class="info-icon">i
                <span class="tooltip-box">
                  <strong>Single Source of Truth</strong>
                  Arquivo de configuração estruturado do Souls Engine v7 em Z:\\souls_engine\\.souls_data\\config\\souls_config.toml.
                  <div class="tooltip-def">Gravado com backup automático .toml.bak</div>
                </span>
              </span>
            </div>
            <div style="display: flex; gap: 8px;">
              <button class="btn" onclick="fetchConfigToml()">
                <span>🔄</span> Recarregar do Disco
              </button>
              <button class="btn btn-green" onclick="saveRawToml()">
                <span>💾</span> Salvar Arquivo TOML
              </button>
            </div>
          </div>
          <p style="font-size: 0.74rem; color: var(--text-muted); margin-bottom: 12px;">
            Edição direta do arquivo centralizado. Ao salvar, o Souls Engine cria um backup automático <code>.toml.bak</code> e reflete os valores em tempo real no daemon:
          </p>
          <textarea id="raw-toml-editor" class="setting-input" style="height: 380px; font-size: 0.78rem; line-height: 1.5; font-family: var(--font-mono); resize: vertical;"></textarea>
        </div>
      </div>'''

    new_raw_subtab = '''      <!-- ==================================================== -->
      <!-- SUB-ABA 8: CONFIGURAÇÃO AVANÇADA & SSOT (TOML)         -->
      <!-- ==================================================== -->
      <div class="settings-subpane" id="subtab-raw">
        <!-- HEADER DA SUB-ABA COM AÇÕES -->
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
          <div>
            <h2 style="font-size: 1.15rem; font-weight: 700; color: #fff; display: flex; align-items: center; gap: 8px; margin: 0 0 4px 0;">
              <span>📄</span>
              <span>Configuração Avançada & SSOT (souls_config.toml)</span>
              <span class="info-icon" style="z-index: 95;">i
                <span class="tooltip-box">
                  <strong>Single Source of Truth (SSOT) Bare-Metal</strong>
                  O arquivo <code>souls_config.toml</code> governa todos os parâmetros físicos do Souls Engine. Mutações aqui gravadas disparam validação sintática estrita no backend, geram backup atômico de segurança (<code>.toml.bak</code>) e aplicam hot-reload instantâneo ao daemon (127.0.0.1:9123).
                </span>
              </span>
            </h2>
            <p style="font-size: 0.78rem; color: var(--text-muted); margin: 0;">
              Arquivo mestre de configuração em disco. Edição com verificação sintática, backup de segurança e hot-reload no daemon.
            </p>
          </div>
          <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
            <button class="btn btn-secondary" onclick="fetchConfigToml()" style="padding: 6px 12px; font-size: 0.75rem;" title="Recarregar versão do disco">
              <span>🔄</span> Recarregar
            </button>
            <button class="btn btn-secondary" onclick="formatTomlBuffer()" style="padding: 6px 12px; font-size: 0.75rem;" title="Harmonizar espaçamento e linhas">
              <span>✨</span> Formatar
            </button>
            <button class="btn btn-cyan" onclick="validateTomlBuffer()" style="padding: 6px 12px; font-size: 0.75rem;" title="Verificar conformidade sintática no backend">
              <span>🔍</span> Validar Sintaxe
            </button>
            <button class="btn btn-purple" onclick="restoreTomlBackup()" style="padding: 6px 12px; font-size: 0.75rem;" title="Restaurar backup íntegro .toml.bak">
              <span>📑</span> Restaurar Backup
            </button>
            <button class="btn btn-green" onclick="saveRawToml()" style="padding: 6px 14px; font-size: 0.75rem;" title="Gravar alterações e aplicar hot-reload (Ctrl+S)">
              <span>💾</span> Salvar & Aplicar
            </button>
          </div>
        </div>

        <!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico) -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 16px;">
          <!-- CARD 1: ARQUIVO MESTRE (CIANO) -->
          <div class="card" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(56, 189, 248, 0.4); background: linear-gradient(180deg, rgba(56, 189, 248, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--cyan); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>📄</span> <span>ARQUIVO MESTRE</span>
            </div>
            <div class="mono" id="kpi-toml-filename" style="font-size: 1.05rem; font-weight: 700; color: var(--cyan); white-space: nowrap;">
              souls_config.toml
            </div>
            <div id="kpi-toml-size" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              ~3.5 KB • SSOT do Daemon
            </div>
          </div>

          <!-- CARD 2: SEÇÕES ATIVAS (VERDE) -->
          <div class="card" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(16, 185, 129, 0.4); background: linear-gradient(180deg, rgba(16, 185, 129, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--green); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>⚙️</span> <span>SEÇÕES ATIVAS</span>
            </div>
            <div class="mono" id="kpi-toml-sections" style="font-size: 1.05rem; font-weight: 700; color: var(--green); white-space: nowrap;">
              9 SEÇÕES CANÔNICAS
            </div>
            <div id="kpi-toml-lines" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              84 linhas de configuração
            </div>
          </div>

          <!-- CARD 3: VALIDADOR DE SINTAXE (ROXO) -->
          <div class="card" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(168, 85, 247, 0.4); background: linear-gradient(180deg, rgba(168, 85, 247, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--purple-light); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>🛡️</span> <span>VALIDAÇÃO SINTÁTICA</span>
            </div>
            <div class="mono" id="kpi-toml-validity" style="font-size: 1.05rem; font-weight: 700; color: var(--purple-light); white-space: nowrap;">
              TOML v1.0 ESTREITO
            </div>
            <div id="kpi-toml-backup-status" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Backup .bak Sincronizado
            </div>
          </div>

          <!-- CARD 4: DESTINO DE PERSISTÊNCIA (ÂMBAR) -->
          <div class="card" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(245, 158, 11, 0.4); background: linear-gradient(180deg, rgba(245, 158, 11, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--amber); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>💾</span> <span>DESTINO EM DISCO</span>
            </div>
            <div class="mono" id="kpi-toml-volume" style="font-size: 1.05rem; font-weight: 700; color: var(--amber); white-space: nowrap;">
              DEV DRIVE ReFS
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Volume Z:\\ Zero-Leak
            </div>
          </div>
        </div>

        <!-- BANNER DE GOVERNANÇA DE CONFIGURAÇÃO -->
        <div style="background: rgba(56, 189, 248, 0.05); border: 1px solid rgba(56, 189, 248, 0.25); border-radius: var(--radius-md); padding: 10px 14px; margin-bottom: 16px; display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
          <div style="font-size: 0.74rem; line-height: 1.5; color: #e2e8f0; flex: 1; min-width: 280px; text-wrap: balance;">
            <strong style="color: var(--cyan);">💡 Governança de Configuração Bare-Metal (AGENTS.md):</strong> Este arquivo é a fonte única da verdade (SSOT) do daemon Souls Engine. Alterações salvas aqui acionam validação sintática estrita no backend e hot-reload no daemon (<code>127.0.0.1:9123</code>). O sistema mantém um backup automático (<code>.toml.bak</code>) antes de cada gravação para garantir recuperação instantânea com 1 clique caso ocorra erro de formatação.
          </div>
          <span style="font-size: 0.68rem; font-family: var(--font-mono); color: var(--cyan); background: rgba(56, 189, 248, 0.12); border: 1px solid rgba(56, 189, 248, 0.3); padding: 3px 8px; border-radius: 4px; white-space: nowrap; align-self: center;">
            • Hot-Reload Atômico Ativo
          </span>
        </div>

        <!-- NAVEGADOR RÁPIDO DE SEÇÕES (SECTION JUMP PILLS) -->
        <div style="background: rgba(15, 23, 42, 0.5); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: var(--radius-md); padding: 10px 14px; margin-bottom: 16px;">
          <div style="font-size: 0.70rem; font-weight: 700; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; display: flex; align-items: center; justify-content: space-between;">
            <span>Navegação Rápida por Seções Canônicas:</span>
            <span style="font-size: 0.66rem; color: #64748b; font-weight: normal;">Clique na seção para pular diretamente no editor</span>
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 8px;">
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[core]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>🌐</span> [core]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[security]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>🛡️</span> [security]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[memory]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>💾</span> [memory]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[inference]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>⚡</span> [inference]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[model_router]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>🔀</span> [model_router]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[obsidian]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>📓</span> [obsidian]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[ast]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>🌲</span> [ast]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[anthropic]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>☁️</span> [anthropic]
            </button>
            <button class="btn btn-secondary" onclick="jumpToTomlSection('[arena]')" style="padding: 4px 10px; font-size: 0.72rem; font-family: var(--font-mono);">
              <span>🏆</span> [arena]
            </button>
          </div>
        </div>

        <!-- CARD DO EDITOR TOML DE ENGENHARIA -->
        <div class="card" style="margin-bottom: 0; padding: 16px;">
          <!-- TOOLBAR SUPERIOR DO EDITOR -->
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; padding-bottom: 10px; border-bottom: 1px solid rgba(255, 255, 255, 0.08); flex-wrap: wrap; gap: 8px;">
            <div style="display: flex; align-items: center; gap: 10px;">
              <span class="badge" id="badge-toml-status" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.68rem; font-family: var(--font-mono);">
                ✓ Sintaxe Válida
              </span>
              <span id="label-toml-metrics" style="font-size: 0.70rem; color: var(--text-muted); font-family: var(--font-mono);">
                Carregando arquivo...
              </span>
            </div>
            <div style="font-size: 0.68rem; color: #64748b; font-family: var(--font-mono);">
              Atalho: <strong style="color: #cbd5e1;">Ctrl+S</strong> para gravar • <strong style="color: #cbd5e1;">Tab</strong> para indentar
            </div>
          </div>

          <!-- ÁREA DO EDITOR DE TEXTO COM ESTILO TERMINAL BARE-METAL -->
          <textarea id="raw-toml-editor" class="setting-input" spellcheck="false" style="height: 460px; font-size: 0.80rem; line-height: 1.55; font-family: 'Consolas', 'Fira Code', 'Cascadia Code', monospace; resize: vertical; background: #070c18; color: #e2e8f0; border-color: rgba(56, 189, 248, 0.2); tab-size: 2;" oninput="onTomlEditorInput()" onkeydown="handleTomlEditorKey(event)"></textarea>

          <!-- STATUS BAR INFERIOR DO EDITOR -->
          <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 10px; font-size: 0.68rem; color: var(--text-muted); font-family: var(--font-mono); flex-wrap: wrap; gap: 8px;">
            <div>
              <span>Caminho Físico: </span>
              <code id="footer-toml-path" style="color: #94a3b8;">Z:\\souls_engine\\.souls_data\\config\\souls_config.toml</code>
            </div>
            <div id="footer-toml-last-saved">
              Status: Sincronizado com o daemon
            </div>
          </div>
        </div>
      </div>'''

    assert old_raw_subtab in content, "old_raw_subtab not found"
    content = content.replace(old_raw_subtab, new_raw_subtab, 1)
    print("✓ Replaced subtab-raw with full-featured TOML Engineering Suite!")

    # 3.2 Add JavaScript functions for TOML Subtab:
    # validateTomlBuffer, formatTomlBuffer, restoreTomlBackup, jumpToTomlSection, onTomlEditorInput, handleTomlEditorKey
    old_fetch_target = """    async function fetchConfigToml() {
      try {
        const res = await fetch('/api/v1/settings/config');
        if (res.ok) {
          const data = await res.json();
          currentConfigData = data;
          const editor = document.getElementById('raw-toml-editor');
          if (editor && data.raw_toml) editor.value = data.raw_toml;
          if (data.parsed) {
            populateSettingsForm(data.parsed);
          }
        }
      } catch (e) {
        console.warn("Falha ao carregar souls_config.toml", e);
      }
    }"""

    new_fetch_replacement = """    async function fetchConfigToml() {
      try {
        const res = await fetch('/api/v1/settings/config');
        if (res.ok) {
          const data = await res.json();
          currentConfigData = data;
          const editor = document.getElementById('raw-toml-editor');
          if (editor && data.raw_toml) {
            editor.value = data.raw_toml;
            updateTomlEditorMetrics(data.raw_toml);
          }
          if (data.file_size_bytes) {
            const kb = (data.file_size_bytes / 1024).toFixed(1);
            const sizeEl = document.getElementById('kpi-toml-size');
            if (sizeEl) sizeEl.innerText = `${kb} KB • SSOT do Daemon`;
          }
          if (data.line_count && data.sections_count) {
            const linesEl = document.getElementById('kpi-toml-lines');
            if (linesEl) linesEl.innerText = `${data.line_count} linhas • ${data.sections_count} seções`;
            const secKpi = document.getElementById('kpi-toml-sections');
            if (secKpi) secKpi.innerText = `${data.sections_count} SEÇÕES CANÔNICAS`;
          }
          if (data.backup_exists) {
            const bStatus = document.getElementById('kpi-toml-backup-status');
            if (bStatus) bStatus.innerText = 'Backup .bak Sincronizado';
          }
          if (data.path) {
            const pEl = document.getElementById('footer-toml-path');
            if (pEl) pEl.innerText = data.path;
          }
          const savedEl = document.getElementById('footer-toml-last-saved');
          if (savedEl) savedEl.innerText = `Sincronizado: ${new Date().toLocaleTimeString('pt-BR')}`;
          
          if (data.parsed) {
            populateSettingsForm(data.parsed);
          }
        }
      } catch (e) {
        console.warn("Falha ao carregar souls_config.toml", e);
      }
    }

    function onTomlEditorInput() {
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      markSettingsUnsaved();
      updateTomlEditorMetrics(editor.value);
    }

    function updateTomlEditorMetrics(text) {
      const metricsEl = document.getElementById('label-toml-metrics');
      if (!metricsEl) return;
      const lines = text.split('\\n').length;
      const bytes = new Blob([text]).size;
      metricsEl.innerText = `${lines} linhas • ${bytes.toLocaleString('pt-BR')} bytes`;
    }

    function handleTomlEditorKey(e) {
      // Atalho Ctrl+S para salvar direto
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        saveRawToml();
        return;
      }
      // Suporte a Tab (insere 2 espaços)
      if (e.key === 'Tab') {
        e.preventDefault();
        const editor = e.target;
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = editor.value.substring(0, start) + '  ' + editor.value.substring(end);
        editor.selectionStart = editor.selectionEnd = start + 2;
        onTomlEditorInput();
      }
    }

    function jumpToTomlSection(sectionTag) {
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      const text = editor.value;
      const idx = text.indexOf(sectionTag);
      if (idx !== -1) {
        editor.focus();
        editor.setSelectionRange(idx, idx + sectionTag.length);
        // Calcular scroll aproximado
        const linesBefore = text.substring(0, idx).split('\\n').length;
        const lineHeight = 19;
        editor.scrollTop = Math.max(0, (linesBefore - 3) * lineHeight);
        showToast(`📍 Seção ${sectionTag} localizada!`, 'info');
      } else {
        showToast(`Seção ${sectionTag} não encontrada no arquivo`, 'warning');
      }
    }

    async function validateTomlBuffer() {
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      const text = editor.value;

      try {
        const res = await fetch('/api/v1/settings/config/validate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ raw_toml: text })
        });
        const data = await res.json();
        const badge = document.getElementById('badge-toml-status');
        const kpiValidity = document.getElementById('kpi-toml-validity');

        if (data.valid) {
          if (badge) {
            badge.innerText = '✓ Sintaxe Válida';
            badge.style.color = 'var(--green)';
            badge.style.background = 'rgba(16, 185, 129, 0.12)';
            badge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
          if (kpiValidity) {
            kpiValidity.innerText = 'TOML v1.0 ESTREITO';
            kpiValidity.style.color = 'var(--purple-light)';
          }
          showToast(`✓ Validação TOML com Sucesso! ${data.section_count} seções válidas.`, 'success');
        } else {
          if (badge) {
            badge.innerText = '⚠️ Erro de Sintaxe';
            badge.style.color = '#ef4444';
            badge.style.background = 'rgba(239, 68, 68, 0.15)';
            badge.style.borderColor = 'rgba(239, 68, 68, 0.4)';
          }
          if (kpiValidity) {
            kpiValidity.innerText = 'ERRO SINTÁTICO';
            kpiValidity.style.color = '#ef4444';
          }
          showToast(`Erro na validação: ${data.error}`, 'error');
        }
      } catch (e) {
        showToast(`Erro de rede na validação: ${e.message}`, 'error');
      }
    }

    function formatTomlBuffer() {
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      let text = editor.value;
      const lines = text.split('\\n');
      const formatted = [];
      let inSection = false;

      for (let i = 0; i < lines.length; i++) {
        let line = lines[i].trim();
        if (line.startsWith('[') && line.endsWith(']')) {
          if (formatted.length > 0 && formatted[formatted.length - 1] !== '') {
            formatted.push('');
          }
          formatted.push(line);
          inSection = true;
        } else if (line.includes('=')) {
          const parts = line.split('=');
          const key = parts[0].trim();
          const val = parts.slice(1).join('=').trim();
          formatted.push(`${key} = ${val}`);
        } else {
          formatted.push(line);
        }
      }

      editor.value = formatted.join('\\n');
      onTomlEditorInput();
      showToast('✨ Buffer TOML formatado com sucesso!', 'info');
    }

    async function restoreTomlBackup() {
      if (!confirm('Deseja restaurar o arquivo de backup souls_config.toml.bak? As alterações não salvas serão sobrescritas.')) {
        return;
      }
      try {
        const res = await fetch('/api/v1/settings/config/restore', { method: 'POST' });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          showToast('✓ Backup souls_config.toml.bak restaurado com sucesso!', 'success');
          await fetchConfigToml();
          dismissUnsavedBanner();
        } else {
          showToast(`Erro ao restaurar backup: ${data.message || 'Arquivo não encontrado'}`, 'error');
        }
      } catch (e) {
        showToast(`Erro de rede ao restaurar backup: ${e.message}`, 'error');
      }
    }"""

    assert old_fetch_target in content, "old_fetch_target not found"
    content = content.replace(old_fetch_target, new_fetch_replacement, 1)
    print("✓ Added TOML JavaScript functions (validate, format, restore, jump, metrics, hotkey)")

    # Ensure 100% LF
    content = content.replace('\r\n', '\n')

    with open(file_path, "w", encoding="utf-8") as f:
        f.write(content)

    print("SUCCESS! Applied all final refinements to dashboard_ui.html!")

if __name__ == '__main__':
    main()

