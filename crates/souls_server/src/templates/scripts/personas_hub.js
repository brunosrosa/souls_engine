    // =========================================================================
    // PERSONAS HUB — ASSISTENTES HERMES, COERENCIA & ESTUDIO DE PERSONAS (v0.6.0)
    // =========================================================================

    let personasHubState = {
      initialized: false,
      currentSubtab: 'gallery',
      overview: null,
      personas: [],
      archived: [],
      activeSlug: null,
      activeDetail: null,
      workbenchMode: 'visual',
      alignPreviewSlug: null,
      studioTemplate: 'studies',
      studioSelectedSkills: new Set(),
      allInstalledSkills: []
    };

    const PERSONA_STUDIO_PRESETS = {
      studies: {
        title: 'Mentor de Estudos & Pesquisa',
        slug: 'estudos',
        color: '#38bdf8',
        privacy: 'standard',
        goal: 'Organizar trilhas de aprendizado semanais, explicar conceitos complexos de forma simples, criar resumos diretos e revisar meu entendimento com perguntas práticas.'
      },
      coding: {
        title: 'Arquiteto de Código & Projetos',
        slug: 'engenharia',
        color: '#8b5cf6',
        privacy: 'standard',
        goal: 'Apoiar no desenho de software limpo, revisão cirúrgica de código sem quebrar contratos existentes e testes automatizados objetivos.'
      },
      finance: {
        title: 'Consultor de Finanças Pessoais',
        slug: 'planejamento-financeiro',
        color: '#10b981',
        privacy: 'strict_private',
        goal: 'Ajudar na organização de fluxo de caixa, revisão de gastos recorrentes e planejamento de metas financeiras com total sigilo local.'
      },
      health: {
        title: 'Guardião de Saúde & Bem-Estar',
        slug: 'bem-estar',
        color: '#06b6d4',
        privacy: 'strict_private',
        goal: 'Acompanhar rotina de sono, energia, hábitos saudáveis e exames de forma preventiva, clara e sem alarmismo.'
      },
      creative: {
        title: 'Parceiro de Escrita & Ideias',
        slug: 'criativo',
        color: '#f59e0b',
        privacy: 'standard',
        goal: 'Estruturar textos, apresentações e ideias criativas preservando meu estilo pessoal e indo direto ao ponto.'
      },
      custom: {
        title: '',
        slug: '',
        color: '#38bdf8',
        privacy: 'standard',
        goal: ''
      }
    };

    function escPersonaHtml(str) {
      return String(str == null ? '' : str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    }

    function notifyPersonaToast(msg, kind) {
      if (typeof showToast === 'function') {
        showToast(msg, kind || 'success');
      } else {
        console.log('[PersonasHub]', kind || 'info', msg);
      }
    }

    async function fetchPersonasCountBadgeOnly() {
      try {
        const res = await fetch('/api/v1/personas/list');
        const data = await res.json();
        if (data.status === 'success' && data.overview) {
          const el = document.getElementById('badge-personas-count');
          if (el) el.textContent = String(data.overview.total_active_personas || 0);
        }
      } catch (_) {}
    }

    function initPersonasHubTab() {
      fetchPersonasEcosystem(!personasHubState.initialized);
      if (!personasHubState.initialized) {
        personasHubState.initialized = true;
        loadSkillsListForPersonaStudio();
        applyPersonaStudioPreset('studies', document.querySelector('#pstudio-templates-bar .skills-topic-pill'));
      }
    }

    function switchPersonasSubtab(subtab) {
      personasHubState.currentSubtab = subtab;
      ['gallery', 'governance', 'studio', 'history'].forEach(k => {
        const btn = document.getElementById('btn-personas-subtab-' + k);
        const panel = document.getElementById('panel-personas-' + k);
        if (btn) btn.classList.toggle('active', k === subtab);
        if (panel) panel.style.display = k === subtab ? 'block' : 'none';
      });

      if (subtab === 'gallery') {
        fetchPersonasEcosystem(false);
      } else if (subtab === 'governance') {
        renderPersonasGovernanceList();
      } else if (subtab === 'studio') {
        populateStudioCloneDropdown();
        loadSkillsListForPersonaStudio();
        if (!document.getElementById('pstudio-soul-preview')?.value) {
          previewStudioPersonaSoul();
        }
      } else if (subtab === 'history') {
        populatePersonasHistorySelectors();
        renderArchivedPersonasList();
      }
    }

    // =========================================================================
    // 1. GALERIA DE PERSONAS & BANCADA DE EDICAO DUPLA
    // =========================================================================

    async function fetchPersonasEcosystem(forceToast) {
      try {
        const res = await fetch('/api/v1/personas/list');
        const data = await res.json();
        if (data.status !== 'success' || !data.overview) {
          if (forceToast) notifyPersonaToast(data.message || 'Não foi possível carregar suas Personas.', 'error');
          return;
        }

        const ov = data.overview;
        personasHubState.overview = ov;
        personasHubState.personas = ov.personas || [];
        personasHubState.archived = ov.archived_personas || [];

        updatePersonasKpis(ov);
        renderPersonasGalleryCards();

        if (personasHubState.currentSubtab === 'governance') {
          renderPersonasGovernanceList();
        } else if (personasHubState.currentSubtab === 'history') {
          populatePersonasHistorySelectors();
          renderArchivedPersonasList();
        }

        if (forceToast) {
          notifyPersonaToast(`Lista atualizada: ${ov.total_active_personas} assistente(s) ativo(s).`, 'success');
        }
      } catch (err) {
        console.error('Erro em fetchPersonasEcosystem:', err);
        if (forceToast) notifyPersonaToast('Falha ao consultar Personas no servidor local.', 'error');
      }
    }

    function updatePersonasKpis(ov) {
      const setTxt = (id, v) => {
        const el = document.getElementById(id);
        if (el) el.textContent = v;
      };

      setTxt('personas-kpi-total', String(ov.total_active_personas || 0));
      setTxt('personas-kpi-sub-active', `${ov.total_active_personas || 0} ativas`);
      setTxt('personas-kpi-sub-archived', `${ov.archived_personas_count || 0} arquivadas`);
      setTxt('personas-kpi-mcp', `${ov.souls_connected_count || 0} / ${ov.total_active_personas || 0}`);
      setTxt(
        'personas-kpi-mcp-sub',
        ov.souls_connected_count === ov.total_active_personas && ov.total_active_personas > 0
          ? 'Todas conectadas ao motor local (127.0.0.1:9123)'
          : 'Algumas Personas ainda estão sem aceleração local'
      );
      setTxt('personas-kpi-score', `${Math.round(ov.avg_alignment_score || 100)}/100`);

      const alertsEl = document.getElementById('personas-kpi-alerts');
      if (alertsEl) {
        if ((ov.privacy_alerts_count || 0) > 0) {
          alertsEl.style.color = '#f87171';
          alertsEl.textContent = `${ov.privacy_alerts_count} alerta(s) de privacidade • ${ov.attention_needed_count} sugestão(ões)`;
        } else if ((ov.attention_needed_count || 0) > 0) {
          alertsEl.style.color = '#fbbf24';
          alertsEl.textContent = `${ov.attention_needed_count} ajuste(s) simples recomendado(s)`;
        } else {
          alertsEl.style.color = '#4ade80';
          alertsEl.textContent = 'Privacidade e conexões em dia';
        }
      }

      setTxt('personas-kpi-skills', String(ov.total_persona_skills_links || 0));
      setTxt('badge-personas-active-count', String(ov.total_active_personas || 0));
      setTxt('badge-personas-count', String(ov.total_active_personas || 0));

      const govPill = document.getElementById('badge-personas-gov-alerts');
      if (govPill) {
        if ((ov.attention_needed_count || 0) > 0) {
          govPill.textContent = String(ov.attention_needed_count);
          govPill.style.display = 'inline-block';
        } else {
          govPill.style.display = 'none';
        }
      }

      const archPill = document.getElementById('badge-personas-archived-pill');
      if (archPill) {
        if ((ov.archived_personas_count || 0) > 0) {
          archPill.textContent = `${ov.archived_personas_count} arquivada(s)`;
          archPill.style.display = 'inline-block';
        } else {
          archPill.style.display = 'none';
        }
      }

      const notInstalled = document.getElementById('personas-not-installed-banner');
      if (notInstalled) {
        notInstalled.style.display = ov.hermes_installed === false ? 'block' : 'none';
      }
    }

    function handlePersonaAvatarError(imgEl) {
      if (!imgEl || !imgEl.parentElement) return;
      imgEl.style.display = 'none';
      imgEl.parentElement.innerHTML = uiIcon('masks', 'icon-md');
    }

    function buildPersonaAvatarHtml(p) {
      if (p.has_custom_avatar) {
        const url = `/api/v1/personas/avatar?slug=${encodeURIComponent(p.slug)}`;
        return `<div class="persona-avatar-ring">
          <img src="${url}" alt="${escPersonaHtml(p.display_title)}" onerror="handlePersonaAvatarError(this)" />
        </div>`;
      }
      const initials = (p.display_title || p.slug || 'P').trim().slice(0, 2).toUpperCase();
      return `<div class="persona-avatar-ring">
        ${escPersonaHtml(initials)}
      </div>`;
    }

    function buildPersonaStatusBadgeHtml(p) {
      if (p.status_badge === 'privacy_alert') {
        return `<span class="persona-pill badge-danger" title="${escPersonaHtml(p.governance_summary)}"><span class="status-dot status-danger"></span> Alerta de Privacidade</span>`;
      }
      if (p.status_badge === 'needs_attention') {
        return `<span class="persona-pill badge-lowcost" title="${escPersonaHtml(p.governance_summary)}"><span class="status-dot status-warn"></span> Falta Conectar Motor</span>`;
      }
      const cleanStatus = (p.status_label || 'Alinhada').replace(/^[\u2713\u2714\u2705]\s*/, '');
      return `<span class="u-d-inline-flex u-items-center u-gap-6p u-text-muted u-s-dbc8d1" title="${escPersonaHtml(p.governance_summary)}"><span class="status-dot status-ok"></span> ${escPersonaHtml(cleanStatus)}</span>`;
    }

    function renderPersonasGalleryCards() {
      const grid = document.getElementById('personas-cards-grid');
      if (!grid) return;

      const list = personasHubState.personas || [];
      if (list.length === 0) {
        grid.innerHTML = '';
        return;
      }

      grid.innerHTML = list.map(p => {
        const isEditing = personasHubState.activeSlug === p.slug;
        const roleMeta = p.is_default ? 'Principal' : `Perfil: ${escPersonaHtml(p.slug)}`;
        const mcpMeta = p.souls_mcp_connected ? 'Souls Engine Ativo' : 'Sem Motor Local';
        const privMeta = p.requires_strict_privacy ? ' · Dados Sensíveis' : '';
        const skillsTxt = p.is_default
          ? 'Skills Globais do Hermes'
          : `${p.active_skills_count} skill(s) dedicada(s)`;

        const alignBtn = (p.findings && p.findings.some(f => f.auto_fixable))
          ? `<button class="btn-action primary u-s-3ada9a" onclick="openGovernancePreviewFor('${escPersonaHtml(p.slug)}')">${uiIcon('sparkles', 'icon-xs')} Revisar &amp; Alinhar</button>`
          : '';

        return `<div class="persona-card ${isEditing ? 'active-edit' : ''}">
          <div>
            <div class="u-d-flex u-justify-between u-items-start u-gap-10p">
              <div class="u-d-flex u-items-center u-gap-3 u-min-w-0">
                ${buildPersonaAvatarHtml(p)}
                <div class="u-min-w-0">
                  <div class="u-d-flex u-items-center u-gap-2 u-flex-wrap">
                    <span class="u-s-03c535">${escPersonaHtml(p.display_title)}</span>
                    <span class="u-text-muted u-s-dbc8d1">&middot; ${roleMeta}</span>
                  </div>
                  <div class="u-text-muted u-mt-2p u-nowrap u-overflow-hidden u-s-f38243" title="${escPersonaHtml(p.subtitle_role)}">
                    ${escPersonaHtml(p.subtitle_role)}
                  </div>
                </div>
              </div>
              ${buildPersonaStatusBadgeHtml(p)}
            </div>

            <div class="skills-mp-desc-clamp u-mt-10p u-text-slate u-s-a3d012">
              ${escPersonaHtml(p.description)}
            </div>

            <div class="persona-meta-strip u-mt-10p u-text-muted u-s-dbc8d1">
              <span class="u-mono u-text-white" title="Modelo principal configurado nesta Persona">${escPersonaHtml(p.model_default || 'padrão')}</span>
              <span>&middot;</span>
              <span>${mcpMeta}${privMeta}</span>
              <span>&middot;</span>
              <span>${escPersonaHtml(skillsTxt)}</span>
            </div>
          </div>

          <div class="u-d-flex u-justify-between u-items-center u-gap-2 u-s-f80f26">
            <span class="u-text-muted u-s-dbc8d1">Coerência: <strong class="u-text-white">${p.alignment_score}/100</strong> &middot; ${p.soul_tokens} tk</span>
            <div class="u-d-flex u-gap-6p u-flex-wrap">
              ${alignBtn}
              <button class="btn-action u-s-3ada9a" onclick="openPersonaWorkbench('${escPersonaHtml(p.slug)}')">${uiIcon('edit', 'icon-xs')} Configurar</button>
              <button class="btn-action u-s-440209" title="Criar uma nova Persona usando esta como ponto de partida" onclick="clonePersonaInStudio('${escPersonaHtml(p.slug)}')">${uiIcon('dna', 'icon-xs')} Clonar</button>
            </div>
          </div>
        </div>`;
      }).join('');
    }

    async function openPersonaWorkbench(slug, modeOverride) {
      personasHubState.activeSlug = slug;
      renderPersonasGalleryCards();

      try {
        const res = await fetch('/api/v1/personas/detail?slug=' + encodeURIComponent(slug));
        const data = await res.json();
        if (data.status !== 'success' || !data.persona) {
          notifyPersonaToast(data.message || 'Não foi possível carregar os detalhes desta Persona.', 'error');
          return;
        }

        personasHubState.activeDetail = data;
        const p = data.persona;

        const wb = document.getElementById('persona-workbench-card');
        if (wb) {
          wb.style.display = 'block';
          wb.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }

        // Header
        const avBox = document.getElementById('pwb-avatar');
        if (avBox) {
          avBox.outerHTML = buildPersonaAvatarHtml(p).replace('class="persona-avatar-ring"', 'class="persona-avatar-ring" id="pwb-avatar"');
        }
        document.getElementById('pwb-title').innerHTML = `${uiIcon('edit', 'icon-sm')} ${escPersonaHtml(p.display_title)} (${p.is_default ? 'Principal' : escPersonaHtml(p.slug)})`;
        document.getElementById('pwb-score-badge').textContent = `Coerência ${p.alignment_score}/100`;
        document.getElementById('pwb-status-badge').innerHTML = buildPersonaStatusBadgeHtml(p);
        document.getElementById('pwb-path').textContent = p.root_path || '';

        const quickAlignBtn = document.getElementById('btn-pwb-quick-align');
        if (quickAlignBtn) {
          quickAlignBtn.style.display = (p.findings || []).some(f => f.auto_fixable) ? 'inline-flex' : 'none';
        }
        const archBtn = document.getElementById('btn-pwb-archive');
        if (archBtn) {
          archBtn.style.display = p.is_default ? 'none' : 'inline-flex';
        }

        // Governance Callout
        renderWorkbenchGovernanceCallout(p);

        // Mode 1: Visual fields
        document.getElementById('pwb-input-title').value = p.display_title || '';
        document.getElementById('pwb-input-color').value = p.accent_color || '#8b5cf6';
        document.getElementById('pwb-input-desc').value = data.raw_profile_description || p.description || '';
        document.getElementById('pwb-input-model').value = p.model_default || '';
        document.getElementById('pwb-input-provider').value = p.model_provider || 'nous';
        document.getElementById('pwb-input-reasoning').value = p.reasoning_effort || 'high';
        document.getElementById('pwb-input-max-turns').value = String(p.max_turns || 500);
        document.getElementById('pwb-chk-souls-mcp').checked = !!p.souls_mcp_connected;
        document.getElementById('pwb-chk-memory').checked = !!p.memory_enabled;
        document.getElementById('pwb-chk-strict-privacy').checked = !!p.requires_strict_privacy;

        renderModelQuickSuggestions(p);
        renderWorkbenchSkills(p, data.skills || []);
        renderWorkbenchAnchoredContracts(data.anatomy?.anchored_paths || [], data.anchored_contracts || []);
        renderWorkbenchReadOnlyMemories(data.memories_read_only || []);

        // Mode 2: SOUL.md & History
        const soulEditor = document.getElementById('pwb-soul-editor');
        if (soulEditor) {
          soulEditor.value = data.soul_content || '';
          soulEditor.dispatchEvent(new Event('input'));
        }
        document.getElementById('pwb-soul-instruction').value = '';
        const diffBox = document.getElementById('pwb-soul-diff-box');
        if (diffBox) diffBox.style.display = 'none';

        renderWorkbenchSoulSections(data.anatomy?.sections || []);
        renderWorkbenchHistoryList(p.slug, data.history || []);

        switchPersonaWorkbenchMode(modeOverride || personasHubState.workbenchMode || 'visual');
      } catch (err) {
        console.error('Erro em openPersonaWorkbench:', err);
        notifyPersonaToast('Erro ao abrir configuração da Persona.', 'error');
      }
    }

    function switchPersonaWorkbenchMode(mode) {
      personasHubState.workbenchMode = mode;
      const vPane = document.getElementById('pwb-pane-visual');
      const sPane = document.getElementById('pwb-pane-soul');
      const vBtn = document.getElementById('btn-pwb-mode-visual');
      const sBtn = document.getElementById('btn-pwb-mode-soul');
      if (vPane) vPane.style.display = mode === 'visual' ? 'grid' : 'none';
      if (sPane) sPane.style.display = mode === 'soul' ? 'grid' : 'none';
      if (vBtn) vBtn.classList.toggle('active', mode === 'visual');
      if (sBtn) sBtn.classList.toggle('active', mode === 'soul');
    }

    function closePersonaWorkbench() {
      const wb = document.getElementById('persona-workbench-card');
      if (wb) wb.style.display = 'none';
      personasHubState.activeSlug = null;
      personasHubState.activeDetail = null;
      renderPersonasGalleryCards();
    }

    function renderWorkbenchGovernanceCallout(p) {
      const box = document.getElementById('pwb-governance-callout');
      if (!box) return;
      const findings = p.findings || [];
      if (findings.length === 0) {
        box.style.display = 'none';
        box.innerHTML = '';
        return;
      }

      const hasCrit = findings.some(f => f.severity === 'critical');
      box.className = 'skills-why-callout ' + (hasCrit ? 'danger' : '');
      box.style.display = 'block';

      const items = findings.map(f => `<li class="u-mb-1">
        <strong>${escPersonaHtml(f.title)}:</strong> ${escPersonaHtml(f.explanation)}
      </li>`).join('');

      box.innerHTML = `
        <div class="u-d-flex u-justify-between u-items-center u-gap-10p u-flex-wrap u-mb-6p">
          <strong class="u-s-221324">${hasCrit ? `${uiIcon('lock', 'icon-xs')} Atenção de Privacidade Detectada nesta Persona` : `${uiIcon('lightbulb', 'icon-xs')} Oportunidades de Alinhamento para esta Persona`}</strong>
          <button class="btn-action primary u-s-43310c" onclick="openGovernancePreviewFor('${escPersonaHtml(p.slug)}')">${uiIcon('search', 'icon-xs')} Ver Prévia &amp; Resolver em 1 Clique</button>
        </div>
        <ul class="u-m-0 u-text-slate-light u-s-d6f02b">${items}</ul>
      `;
    }

    function buildDynamicPersonaModelPresets(p) {
      const recModel = personasHubState.overview?.recommended_private_model || 'deepseek/deepseek-chat';
      const recProv = personasHubState.overview?.recommended_private_provider || 'nous';
      const presets = [
        { label: `Padrão Privado (${recModel})`, icon: 'shield', model: recModel, prov: recProv }
      ];

      const champs = personasHubState.cloudChampions || [];
      const execChamp = champs.find(c => (c.profile || '').toLowerCase() === 'executive') || champs[0];
      const codingChamp = champs.find(c => (c.profile || '').toLowerCase() === 'coding') || champs[1];

      if (execChamp && execChamp.quality_leader) {
        const ql = execChamp.quality_leader;
        presets.push({
          label: `Campeão Cloud (${ql.display_name || ql.canonical_id})`,
          icon: 'crown',
          model: ql.canonical_id,
          prov: ql.provider || 'nous'
        });
      }
      if (codingChamp && codingChamp.best_value) {
        const bv = codingChamp.best_value;
        presets.push({
          label: `Melhor ROI (${bv.display_name || bv.canonical_id})`,
          icon: 'zap',
          model: bv.canonical_id,
          prov: bv.provider || 'nous'
        });
      }

      const locals = personasHubState.localModels || [];
      if (locals.length > 0) {
        const bestLocal = locals.find(m => (m.tier || '').includes('Tier 1') && (m.target || '').includes('dGPU'))
          || locals[0];
        const lName = bestLocal.name || bestLocal.model || bestLocal.filename;
        presets.push({
          label: `Silício Local (${lName})`,
          icon: 'cpu',
          model: bestLocal.model || lName,
          prov: 'local_silicon'
        });
      } else {
        presets.push({
          label: 'Silício Local (Sem GGUF instalado)',
          icon: 'cpu',
          model: 'local_silicon',
          prov: 'local_silicon'
        });
      }

      // Deduplicate by model+prov
      const seen = new Set();
      return presets.filter(item => {
        const k = `${item.model}::${item.prov}`;
        if (seen.has(k)) return false;
        seen.add(k);
        return true;
      });
    }

    function renderModelQuickSuggestions(p) {
      const box = document.getElementById('pwb-model-suggestions');
      if (!box) return;

      const draw = () => {
        const presets = buildDynamicPersonaModelPresets(p);
        box.innerHTML = `<span class="u-text-muted u-s-97af25">Sugestões ao vivo (Arena &amp; Silício):</span>` +
          presets.map(pr => `<button type="button" class="skills-topic-pill u-s-eea5f2" onclick="applyPersonaModelChip('${escPersonaHtml(pr.model)}', '${escPersonaHtml(pr.prov)}')">${uiIcon(pr.icon, 'icon-xs')} ${escPersonaHtml(pr.label)}</button>`).join('');
      };

      draw();

      if (!personasHubState.modelSuggestionsLoaded && !personasHubState.modelSuggestionsLoading) {
        personasHubState.modelSuggestionsLoading = true;
        Promise.allSettled([
          fetch('/api/v1/cloud/champions').then(r => r.ok ? r.json() : null),
          fetch('/api/v1/models/local').then(r => r.ok ? r.json() : null)
        ]).then(([cRes, lRes]) => {
          if (cRes.status === 'fulfilled' && cRes.value && Array.isArray(cRes.value.champions)) {
            personasHubState.cloudChampions = cRes.value.champions;
          }
          if (lRes.status === 'fulfilled' && lRes.value && Array.isArray(lRes.value.models)) {
            personasHubState.localModels = lRes.value.models;
          }
          personasHubState.modelSuggestionsLoaded = true;
          personasHubState.modelSuggestionsLoading = false;
          draw();
        }).catch(() => {
          personasHubState.modelSuggestionsLoading = false;
        });
      }
    }

    function applyPersonaModelChip(model, provider) {
      const mEl = document.getElementById('pwb-input-model');
      const pEl = document.getElementById('pwb-input-provider');
      if (mEl) mEl.value = model;
      if (pEl) pEl.value = provider;
    }

    function renderWorkbenchSkills(p, skills) {
      const box = document.getElementById('pwb-skills-list');
      if (!box) return;
      if (p.is_default) {
        box.innerHTML = `<div class="u-text-slate u-s-a2b44b">
          O perfil <strong>Principal</strong> acessa automaticamente todas as habilidades globais ativas no seu Hermes (${skills.length} detectadas). Você pode gerenciá-las na aba <strong>${uiIcon('puzzle', 'icon-xs')} Skills Hub</strong>.
        </div>`;
        return;
      }
      if (!skills || skills.length === 0) {
        box.innerHTML = `<span class="u-text-muted u-s-a2b44b">Nenhuma skill dedicada instalada nesta Persona ainda. Use o botão ao lado para sincronizar skills do catálogo!</span>`;
        return;
      }
      box.innerHTML = skills.map(s => {
        const bg = s.enabled ? 'rgba(56,189,248,0.12)' : 'rgba(148,163,184,0.1)';
        const col = s.enabled ? '#38bdf8' : '#71717A';
        return `<span class="u-mono u-s-6ad55f" title="${escPersonaHtml(s.description || '')}" style="background:${bg}; color:${col};">
          ${s.enabled ? uiIcon('check', 'icon-xs') : uiIcon('pause', 'icon-xs')} ${escPersonaHtml(s.slug)}
        </span>`;
      }).join('');
    }

    function renderWorkbenchAnchoredContracts(paths, contractPreviews) {
      const box = document.getElementById('pwb-anchored-list');
      const prevEl = document.getElementById('pwb-contract-preview-box');
      if (prevEl) prevEl.style.display = 'none';
      if (!box) return;

      if (!paths || paths.length === 0) {
        box.innerHTML = '<div class="u-text-dark u-s-a2b44b">Nenhum caminho de cofre ou contrato externo citado no SOUL.md desta Persona.</div>';
        return;
      }

      window._personaContractMap = {};
      (contractPreviews || []).forEach(c => {
        window._personaContractMap[c.path] = c.content_preview || '';
      });

      box.innerHTML = paths.map(ap => {
        const okBadge = ap.exists_on_host
          ? `<span class="u-fw-700 u-s-41afd4">${uiIcon('check', 'icon-xs')} Encontrado</span>`
          : `<span class="u-text-amber-light u-fw-700 u-s-cb8a8e">${uiIcon('alert-triangle', 'icon-xs')} Não localizado</span>`;
        const canPreview = ap.exists_on_host && window._personaContractMap[ap.resolved_path];
        const previewBtn = canPreview
          ? `<button class="btn-action u-s-cb7d09" onclick="showAnchoredContractPreview('${escPersonaHtml(ap.resolved_path)}')">${uiIcon('book', 'icon-xs')} Ler</button>`
          : '';

        return `<div class="u-d-flex u-justify-between u-items-center u-gap-2 u-s-362d9d">
          <code class="u-text-slate u-overflow-hidden u-nowrap u-s-9f3798" title="${escPersonaHtml(ap.resolved_path)}">${escPersonaHtml(ap.raw_path)}</code>
          <div class="u-d-flex u-items-center u-gap-6p u-shrink-0">${okBadge}${previewBtn}</div>
        </div>`;
      }).join('');
    }

    function showAnchoredContractPreview(resolvedPath) {
      const prevEl = document.getElementById('pwb-contract-preview-box');
      if (!prevEl) return;
      const txt = (window._personaContractMap && window._personaContractMap[resolvedPath]) || '';
      if (!txt) return;
      prevEl.textContent = txt;
      prevEl.style.display = prevEl.style.display === 'block' ? 'none' : 'block';
    }

    function renderWorkbenchReadOnlyMemories(memories) {
      const box = document.getElementById('pwb-memories-readonly');
      if (!box) return;
      if (!memories || memories.length === 0) {
        box.innerHTML = '<span class="u-text-dark">Ainda não há notas gravadas em <code>memories/</code> para esta Persona.</span>';
        return;
      }
      box.innerHTML = memories.map(m => `<div class="u-mb-6p u-s-eed50e">
        <div class="u-fw-700 u-text-slate-light u-s-6444dd">${uiIcon('file-text', 'icon-xs')} ${escPersonaHtml(m.rel_path)} <span class="u-text-dark u-s-1c653f">(${m.size_bytes} bytes • Somente Leitura)</span></div>
        <div class="u-mt-3p u-text-muted u-pre-wrap u-s-dbc8d1">${escPersonaHtml((m.content_preview || '').slice(0, 280))}</div>
      </div>`).join('');
    }

    function renderWorkbenchSoulSections(sections) {
      const box = document.getElementById('pwb-soul-sections');
      if (!box) return;
      if (!sections || sections.length === 0) {
        box.innerHTML = '<div class="u-text-dark u-s-a2b44b">Adicione subtítulos com <code>## Nome da Seção</code> para organizar os capítulos.</div>';
        return;
      }
      box.innerHTML = sections.map(sec => `<div class="u-s-6c0171">
        <div class="u-d-flex u-justify-between u-items-center">
          <strong class="u-text-slate-light u-s-c79ecd">${escPersonaHtml(sec.heading)}</strong>
          <span class="u-text-cyan u-s-923ef2">linha ${sec.line_start} • ${sec.bullet_count} regra(s)</span>
        </div>
        ${sec.preview ? `<div class="u-text-muted u-mt-3p u-s-dbc8d1">${escPersonaHtml(sec.preview)}</div>` : ''}
      </div>`).join('');
    }

    function renderWorkbenchHistoryList(slug, history) {
      const box = document.getElementById('pwb-history-list');
      if (!box) return;
      if (!history || history.length === 0) {
        box.innerHTML = '<div class="u-text-dark">Nenhuma versão anterior salva ainda para esta Persona.</div>';
        return;
      }
      box.innerHTML = history.slice(0, 10).map(h => {
        const dt = h.timestamp_epoch ? new Date(h.timestamp_epoch * 1000).toLocaleString('pt-BR') : h.snapshot_id;
        return `<div class="u-d-flex u-justify-between u-items-center u-gap-2 u-s-95ee69">
          <div>
            <strong class="u-text-slate-light">${escPersonaHtml(h.target_file)}</strong>
            <span class="u-text-muted u-s-29ab0e">${escPersonaHtml(dt)} (${escPersonaHtml(h.reason)})</span>
          </div>
          <button class="btn-action u-s-cd187d" onclick="rollbackPersonaSnapshot('${escPersonaHtml(slug)}', '${escPersonaHtml(h.snapshot_id)}')">${uiIcon('history', 'icon-xs')} Restaurar</button>
        </div>`;
      }).join('');
    }

    async function savePersonaVisualSettings() {
      const slug = personasHubState.activeSlug;
      if (!slug) return;

      const payload = {
        slug,
        display_title: document.getElementById('pwb-input-title')?.value || null,
        accent_color: document.getElementById('pwb-input-color')?.value || null,
        description: document.getElementById('pwb-input-desc')?.value || null,
        model_default: document.getElementById('pwb-input-model')?.value || null,
        model_provider: document.getElementById('pwb-input-provider')?.value || null,
        reasoning_effort: document.getElementById('pwb-input-reasoning')?.value || null,
        max_turns: Number(document.getElementById('pwb-input-max-turns')?.value || 500),
        souls_mcp_connected: !!document.getElementById('pwb-chk-souls-mcp')?.checked,
        memory_enabled: !!document.getElementById('pwb-chk-memory')?.checked,
        sanitize_free_fallbacks: !!document.getElementById('pwb-chk-strict-privacy')?.checked,
        reason: 'painel_visual'
      };

      try {
        const res = await fetch('/api/v1/personas/update', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Falha ao salvar alterações da Persona.', 'error');
          return;
        }
        notifyPersonaToast(`Configurações de "${slug}" salvas com backup automático!`, 'success');
        await fetchPersonasEcosystem(false);
        await openPersonaWorkbench(slug, 'visual');
      } catch (err) {
        console.error('Erro em savePersonaVisualSettings:', err);
        notifyPersonaToast('Erro ao salvar configurações.', 'error');
      }
    }

    async function refineCurrentPersonaSoul() {
      const slug = personasHubState.activeSlug;
      const editor = document.getElementById('pwb-soul-editor');
      const instrEl = document.getElementById('pwb-soul-instruction');
      if (!slug || !editor) return;

      const instrVal = (instrEl?.value || '').trim();
      const provVal = (document.getElementById('pwb-input-provider')?.value || '').toLowerCase();
      const soulsMcpOn = Boolean(document.getElementById('pwb-chk-souls-mcp')?.checked);
      const preferLocalSilicon = provVal === 'local_silicon' || provVal === 'llamacpp' || provVal === 'custom' || (soulsMcpOn && instrVal.length > 0);

      try {
        const res = await fetch('/api/v1/personas/refine-soul', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            current_soul: editor.value,
            instruction: instrVal || null,
            use_local_model: preferLocalSilicon
          })
        });
        const data = await res.json();
        if (data.status !== 'success') return;
        editor.value = data.content || editor.value;
        if (instrEl) instrEl.value = '';
        if (data.diff) {
          renderPersonaDiffLines('pwb-soul-diff-box', [data.diff]);
          document.getElementById('pwb-soul-diff-box').style.display = 'block';
        }
        notifyPersonaToast((data.notes || []).join(' • ') || 'Texto organizado! Clique em Salvar para confirmar.', 'success');
      } catch (err) {
        console.error('Erro em refineCurrentPersonaSoul:', err);
      }
    }

    async function previewCurrentSoulDiff() {
      const orig = personasHubState.activeDetail?.soul_content || '';
      const after = document.getElementById('pwb-soul-editor')?.value || '';
      const box = document.getElementById('pwb-soul-diff-box');
      if (!box) return;

      try {
        const res = await fetch('/api/v1/personas/diff', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            file_name: 'SOUL.md',
            before_content: orig,
            after_content: after
          })
        });
        const data = await res.json();
        if (data.status === 'success' && data.diff) {
          renderPersonaDiffLines('pwb-soul-diff-box', [data.diff]);
          box.style.display = 'block';
        }
      } catch (err) {
        console.error('Erro em previewCurrentSoulDiff:', err);
      }
    }

    async function savePersonaSoulOnly() {
      const slug = personasHubState.activeSlug;
      const editor = document.getElementById('pwb-soul-editor');
      if (!slug || !editor) return;

      try {
        const res = await fetch('/api/v1/personas/update', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            soul_content: editor.value,
            reason: 'edicao_soul_md'
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Erro ao salvar SOUL.md.', 'error');
          return;
        }
        notifyPersonaToast(`Personalidade (SOUL.md) de "${slug}" salva com backup anterior!`, 'success');
        await fetchPersonasEcosystem(false);
        await openPersonaWorkbench(slug, 'soul');
      } catch (err) {
        console.error('Erro em savePersonaSoulOnly:', err);
      }
    }

    function jumpFromPersonaToSkillsHub() {
      const slug = personasHubState.activeSlug;
      if (typeof switchTab === 'function') {
        switchTab('tab-skills');
        if (slug && slug !== 'default' && typeof setSkillsSurfaceFilter === 'function') {
          setSkillsSurfaceFilter('personas');
        }
      }
    }

    // =========================================================================
    // 2. COERENCIA & PRIVACIDADE — DIAGNOSTICO & ALINHAMENTO EM 1 CLIQUE
    // =========================================================================

    function openGovernancePreviewForCurrent() {
      if (!personasHubState.activeSlug) return;
      openGovernancePreviewFor(personasHubState.activeSlug);
    }

    async function openGovernancePreviewFor(slug) {
      switchPersonasSubtab('governance');
      personasHubState.alignPreviewSlug = slug;

      const emptyEl = document.getElementById('align-preview-empty');
      const contentEl = document.getElementById('align-preview-content');
      const headerEl = document.getElementById('align-preview-header');
      if (headerEl) headerEl.textContent = `Prévia de Alinhamento: ${slug}`;

      try {
        const res = await fetch('/api/v1/personas/align', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, dry_run: true })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Não foi possível gerar a prévia.', 'error');
          return;
        }

        if (emptyEl) emptyEl.style.display = 'none';
        if (contentEl) contentEl.style.display = 'block';

        const actions = data.actions_explained || [];
        const actionsBox = document.getElementById('align-actions-list');
        if (actionsBox) {
          if (actions.length === 0) {
            actionsBox.innerHTML = `${uiIcon('check-circle', 'icon-xs')} Esta Persona já está 100% alinhada — nenhuma modificação pendente.`;
          } else {
            actionsBox.innerHTML = `<div class="u-fw-700 u-s-2df161">O que faremos em "${escPersonaHtml(slug)}":</div>` +
              `<ul class="u-m-0 u-s-9d1242">` +
              actions.map(a => `<li class="u-mb-1">${escPersonaHtml(a)}</li>`).join('') +
              `</ul>`;
          }
        }

        const diffs = data.changed_files || [];
        renderPersonaDiffLines('align-diff-viewer', diffs);

        const confirmBtn = document.getElementById('btn-confirm-align');
        if (confirmBtn) {
          confirmBtn.style.display = diffs.length > 0 ? 'inline-flex' : 'none';
        }
      } catch (err) {
        console.error('Erro em openGovernancePreviewFor:', err);
      }
    }

    async function confirmApplyPersonaAlignment() {
      const slug = personasHubState.alignPreviewSlug;
      if (!slug) return;

      const btn = document.getElementById('btn-confirm-align');
      if (btn) btn.disabled = true;

      try {
        const res = await fetch('/api/v1/personas/align', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, dry_run: false })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Erro ao aplicar alinhamento.', 'error');
          return;
        }
        notifyPersonaToast(`"${slug}" alinhada com sucesso! Versões anteriores guardadas no Histórico.`, 'success');
        await fetchPersonasEcosystem(false);
        await openGovernancePreviewFor(slug);
      } catch (err) {
        console.error('Erro em confirmApplyPersonaAlignment:', err);
      } finally {
        if (btn) btn.disabled = false;
      }
    }

    function renderPersonasGovernanceList() {
      const box = document.getElementById('personas-gov-cards-list');
      if (!box) return;

      const items = [];
      (personasHubState.personas || []).forEach(p => {
        (p.findings || []).forEach(f => {
          items.push({ persona: p, finding: f });
        });
      });

      if (items.length === 0) {
        box.innerHTML = `<div class="u-text-center u-s-39f44c">
          ${uiIcon('check-circle', 'icon-sm')} Excelente! Todas as suas Personas estão com privacidade protegida, descrições claras e aceleração Souls Engine ativa.
        </div>`;
        return;
      }

      const rank = { critical: 1, warning: 2, info: 3 };
      items.sort((a, b) => (rank[a.finding.severity] || 9) - (rank[b.finding.severity] || 9));

      box.innerHTML = items.map(({ persona, finding }) => {
        const isCrit = finding.severity === 'critical';
        const isWarn = finding.severity === 'warning';
        const border = isCrit ? 'rgba(239,68,68,0.42)' : (isWarn ? 'rgba(245,158,11,0.38)' : 'rgba(56,189,248,0.3)');
        const bg = isCrit ? 'rgba(239,68,68,0.09)' : (isWarn ? 'rgba(245,158,11,0.08)' : 'rgba(17, 17, 19,0.55)');
        const iconName = isCrit ? 'lock' : (isWarn ? 'plug' : 'lightbulb');

        const btnHtml = finding.auto_fixable
          ? `<button class="btn-action primary u-s-e0a9f8" onclick="openGovernancePreviewFor('${escPersonaHtml(persona.slug)}')">${uiIcon('sparkles', 'icon-xs')} Revisar Ajuste</button>`
          : `<button class="btn-action u-s-e0a9f8" onclick="switchPersonasSubtab('gallery'); openPersonaWorkbench('${escPersonaHtml(persona.slug)}', 'soul');">${uiIcon('edit', 'icon-xs')} Abrir SOUL.md</button>`;

        return `<div class="u-d-flex u-justify-between u-items-center u-gap-3 u-s-2330bb" style="background:${bg}; border:1px solid ${border};">
          <div class="u-min-w-0">
            <div class="u-fw-700 u-s-668559">
              ${uiIcon(iconName, 'icon-xs')} ${escPersonaHtml(persona.display_title)} <span class="u-text-muted u-fw-500">(${escPersonaHtml(persona.slug)})</span> — ${escPersonaHtml(finding.title)}
            </div>
            <div class="u-text-slate u-mt-1 u-s-a60830">${escPersonaHtml(finding.explanation)}</div>
          </div>
          <div class="u-shrink-0">${btnHtml}</div>
        </div>`;
      }).join('');
    }

    function renderPersonaDiffLines(containerId, fileDiffs) {
      const box = document.getElementById(containerId);
      if (!box) return;
      if (!fileDiffs || fileDiffs.length === 0) {
        box.innerHTML = `<div class="u-s-39cf99">${uiIcon('check', 'icon-xs')} Nenhuma diferença pendente nos arquivos.</div>`;
        return;
      }

      box.innerHTML = fileDiffs.map(fd => {
        if (fd.unchanged) {
          return `<div class="u-mb-2 u-s-80aa08">${uiIcon('check', 'icon-xs')} <strong>${escPersonaHtml(fd.file_name)}</strong>: sem alterações.</div>`;
        }
        const hdr = `<div class="u-mb-1 u-text-slate-light u-fw-700 u-s-e6f093">
          ${uiIcon('file-text', 'icon-xs')} ${escPersonaHtml(fd.file_name)} <span class="u-fw-500 u-text-muted">(+${fd.additions} / -${fd.deletions} linhas)</span>
        </div>`;

        // Filter contextual lines around changes so large 350-line config.yaml stays super readable!
        const allLines = fd.lines || [];
        const changedIdx = new Set();
        allLines.forEach((ln, idx) => {
          if (ln.tag !== 'equal') {
            for (let k = Math.max(0, idx - 3); k <= Math.min(allLines.length - 1, idx + 3); k++) {
              changedIdx.add(k);
            }
          }
        });

        let prevIdx = -1;
        const body = [];
        allLines.forEach((ln, idx) => {
          if (!changedIdx.has(idx)) return;
          if (prevIdx !== -1 && idx > prevIdx + 1) {
            body.push('<div class="u-s-47ade5">  ⋯</div>');
          }
          prevIdx = idx;
          if (ln.tag === 'insert') {
            body.push(`<div class="u-pre-wrap u-s-cda35a">+ ${escPersonaHtml(ln.text)}</div>`);
          } else if (ln.tag === 'delete') {
            body.push(`<div class="u-pre-wrap u-s-16928d">- ${escPersonaHtml(ln.text)}</div>`);
          } else {
            body.push(`<div class="u-text-muted u-pre-wrap u-s-7e80e0">  ${escPersonaHtml(ln.text)}</div>`);
          }
        });

        return `<div class="u-mb-3">${hdr}${body.join('')}</div>`;
      }).join('');
    }

    // =========================================================================
    // 3. ESTUDIO DE CRIACAO & CLONAGEM DE PERSONAS
    // =========================================================================

    function openPersonaStudioClean() {
      switchPersonasSubtab('studio');
      applyPersonaStudioPreset('studies', document.querySelector('#pstudio-templates-bar .skills-topic-pill'));
    }

    function clonePersonaInStudio(donorSlug) {
      switchPersonasSubtab('studio');
      populateStudioCloneDropdown();
      const sel = document.getElementById('pstudio-clone-from');
      if (sel) sel.value = donorSlug;

      const donor = (personasHubState.personas || []).find(p => p.slug === donorSlug);
      if (donor) {
        document.getElementById('pstudio-title').value = `${donor.display_title} (Cópia)`;
        document.getElementById('pstudio-slug').value = donor.slug === 'default' ? 'novo-assistente' : `${donor.slug}-v2`;
        document.getElementById('pstudio-color').value = donor.accent_color || '#38bdf8';
        document.getElementById('pstudio-privacy').value = donor.requires_strict_privacy ? 'strict_private' : 'standard';
        document.getElementById('pstudio-goal').value = donor.description || '';
        previewStudioPersonaSoul();
      }
    }

    function populateStudioCloneDropdown() {
      const sel = document.getElementById('pstudio-clone-from');
      if (!sel) return;
      const cur = sel.value || 'default';
      const opts = (personasHubState.personas || []).map(p => {
        const prefix = p.is_default ? '[Principal] ' : '';
        return `<option value="${escPersonaHtml(p.slug)}">${prefix}${escPersonaHtml(p.display_title)} (${escPersonaHtml(p.slug)})</option>`;
      });
      sel.innerHTML = opts.length > 0 ? opts.join('') : '<option value="default">[Principal] Hermes Principal (default)</option>';
      if ((personasHubState.personas || []).some(p => p.slug === cur)) {
        sel.value = cur;
      }
    }

    function applyPersonaStudioPreset(kind, btnEl) {
      personasHubState.studioTemplate = kind;
      const bar = document.getElementById('pstudio-templates-bar');
      if (bar && btnEl) {
        bar.querySelectorAll('.skills-topic-pill').forEach(b => b.classList.remove('active'));
        btnEl.classList.add('active');
      }
      const preset = PERSONA_STUDIO_PRESETS[kind] || PERSONA_STUDIO_PRESETS.custom;
      document.getElementById('pstudio-title').value = preset.title;
      document.getElementById('pstudio-slug').value = preset.slug;
      document.getElementById('pstudio-color').value = preset.color;
      document.getElementById('pstudio-privacy').value = preset.privacy;
      document.getElementById('pstudio-goal').value = preset.goal;
      previewStudioPersonaSoul();
    }

    function autoFillPersonaStudioSlug() {
      const title = document.getElementById('pstudio-title')?.value || '';
      const slugEl = document.getElementById('pstudio-slug');
      if (!slugEl) return;
      slugEl.value = title
        .toLowerCase()
        .normalize('NFD')
        .replace(/[\u0300-\u036f]/g, '')
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
        .slice(0, 36);
      previewStudioPersonaSoul();
    }

    async function previewStudioPersonaSoul() {
      const title = (document.getElementById('pstudio-title')?.value || 'Novo Assistente').trim();
      const slug = (document.getElementById('pstudio-slug')?.value || 'nova-persona').trim();
      const goal = (document.getElementById('pstudio-goal')?.value || 'Apoiar suas tarefas com clareza, organização e proatividade.').trim();
      const strict = document.getElementById('pstudio-privacy')?.value === 'strict_private';
      const engine = document.getElementById('pstudio-engine')?.value || 'deterministic_scaffold';

      const privSection = strict
        ? `## 2. Privacidade Absoluta & Soberania de Dados\n- **Privacidade Absoluta (FACTO)**: Documentos sensíveis, dados pessoais, médicos ou financeiros **NUNCA** devem ser enviados para APIs externas do Free Tier.\n- Priorize modelos privados autenticados ou processamento 100% local via GPU/SLM do **Souls Engine**.`
        : `## 2. Governança de Contexto & Eficiência\n- Utilize as ferramentas locais do **Souls Engine** (\`127.0.0.1:9123/mcp\`) para buscar memórias relevantes e economizar contexto.\n- Antes de delegar subtarefas longas (\`delegate_task\`), consulte \`route_subtask\` para escolher o modelo ideal.`;

      let md = `# SOUL.md — ${title}\n\n` +
        `Você é **${title}**, um assistente dedicado e parceiro estratégico. Sua missão central: ${goal}\n\n` +
        `## 1. Postura & Diretrizes de Atuação\n` +
        `- Atuar com proatividade, clareza e foco prático nos resultados.\n` +
        `- Transformar pedidos complexos em etapas simples, verificáveis e bem estruturadas.\n\n` +
        `${privSection}\n\n` +
        `## 3. Estilo de Comunicação\n` +
        `- Responda sempre em **Português do Brasil (pt-BR)**, de forma direta e acolhedora, destacando **conceitos-chave em negrito**.\n`;

      const prevEl = document.getElementById('pstudio-soul-preview');
      if (prevEl) {
        prevEl.value = md;
        prevEl.dispatchEvent(new Event('input'));
      }

      if (engine === 'local_silicon' && prevEl) {
        try {
          const res = await fetch('/api/v1/personas/refine-soul', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              slug: slug || 'studio-draft',
              current_soul: md,
              instruction: `Enriquecer diretrizes para o objetivo: ${goal}`,
              use_local_model: true
            })
          });
          if (res.ok) {
            const data = await res.json();
            if (data.status === 'success' && data.content) {
              prevEl.value = data.content;
              prevEl.dispatchEvent(new Event('input'));
            }
          }
        } catch (_) {}
      }
    }

    async function loadSkillsListForPersonaStudio() {
      if (personasHubState.allInstalledSkills.length > 0) {
        renderStudioSkillsPicker();
        return;
      }
      try {
        const res = await fetch('/api/v1/skills/installed');
        const data = await res.json();
        if (data.status === 'success' && data.overview) {
          personasHubState.allInstalledSkills = data.overview.skills || [];
          renderStudioSkillsPicker();
        }
      } catch (_) {}
    }

    function renderStudioSkillsPicker() {
      const box = document.getElementById('pstudio-skills-picker');
      if (!box) return;
      const q = (document.getElementById('pstudio-skills-filter')?.value || '').trim().toLowerCase();
      const filtered = (personasHubState.allInstalledSkills || []).filter(s =>
        s.enabled !== false && (!q || s.slug.includes(q) || (s.description || '').toLowerCase().includes(q))
      );

      if (filtered.length === 0) {
        box.innerHTML = '<span class="u-text-dark u-s-6444dd">Nenhuma skill correspondente.</span>';
        return;
      }

      box.innerHTML = filtered.slice(0, 40).map(s => {
        const active = personasHubState.studioSelectedSkills.has(s.slug);
        const bg = active ? 'rgba(6,182,212,0.2)' : 'rgba(17, 17, 19,0.6)';
        const border = active ? 'rgba(6,182,212,0.55)' : 'rgba(148,163,184,0.18)';
        const col = active ? '#67e8f9' : '#E4E4E7';
        return `<button class="u-mono u-pointer u-s-d6ce2d" type="button" onclick="toggleStudioPersonaSkill('${escPersonaHtml(s.slug)}')" title="${escPersonaHtml(s.description || '')}" style="background:${bg}; border:1px solid ${border}; color:${col};">${active ? uiIcon('check', 'icon-xs') : uiIcon('plus', 'icon-xs')} ${escPersonaHtml(s.slug)}</button>`;
      }).join('');
    }

    function toggleStudioPersonaSkill(slug) {
      if (personasHubState.studioSelectedSkills.has(slug)) {
        personasHubState.studioSelectedSkills.delete(slug);
      } else {
        personasHubState.studioSelectedSkills.add(slug);
      }
      renderStudioSkillsPicker();
    }

    async function submitCreatePersona() {
      const title = (document.getElementById('pstudio-title')?.value || '').trim();
      const slug = (document.getElementById('pstudio-slug')?.value || '').trim();
      const goal = (document.getElementById('pstudio-goal')?.value || '').trim();
      if (!title || !slug || !goal) {
        notifyPersonaToast('Preencha o nome, o identificador curto e o objetivo da nova Persona.', 'warning');
        return;
      }

      const btn = document.getElementById('btn-pstudio-create');
      if (btn) btn.disabled = true;

      try {
        const res = await fetch('/api/v1/personas/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            slug,
            display_title: title,
            description: goal.slice(0, 190),
            template_kind: personasHubState.studioTemplate,
            goal_prompt: goal,
            privacy_mode: document.getElementById('pstudio-privacy')?.value || 'standard',
            engine: document.getElementById('pstudio-engine')?.value || 'deterministic_scaffold',
            clone_from: document.getElementById('pstudio-clone-from')?.value || 'default',
            connect_souls_mcp: true,
            initial_skills: Array.from(personasHubState.studioSelectedSkills),
            accent_color: document.getElementById('pstudio-color')?.value || '#38bdf8'
          })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Não foi possível criar a Persona.', 'error');
          return;
        }

        // If user manually customized the preview textarea, save that exact text too
        const customSoul = (document.getElementById('pstudio-soul-preview')?.value || '').trim();
        if (customSoul && data.slug) {
          await fetch('/api/v1/personas/update', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              slug: data.slug,
              soul_content: customSoul,
              reason: 'studio_initial_soul'
            })
          });
        }

        personasHubState.studioSelectedSkills.clear();
        notifyPersonaToast(`Persona "${data.slug}" criada e conectada ao Souls Engine!`, 'success');
        switchPersonasSubtab('gallery');
        await fetchPersonasEcosystem(false);
        await openPersonaWorkbench(data.slug, 'visual');
      } catch (err) {
        console.error('Erro em submitCreatePersona:', err);
        notifyPersonaToast('Erro ao criar nova Persona.', 'error');
      } finally {
        if (btn) btn.disabled = false;
      }
    }

    // =========================================================================
    // 4. HISTORICO, ARQUIVADAS & COMPARADOR LADO A LADO
    // =========================================================================

    function populatePersonasHistorySelectors() {
      const list = personasHubState.personas || [];
      const optsHtml = list.map(p => `<option value="${escPersonaHtml(p.slug)}">${escPersonaHtml(p.display_title)} (${escPersonaHtml(p.slug)})</option>`).join('');

      const leftSel = document.getElementById('pcomp-left-select');
      const rightSel = document.getElementById('pcomp-right-select');
      const histSel = document.getElementById('phist-persona-select');

      if (leftSel && list.length > 0) {
        const prev = leftSel.value;
        leftSel.innerHTML = optsHtml;
        leftSel.value = list.some(p => p.slug === prev) ? prev : list[0].slug;
      }
      if (rightSel && list.length > 0) {
        const prev = rightSel.value;
        rightSel.innerHTML = optsHtml;
        rightSel.value = list.some(p => p.slug === prev) ? prev : (list[1] ? list[1].slug : list[0].slug);
      }
      if (histSel && list.length > 0) {
        const prev = histSel.value;
        histSel.innerHTML = optsHtml;
        histSel.value = list.some(p => p.slug === prev) ? prev : list[0].slug;
        loadPersonaHistoryTab();
      }
    }

    async function runTwoPersonasComparison() {
      const leftSlug = document.getElementById('pcomp-left-select')?.value;
      const rightSlug = document.getElementById('pcomp-right-select')?.value;
      if (!leftSlug || !rightSlug) return;

      try {
        const res = await fetch('/api/v1/personas/diff', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ left_slug: leftSlug, right_slug: rightSlug })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Erro ao comparar Personas.', 'error');
          return;
        }

        document.getElementById('pcomp-th-left').textContent = data.left?.display_title || leftSlug;
        document.getElementById('pcomp-th-right').textContent = data.right?.display_title || rightSlug;

        const tbody = document.getElementById('pcomp-table-body');
        if (tbody) {
          tbody.innerHTML = (data.config_comparison || []).map(row => {
            const diff = String(row.left) !== String(row.right);
            const highlight = diff ? 'color:#fbbf24; font-weight:700;' : 'color:#E4E4E7;';
            return `<tr>
              <td class="u-text-muted">${escPersonaHtml(row.label)}</td>
              <td style="${highlight};">${escPersonaHtml(row.left)}</td>
              <td style="${highlight};">${escPersonaHtml(row.right)}</td>
            </tr>`;
          }).join('');
        }

        if (data.soul_diff) {
          renderPersonaDiffLines('pcomp-soul-diff', [data.soul_diff]);
        }
      } catch (err) {
        console.error('Erro em runTwoPersonasComparison:', err);
      }
    }

    async function loadPersonaHistoryTab() {
      const slug = document.getElementById('phist-persona-select')?.value;
      const box = document.getElementById('phist-snapshots-list');
      if (!slug || !box) return;

      try {
        const res = await fetch('/api/v1/personas/detail?slug=' + encodeURIComponent(slug));
        const data = await res.json();
        if (data.status !== 'success') return;
        const hist = data.history || [];
        if (hist.length === 0) {
          box.innerHTML = `<div class="u-text-muted">"${escPersonaHtml(slug)}" ainda não possui alterações anteriores salvas neste histórico.</div>`;
          return;
        }
        box.innerHTML = hist.map(h => {
          const dt = h.timestamp_epoch ? new Date(h.timestamp_epoch * 1000).toLocaleString('pt-BR') : h.snapshot_id;
          return `<div class="u-d-flex u-justify-between u-items-center u-gap-2 u-s-66b76c">
            <div>
              <strong class="u-text-slate-light">${escPersonaHtml(h.target_file)}</strong>
              <span class="u-text-muted u-s-09f9b3">${escPersonaHtml(dt)} (${escPersonaHtml(h.reason)})</span>
            </div>
            <button class="btn-action u-s-1f75c8" onclick="rollbackPersonaSnapshot('${escPersonaHtml(slug)}', '${escPersonaHtml(h.snapshot_id)}')">${uiIcon('history', 'icon-xs')} Restaurar</button>
          </div>`;
        }).join('');
      } catch (err) {
        console.error('Erro em loadPersonaHistoryTab:', err);
      }
    }

    function renderArchivedPersonasList() {
      const box = document.getElementById('phist-archived-list');
      if (!box) return;
      const list = personasHubState.archived || [];
      if (list.length === 0) {
        box.innerHTML = '<div class="u-text-dark">Nenhuma Persona arquivada no momento.</div>';
        return;
      }
      box.innerHTML = list.map(a => `<div class="u-d-flex u-justify-between u-items-center u-gap-10p u-s-930b91">
        <div>
          <strong class="u-s-84c8c1">${escPersonaHtml(a.display_title)} (${escPersonaHtml(a.slug)})</strong>
          <div class="u-text-muted u-s-dbc8d1">${escPersonaHtml(a.description || '')}</div>
        </div>
        <button class="btn-action primary u-s-42d134" onclick="restoreArchivedPersona('${escPersonaHtml(a.slug)}')">${uiIcon('play', 'icon-xs')} Reativar no Hermes</button>
      </div>`).join('');
    }

    async function archiveCurrentPersona() {
      const slug = personasHubState.activeSlug;
      if (!slug || slug === 'default') return;
      const ok = window.appConfirm ? await window.appConfirm(
        `Deseja arquivar temporariamente a Persona "${slug}"?\n\n` +
        `Todas as conversas, configurações e memórias dela ficarão guardadas em segurança e você poderá reativá-la com 1 clique na aba "Histórico & Comparar".`,
        { title: 'Arquivar Persona', icon: 'archive', confirmText: 'Arquivar com Segurança' }
      ) : window.confirm(
        `Deseja arquivar temporariamente a Persona "${slug}"?\n\n` +
        `Todas as conversas, configurações e memórias dela ficarão guardadas em segurança e você poderá reativá-la com 1 clique na aba "Histórico & Comparar".`
      );
      if (!ok) return;

      try {
        const res = await fetch('/api/v1/personas/archive', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, restore: false })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Erro ao arquivar Persona.', 'error');
          return;
        }
        closePersonaWorkbench();
        notifyPersonaToast(`"${slug}" foi arquivada com segurança!`, 'success');
        await fetchPersonasEcosystem(false);
      } catch (err) {
        console.error('Erro em archiveCurrentPersona:', err);
      }
    }

    async function restoreArchivedPersona(slug) {
      try {
        const res = await fetch('/api/v1/personas/archive', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, restore: true })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Erro ao reativar Persona.', 'error');
          return;
        }
        notifyPersonaToast(`Persona "${slug}" reativada e disponível novamente no Hermes!`, 'success');
        await fetchPersonasEcosystem(false);
      } catch (err) {
        console.error('Erro em restoreArchivedPersona:', err);
      }
    }

    async function rollbackPersonaSnapshot(slug, snapshotId) {
      try {
        const res = await fetch('/api/v1/personas/rollback', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ slug, snapshot_id: snapshotId })
        });
        const data = await res.json();
        if (data.status !== 'success') {
          notifyPersonaToast(data.message || 'Falha ao restaurar versão.', 'error');
          return;
        }
        notifyPersonaToast(`Versão anterior de ${data.restored_file} restaurada para "${slug}"!`, 'success');
        await fetchPersonasEcosystem(false);
        if (personasHubState.activeSlug === slug) {
          await openPersonaWorkbench(slug, personasHubState.workbenchMode);
        }
        if (personasHubState.currentSubtab === 'history') {
          await loadPersonaHistoryTab();
        }
      } catch (err) {
        console.error('Erro em rollbackPersonaSnapshot:', err);
      }
    }

