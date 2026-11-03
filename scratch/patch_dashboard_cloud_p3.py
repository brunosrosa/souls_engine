import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    html = f.read()

# 1. Update defaultCloudColVisibility
old_default_cols = """    const defaultCloudColVisibility = {
      status: true,
      score: true,
      roi: true,
      nous: true,
      or: true,
      coding: true,
      bfcl: true,
      reasoning: true,
      ifeval: true,
      tps: true,
      ctx: true,
      prov: true
    };"""

new_default_cols = """    const defaultCloudColVisibility = {
      status: true,
      score: true,
      roi: true,
      nous: true,
      or: true,
      intel: true,
      coding: true,
      bfcl: true,
      reasoning: true,
      design: true,
      vision: true,
      ifeval: true,
      ttft: true,
      tps: true,
      ctx: true,
      prov: true
    };
    let currentCloudModalityFilter = 'all'; // 'all', 'multimodal', 'text'
    let selectedManufacturers = new Set(); // vazio significa todos

    function setCloudModalityFilter(modality) {
      currentCloudModalityFilter = modality;
      document.querySelectorAll('#cloud-modality-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-cloud-modal-${modality === 'multimodal' ? 'multi' : (modality === 'text' ? 'text' : 'all')}`);
      if (activeBtn) activeBtn.classList.add('active');
      renderCloudRankedTable();
    }

    function toggleMfgMenu(e) {
      if (e) e.stopPropagation();
      const menu = document.getElementById('mfg-dropdown-menu');
      if (!menu) return;
      menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
    }

    function toggleMfg(mfg, isChecked) {
      if (isChecked) {
        selectedManufacturers.add(mfg.toLowerCase());
      } else {
        selectedManufacturers.delete(mfg.toLowerCase());
      }
      updateMfgLabel();
      renderCloudRankedTable();
    }

    function toggleAllMfgs(selectAll) {
      const checkboxes = document.querySelectorAll('#mfg-checkboxes-container input[type="checkbox"]');
      checkboxes.forEach(chk => {
        chk.checked = selectAll;
        const mfg = chk.getAttribute('data-mfg');
        if (selectAll) {
          selectedManufacturers.add(mfg.toLowerCase());
        } else {
          selectedManufacturers.delete(mfg.toLowerCase());
        }
      });
      if (!selectAll) {
        selectedManufacturers.clear();
      }
      updateMfgLabel();
      renderCloudRankedTable();
    }

    function updateMfgLabel() {
      const label = document.getElementById('mfg-filter-label');
      if (!label) return;
      if (selectedManufacturers.size === 0) {
        label.innerText = 'Todos';
      } else {
        label.innerText = `(${selectedManufacturers.size}) Filtrados`;
      }
    }

    function initMfgCheckboxes() {
      const container = document.getElementById('mfg-checkboxes-container');
      if (!container) return;
      const knownMfgs = [
        { id: 'openai', name: 'OpenAI' },
        { id: 'anthropic', name: 'Anthropic' },
        { id: 'google', name: 'Google (Gemini)' },
        { id: 'deepseek', name: 'DeepSeek' },
        { id: 'qwen', name: 'Qwen (Alibaba)' },
        { id: 'meta', name: 'Meta (Llama)' },
        { id: 'mistralai', name: 'Mistral AI' },
        { id: 'cohere', name: 'Cohere' },
        { id: 'xai', name: 'xAI (Grok)' },
        { id: 'moonshot', name: 'Moonshot AI' }
      ];
      container.innerHTML = knownMfgs.map(m => `
        <label class="mfg-checkbox-item">
          <input type="checkbox" data-mfg="${m.id}" onchange="toggleMfg('${m.id}', this.checked)" ${selectedManufacturers.has(m.id) ? 'checked' : ''}>
          <span>${m.name}</span>
        </label>
      `).join('');
    }

    document.addEventListener('click', (e) => {
      const menu = document.getElementById('mfg-dropdown-menu');
      const btn = document.getElementById('btn-mfg-toggle');
      if (menu && menu.style.display !== 'none') {
        if (!menu.contains(e.target) && !btn.contains(e.target)) {
          menu.style.display = 'none';
        }
      }
    });"""

if old_default_cols in html:
    html = html.replace(old_default_cols, new_default_cols)
    print("Replaced defaultCloudColVisibility and added filters")
else:
    print("WARNING: old_default_cols not found")

# 2. Update handleCloudSort and updateHeaderSortIcons
old_sort_fns = """    function handleCloudSort(val) {
      cloudSortOrder = val || 'score_desc';
      if (val === 'score_desc') { currentSortCol = 'score'; currentSortAsc = false; }
      else if (val === 'roi_desc') { currentSortCol = 'roi'; currentSortAsc = false; }
      else if (val === 'nous_asc') { currentSortCol = 'nous_cost'; currentSortAsc = true; }
      else if (val === 'or_asc') { currentSortCol = 'or_cost'; currentSortAsc = true; }
      else if (val === 'coding_desc') { currentSortCol = 'coding'; currentSortAsc = false; }
      else if (val === 'bfcl_desc') { currentSortCol = 'bfcl'; currentSortAsc = false; }
      else if (val === 'reasoning_desc') { currentSortCol = 'reasoning'; currentSortAsc = false; }
      else if (val === 'ifeval_desc') { currentSortCol = 'ifeval'; currentSortAsc = false; }
      else if (val === 'tps_desc') { currentSortCol = 'tps'; currentSortAsc = false; }
      else if (val === 'ctx_desc') { currentSortCol = 'ctx'; currentSortAsc = false; }
      updateHeaderSortIcons();
      renderCloudRankedTable();
    }

    function setTableSort(col) {
      if (currentSortCol === col) {
        currentSortAsc = !currentSortAsc;
      } else {
        currentSortCol = col;
        currentSortAsc = (col === 'nous_cost' || col === 'or_cost' || col === 'name' || col === 'rank');
      }
      updateHeaderSortIcons();
      renderCloudRankedTable();
    }

    function updateHeaderSortIcons() {
      const cols = ['rank', 'name', 'score', 'roi', 'nous_cost', 'or_cost', 'coding', 'bfcl', 'reasoning', 'ifeval', 'tps', 'ctx'];"""

new_sort_fns = """    function handleCloudSort(val) {
      cloudSortOrder = val || 'score_desc';
      if (val === 'score_desc') { currentSortCol = 'score'; currentSortAsc = false; }
      else if (val === 'roi_desc') { currentSortCol = 'roi'; currentSortAsc = false; }
      else if (val === 'nous_asc') { currentSortCol = 'nous_cost'; currentSortAsc = true; }
      else if (val === 'or_asc') { currentSortCol = 'or_cost'; currentSortAsc = true; }
      else if (val === 'intel_desc') { currentSortCol = 'intel'; currentSortAsc = false; }
      else if (val === 'coding_desc') { currentSortCol = 'coding'; currentSortAsc = false; }
      else if (val === 'bfcl_desc') { currentSortCol = 'bfcl'; currentSortAsc = false; }
      else if (val === 'reasoning_desc') { currentSortCol = 'reasoning'; currentSortAsc = false; }
      else if (val === 'design_desc') { currentSortCol = 'design'; currentSortAsc = false; }
      else if (val === 'vision_desc') { currentSortCol = 'vision'; currentSortAsc = false; }
      else if (val === 'ifeval_desc') { currentSortCol = 'ifeval'; currentSortAsc = false; }
      else if (val === 'ttft_asc') { currentSortCol = 'ttft'; currentSortAsc = true; }
      else if (val === 'tps_desc') { currentSortCol = 'tps'; currentSortAsc = false; }
      else if (val === 'ctx_desc') { currentSortCol = 'ctx'; currentSortAsc = false; }
      updateHeaderSortIcons();
      renderCloudRankedTable();
    }

    function setTableSort(col) {
      if (currentSortCol === col) {
        currentSortAsc = !currentSortAsc;
      } else {
        currentSortCol = col;
        currentSortAsc = (col === 'nous_cost' || col === 'or_cost' || col === 'name' || col === 'rank' || col === 'ttft');
      }
      updateHeaderSortIcons();
      renderCloudRankedTable();
    }

    function updateHeaderSortIcons() {
      const cols = ['rank', 'name', 'score', 'roi', 'nous_cost', 'or_cost', 'intel', 'coding', 'bfcl', 'reasoning', 'design', 'vision', 'ifeval', 'ttft', 'tps', 'ctx'];"""

if old_sort_fns in html:
    html = html.replace(old_sort_fns, new_sort_fns)
    print("Updated sort fns and sort icons")
else:
    print("WARNING: old_sort_fns not found")

# 3. Update updateActiveHermesBanner to also populate cloud-quick-active-model
old_update_banner = """            if (nameEl) nameEl.innerText = mb.model || 'deepseek/deepseek-chat';
            if (presetEl) presetEl.innerText = `Preset: ${mb.active_profile_name || 'Frontier Balanced'}`;"""

new_update_banner = """            if (nameEl) nameEl.innerText = mb.model || 'deepseek/deepseek-chat';
            const quickModelEl = document.getElementById('cloud-quick-active-model');
            if (quickModelEl) quickModelEl.innerText = mb.model || 'deepseek/deepseek-chat';
            if (presetEl) presetEl.innerText = `Preset: ${mb.active_profile_name || 'Frontier Balanced'}`;"""

if old_update_banner in html:
    html = html.replace(old_update_banner, new_update_banner)
    print("Updated updateActiveHermesBanner")
else:
    print("WARNING: old_update_banner not found")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(html)
print("Step 3 done")

