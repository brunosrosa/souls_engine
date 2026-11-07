    // =========================================================================
    // SKILLS HUB — GOVERNANCA MULTI-AMBIENTE, EXPLORADOR 101k+, ESTUDIO & CUIDADOS
    // =========================================================================

    let skillsHubState = {
      initialized: false,
      currentSubtab: 'installed',
      surfaceFilter: 'all',
      sortCol: 'name',
      sortAsc: true,
      overview: null,
      skills: [],
      personas: [],
      availableSurfaces: [],
      inspectedSlug: null,
      inspectedRecord: null,
      inspectedHistory: [],
      marketplaceOffset: 0,
      marketplaceLimit: 35,
      marketplaceTotal: 0,
      installedSlugsSet: new Set(),
      recommendations: [],
      recommendationsHidden: false,
      quarantineItems: [],
      activeQuarantineSlug: null,
      mcpTools: [],
      forgeSelectedTools: new Set(),
      forgeLastResult: null,
      mpSearchTimer: null
    };

    const SKILL_CATEGORY_LABELS = {
      'code': '💻 Código & Dev',
      'mcp': '🔌 Integração MCP',
      'security': '🛡️ Segurança',
      'finance': '💰 Finanças',
      'health': '🩺 Saúde',
      'notes': '📝 Notas & Docs',
      'automation': '🤖 Automação',
      'ai-data': '🧠 IA & Dados',
      'productivity': '⚡ Produtividade',
      'general': '📦 Geral'
    };

    function formatCategoryLabel(cat) {
      if (!cat) return '📦 Geral';
      const clean = String(cat).toLowerCase();
      return SKILL_CATEGORY_LABELS[clean] || cat;
    }

    function formatTrustLabel(trust) {
      const t = String(trust || '').toLowerCase();
      if (t === 'builtin' || t === 'official') return { label: 'Oficial', color: '#4ade80', bg: 'rgba(34,197,94,0.12)', border: 'rgba(34,197,94,0.3)' };
      if (t === 'trusted' || t === 'verified') return { label: 'Verificada', color: '#38bdf8', bg: 'rgba(56,189,248,0.12)', border: 'rgba(56,189,248,0.3)' };
      return { label: 'Comunidade', color: '#cbd5e1', bg: 'rgba(148,163,184,0.12)', border: 'rgba(148,163,184,0.26)' };
    }

    function formatSourceLabel(src) {
      const s = String(src || '').toLowerCase();
      if (s === 'official') return 'Hermes Oficial';
      if (s === 'github' || s === 'github-live') return 'GitHub';
      if (s === 'clawhub') return 'ClawHub';
      if (s === 'skills.sh') return 'skills.sh';
      if (s === 'lobehub') return 'LobeHub';
      if (s === 'browse-sh') return 'browse.sh';
      return src || 'Catálogo';
    }

    function escapeSkillsHtml(str) {
      return String(str == null ? '' : str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    }

    function showSkillsToast(msg, kind) {
      if (typeof showToast === 'function') {
        showToast(msg, kind || 'info');
      } else {
        console.log('[SkillsHub]', kind || 'info', msg);
      }
    }

    function initSkillsHubTab() {
      fetchInstalledSkills(!skillsHubState.initialized);
      if (!skillsHubState.initialized) {
        skillsHubState.initialized = true;
        fetchQuarantineList();
        loadForgeLocalModels();
        loadForgeMcpCatalog();
      }
    }

    function switchSkillsSubtab(subtab) {
      skillsHubState.currentSubtab = subtab;
      const subtabs = ['installed', 'explorer', 'forge', 'health'];
      subtabs.forEach(k => {
        const btn = document.getElementById('btn-skills-subtab-' + k);
        const panel = document.getElementById('panel-skills-' + k);
        if (btn) btn.classList.toggle('active', k === subtab);
        if (panel) panel.style.display = (k === subtab) ? 'block' : 'none';
      });

      if (subtab === 'installed') {
        fetchInstalledSkills(false);
      } else if (subtab === 'explorer') {
        runMarketplaceSearch(skillsHubState.marketplaceOffset);
        fetchQuarantineList();
      } else if (subtab === 'forge') {
        populateForgeSkillPicker();
        loadForgeMcpCatalog();
        loadForgeLocalModels();
        renderTargetCheckboxes('forge-publish-targets', ['workspace', 'hermes_global']);
      } else if (subtab === 'health') {
        renderSkillsHealthRadar();
      }
    }

    // =========================================================================
    // 1. MINHAS SKILLS — INVENTARIO MULTI-AMBIENTE, ORDENACAO & AUTO-CORRECAO
    // =========================================================================

    async function fetchInstalledSkills(forceToast) {
      try {
        const res = await fetch('/api/v1/skills/installed');
        const data = await res.json();
        if (data.status !== 'success' || !data.overview) {
          if (forceToast) showSkillsToast(data.message || 'Não foi possível atualizar suas skills.', 'error');
          return;
        }

        const ov = data.overview;
        skillsHubState.overview = ov;
        skillsHubState.skills = ov.skills || [];
        skillsHubState.personas = ov.detected_personas || [];
        skillsHubState.availableSurfaces = (ov.available_surfaces || []).filter(s => s.installed_on_host);
        skillsHubState.installedSlugsSet = new Set((ov.skills || []).map(s => s.slug));

        updateSkillsHubKpis(ov);
        renderSurfaceFilterPills();
        updateSortIndicators();
        renderInstalledSkillsTable();

        if (skillsHubState.currentSubtab === 'health') {
          renderSkillsHealthRadar();
        }
        if (forceToast) {
          showSkillsToast(`Lista atualizada: ${ov.total_unique_skills} habilidades verificadas.`, 'success');
        }
      } catch (err) {
        console.error('Erro em fetchInstalledSkills:', err);
        if (forceToast) showSkillsToast('Falha de comunicação com o servidor local.', 'error');
      }
    }

    function updateSkillsHubKpis(ov) {
      const setTxt = (id, v) => {
        const el = document.getElementById(id);
        if (el) el.textContent = v;
      };

      setTxt('skills-kpi-total', String(ov.total_unique_skills || 0));
      setTxt('skills-kpi-synced', `${ov.synced_skills_count || 0} em dia`);
      setTxt('skills-kpi-drifted', `${ov.drifted_skills_count || 0} diferentes`);
      setTxt('skills-kpi-paused', `${ov.disabled_skills_count || 0} pausadas`);
      setTxt('skills-kpi-installs', `${ov.total_surface_installations || 0} cópias ativas nos ambientes`);
      setTxt('skills-kpi-boot-tokens', `${(ov.total_level1_boot_tokens || 0).toLocaleString('pt-BR')} tokens`);
      setTxt('skills-kpi-l2-tokens', (ov.total_level2_body_tokens || 0).toLocaleString('pt-BR'));
      setTxt('skills-kpi-marketplace-total', (ov.marketplace_indexed_count || 0).toLocaleString('pt-BR'));
      setTxt('skills-kpi-quarantine-count', String(ov.quarantine_count || 0));
      setTxt('skills-kpi-avg-score', `${Math.round(ov.avg_quality_score || 0)}/100`);

      const over60Count = (ov.skills || []).filter(s => s.enabled !== false && (s.description_len || 0) > 60).length;
      setTxt('skills-kpi-over60', `${over60Count} resumos longos`);
      setTxt('skills-kpi-zombies', `${ov.zombie_skills_count || 0} sem uso recente`);

      setTxt('badge-skills-installed-count', String(ov.total_unique_skills || 0));

      const qPill = document.getElementById('badge-skills-quarantine-pill');
      if (qPill) {
        if ((ov.quarantine_count || 0) > 0) {
          qPill.textContent = `${ov.quarantine_count} em revisão`;
          qPill.style.display = 'inline-block';
        } else {
          qPill.style.display = 'none';
        }
      }

      const alertsTotal = (ov.caution_or_dangerous_count || 0) + (ov.drifted_skills_count || 0) + over60Count;
      const hPill = document.getElementById('badge-skills-health-alerts');
      if (hPill) {
        if (alertsTotal > 0) {
          hPill.textContent = String(alertsTotal);
          hPill.style.display = 'inline-block';
        } else {
          hPill.style.display = 'none';
        }
      }
    }

    function getInstalledHostGroups() {
      const avail = skillsHubState.availableSurfaces || [];
      if (avail.length === 0) {
        return {
          hasWorkspace: true,
          workspaceLabel: 'Souls Local',
          workspaceBadge: 'WS',
          hasHermes: true,
          hasPersonas: (skillsHubState.personas || []).length > 0,
          hasAntigravity: false
        };
      }
      const ws = avail.find(a => a.group === 'workspace');
      return {
        hasWorkspace: !!ws,
        workspaceLabel: ws ? ws.label.replace(/^🦀\s*/, '').replace(/^📦\s*/, '') : 'Souls Local',
        workspaceBadge: ws ? (ws.short_badge || 'WS') : 'WS',
        hasHermes: avail.some(a => a.group === 'hermes_global'),
        hasPersonas: avail.some(a => a.group === 'personas'),
        hasAntigravity: avail.some(a => a.group === 'antigravity')
      };
    }

    function renderSurfaceFilterPills() {
      const bar = document.getElementById('skills-surface-filters');
      if (!bar) return;
      const g = getInstalledHostGroups();
      const pills = [
        { key: 'all', label: 'Todas' }
      ];
      if (g.hasWorkspace) pills.push({ key: 'workspace', label: `🦀 ${g.workspaceLabel}` });
      if (g.hasHermes) pills.push({ key: 'hermes_global', label: '🧠 Hermes' });
      if (g.hasPersonas) pills.push({ key: 'personas', label: `🎭 Personas (${(skillsHubState.personas || []).length})` });
      if (g.hasAntigravity) pills.push({ key: 'antigravity', label: '🚀 Antigravity' });

      const driftCount = (skillsHubState.skills || []).filter(s => !s.in_sync).length;
      const pausedCount = (skillsHubState.skills || []).filter(s => s.enabled === false).length;
      if (driftCount > 0) pills.push({ key: 'drift', label: `⚡ Diferentes (${driftCount})` });
      if (pausedCount > 0) pills.push({ key: 'paused', label: `⏸️ Pausadas (${pausedCount})` });
      pills.push({ key: 'pinned', label: '📌 Fixadas' });

      // Reset filter if previously selected filter no longer exists
      if (!pills.some(p => p.key === skillsHubState.surfaceFilter)) {
        skillsHubState.surfaceFilter = 'all';
      }

      bar.innerHTML = pills.map(p => {
        const active = skillsHubState.surfaceFilter === p.key ? 'active' : '';
        return `<button class="subtab-btn ${active}" style="padding:5px 10px; font-size:0.74rem;" onclick="setSkillsSurfaceFilter('${p.key}')">${escapeSkillsHtml(p.label)}</button>`;
      }).join('');
    }

    function setSkillsSurfaceFilter(filterKey) {
      skillsHubState.surfaceFilter = filterKey;
      renderSurfaceFilterPills();
      renderInstalledSkillsTable();
    }

    function toggleInstalledSkillsSort(colKey) {
      if (skillsHubState.sortCol === colKey) {
        skillsHubState.sortAsc = !skillsHubState.sortAsc;
      } else {
        skillsHubState.sortCol = colKey;
        // Natural default direction per column
        if (colKey === 'name') {
          skillsHubState.sortAsc = true;
        } else {
          skillsHubState.sortAsc = false;
        }
      }
      updateSortIndicators();
      renderInstalledSkillsTable();
    }

    function updateSortIndicators() {
      const cols = ['name', 'surfaces', 'sync', 'tokens', 'usage', 'score', 'security'];
      cols.forEach(c => {
        const ind = document.getElementById('sort-ind-' + c);
        if (!ind) return;
        const th = ind.closest('th');
        if (c === skillsHubState.sortCol) {
          ind.textContent = skillsHubState.sortAsc ? '↑' : '↓';
          if (th) th.classList.add('sorted');
        } else {
          ind.textContent = '↕';
          if (th) th.classList.remove('sorted');
        }
      });
    }

    function securitySeverityRank(s) {
      const v = String(s.security_verdict || 'safe').toLowerCase();
      let rank = 0;
      if (v === 'dangerous') rank = 30;
      else if (v === 'caution') rank = 20;
      if (s.win11_compatible === false) rank += 5;
      return rank;
    }

    function syncStateRank(s) {
      if (s.enabled === false) return 1;
      if (!s.in_sync) return 3;
      return 2;
    }

    function sortInstalledSkills(list) {
      const col = skillsHubState.sortCol || 'name';
      const dir = skillsHubState.sortAsc ? 1 : -1;
      return list.slice().sort((a, b) => {
        // Keep pinned at top only when sorting by name default, otherwise honor user's explicit column click
        if (col === 'name') {
          return a.slug.localeCompare(b.slug) * dir;
        }
        if (col === 'surfaces') {
          const diff = ((a.surfaces || []).length - (b.surfaces || []).length) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        if (col === 'sync') {
          const diff = (syncStateRank(a) - syncStateRank(b)) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        if (col === 'tokens') {
          const ta = a.enabled === false ? 0 : (a.level1_tokens || 0);
          const tb = b.enabled === false ? 0 : (b.level1_tokens || 0);
          const diff = (ta - tb) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        if (col === 'usage') {
          const ua = (a.usage && a.usage.use_count) || 0;
          const ub = (b.usage && b.usage.use_count) || 0;
          const diff = (ua - ub) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        if (col === 'score') {
          const diff = ((a.quality_score || 0) - (b.quality_score || 0)) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        if (col === 'security') {
          const diff = (securitySeverityRank(a) - securitySeverityRank(b)) * dir;
          return diff !== 0 ? diff : a.slug.localeCompare(b.slug);
        }
        return a.slug.localeCompare(b.slug);
      });
    }

    function buildSurfaceMatrixBadges(s) {
      const g = getInstalledHostGroups();
      const targets = new Set((s.surfaces || []).map(x => x.target_key));
      const personaCount = (s.surfaces || []).filter(x => String(x.target_key || '').startsWith('persona:')).length;
      const pills = [];

      const mkBadge = (present, text, activeStyle, title) => {
        if (present) {
          return `<span title="${escapeSkillsHtml(title)}" style="padding:2px 6px; border-radius:5px; font-size:0.68rem; font-weight:700; ${activeStyle}">${escapeSkillsHtml(text)}</span>`;
        }
        return `<span title="${escapeSkillsHtml(title)} (não instalada aqui)" style="padding:2px 6px; border-radius:5px; font-size:0.68rem; color:#475569; border:1px solid rgba(148,163,184,0.12); background:rgba(15,23,42,0.35); opacity:0.55;">${escapeSkillsHtml(text)}</span>`;
      };

      if (g.hasWorkspace) {
        pills.push(mkBadge(
          targets.has('workspace'),
          g.workspaceBadge,
          'background:rgba(56,189,248,0.14); color:#38bdf8; border:1px solid rgba(56,189,248,0.35);',
          g.workspaceLabel
        ));
      }
      if (g.hasHermes) {
        pills.push(mkBadge(
          targets.has('hermes_global'),
          'HG',
          'background:rgba(192,132,252,0.14); color:#c084fc; border:1px solid rgba(192,132,252,0.35);',
          'Hermes Principal'
        ));
      }
      if (g.hasPersonas) {
        pills.push(mkBadge(
          personaCount > 0,
          personaCount > 0 ? `P:${personaCount}` : 'P',
          'background:rgba(244,114,182,0.14); color:#f472b6; border:1px solid rgba(244,114,182,0.35);',
          personaCount > 0 ? `Ativa em ${personaCount} Persona(s)` : 'Personas Hermes'
        ));
      }
      if (g.hasAntigravity) {
        pills.push(mkBadge(
          targets.has('antigravity'),
          'AGY',
          'background:rgba(52,211,153,0.14); color:#34d399; border:1px solid rgba(52,211,153,0.35);',
          'Antigravity Global'
        ));
      }

      return `<div style="display:flex; justify-content:center; gap:4px; flex-wrap:nowrap; white-space:nowrap;">${pills.join('')}</div>`;
    }

    function buildFriendlySecurityBadge(s) {
      const verdict = String(s.security_verdict || 'safe').toLowerCase();
      const winOk = s.win11_compatible !== false;
      const summary = s.security_human_summary || 'Habilidade verificada sem alertas.';

      if (verdict === 'dangerous') {
        return `<span onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')" title="${escapeSkillsHtml(summary)} — Clique para ver os detalhes" style="cursor:pointer; display:inline-block; padding:3px 8px; border-radius:999px; font-size:0.7rem; font-weight:700; background:rgba(239,68,68,0.16); color:#f87171; border:1px solid rgba(239,68,68,0.4);">🛑 Revisar</span>`;
      }
      if (verdict === 'caution' && !winOk) {
        return `<span onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')" title="${escapeSkillsHtml(summary)} — Clique para entender o motivo" style="cursor:pointer; display:inline-block; padding:3px 8px; border-radius:999px; font-size:0.7rem; font-weight:700; background:rgba(245,158,11,0.15); color:#fbbf24; border:1px solid rgba(245,158,11,0.38);">⚠️ Atenção • Mac/Linux</span>`;
      }
      if (verdict === 'caution') {
        return `<span onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')" title="${escapeSkillsHtml(summary)} — Clique para entender o motivo" style="cursor:pointer; display:inline-block; padding:3px 8px; border-radius:999px; font-size:0.7rem; font-weight:700; background:rgba(245,158,11,0.15); color:#fbbf24; border:1px solid rgba(245,158,11,0.38);">⚠️ Atenção</span>`;
      }
      if (!winOk) {
        return `<span onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')" title="${escapeSkillsHtml(summary)} — Clique para ver detalhes" style="cursor:pointer; display:inline-block; padding:3px 8px; border-radius:999px; font-size:0.7rem; font-weight:700; background:rgba(56,189,248,0.14); color:#38bdf8; border:1px solid rgba(56,189,248,0.35);">🐧 Só Mac/Linux</span>`;
      }
      return `<span title="${escapeSkillsHtml(summary)}" style="display:inline-block; padding:3px 8px; border-radius:999px; font-size:0.7rem; font-weight:700; background:rgba(34,197,94,0.13); color:#4ade80; border:1px solid rgba(34,197,94,0.3);">✓ Verificada</span>`;
    }

    function renderInstalledSkillsTable() {
      const tbody = document.getElementById('skills-installed-tbody');
      if (!tbody) return;

      const q = (document.getElementById('skills-installed-search')?.value || '').trim().toLowerCase();
      const sf = skillsHubState.surfaceFilter || 'all';

      const filtered = (skillsHubState.skills || []).filter(s => {
        if (sf === 'workspace' && !(s.surfaces || []).some(x => x.target_key === 'workspace')) return false;
        if (sf === 'hermes_global' && !(s.surfaces || []).some(x => x.target_key === 'hermes_global')) return false;
        if (sf === 'personas' && !(s.surfaces || []).some(x => String(x.target_key).startsWith('persona:'))) return false;
        if (sf === 'antigravity' && !(s.surfaces || []).some(x => x.target_key === 'antigravity')) return false;
        if (sf === 'drift' && s.in_sync) return false;
        if (sf === 'paused' && s.enabled !== false) return false;
        if (sf === 'pinned' && !(s.usage && s.usage.pinned)) return false;

        if (!q) return true;
        const hay = [
          s.slug,
          s.name,
          s.description,
          s.category || '',
          ...(s.tags || []),
          ...(s.mcp_tools_detected || [])
        ].join(' ').toLowerCase();
        return hay.includes(q);
      });

      const sorted = sortInstalledSkills(filtered);
      if (sorted.length === 0) {
        tbody.innerHTML = `<tr><td colspan="8" style="text-align:center; padding:26px; color:#64748b;">Nenhuma habilidade encontrada com os filtros atuais.</td></tr>`;
        return;
      }

      tbody.innerHTML = sorted.map(s => {
        const isPaused = s.enabled === false;
        const isPinned = !!(s.usage && s.usage.pinned);
        const descLen = s.description_len || (s.description || '').length;
        const descWarn = (!isPaused && descLen > 60)
          ? ` <span title="Resumo acima de 60 caracteres aumenta o consumo fixo da IA. Clique em Editar -> Corrigir Automaticamente." style="color:#fbbf24; font-size:0.69rem; font-weight:600;">(${descLen} caracteres)</span>`
          : '';

        const catBadge = s.category
          ? `<span style="margin-left:6px; padding:1px 7px; border-radius:999px; font-size:0.68rem; background:rgba(139,92,246,0.14); color:#c4b5fd; border:1px solid rgba(139,92,246,0.28);">${escapeSkillsHtml(s.category)}</span>`
          : '';

        const pinIcon = isPinned ? '<span title="Fixada" style="margin-right:4px;">📌</span>' : '';

        let stateHtml = '';
        if (isPaused) {
          stateHtml = `<span title="Pausada temporariamente — não consome memória da IA" style="display:inline-block; white-space:nowrap; padding:2px 8px; border-radius:999px; font-size:0.7rem; background:rgba(148,163,184,0.14); color:#94a3b8; border:1px solid rgba(148,163,184,0.28);">⏸️ Pausada</span>`;
        } else if (s.in_sync) {
          stateHtml = `<span title="Mesmo conteúdo em todos os ambientes instalados" style="display:inline-block; white-space:nowrap; padding:2px 8px; border-radius:999px; font-size:0.7rem; background:rgba(34,197,94,0.12); color:#4ade80; border:1px solid rgba(34,197,94,0.28);">✓ Em dia</span>`;
        } else {
          stateHtml = `<span onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')" title="${s.unique_hashes_count} versões diferentes entre seus ambientes — clique para sincronizar" style="cursor:pointer; display:inline-block; white-space:nowrap; padding:2px 8px; border-radius:999px; font-size:0.7rem; background:rgba(245,158,11,0.15); color:#fbbf24; border:1px solid rgba(245,158,11,0.35);">⚡ Diferente</span>`;
        }

        const weightHtml = isPaused
          ? `<span style="color:#64748b; font-size:0.74rem;" title="Pausada: 0 tokens fixos no início das conversas">0 tk <span style="font-size:0.66rem;">(pausada)</span></span>`
          : `<span title="Consumo fixo na abertura: ${s.level1_tokens} tokens • Texto completo quando usada: ${s.level2_tokens} tokens"><strong style="color:#38bdf8;">${s.level1_tokens}</strong> <span style="color:#64748b;">/ ${s.level2_tokens}</span></span>`;

        const useCount = (s.usage && s.usage.use_count) || 0;
        const score = s.quality_score || 0;
        const scoreColor = score >= 85 ? '#4ade80' : (score >= 65 ? '#fbbf24' : '#f87171');

        const pauseBtnIcon = isPaused ? '▶️' : '⏸️';
        const pauseBtnTitle = isPaused
          ? 'Reativar esta skill nos ambientes'
          : 'Pausar temporariamente (economiza memória da IA sem apagar arquivos)';

        return `<tr class="${isPaused ? 'skills-row-paused' : ''}">
          <td>
            <div style="font-weight:700; color:#f1f5f9; font-size:0.84rem; cursor:pointer;" onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')">
              ${pinIcon}${escapeSkillsHtml(s.slug)}${catBadge}
            </div>
            <div style="color:#94a3b8; font-size:0.75rem; margin-top:3px; line-height:1.35;">
              ${escapeSkillsHtml(s.description || 'Sem resumo cadastrado.')}${descWarn}
            </div>
          </td>
          <td style="text-align:center;">${buildSurfaceMatrixBadges(s)}</td>
          <td style="text-align:center;">${stateHtml}</td>
          <td style="text-align:right; font-family:'JetBrains Mono',monospace; font-size:0.76rem;">${weightHtml}</td>
          <td style="text-align:right; font-family:'JetBrains Mono',monospace; font-size:0.76rem; color:${useCount > 0 ? '#e2e8f0' : '#64748b'};">${useCount}x</td>
          <td style="text-align:center; font-weight:800; color:${scoreColor}; font-size:0.84rem;">${score}</td>
          <td style="text-align:center;">${buildFriendlySecurityBadge(s)}</td>
          <td style="text-align:right; white-space:nowrap;">
            <div style="display:inline-flex; gap:4px;">
              <button class="btn-action" style="padding:4px 8px; font-size:0.74rem;" title="Abrir editor, ver diagnósticos e corrigir automaticamente" onclick="inspectSkill('${escapeSkillsHtml(s.slug)}')">✏️</button>
              <button class="btn-action" style="padding:4px 8px; font-size:0.74rem;" title="${pauseBtnTitle}" onclick="toggleSkillDisabled('${escapeSkillsHtml(s.slug)}', ${!isPaused})">${pauseBtnIcon}</button>
              <button class="btn-action" style="padding:4px 8px; font-size:0.74rem;" title="Evoluir ou adaptar no Estúdio de Criação" onclick="openInstalledInForge('${escapeSkillsHtml(s.slug)}')">✨</button>
              <button class="btn-action" style="padding:4px 8px; font-size:0.74rem; ${isPinned ? 'border-color:#38bdf8; color:#38bdf8;' : ''}" title="${isPinned ? 'Desafixar do topo' : 'Fixar skill favorita'}" onclick="toggleSkillPin('${escapeSkillsHtml(s.slug)}', ${!isPinned})">📌</button>
              <button class="btn-action" style="padding:4px 8px; font-size:0.74rem; color:#fca5a5;" title="Excluir skill (salva cópia de segurança no Histórico antes)" onclick="confirmAndDeleteSkill('${escapeSkillsHtml(s.slug)}')">🗑️</button>
            </div>
          </td>
        </tr>`;
      }).join('');
    }

    function buildAvailableTargetsList() {
      const avail = skillsHubState.availableSurfaces || [];
      if (avail.length > 0) {
        return avail.map(a => ({
          key: a.key,
          label: a.label
        }));
      }
      const fallback = [
        { key: 'workspace', label: '🦀 Souls Local' },
        { key: 'hermes_global', label: '🧠 Hermes Principal' }
      ];
      (skillsHubState.personas || []).forEach(p => {
        fallback.push({ key: 'persona:' + p, label: '🎭 Persona: ' + p });
      });
      return fallback;
    }

    function renderTargetCheckboxes(containerId, checkedKeys) {
      const box = document.getElementById(containerId);
      if (!box) return;
      const checkedSet = new Set(checkedKeys || []);
      const targets = buildAvailableTargetsList();

      box.innerHTML = targets.map(t => {
        const checked = checkedSet.has(t.key) ? 'checked' : '';
        return `<label style="display:inline-flex; align-items:center; gap:5px; padding:4px 9px; border-radius:6px; background:rgba(15,23,42,0.65); border:1px solid rgba(148,163,184,0.2); font-size:0.74rem; color:#e2e8f0; cursor:pointer;">
          <input type="checkbox" value="${escapeSkillsHtml(t.key)}" ${checked} />
          <span>${escapeSkillsHtml(t.label)}</span>
        </label>`;
      }).join('');
    }

    function getCheckedTargetKeys(containerId) {
      const box = document.getElementById(containerId);
      if (!box) return [];
      return Array.from(box.querySelectorAll('input[type="checkbox"]:checked')).map(el => el.value);
    }

    async function inspectSkill(slug, surfaceKey) {
      skillsHubState.inspectedSlug = slug;
      try {
        let url = '/api/v1/skills/detail?slug=' + encodeURIComponent(slug);
        if (surfaceKey) url += '&surface=' + encodeURIComponent(surfaceKey);
        const res = await fetch(url);
        const data = await res.json();
        if (data.status !== 'success' || !data.record) {
          showSkillsToast(data.message || 'Não foi possível abrir os detalhes desta habilidade.', 'error');
          return;
        }

        const rec = data.record;
        skillsHubState.inspectedRecord = rec;
        skillsHubState.inspectedHistory = data.history || [];

        const card = document.getElementById('skills-inspector-card');
        if (card) {
          card.style.display = 'block';
          card.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }

        document.getElementById('insp-skill-title').textContent = `✏️ Editando: ${rec.slug}`;
        const scoreBadge = document.getElementById('insp-skill-score-badge');
        if (scoreBadge) {
          scoreBadge.textContent = `Qualidade ${rec.quality_score}/100`;
        }

        const verdictBadge = document.getElementById('insp-skill-verdict-badge');
        if (verdictBadge) {
          verdictBadge.innerHTML = buildFriendlySecurityBadge(rec);
          verdictBadge.style.background = 'transparent';
          verdictBadge.style.border = 'none';
          verdictBadge.style.padding = '0';
        }

        const activeSurfObj = (rec.surfaces || []).find(s => s.target_key === data.active_surface) || (rec.surfaces || [])[0];
        document.getElementById('insp-skill-path').textContent = activeSurfObj ? activeSurfObj.skill_md_path : '';

        // Update Pause/Resume button in Inspector header
        const pauseBtn = document.getElementById('btn-insp-toggle-pause');
        if (pauseBtn) {
          if (rec.enabled === false) {
            pauseBtn.textContent = '▶️ Reativar Skill';
            pauseBtn.style.borderColor = 'rgba(34,197,94,0.4)';
            pauseBtn.style.color = '#4ade80';
          } else {
            pauseBtn.textContent = '⏸️ Pausar';
            pauseBtn.style.borderColor = '';
            pauseBtn.style.color = '';
          }
        }

        const driftBtn = document.getElementById('btn-insp-compare-drift');
        if (driftBtn) {
          driftBtn.style.display = rec.in_sync ? 'none' : 'inline-block';
        }

        // Hide autofix guarantee banner until user runs autofix
        const guaranteeEl = document.getElementById('insp-autofix-guarantee');
        if (guaranteeEl) guaranteeEl.style.display = 'none';

        // Surface switcher dropdown
        const surfSelect = document.getElementById('insp-surface-select');
        if (surfSelect) {
          surfSelect.innerHTML = (rec.surfaces || []).map(s => {
            const sel = s.target_key === data.active_surface ? 'selected' : '';
            const st = s.enabled === false ? ' (pausada)' : '';
            return `<option value="${escapeSkillsHtml(s.target_key)}" ${sel}>${escapeSkillsHtml(s.label)}${st}</option>`;
          }).join('');
        }

        // Populate editor & live description meter
        const editor = document.getElementById('insp-skill-editor');
        if (editor) {
          editor.value = data.content || '';
          updateLiveDescCounter('insp-skill-editor', 'insp-desc-counter');
        }

        // Render Security / Compatibility Human Explanation Callout (#insp-security-explainer)
        renderInspectorSecurityExplainer(rec, data.scan_report);

        // Render Linter issues + inline autofix visibility
        renderInspectorLintIssues(rec.lint_issues || []);

        // Render MCP Tools & Aux files
        const toolsBox = document.getElementById('insp-mcp-tools-list');
        if (toolsBox) {
          const tools = rec.mcp_tools_detected || [];
          toolsBox.innerHTML = tools.length > 0
            ? tools.map(t => `<span style="padding:2px 8px; border-radius:999px; font-size:0.72rem; font-family:'JetBrains Mono',monospace; background:rgba(56,189,248,0.12); color:#38bdf8; border:1px solid rgba(56,189,248,0.28);">${escapeSkillsHtml(t)}</span>`).join('')
            : '<span style="color:#64748b; font-size:0.76rem;">Nenhuma ferramenta externa específica detectada.</span>';
        }

        const auxBox = document.getElementById('insp-aux-files-list');
        if (auxBox) {
          const aux = data.auxiliary_files || [];
          const auxLabels = aux.map(a => {
            if (typeof a === 'string') return a;
            if (a && a.folder && a.name) return `${a.folder}/${a.name}`;
            return (a && (a.name || a.path)) || '';
          }).filter(Boolean);
          auxBox.textContent = auxLabels.length > 0
            ? `Arquivos de apoio (${auxLabels.length}): ${auxLabels.join(', ')}`
            : 'Sem scripts ou anexos extras nesta pasta (apenas SKILL.md).';
        }

        // Render sync target checkboxes pre-selecting surfaces where this skill already lives
        const activeTargets = (rec.surfaces || []).map(s => s.target_key);
        renderTargetCheckboxes('insp-sync-targets', activeTargets);

        // Render version history
        renderInspectorHistory(rec.slug, data.history || []);
      } catch (err) {
        console.error('Erro em inspectSkill:', err);
        showSkillsToast('Falha ao abrir detalhes da habilidade.', 'error');
      }
    }

    function switchInspectedSurface() {
      const sel = document.getElementById('insp-surface-select');
      if (!sel || !skillsHubState.inspectedSlug) return;
      inspectSkill(skillsHubState.inspectedSlug, sel.value);
    }

    function closeSkillInspector() {
      const card = document.getElementById('skills-inspector-card');
      if (card) card.style.display = 'none';
      skillsHubState.inspectedSlug = null;
      skillsHubState.inspectedRecord = null;
    }

    function renderInspectorSecurityExplainer(rec, scanReport) {
      const box = document.getElementById('insp-security-explainer');
      if (!box) return;

      const reasons = (rec.security_reasons && rec.security_reasons.length > 0)
        ? rec.security_reasons
        : ((scanReport && scanReport.findings) || []);

      const isSafe = String(rec.security_verdict || 'safe').toLowerCase() === 'safe' && rec.win11_compatible !== false;
      if (isSafe && reasons.length === 0) {
        box.style.display = 'none';
        box.innerHTML = '';
        return;
      }

      const isDanger = String(rec.security_verdict).toLowerCase() === 'dangerous';
      box.className = 'skills-why-callout ' + (isDanger ? 'danger' : (!rec.win11_compatible && rec.security_verdict === 'safe' ? 'info' : ''));
      box.style.display = 'block';

      const headerTitle = isDanger
        ? '🛑 Por que esta habilidade pede revisão cuidadosa?'
        : (!rec.win11_compatible && rec.security_verdict === 'safe'
            ? '🐧 Compatibilidade de Sistema Operacional (Mac / Linux)'
            : '💡 Por que esta habilidade recebeu o aviso "Atenção"?');

      const summaryStr = String(rec.security_human_summary || '').trim();
      const itemsHtml = reasons.slice(0, 5).map(f => {
        const loc = f.line > 0 ? ` (linha ${f.line})` : '';
        const recText = String(f.recommendation || f.category || '').trim();
        const sameAsSummary = recText && recText === summaryStr;
        const snip = f.snippet ? `<code style="display:block; margin-top:3px; padding:3px 7px; border-radius:5px; background:rgba(15,23,42,0.65); color:#cbd5e1; font-size:0.72rem; overflow-x:auto;">${escapeSkillsHtml(f.snippet)}</code>` : '';
        return `<li style="margin-bottom:6px;">
          ${sameAsSummary ? `<span style="color:#94a3b8; font-size:0.73rem;">Origem: ${escapeSkillsHtml(f.file || 'SKILL.md')}${loc}</span>` : `<strong>${escapeSkillsHtml(recText)}</strong> <span style="color:#94a3b8; font-size:0.72rem;">— ${escapeSkillsHtml(f.file || 'SKILL.md')}${loc}</span>`}
          ${snip}
        </li>`;
      }).join('');

      box.innerHTML = `
        <div style="font-weight:700; color:#f8fafc; margin-bottom:4px;">${headerTitle}</div>
        <div style="color:#e2e8f0; margin-bottom:${itemsHtml ? '8px' : '0'};">${escapeSkillsHtml(summaryStr)}</div>
        ${itemsHtml ? `<ul style="margin:0; padding-left:18px; color:#cbd5e1; font-size:0.76rem;">${itemsHtml}</ul>` : ''}
      `;
    }

    function renderInspectorLintIssues(issues) {
      const box = document.getElementById('insp-lint-list');
      const inlineBtn = document.getElementById('btn-insp-inline-autofix');
      if (!box) return;

      if (!issues || issues.length === 0) {
        if (inlineBtn) inlineBtn.style.display = 'none';
        box.innerHTML = `<div style="padding:8px 10px; border-radius:7px; background:rgba(34,197,94,0.1); border:1px solid rgba(34,197,94,0.28); color:#4ade80; font-size:0.77rem;">
          ✓ Tudo ótimo! Cabeçalho enxuto, estrutura clara e formatação limpa.
        </div>`;
        return;
      }

      const hasFixableHeader = issues.some(i =>
        ['DESC_BLOAT', 'DESC_MISSING', 'MISSING_FRONTMATTER', 'BOM_OR_LEADING_SPACE', 'CRLF_DETECTED', 'DESC_BUZZWORDS', 'NAME_MISMATCH'].includes(i.code)
      );
      if (inlineBtn) {
        inlineBtn.style.display = hasFixableHeader ? 'inline-block' : 'none';
      }

      box.innerHTML = issues.map(i => {
        const isErr = i.severity === 'error';
        const isWarn = i.severity === 'warning';
        const col = isErr ? '#f87171' : (isWarn ? '#fbbf24' : '#38bdf8');
        const bg = isErr ? 'rgba(239,68,68,0.1)' : (isWarn ? 'rgba(245,158,11,0.1)' : 'rgba(56,189,248,0.08)');
        const icon = isErr ? '🛑' : (isWarn ? '💡' : 'ℹ️');
        return `<div style="padding:7px 10px; border-radius:6px; background:${bg}; border-left:3px solid ${col}; font-size:0.76rem; color:#e2e8f0;">
          ${icon} ${escapeSkillsHtml(i.message)}
        </div>`;
      }).join('');
    }

    function renderInspectorHistory(slug, historyList) {
      const box = document.getElementById('insp-history-list');
      if (!box) return;
      if (!historyList || historyList.length === 0) {
        box.innerHTML = '<div style="color:#64748b;">Nenhuma versão anterior salva ainda.</div>';
        return;
      }
      box.innerHTML = historyList.slice(0, 8).map(h => {
        const dt = h.timestamp_epoch ? new Date(h.timestamp_epoch * 1000).toLocaleString('pt-BR') : h.snapshot_id;
        return `<div style="display:flex; justify-content:space-between; align-items:center; padding:5px 8px; border-radius:6px; background:rgba(15,23,42,0.55); border:1px solid rgba(148,163,184,0.12);">
          <div>
            <span style="color:#e2e8f0; font-weight:600;">${escapeSkillsHtml(dt)}</span>
            <span style="color:#94a3b8; margin-left:6px;">(${escapeSkillsHtml(h.reason || 'backup')})</span>
          </div>
          <button class="btn-action" style="padding:2px 8px; font-size:0.7rem;" onclick="rollbackSkillToSnapshot('${escapeSkillsHtml(slug)}', '${escapeSkillsHtml(h.snapshot_id)}')">⏪ Restaurar</button>
        </div>`;
      }).join('');
    }

    async function autofixInspectedSkill() {
      const slug = skillsHubState.inspectedSlug;
      const editor = document.getElementById('insp-skill-editor');
      const btn = document.getElementById('btn-insp-autofix');
      if (!slug || !editor) return;

      const origLabel = btn ? btn.textContent : '';
      if (btn) {
        btn.disabled = true;
        btn.textContent = '⏳ Ajustando com cuidado...';
      }

      try {
        const res = await fetch('/api/v1/skills/autofix', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            content: editor.value,
            use_local_model: true
          })
        });
        const data = await res.json();
        if (data.status !== 'success' || !data.content) {
          showSkillsToast(data.message || 'Não foi possível aplicar a correção automática.', 'error');
          return;
        }

        editor.value = data.content;
        updateLiveDescCounter('insp-skill-editor', 'insp-desc-counter');

        const scoreBadge = document.getElementById('insp-skill-score-badge');
        if (scoreBadge && data.after_score != null) {
          scoreBadge.textContent = `Qualidade ${data.before_score} → ${data.after_score}/100`;
        }

        renderInspectorLintIssues(data.lint_issues || []);

        const guaranteeEl = document.getElementById('insp-autofix-guarantee');
        if (guaranteeEl) {
          const fixesText = (data.applied_fixes || []).join(' • ');
          guaranteeEl.textContent = `✓ Correção pronta (${fixesText}). Todas as instruções originais da skill foram 100% preservadas — clique em Salvar abaixo para aplicar.`;
          guaranteeEl.style.display = 'block';
        }

        showSkillsToast(`Cabeçalho otimizado (${data.before_score} → ${data.after_score} pts) preservando 100% do conteúdo! Clique em Salvar.`, 'success');
      } catch (err) {
        console.error('Erro em autofixInspectedSkill:', err);
        showSkillsToast('Erro ao executar a correção automática.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.textContent = origLabel || '✨ Corrigir Automaticamente';
        }
      }
    }

    async function quickAutofixAndOpen(slug) {
      switchSkillsSubtab('installed');
      await inspectSkill(slug);
      await autofixInspectedSkill();
    }

    async function saveAndSyncInspectedSkill() {
      const rec = skillsHubState.inspectedRecord;
      const editor = document.getElementById('insp-skill-editor');
      if (!rec || !editor) return;

      const targets = getCheckedTargetKeys('insp-sync-targets');
      if (targets.length === 0) {
        showSkillsToast('Selecione pelo menos um ambiente onde deseja salvar esta habilidade.', 'warning');
        return;
      }

      try {
        const res = await fetch('/api/v1/skills/sync', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug: rec.slug,
            content: editor.value,
            targets,
            category: rec.category || null,
            reason: 'dashboard_edit'
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Falha ao salvar alterações.', 'error');
          return;
        }
        showSkillsToast(`"${rec.slug}" salva e sincronizada em ${targets.length} ambiente(s)!`, 'success');
        await fetchInstalledSkills(false);
        await inspectSkill(rec.slug);
      } catch (err) {
        console.error('Erro em saveAndSyncInspectedSkill:', err);
        showSkillsToast('Erro ao salvar alterações.', 'error');
      }
    }

    async function toggleSkillPin(slug, pinned) {
      try {
        const res = await fetch('/api/v1/skills/state', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, pinned })
        });
        const data = await res.json();
        if (data.status === 'success') {
          showSkillsToast(pinned ? `"${slug}" fixada nas favoritas.` : `"${slug}" desafixada.`, 'success');
          fetchInstalledSkills(false);
        }
      } catch (err) {
        console.error('Erro em toggleSkillPin:', err);
      }
    }

    async function toggleSkillDisabled(slug, disabled) {
      try {
        const res = await fetch('/api/v1/skills/state', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, disabled })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Não foi possível alterar o estado desta habilidade.', 'error');
          return;
        }
        showSkillsToast(
          disabled
            ? `"${slug}" foi pausada. Ela não consumirá memória da IA até você reativá-la.`
            : `"${slug}" foi reativada e já está disponível novamente!`,
          'success'
        );
        await fetchInstalledSkills(false);
        if (skillsHubState.inspectedSlug === slug) {
          await inspectSkill(slug);
        }
      } catch (err) {
        console.error('Erro em toggleSkillDisabled:', err);
        showSkillsToast('Erro ao pausar/reativar habilidade.', 'error');
      }
    }

    function togglePauseInspectedSkill() {
      const rec = skillsHubState.inspectedRecord;
      if (!rec) return;
      const currentlyPaused = rec.enabled === false;
      toggleSkillDisabled(rec.slug, !currentlyPaused);
    }

    async function confirmAndDeleteSkill(slug) {
      const ok = window.confirm(
        `Deseja realmente excluir a habilidade "${slug}" dos seus ambientes?\n\n` +
        `Fique tranquilo: uma cópia automática de segurança será guardada no Histórico caso precise recuperá-la depois.`
      );
      if (!ok) return;

      try {
        const res = await fetch('/api/v1/skills/delete', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, targets: [] })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Não foi possível excluir a habilidade.', 'error');
          return;
        }
        if (skillsHubState.inspectedSlug === slug) {
          closeSkillInspector();
        }
        showSkillsToast(`"${slug}" removida com backup salvo no Histórico.`, 'success');
        await fetchInstalledSkills(false);
      } catch (err) {
        console.error('Erro em confirmAndDeleteSkill:', err);
        showSkillsToast('Erro ao excluir habilidade.', 'error');
      }
    }

    function deleteInspectedSkill() {
      if (!skillsHubState.inspectedSlug) return;
      confirmAndDeleteSkill(skillsHubState.inspectedSlug);
    }

    async function rollbackSkillToSnapshot(slug, snapshotId) {
      try {
        const res = await fetch('/api/v1/skills/rollback', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, snapshot_id: snapshotId, targets: [] })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Falha ao restaurar versão anterior.', 'error');
          return;
        }
        showSkillsToast(`Versão restaurada com sucesso para "${slug}"!`, 'success');
        await fetchInstalledSkills(false);
        if (skillsHubState.inspectedSlug === slug) {
          await inspectSkill(slug);
        }
        if (skillsHubState.currentSubtab === 'health') {
          loadHealthSkillTimeline();
        }
      } catch (err) {
        console.error('Erro em rollbackSkillToSnapshot:', err);
      }
    }

    function updateLiveDescCounter(textareaId, badgeId) {
      const ta = document.getElementById(textareaId);
      const badge = document.getElementById(badgeId);
      if (!ta || !badge) return;
      const text = ta.value || '';
      let desc = '';
      if (text.startsWith('---')) {
        const endIdx = text.indexOf('\n---', 3);
        if (endIdx !== -1) {
          const fm = text.slice(3, endIdx);
          const lines = fm.split('\n');
          for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();
            if (line.startsWith('description:')) {
              desc = line.slice('description:'.length).trim().replace(/^["']|["']$/g, '');
              if ((desc === '|' || desc === '>') && i + 1 < lines.length) {
                desc = lines[i + 1].trim();
              }
              break;
            }
          }
        }
      }
      const n = Array.from(desc).length;
      badge.textContent = `Resumo: ${n}/60 caracteres`;
      if (n > 0 && n <= 60) {
        badge.style.background = 'rgba(34,197,94,0.14)';
        badge.style.color = '#4ade80';
      } else if (n === 0) {
        badge.style.background = 'rgba(239,68,68,0.15)';
        badge.style.color = '#f87171';
      } else {
        badge.style.background = 'rgba(245,158,11,0.16)';
        badge.style.color = '#fbbf24';
      }
    }

    // =========================================================================
    // 2. DESCOBRIR NOVAS — VITRINE PERSONALIZADA, FILTROS & AREA DE REVISAO
    // =========================================================================

    function toggleRecommendationsShelf() {
      skillsHubState.recommendationsHidden = !skillsHubState.recommendationsHidden;
      const shelf = document.getElementById('skills-recommendations-shelf');
      if (shelf) {
        shelf.style.display = skillsHubState.recommendationsHidden ? 'none' : 'block';
      }
    }

    function applyQuickExplorerTopic(query, source, category, btnEl) {
      const qEl = document.getElementById('skills-marketplace-query');
      const srcEl = document.getElementById('skills-marketplace-source');
      const catEl = document.getElementById('skills-marketplace-category');
      if (qEl) qEl.value = query || '';
      if (srcEl && source) srcEl.value = source;
      if (catEl && category) catEl.value = category;

      const bar = document.getElementById('skills-quick-topics');
      if (bar && btnEl) {
        bar.querySelectorAll('.skills-topic-pill').forEach(b => b.classList.remove('active'));
        btnEl.classList.add('active');
      }
      runMarketplaceSearch(0);
    }

    function debouncedMarketplaceSearch() {
      if (skillsHubState.mpSearchTimer) clearTimeout(skillsHubState.mpSearchTimer);
      skillsHubState.mpSearchTimer = setTimeout(() => {
        runMarketplaceSearch(0);
      }, 220);
    }

    function pageMarketplace(dir) {
      const next = Math.max(0, skillsHubState.marketplaceOffset + dir * skillsHubState.marketplaceLimit);
      if (next >= skillsHubState.marketplaceTotal && dir > 0) return;
      runMarketplaceSearch(next);
    }

    async function runMarketplaceSearch(offset) {
      skillsHubState.marketplaceOffset = offset || 0;
      const q = (document.getElementById('skills-marketplace-query')?.value || '').trim();
      const category = document.getElementById('skills-marketplace-category')?.value || 'all';
      const source = document.getElementById('skills-marketplace-source')?.value || 'all';
      const trust = document.getElementById('skills-marketplace-trust')?.value || 'all';
      const liveGh = !!document.getElementById('skills-marketplace-live-gh')?.checked;

      const params = new URLSearchParams({
        q,
        category,
        source,
        trust,
        live_github: String(liveGh),
        limit: String(skillsHubState.marketplaceLimit),
        offset: String(skillsHubState.marketplaceOffset)
      });

      try {
        const res = await fetch('/api/v1/skills/marketplace/search?' + params.toString());
        const data = await res.json();
        if (data.status !== 'success') return;

        skillsHubState.marketplaceTotal = data.total_matches || 0;
        if (Array.isArray(data.installed_slugs)) {
          skillsHubState.installedSlugsSet = new Set(data.installed_slugs);
        }
        if (Array.isArray(data.recommendations) && data.recommendations.length > 0) {
          skillsHubState.recommendations = data.recommendations;
          renderMarketplaceRecommendations(data.recommendations);
        }

        const pill = document.getElementById('explorer-latency-pill');
        if (pill) {
          const lat = Number(data.latency_ms || 0).toFixed(1);
          const idxTotal = Number(data.indexed_total || 0).toLocaleString('pt-BR');
          pill.textContent = `⚡ ${lat} ms • ${idxTotal} disponíveis`;
        }

        renderMarketplaceResultsTable(data.results || []);
      } catch (err) {
        console.error('Erro em runMarketplaceSearch:', err);
      }
    }

    function renderMarketplaceRecommendations(recs) {
      const shelf = document.getElementById('skills-recommendations-shelf');
      const grid = document.getElementById('skills-recommendations-grid');
      if (!shelf || !grid) return;

      if (!recs || recs.length === 0 || skillsHubState.recommendationsHidden) {
        shelf.style.display = 'none';
        return;
      }

      shelf.style.display = 'block';
      grid.innerHTML = recs.map(r => {
        const item = r.item || {};
        const slugName = item.name || item.identifier || 'skill';
        const repoText = item.repo || formatSourceLabel(item.source);
        return `<div class="skills-rec-card">
          <div>
            <span class="skills-rec-reason">${escapeSkillsHtml(r.reason_label || '✨ Recomendada')}</span>
            <div style="font-weight:700; color:#f8fafc; font-size:0.84rem;">${escapeSkillsHtml(slugName)}</div>
            <div class="skills-mp-desc-clamp" style="margin-top:4px; font-size:0.74rem;">${escapeSkillsHtml(item.description || '')}</div>
          </div>
          <div style="display:flex; justify-content:space-between; align-items:center; gap:8px; margin-top:4px; padding-top:6px; border-top:1px solid rgba(148,163,184,0.12);">
            <span style="font-size:0.69rem; color:#94a3b8; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" title="${escapeSkillsHtml(repoText)}">${escapeSkillsHtml(repoText)}</span>
            <button class="btn-action primary" style="padding:3px 9px; font-size:0.72rem; flex-shrink:0;" onclick="fetchCatalogItemToQuarantine('${escapeSkillsHtml(item.identifier)}', '${escapeSkillsHtml(item.repo || '')}', '${escapeSkillsHtml(item.path || '')}')">🛡️ Analisar</button>
          </div>
        </div>`;
      }).join('');
    }

    function renderMarketplaceResultsTable(list) {
      const tbody = document.getElementById('skills-marketplace-tbody');
      if (!tbody) return;

      if (!list || list.length === 0) {
        tbody.innerHTML = `<tr><td colspan="6" style="text-align:center; padding:26px; color:#64748b;">Nenhuma habilidade encontrada para este filtro. Tente outro termo ou tema acima.</td></tr>`;
      } else {
        tbody.innerHTML = list.map(item => {
          const cleanName = String(item.name || '').toLowerCase();
          const alreadyInstalled = skillsHubState.installedSlugsSet.has(cleanName);
          const trustObj = formatTrustLabel(item.trust_level);
          const catLabel = formatCategoryLabel(item.category);
          const srcLabel = formatSourceLabel(item.source);
          const repoDisplay = item.repo ? item.repo : srcLabel;

          const actionHtml = alreadyInstalled
            ? `<span style="display:inline-block; padding:4px 9px; border-radius:999px; font-size:0.71rem; font-weight:600; background:rgba(34,197,94,0.12); color:#4ade80; border:1px solid rgba(34,197,94,0.28);">✓ Já instalada</span>`
            : `<button class="btn-action primary" style="padding:4px 10px; font-size:0.73rem;" onclick="fetchCatalogItemToQuarantine('${escapeSkillsHtml(item.identifier)}', '${escapeSkillsHtml(item.repo || '')}', '${escapeSkillsHtml(item.path || '')}')">🛡️ Analisar</button>`;

          return `<tr>
            <td style="overflow:hidden; text-overflow:ellipsis;">
              <div style="font-weight:700; color:#f1f5f9; font-size:0.83rem; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;" title="${escapeSkillsHtml(item.identifier)}">${escapeSkillsHtml(item.name || item.identifier)}</div>
              <div style="font-size:0.69rem; color:#94a3b8; margin-top:2px;">${escapeSkillsHtml(catLabel)}</div>
            </td>
            <td>
              <div class="skills-mp-desc-clamp" title="${escapeSkillsHtml(item.description || '')}">
                ${escapeSkillsHtml(item.description || 'Sem descrição informada pelo autor.')}
              </div>
            </td>
            <td style="text-align:center; font-size:0.74rem; color:#38bdf8; white-space:nowrap;">${escapeSkillsHtml(srcLabel)}</td>
            <td style="text-align:center;">
              <span style="padding:2px 8px; border-radius:999px; font-size:0.69rem; font-weight:600; color:${trustObj.color}; background:${trustObj.bg}; border:1px solid ${trustObj.border};">${escapeSkillsHtml(trustObj.label)}</span>
            </td>
            <td class="skills-mp-repo-cell" title="${escapeSkillsHtml(repoDisplay)}">${escapeSkillsHtml(repoDisplay)}</td>
            <td style="text-align:right; white-space:nowrap;">${actionHtml}</td>
          </tr>`;
        }).join('');
      }

      const startN = list.length > 0 ? skillsHubState.marketplaceOffset + 1 : 0;
      const endN = skillsHubState.marketplaceOffset + list.length;
      const totalStr = (skillsHubState.marketplaceTotal || 0).toLocaleString('pt-BR');
      const infoEl = document.getElementById('skills-marketplace-page-info');
      if (infoEl) {
        infoEl.textContent = list.length > 0
          ? `Mostrando ${startN}–${endN} de ${totalStr} habilidades encontradas`
          : '0 resultados encontrados';
      }

      const prevBtn = document.getElementById('btn-mp-prev');
      const nextBtn = document.getElementById('btn-mp-next');
      if (prevBtn) prevBtn.disabled = skillsHubState.marketplaceOffset <= 0;
      if (nextBtn) nextBtn.disabled = endN >= skillsHubState.marketplaceTotal;
    }

    async function fetchCatalogItemToQuarantine(identifier, repo, path) {
      showSkillsToast(`Baixando "${identifier}" para a Área de Revisão...`, 'info');
      try {
        const res = await fetch('/api/v1/skills/quarantine/fetch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            identifier,
            repo: repo || null,
            path: path || null
          })
        });
        const data = await res.json();
        if (data.status !== 'success' || !data.item) {
          showSkillsToast(data.message || 'Não foi possível baixar esta habilidade da origem.', 'error');
          return;
        }
        showSkillsToast(`"${data.item.slug}" pronta para sua revisão abaixo!`, 'success');
        await fetchQuarantineList(data.item.slug);
        await fetchInstalledSkills(false);
        const wb = document.getElementById('skills-quarantine-workbench');
        if (wb) wb.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      } catch (err) {
        console.error('Erro em fetchCatalogItemToQuarantine:', err);
        showSkillsToast('Erro de rede ao baixar habilidade para revisão.', 'error');
      }
    }

    async function fetchDirectUrlToQuarantine() {
      const inp = document.getElementById('skills-direct-url-input');
      const url = (inp?.value || '').trim();
      if (!url) {
        showSkillsToast('Cole o link de um repositório ou arquivo SKILL.md do GitHub primeiro.', 'warning');
        return;
      }
      showSkillsToast('Baixando link para a Área de Revisão...', 'info');
      try {
        const res = await fetch('/api/v1/skills/quarantine/fetch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ url })
        });
        const data = await res.json();
        if (data.status !== 'success' || !data.item) {
          showSkillsToast(data.message || 'Não foi possível ler um SKILL.md neste endereço.', 'error');
          return;
        }
        if (inp) inp.value = '';
        showSkillsToast(`"${data.item.slug}" baixada para revisão com sucesso!`, 'success');
        await fetchQuarantineList(data.item.slug);
        await fetchInstalledSkills(false);
      } catch (err) {
        console.error('Erro em fetchDirectUrlToQuarantine:', err);
        showSkillsToast('Erro ao buscar link externo.', 'error');
      }
    }

    async function fetchQuarantineList(selectSlug) {
      try {
        const res = await fetch('/api/v1/skills/quarantine');
        const data = await res.json();
        if (data.status !== 'success') return;
        const items = data.items || [];
        skillsHubState.quarantineItems = items;

        const emptyEl = document.getElementById('skills-quarantine-empty');
        const wbEl = document.getElementById('skills-quarantine-workbench');
        if (items.length === 0) {
          if (emptyEl) emptyEl.style.display = 'block';
          if (wbEl) wbEl.style.display = 'none';
          return;
        }
        if (emptyEl) emptyEl.style.display = 'none';
        if (wbEl) wbEl.style.display = 'block';

        const chosenSlug = selectSlug || skillsHubState.activeQuarantineSlug || items[0].slug;
        selectQuarantineItem(chosenSlug);
      } catch (err) {
        console.error('Erro em fetchQuarantineList:', err);
      }
    }

    function selectQuarantineItem(slug) {
      const items = skillsHubState.quarantineItems || [];
      const item = items.find(i => i.slug === slug) || items[0];
      if (!item) return;
      skillsHubState.activeQuarantineSlug = item.slug;

      const pillsBox = document.getElementById('skills-quarantine-pills');
      if (pillsBox) {
        pillsBox.innerHTML = items.map(i => {
          const active = i.slug === item.slug ? 'active' : '';
          const vIcon = i.scan_report?.verdict === 'dangerous' ? '🛑' : (i.scan_report?.verdict === 'caution' ? '⚠️' : '✓');
          return `<button class="subtab-btn ${active}" style="padding:5px 11px; font-size:0.75rem;" onclick="selectQuarantineItem('${escapeSkillsHtml(i.slug)}')">${vIcon} ${escapeSkillsHtml(i.slug)}</button>`;
        }).join('');
      }

      const prevSlugEl = document.getElementById('q-preview-slug');
      const prevContentEl = document.getElementById('q-preview-content');
      if (prevSlugEl) prevSlugEl.textContent = `${item.slug}/SKILL.md (${item.level1_tokens} tokens fixos • Qualidade ${item.quality_score}/100)`;
      if (prevContentEl) prevContentEl.textContent = item.content || '';

      const verdict = item.scan_report?.verdict || 'safe';
      const winOk = item.scan_report?.win11_compatible !== false;
      const summaryEl = document.getElementById('q-scan-summary');
      if (summaryEl) {
        if (verdict === 'dangerous') {
          summaryEl.style.background = 'rgba(239,68,68,0.14)';
          summaryEl.style.border = '1px solid rgba(239,68,68,0.4)';
          summaryEl.style.color = '#fca5a5';
          summaryEl.innerHTML = '<strong>🛑 Atenção Redobrada:</strong> Encontramos comandos potencialmente sensíveis nesta skill. Recomendamos adaptá-la no Estúdio antes de instalar.';
        } else if (verdict === 'caution' || !winOk) {
          summaryEl.style.background = 'rgba(245,158,11,0.13)';
          summaryEl.style.border = '1px solid rgba(245,158,11,0.38)';
          summaryEl.style.color = '#fde68a';
          summaryEl.innerHTML = !winOk
            ? '<strong>🐧 Aviso de Compatibilidade:</strong> Esta skill menciona comandos de Mac/Linux. Você pode clicar em "Adaptar no Estúdio" para convertê-la para Windows 11!'
            : '<strong>⚠️ Revisão Leve:</strong> A skill usa comandos de terminal ou variáveis de ambiente. Confira os itens abaixo antes de aprovar.';
        } else {
          summaryEl.style.background = 'rgba(34,197,94,0.12)';
          summaryEl.style.border = '1px solid rgba(34,197,94,0.3)';
          summaryEl.style.color = '#86efac';
          summaryEl.innerHTML = '<strong>✓ Tudo Limpo:</strong> Nenhum comportamento suspeito encontrado e totalmente compatível com Windows 11.';
        }
      }

      const findingsBox = document.getElementById('q-scan-findings');
      if (findingsBox) {
        const findings = item.scan_report?.findings || [];
        if (findings.length === 0) {
          findingsBox.innerHTML = '<div style="font-size:0.76rem; color:#94a3b8;">Nenhum alerta de segurança nos arquivos analisados.</div>';
        } else {
          findingsBox.innerHTML = findings.map(f => `<div style="padding:7px 9px; border-radius:6px; background:rgba(15,23,42,0.6); border-left:3px solid #fbbf24; font-size:0.75rem; color:#e2e8f0;">
            <div><strong>${escapeSkillsHtml(f.recommendation)}</strong> <span style="color:#94a3b8;">(linha ${f.line})</span></div>
            <code style="display:block; margin-top:3px; color:#cbd5e1; font-size:0.71rem;">${escapeSkillsHtml(f.snippet)}</code>
          </div>`).join('');
        }
      }

      renderTargetCheckboxes('q-install-targets', ['workspace', 'hermes_global']);
    }

    function forkQuarantineItemToForge() {
      const item = (skillsHubState.quarantineItems || []).find(i => i.slug === skillsHubState.activeQuarantineSlug);
      if (!item) return;
      switchSkillsSubtab('forge');
      const modeEl = document.getElementById('forge-mode');
      if (modeEl) {
        modeEl.value = 'fork_refine';
        onForgeModeChange();
      }
      document.getElementById('forge-slug').value = item.slug;
      document.getElementById('forge-base-content').value = item.content || '';
      document.getElementById('forge-goal-prompt').value =
        'Adapte esta habilidade para funcionar perfeitamente no Windows 11 (PowerShell), mantendo resumo até 60 caracteres e preservando o objetivo principal.';
    }

    async function approveAndInstallQuarantineItem() {
      const item = (skillsHubState.quarantineItems || []).find(i => i.slug === skillsHubState.activeQuarantineSlug);
      if (!item) return;

      const targets = getCheckedTargetKeys('q-install-targets');
      if (targets.length === 0) {
        showSkillsToast('Marque ao menos um ambiente onde deseja instalar esta habilidade.', 'warning');
        return;
      }

      try {
        const res = await fetch('/api/v1/skills/sync', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug: item.slug,
            content: item.content,
            targets,
            category: 'imported',
            reason: 'quarantine_approved'
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Erro ao instalar habilidade.', 'error');
          return;
        }
        showSkillsToast(`"${item.slug}" instalada com sucesso em ${targets.length} ambiente(s)!`, 'success');
        await fetchInstalledSkills(false);
        runMarketplaceSearch(skillsHubState.marketplaceOffset);
      } catch (err) {
        console.error('Erro em approveAndInstallQuarantineItem:', err);
        showSkillsToast('Erro ao concluir instalação.', 'error');
      }
    }

    // =========================================================================
    // 3. CRIAR & ADAPTAR — ESTUDIO ASSISTIDO DE SKILLS
    // =========================================================================

    function openForgeFromScratch() {
      switchSkillsSubtab('forge');
      const modeEl = document.getElementById('forge-mode');
      if (modeEl) {
        modeEl.value = 'from_scratch';
        onForgeModeChange();
      }
      document.getElementById('forge-slug').value = '';
      document.getElementById('forge-goal-prompt').focus();
    }

    async function openInstalledInForge(slug) {
      switchSkillsSubtab('forge');
      const modeEl = document.getElementById('forge-mode');
      if (modeEl) {
        modeEl.value = 'evolve';
        onForgeModeChange();
      }
      const picker = document.getElementById('forge-existing-picker');
      if (picker) {
        populateForgeSkillPicker();
        picker.value = slug;
      }
      await loadExistingSkillIntoForge(slug);
    }

    function sendInspectedSkillToForge() {
      if (!skillsHubState.inspectedSlug) return;
      openInstalledInForge(skillsHubState.inspectedSlug);
    }

    function onForgeModeChange() {
      const mode = document.getElementById('forge-mode')?.value || 'from_scratch';
      const isCreate = (mode === 'from_scratch' || mode === 'create');
      const baseBox = document.getElementById('forge-base-container');
      if (baseBox) {
        baseBox.style.display = isCreate ? 'none' : 'block';
      }
      if (!isCreate) {
        populateForgeSkillPicker();
      }
    }

    // HTML handler aliases for full compatibility
    function handleForgeModeChange() { onForgeModeChange(); }
    function handleForgeEngineChange() { onForgeEngineChange(); }
    function loadInstalledIntoForgeBase(val) { return loadExistingSkillIntoForge(val); }
    function reloadInspectedSurface(val) {
      if (skillsHubState.inspectedSlug) return inspectSkill(skillsHubState.inspectedSlug, val);
    }
    function compareInspectedSkillDrift() { return openDriftDiffFromInspector(); }
    function runSkillsForge() { return runSkillForge(); }
    function stepMarketplacePage(dir) { return pageMarketplace(dir); }
    function toggleForgeRightView(view) { return switchForgeOutputView(view); }
    function renderForgeMcpToolsSelector() { return filterForgeMcpTools(); }

    function onForgeEngineChange() {
      const eng = document.getElementById('forge-engine')?.value || 'deterministic_scaffold';
      const row = document.getElementById('forge-local-model-row');
      if (row) {
        row.style.display = eng === 'local_silicon' ? 'block' : 'none';
      }
    }

    function populateForgeSkillPicker() {
      const picker = document.getElementById('forge-existing-picker');
      if (!picker) return;
      const curVal = picker.value;
      picker.innerHTML = `<option value="">-- Escolha uma skill instalada para usar como base --</option>` +
        (skillsHubState.skills || []).map(s => `<option value="${escapeSkillsHtml(s.slug)}">${escapeSkillsHtml(s.slug)}</option>`).join('');
      if (curVal) picker.value = curVal;
    }

    async function loadExistingSkillIntoForge(explicitSlug) {
      const slug = explicitSlug || document.getElementById('forge-existing-picker')?.value;
      if (!slug) return;
      try {
        const res = await fetch('/api/v1/skills/detail?slug=' + encodeURIComponent(slug));
        const data = await res.json();
        if (data.status === 'success' && data.record) {
          document.getElementById('forge-slug').value = data.record.slug;
          document.getElementById('forge-category').value = data.record.category || '';
          document.getElementById('forge-base-content').value = data.content || '';
          skillsHubState.forgeSelectedTools = new Set(data.record.mcp_tools_detected || []);
          filterForgeMcpTools();
        }
      } catch (err) {
        console.error('Erro em loadExistingSkillIntoForge:', err);
      }
    }

    async function loadForgeMcpCatalog() {
      if (skillsHubState.mcpTools.length > 0) {
        filterForgeMcpTools();
        return;
      }
      try {
        const res = await fetch('/api/v1/tools/catalog');
        const data = await res.json();
        if (data.status === 'success' && Array.isArray(data.tools)) {
          skillsHubState.mcpTools = data.tools;
          filterForgeMcpTools();
        }
      } catch (err) {
        console.error('Erro em loadForgeMcpCatalog:', err);
      }
    }

    function filterForgeMcpTools() {
      const box = document.getElementById('forge-mcp-tools-selector');
      if (!box) return;
      const q = (document.getElementById('forge-tools-filter')?.value || '').trim().toLowerCase();
      const tools = (skillsHubState.mcpTools || []).filter(t =>
        !q || String(t.name || '').toLowerCase().includes(q) || String(t.description || '').toLowerCase().includes(q)
      );

      if (tools.length === 0) {
        box.innerHTML = '<span style="color:#64748b; font-size:0.74rem;">Nenhuma ferramenta correspondente.</span>';
        return;
      }

      box.innerHTML = tools.map(t => {
        const active = skillsHubState.forgeSelectedTools.has(t.name);
        const bg = active ? 'rgba(56,189,248,0.2)' : 'rgba(15,23,42,0.6)';
        const border = active ? 'rgba(56,189,248,0.55)' : 'rgba(148,163,184,0.18)';
        const col = active ? '#38bdf8' : '#cbd5e1';
        return `<button type="button" onclick="toggleForgeToolSelection('${escapeSkillsHtml(t.name)}')" title="${escapeSkillsHtml(t.description || '')}" style="padding:3px 8px; border-radius:999px; font-size:0.71rem; font-family:'JetBrains Mono',monospace; background:${bg}; border:1px solid ${border}; color:${col}; cursor:pointer;">${active ? '✓ ' : '+ '}${escapeSkillsHtml(t.name)}</button>`;
      }).join('');
    }

    function toggleForgeToolSelection(toolName) {
      if (skillsHubState.forgeSelectedTools.has(toolName)) {
        skillsHubState.forgeSelectedTools.delete(toolName);
      } else {
        skillsHubState.forgeSelectedTools.add(toolName);
      }
      filterForgeMcpTools();
    }

    async function loadForgeLocalModels() {
      const sel = document.getElementById('forge-local-model-select');
      if (!sel || sel.options.length > 1) return;
      try {
        const res = await fetch('/api/v1/models/local');
        const data = await res.json();
        const models = data.models || [];
        models.forEach(m => {
          const name = m.filename || m.name || m.id;
          if (!name) return;
          const opt = document.createElement('option');
          opt.value = name;
          opt.textContent = name;
          sel.appendChild(opt);
        });
      } catch (_) {
        // Keep automatic default option cleanly
      }
    }

    function switchForgeOutputView(view) {
      const codePane = document.getElementById('forge-code-pane');
      const diffPane = document.getElementById('forge-diff-pane');
      const btnCode = document.getElementById('btn-forge-view-code');
      const btnDiff = document.getElementById('btn-forge-view-diff');
      if (codePane) codePane.style.display = view === 'code' ? 'block' : 'none';
      if (diffPane) diffPane.style.display = view === 'diff' ? 'block' : 'none';
      if (btnCode) btnCode.classList.toggle('active', view === 'code');
      if (btnDiff) btnDiff.classList.toggle('active', view === 'diff');
    }

    async function runSkillForge() {
      const goal = (document.getElementById('forge-goal-prompt')?.value || '').trim();
      if (!goal) {
        showSkillsToast('Conte primeiro o que você deseja que a habilidade ensine à IA.', 'warning');
        return;
      }

      const mode = document.getElementById('forge-mode')?.value || 'create';
      const engine = document.getElementById('forge-engine')?.value || 'deterministic_scaffold';
      const slug = (document.getElementById('forge-slug')?.value || '').trim();
      const category = (document.getElementById('forge-category')?.value || '').trim();
      const baseContent = (document.getElementById('forge-base-content')?.value || '').trim();
      const localModel = (document.getElementById('forge-local-model-select')?.value || '').trim();

      const btn = document.getElementById('btn-forge-run');
      const origTxt = btn ? btn.textContent : '';
      if (btn) {
        btn.disabled = true;
        btn.textContent = '⏳ Criando rascunho inteligente...';
      }

      try {
        const res = await fetch('/api/v1/skills/forge/generate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            mode,
            engine,
            slug: slug || null,
            category: category || null,
            goal_prompt: goal,
            base_content: (mode === 'from_scratch' || mode === 'create') ? null : (baseContent || null),
            selected_mcp_tools: Array.from(skillsHubState.forgeSelectedTools),
            local_model: localModel || null
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Falha ao gerar habilidade no Estúdio.', 'error');
          return;
        }

        skillsHubState.forgeLastResult = data;
        if (data.slug && !slug) {
          document.getElementById('forge-slug').value = data.slug;
        }

        const outEditor = document.getElementById('forge-output-editor');
        if (outEditor) {
          outEditor.value = data.content || '';
          updateLiveDescCounter('forge-output-editor', 'forge-desc-meter');
        }

        document.getElementById('forge-out-score').textContent = `Qualidade: ${data.quality_score}/100`;
        document.getElementById('forge-out-tokens').textContent = `Peso: ${data.level1_tokens} tk fixos`;
        const verdEl = document.getElementById('forge-out-verdict');
        if (verdEl) {
          verdEl.textContent = data.scan_report?.verdict === 'safe' ? '✓ Verificada' : '⚠️ Revisar';
        }
        document.getElementById('forge-engine-note').textContent = data.engine_note || '';

        const lintBox = document.getElementById('forge-lint-feedback');
        if (lintBox) {
          const issues = data.lint_issues || [];
          lintBox.innerHTML = issues.length === 0
            ? '<span style="color:#4ade80;">✓ Habilidade pronta, enxuta e dentro das melhores práticas!</span>'
            : issues.map(i => `<div style="color:#fbbf24;">💡 ${escapeSkillsHtml(i.message)}</div>`).join('');
        }

        if (data.diff) {
          renderMyersDiffInto('forge-diff-viewer', data.diff);
        } else {
          document.getElementById('forge-diff-viewer').textContent = 'Sem versão base anterior para comparar.';
        }

        showSkillsToast(`Habilidade "${data.slug}" pronta! Revise ao lado e clique em Publicar.`, 'success');
      } catch (err) {
        console.error('Erro em runSkillForge:', err);
        showSkillsToast('Erro ao gerar rascunho da habilidade.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.textContent = origTxt || '✨ Gerar Habilidade';
        }
      }
    }

    async function publishForgedSkill() {
      const content = (document.getElementById('forge-output-editor')?.value || '').trim();
      const slug = (document.getElementById('forge-slug')?.value || skillsHubState.forgeLastResult?.slug || '').trim();
      const category = (document.getElementById('forge-category')?.value || '').trim();
      if (!content || !slug) {
        showSkillsToast('Gere ou preencha o nome e o texto da habilidade antes de publicar.', 'warning');
        return;
      }

      const targets = getCheckedTargetKeys('forge-publish-targets');
      if (targets.length === 0) {
        showSkillsToast('Escolha pelo menos um ambiente onde publicar a habilidade.', 'warning');
        return;
      }

      try {
        const res = await fetch('/api/v1/skills/sync', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            content,
            targets,
            category: category || null,
            reason: 'studio_publish'
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          showSkillsToast(data.message || 'Erro ao publicar habilidade.', 'error');
          return;
        }
        showSkillsToast(`"${data.slug || slug}" publicada com sucesso!`, 'success');
        await fetchInstalledSkills(false);
      } catch (err) {
        console.error('Erro em publishForgedSkill:', err);
        showSkillsToast('Erro ao publicar habilidade.', 'error');
      }
    }

    // =========================================================================
    // 4. CUIDADOS & HISTORICO — RECOMENDACOES ACIONAVEIS & COMPARADOR VISUAL
    // =========================================================================

    function renderSkillsHealthRadar() {
      const ov = skillsHubState.overview;
      if (!ov) return;

      const curBox = document.getElementById('health-curator-summary');
      if (curBox) {
        const pausedN = ov.disabled_skills_count || 0;
        curBox.innerHTML = `Você possui <strong>${ov.total_unique_skills}</strong> habilidades instaladas ` +
          `(${pausedN > 0 ? `<strong>${pausedN}</strong> pausadas, ` : ''}` +
          `consumo atual de <strong>${(ov.total_level1_boot_tokens || 0).toLocaleString('pt-BR')} tokens</strong> no início das conversas).`;
      }

      // Populate selector on right column
      const sel = document.getElementById('health-skill-selector');
      if (sel) {
        const prev = sel.value;
        sel.innerHTML = `<option value="">-- Selecione uma habilidade instalada --</option>` +
          (skillsHubState.skills || []).map(s => {
            const tag = !s.in_sync ? ' ⚡ (diferente entre ambientes)' : '';
            return `<option value="${escapeSkillsHtml(s.slug)}">${escapeSkillsHtml(s.slug)}${tag}</option>`;
          }).join('');
        if (prev) sel.value = prev;
      }

      const oppBox = document.getElementById('health-opportunities-list');
      if (!oppBox) return;

      const cards = [];
      (skillsHubState.skills || []).forEach(s => {
        if (s.enabled === false) return;

        // 1. Drift across surfaces
        if (!s.in_sync) {
          cards.push({
            prio: 1,
            border: 'rgba(245,158,11,0.4)',
            bg: 'rgba(245,158,11,0.08)',
            title: `⚡ ${s.slug} — versões diferentes nos seus ambientes`,
            desc: `Encontramos ${s.unique_hashes_count} versões diferentes desta skill entre seus ambientes. Compare lado a lado para escolher a melhor.`,
            actionHtml: `<button class="btn-action primary" style="padding:4px 10px; font-size:0.73rem;" onclick="inspectHealthDriftFor('${escapeSkillsHtml(s.slug)}')">🔍 Comparar Versões</button>`
          });
        }

        // 2. Security / Win11 specific explanation
        if (s.security_verdict !== 'safe' || s.win11_compatible === false) {
          const isMac = s.win11_compatible === false && s.security_verdict === 'safe';
          cards.push({
            prio: 2,
            border: isMac ? 'rgba(56,189,248,0.35)' : 'rgba(245,158,11,0.35)',
            bg: isMac ? 'rgba(56,189,248,0.07)' : 'rgba(245,158,11,0.07)',
            title: `${isMac ? '🐧' : '⚠️'} ${s.slug} — ${isMac ? 'feita originalmente para Mac/Linux' : 'aviso preventivo'}`,
            desc: s.security_human_summary || 'Clique para ver o motivo detalhado.',
            actionHtml: isMac
              ? `<button class="btn-action" style="padding:4px 10px; font-size:0.73rem;" onclick="openInstalledInForge('${escapeSkillsHtml(s.slug)}')">✨ Adaptar p/ Windows</button>`
              : `<button class="btn-action" style="padding:4px 10px; font-size:0.73rem;" onclick="inspectSkillAndSwitch('${escapeSkillsHtml(s.slug)}')">✏️ Ver Motivo</button>`
          });
        }

        // 3. Long summary > 60 chars (1-click safe autofix!)
        if ((s.description_len || 0) > 60) {
          cards.push({
            prio: 3,
            border: 'rgba(192,132,252,0.35)',
            bg: 'rgba(192,132,252,0.07)',
            title: `✂️ ${s.slug} — resumo com ${s.description_len} caracteres`,
            desc: `Encurtar o resumo para até 60 caracteres economiza memória fixa da IA sem alterar nenhuma instrução da skill.`,
            actionHtml: `<button class="btn-action primary" style="padding:4px 10px; font-size:0.73rem;" onclick="quickAutofixAndOpen('${escapeSkillsHtml(s.slug)}')">✨ Ajustar Resumo</button>`
          });
        }
      });

      // Add up to 5 unused skills as gentle optional pause suggestions
      const unusedList = (skillsHubState.skills || []).filter(s => s.enabled !== false && s.is_zombie).slice(0, 5);
      unusedList.forEach(s => {
        cards.push({
          prio: 4,
          border: 'rgba(148,163,184,0.22)',
          bg: 'rgba(15,23,42,0.5)',
          title: `💤 ${s.slug} — ainda sem uso registrado`,
          desc: `Se você não estiver usando esta skill no momento, pode pausá-la para economizar ${s.level1_tokens} tokens fixos e reativá-la quando quiser.`,
          actionHtml: `<button class="btn-action" style="padding:4px 10px; font-size:0.73rem;" onclick="toggleSkillDisabled('${escapeSkillsHtml(s.slug)}', true)">⏸️ Pausar</button>`
        });
      });

      cards.sort((a, b) => a.prio - b.prio);

      if (cards.length === 0) {
        oppBox.innerHTML = `<div style="padding:18px; text-align:center; color:#4ade80; background:rgba(34,197,94,0.08); border-radius:8px; border:1px solid rgba(34,197,94,0.25);">
          ✓ Seu conjunto de habilidades está super enxuto, sincronizado e em dia!
        </div>`;
        return;
      }

      oppBox.innerHTML = cards.map(c => `<div style="padding:10px 12px; border-radius:8px; background:${c.bg}; border:1px solid ${c.border}; display:flex; justify-content:space-between; align-items:center; gap:12px;">
        <div style="min-width:0;">
          <div style="font-weight:700; color:#f1f5f9; font-size:0.81rem;">${escapeSkillsHtml(c.title)}</div>
          <div style="color:#cbd5e1; font-size:0.74rem; margin-top:3px; line-height:1.4;">${escapeSkillsHtml(c.desc)}</div>
        </div>
        <div style="flex-shrink:0;">${c.actionHtml}</div>
      </div>`).join('');
    }

    function inspectSkillAndSwitch(slug) {
      switchSkillsSubtab('installed');
      inspectSkill(slug);
    }

    async function loadHealthSkillTimeline() {
      const slug = document.getElementById('health-skill-selector')?.value;
      const box = document.getElementById('health-timeline-box');
      if (!box) return;
      if (!slug) {
        box.innerHTML = '<div style="color:#64748b;">Escolha uma skill acima para ver o histórico de versões salvas.</div>';
        return;
      }

      try {
        const res = await fetch('/api/v1/skills/detail?slug=' + encodeURIComponent(slug));
        const data = await res.json();
        if (data.status !== 'success') return;
        const hist = data.history || [];
        if (hist.length === 0) {
          box.innerHTML = `<div style="color:#94a3b8;">"${escapeSkillsHtml(slug)}" ainda não possui versões anteriores gravadas no histórico.</div>`;
          return;
        }
        box.innerHTML = hist.map(h => {
          const dt = h.timestamp_epoch ? new Date(h.timestamp_epoch * 1000).toLocaleString('pt-BR') : h.snapshot_id;
          return `<div style="display:flex; justify-content:space-between; align-items:center; padding:5px 8px; margin-bottom:4px; border-radius:6px; background:rgba(15,23,42,0.6); border:1px solid rgba(148,163,184,0.14);">
            <span><strong>${escapeSkillsHtml(dt)}</strong> <span style="color:#94a3b8;">(${escapeSkillsHtml(h.reason)})</span></span>
            <button class="btn-action" style="padding:2px 8px; font-size:0.7rem;" onclick="rollbackSkillToSnapshot('${escapeSkillsHtml(slug)}', '${escapeSkillsHtml(h.snapshot_id)}')">⏪ Restaurar</button>
          </div>`;
        }).join('');
      } catch (err) {
        console.error('Erro em loadHealthSkillTimeline:', err);
      }
    }

    async function openDriftDiffFromInspector() {
      if (!skillsHubState.inspectedRecord) return;
      const slug = skillsHubState.inspectedRecord.slug;
      switchSkillsSubtab('health');
      await inspectHealthDriftFor(slug);
    }

    async function inspectHealthDriftFor(slug) {
      const sel = document.getElementById('health-skill-selector');
      if (sel) {
        sel.value = slug;
        await loadHealthSkillTimeline();
      }
      await compareSelectedHealthDrift(slug);
    }

    async function compareSelectedHealthDrift(explicitSlug) {
      const slug = explicitSlug || document.getElementById('health-skill-selector')?.value;
      if (!slug) {
        showSkillsToast('Escolha uma habilidade no seletor primeiro.', 'warning');
        return;
      }

      const rec = (skillsHubState.skills || []).find(s => s.slug === slug);
      if (!rec || (rec.surfaces || []).length < 2) {
        document.getElementById('health-diff-metrics').textContent = `"${slug}" está presente em apenas 1 ambiente no momento.`;
        document.getElementById('health-diff-viewer').textContent = 'Não há dois ambientes diferentes para comparar nesta skill.';
        return;
      }

      const leftSurf = rec.surfaces[0].target_key;
      let rightSurf = rec.surfaces[1].target_key;
      for (let i = 1; i < rec.surfaces.length; i++) {
        if (rec.surfaces[i].sha256_lf !== rec.surfaces[0].sha256_lf) {
          rightSurf = rec.surfaces[i].target_key;
          break;
        }
      }

      try {
        const res = await fetch('/api/v1/skills/diff', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            left_surface: leftSurf,
            right_surface: rightSurf
          })
        });
        const data = await res.json();
        const report = data?.diff || data;
        if (data.status !== 'success' || !report) {
          showSkillsToast(data.message || 'Não foi possível comparar os ambientes.', 'error');
          return;
        }

        const metricsEl = document.getElementById('health-diff-metrics');
        if (metricsEl) {
          metricsEl.textContent = report.unchanged
            ? `✓ As cópias de "${slug}" em ${leftSurf} e ${rightSurf} são idênticas!`
            : `Comparando ${leftSurf} ↔ ${rightSurf}: +${report.additions} linhas / -${report.deletions} linhas • Qualidade ${report.before_score} → ${report.after_score}`;
        }
        renderMyersDiffInto('health-diff-viewer', report);
      } catch (err) {
        console.error('Erro em compareSelectedHealthDrift:', err);
      }
    }

    function renderMyersDiffInto(containerId, rawReport) {
      const box = document.getElementById(containerId);
      if (!box) return;
      const report = rawReport?.diff || rawReport;
      if (!report || !Array.isArray(report.lines)) {
        box.textContent = 'Nenhuma diferença disponível.';
        return;
      }
      if (report.unchanged) {
        box.innerHTML = `<div style="color:#4ade80; padding:8px;">✓ Os arquivos são 100% idênticos!</div>`;
        return;
      }

      const header = `<div style="padding:6px 8px; margin-bottom:8px; border-radius:6px; background:rgba(30,41,59,0.75); color:#cbd5e1; font-size:0.73rem;">
        <strong>Resumo:</strong> +${report.additions || 0} adições / -${report.deletions || 0} remoções • Qualidade: ${report.before_score ?? '-'} → ${report.after_score ?? '-'}
      </div>`;

      const linesHtml = report.lines.slice(0, 400).map(ln => {
        if (ln.tag === 'insert') {
          return `<div style="background:rgba(34,197,94,0.14); color:#86efac; padding:1px 6px; white-space:pre-wrap;">+ ${escapeSkillsHtml(ln.text)}</div>`;
        }
        if (ln.tag === 'delete') {
          return `<div style="background:rgba(239,68,68,0.14); color:#fca5a5; padding:1px 6px; white-space:pre-wrap;">- ${escapeSkillsHtml(ln.text)}</div>`;
        }
        return `<div style="color:#94a3b8; padding:1px 6px; white-space:pre-wrap;">  ${escapeSkillsHtml(ln.text)}</div>`;
      }).join('');

      box.innerHTML = header + linesHtml;
    }
