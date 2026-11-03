with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('\r\n', '\n')

# 1. Update Tab button
target_btn = """      <button class="tab-btn" id="btn-tab-settings" onclick="switchTab('tab-settings')">
        <span>⚙️ Governança & Presets</span>
      </button>"""

replacement_btn = """      <button class="tab-btn" id="btn-tab-settings" onclick="switchTab('tab-settings')">
        <span>⚙️ Settings</span>
      </button>"""

assert target_btn in content, "Error: target_btn not found"
content = content.replace(target_btn, replacement_btn)
print("1. Tab button renamed to Settings.")

# 2. Add CSS for settings-layout and settings-sidebar
target_css = """    .settings-subtabs {
      display: flex;
      gap: 6px;
      overflow-x: auto;
      padding-bottom: 8px;
      margin-bottom: 18px;
      border-bottom: 1px solid var(--border-subtle);
      scrollbar-width: thin;
      align-items: center;
    }"""

replacement_css = """    /* SETTINGS SIDEBAR LAYOUT (SETTINGS TAB) */
    .settings-layout {
      display: flex;
      gap: 22px;
      align-items: flex-start;
      width: 100%;
    }

    .settings-sidebar {
      width: 260px;
      min-width: 260px;
      max-width: 260px;
      background: rgba(13, 19, 36, 0.85);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-md);
      padding: 14px 10px;
      display: flex;
      flex-direction: column;
      gap: 12px;
      position: sticky;
      top: 16px;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
      backdrop-filter: blur(14px);
    }

    .settings-sidebar-header {
      padding: 4px 8px 10px 8px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.07);
      display: flex;
      align-items: center;
      justify-content: space-between;
    }

    .settings-sidebar-title {
      font-size: 0.72rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--text-muted);
      font-family: var(--font-mono);
    }

    .settings-sidebar-badge {
      font-size: 0.62rem;
      font-family: var(--font-mono);
      color: var(--cyan);
      background: rgba(6, 182, 212, 0.12);
      border: 1px solid rgba(6, 182, 212, 0.3);
      padding: 2px 6px;
      border-radius: 4px;
      font-weight: 600;
    }

    .settings-sidebar-nav {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }

    .settings-sidebar .settings-subtab-btn {
      width: 100%;
      text-align: left;
      padding: 9px 12px;
      display: flex;
      align-items: center;
      gap: 12px;
      border-radius: var(--radius-sm);
      border: 1px solid transparent;
      background: transparent;
      color: #94a3b8;
      transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
      cursor: pointer;
      box-shadow: none;
    }

    .settings-sidebar .settings-subtab-btn:hover {
      background: rgba(255, 255, 255, 0.05);
      color: #f8fafc;
      border-color: rgba(255, 255, 255, 0.1);
      transform: translateX(3px);
    }

    .settings-sidebar .settings-subtab-btn.active {
      background: rgba(6, 182, 212, 0.12);
      border-color: rgba(6, 182, 212, 0.45);
      color: #fff;
      box-shadow: 0 0 16px rgba(6, 182, 212, 0.18);
      border-left: 3px solid var(--cyan);
    }

    .settings-sidebar .settings-subtab-btn.active .nav-label {
      color: var(--cyan);
    }

    .settings-sidebar .nav-icon {
      font-size: 1.15rem;
      width: 24px;
      text-align: center;
      flex-shrink: 0;
    }

    .settings-sidebar .nav-text {
      display: flex;
      flex-direction: column;
      min-width: 0;
      flex: 1;
    }

    .settings-sidebar .nav-label {
      font-size: 0.80rem;
      font-weight: 600;
      color: #e2e8f0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .settings-sidebar .nav-desc {
      font-size: 0.67rem;
      color: var(--text-muted);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin-top: 2px;
    }

    .settings-sidebar-footer {
      margin-top: 8px;
      padding-top: 10px;
      border-top: 1px solid rgba(255, 255, 255, 0.07);
      display: flex;
      gap: 8px;
    }

    .settings-content-area {
      flex: 1;
      min-width: 0;
    }

    @media (max-width: 1080px) {
      .settings-layout {
        flex-direction: column;
      }
      .settings-sidebar {
        width: 100%;
        max-width: 100%;
        position: static;
      }
      .settings-sidebar-nav {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      }
    }

    .settings-subtabs {
      display: none;
    }"""

assert target_css in content, "Error: target_css not found"
content = content.replace(target_css, replacement_css)
print("2. CSS for settings layout and sidebar added.")

# 3. Replace horizontal subtabs with settings-layout, aside, and main
target_html_start = """    <section class="tab-pane" id="tab-settings">
      
      <!-- SUB-ABAS HORIZONTAIS DE NAVEGAÇÃO -->
      <div class="settings-subtabs">
        <button class="settings-subtab-btn active" id="btn-subtab-brain" onclick="switchSettingsSubtab('subtab-brain')">
          <span>🎯</span> Master Brain & Tríade
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-reasoning" onclick="switchSettingsSubtab('subtab-reasoning')">
          <span>🧠</span> Reasoning & Budget
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-finops" onclick="switchSettingsSubtab('subtab-finops')">
          <span>⚡</span> FinOps & Silício (RTX 2060m)
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-router" onclick="switchSettingsSubtab('subtab-router')">
          <span>🔀</span> ParetoBandit Router
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-memory" onclick="switchSettingsSubtab('subtab-memory')">
          <span>💾</span> Memória & LadybugDB
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-sandbox" onclick="switchSettingsSubtab('subtab-sandbox')">
          <span>🛡️</span> Sandbox & AST Code
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-obsidian" onclick="switchSettingsSubtab('subtab-obsidian')">
          <span>📓</span> Obsidian DB Curator
        </button>
        <button class="settings-subtab-btn" id="btn-subtab-raw" onclick="switchSettingsSubtab('subtab-raw')">
          <span>📄</span> Raw souls_config.toml
        </button>
      </div>

      <!-- ==================================================== -->
      <!-- SUB-ABA 1: MASTER BRAIN & TRÍADE DA CLOUD ARENA -->
      <!-- ==================================================== -->
      <div class="settings-subpane active" id="subtab-brain">"""

replacement_html_start = """    <section class="tab-pane" id="tab-settings">
      <div class="settings-layout">
        <!-- SIDEBAR DE NAVEGAÇÃO VERTICAL (MENU À ESQUERDA) -->
        <aside class="settings-sidebar">
          <div class="settings-sidebar-header">
            <span class="settings-sidebar-title">Painéis de Ajuste</span>
            <span class="settings-sidebar-badge">v7 Bare-Metal</span>
          </div>
          <nav class="settings-sidebar-nav">
            <button class="settings-subtab-btn active" id="btn-subtab-brain" onclick="switchSettingsSubtab('subtab-brain')">
              <span class="nav-icon">🎯</span>
              <div class="nav-text">
                <span class="nav-label">Master Brain & Tríades</span>
                <span class="nav-desc">Perfis & Pareto Arena</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-reasoning" onclick="switchSettingsSubtab('subtab-reasoning')">
              <span class="nav-icon">🧠</span>
              <div class="nav-text">
                <span class="nav-label">Reasoning & Budget</span>
                <span class="nav-desc">Entropia de Shannon</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-finops" onclick="switchSettingsSubtab('subtab-finops')">
              <span class="nav-icon">⚡</span>
              <div class="nav-text">
                <span class="nav-label">FinOps & Silício</span>
                <span class="nav-desc">RTX 2060m & Watchdog</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-router" onclick="switchSettingsSubtab('subtab-router')">
              <span class="nav-icon">🔀</span>
              <div class="nav-text">
                <span class="nav-label">ParetoBandit Router</span>
                <span class="nav-desc">Heurísticas & Despacho</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-memory" onclick="switchSettingsSubtab('subtab-memory')">
              <span class="nav-icon">💾</span>
              <div class="nav-text">
                <span class="nav-label">Memória & LadybugDB</span>
                <span class="nav-desc">Grafo L2 & FrankenSQLite</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-sandbox" onclick="switchSettingsSubtab('subtab-sandbox')">
              <span class="nav-icon">🛡️</span>
              <div class="nav-text">
                <span class="nav-label">Sandbox & AST Code</span>
                <span class="nav-desc">Confinamento & Cirurgia</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-obsidian" onclick="switchSettingsSubtab('subtab-obsidian')">
              <span class="nav-icon">📓</span>
              <div class="nav-text">
                <span class="nav-label">Obsidian DB Curator</span>
                <span class="nav-desc">Wiki, Raw & Sync ReFS</span>
              </div>
            </button>
            <button class="settings-subtab-btn" id="btn-subtab-raw" onclick="switchSettingsSubtab('subtab-raw')">
              <span class="nav-icon">📄</span>
              <div class="nav-text">
                <span class="nav-label">Raw souls_config.toml</span>
                <span class="nav-desc">Editor Bare-Metal</span>
              </div>
            </button>
          </nav>
          <div class="settings-sidebar-footer">
            <button class="btn btn-sm" onclick="fetchConfigToml()" title="Recarregar configuração do disco" style="flex: 1; justify-content: center; font-size: 0.72rem; padding: 6px 8px;">
              <span>🔄</span> Recarregar
            </button>
            <button class="btn btn-green btn-sm" onclick="saveSettings()" title="Gravar alterações no disco" style="flex: 1; justify-content: center; font-size: 0.72rem; padding: 6px 8px;">
              <span>💾</span> Salvar
            </button>
          </div>
        </aside>

        <!-- ÁREA PRINCIPAL DE CONTEÚDO (CONTEÚDO À DIREITA) -->
        <main class="settings-content-area">
          <!-- ==================================================== -->
          <!-- SUB-ABA 1: MASTER BRAIN & TRÍADE DA CLOUD ARENA -->
          <!-- ==================================================== -->
          <div class="settings-subpane active" id="subtab-brain">"""

assert target_html_start in content, "Error: target_html_start not found"
content = content.replace(target_html_start, replacement_html_start)
print("3. HTML start replaced with sidebar and main content area.")

# 4. Close main and settings-layout before </section>
target_html_end = """          <textarea id="raw-toml-editor" class="setting-input" style="height: 380px; font-size: 0.78rem; line-height: 1.5; font-family: var(--font-mono); resize: vertical;"></textarea>
        </div>
      </div>

    </section>"""

replacement_html_end = """          <textarea id="raw-toml-editor" class="setting-input" style="height: 380px; font-size: 0.78rem; line-height: 1.5; font-family: var(--font-mono); resize: vertical;"></textarea>
        </div>
      </div>
        </main>
      </div> <!-- .settings-layout -->
    </section>"""

assert target_html_end in content, "Error: target_html_end not found"
content = content.replace(target_html_end, replacement_html_end)
print("4. HTML end closed with </main> and </div> <!-- .settings-layout -->.")

# 5. Fix applyTriadModel to support subagent profiles properly
target_apply_triad = """    function applyTriadModel(role) {
      if (!currentTriadData || !currentTriadData[role]) {
        showToast("Dados do modelo não carregados.", "warning");
        return;
      }
      const item = currentTriadData[role];
      await setHermesMasterModel(item.canonical_id, item.provider, activePreset);
    }"""

replacement_apply_triad = """    async function applyTriadModel(role) {
      if (!currentTriadData || !currentTriadData[role]) {
        showToast("Dados do modelo não carregados.", "warning");
        return;
      }
      const item = currentTriadData[role];
      const currentProfile = profilesData.find(p => p.id === activePreset);
      const isMaster = currentProfile ? (currentProfile.role === 'orchestrator') : (activePreset === 'orchestrator' || activePreset === 'frontier_balanced');

      if (isMaster) {
        await setHermesMasterModel(item.canonical_id, item.provider, activePreset);
      } else {
        await saveSubagentProfilePreset(activePreset, item.canonical_id, item.provider);
      }
    }"""

if target_apply_triad in content:
    content = content.replace(target_apply_triad, replacement_apply_triad)
    print("5. applyTriadModel updated to async and supports both master and subagent profiles.")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Settings refactor successfully written to crates/souls_server/src/dashboard_ui.html!")

