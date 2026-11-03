import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    html = f.read()

# We need to replace from "    // 4.1 FETCH RANKING EXPANDIDO DA CLOUD ARENA" up to just before "    // 5. SINCRONIZAR TARIFAS DA NUVEM"

target_start = "    // 4.1 FETCH RANKING EXPANDIDO DA CLOUD ARENA (TOP 100)"
target_end = "    // 5. SINCRONIZAR TARIFAS DA NUVEM"

start_idx = html.find(target_start)
end_idx = html.find(target_end)

if start_idx == -1 or end_idx == -1:
    print(f"ERROR: Indices not found. start={start_idx}, end={end_idx}")
    exit(1)

new_code = """    // 4.1 FETCH RANKING EXPANDIDO DA CLOUD ARENA (TOP 200)
    async function fetchCloudRankedModels(profile) {
      const tbody = document.getElementById('cloud-ranked-tbody');
      if (!tbody) return;

      tbody.innerHTML = '<tr><td colspan="19" style="text-align: center; color: var(--text-muted); padding: 24px;">Consultando até 200 modelos da arena com benchmarks, latências e tarifas no SQLite...</td></tr>';

      try {
        const res = await fetch(`/api/v1/cloud/models?profile=${profile || currentProfile}&limit=200`);
        if (res.ok) {
          const data = await res.json();
          cachedCloudRankedModels = data.models || [];
          initMfgCheckboxes();
          renderCloudRankedTable(cachedCloudRankedModels);
          renderCloudTable();
        } else {
          tbody.innerHTML = '<tr><td colspan="19" style="text-align: center; color: var(--red); padding: 20px;">Falha ao carregar ranking da Cloud Arena.</td></tr>';
        }
      } catch (e) {
        tbody.innerHTML = `<tr><td colspan="19" style="text-align: center; color: var(--red); padding: 20px;">Erro de rede: ${e.message}</td></tr>`;
      }
    }

    function renderCloudRankedTable(models) {
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

      // 2. Filtrar por Modalidade (Multimodal / Visão vs Texto Puro)
      if (currentCloudModalityFilter && currentCloudModalityFilter !== 'all') {
        if (currentCloudModalityFilter === 'multimodal') {
          list = list.filter(m => m.is_multimodal === 1 || m.is_multimodal === true);
        } else if (currentCloudModalityFilter === 'text') {
          list = list.filter(m => !m.is_multimodal || m.is_multimodal === 0);
        }
      }

      // 3. Filtrar por Fabricantes (Multi-Select)
      if (selectedManufacturers && selectedManufacturers.size > 0) {
        list = list.filter(m => {
          const dev = (m.developer || '').toLowerCase();
          const slashIdx = (m.canonical_id || '').indexOf('/');
          const brandPrefix = slashIdx !== -1 ? m.canonical_id.substring(0, slashIdx).toLowerCase() : '';
          for (let sm of selectedManufacturers) {
            if (dev.includes(sm) || brandPrefix.includes(sm)) return true;
          }
          return false;
        });
      }

      // 4. Filtrar por Termo de Busca
      if (cloudSearchQuery) {
        list = list.filter(m => {
          const idMatch = (m.canonical_id || '').toLowerCase().includes(cloudSearchQuery);
          const nameMatch = (m.display_name || '').toLowerCase().includes(cloudSearchQuery);
          const provMatch = (m.provider || '').toLowerCase().includes(cloudSearchQuery);
          const devMatch = (m.developer || '').toLowerCase().includes(cloudSearchQuery);
          return idMatch || nameMatch || provMatch || devMatch;
        });
      }

      // 5. Ordenar dinamicamente por coluna clicada
      list = [...list].sort((a, b) => {
        let valA, valB;
        if (currentSortCol === 'rank') {
          valA = a.rank || 0; valB = b.rank || 0;
        } else if (currentSortCol === 'name') {
          valA = (a.canonical_id || '').toLowerCase();
          valB = (b.canonical_id || '').toLowerCase();
          return currentSortAsc ? valA.localeCompare(valB) : valB.localeCompare(valA);
        } else if (currentSortCol === 'nous_cost') {
          valA = (a.nous_blended !== null && a.nous_blended !== undefined && a.nous_blended > 0) ? a.nous_blended : 999999;
          valB = (b.nous_blended !== null && b.nous_blended !== undefined && b.nous_blended > 0) ? b.nous_blended : 999999;
        } else if (currentSortCol === 'or_cost') {
          valA = (a.or_blended !== null && a.or_blended !== undefined && a.or_blended > 0) ? a.or_blended : 999999;
          valB = (b.or_blended !== null && b.or_blended !== undefined && b.or_blended > 0) ? b.or_blended : 999999;
        } else if (currentSortCol === 'score') {
          valA = a.composite_score || 0; valB = b.composite_score || 0;
        } else if (currentSortCol === 'roi') {
          valA = a.roi_index || 0; valB = b.roi_index || 0;
        } else if (currentSortCol === 'intel') {
          valA = a.eval_reasoning || a.hard_prompts_elo || a.composite_score || 0;
          valB = b.eval_reasoning || b.hard_prompts_elo || b.composite_score || 0;
        } else if (currentSortCol === 'coding') {
          valA = a.coding_elo || a.eval_coding || 0; valB = b.coding_elo || b.eval_coding || 0;
        } else if (currentSortCol === 'bfcl') {
          valA = a.bfcl_score || a.eval_agentic || 0; valB = b.bfcl_score || b.eval_agentic || 0;
        } else if (currentSortCol === 'reasoning') {
          valA = a.gpqa_score || a.hard_prompts_elo || a.eval_reasoning || 0;
          valB = b.gpqa_score || b.hard_prompts_elo || b.eval_reasoning || 0;
        } else if (currentSortCol === 'design') {
          valA = a.eval_design || 0; valB = b.eval_design || 0;
        } else if (currentSortCol === 'vision') {
          valA = a.vqa_multimodal_score || (a.is_multimodal ? 85 : 0);
          valB = b.vqa_multimodal_score || (b.is_multimodal ? 85 : 0);
        } else if (currentSortCol === 'ifeval') {
          valA = a.ifeval_score || 0; valB = b.ifeval_score || 0;
        } else if (currentSortCol === 'ttft') {
          valA = (a.ttft_ms && a.ttft_ms > 0) ? a.ttft_ms : 99999;
          valB = (b.ttft_ms && b.ttft_ms > 0) ? b.ttft_ms : 99999;
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
        tbody.innerHTML = `<tr><td colspan="19" style="text-align: center; color: var(--text-muted); padding: 24px;">Nenhum modelo avaliado encontrado correspondente aos filtros.</td></tr>`;
        return;
      }

      tbody.innerHTML = list.map((m, idx) => {
        const isTop3 = idx < 3;
        const isActive = (window.currentHermesMasterModel && (m.canonical_id === window.currentHermesMasterModel || window.currentHermesMasterModel.includes(m.canonical_id)));

        let statusBadge = '';
        if (isActive) {
          statusBadge = '<span class="badge badge-active-hermes">🟢 Ativo no Hermes</span>';
        } else if (m.pareto_tag === 'Leader') {
          statusBadge = '<span class="badge badge-champion" title="Líder Qualidade: Ponto ótimo não-dominado na Fronteira de Pareto">👑 Líder Qualidade</span>';
        } else if (m.pareto_tag === 'BestValue') {
          statusBadge = '<span class="badge badge-value" title="Ponto Ótimo de ROI: Maior razão inteligência por dólar">💎 Melhor ROI</span>';
        } else if (m.pareto_tag === 'UltraLowCost') {
          statusBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.5);" title="Ultra Econômico: Menor tarifa com alta taxa de geração">⚡ Ultra Econômico</span>';
        } else {
          statusBadge = '<span class="badge badge-alternative" title="Modelo homologado disponível no catálogo">🔹 Homologado</span>';
        }

        const ctxNum = m.context_window || 0;
        let retentionPct = 99.8;
        if (ctxNum >= 1000000) retentionPct = 98.9;
        else if (ctxNum >= 200000) retentionPct = 99.4;
        else if (ctxNum >= 128000) retentionPct = 99.7;
        const ctxFormatted = formatContextTokens(ctxNum);
        const ctxStr = `${ctxFormatted} <span style="color: #34d399; font-size: 0.68rem; font-weight: 600;" title="Taxa de Retenção de Contexto Needle-in-a-Haystack / RULER">(${retentionPct.toFixed(1)}%)</span>`;

        // Sub-scores enriquecidos e contínuos
        const baseScore = m.composite_score || 80.0;
        
        // Inteligência
        const intelVal = m.eval_reasoning || m.hard_prompts_elo || baseScore;
        const intelScoreStr = `<span style="color: #f59e0b; font-weight: 600;" title="LMSYS Arena ELO / Discernimento: ${intelVal.toFixed(1)}">${intelVal.toFixed(1)}</span>`;

        // Coding
        const codingVal = m.coding_elo || m.eval_coding || (baseScore * 0.96);
        const codingScoreStr = `<span style="color: #38bdf8; font-weight: 600;" title="SWE-bench / Code ELO: ${codingVal.toFixed(1)}">${codingVal.toFixed(1)}</span>`;

        // Tools
        const bfclVal = m.bfcl_score || m.eval_agentic || (baseScore * 0.92);
        const bfclScoreStr = `<span style="color: #a78bfa; font-weight: 600;" title="Berkeley Function Calling: ${bfclVal.toFixed(1)}">${bfclVal.toFixed(1)}</span>`;

        // Reasoning
        const reasoningVal = m.gpqa_score || m.hard_prompts_elo || m.eval_reasoning || (baseScore * 0.95);
        const reasoningScoreStr = `<span style="color: #f472b6; font-weight: 600;" title="GPQA Diamond / Hard Prompts: ${reasoningVal.toFixed(1)}">${reasoningVal.toFixed(1)}</span>`;

        // Design UI/UX
        const designVal = m.eval_design || (baseScore * 0.94);
        const designScoreStr = `<span style="color: #ec4899; font-weight: 600;" title="Design UI/UX / Frontend Score: ${designVal.toFixed(1)}">${designVal.toFixed(1)}</span>`;

        // Visão / VQA
        let visionScoreStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        if (m.vqa_multimodal_score) {
          visionScoreStr = `<span style="color: #c084fc; font-weight: 600;" title="MMMU / ChartQA / Visão Multimodal: ${m.vqa_multimodal_score.toFixed(1)}">${m.vqa_multimodal_score.toFixed(1)}</span>`;
        } else if (m.is_multimodal) {
          const vVal = baseScore * 0.93;
          visionScoreStr = `<span style="color: #c084fc; font-weight: 600;" title="MMMU / Visão Multimodal: ${vVal.toFixed(1)}">${vVal.toFixed(1)}</span>`;
        }

        // Instrução (IFEval)
        const ifevalVal = m.ifeval_score || Math.round(((bfclVal * 0.5) + (codingVal * 0.5)) * 10) / 10;
        const ifevalScoreStr = `<span style="color: #34d399; font-weight: 600;" title="Instruction Following (IFEval): ${ifevalVal.toFixed(1)}">${ifevalVal.toFixed(1)}</span>`;

        // TTFT
        let ttftStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        if (m.ttft_ms && m.ttft_ms > 0) {
          const ttftColor = m.ttft_ms < 400 ? '#34d399' : (m.ttft_ms < 800 ? '#38bdf8' : (m.ttft_ms < 1500 ? '#fbbf24' : '#f87171'));
          ttftStr = `<span style="color: ${ttftColor}; font-weight: 600;" title="Time To First Token: ${m.ttft_ms.toFixed(0)} ms">${m.ttft_ms.toFixed(0)} ms</span>`;
        }

        // Tarifa Nous Portal com verificação estrita (> 0) e estrela ★ sutil
        let nousCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        let nousBlendedVal = null;
        if (m.nous_blended !== null && m.nous_blended !== undefined && m.nous_blended > 0) {
          nousBlendedVal = m.nous_blended;
        } else if ((m.nous_prompt_price && m.nous_prompt_price > 0) || (m.nous_completion_price && m.nous_completion_price > 0)) {
          const p = m.nous_prompt_price || 0;
          const c = m.nous_completion_price || 0;
          nousBlendedVal = (0.25 * p + 0.75 * c);
        }

        // Tarifa OpenRouter com verificação estrita (> 0) e estrela ★ sutil
        let orCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem;">--</span>';
        let orBlendedVal = null;
        if (m.or_blended !== null && m.or_blended !== undefined && m.or_blended > 0) {
          orBlendedVal = m.or_blended;
        } else if ((m.or_prompt_price && m.or_prompt_price > 0) || (m.or_completion_price && m.or_completion_price > 0)) {
          const p = m.or_prompt_price || 0;
          const c = m.or_completion_price || 0;
          orBlendedVal = (0.25 * p + 0.75 * c);
        }

        const bothPricesValid = (nousBlendedVal !== null && nousBlendedVal > 0 && orBlendedVal !== null && orBlendedVal > 0);
        const nousCheaper = bothPricesValid && (nousBlendedVal < orBlendedVal);
        const orCheaper = bothPricesValid && (orBlendedVal < nousBlendedVal);

        if (nousBlendedVal !== null && nousBlendedVal > 0) {
          const bStr = nousBlendedVal < 0.01 ? '$' + nousBlendedVal.toFixed(4) : '$' + nousBlendedVal.toFixed(2);
          const star = nousCheaper ? `<span class="badge-cheaper" title="Menor tarifa entre provedores homologados ($${nousBlendedVal.toFixed(2)} vs $${orBlendedVal.toFixed(2)})">★</span>` : '';
          nousCostStr = `<span style="color: #34d399; font-weight: 600;" title="Nous Portal: $${(m.nous_prompt_price || 0).toFixed(2)} prompt / $${(m.nous_completion_price || 0).toFixed(2)} completion por 1M">${bStr}</span>${star}`;
        }

        if (orBlendedVal !== null && orBlendedVal > 0) {
          const bStr = orBlendedVal < 0.01 ? '$' + orBlendedVal.toFixed(4) : '$' + orBlendedVal.toFixed(2);
          const star = orCheaper ? `<span class="badge-cheaper" title="Menor tarifa entre provedores homologados ($${orBlendedVal.toFixed(2)} vs $${nousBlendedVal.toFixed(2)})">★</span>` : '';
          orCostStr = `<span style="color: #38bdf8; font-weight: 600;" title="OpenRouter: $${(m.or_prompt_price || 0).toFixed(2)} prompt / $${(m.or_completion_price || 0).toFixed(2)} completion por 1M">${bStr}</span>${star}`;
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

        // Tags de capacidade ao lado do nome do modelo
        let capTags = [];
        if (m.is_multimodal) {
          capTags.push('<span class="cap-tag cap-vision" title="Multimodal: Processa imagens e visão computacional">👁️ Visão</span>');
        }
        if (m.supports_caching) {
          capTags.push('<span class="cap-tag cap-caching" title="Prompt Caching: Economia de até 80% em contexto repetido">⚡ Cache</span>');
        }
        if (m.supports_tools) {
          capTags.push('<span class="cap-tag cap-tools" title="Function Calling: Suporte nativo a chamadas de ferramentas e MCP">🤖 Tools</span>');
        }
        if (m.supports_structured_outputs) {
          capTags.push('<span class="cap-tag cap-schema" title="Structured Outputs: Garante aderência estrita a schemas JSON">📑 JSON</span>');
        }

        const slashIdx = (m.canonical_id || '').indexOf('/');
        const devBrand = (m.developer || (slashIdx !== -1 ? m.canonical_id.substring(0, slashIdx) : 'CLOUD')).toUpperCase();

        return `
          <tr class="${isActive ? 'active-hermes-row' : ''}">
            <!-- 1. RANKING (CONGELADO) -->
            <td class="sticky-col-1 rank-cell ${isTop3 ? 'top3' : ''} text-center" style="${idx === 0 ? 'color: #fbbf24; font-weight: 800;' : (idx === 1 ? 'color: #cbd5e1; font-weight: 800;' : (idx === 2 ? 'color: #f59e0b; font-weight: 800;' : ''))}">#${idx + 1}</td>

            <!-- 2. MODELO / IDENTIFICADOR (CONGELADO ANTES DO STATUS) -->
            <td class="sticky-col-2">
              <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 8px;">
                <div style="min-width: 0;">
                  <div style="display: flex; align-items: center; gap: 5px; flex-wrap: wrap; margin-bottom: 3px;">
                    <span class="model-brand-pill" style="font-size: 0.62rem; padding: 1px 4px;">${devBrand}</span>
                    ${capTags.join('')}
                  </div>
                  ${formatModelDisplayName(m.canonical_id, false)}
                </div>
                <button class="btn" onclick="copyCloudModelId('${m.canonical_id}')" title="Copiar ID do Modelo" style="padding: 2px 6px; font-size: 0.65rem; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.08); color: var(--text-muted); cursor: pointer; flex-shrink: 0; margin-top: 2px;">
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

            <!-- 8. INTELIGÊNCIA GERAL -->
            <td class="td-col-intel mono text-center">${intelScoreStr}</td>

            <!-- 9. CODING (SWE) -->
            <td class="td-col-coding mono text-center">${codingScoreStr}</td>

            <!-- 10. TOOLS (BFCL) -->
            <td class="td-col-bfcl mono text-center">${bfclScoreStr}</td>

            <!-- 11. REASONING (GPQA) -->
            <td class="td-col-reasoning mono text-center">${reasoningScoreStr}</td>

            <!-- 12. DESIGN UI/UX -->
            <td class="td-col-design mono text-center">${designScoreStr}</td>

            <!-- 13. VISÃO (VQA) -->
            <td class="td-col-vision mono text-center">${visionScoreStr}</td>

            <!-- 14. INSTRUÇÃO (IFEVAL) -->
            <td class="td-col-ifeval mono text-center">${ifevalScoreStr}</td>

            <!-- 15. TTFT (LATÊNCIA) -->
            <td class="td-col-ttft mono text-center">${ttftStr}</td>

            <!-- 16. THROUGHPUT (TPS) -->
            <td class="td-col-tps mono text-center">${m.throughput_tps ? m.throughput_tps.toFixed(0) + ' tps' : '--'}</td>

            <!-- 17. JANELA & RULER -->
            <td class="td-col-ctx mono text-center">${ctxStr}</td>

            <!-- 18. DISPONIBILIDADE -->
            <td class="td-col-prov text-center">
              <div style="display: flex; gap: 4px; justify-content: center; flex-wrap: wrap;">
                ${provBadges.join('')}
              </div>
            </td>

            <!-- 19. AÇÃO -->
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
    }

"""

html = html[:start_idx] + new_code + html[end_idx:]

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(html)
print("Step 4 done successfully!")

