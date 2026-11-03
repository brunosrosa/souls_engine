import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    html = f.read()

# 1. Update tab badge
html = html.replace(
    '<span class="tab-count-badge" id="badge-cloud-profiles-count">5 Perfis</span>',
    '<span class="tab-count-badge" id="badge-cloud-profiles-count">6 Perfis</span>'
)

# 2. Update CSS: add cap-tag and mfg styles, refine badge-cheaper
old_cheaper_css = """    .badge-cheaper {
      background: rgba(16, 185, 129, 0.18);
      color: #34d399;
      border: 1px solid rgba(16, 185, 129, 0.4);
      font-size: 0.6rem;
      padding: 1px 4px;
      border-radius: 3px;
      margin-left: 4px;
    }"""

new_cheaper_css = """    .badge-cheaper {
      color: #fbbf24;
      font-weight: 800;
      font-size: 0.85rem;
      margin-left: 3px;
      cursor: help;
      vertical-align: middle;
      display: inline-block;
    }
    .cap-tag {
      display: inline-flex;
      align-items: center;
      gap: 3px;
      font-size: 0.62rem;
      font-family: var(--font-mono);
      font-weight: 600;
      padding: 1px 4px;
      border-radius: 3px;
      background: rgba(255, 255, 255, 0.06);
      color: #94a3b8;
      border: 1px solid rgba(255, 255, 255, 0.1);
      white-space: nowrap;
      cursor: help;
    }
    .cap-tag.cap-vision {
      background: rgba(168, 85, 247, 0.15);
      color: #c084fc;
      border-color: rgba(168, 85, 247, 0.35);
    }
    .cap-tag.cap-caching {
      background: rgba(245, 158, 11, 0.15);
      color: #fbbf24;
      border-color: rgba(245, 158, 11, 0.35);
    }
    .cap-tag.cap-tools {
      background: rgba(56, 189, 248, 0.15);
      color: #38bdf8;
      border-color: rgba(56, 189, 248, 0.35);
    }
    .cap-tag.cap-schema {
      background: rgba(16, 185, 129, 0.15);
      color: #34d399;
      border-color: rgba(16, 185, 129, 0.35);
    }
    .mfg-checkbox-item {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 0.72rem;
      color: #cbd5e1;
      cursor: pointer;
      padding: 3px 6px;
      border-radius: 3px;
      user-select: none;
    }
    .mfg-checkbox-item:hover {
      background: rgba(255, 255, 255, 0.05);
      color: #fff;
    }"""

if old_cheaper_css in html:
    html = html.replace(old_cheaper_css, new_cheaper_css)
    print("Replaced cheaper CSS and added cap-tag/mfg CSS")
else:
    print("WARNING: old_cheaper_css not found")

# 3. Add secondary filter bar (Modality + Manufacturer + Hermes indicator)
search_filter_bar = """        <!-- SUBTEXTO DINÂMICO DO PERFIL ATIVO -->"""

new_secondary_filters = """        <!-- BARRA DE FILTROS SECUNDÁRIOS: MODALIDADE + FABRICANTE + CÉREBRO HERMES -->
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px; margin-bottom: 10px; background: rgba(15, 23, 42, 0.55); padding: 5px 12px; border-radius: var(--radius-md); border: 1px solid var(--border-subtle);">
          <div style="display: flex; align-items: center; gap: 14px; flex-wrap: wrap;">
            <!-- Modalidade -->
            <div style="display: flex; align-items: center; gap: 6px;">
              <span style="font-size: 0.68rem; color: var(--text-muted); text-transform: uppercase; font-family: var(--font-mono); font-weight: 700; letter-spacing: 0.05em;">Modalidade:</span>
              <div class="filter-pills-group" id="cloud-modality-filter-pills">
                <button class="filter-provider-btn active" id="btn-cloud-modal-all" onclick="setCloudModalityFilter('all')">🌐 Todas</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-multi" onclick="setCloudModalityFilter('multimodal')" title="Apenas modelos multimodais que processam imagens e visão">👁️ Multimodal (Visão)</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-text" onclick="setCloudModalityFilter('text')" title="Modelos focados puramente em texto, código e raciocínio">📝 Texto Puro</button>
              </div>
            </div>

            <!-- Fabricantes Multi-Select -->
            <div style="position: relative;">
              <button class="btn" id="btn-mfg-toggle" onclick="toggleMfgMenu(event)" style="background: rgba(15, 23, 42, 0.85); border: 1px solid var(--border-subtle); color: #cbd5e1; font-size: 0.74rem; padding: 4px 10px; display: flex; align-items: center; gap: 6px; cursor: pointer; border-radius: var(--radius-sm);">
                <span>🏢</span> Fabricante: <span id="mfg-filter-label" style="color: var(--cyan); font-weight: 600;">Todos</span> ▾
              </button>
              <div id="mfg-dropdown-menu" style="display: none; position: absolute; left: 0; top: 100%; margin-top: 5px; background: #0f172a; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 10px; width: 230px; z-index: 120; box-shadow: 0 12px 30px rgba(0,0,0,0.75);">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; border-bottom: 1px solid rgba(255,255,255,0.08); padding-bottom: 5px;">
                  <span style="font-size: 0.72rem; font-weight: 700; color: #fff;">Fabricantes</span>
                  <div style="display: flex; gap: 6px;">
                    <button onclick="toggleAllMfgs(true)" style="background: none; border: none; color: var(--cyan); font-size: 0.68rem; cursor: pointer;">Todos</button>
                    <span style="color: var(--text-muted);">|</span>
                    <button onclick="toggleAllMfgs(false)" style="background: none; border: none; color: #a78bfa; font-size: 0.68rem; cursor: pointer;">Limpar</button>
                  </div>
                </div>
                <div style="display: flex; flex-direction: column; gap: 3px; max-height: 220px; overflow-y: auto;" id="mfg-checkboxes-container">
                  <!-- Gerado dinamicamente via JS -->
                </div>
              </div>
            </div>
          </div>

          <!-- Indicador Cérebro Hermes Ativo -->
          <div style="display: flex; align-items: center; gap: 8px; font-size: 0.72rem;">
            <span style="color: var(--text-muted);">🤖 Cérebro Hermes Ativo:</span>
            <code class="mono" id="cloud-quick-active-model" style="color: #34d399; background: rgba(16, 185, 129, 0.12); padding: 2px 7px; border-radius: 4px; border: 1px solid rgba(16, 185, 129, 0.35); font-weight: 600;">deepseek/deepseek-chat</code>
          </div>
        </div>

        <!-- SUBTEXTO DINÂMICO DO PERFIL ATIVO -->"""

if search_filter_bar in html:
    html = html.replace(search_filter_bar, new_secondary_filters)
    print("Added secondary filters bar")
else:
    print("WARNING: search_filter_bar not found")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(html)
print("Step 1 done")

