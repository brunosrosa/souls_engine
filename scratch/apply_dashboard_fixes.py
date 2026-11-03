with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('\r\n', '\n')

old_tooltip_css = """    .tooltip-box {
      visibility: hidden;
      opacity: 0;
      position: absolute;
      bottom: 130%;
      left: 50%;
      transform: translateX(-50%) translateY(6px);
      width: 280px;
      background: rgba(10, 16, 28, 0.96);
      backdrop-filter: blur(16px);
      -webkit-backdrop-filter: blur(16px);
      border: 1px solid rgba(6, 182, 212, 0.35);
      box-shadow: 0 12px 28px rgba(0, 0, 0, 0.7);
      border-radius: var(--radius-md);
      padding: 10px 12px;
      color: var(--text-main);
      font-family: var(--font-sans);
      font-size: 0.73rem;
      line-height: 1.4;
      z-index: 500;
      pointer-events: none;
      transition: opacity 0.2s ease, transform 0.2s ease, visibility 0.2s ease;
    }

    .tooltip-box strong {
      color: var(--cyan);
      display: block;
      margin-bottom: 3px;
      font-size: 0.76rem;
    }

    .tooltip-box .tooltip-impact {
      margin-top: 5px;
      padding-top: 5px;
      border-top: 1px solid rgba(255, 255, 255, 0.08);
      color: var(--amber);
      font-size: 0.70rem;
    }

    .tooltip-box .tooltip-def {
      color: var(--text-muted);
      font-size: 0.68rem;
      font-family: var(--font-mono);
      margin-top: 2px;
    }

    .info-icon:hover .tooltip-box {
      visibility: visible;
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }"""

new_tooltip_css = """    .tooltip-box {
      visibility: hidden;
      opacity: 0;
      position: absolute;
      top: calc(100% + 8px);
      left: 0;
      transform: translateY(-4px);
      width: 320px;
      max-width: 90vw;
      background: rgba(8, 14, 26, 0.98);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
      border: 1px solid rgba(6, 182, 212, 0.5);
      box-shadow: 0 16px 36px rgba(0, 0, 0, 0.85);
      border-radius: var(--radius-md);
      padding: 12px 14px;
      color: #f1f5f9;
      font-family: var(--font-sans);
      font-size: 0.75rem;
      line-height: 1.45;
      z-index: 99999;
      pointer-events: none;
      transition: opacity 0.2s ease, transform 0.2s ease, visibility 0.2s ease;
    }

    .tooltip-box strong {
      color: var(--cyan);
      display: block;
      margin-bottom: 4px;
      font-size: 0.78rem;
    }

    .tooltip-box .tooltip-impact {
      margin-top: 6px;
      padding-top: 6px;
      border-top: 1px solid rgba(255, 255, 255, 0.1);
      color: var(--amber);
      font-size: 0.72rem;
      line-height: 1.4;
    }

    .tooltip-box .tooltip-def {
      color: var(--text-muted);
      font-size: 0.69rem;
      font-family: var(--font-mono);
      margin-top: 4px;
    }

    .info-icon:hover .tooltip-box {
      visibility: visible;
      opacity: 1;
      transform: translateY(0);
    }"""

assert old_tooltip_css in content, "Error: old_tooltip_css not found"
content = content.replace(old_tooltip_css, new_tooltip_css)
print("2. Tooltip CSS updated with top placement and z-index 99999.")

# 3. Fix summary bar active-model-indicator (remove unicode dot)
old_indicator = '<span id="active-model-indicator" class="active-hermes-pill">● Ativo no Hermes</span>'
new_indicator = '<span id="active-model-indicator" class="active-hermes-pill">Ativo no Hermes</span>'
assert old_indicator in content, "Error: old_indicator not found"
content = content.replace(old_indicator, new_indicator)
print("3. Summary bar indicator updated.")

# 4. Update profiles cards grid min-width from 210px to 235px
old_grid = 'id="profiles-cards-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 12px; margin-bottom: 20px;"'
new_grid = 'id="profiles-cards-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(235px, 1fr)); gap: 14px; margin-bottom: 20px;"'
assert old_grid in content, "Error: old_grid not found"
content = content.replace(old_grid, new_grid)
print("4. profiles-cards-grid minmax updated to 235px.")

# 5. Fix renderProfileCards in JS (inspectingBadge and activeHermesBadge)
old_render_cards = """        const activeHermesBadge = p.is_active ? `
          <span class="active-hermes-pill">● Ativo no Hermes</span>
        ` : '';

        const inspectingBadge = isActive ? `
          <span class="inspecting-pill">Inspecionando ➔</span>
        ` : '';

        html += `
          <div class="preset-card ${isActive ? 'active' : ''}" id="preset-${p.id}" onclick="selectProfilePreset('${p.id}')">
            ${deleteBtn}
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;">
              <span class="role-badge ${roleClass}">${roleLabel}</span>
              <span class="category-tag">${p.arena_category}</span>
            </div>
            <div class="preset-title">
              <span>${p.name}</span>
              ${inspectingBadge}
            </div>
            <div class="preset-desc">${p.description}</div>

            <div class="preset-defined-box">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span class="defined-label">Modelo Definido:</span>
                ${activeHermesBadge}
              </div>
              <span class="defined-model-code" title="${p.target_model}">${p.target_model || 'Auto (Best Value)'}</span>
            </div>
          </div>
        `;"""

new_render_cards = """        const activeHermesBadge = p.is_active ? `
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
        `;"""

assert old_render_cards in content, "Error: old_render_cards not found"
content = content.replace(old_render_cards, new_render_cards)
print("5. renderProfileCards updated.")

# 6. Fix category mapping in renderTriadCards (orchestrator -> master category)
old_cat_map = """      // Refina com dados ao vivo da Cloud Arena se disponível
      const categoryToArena = {
        coding: "coding",
        reasoning: "math",
        fast: "fast",
        agentic: "agentic",
        design: "creative"
      };
      const cat = currentProfile ? currentProfile.arena_category : activePreset;
      const arenaProfileName = categoryToArena[cat] || cat;"""

new_cat_map = """      // Refina com dados ao vivo da Cloud Arena se disponível
      const categoryToArena = {
        master: "master",
        coding: "coding",
        reasoning: "math",
        fast: "fast",
        agentic: "agentic",
        design: "creative",
        creative: "creative"
      };
      let cat = currentProfile ? currentProfile.arena_category : activePreset;
      if (currentProfile && (currentProfile.role === 'orchestrator' || currentProfile.id === 'frontier_balanced')) {
        cat = 'master';
      }
      const arenaProfileName = categoryToArena[cat] || cat;"""

assert old_cat_map in content, "Error: old_cat_map not found"
content = content.replace(old_cat_map, new_cat_map)
print("6. categoryToArena updated with master mapping.")

# 7. Ensure fetchProfiles fetches cloudChampions if missing
old_fetch_profiles = """    async function fetchProfiles() {
      try {
        const res = await fetch('/api/v1/profiles');"""

new_fetch_profiles = """    async function fetchProfiles() {
      try {
        if (!cloudChampionsData || cloudChampionsData.length === 0) {
          fetchCloudChampions();
        }
        const res = await fetch('/api/v1/profiles');"""

assert old_fetch_profiles in content, "Error: old_fetch_profiles not found"
content = content.replace(old_fetch_profiles, new_fetch_profiles)
print("7. fetchProfiles updated to trigger fetchCloudChampions.")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("dashboard_ui.html successfully updated!")
