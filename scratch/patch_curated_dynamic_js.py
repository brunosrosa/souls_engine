import re

with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    content = f.read()

# Let's inspect the target block for filterCuratedProfile and renderCuratedCatalog
old_block_start = "    function filterCuratedProfile(profile) {"
assert old_block_start in content, "old_block_start not found"

new_js = """    let currentCuratedViewMode = 'trusted';
    let trendingCatalogData = [];

    function setCuratedViewMode(mode) {
      currentCuratedViewMode = mode;
      const btnTrusted = document.getElementById('btn-curated-mode-trusted');
      const btnTrending = document.getElementById('btn-curated-mode-trending');
      if (btnTrusted && btnTrending) {
        if (mode === 'trusted') {
          btnTrusted.style.background = 'var(--purple)';
          btnTrusted.style.color = '#fff';
          btnTrusted.style.fontWeight = '600';
          btnTrending.style.background = 'transparent';
          btnTrending.style.color = 'var(--text-muted)';
          btnTrending.style.fontWeight = '500';
        } else {
          btnTrending.style.background = 'var(--purple)';
          btnTrending.style.color = '#fff';
          btnTrending.style.fontWeight = '600';
          btnTrusted.style.background = 'transparent';
          btnTrusted.style.color = 'var(--text-muted)';
          btnTrusted.style.fontWeight = '500';
        }
      }

      if (mode === 'trending' && (!trendingCatalogData || trendingCatalogData.length === 0)) {
        loadTrendingCatalog();
      } else {
        renderCuratedCatalog(curatedCatalogData, hubHardwareData);
      }
    }

    async function loadTrendingCatalog() {
      const container = document.getElementById('curated-models-grid');
      if (container) {
        container.innerHTML = '<div style="color: var(--cyan); text-align: center; grid-column: 1 / -1; padding: 30px;"><span style="display:inline-block; animation:spin 1s linear infinite; margin-right:8px;">⏳</span> Buscando novidades em alta no Hugging Face Hub para a sua máquina...</div>';
      }
      try {
        const res = await fetch('/api/v1/inventory/search?query=gguf&limit=25');
        if (res.ok) {
          const data = await res.json();
          const repos = data.results || [];
          trendingCatalogData = repos.map(r => {
            const details = parseRepoModelDetails(r.id, r.tags || []);
            const paramSize = details.activeParams || details.totalParams || 3.0;
            const fit = calculateFitFromParamEstimate(paramSize);
            const isSweet = paramSize <= 4.0;
            const repoSimple = r.id.split('/').pop() || r.id;
            return {
              model_name: repoSimple,
              repo_id: r.id,
              author: r.author || r.id.split('/')[0] || 'Comunidade',
              cognitive_profile: details.specialty === 'coding' ? 'coding' : details.specialty === 'reasoning' ? 'reasoning' : details.specialty === 'vision' ? 'vision' : 'all',
              category_role: isSweet ? 'SweetSpot' : 'Heavyweight',
              why_recommended: `Novidade em alta na comunidade com ${Number(r.downloads || 0).toLocaleString()} downloads e ${r.likes || 0} curtidas.`,
              downloads: r.downloads,
              likes: r.likes,
              paramSize: paramSize,
              is_trending: true,
              variants: [
                {
                  quant_tag: 'Q4_K_M',
                  size_bytes: Math.round(paramSize * 0.65 * 1024 * 1024 * 1024),
                  hardware_fit: fit,
                  is_recommended_fit: true,
                  download_url: `https://huggingface.co/${r.id}/resolve/main/${repoSimple}-Q4_K_M.gguf`,
                  file_name: `${repoSimple}-Q4_K_M.gguf`
                }
              ]
            };
          }).filter(m => m.paramSize <= 14.0);
          renderCuratedCatalog(trendingCatalogData, hubHardwareData);
        } else {
          if (container) container.innerHTML = '<div style="color: var(--text-muted); text-align: center; grid-column: 1 / -1; padding: 20px;">Não foi possível buscar as novidades da nuvem agora.</div>';
        }
      } catch (err) {
        if (container) container.innerHTML = '<div style="color: var(--text-muted); text-align: center; grid-column: 1 / -1; padding: 20px;">Falha de conexão ao buscar novidades da nuvem.</div>';
      }
    }

    async function syncCuratedCatalogLive() {
      const btn = document.getElementById('btn-sync-catalog');
      const syncBadge = document.getElementById('catalog-last-sync-badge');
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:4px;">⏳</span> Sincronizando...';
      }
      try {
        const [recRes, arenaRes] = await Promise.all([
          fetch('/api/v1/inventory/recommendations'),
          fetch('/api/v1/arena/models')
        ]);

        if (recRes.ok) {
          const data = await recRes.json();
          curatedCatalogData = data.catalog || [];
          hubHardwareData = data.hardware;
          hubRecommendationsData = data.triad;
        }

        if (arenaRes.ok) {
          const dataArena = await arenaRes.json();
          arenaModelsData = dataArena.models || [];
        }

        if (currentCuratedViewMode === 'trending') {
          await loadTrendingCatalog();
        } else {
          renderCuratedCatalog(curatedCatalogData, hubHardwareData);
        }
        renderRecommendedTriad(hubRecommendationsData);

        if (syncBadge) {
          const now = new Date();
          const timeStr = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          syncBadge.innerText = `✓ Sincronizado às ${timeStr}`;
          syncBadge.style.display = 'inline-flex';
        }
        showToast('✨ Catálogo e benchmarks empíricos sincronizados com sucesso!', 'success');
      } catch (err) {
        console.error('Falha ao sincronizar catálogo:', err);
        showToast('Catálogo mantido no baseline seguro.', 'warning');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = '<span id="btn-sync-catalog-icon">✨</span> Sincronizar Catálogo';
        }
      }
    }

    function filterCuratedProfile(profile) {
      currentCuratedProfileFilter = profile;
      document.querySelectorAll('#curated-profile-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const activeBtn = Array.from(document.querySelectorAll('#curated-profile-filters .profile-pill')).find(b => {
        if (profile === 'all') return b.innerText.includes('Todos');
        return b.getAttribute('onclick') && b.getAttribute('onclick').includes(`'${profile}'`);
      });
      if (activeBtn) activeBtn.classList.add('active');
      renderCuratedCatalog(currentCuratedViewMode === 'trending' ? trendingCatalogData : curatedCatalogData, hubHardwareData);
    }

    let filteredCuratedModels = [];

    function renderCuratedCatalog(catalog, hw) {
      const container = document.getElementById('curated-models-grid');
      if (!container) return;

      const sourceList = (catalog && catalog.length > 0) ? catalog : (currentCuratedViewMode === 'trending' ? trendingCatalogData : curatedCatalogData);

      filteredCuratedModels = (sourceList || []).filter(m => {
        if (currentCuratedProfileFilter === 'all') return true;
        const prof = (m.cognitive_profile || '').toLowerCase();
        return prof === currentCuratedProfileFilter.toLowerCase();
      });

      if (filteredCuratedModels.length === 0) {
        container.innerHTML = '<div style="color: var(--text-muted); text-align: center; grid-column: 1 / -1; padding: 20px;">Nenhum modelo encontrado para este filtro.</div>';
        return;
      }

      container.innerHTML = filteredCuratedModels.map((m, idx) => {
        const variants = m.variants || [];
        const recommendedVariant = variants.find(v => v.is_recommended_fit) || variants[0] || {};

        // Verificação inteligente com base no inventário real local
        const matched = findInstalledModelMatch(recommendedVariant.file_name, m.repo_id, m.model_name);
        const isDown = Boolean(m.is_downloaded || matched);
        const activeFile = (matched && matched.realFileName) ? matched.realFileName : (m.local_file_path || recommendedVariant.file_name);

        const quantOptions = variants.map(v => `
          <option value="${v.quant_tag}" ${v.is_recommended_fit ? 'selected' : ''} data-size="${v.size_bytes}" data-url="${v.download_url}" data-file="${v.file_name}" data-fit="${v.hardware_fit}" data-sha256="${v.sha256 || ''}" data-proj-file="${v.projector_file || ''}" data-proj-url="${v.projector_url || ''}">
            ${v.quant_tag} (${(v.size_bytes / (1024**3)).toFixed(2)} GB)
          </option>
        `).join('');

        const fitBadge = getFitBadgeHtml(recommendedVariant.hardware_fit);
        const isSweet = m.category_role === 'SweetSpot';

        // Procura se o modelo tem benchmark real medido na Silicon Arena deste PC
        const arenaMatch = (arenaModelsData || []).find(am =>
          (am.file_name && (am.file_name === recommendedVariant.file_name || activeFile.includes(am.file_name))) ||
          (am.name && m.model_name && (am.name.toLowerCase().includes(m.model_name.toLowerCase()) || m.model_name.toLowerCase().includes(am.name.toLowerCase())))
        );

        const hasRealBench = Boolean(arenaMatch && arenaMatch.latest_tps);

        return `
          <div class="curated-card ${isDown ? 'is-installed-card' : ''}" id="curated-card-${idx}">
            <div>
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; flex-wrap: wrap; gap: 8px;">
                <div style="display: flex; gap: 6px; align-items: center; flex-wrap: wrap;">
                  ${m.is_trending ? `
                    <span class="badge" style="background: rgba(249, 115, 22, 0.15); color: #fb923c; border: 1px solid rgba(249, 115, 22, 0.3); font-size: 0.65rem; padding: 2px 7px; font-weight: 700;">
                      🔥 Novidade em Alta
                    </span>
                  ` : `
                    <span class="badge" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.3); font-size: 0.65rem; padding: 2px 7px; font-weight: 700;">
                      ⭐ Mais Confiável
                    </span>
                  `}
                  <span class="badge" style="background: ${isSweet ? 'rgba(16, 185, 129, 0.15)' : 'rgba(168, 85, 247, 0.15)'}; color: ${isSweet ? '#34d399' : 'var(--purple-light)'};">
                    ${isSweet ? '⚡ Sweet Spot (45+ tps)' : '💎 Heavyweight (Teto)'}
                  </span>
                  ${m.is_multimodal_vlm ? '<span class="badge" style="background: rgba(6, 182, 212, 0.15); color: var(--cyan);">👁️ VLM Visão</span>' : ''}
                </div>
                <div style="display: flex; align-items: center; gap: 10px;">
                  <span id="curated-fit-${idx}">${fitBadge}</span>
                  <span id="curated-badge-installed-${idx}">
                    ${isDown ? '<span class="badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.4); font-size: 0.65rem; padding: 2px 8px; font-weight: 700; border-radius: 9999px;">✓ Instalado</span>' : ''}
                  </span>
                </div>
              </div>
              <div style="font-size: 1.02rem; font-weight: 700; color: #fff; font-family: var(--font-mono); margin-top: 4px;">
                ${m.model_name}
              </div>

              <div style="display: flex; flex-direction: column; gap: 4px; font-size: 0.72rem; font-family: var(--font-mono); margin-top: 6px; background: rgba(15, 23, 42, 0.35); padding: 7px 10px; border-radius: 6px; border: 1px solid rgba(255, 255, 255, 0.05);">
                <div>👤 <span style="color: var(--text-muted);">Autor:</span> <strong style="color: #fff;">${m.author || '--'}</strong></div>
                <div>📦 <span style="color: var(--text-muted);">Repo:</span> <span style="color: var(--cyan);">${m.repo_id}</span></div>
                <div style="display: flex; align-items: center; gap: 6px; margin-top: 2px;">
                  <span>🎯</span> <span style="color: var(--text-muted);">Perfil Indicado:</span> ${renderProfileTagBadge(m.cognitive_profile)}
                </div>
              </div>

              <p style="font-size: 0.74rem; color: #cbd5e1; margin-top: 8px; line-height: 1.4;">
                ${m.why_recommended || ''}
              </p>

              <div style="margin-top: 8px; display: flex; flex-wrap: wrap; gap: 4px;">
                ${(m.highlights || []).map(h => `<span class="badge" style="background: rgba(255,255,255,0.05); color: #94a3b8; font-size: 0.65rem;">${h}</span>`).join('')}
              </div>

              <div style="margin-top: 10px; display: grid; grid-template-columns: repeat(2, 1fr); gap: 6px; background: rgba(15,23,42,0.4); padding: 8px; border-radius: var(--radius-sm); font-size: 0.7rem; font-family: var(--font-mono);">
                ${m.is_trending ? `
                  <div>Downloads: <strong style="color: var(--cyan);">${m.downloads ? Number(m.downloads).toLocaleString() : '--'}</strong></div>
                  <div>Likes: <strong style="color: #facc15;">${m.likes || 0}</strong></div>
                  <div>Parâmetros: <strong style="color: #fff;">~${(m.paramSize || 3).toFixed(1)}B</strong></div>
                  <div>Silício: <strong style="color: #34d399;">${isSweet ? '100% VRAM' : 'VRAM + RAM'}</strong></div>
                ` : `
                  <div>Coding: <strong style="color: var(--cyan);">${m.benchmark_scores && m.benchmark_scores.coding_score ? m.benchmark_scores.coding_score.toFixed(1) : '--'}</strong></div>
                  <div>MMLU: <strong style="color: #fff;">${m.benchmark_scores && m.benchmark_scores.mmlu_score ? m.benchmark_scores.mmlu_score.toFixed(1) : '--'}</strong></div>
                  <div>IFEval: <strong style="color: #fff;">${m.benchmark_scores && m.benchmark_scores.ifeval_score ? m.benchmark_scores.ifeval_score.toFixed(1) : '--'}</strong></div>
                  <div>Speed: <strong style="color: #34d399;">~${m.benchmark_scores && m.benchmark_scores.speed_tokens_sec_estimate ? m.benchmark_scores.speed_tokens_sec_estimate.toFixed(0) : '--'} tps</strong></div>
                `}
                ${hasRealBench ? `
                  <div style="grid-column: 1 / -1; background: rgba(16, 185, 129, 0.12); border: 1px solid rgba(16, 185, 129, 0.3); border-radius: 4px; padding: 4px 8px; color: var(--emerald); font-weight: 700; margin-top: 3px; display: flex; justify-content: space-between; align-items: center;">
                    <span>🏅 Silício Local Medido:</span>
                    <span>${arenaMatch.latest_tps.toFixed(1)} tok/s</span>
                  </div>
                ` : ''}
              </div>
            </div>

            <div style="margin-top: 12px; border-top: 1px solid var(--border-subtle); padding-top: 10px;">
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
                <label style="font-size: 0.72rem; color: var(--text-muted);">Quantização:</label>
                <select id="curated-select-${idx}" class="setting-input" style="padding: 3px 8px; font-size: 0.74rem; width: auto;" onchange="onCuratedQuantChange(${idx})">
                  ${quantOptions}
                </select>
              </div>

              <div id="curated-actions-${idx}">
                ${isDown ? `
                  <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                    <button class="btn btn-green" style="flex: 1; min-width: 120px; justify-content: center; font-size: 0.76rem;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                      <span>⚡</span> Usar no Hermes
                    </button>
                    <button class="btn btn-cyan" style="padding: 6px 10px; font-size: 0.74rem;" onclick="testModelInArena('${activeFile}')" title="Testar na Local Silicon Arena">
                      <span>🏟️</span> Arena
                    </button>
                    <button class="btn" style="padding: 6px 10px; font-size: 0.74rem;" title="Baixar novamente" onclick="triggerCuratedDownload(${idx}, '${m.repo_id}')">
                      <span>🔄</span>
                    </button>
                  </div>
                ` : `
                  <button class="btn btn-cyan" style="width: 100%; justify-content: center;" onclick="triggerCuratedDownload(${idx}, '${m.repo_id}')">
                    <span>⬇️</span> ${m.is_multimodal_vlm ? 'Baixar Bundle VLM (GGUF + mmproj)' : 'Baixar Seleção'}
                  </button>
                `}
              </div>
            </div>
          </div>
        `;
      }).join('');
    }"""

# Find exact target region to replace
start_idx = content.find("    function filterCuratedProfile(profile) {")
end_idx = content.find("    function onCuratedQuantChange(idx) {")

assert start_idx != -1 and end_idx != -1, "Target boundaries not found"

content = content[:start_idx] + new_js + "\n\n" + content[end_idx:]

with open("crates/souls_server/src/dashboard_ui.html", "w", encoding="utf-8") as f:
    f.write(content)

print("Curated catalog JS updated with dynamic modes and live sync!")

