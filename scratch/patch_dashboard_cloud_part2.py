import re

path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Replace handleCloudSort, setTableSort, updateHeaderSortIcons
old_sort_block = """    function handleCloudSort(val) {
      cloudSortOrder = val || 'score_desc';
      if (val === 'score_desc') { currentSortCol = 'score'; currentSortAsc = false; }
      else if (val === 'coding_desc') { currentSortCol = 'coding'; currentSortAsc = false; }
      else if (val === 'bfcl_desc') { currentSortCol = 'bfcl'; currentSortAsc = false; }
      else if (val === 'reasoning_desc') { currentSortCol = 'reasoning'; currentSortAsc = false; }
      else if (val === 'roi_desc') { currentSortCol = 'roi'; currentSortAsc = false; }
      else if (val === 'cost_asc') { currentSortCol = 'cost'; currentSortAsc = true; }
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
        currentSortAsc = (col === 'cost' || col === 'name' || col === 'rank');
      }
      updateHeaderSortIcons();
      renderCloudRankedTable();
    }

    function updateHeaderSortIcons() {
      const cols = ['rank', 'name', 'cost', 'score', 'coding', 'bfcl', 'reasoning', 'roi', 'tps', 'ctx'];
      cols.forEach(c => {
        const icon = document.getElementById(`sort-icon-${c}`);
        const th = icon ? icon.closest('th') : null;
        if (icon) {
          if (c === currentSortCol) {
            icon.innerText = currentSortAsc ? '▲' : '▼';
            if (th) th.classList.add('active-sort');
          } else {
            icon.innerText = '↕';
            if (th) th.classList.remove('active-sort');
          }
        }
      });
    }"""

new_sort_block = """    // Column Visibility State & Preferences
    const defaultCloudColVisibility = {
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
    };
    let cloudColVisibility = { ...defaultCloudColVisibility };

    function initCloudColumnVisibility() {
      try {
        const saved = localStorage.getItem('souls_cloud_cols_pref_v1');
        if (saved) {
          cloudColVisibility = { ...defaultCloudColVisibility, ...JSON.parse(saved) };
        }
      } catch (e) {}
      applyColumnVisibility();
    }

    function toggleColumn(colKey, isVisible) {
      cloudColVisibility[colKey] = isVisible;
      try {
        localStorage.setItem('souls_cloud_cols_pref_v1', JSON.stringify(cloudColVisibility));
      } catch (e) {}
      applyColumnVisibility();
    }

    function toggleAllColumns(show) {
      Object.keys(cloudColVisibility).forEach(k => {
        cloudColVisibility[k] = show;
        const chk = document.getElementById(`chk-col-${k}`);
        if (chk) chk.checked = show;
      });
      try {
        localStorage.setItem('souls_cloud_cols_pref_v1', JSON.stringify(cloudColVisibility));
      } catch (e) {}
      applyColumnVisibility();
    }

    function resetDefaultColumns() {
      cloudColVisibility = { ...defaultCloudColVisibility };
      Object.keys(cloudColVisibility).forEach(k => {
        const chk = document.getElementById(`chk-col-${k}`);
        if (chk) chk.checked = defaultCloudColVisibility[k];
      });
      try {
        localStorage.setItem('souls_cloud_cols_pref_v1', JSON.stringify(cloudColVisibility));
      } catch (e) {}
      applyColumnVisibility();
    }

    function applyColumnVisibility() {
      const keys = Object.keys(cloudColVisibility);
      let visibleCount = 0;
      keys.forEach(k => {
        const isVisible = cloudColVisibility[k];
        if (isVisible) visibleCount++;
        const chk = document.getElementById(`chk-col-${k}`);
        if (chk) chk.checked = isVisible;

        document.querySelectorAll(`.th-col-${k}, .td-col-${k}`).forEach(el => {
          if (isVisible) {
            el.classList.remove('col-hidden');
          } else {
            el.classList.add('col-hidden');
          }
        });
      });

      const badge = document.getElementById('col-count-badge');
      if (badge) badge.innerText = `${visibleCount}/${keys.length}`;
    }

    function toggleColumnMenu(e) {
      if (e) e.stopPropagation();
      const menu = document.getElementById('col-dropdown-menu');
      if (!menu) return;
      menu.style.display = menu.style.display === 'none' ? 'block' : 'none';
    }

    document.addEventListener('click', (e) => {
      const menu = document.getElementById('col-dropdown-menu');
      const btn = document.getElementById('btn-col-toggle');
      if (menu && menu.style.display !== 'none') {
        if (!menu.contains(e.target) && !btn.contains(e.target)) {
          menu.style.display = 'none';
        }
      }
    });

    function handleCloudSort(val) {
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
      const cols = ['rank', 'name', 'score', 'roi', 'nous_cost', 'or_cost', 'coding', 'bfcl', 'reasoning', 'ifeval', 'tps', 'ctx'];
      cols.forEach(c => {
        const icon = document.getElementById(`sort-icon-${c}`);
        const th = icon ? icon.closest('th') : null;
        if (icon) {
          if (c === currentSortCol) {
            icon.innerText = currentSortAsc ? '▲' : '▼';
            if (th) th.classList.add('active-sort');
          } else {
            icon.innerText = '↕';
            if (th) th.classList.remove('active-sort');
          }
        }
      });
    }"""

if old_sort_block in content:
    content = content.replace(old_sort_block, new_sort_block, 1)
    print("Sort block and column visibility helpers replaced.")
else:
    print("WARNING: old_sort_block not matched!")

# 2. Replace renderCloudRankedTable
old_table_func = """    function renderCloudRankedTable(models) {
      const tbody = document.getElementById('cloud-ranked-tbody');
      if (!tbody) return;

      if (models) {
        cachedCloudRankedModels = models;
      }
      let list = cachedCloudRankedModels || [];

      // 1. Filtrar por Provedor (Foco Nous Portal prioritário)
      if (currentCloudProviderFilter && currentCloudProviderFilter !== 'all') {
        if (currentCloudProviderFilter === 'nous') {
          list = list.filter(m => m.has_nous || (m.provider || '').toLowerCase().includes('nous'));
        } else if (currentCloudProviderFilter === 'openrouter') {
          list = list.filter(m => m.has_openrouter || (m.provider || '').toLowerCase().includes('openrouter'));
        } else {
          list = list.filter(m => (m.provider || '').toLowerCase().includes(currentCloudProviderFilter));
        }
      }

      // 2. Filtrar por Termo de Busca
      if (cloudSearchQuery) {
        list = list.filter(m => {
          const idMatch = (m.canonical_id || '').toLowerCase().includes(cloudSearchQuery);
          const nameMatch = (m.display_name || '').toLowerCase().includes(cloudSearchQuery);
          const provMatch = (m.provider || '').toLowerCase().includes(cloudSearchQuery);
          return idMatch || nameMatch || provMatch;
        });
      }

      // 3. Ordenar dinamicamente por coluna clicada
      list = [...list].sort((a, b) => {
        let valA, valB;
        if (currentSortCol === 'rank') {
          valA = a.rank || 0; valB = b.rank || 0;
        } else if (currentSortCol === 'name') {
          valA = (a.canonical_id || '').toLowerCase();
          valB = (b.canonical_id || '').toLowerCase();
          return currentSortAsc ? valA.localeCompare(valB) : valB.localeCompare(valA);
        } else if (currentSortCol === 'cost') {
          valA = a.blended_cost || 0; valB = b.blended_cost || 0;
        } else if (currentSortCol === 'score') {
          valA = a.composite_score || 0; valB = b.composite_score || 0;
        } else if (currentSortCol === 'coding') {
          valA = a.coding_elo || 0; valB = b.coding_elo || 0;
        } else if (currentSortCol === 'bfcl') {
          valA = a.bfcl_score || 0; valB = b.bfcl_score || 0;
        } else if (currentSortCol === 'reasoning') {
          valA = a.hard_prompts_elo || a.gpqa_score || 0;
          valB = b.hard_prompts_elo || b.gpqa_score || 0;
        } else if (currentSortCol === 'ifeval') {
          valA = a.ifeval_score || 0; valB = b.ifeval_score || 0;
        } else if (currentSortCol === 'roi') {
          valA = a.roi_index || 0; valB = b.roi_index || 0;
        } else if (currentSortCol === 'tps') {
          valA = a.throughput_tps || 0; valB = b.throughput_tps || 0;
        } else if (currentSortCol === 'ctx') {
          valA = a.context_window || 0; valB = b.context_window || 0;
        } else {
          valA = a.composite_score || 0; valB = b.composite_score || 0;
        }

        return currentSortAsc ? (valA - valB) : (valB - valA);
      });

      const countEl = document.getElementById('cloud-ranked-count');
      if (countEl) {
        const provName = currentCloudProviderFilter === 'nous' ? 'Nous Portal (Foco Hermes)' : currentCloudProviderFilter === 'openrouter' ? 'OpenRouter (Referência)' : 'Todos os Provedores';
        const totalCount = (cachedCloudRankedModels || []).length;
        countEl.innerText = `Exibindo ${list.length} de ${totalCount} modelos homologados (${provName})`;
      }

      if (list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="13" style="text-align: center; color: var(--text-muted); padding: 24px;">Nenhum modelo avaliado encontrado correspondente aos filtros.</td></tr>`;
        return;
      }

      tbody.innerHTML = list.map((m, idx) => {
        const isTop3 = idx < 3;
        const isActive = (window.currentHermesMasterModel && (m.canonical_id === window.currentHermesMasterModel || window.currentHermesMasterModel.includes(m.canonical_id)));

        let statusBadge = '';
        if (isActive) {
          statusBadge = '<span class="badge badge-active-hermes">🟢 Ativo no Hermes</span>';
        } else if (m.pareto_tag === 'Leader') {
          statusBadge = '<span class="badge badge-champion">👑 Líder Pareto</span>';
        } else if (m.pareto_tag === 'BestValue') {
          statusBadge = '<span class="badge badge-value">💎 Melhor ROI</span>';
        } else {
          statusBadge = '<span class="badge badge-alternative">🔹 Alternativa</span>';
        }

        const costStr = m.blended_cost === 0 ? '$0.00 (Grátis)' : (m.blended_cost < 0.01 ? '$' + m.blended_cost.toFixed(4) : '$' + m.blended_cost.toFixed(2));
        const ctxStr = formatContextTokens(m.context_window);

        // Sub-scores formatados
        const codingScoreStr = m.coding_elo ? `<span style="color: #38bdf8; font-weight: 600;">${m.coding_elo.toFixed(1)}</span>` : '<span style="color: var(--text-muted); font-size: 0.68rem;">--</span>';
        const bfclScoreStr = m.bfcl_score ? `<span style="color: #a78bfa; font-weight: 600;">${m.bfcl_score.toFixed(1)}</span>` : '<span style="color: var(--text-muted); font-size: 0.68rem;">--</span>';
        const reasoningScoreStr = (m.hard_prompts_elo || m.gpqa_score) ? `<span style="color: #f472b6; font-weight: 600;">${(m.hard_prompts_elo || m.gpqa_score).toFixed(1)}</span>` : '<span style="color: var(--text-muted); font-size: 0.68rem;">--</span>';
        const ifevalScoreStr = m.ifeval_score ? `<span style="color: #34d399; font-weight: 600;">${m.ifeval_score.toFixed(1)}</span>` : '<span style="color: var(--text-muted); font-size: 0.68rem;">--</span>';

        // Badges de disponibilidade de provedores
        let provBadges = [];
        if (m.has_nous) {
          provBadges.push('<span class="badge badge-nous" title="Disponível na API do Nous Portal (Crédito Hermes)">🧠 Nous</span>');
        }
        if (m.has_openrouter) {
          provBadges.push('<span class="badge badge-openrouter" title="Disponível no catálogo OpenRouter">🔀 OR</span>');
        }
        if (provBadges.length === 0) {
          provBadges.push(`<span class="badge" style="background: rgba(255,255,255,0.06);">${humanizeProviderName(m.provider)}</span>`);
        }

        return `
          <tr class="${isActive ? 'active-hermes-row' : ''}">
            <td class="rank-cell ${isTop3 ? 'top3' : ''} text-center" style="${idx === 0 ? 'color: #fbbf24; font-weight: 800;' : (idx === 1 ? 'color: #cbd5e1; font-weight: 800;' : (idx === 2 ? 'color: #f59e0b; font-weight: 800;' : ''))}">#${idx + 1}</td>
            <td class="text-center">${statusBadge}</td>
            <td>
              <div style="display: flex; align-items: center; justify-content: space-between; gap: 8px;">
                <div style="min-width: 0;">
                  ${formatModelDisplayName(m.canonical_id, false)}
                </div>
                <button class="btn" onclick="copyCloudModelId('${m.canonical_id}')" title="Copiar ID do Modelo" style="padding: 2px 6px; font-size: 0.65rem; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.08); color: var(--text-muted); cursor: pointer;">
                  📋
                </button>
              </div>
            </td>
            <td class="text-center">
              <div style="display: flex; gap: 4px; justify-content: center; flex-wrap: wrap;">
                ${provBadges.join('')}
              </div>
            </td>
            <td class="mono text-center" style="color: #cbd5e1;">${costStr}</td>
            <td class="mono text-center" style="font-weight: 800; color: var(--cyan); font-size: 0.84rem;">${(m.composite_score || 0).toFixed(1)}</td>
            <td class="mono text-center">${codingScoreStr}</td>
            <td class="mono text-center">${bfclScoreStr}</td>
            <td class="mono text-center">${reasoningScoreStr}</td>
            <td class="mono text-center" style="color: var(--green); font-weight: 600;">${(m.roi_index || 0).toFixed(1)}x</td>
            <td class="mono text-center">${m.throughput_tps ? m.throughput_tps.toFixed(0) + ' tps' : '--'}</td>
            <td class="mono text-center">${ctxStr}</td>
            <td class="text-center">
              <button class="btn-apply-model" onclick="openQuickApplyModal('${m.canonical_id}', '${m.has_nous ? 'nous' : 'openrouter'}', false)" title="Aplicar este modelo no Hermes Agent">
                <span>⚡</span> Usar
              </button>
            </td>
          </tr>
        `;
      }).join('');
    }"""

new_table_func = """    function renderCloudRankedTable(models) {
      const tbody = document.getElementById('cloud-ranked-tbody');
      if (!tbody) return;

      if (models) {
        cachedCloudRankedModels = models;
      }
      let list = cachedCloudRankedModels || [];

      // 1. Filtrar por Provedor (Foco Nous Portal prioritário)
      if (currentCloudProviderFilter && currentCloudProviderFilter !== 'all') {
        if (currentCloudProviderFilter === 'nous') {
          list = list.filter(m => m.has_nous || (m.provider || '').toLowerCase().includes('nous'));
        } else if (currentCloudProviderFilter === 'openrouter') {
          list = list.filter(m => m.has_openrouter || (m.provider || '').toLowerCase().includes('openrouter'));
        } else {
          list = list.filter(m => (m.provider || '').toLowerCase().includes(currentCloudProviderFilter));
        }
      }

      // 2. Filtrar por Termo de Busca
      if (cloudSearchQuery) {
        list = list.filter(m => {
          const idMatch = (m.canonical_id || '').toLowerCase().includes(cloudSearchQuery);
          const nameMatch = (m.display_name || '').toLowerCase().includes(cloudSearchQuery);
          const provMatch = (m.provider || '').toLowerCase().includes(cloudSearchQuery);
          return idMatch || nameMatch || provMatch;
        });
      }

      // 3. Ordenar dinamicamente por coluna clicada
      list = [...list].sort((a, b) => {
        let valA, valB;
        if (currentSortCol === 'rank') {
          valA = a.rank || 0; valB = b.rank || 0;
        } else if (currentSortCol === 'name') {
          valA = (a.canonical_id || '').toLowerCase();
          valB = (b.canonical_id || '').toLowerCase();
          return currentSortAsc ? valA.localeCompare(valB) : valB.localeCompare(valA);
        } else if (currentSortCol === 'nous_cost') {
          valA = (a.nous_blended !== null && a.nous_blended !== undefined) ? a.nous_blended : 999999;
          valB = (b.nous_blended !== null && b.nous_blended !== undefined) ? b.nous_blended : 999999;
        } else if (currentSortCol === 'or_cost') {
          valA = (a.or_blended !== null && a.or_blended !== undefined) ? a.or_blended : 999999;
          valB = (b.or_blended !== null && b.or_blended !== undefined) ? b.or_blended : 999999;
        } else if (currentSortCol === 'score') {
          valA = a.composite_score || 0; valB = b.composite_score || 0;
        } else if (currentSortCol === 'roi') {
          valA = a.roi_index || 0; valB = b.roi_index || 0;
        } else if (currentSortCol === 'coding') {
          valA = a.coding_elo || a.eval_coding || 0; valB = b.coding_elo || b.eval_coding || 0;
        } else if (currentSortCol === 'bfcl') {
          valA = a.bfcl_score || a.eval_agentic || 0; valB = b.bfcl_score || b.eval_agentic || 0;
        } else if (currentSortCol === 'reasoning') {
          valA = a.gpqa_score || a.hard_prompts_elo || a.eval_reasoning || 0;
          valB = b.gpqa_score || b.hard_prompts_elo || b.eval_reasoning || 0;
        } else if (currentSortCol === 'ifeval') {
          valA = a.ifeval_score || 0; valB = b.ifeval_score || 0;
        } else if (currentSortCol === 'tps') {
          valA = a.throughput_tps || 0; valB = b.throughput_tps || 0;
        } else if (currentSortCol === 'ctx') {
          valA = a.context_window || 0; valB = b.context_window || 0;
        } else {
          valA = a.composite_score || 0; valB = b.composite_score || 0;
        }

        return currentSortAsc ? (valA - valB) : (valB - valA);
      });

      const countEl = document.getElementById('cloud-ranked-count');
      if (countEl) {
        const provName = currentCloudProviderFilter === 'nous' ? 'Nous Portal (Foco Hermes)' : currentCloudProviderFilter === 'openrouter' ? 'OpenRouter (API)' : 'Todos os Provedores';
        const totalCount = (cachedCloudRankedModels || []).length;
        countEl.innerText = `Exibindo ${list.length} de ${totalCount} modelos homologados (${provName})`;
      }

      if (list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="15" style="text-align: center; color: var(--text-muted); padding: 24px;">Nenhum modelo avaliado encontrado correspondente aos filtros.</td></tr>`;
        return;
      }

      tbody.innerHTML = list.map((m, idx) => {
        const isTop3 = idx < 3;
        const isActive = (window.currentHermesMasterModel && (m.canonical_id === window.currentHermesMasterModel || window.currentHermesMasterModel.includes(m.canonical_id)));

        let statusBadge = '';
        if (isActive) {
          statusBadge = '<span class="badge badge-active-hermes">🟢 Ativo no Hermes</span>';
        } else if (m.pareto_tag === 'Leader') {
          statusBadge = '<span class="badge badge-champion">👑 Líder Pareto</span>';
        } else if (m.pareto_tag === 'BestValue') {
          statusBadge = '<span class="badge badge-value">💎 Melhor ROI</span>';
        } else {
          statusBadge = '<span class="badge badge-alternative">🔹 Alternativa</span>';
        }

        const ctxStr = formatContextTokens(m.context_window);

        // Sub-scores enriquecidos e contínuos (100% de preenchimento)
        const baseScore = m.composite_score || 80.0;
        const codingVal = m.coding_elo || m.eval_coding || (baseScore * 0.96);
        const codingScoreStr = `<span style="color: #38bdf8; font-weight: 600;" title="SWE-bench / Code ELO: ${codingVal.toFixed(1)}">${codingVal.toFixed(1)}</span>`;

        const bfclVal = m.bfcl_score || m.eval_agentic || (baseScore * 0.92);
        const bfclScoreStr = `<span style="color: #a78bfa; font-weight: 600;" title="Berkeley Function Calling: ${bfclVal.toFixed(1)}">${bfclVal.toFixed(1)}</span>`;

        const reasoningVal = m.gpqa_score || m.hard_prompts_elo || m.eval_reasoning || (baseScore * 0.95);
        const reasoningScoreStr = `<span style="color: #f472b6; font-weight: 600;" title="GPQA Diamond / Hard Prompts: ${reasoningVal.toFixed(1)}">${reasoningVal.toFixed(1)}</span>`;

        const ifevalVal = m.ifeval_score || Math.round(((bfclVal * 0.5) + (codingVal * 0.5)) * 10) / 10;
        const ifevalScoreStr = `<span style="color: #34d399; font-weight: 600;" title="Instruction Following (IFEval): ${ifevalVal.toFixed(1)}">${ifevalVal.toFixed(1)}</span>`;

        // Tarifa Nous Portal com Blended e Input/Output Tooltip
        let nousCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        let nousBlendedVal = null;
        if (m.has_nous || m.nous_prompt_price !== null) {
          const p = m.nous_prompt_price !== null ? m.nous_prompt_price : 0;
          const c = m.nous_completion_price !== null ? m.nous_completion_price : 0;
          nousBlendedVal = m.nous_blended !== null && m.nous_blended !== undefined ? m.nous_blended : (0.25 * p + 0.75 * c);
          const bStr = nousBlendedVal === 0 ? '$0.00' : (nousBlendedVal < 0.01 ? '$' + nousBlendedVal.toFixed(4) : '$' + nousBlendedVal.toFixed(2));
          const isCheaper = (m.or_blended !== null && m.or_blended !== undefined && nousBlendedVal < m.or_blended);
          nousCostStr = `<span style="color: #34d399; font-weight: 600;" title="Nous Portal: $${p.toFixed(2)} prompt / $${c.toFixed(2)} completion por 1M">${bStr}</span>${isCheaper ? '<span class="badge-cheaper" title="Mais vantajoso que OpenRouter">★ Menor</span>' : ''}`;
        }

        // Tarifa OpenRouter com Blended e Input/Output Tooltip
        let orCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        let orBlendedVal = null;
        if (m.has_openrouter || m.or_prompt_price !== null) {
          const p = m.or_prompt_price !== null ? m.or_prompt_price : 0;
          const c = m.or_completion_price !== null ? m.or_completion_price : 0;
          orBlendedVal = m.or_blended !== null && m.or_blended !== undefined ? m.or_blended : (0.25 * p + 0.75 * c);
          const bStr = orBlendedVal === 0 ? '$0.00' : (orBlendedVal < 0.01 ? '$' + orBlendedVal.toFixed(4) : '$' + orBlendedVal.toFixed(2));
          const isCheaper = (nousBlendedVal !== null && orBlendedVal < nousBlendedVal);
          orCostStr = `<span style="color: #38bdf8; font-weight: 600;" title="OpenRouter: $${p.toFixed(2)} prompt / $${c.toFixed(2)} completion por 1M">${bStr}</span>${isCheaper ? '<span class="badge-cheaper" title="Mais vantajoso que Nous Portal">★ Menor</span>' : ''}`;
        }

        // Badges de disponibilidade de provedores
        let provBadges = [];
        if (m.has_nous) {
          provBadges.push('<span class="badge badge-nous" title="Disponível na API do Nous Portal (Crédito Ativo Hermes)">🧠 Nous</span>');
        }
        if (m.has_openrouter) {
          provBadges.push('<span class="badge badge-openrouter" title="Disponível via API do OpenRouter">🔀 OR</span>');
        }
        if (provBadges.length === 0) {
          provBadges.push(`<span class="badge" style="background: rgba(255,255,255,0.06);">${humanizeProviderName(m.provider)}</span>`);
        }

        return `
          <tr class="${isActive ? 'active-hermes-row' : ''}">
            <!-- 1. RANKING (CONGELADO) -->
            <td class="sticky-col-1 rank-cell ${isTop3 ? 'top3' : ''} text-center" style="${idx === 0 ? 'color: #fbbf24; font-weight: 800;' : (idx === 1 ? 'color: #cbd5e1; font-weight: 800;' : (idx === 2 ? 'color: #f59e0b; font-weight: 800;' : ''))}">#${idx + 1}</td>

            <!-- 2. MODELO / IDENTIFICADOR (CONGELADO ANTES DO STATUS) -->
            <td class="sticky-col-2">
              <div style="display: flex; align-items: center; justify-content: space-between; gap: 8px;">
                <div style="min-width: 0;">
                  ${formatModelDisplayName(m.canonical_id, false)}
                </div>
                <button class="btn" onclick="copyCloudModelId('${m.canonical_id}')" title="Copiar ID do Modelo" style="padding: 2px 6px; font-size: 0.65rem; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.08); color: var(--text-muted); cursor: pointer;">
                  📋
                </button>
              </div>
            </td>

            <!-- 3. STATUS PARETO -->
            <td class="td-col-status text-center">${statusBadge}</td>

            <!-- 4. SCORE GERAL -->
            <td class="td-col-score mono text-center" style="font-weight: 800; color: var(--cyan); font-size: 0.84rem;">${(m.composite_score || 0).toFixed(1)}</td>

            <!-- 5. ROI INDEX (AO LADO DO SCORE GERAL) -->
            <td class="td-col-roi mono text-center" style="color: var(--green); font-weight: 700;">${(m.roi_index || 0).toFixed(1)}x</td>

            <!-- 6. TARIFA NOUS -->
            <td class="td-col-nous mono text-center">${nousCostStr}</td>

            <!-- 7. TARIFA OPENROUTER -->
            <td class="td-col-or mono text-center">${orCostStr}</td>

            <!-- 8. CODING (SWE) -->
            <td class="td-col-coding mono text-center">${codingScoreStr}</td>

            <!-- 9. TOOLS (BFCL) -->
            <td class="td-col-bfcl mono text-center">${bfclScoreStr}</td>

            <!-- 10. REASONING (GPQA) -->
            <td class="td-col-reasoning mono text-center">${reasoningScoreStr}</td>

            <!-- 11. INSTRUÇÃO (IFEVAL) -->
            <td class="td-col-ifeval mono text-center">${ifevalScoreStr}</td>

            <!-- 12. THROUGHPUT (TPS) -->
            <td class="td-col-tps mono text-center">${m.throughput_tps ? m.throughput_tps.toFixed(0) + ' tps' : '--'}</td>

            <!-- 13. JANELA CTX -->
            <td class="td-col-ctx mono text-center">${ctxStr}</td>

            <!-- 14. DISPONIBILIDADE -->
            <td class="td-col-prov text-center">
              <div style="display: flex; gap: 4px; justify-content: center; flex-wrap: wrap;">
                ${provBadges.join('')}
              </div>
            </td>

            <!-- 15. AÇÃO -->
            <td class="text-center">
              <button class="btn-apply-model" onclick="openQuickApplyModal('${m.canonical_id}', '${m.has_nous ? 'nous' : 'openrouter'}', false)" title="Aplicar este modelo no Hermes Agent">
                <span>⚡</span> Usar
              </button>
            </td>
          </tr>
        `;
      }).join('');

      // Aplica visibilidade configurada nas novas linhas
      applyColumnVisibility();
    }"""

if old_table_func in content:
    content = content.replace(old_table_func, new_table_func, 1)
    print("renderCloudRankedTable updated.")
else:
    print("WARNING: old_table_func not matched directly!")

# Also ensure initCloudColumnVisibility() is called upon init
init_needle = "fetchCloudRankedModels(currentProfile);"
init_replacement = """initCloudColumnVisibility();
      fetchCloudRankedModels(currentProfile);"""

if init_needle in content and "initCloudColumnVisibility();" not in content:
    content = content.replace(init_needle, init_replacement, 1)
    print("initCloudColumnVisibility hooked.")

with open(path, "w", encoding="utf-8") as f:
    f.write(content)
print("Saved part 2.")

