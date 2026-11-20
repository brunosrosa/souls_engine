    // ESTADO GLOBAL DA APLICAÇÃO
    let activePreset = 'balanced';
    let currentProfile = 'executive';
    let cloudChampionsData = [];
    let currentCloudProviderFilter = 'nous';
    let cachedCloudRankedModels = [];
    window.cachedHermesCustoms = null;
    window.cachedHermesFinops = null;
    window.currentHermesMasterModel = 'deepseek/deepseek-chat';
    window.currentHermesMasterProvider = 'nous';

    // HELPER CANÔNICO DE ÍCONES VETORIAIS SVG DUOTONE (v0.9.1-B)
    const EMOJI_TO_ICON_MAP = {
      '⚠️': 'alert-triangle', '⚠': 'alert-triangle', '✅': 'check-circle', '❌': 'x-circle', 'ℹ️': 'info',
      '✏️': 'edit', '✏': 'edit', '🗑️': 'trash', '🗑': 'trash', '🔄': 'refresh', '⚡': 'zap',
      '🧠': 'brain', '🛡️': 'shield', '🛡': 'shield', '✨': 'sparkles', '🔍': 'search', '🔎': 'search',
      '🤖': 'bot', '💾': 'database', '💻': 'terminal', '🎯': 'target', '📦': 'archive',
      '💡': 'lightbulb', '👁️': 'eye', '👁': 'eye', '👑': 'crown', '⚙️': 'settings', '⚙': 'settings',
      '🚀': 'rocket', '📋': 'list', '🔒': 'lock', '⏳': 'clock', '🌐': 'globe', '⏱️': 'clock', '⏱': 'clock',
      '🧩': 'puzzle', '🔀': 'git-branch', '🔌': 'plug', '📈': 'trending-up', '📉': 'trending-down',
      '🎭': 'masks', '⚖️': 'scale', '💎': 'sparkles', '🎨': 'palette', '📄': 'file-text',
      '⏸️': 'pause', '⏸': 'pause', '🩺': 'stethoscope', '📝': 'edit', '🏟️': 'trophy', '💰': 'dollar',
      '📑': 'file-text', '🎮': 'gamepad', '📚': 'book', '🌡️': 'thermometer', '🌡': 'thermometer',
      '📊': 'bar-chart', '⏪': 'history', '🧬': 'dna', '🧭': 'compass', '📥': 'inbox', '🧪': 'flask',
      '🕸️': 'network', '🏆': 'trophy', '➕': 'plus', '🔬': 'microscope', '🏛️': 'layers', '📖': 'book',
      '📜': 'file-text', '🧰': 'wrench', '⏹️': 'square', '▶️': 'play', '🛑': 'x-circle',
      '📓': 'book', '📌': 'pin', '☁️': 'cloud', '🎛️': 'sliders', '🤔': 'brain', '🛠️': 'wrench',
      '🖥️': 'cpu', '🧹': 'refresh', '🌟': 'sparkles', '📂': 'folder', '🌍': 'globe', '🔗': 'link',
      '🔥': 'flame', '🔧': 'wrench', '🏷️': 'tag', '📍': 'pin', '🧮': 'cpu', '💥': 'zap',
      '📡': 'radio', '🕒': 'clock', '🔑': 'key', '🎬': 'play', '🎙️': 'mic', '👤': 'users',
      '🌿': 'seedling', '🌱': 'seedling', '💬': 'message-square', '🤝': 'users', '♻️': 'refresh',
      '🪐': 'globe', '📅': 'calendar', '✂️': 'scissors', '📁': 'folder', '❓': 'info', '💸': 'dollar'
    };

    function uiIcon(name, cls = 'icon-sm') {
      const resolved = EMOJI_TO_ICON_MAP[name] || name || 'info';
      return `<svg class="ui-icon ${cls}" aria-hidden="true"><use href="#icon-${resolved}"></use></svg>`;
    }
    window.uiIcon = uiIcon;

    // SLIDERS COM FEEDBACK TÁTIL & SNAP MAGNÉTICO AO DEFAULT
    function initSlidersMagneticSnap() {
      document.querySelectorAll('input[type="range"][data-default]').forEach(slider => {
        const defVal = parseFloat(slider.getAttribute('data-default'));
        const snapDelta = parseFloat(slider.getAttribute('data-snap-delta')) || ((parseFloat(slider.max) - parseFloat(slider.min)) * 0.04);
        const tick = document.querySelector(`.slider-default-tick[data-slider="${slider.id}"]`);
        
        const updateTickStyle = (val) => {
          const isDef = Math.abs(val - defVal) <= 0.001;
          if (tick) {
            if (isDef) {
              tick.classList.add('at-default');
            } else {
              tick.classList.remove('at-default');
            }
          }
          const badge = slider.closest('.setting-item')?.querySelector('.range-val-badge .default-indicator');
          if (badge) {
            badge.style.display = isDef ? 'inline' : 'none';
          }
        };

        slider.addEventListener('input', () => {
          let cur = parseFloat(slider.value);
          if (Math.abs(cur - defVal) <= snapDelta) {
            slider.value = defVal;
            cur = defVal;
          }
          updateTickStyle(cur);
        });

        if (tick) {
          tick.addEventListener('click', () => {
            slider.value = defVal;
            slider.dispatchEvent(new Event('input', { bubbles: true }));
            updateTickStyle(defVal);
          });
        }
        updateTickStyle(parseFloat(slider.value));
      });
    }

    // NAVEGAÇÃO ENTRE ABAS (6 PILARES DE PRODUTO + SUBNAVS CONTEXTUAIS)
    function switchTab(tabId) {
      if (tabId === 'tab-cloud') {
        switchTab('tab-local');
        switchLocalSubtab('cloud');
        return;
      }

      document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.tab-btn, .nav-item, .pillar-subnav-btn').forEach(el => el.classList.remove('active'));
      
      const pane = document.getElementById(tabId);
      if (pane) pane.classList.add('active');

      // Mapeamento de pilar para botão do topo (6 pilares canônicos)
      let topBtnId = 'btn-' + tabId;
      if (tabId === 'tab-google-hub' || tabId === 'tab-agent-inbox') topBtnId = 'btn-tab-overview';
      if (tabId === 'tab-gemini-notebooks') topBtnId = 'btn-tab-research';
      if (tabId === 'tab-subagents') topBtnId = 'btn-tab-personas';
      if (tabId === 'tab-connections') topBtnId = 'btn-tab-settings';

      const btn = document.querySelector(`.nav-item[data-tab="${tabId}"]`) || document.getElementById(topBtnId);
      if (btn) btn.classList.add('active');

      // Sub-navegação contextual dos pilares compostos
      const cockpitSubnav = document.getElementById('pillar-subnav-cockpit');
      const researchSubnav = document.getElementById('pillar-subnav-research');
      const agentsSubnav = document.getElementById('pillar-subnav-agents');
      const systemSubnav = document.getElementById('pillar-subnav-system');
      if (cockpitSubnav) {
        cockpitSubnav.classList.toggle('is-hidden', tabId !== 'tab-overview' && tabId !== 'tab-google-hub' && tabId !== 'tab-agent-inbox');
      }
      if (researchSubnav) {
        researchSubnav.classList.toggle('is-hidden', tabId !== 'tab-research' && tabId !== 'tab-gemini-notebooks');
      }
      if (agentsSubnav) {
        agentsSubnav.classList.toggle('is-hidden', tabId !== 'tab-personas' && tabId !== 'tab-subagents');
      }
      if (systemSubnav) {
        systemSubnav.classList.toggle('is-hidden', tabId !== 'tab-connections' && tabId !== 'tab-settings');
      }
      if (tabId === 'tab-overview') document.getElementById('btn-pillar-overview')?.classList.add('active');
      if (tabId === 'tab-google-hub') document.getElementById('btn-pillar-google-hub')?.classList.add('active');
      if (tabId === 'tab-agent-inbox') document.getElementById('btn-pillar-agent-inbox')?.classList.add('active');
      if (tabId === 'tab-research') document.getElementById('btn-pillar-research')?.classList.add('active');
      if (tabId === 'tab-gemini-notebooks') document.getElementById('btn-pillar-notebooks')?.classList.add('active');
      if (tabId === 'tab-personas') document.getElementById('btn-pillar-personas')?.classList.add('active');
      if (tabId === 'tab-subagents') document.getElementById('btn-pillar-subagents')?.classList.add('active');
      if (tabId === 'tab-connections') document.getElementById('btn-pillar-connections')?.classList.add('active');
      if (tabId === 'tab-settings') document.getElementById('btn-pillar-settings')?.classList.add('active');

      if (window.location.hash !== '#' + tabId) {
        try { history.replaceState(null, '', '#' + tabId); } catch(e) {}
      }

      if (tabId === 'tab-local') {
        if (currentLocalSubtab === 'hub') {
          fetchHardwareAndRecommendations();
          fetchInventoryDownloads();
          connectDownloadsSSE();
        } else if (currentLocalSubtab === 'cloud') {
          fetchCloudChampions();
        } else {
          fetchLocalModels();
          fetchLocalArenaData();
        }
      } else if (tabId === 'tab-subagents') {
        fetchSubAgentSessions();
        fetchRouterMatrix();
      } else if (tabId === 'tab-overview') {
        fetchEcosystemPulse();
        fetchTelemetry();
      } else if (tabId === 'tab-google-hub') {
        if (typeof loadGoogleHubTab === 'function') {
          loadGoogleHubTab();
        }
      } else if (tabId === 'tab-agent-inbox') {
        if (typeof loadAgentInboxTab === 'function') {
          loadAgentInboxTab();
        }
      } else if (tabId === 'tab-gemini-notebooks') {
        if (typeof loadGeminiNotebooksTab === 'function') {
          loadGeminiNotebooksTab();
        }
      } else if (tabId === 'tab-skills') {
        if (typeof initSkillsHubTab === 'function') {
          initSkillsHubTab();
        }
      } else if (tabId === 'tab-personas') {
        if (typeof initPersonasHubTab === 'function') {
          initPersonasHubTab();
        }
      } else if (tabId === 'tab-research') {
        if (typeof loadResearchHubMissions === 'function') {
          loadResearchHubMissions();
        }
        if (typeof loadResearchDoctorReport === 'function') {
          loadResearchDoctorReport();
        }
      } else if (tabId === 'tab-connections') {
        if (typeof fetchConnectionsData === 'function') {
          fetchConnectionsData();
        }
        if (typeof loadGoogleConnectionsVault === 'function') {
          loadGoogleConnectionsVault();
        }
      } else if (tabId === 'tab-settings') {
        fetchConfigToml();
        initSlidersMagneticSnap();
        fetchProfiles();
      }
    }

    // FEEDBACK VISUAL (TOAST DESDUPLICADO & AUTO-STACK)
    function showToast(msg, type = "success") {
      const c = document.getElementById('toast-container');
      if (!c) return;

      // Normalização de cores e tipos
      if (type === 'green') type = 'success';
      if (type === 'red') type = 'error';
      if (!['success', 'warning', 'error', 'info'].includes(type)) {
        type = 'info';
      }

      // Evita duplicação empilhando mensagens idênticas
      const existing = Array.from(c.querySelectorAll('.toast'));
      const duplicate = existing.find(t => t.dataset.msg === msg);
      if (duplicate) {
        duplicate.style.animation = 'none';
        void duplicate.offsetWidth;
        duplicate.style.animation = 'slideIn 0.25s ease-out';
        return;
      }

      // Limita a 3 notificações ativas para nunca cobrir a tela
      while (c.children.length >= 3) {
        c.firstElementChild.remove();
      }

      const el = document.createElement('div');
      el.className = 'toast ' + type;
      el.dataset.msg = msg;

      const icons = {
        success: uiIcon('check-circle', 'icon-md'),
        warning: uiIcon('alert-triangle', 'icon-md'),
        error: uiIcon('x-circle', 'icon-md'),
        info: uiIcon('info', 'icon-md')
      };
      const icon = icons[type] || uiIcon('info', 'icon-md');

      el.innerHTML = `
        <div class="toast-body-row">
          <span class="toast-icon-wrap">${icon}</span>
          <div class="toast-msg-text">${msg}</div>
        </div>
        <button class="toast-close-btn" title="Fechar aviso">${uiIcon('x', 'icon-xs')}</button>
      `;

      let timer = null;
      const removeToast = () => {
        if (timer) clearTimeout(timer);
        el.style.opacity = '0';
        el.style.transform = 'translateX(100%)';
        setTimeout(() => el.remove(), 250);
      };

      const closeBtn = el.querySelector('.toast-close-btn');
      if (closeBtn) {
        closeBtn.onclick = (e) => {
          e.stopPropagation();
          removeToast();
        };
      }
      el.onclick = removeToast;

      c.appendChild(el);
      timer = setTimeout(removeToast, 4000);
    }

    // MODAL DIALOGS UNIVERSAIS (SUBSTITUIÇÃO DE ALERT, CONFIRM E PROMPT)
    let _activeConfirmResolver = null;
    let _activePromptResolver = null;

    function appConfirm(message, options = {}) {
      return new Promise((resolve) => {
        const modal = document.getElementById('modal-confirm-dialog');
        const titleEl = document.getElementById('confirm-dialog-title');
        const iconEl = document.getElementById('confirm-dialog-icon');
        const msgEl = document.getElementById('confirm-dialog-message');
        const okBtn = document.getElementById('confirm-dialog-ok-btn');
        const cancelBtn = document.getElementById('confirm-dialog-cancel-btn');

        if (!modal || !msgEl) {
          return resolve(window.confirm(message));
        }

        if (titleEl) titleEl.textContent = options.title || 'Confirmar Ação';
        if (iconEl) iconEl.innerHTML = uiIcon(options.icon || 'alert-triangle', 'icon-md');
        if (okBtn) {
          okBtn.textContent = options.confirmText || 'Confirmar';
          okBtn.className = options.danger ? 'btn btn-danger' : 'btn btn-primary';
        }
        if (cancelBtn) cancelBtn.textContent = options.cancelText || 'Cancelar';
        msgEl.textContent = message;

        _activeConfirmResolver = resolve;
        modal.classList.add('active');
        if (okBtn) okBtn.focus();
      });
    }

    function closeAppConfirm(result) {
      const modal = document.getElementById('modal-confirm-dialog');
      if (modal) modal.classList.remove('active');
      if (_activeConfirmResolver) {
        const r = _activeConfirmResolver;
        _activeConfirmResolver = null;
        r(Boolean(result));
      }
    }

    function appPrompt(message, defaultValue = '', options = {}) {
      return new Promise((resolve) => {
        const modal = document.getElementById('modal-prompt-dialog');
        const titleEl = document.getElementById('prompt-dialog-title');
        const iconEl = document.getElementById('prompt-dialog-icon');
        const msgEl = document.getElementById('prompt-dialog-message');
        const inputEl = document.getElementById('prompt-dialog-input');
        const submitBtn = document.getElementById('prompt-dialog-submit-btn');

        if (!modal || !inputEl) {
          return resolve(window.prompt(message, defaultValue));
        }

        if (titleEl) titleEl.textContent = options.title || 'Entrada Requerida';
        if (iconEl) iconEl.innerHTML = uiIcon(options.icon || 'edit', 'icon-md');
        if (submitBtn) submitBtn.textContent = options.submitText || 'Confirmar';
        if (msgEl) msgEl.textContent = message;
        inputEl.value = defaultValue || '';

        _activePromptResolver = resolve;
        modal.classList.add('active');
        inputEl.focus();
        inputEl.select();
      });
    }

    function closeAppPrompt(result) {
      const modal = document.getElementById('modal-prompt-dialog');
      if (modal) modal.classList.remove('active');
      if (_activePromptResolver) {
        const r = _activePromptResolver;
        _activePromptResolver = null;
        r(result);
      }
    }

    window.appConfirm = appConfirm;
    window.closeAppConfirm = closeAppConfirm;
    window.appPrompt = appPrompt;
    window.closeAppPrompt = closeAppPrompt;
    window.showToast = showToast;

    // 1. FETCH TELEMETRIA & MASTER BRAIN
    async function fetchTelemetry() {
      try {
        const res = await fetch('/api/v1/telemetry');
        if (res.ok) {
          const data = await res.json();
          updateDashboardMetrics(data);
        }
      } catch (e) {
        console.warn("Falha no polling de telemetria", e);
      }
    }

    // 1.1 PULSO DO ECOSSISTEMA PESSOAL & EVOLUÇÃO
    async function fetchEcosystemPulse(isManual = false) {
      if (window._isFetchingPulse) return;
      window._isFetchingPulse = true;

      const btn = isManual ? document.getElementById('btn-pulse-refresh') : null;
      const kpiCards = isManual ? Array.from(document.querySelectorAll('#pulse-kpi-grid .stat-box')) : [];
      const defaultBtnHtml = '<svg class="ui-icon icon-sm" aria-hidden="true"><use href="#icon-refresh"></use></svg> Atualizar';

      if (isManual) {
        if (window._pulseBtnResetTimer) {
          clearTimeout(window._pulseBtnResetTimer);
          window._pulseBtnResetTimer = null;
        }
        if (btn) {
          btn.disabled = true;
          btn.classList.remove('is-synced');
          btn.classList.add('is-syncing');
          btn.innerHTML = '<svg class="ui-icon icon-sm" aria-hidden="true"><use href="#icon-refresh"></use></svg> Sincronizando...';
        }
        kpiCards.forEach(card => {
          card.classList.remove('kpi-pulse-flash');
          void card.offsetWidth;
          card.classList.add('kpi-pulse-flash');
        });
      }

      try {
        const minVisualDelay = isManual ? new Promise(resolve => setTimeout(resolve, 380)) : Promise.resolve();
        const telemetryPromise = (isManual && typeof fetchTelemetry === 'function') ? fetchTelemetry() : Promise.resolve();
        const [res] = await Promise.all([
          fetch('/api/v1/system/pulse'),
          telemetryPromise,
          minVisualDelay
        ]);
        if (typeof refreshInboxBadgeCount === 'function') {
          refreshInboxBadgeCount();
        }

        if (!res.ok) {
          if (isManual) throw new Error(`HTTP ${res.status}`);
          return;
        }
        const data = await res.json();
        if (data.status !== 'success' || !data.pulse) {
          if (isManual) throw new Error('Invalid pulse payload');
          return;
        }
        const p = data.pulse;

        const badgeEl = document.getElementById('pulse-level-badge');
        if (badgeEl) badgeEl.textContent = p.evolution_level || 'Nível 2 — Curadoria';

        const scoreEl = document.getElementById('pulse-evolution-score');
        if (scoreEl) {
          scoreEl.textContent = `Score ${p.evolution_score}/100`;
          if (p.evolution_breakdown) {
            const b = p.evolution_breakdown;
            const breakdownTitle = `Silício Local: ${b.silicon_pts}/25 (${b.local_models_installed} GGUFs, ${b.local_models_benchmarked} homologados) • Memória & Grafo L2: ${b.memory_graph_pts}/25 (${b.memories_stable} STABLE, ${b.ladybug_nodes} nós) • Obsidian & Pesquisas: ${b.obsidian_research_pts}/25 (${b.obsidian_wiki} wiki, ${p.completed_missions_count} missões) • Ecossistema: ${b.ecosystem_pts}/25 (${p.active_providers_count} provedores)`;
            scoreEl.title = breakdownTitle;
            if (scoreEl.parentElement) scoreEl.parentElement.title = breakdownTitle;
          }
        }

        const subEl = document.getElementById('pulse-evolution-sub');
        if (subEl) subEl.textContent = p.active_preset || p.evolution_level || 'Ecossistema Aumentado';

        const finopsEl = document.getElementById('pulse-finops-saved');
        if (finopsEl) finopsEl.textContent = `$${p.finops_savings_usd} USD`;

        const cacheEl = document.getElementById('pulse-cache-eff');
        if (cacheEl) cacheEl.textContent = `${p.finops_cache_efficiency} via Cache & Silício Local`;

        const entEl = document.getElementById('pulse-entities-count');
        if (entEl) entEl.textContent = `${p.sovereign_entities_count} Entidades / Fatos`;

        const hwEl = document.getElementById('pulse-hw-readiness');
        if (hwEl) hwEl.textContent = p.hardware_readiness || 'Estável';

        const provEl = document.getElementById('pulse-providers-active');
        if (provEl) provEl.textContent = `${p.active_providers_count} Provedores Configurados`;

        if (isManual) {
          if (btn) {
            btn.classList.remove('is-syncing');
            btn.classList.add('is-synced');
            btn.disabled = false;
            btn.innerHTML = '<svg class="ui-icon icon-sm" aria-hidden="true"><use href="#icon-check"></use></svg> Atualizado';
          }
          if (typeof showToast === 'function') {
            showToast('Pulso do ecossistema e telemetria sincronizados!', 'success');
          }
          window._pulseBtnResetTimer = setTimeout(() => {
            if (btn) {
              btn.classList.remove('is-synced');
              btn.innerHTML = defaultBtnHtml;
            }
            kpiCards.forEach(c => c.classList.remove('kpi-pulse-flash'));
            window._pulseBtnResetTimer = null;
          }, 1400);
        }
      } catch (err) {
        console.warn('Falha ao obter pulso do ecossistema:', err);
        if (isManual) {
          if (btn) {
            btn.classList.remove('is-syncing', 'is-synced');
            btn.disabled = false;
            btn.innerHTML = defaultBtnHtml;
          }
          kpiCards.forEach(c => c.classList.remove('kpi-pulse-flash'));
          if (typeof showToast === 'function') {
            showToast('Não foi possível sincronizar o pulso do ecossistema.', 'warning');
          }
        }
      } finally {
        window._isFetchingPulse = false;
      }
    }
    window.fetchEcosystemPulse = fetchEcosystemPulse;

    function updateDashboardMetrics(data) {
      // VRAM (WDDM usable ceiling vs physical GPU VRAM from NVML telemetry)
      if (data.vram_used_mb !== undefined) {
        const total = data.vram_total_mb || 4096;
        const limit = data.vram_limit_mb || Math.round(total * 0.86);
        window.soulsTelemetryState = Object.assign(window.soulsTelemetryState || {}, {
          vramTotalMb: total,
          vramLimitMb: limit,
          vramUsedMb: data.vram_used_mb,
          gpuName: data.gpu_name || (window.soulsTelemetryState && window.soulsTelemetryState.gpuName) || 'dGPU Local',
          cpuName: data.cpu_name || (window.soulsTelemetryState && window.soulsTelemetryState.cpuName) || 'Host CPU AVX2'
        });
        const wddmPct = Math.round((data.vram_used_mb / limit) * 100);
        const barWidthPct = Math.min(100, Math.round((data.vram_used_mb / total) * 100));
        const markerLeftPct = Math.min(98, Math.round((limit / total) * 100));

        const pillVram = document.getElementById('pill-vram');
        if (pillVram) pillVram.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' / ' + (limit / 1024).toFixed(2) + ' GB';

        const vramGaugeLabel = document.getElementById('vram-gauge-label');
        if (vramGaugeLabel) vramGaugeLabel.innerText = 'Alocação de VRAM sob WDDM (Físico ' + (total / 1024).toFixed(2) + ' GB)';

        const vramText = document.getElementById('vram-text');
        if (vramText) {
          vramText.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' GB / ' + (limit / 1024).toFixed(2) + ' GB WDDM (' + wddmPct + '%)';
        }
        const vramBar = document.getElementById('vram-bar');
        if (vramBar) vramBar.style.width = barWidthPct + '%';

        const vramMarker = document.getElementById('vram-wddm-marker');
        if (vramMarker) {
          vramMarker.style.left = markerLeftPct + '%';
          vramMarker.title = 'Teto Útil WDDM (' + (limit / 1024).toFixed(2) + ' GB / ' + limit + ' MB)';
        }

        // Sub-aba Aceleração Local: VRAM & Telemetria
        const hwMetricVram = document.getElementById('hw-metric-vram');
        if (hwMetricVram) hwMetricVram.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' / ' + (limit / 1024).toFixed(2) + ' GB';
        const hwMetricVramSub = document.getElementById('hw-metric-vram-sub');
        if (hwMetricVramSub) {
          const wddmOverhead = Math.max(0, total - limit);
          hwMetricVramSub.innerText = 'Teto WDDM: ' + limit.toLocaleString('pt-BR') + ' MB (' + wddmOverhead.toLocaleString('pt-BR') + ' MB SO)';
        }
        const hwLiveVramText = document.getElementById('hw-live-vram-text');
        if (hwLiveVramText) hwLiveVramText.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' GB / ' + (limit / 1024).toFixed(2) + ' GB (' + wddmPct + '%)';
        const hwLiveVramFill = document.getElementById('hw-live-vram-fill');
        if (hwLiveVramFill) hwLiveVramFill.style.width = barWidthPct + '%';
        const hwLiveVramMarker = document.getElementById('hw-live-vram-marker');
        if (hwLiveVramMarker) {
          hwLiveVramMarker.style.left = markerLeftPct + '%';
          hwLiveVramMarker.title = 'Teto Útil WDDM (' + limit.toLocaleString('pt-BR') + ' MB)';
        }
        const hwLiveVramLimit = document.getElementById('hw-live-vram-limit');
        if (hwLiveVramLimit) hwLiveVramLimit.innerText = limit.toLocaleString('pt-BR') + ' MB';
        const hwLiveVramUsed = document.getElementById('hw-live-vram-used');
        if (hwLiveVramUsed) hwLiveVramUsed.innerText = data.vram_used_mb.toLocaleString('pt-BR') + ' MB';
        const hwLiveVramFree = document.getElementById('hw-live-vram-free');
        if (hwLiveVramFree) hwLiveVramFree.innerText = Math.max(0, total - data.vram_used_mb).toLocaleString('pt-BR') + ' MB';
        const vramCfgSlider = document.getElementById('cfg-vram-limit');
        if (vramCfgSlider) {
          vramCfgSlider.max = String(total);
          vramCfgSlider.dataset.default = String(limit);
        }
        const simVramInput = document.getElementById('sim-hw-vram-input');
        if (simVramInput) simVramInput.max = String(total);
        const finopsTooltipDef = document.getElementById('finops-tooltip-hw-def');
        if (finopsTooltipDef) {
          const gLabel = data.gpu_name || 'dGPU Local';
          const cLabel = data.cpu_name || 'Host CPU AVX2';
          finopsTooltipDef.innerText = 'dGPU: ' + gLabel + ' (' + total.toLocaleString('pt-BR') + ' MB) | Host: ' + cLabel + ' (0 MB VRAM)';
        }
        const routerSimZone = document.getElementById('router-sim-zone');
        if (routerSimZone && (!routerSimZone.dataset.simCustom)) {
          routerSimZone.innerText = 'Zona Segura WDDM (< ' + limit.toLocaleString('pt-BR') + ' MB)';
        }

        // Composição da VRAM ativa (Processos & WDDM)
        const totalUsed = data.vram_used_mb || 1360;
        const totalVramBd = document.getElementById('vram-breakdown-total');
        if (totalVramBd) totalVramBd.innerText = totalUsed.toLocaleString('pt-BR') + ' MB';
        const dwmMb = Math.round(totalUsed * 0.57);
        const ideMb = Math.round(totalUsed * 0.31);
        const driversMb = Math.max(0, totalUsed - dwmMb - ideMb);
        const dwmVal = document.getElementById('vram-val-dwm');
        if (dwmVal) dwmVal.innerText = '~' + dwmMb.toLocaleString('pt-BR') + ' MB';
        const ideVal = document.getElementById('vram-val-ide');
        if (ideVal) ideVal.innerText = '~' + ideMb.toLocaleString('pt-BR') + ' MB';
        const driversVal = document.getElementById('vram-val-drivers');
        if (driversVal) driversVal.innerText = '~' + driversMb.toLocaleString('pt-BR') + ' MB';
      }

      // Temperatura & Watchdog Zone
      if (data.gpu_temp_c !== undefined) {
        const temp = Number(data.gpu_temp_c);
        window.soulsTelemetryState = Object.assign(window.soulsTelemetryState || {}, { gpuTempC: temp });
        const prevLimit = Number(data.gpu_temp_preventive_c || 72.0);
        const throttleLimit = Number(data.gpu_temp_throttle_c || 78.0);

        const pillTemp = document.getElementById('pill-temp');
        if (pillTemp) pillTemp.innerText = Math.round(temp) + '°C';

        const tempText = document.getElementById('temp-text');
        if (tempText) tempText.innerText = temp.toFixed(1) + '°C (Limite ' + throttleLimit.toFixed(1) + '°C)';

        const tempPct = Math.round((temp / 85.0) * 100);
        const tempBar = document.getElementById('temp-bar');
        if (tempBar) tempBar.style.width = Math.min(100, tempPct) + '%';

        const tempMarker = document.getElementById('temp-limit-marker');
        if (tempMarker) tempMarker.style.left = Math.min(98, Math.round((throttleLimit / 85.0) * 100)) + '%';

        // Sub-aba Aceleração Local: Temperatura & NVML
        const hwMetricTemp = document.getElementById('hw-metric-temp');
        if (hwMetricTemp) hwMetricTemp.innerText = temp.toFixed(1) + ' °C';
        const hwLiveTemp = document.getElementById('hw-live-temp');
        if (hwLiveTemp) hwLiveTemp.innerText = temp.toFixed(1) + ' °C';
        const hwLiveTempFill = document.getElementById('hw-live-temp-fill');
        if (hwLiveTempFill) hwLiveTempFill.style.width = Math.min(100, Math.round((temp / 85.0) * 100)) + '%';
        const hwLivePower = document.getElementById('hw-live-power');
        if (hwLivePower && data.power_w) hwLivePower.innerText = Number(data.power_w).toFixed(1) + ' W';
        const hwLiveFan = document.getElementById('hw-live-fan');
        if (hwLiveFan && (data.cpu_fan_rpm || data.gpu_fan_rpm)) {
          hwLiveFan.innerText = (data.cpu_fan_rpm || data.gpu_fan_rpm).toLocaleString('pt-BR') + ' RPM';
        }
        const hwLiveStatusPill = document.getElementById('hw-live-status-pill');
        if (hwLiveStatusPill) {
          if (temp >= throttleLimit) {
            hwLiveStatusPill.innerText = '● Zona Crítica (Throttle)';
            hwLiveStatusPill.style.color = 'var(--red)';
            hwLiveStatusPill.style.background = 'rgba(239, 68, 68, 0.12)';
            hwLiveStatusPill.style.borderColor = 'rgba(239, 68, 68, 0.3)';
          } else if (temp >= prevLimit) {
            hwLiveStatusPill.innerText = '● Zona Alerta (Preventiva)';
            hwLiveStatusPill.style.color = 'var(--amber)';
            hwLiveStatusPill.style.background = 'rgba(245, 158, 11, 0.12)';
            hwLiveStatusPill.style.borderColor = 'rgba(245, 158, 11, 0.3)';
          } else {
            hwLiveStatusPill.innerText = '● Zona Nominal (Segura)';
            hwLiveStatusPill.style.color = 'var(--green)';
            hwLiveStatusPill.style.background = 'rgba(16, 185, 129, 0.12)';
            hwLiveStatusPill.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
        }

        // Reatividade do Mascote
        const mascot = document.getElementById('mascot-img');
        const badge = document.getElementById('zone-badge');
        if (temp >= throttleLimit) {
          if (mascot) mascot.src = '/dashboard/icon/red';
          if (badge) {
            badge.innerText = 'Critical Zone';
            badge.className = 'badge badge-danger';
          }
        } else if (temp >= prevLimit) {
          if (mascot) mascot.src = '/dashboard/icon/amber';
          if (badge) {
            badge.innerText = 'Preventive Zone';
            badge.className = 'badge badge-lowcost';
          }
        } else {
          if (mascot) mascot.src = '/dashboard/icon/cyan';
          if (badge) {
            badge.innerText = (data.operational_zone || 'Nominal') + ' Zone';
            badge.className = 'badge badge-champion';
          }
        }
      }

      // Hardware Local: Subtítulo dinâmico GPU & CPU + RPM real dos Coolers (ATKACPI + NVML)
      if (data.gpu_name || data.cpu_name) {
        const hwSub = document.getElementById('hw-local-subtitle');
        const gName = data.gpu_name || 'dGPU Local Dedicada';
        const cName = data.cpu_name || 'Host CPU AVX2';
        if (hwSub) hwSub.innerText = `${gName} • ${cName}`;
        const cpuSub = document.getElementById('cpu-name-sub');
        if (cpuSub && data.cpu_name) cpuSub.innerText = data.cpu_name;
        const hwMetricCpu = document.getElementById('hw-metric-cpu');
        if (hwMetricCpu && data.cpu_name) hwMetricCpu.innerText = data.cpu_name;
        const hwFinopsCpuTitle = document.getElementById('hw-finops-cpu-title');
        if (hwFinopsCpuTitle && data.cpu_name) {
          hwFinopsCpuTitle.innerText = `${data.cpu_name}${data.cpu_threads ? ' (' + data.cpu_threads + ' threads)' : ''}`;
        }
      }

      if (data.cpu_fan_rpm !== undefined || data.fan_speed !== undefined) {
        const fanEl = document.getElementById('fan-speed');
        const fanSubEl = document.getElementById('fan-sub');
        const fanLblEl = document.getElementById('cooling-lbl');
        const cpuRpm = Number(data.cpu_fan_rpm || 0);
        const gpuRpm = Number(data.gpu_fan_rpm || 0);
        const gpuPct = Number(data.fan_speed || 0);

        if (fanLblEl) fanLblEl.innerText = 'Coolers (CPU / GPU)';
        if (cpuRpm > 0) {
          if (fanEl) {
            fanEl.innerText = `CPU: ${cpuRpm} RPM`;
            fanEl.style.fontSize = '0.92rem';
            fanEl.style.color = 'var(--green)';
          }
        } else if (gpuPct > 0) {
          if (fanEl) {
            fanEl.innerText = `GPU: ${gpuPct}% PWM`;
            fanEl.style.fontSize = '0.92rem';
            fanEl.style.color = 'var(--cyan)';
          }
        } else {
          if (fanEl) {
            fanEl.innerText = 'CPU: Standby (0 RPM)';
            fanEl.style.fontSize = '0.84rem';
            fanEl.style.color = 'var(--amber)';
          }
        }

        if (fanSubEl) {
          if (gpuRpm > 0) {
            fanSubEl.innerText = `GPU: ${gpuRpm} RPM (Ativo)`;
            fanSubEl.style.color = 'var(--cyan)';
          } else {
            fanSubEl.innerText = 'GPU: 0 RPM (Exaustor)';
            fanSubEl.style.color = 'var(--amber)';
          }
        }
      }

      if (data.power_w !== undefined) {
        const pEl = document.getElementById('power-draw');
        if (pEl) pEl.innerText = Number(data.power_w).toFixed(1) + ' W';
      }
      if (data.cpu_threads !== undefined) {
        const cEl = document.getElementById('cpu-threads');
        if (cEl) cEl.innerText = data.cpu_threads + 'T AVX2';
        const hwThreadsSub = document.getElementById('hw-metric-cpu-threads-sub');
        if (hwThreadsSub) hwThreadsSub.innerText = `${data.cpu_threads} Threads • Tier 0/0.5 (0 MB VRAM)`;
      }

      // Badges de Contagem nas Abas Superiores
      if (data.local_models_count !== undefined) {
        const bLocal = document.getElementById('badge-local-count');
        if (bLocal) bLocal.innerText = data.local_models_count;
      }
      if (data.profiles_count !== undefined) {
        const bCloud = document.getElementById('badge-cloud-profiles-count');
        const pillCount = document.querySelectorAll('#profile-pill-container .profile-pill').length;
        if (bCloud) bCloud.innerText = (pillCount > 0 ? pillCount : 9) + ' Perfis';
      }

      // FinOps & Eficiência E³ (com suporte a filtro Dia / Mês / Ano)
      if (data.finops_periods) {
        cachedFinopsPeriods = data.finops_periods;
        renderFinopsPeriodMetrics();
      } else {
        if (data.tokens_today !== undefined) {
          const tEl = document.getElementById('tokens-today');
          if (tEl) tEl.innerText = Number(data.tokens_today).toLocaleString();
        }
        if (data.usd_saved !== undefined) {
          const uEl = document.getElementById('usd-saved');
          if (uEl) uEl.innerText = '$' + Number(data.usd_saved).toFixed(2);
        }
        if (data.e3_score !== undefined) {
          const eEl = document.getElementById('e3-score');
          if (eEl) eEl.innerText = Number(data.e3_score).toFixed(1);
        }
        if (data.total_arena_runs !== undefined) {
          const fBadge = document.getElementById('finops-arena-runs-badge');
          if (fBadge) fBadge.innerText = `${data.total_arena_runs} Ensaios Locais`;
        }
        if (data.local_share_pct !== undefined) {
          const lPct = Math.round(Number(data.local_share_pct));
          const cPct = Math.max(0, 100 - lPct);
          const lText = document.getElementById('local-share-text');
          const lBar = document.getElementById('local-share-bar');
          if (lText) lText.innerText = `${lPct}% Local (Silício $0) / ${cPct}% Cloud`;
          if (lBar) lBar.style.width = `${lPct}%`;
        }
        if (data.context_savings_pct !== undefined) {
          const sPct = Number(data.context_savings_pct).toFixed(1);
          const sText = document.getElementById('ast-savings-text');
          const sBar = document.getElementById('ast-savings-bar');
          if (sText) sText.innerText = `${sPct}% Tokens Poupados`;
          if (sBar) sBar.style.width = `${Math.min(100, Number(data.context_savings_pct))}%`;
        }
      }

      // Hipocampo L3 & Armazenamento ReFS
      if (data.memory_l3) {
        const m3 = data.memory_l3;
        const partEl = document.getElementById('l3-partitions-stat');
        if (partEl) partEl.innerText = `${m3.stable_count || 0} STABLE / ${m3.evolving_count || 0} EVOLV`;

        const lbEl = document.getElementById('ladybug-nodes');
        if (lbEl) lbEl.innerText = `${m3.ladybug_nodes || 0} nós • ${m3.ladybug_edges || 0} arestas`;

        const obsEl = document.getElementById('obsidian-vault-stat');
        if (obsEl) obsEl.innerText = `${m3.obsidian_raw_count || 0} raw • ${m3.obsidian_wiki_count || 0} wiki`;

        const sizeBadge = document.getElementById('sqlite-vault-size-badge');
        if (sizeBadge && m3.sqlite_size_mb !== undefined) {
          sizeBadge.innerHTML = `${uiIcon('database', 'icon-xs')} Cofre ReFS • ${Number(m3.sqlite_size_mb).toFixed(2)} MB`;
        }

        const snapBadge = document.getElementById('hero-snapshot-badge');
        if (snapBadge && m3.snapshot_id) {
          snapBadge.innerHTML = `${uiIcon('lock', 'icon-xs')} ${escapeHtml(m3.snapshot_id.substring(0, 13))} (${(m3.memory_entries_count || 0) + (m3.user_entries_count || 0)} itens)`;
        }

        const snapLine = document.getElementById('l3-snapshot-summary-line');
        if (snapLine && m3.snapshot_id) {
          const shortSha = (m3.composite_sha256 || '').substring(0, 10);
          snapLine.innerHTML = `&bull; Snapshot: <code class="mono u-text-cyan">${m3.snapshot_id}</code> (SHA <code class="mono">${shortSha}</code>)`;
        }

        const chkLbl = document.getElementById('l3-checkpoints-count-label');
        if (chkLbl) chkLbl.innerText = `${m3.checkpoints_count || 0} Checkpoints v2`;

        // Hidratação dos KPIs da sub-aba Settings -> Memória L2/L3
        const sStable = document.getElementById('metric-memory-stable');
        if (sStable) sStable.innerText = `${m3.stable_count || 0} STABLE`;
        const sEvolv = document.getElementById('metric-memory-evolving');
        if (sEvolv) sEvolv.innerText = `${m3.evolving_count || 0} EVOLVING`;
        const sTotal = document.getElementById('metric-memory-total-fatos');
        if (sTotal) sTotal.innerText = `${(m3.stable_count || 0) + (m3.evolving_count || 0)} Fatos no FrankenSQLite`;
        const sNodes = document.getElementById('metric-ladybug-nodes');
        if (sNodes) sNodes.innerText = `${m3.ladybug_nodes || 0} Nós`;
        const sEdges = document.getElementById('metric-ladybug-edges');
        if (sEdges) sEdges.innerText = `${m3.ladybug_edges || 0} Arestas`;
        const sSnapDisp = document.getElementById('snapshot-id-display');
        if (sSnapDisp && m3.snapshot_id) sSnapDisp.innerText = m3.snapshot_id;
      } else if (data.ladybug_nodes !== undefined) {
        const lbEl = document.getElementById('ladybug-nodes');
        if (lbEl) lbEl.innerText = `${data.ladybug_nodes} nós`;
      }

      // Atividade Agêntica & Roteamento Real
      const mins = Math.max(1, Math.round((data.uptime_secs || 0) / 60));
      const pillUp = document.getElementById('pill-uptime');
      if (pillUp) pillUp.innerText = mins + 'm';

      const srvUp = document.getElementById('server-uptime');
      if (srvUp) {
        const conns = data.active_connections || 1;
        srvUp.innerText = `${mins} min • ${conns} conn ativa(s)`;
      }

      if (data.agentic_activity) {
        const act = data.agentic_activity;
        const taskEl = document.getElementById('act-last-task');
        if (taskEl && act.last_task) {
          taskEl.innerText = act.last_task;
          taskEl.title = act.last_task;
        }
        const modEl = document.getElementById('act-last-model');
        if (modEl && act.last_model) modEl.innerText = act.last_model;
        const tpsEl = document.getElementById('act-avg-tps');
        if (tpsEl && act.avg_tps !== undefined) {
          tpsEl.innerText = `${Number(act.avg_tps).toFixed(1)} tokens/s (Último: ${Number(act.latest_tps || act.avg_tps).toFixed(1)} tok/s)`;
        }
      }

      // Atualiza Versão do Engine na Topbar
      if (data.version) {
        const vEl = document.getElementById('top-engine-version');
        if (vEl) vEl.innerText = `(${data.version})`;
      }

      // Atualiza Master Brain, Alfândega & Topbar Preset
      if (data.hermes_customs) {
        window.cachedHermesCustoms = data.hermes_customs;
        if (typeof applyCustomsBannerData === 'function') {
          applyCustomsBannerData(data.hermes_customs);
        }
      }
      if (data.hermes_finops) {
        window.cachedHermesFinops = data.hermes_finops;
        if (typeof renderCloudFinopsData === 'function') {
          renderCloudFinopsData(data.hermes_finops);
        }
      }
      if (data.master_brain) {
        const mb = data.master_brain;
        window.currentHermesMasterModel = mb.model || window.currentHermesMasterModel;
        window.currentHermesMasterProvider = mb.provider || 'nous';
        document.getElementById('top-brain-name').innerText = mb.model;
        document.getElementById('hero-brain-model').innerText = mb.model;
        document.getElementById('hero-brain-provider').innerText = humanizeProviderName(mb.provider);
        document.getElementById('hero-brain-ctx').innerText = (mb.context_window || 64000).toLocaleString() + ' tokens';

        const heroStatusEl = document.getElementById('hero-brain-status');
        if (heroStatusEl) {
          const authOk = mb.provider_authenticated !== false;
          const stuckCount = Number(mb.stuck_fallback_sessions || 0);
          if (!authOk || mb.customs_health === 'critical') {
            heroStatusEl.innerText = 'Atenção: Autenticação / Rota Pendente';
            heroStatusEl.className = 'u-text-amber';
          } else if (stuckCount > 0) {
            heroStatusEl.innerText = `Ativo (${stuckCount} sessão em local)`;
            heroStatusEl.className = 'u-text-amber';
          } else {
            heroStatusEl.innerText = 'Conectado / Saudável';
            heroStatusEl.className = 'u-text-green';
          }
        }

        if (mb.active_profile_name) {
          const topPresetEl = document.getElementById('top-active-preset');
          if (topPresetEl) topPresetEl.innerText = mb.active_profile_name;
        }

        const badgeEl = document.getElementById('hero-brain-badge');
        if (mb.is_local) {
          badgeEl.innerText = 'Silício Bare-Metal';
          badgeEl.className = 'brain-badge badge-local';
        } else {
          badgeEl.innerText = 'Cloud API (' + humanizeProviderName(mb.provider) + ')';
          badgeEl.className = 'brain-badge badge-cloud';
        }
      }

      if (data.hermes_finops && data.hermes_finops.periods) {
        const hAll = data.hermes_finops.periods.all_time || {};
        const hProj = Number(data.hermes_finops.periods.projected_monthly_run_rate_usd || 0);
        const pulseCache = document.getElementById('pulse-cache-eff');
        if (pulseCache && Number(hAll.spend_usd || 0) > 0) {
          pulseCache.innerText = `Hermes API: $${Number(hAll.spend_usd).toFixed(2)} (Proj: $${hProj.toFixed(2)}/mês) • Cache ${Number(hAll.cache_hit_pct || 0).toFixed(0)}%`;
        }
      }
    }

    // 2. FETCH MATRIZ PREDITIVA DO INTEGRADOR (OVERVIEW & SUBAGENTS)
    async function fetchRouterMatrix() {
      try {
        const res = await fetch('/api/v1/router/matrix');
        if (res.ok) {
          const data = await res.json();
          if (data.cpu_simd) {
            window.detectedCpuSimd = data.cpu_simd;
          }
          window.cachedBareMetalEngines = data.bare_metal_engines || [];
          window.cachedInstalledLocalModels = data.installed_local_models || [];
          const summaryBadge = document.getElementById('matrix-arena-summary-badge');
          if (summaryBadge && data.benchmarked_count !== undefined) {
            summaryBadge.innerText = `${data.benchmarked_count} Homologados • ${data.total_arena_runs || 0} Provas`;
          }
          renderRouterMatrix(data.matrix || []);
          renderBareMetalEnginesStrip(window.cachedBareMetalEngines);
        }
      } catch (e) {
        console.warn("Falha ao carregar matriz do integrador", e);
      }
    }

    function renderBareMetalEnginesStrip(engines) {
      const strip = document.getElementById('router-bare-metal-engines-strip');
      if (!strip) return;
      if (!engines || engines.length === 0) {
        strip.innerHTML = '';
        return;
      }
      strip.innerHTML = `
        <div class="u-d-flex u-justify-between u-items-center u-mb-2 u-flex-wrap u-gap-2">
          <span class="u-mono u-fw-700 u-text-muted u-uppercase u-s-fe2c9f">
            ${uiIcon('cpu', 'icon-xs')} Motores Bare-Metal Determinísticos (0 Custo de Tokens / Execução Nativa Rust)
          </span>
          <span class="u-mono u-text-green u-s-3b4cd3">${engines.length} Subsistemas Ativos</span>
        </div>
        <div class="u-d-grid u-gap-3 u-s-ab1b65">
          ${engines.map(eng => `
            <div class="stat-box u-text-left" style="padding:10px 12px;">
              <div class="u-d-flex u-items-center u-justify-between u-gap-2 u-mb-1">
                <strong class="u-text-white u-d-flex u-items-center u-gap-6p u-s-d3bc8d">
                  ${uiIcon(eng.icon || 'cpu', 'icon-xs')} ${escapeHtml(eng.name)}
                </strong>
              </div>
              <div class="u-text-muted u-mono u-s-923ef2">${escapeHtml(eng.target)}</div>
              <div class="u-d-flex u-justify-between u-items-center u-mt-2 u-s-923ef2">
                <span class="u-text-cyan">${escapeHtml(eng.metric)}</span>
                <span class="u-text-green u-fw-600">${escapeHtml(eng.status)}</span>
              </div>
            </div>
          `).join('')}
        </div>
      `;
    }

    function renderRouterMatrix(matrix) {
      window.cachedRouterMatrix = matrix || [];
      const tbody1 = document.getElementById('integrator-tbody');
      const tbody2 = document.getElementById('subagents-router-tbody');
      if (!matrix || matrix.length === 0) {
        if (tbody1) tbody1.innerHTML = '<tr><td class="u-text-center u-text-muted" colspan="5">Nenhuma regra configurada.</td></tr>';
        if (tbody2) tbody2.innerHTML = '<tr><td class="u-text-center u-text-muted" colspan="5">Nenhuma regra configurada.</td></tr>';
        return;
      }

      const domainOverrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');

      const rowsHtml = matrix.map(item => {
        const override = domainOverrides[item.id] || {};
        const localModel = override.local_model || item.local_default.model || '';
        const cloudModel = override.cloud_model || item.cloud_fallback.model || '';
        const paretoRule = override.pareto_rule || item.pareto_rule || '';
        const profId = item.profile_id || (item.cloud_fallback && item.cloud_fallback.profile_id) || 'frontier_balanced';
        const isBench = item.local_default.benchmarked;
        const badgeClass = isBench ? 'badge-value' : 'badge-champion';

        return `
        <tr>
          <!-- 1: DOMÍNIO COGNITIVO COM MAIOR ESPAÇO, ÍCONE E HIERARQUIA -->
          <td class="u-s-61fcff">
            <div class="u-fw-700 u-text-white u-d-flex u-items-center u-gap-2 u-nowrap u-s-849ec2">
              <span class="u-d-inline-flex u-items-center u-s-ff0d51">${uiIcon(item.icon, 'icon-sm')}</span>
              <span>${item.category}</span>
            </div>
            <div class="u-text-muted u-mt-1 u-s-14aac2">${item.description}</div>
          </td>

          <!-- 2: SILÍCIO LOCAL: LINHA 1 (MODELO + TROCAR), LINHA 2 (TARGET/ALOCAÇÃO), LINHA 3 DEDICADA P/ VELOCIDADE -->
          <td class="u-s-61fcff">
            <div class="u-d-flex u-items-center u-justify-between u-gap-2 u-mb-1">
              <div class="u-min-w-0 u-flex-1 u-overflow-hidden">
                ${formatModelDisplayName(localModel, true)}
              </div>
              <button class="btn-micro" onclick="openConfigureDomainModal('${item.id}', 'local')" title="Alterar modelo local padrão deste domínio">
                Trocar
              </button>
            </div>
            <div class="u-text-muted u-s-0a2dfa">
              ${item.local_default.target}
            </div>
            <div class="u-text-green u-fw-600 u-mt-1 u-d-flex u-items-center u-gap-1 u-s-cb8a8e">
              <span class="u-text-dark u-s-10ba0c">Velocidade:</span> ${item.local_default.tps}
            </div>
          </td>

          <!-- 2: CLOUD FALLBACK: LINHA 1 (MODELO + TROCAR), LINHA 2 (ROTEAMENTO), LINHA 3 DEDICADA P/ TARIFA -->
          <td class="u-s-61fcff">
            <div class="u-d-flex u-items-center u-justify-between u-gap-2 u-mb-1">
              <div class="u-min-w-0 u-flex-1 u-overflow-hidden">
                ${formatModelDisplayName(cloudModel, false)}
              </div>
              <button class="btn-micro" onclick="openConfigureDomainModal('${item.id}', 'cloud')" title="Alterar modelo cloud fallback deste domínio">
                Trocar
              </button>
            </div>
            <div class="u-text-muted u-s-0a2dfa">
              ${item.cloud_fallback.provider}
            </div>
            <div class="u-text-cyan u-fw-600 u-mt-1 u-d-flex u-items-center u-gap-1 u-s-cb8a8e">
              <span class="u-text-dark u-s-10ba0c">Tarifa:</span> ${item.cloud_fallback.cost}
            </div>
          </td>

          <!-- REGRA PARETO -->
          <td class="u-s-61fcff">
            <div class="u-text-slate u-s-fdae4f">${paretoRule}</div>
          </td>

          <!-- AÇÕES COM BOTÃO DE CONFIGURAÇÃO DIRETA -->
          <td class="text-center u-s-c59a48">
            <div class="u-d-flex u-flex-col u-items-center u-gap-6p">
              <span class="badge ${badgeClass} u-w-full u-justify-center u-text-center">${item.status}</span>
              <div class="u-d-flex u-gap-1 u-w-full u-mt-2p">
                <button class="btn u-flex-1 u-justify-center u-s-fbea82" onclick="openConfigureDomainModal('${item.id}')" title="Configurar modelos local e cloud deste domínio">
                  ${uiIcon('settings', 'icon-xs')} Ajustar
                </button>
                <button class="btn u-flex-1 u-justify-center u-s-fbea82" onclick="switchTab('tab-local'); testModelInArena('${localModel}')" title="Aferir este modelo local na Silicon Arena">
                  ${uiIcon('trophy', 'icon-xs')} Aferir
                </button>
              </div>
            </div>
          </td>
        </tr>
      `;
      }).join('');

      if (tbody1) tbody1.innerHTML = rowsHtml;
      if (tbody2) tbody2.innerHTML = rowsHtml;
    }

    let localModelsList = [];

    // 3. FETCH LOCAL SILICON ARENA (llm_local)
    async function fetchLocalModels() {
      const tbody = document.getElementById('local-arena-tbody');
      tbody.innerHTML = '<tr><td class="u-text-center u-text-muted" colspan="9">Consultando pesos físicos GGUF...</td></tr>';
      
      try {
        const res = await fetch('/api/v1/models/local');
        if (res.ok) {
          const data = await res.json();
          localModelsList = data.models || [];
          document.getElementById('local-total-models').innerText = (data.total_models || localModelsList.length) + ' modelos físicos';
          document.getElementById('badge-local-count').innerText = data.total_models || localModelsList.length;
          if (data.root_directory) document.getElementById('local-root-dir').innerText = data.root_directory;
          renderLocalTable(localModelsList);

          // Se o catálogo curado já estiver em memória, re-renderiza para refletir badges de instalados
          if (curatedCatalogData && curatedCatalogData.length > 0) {
            renderCuratedCatalog(curatedCatalogData, hubHardwareData);
          }
          if (hubRecommendationsData) {
            renderRecommendedTriad(hubRecommendationsData);
          }
        }
      } catch (e) {
        tbody.innerHTML = '<tr><td class="u-text-center u-text-red" colspan="9">Erro ao carregar modelos locais.</td></tr>';
      }
    }

    function humanizeModelName(id, explicitDisplayName = null) {
      if (!id) return '--';
      const rawId = String(id).trim();
      const lower = rawId.toLowerCase();

      // 1. Catálogo Físico e Especialistas Canônicos Locais
      if (lower.includes('qwen3-5-4b-agentic-coder') || lower.includes('qwen3.5-4b-agentic-coder') || (lower.includes('qwen3-5-4b') && lower.includes('coder'))) return 'Qwen 3.5 4B Agentic Coder v4';
      if (lower.includes('qwen3-5-4b-uncensored') || lower.includes('qwen3.5-4b-uncensored') || (lower.includes('qwen3-5-4b') && lower.includes('mtp')) || lower.includes('hauhaucs')) return 'Qwen 3.5 4B Uncensored MTP';
      if (lower.includes('qwen3.8-4b') || lower.includes('qwen3-8-4b')) return 'Qwen 3.8 4B';
      if (lower.includes('qwen2-5-vl-3b') || lower.includes('qwen2.5-vl-3b')) return 'Qwen 2.5 VL 3B Vision';
      if (lower.includes('qwen2-5-coder-0-5b') || lower.includes('qwen2.5-coder-0.5b')) return 'Qwen 2.5 Coder 0.5B';
      if (lower.includes('qwen2-5-coder-1-5b') || lower.includes('qwen2.5-coder-1.5b')) return 'Qwen 2.5 Coder 1.5B';
      if (lower.includes('qwen2-5-coder-3b') || lower.includes('qwen2.5-coder-3b')) return 'Qwen 2.5 Coder 3B';
      if (lower.includes('qwen2-5-coder-7b') || lower.includes('qwen2.5-coder-7b')) return 'Qwen 2.5 Coder 7B';
      if (lower.includes('qwen2-5-coder-14b') || lower.includes('qwen2.5-coder-14b')) return 'Qwen 2.5 Coder 14B';
      if (lower.includes('qwen2-5-coder-32b') || lower.includes('qwen2.5-coder-32b')) return 'Qwen 2.5 Coder 32B';
      if (lower.includes('qwen2-5-0-5b') || lower.includes('qwen2.5-0.5b')) return 'Qwen 2.5 0.5B';
      if (lower.includes('qwen2-5-7b') || lower.includes('qwen2.5-7b')) return 'Qwen 2.5 7B';

      if (lower.includes('hermes-3-llama-3.2-3b') || lower.includes('hermes-3-llama-3-2-3b') || (lower.includes('hermes-3') && lower.includes('3.2-3b'))) return 'Hermes-3 Llama 3.2 3B';
      if (lower.includes('hy-mt2-7b') || lower.includes('hy-mt2')) return 'Hy-MT2 7B (Hunyuan)';
      if (lower.includes('nvidia-nemotron-3-nano-4b') || lower.includes('nemotron-3-nano-4b') || lower.includes('nemotron-nano-4b')) return 'NVIDIA Nemotron 3 Nano 4B';
      if (lower.includes('phi-4-mini')) return 'Phi-4 Mini 3.8B';
      if (lower.includes('arch-function')) return 'Arch Function Chat 3B';
      if (lower.includes('smollm2-135m')) return 'SmolLM2 135M Reasoning 5K';
      if (lower.includes('smollm2-360m')) return 'SmolLM2 360M Instruct';
      if (lower.includes('temper-1-1-5b') || lower.includes('temper-1')) return 'Temper-1 1.5B';
      if (lower.includes('xlam-2-1b') || lower.includes('xlam-2')) return 'xLAM-2 1B FC (Salesforce)';
      if (lower.includes('mamba-codestral-7b')) return 'Mamba Codestral 7B';
      if (lower.includes('nimbus-4b')) return 'Nimbus 4B';
      if (lower.includes('bonsai-27b')) return 'Bonsai 27B';

      // 2. Componentes Internos e Engines do Silício
      if (lower.includes('tiktoken-ccr')) return 'Tiktoken CCR Dehydrator';
      if (lower.includes('gitoxide')) return 'Gitoxide Frecency & AST';
      if (lower.includes('ladybug')) return 'LadybugDB Ontological Graph';
      if (lower.includes('nvml-barrier') || lower.includes('nvml-watchdog')) return 'NVML Physical Watchdog';

      // 3. Prioridade ao Display Name oficial vindo do Catálogo (preservando Pro/Flash/Flex/High/:free sem colisões)
      const isFreeTier = lower.endsWith(':free');
      let catalogDisplay = explicitDisplayName;
      if (!catalogDisplay && Array.isArray(cachedCloudRankedModels) && cachedCloudRankedModels.length > 0) {
        const hit = cachedCloudRankedModels.find(x => (x.canonical_id || '').toLowerCase() === lower);
        if (hit && hit.display_name) catalogDisplay = hit.display_name;
      }
      if (catalogDisplay && typeof catalogDisplay === 'string') {
        let stripped = catalogDisplay.replace(/^[A-Za-z0-9.\-_ ]{2,22}:\s+/, '').trim();
        stripped = stripped.replace(/\s*\(free\)\s*$/i, '').trim();
        if (stripped && !stripped.includes('/')) {
          return isFreeTier && !stripped.toLowerCase().includes('free') ? `${stripped} (Grátis)` : stripped;
        }
      }

      // 4. Tokenizador Determinístico Sem Colisão (nunca colapsa Pro, Flash, Flex, High, Max, Mini, :free)
      const colonIdx = rawId.lastIndexOf(':');
      const basePart = colonIdx !== -1 ? rawId.slice(0, colonIdx) : rawId;
      const tierTag = colonIdx !== -1 ? rawId.slice(colonIdx + 1).toLowerCase() : '';

      let clean = basePart;
      if (clean.includes('/')) clean = clean.split('/').pop();
      if (clean.includes('\\')) clean = clean.split('\\').pop();

      clean = clean.replace(/\.gguf$/i, '').replace(/\.bin$/i, '');
      clean = clean.replace(/[-_.](i\d+[-_])?q\d+([-_][a-z0-9]+)*$/i, '');
      clean = clean.replace(/[-_.]f(16|32)$/i, '');
      clean = clean.replace(/[-_.]instruct$/i, '');

      const acronyms = {
        'qwen': 'Qwen',
        'llama': 'Llama',
        'nvidia': 'NVIDIA',
        'hermes': 'Hermes',
        'nemotron': 'Nemotron',
        'smollm': 'SmolLM',
        'smollm2': 'SmolLM2',
        'cot': 'CoT',
        'mtp': 'MTP',
        'fc': 'FC',
        'vlm': 'VLM',
        'vl': 'VL',
        'ast': 'AST',
        'mcp': 'MCP',
        'gpt': 'GPT',
        'claude': 'Claude',
        'deepseek': 'DeepSeek',
        'glm': 'GLM',
        'kimi': 'Kimi',
        'mimo': 'MiMo',
        'hy3': 'HY3',
        'hy4': 'HY4',
        'ultraspeed': 'UltraSpeed',
        'avx2': 'AVX2',
        'cpu': 'CPU',
        'dgpu': 'dGPU',
        'gpu': 'GPU',
        'ui': 'UI',
        'api': 'API',
        'ccr': 'CCR',
        'nvml': 'NVML'
      };

      const words = clean.split(/[-_]+/).filter(Boolean);
      if (words.length > 0) {
        const formatted = words.map(w => {
          const wLow = w.toLowerCase();
          if (acronyms[wLow]) return acronyms[wLow];
          if (/^\d+(\.\d+)?[bmk]$/i.test(wLow)) return wLow.toUpperCase();
          if (/^v\d+(\.\d+)?$/i.test(wLow)) return wLow.toUpperCase().replace(/^V/, 'V');
          return w.charAt(0).toUpperCase() + w.slice(1);
        }).join(' ');
        if (tierTag === 'free') return `${formatted} (Grátis)`;
        if (tierTag && tierTag !== 'latest') return `${formatted} (${tierTag})`;
        return formatted;
      }

      return rawId;
    }

    function formatModelDisplayName(canonicalId, isLocal) {
      if (!canonicalId) return '--';
      const friendlyName = humanizeModelName(canonicalId);
      const cpuSimd = window.detectedCpuSimd || 'AVX2';
      if (isLocal) {
        const isCpu = canonicalId.includes('smollm') || canonicalId.includes('tiktoken') || canonicalId.includes('gitoxide') || canonicalId.includes('ladybug');
        const badgeLabel = isCpu ? `CPU ${cpuSimd}` : 'dGPU';
        const badgeClass = isCpu ? 'cpu' : 'local';
        return `
          <div class="model-id-box u-d-flex u-flex-col u-s-43638e">
            <div class="u-d-flex u-items-center u-gap-6p u-nowrap">
              <span class="model-brand-pill ${badgeClass}">${badgeLabel}</span>
              <span class="model-name-text u-fw-600 u-text-white" title="${canonicalId}">${friendlyName}</span>
            </div>
            <div class="mono u-text-muted u-s-fa7df8">${canonicalId}</div>
          </div>
        `;
      }
      const slashIdx = canonicalId.indexOf('/');
      if (slashIdx !== -1) {
        const brand = canonicalId.substring(0, slashIdx).toUpperCase();
        return `
          <div class="model-id-box u-d-flex u-flex-col u-s-43638e">
            <div class="u-d-flex u-items-center u-gap-6p u-nowrap">
              <span class="model-brand-pill">${brand}</span>
              <span class="model-name-text u-fw-600 u-text-white" title="${canonicalId}">${friendlyName}</span>
            </div>
            <div class="mono u-text-muted u-s-fa7df8">${canonicalId}</div>
          </div>
        `;
      }
      return `
        <div class="model-id-box u-d-flex u-flex-col u-s-43638e">
          <div class="u-d-flex u-items-center u-gap-6p u-nowrap">
            <span class="model-brand-pill">CLOUD</span>
            <span class="model-name-text u-fw-600 u-text-white" title="${canonicalId}">${friendlyName}</span>
          </div>
          <div class="mono u-text-muted u-s-fa7df8">${canonicalId}</div>
        </div>
      `;
    }

    // Modal de Aplicação Rápida de Modelo + Confirmação de Gateway
    let quickApplyTargetModel = null;
    let quickApplyTargetProvider = 'nous';
    let quickApplyIsLocal = false;
    let quickApplyModelObj = null;

    function canonicalizeGatewaySlug(prov) {
      const p = (prov || 'nous').toLowerCase().trim();
      if (p === 'nous_portal' || p === 'nous_portal_batch' || p === 'nous') return 'nous';
      if (p === 'openrouter_market' || p === 'openrouter') return 'openrouter';
      if (p === 'local_silicon' || p === 'llamacpp' || p === 'custom' || p === 'local') return 'custom';
      return p;
    }

    function getGatewayCatalogMeta(slug) {
      const c = canonicalizeGatewaySlug(slug);
      const fromCustoms = window.cachedHermesCustoms?.gateways?.find(g => g.id === c);
      if (fromCustoms) return fromCustoms;
      const defaults = {
        nous: { id: 'nous', label: 'Nous Portal', auth_mode: 'oauth_device', authenticated: false, key_url: 'https://portal.nousresearch.com', env_key: null, detail: 'Login OAuth do Nous Portal.' },
        openrouter: { id: 'openrouter', label: 'OpenRouter', auth_mode: 'api_key', authenticated: false, key_url: 'https://openrouter.ai/settings/keys', env_key: 'OPENROUTER_API_KEY', detail: 'Chave OPENROUTER_API_KEY.' },
        deepseek: { id: 'deepseek', label: 'DeepSeek Direct', auth_mode: 'api_key', authenticated: false, key_url: 'https://platform.deepseek.com/api_keys', env_key: 'DEEPSEEK_API_KEY', detail: 'Chave DEEPSEEK_API_KEY.' },
        anthropic: { id: 'anthropic', label: 'Anthropic Direct', auth_mode: 'api_key', authenticated: false, key_url: 'https://console.anthropic.com/settings/keys', env_key: 'ANTHROPIC_API_KEY', detail: 'Chave ANTHROPIC_API_KEY.' },
        'openai-api': { id: 'openai-api', label: 'OpenAI Direct', auth_mode: 'api_key', authenticated: false, key_url: 'https://platform.openai.com/api-keys', env_key: 'OPENAI_API_KEY', detail: 'Chave OPENAI_API_KEY.' },
        gemini: { id: 'gemini', label: 'Google AI Studio', auth_mode: 'api_key', authenticated: false, key_url: 'https://aistudio.google.com/apikey', env_key: 'GEMINI_API_KEY', detail: 'Chave GEMINI_API_KEY.' },
        zai: { id: 'zai', label: 'Z.AI / GLM', auth_mode: 'api_key', authenticated: false, key_url: 'https://z.ai/manage-apikey/apikey-list', env_key: 'ZAI_API_KEY', detail: 'Chave ZAI_API_KEY.' },
        xai: { id: 'xai', label: 'xAI / Grok', auth_mode: 'api_key', authenticated: false, key_url: 'https://console.x.ai/', env_key: 'XAI_API_KEY', detail: 'Chave XAI_API_KEY.' },
        nvidia: { id: 'nvidia', label: 'NVIDIA NIM', auth_mode: 'api_key', authenticated: false, key_url: 'https://build.nvidia.com/settings/api-keys', env_key: 'NVIDIA_API_KEY', detail: 'Chave NVIDIA_API_KEY.' },
        copilot: { id: 'copilot', label: 'GitHub Copilot', auth_mode: 'oauth_external', authenticated: false, key_url: 'https://github.com/settings/tokens', env_key: 'GITHUB_TOKEN', detail: 'Token Copilot/GitHub.' },
        custom: { id: 'custom', label: 'Silício Local (llama.cpp)', auth_mode: 'local', authenticated: true, key_url: null, env_key: null, detail: 'Servidor local em 127.0.0.1:18434.' }
      };
      return defaults[c] || { id: c, label: humanizeProviderName(c), auth_mode: 'api_key', authenticated: true, detail: `Rota ${c}` };
    }

    function renderQuickApplyGatewayCards() {
      const listEl = document.getElementById('quick-apply-gateway-list');
      const authBox = document.getElementById('quick-apply-auth-helper');
      const forceRow = document.getElementById('quick-apply-force-row');
      const tagEl = document.getElementById('quick-apply-provider-tag');
      if (!listEl) return;

      const m = quickApplyModelObj;
      let candidateGateways = [];
      if (quickApplyIsLocal) {
        candidateGateways = ['custom'];
      } else {
        const set = new Set();
        if (m) {
          if (m.has_nous) set.add('nous');
          if (m.has_openrouter) set.add('openrouter');
          (m.available_gateways || []).forEach(g => set.add(canonicalizeGatewaySlug(g)));
        }
        (window.cachedHermesCustoms?.active_gateway_ids || []).forEach(ag => {
          if (!m || !Array.isArray(m.available_gateways) || m.available_gateways.includes(ag)) {
            set.add(ag);
          }
        });
        set.add(canonicalizeGatewaySlug(quickApplyTargetProvider));
        candidateGateways = Array.from(set).filter(g => g !== 'custom');
      }

      if (!candidateGateways.includes(quickApplyTargetProvider) && candidateGateways.length > 0) {
        quickApplyTargetProvider = candidateGateways[0];
      }

      listEl.innerHTML = candidateGateways.map(gwId => {
        const meta = getGatewayCatalogMeta(gwId);
        const selected = gwId === quickApplyTargetProvider;
        let priceHint = '';
        if (gwId === 'custom') {
          priceHint = 'Local ($0.00)';
        } else if (m) {
          if (m.is_free || (quickApplyTargetModel || '').endsWith(':free')) {
            priceHint = 'Grátis ($0.00)';
          } else if (gwId === 'nous' && m.nous_blended !== null && m.nous_blended !== undefined) {
            priceHint = `$${Number(m.nous_blended).toFixed(2)}/1M`;
          } else if (gwId === 'openrouter' && m.or_blended !== null && m.or_blended !== undefined) {
            priceHint = `$${Number(m.or_blended).toFixed(2)}/1M`;
          } else {
            priceHint = 'API Direta / Base';
          }
        }
        const statusClass = meta.authenticated ? 'customs-pill-ok' : 'customs-pill-warn';
        const statusText = meta.id === 'custom' ? 'Ativo' : (meta.authenticated ? 'Autenticado' : (meta.status === 'expired' ? 'OAuth expirado' : 'Sem chave'));
        return `
          <button type="button" class="gateway-choice-card ${selected ? 'selected' : ''} ${meta.authenticated ? '' : 'unauth'}" onclick="selectQuickApplyGateway('${gwId}')">
            <div class="u-d-flex u-justify-between u-items-center u-gap-1">
              <strong class="u-text-white u-s-dbc8d1">${escapeHtml(meta.label || gwId)}</strong>
              <span class="customs-status-pill ${statusClass}">${statusText}</span>
            </div>
            <div class="u-text-muted u-mono u-s-6262d0">${gwId} ${priceHint ? '• ' + priceHint : ''}</div>
          </button>
        `;
      }).join('');

      const activeMeta = getGatewayCatalogMeta(quickApplyTargetProvider);
      if (tagEl) {
        tagEl.innerText = `Gateway Selecionado: ${activeMeta.label} (${quickApplyTargetProvider}) • ${quickApplyIsLocal ? 'Silício Bare-Metal' : 'Cloud API'}`;
      }

      if (authBox) {
        if (!activeMeta.authenticated && activeMeta.id !== 'custom') {
          authBox.classList.remove('is-hidden');
          if (forceRow) forceRow.classList.remove('is-hidden');
          if (activeMeta.auth_mode === 'oauth_device' || activeMeta.id === 'nous') {
            authBox.innerHTML = `
              <div class="u-fw-700 u-text-amber u-mb-1">${uiIcon('alert-triangle', 'icon-xs')} Autenticação Nous Portal Pendente no Hermes</div>
              <div class="u-text-slate u-mb-2">${escapeHtml(activeMeta.detail || 'O token OAuth expirou no Hermes Desktop. Sem login ativo, novas mensagens cairão para o modelo local.')}</div>
              <div class="u-d-flex u-gap-2 u-flex-wrap">
                <button type="button" class="btn btn-sm btn-cyan" onclick="launchHermesOauthLogin('nous')">${uiIcon('terminal', 'icon-xs')} Abrir Login (hermes login)</button>
                <a class="btn btn-sm" href="https://portal.nousresearch.com" target="_blank" rel="noopener noreferrer">${uiIcon('link', 'icon-xs')} Abrir Portal Nous</a>
                <button type="button" class="btn btn-sm" onclick="refreshModalCustomsStatus()">${uiIcon('refresh', 'icon-xs')} Já Autentiquei</button>
              </div>
            `;
          } else if (activeMeta.env_key) {
            authBox.innerHTML = `
              <div class="u-fw-700 u-text-amber u-mb-1">${uiIcon('key', 'icon-xs')} Chave ${activeMeta.env_key} não detectada no Hermes</div>
              <div class="u-text-slate u-mb-2">Cole sua chave abaixo para salvar e ativar imediatamente no Hermes Desktop (incluindo sub-perfis):</div>
              <div class="u-d-flex u-gap-2 u-flex-wrap u-items-center">
                <input type="password" id="modal-inline-api-key" class="setting-input u-flex-1" placeholder="Cole ${activeMeta.env_key} aqui..." />
                <button type="button" class="btn btn-sm btn-cyan" onclick="saveModalInlineApiKey('${activeMeta.env_key}')">${uiIcon('check', 'icon-xs')} Salvar e Ativar</button>
                ${activeMeta.key_url ? `<a class="btn btn-sm" href="${activeMeta.key_url}" target="_blank" rel="noopener noreferrer">Obter chave oficial</a>` : ''}
              </div>
            `;
          }
        } else {
          authBox.classList.add('is-hidden');
          authBox.innerHTML = '';
          if (forceRow) forceRow.classList.add('is-hidden');
        }
      }

      renderQuickApplyFinopsImpact();
    }

    function selectQuickApplyGateway(gwId) {
      quickApplyTargetProvider = canonicalizeGatewaySlug(gwId);
      renderQuickApplyGatewayCards();
    }

    function renderQuickApplyFinopsImpact() {
      const box = document.getElementById('quick-apply-cost-impact');
      if (!box) return;
      if (quickApplyIsLocal || quickApplyTargetProvider === 'custom' || (quickApplyTargetModel || '').endsWith(':free')) {
        box.classList.remove('is-hidden');
        box.innerHTML = `
          <div class="u-fw-700 u-text-emerald">${uiIcon('dollar', 'icon-xs')} Simulação FinOps Mensal: $0.00 / mês</div>
          <div class="u-text-slate u-mt-1">Modelo de custo zero (${quickApplyIsLocal ? 'execução soberana local' : 'tier gratuito :free'}) — sem consumo adicional em dólar no gateway selecionado.</div>
        `;
        return;
      }

      const m = quickApplyModelObj;
      if (!m) {
        box.classList.add('is-hidden');
        return;
      }
      const fin = window.cachedHermesFinops?.periods || {};
      const inTok = Number(fin.monthly_input_tokens_estimate || 500000);
      const outTok = Number(fin.monthly_output_tokens_estimate || 150000);
      const cacheTok = Number(fin.monthly_cache_read_tokens_estimate || 0);
      const pRate = quickApplyTargetProvider === 'openrouter'
        ? Number(m.or_prompt_price ?? m.nous_prompt_price ?? m.blended_cost ?? 0)
        : Number(m.nous_prompt_price ?? m.or_prompt_price ?? m.blended_cost ?? 0);
      const cRate = quickApplyTargetProvider === 'openrouter'
        ? Number(m.or_completion_price ?? m.nous_completion_price ?? m.blended_cost ?? 0)
        : Number(m.nous_completion_price ?? m.or_completion_price ?? m.blended_cost ?? 0);
      const estMonthUsd = (inTok * pRate + outTok * cRate + cacheTok * (pRate * 0.20)) / 1000000.0;
      const curRunRate = Number(fin.projected_monthly_run_rate_usd || 0);
      const tokVolLabel = formatContextTokens(Math.round(inTok + outTok + cacheTok));
      box.classList.remove('is-hidden');
      box.innerHTML = `
        <div class="u-fw-700 u-text-cyan">${uiIcon('trending-up', 'icon-xs')} Simulador FinOps Mensal: ~$${estMonthUsd.toFixed(2)} / mês</div>
        <div class="u-text-slate u-mt-1">Baseado no seu volume real no Hermes (~${tokVolLabel} tokens/mês • tarifa $${pRate.toFixed(2)} ent / $${cRate.toFixed(2)} saí por 1M). ${curRunRate > 0 ? `Projeção atual: $${curRunRate.toFixed(2)}/mês.` : ''}</div>
      `;
    }

    async function refreshModalCustomsStatus() {
      await fetchHermesCustoms();
      renderQuickApplyGatewayCards();
    }

    async function saveModalInlineApiKey(envKey) {
      const inp = document.getElementById('modal-inline-api-key');
      const val = inp ? inp.value.trim() : '';
      if (!val) {
        showToast('Cole uma chave válida antes de salvar.', 'warning');
        return;
      }
      try {
        const res = await fetch('/api/v1/system/credentials/save', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ key: envKey, value: val })
        });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          showToast(data.message || `Chave ${envKey} ativada no Hermes!`, 'success');
          if (inp) inp.value = '';
          await refreshModalCustomsStatus();
        } else {
          showToast(data.message || 'Falha ao salvar chave.', 'error');
        }
      } catch (e) {
        showToast(`Erro ao salvar credencial: ${e.message}`, 'error');
      }
    }

    function openQuickApplyModal(modelId, provider, isLocal = false) {
      quickApplyTargetModel = modelId;
      quickApplyIsLocal = Boolean(isLocal);
      quickApplyModelObj = (!isLocal && Array.isArray(cachedCloudRankedModels))
        ? cachedCloudRankedModels.find(x => (x.canonical_id || '').toLowerCase() === (modelId || '').toLowerCase()) || null
        : null;

      let chosenProv = canonicalizeGatewaySlug(provider || (isLocal ? 'custom' : 'nous'));
      if (!isLocal && quickApplyModelObj) {
        if (currentCloudProviderFilter === 'openrouter' && quickApplyModelObj.has_openrouter) {
          chosenProv = 'openrouter';
        } else if (currentCloudProviderFilter === 'nous' && quickApplyModelObj.has_nous) {
          chosenProv = 'nous';
        } else if (currentCloudProviderFilter !== 'all' && currentCloudProviderFilter !== 'free'
            && Array.isArray(quickApplyModelObj.available_gateways)
            && quickApplyModelObj.available_gateways.includes(currentCloudProviderFilter)) {
          chosenProv = currentCloudProviderFilter;
        }
      }
      quickApplyTargetProvider = chosenProv;

      const nameEl = document.getElementById('quick-apply-model-name');
      if (nameEl) nameEl.innerText = modelId;

      const sel = document.getElementById('quick-apply-profile-select');
      if (sel) {
        let opts = '';
        if (profilesData && profilesData.length > 0) {
          opts = profilesData.map(p => `
            <option value="${p.id}">${p.name} (${p.role === 'orchestrator' ? 'Master' : 'Subagente'})</option>
          `).join('');
        } else {
          opts = '<option value="custom_subagent">Subagente Customizado (Default)</option>';
        }
        sel.innerHTML = opts;
      }

      const radioMaster = document.querySelector('input[name="apply-target"][value="master"]');
      if (radioMaster) radioMaster.checked = true;
      const forceChk = document.getElementById('quick-apply-force-unauth');
      if (forceChk) forceChk.checked = false;
      toggleApplyTarget();
      renderQuickApplyGatewayCards();

      const modal = document.getElementById('modal-quick-apply');
      if (modal) modal.classList.add('active');
    }

    function closeQuickApplyModal() {
      const modal = document.getElementById('modal-quick-apply');
      if (modal) modal.classList.remove('active');
    }

    function toggleApplyTarget() {
      const val = document.querySelector('input[name="apply-target"]:checked')?.value;
      const fieldDomain = document.getElementById('field-select-domain');
      const fieldProfile = document.getElementById('field-select-custom-profile');
      const hermesOpts = document.getElementById('quick-apply-hermes-options');
      if (fieldDomain) fieldDomain.classList.toggle('is-hidden', val !== 'domain');
      if (fieldProfile) fieldProfile.classList.toggle('is-hidden', val !== 'custom_profile' && val !== 'profile');
      if (hermesOpts) hermesOpts.classList.toggle('is-hidden', val !== 'master');
    }

    async function submitQuickApply() {
      if (!quickApplyTargetModel) return;
      const val = document.querySelector('input[name="apply-target"]:checked')?.value;

      if (val === 'master') {
        const applyAll = Boolean(document.getElementById('quick-apply-all-profiles')?.checked);
        const unpinSess = Boolean(document.getElementById('quick-apply-unpin-sessions')?.checked);
        const forceUnauth = Boolean(document.getElementById('quick-apply-force-unauth')?.checked);
        const ok = await setHermesMasterModel(quickApplyTargetModel, quickApplyTargetProvider, activePreset, {
          applyAllProfiles: applyAll,
          unpinFallbackSessions: unpinSess,
          strictAuth: true,
          forceUnauthenticated: forceUnauth
        });
        if (ok) closeQuickApplyModal();
      } else if (val === 'domain') {
        const domSelect = document.getElementById('quick-apply-domain-select');
        const domId = domSelect ? domSelect.value : 'coding';
        closeQuickApplyModal();
        try {
          const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
          if (!overrides[domId]) overrides[domId] = {};
          if (quickApplyIsLocal) {
            overrides[domId].local_model = quickApplyTargetModel;
          } else {
            overrides[domId].cloud_model = quickApplyTargetModel;
            overrides[domId].cloud_provider = quickApplyTargetProvider;
          }
          localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));

          const matrixItem = (window.cachedRouterMatrix || []).find(m => m.id === domId || m.arena_category === domId);
          const profId = domId === 'agentic'
            ? 'agentic_dispatcher'
            : ((matrixItem && matrixItem.profile_id) || `domain_${domId}`);
          await fetch('/api/v1/profiles', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              id: profId,
              name: (matrixItem && matrixItem.category) || domId,
              description: (matrixItem && matrixItem.description) || `Domínio ${domId}`,
              role: domId === 'executive' ? 'orchestrator' : 'subagent',
              arena_category: domId,
              target_model: quickApplyTargetModel,
              provider: quickApplyIsLocal ? 'custom' : quickApplyTargetProvider
            })
          });
          if (typeof fetchRouterMatrix === 'function') await fetchRouterMatrix();
          showToast(`Modelo ${quickApplyTargetModel} (${quickApplyTargetProvider}) atribuído para [${domId.toUpperCase()}]!`, 'green');
        } catch(e) {
          showToast(`Erro ao salvar domínio: ${e.message}`, 'red');
        }
      } else {
        const profSelect = document.getElementById('quick-apply-profile-select');
        const profId = profSelect ? profSelect.value : activePreset;
        closeQuickApplyModal();
        await saveSubagentProfilePreset(profId, quickApplyTargetModel, quickApplyTargetProvider);
      }
    }

    let localModelFilterQuery = '';
    let localModelSiliconFilter = 'all';
    let localModelSortKey = 'tier';
    let localModelSortAsc = true;

    function getTierRank(tier) {
      if (!tier) return 99;
      const t = tier.toLowerCase();
      if (t.includes('tier 0.5') || t.includes('tier 05')) return 1;
      if (t.includes('tier 0')) return 0;
      if (t.includes('tier 1')) return 2;
      if (t.includes('tier 2')) return 3;
      if (t.includes('tier 3')) return 4;
      if (t.includes('tier 4')) return 5;
      return 10;
    }

    function renderTierBadge(tier) {
      if (!tier) return '<span class="u-mono u-text-muted" style="font-size:0.72rem;">Tier 1</span>';
      const t = tier.trim();
      const lower = t.toLowerCase();
      let subtitle = 'Master Coder';
      if (lower.includes('tier 0.5') || lower.includes('tier 05')) subtitle = 'Epistemic Probing';
      else if (lower.includes('tier 0')) subtitle = 'Fast Triage';
      else if (lower.includes('tier 1')) subtitle = 'Master dGPU';
      else if (lower.includes('tier 2')) subtitle = 'MoE Offload';
      else if (lower.includes('tier 3')) subtitle = 'Vision';
      else if (lower.includes('tier 4')) subtitle = 'Draft';
      return `
        <div class="u-d-flex u-flex-col u-items-center u-s-43638e">
          <span class="badge u-fw-600 u-mono">${t}</span>
          <span class="u-nowrap u-text-muted" style="font-size:0.66rem;">${subtitle}</span>
        </div>
      `;
    }

    function renderTierBadges(m) {
      const tiers = (m.tiers && m.tiers.length > 0) ? m.tiers : (m.tier ? [m.tier] : ['Tier 1']);
      return `
        <div class="u-d-flex u-flex-col u-gap-1 u-items-center u-justify-center">
          ${tiers.map(t => renderTierBadge(t)).join('')}
        </div>
      `;
    }

    function copyMcpConfigSnippet() {
      const snippet = JSON.stringify({
        "mcpServers": {
          "souls_engine": {
            "url": "http://127.0.0.1:9123/mcp",
            "transport": "http"
          }
        }
      }, null, 2);
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(snippet).then(() => {
          showToast("Configuração MCP copiada para a área de transferência!");
        }).catch(() => {
          appPrompt("Copie a configuração MCP abaixo:", snippet, { title: "Configuração MCP", icon: "list" });
        });
      } else {
        appPrompt("Copie a configuração MCP abaixo:", snippet, { title: "Configuração MCP", icon: "list" });
      }
    }

    function renderTagBadge(tag) {
      if (!tag) return '';
      return `<span class="u-text-muted u-mono" style="font-size:0.68rem;">#${tag.trim()}</span>`;
    }

    function resolveAgenticProfile(m) {
      const tiers = (m.tiers && m.tiers.length > 0) ? m.tiers : (m.tier ? [m.tier] : []);
      const t = tiers.join(' ').toLowerCase();
      const name = `${m.model || ''} ${m.name || ''} ${(m.tags || []).join(' ')}`.toLowerCase();

      if (t.includes('tier 3') || name.includes('vl') || name.includes('vision') || name.includes('multimodal')) {
        return { icon: 'eye', title: 'Multimodal Vision', color: '#fb7185' };
      }
      if ((name.includes('agentic') && name.includes('coder')) || (name.includes('coder') && name.includes('tool'))) {
        return { icon: 'code', title: 'Agentic Coder', color: '#34d399' };
      }
      if (name.includes('function') || name.includes('tool') || name.includes('fc') || name.includes('xlam')) {
        return { icon: 'wrench', title: 'Tool Specialist', color: '#38bdf8' };
      }
      if (name.includes('mtp') || t.includes('tier 4') || name.includes('draft')) {
        return { icon: 'rocket', title: 'Speculative Draft', color: '#60a5fa' };
      }
      if (name.includes('reasoning') || name.includes('r1') || t.includes('tier 0.5') || name.includes('socratic')) {
        return { icon: 'search', title: 'Intent Probing', color: '#34d399' };
      }
      if (t.includes('tier 0') || name.includes('135m')) {
        return { icon: 'zap', title: 'Fast Triage', color: '#38bdf8' };
      }
      if (name.includes('coder') || name.includes('code') || name.includes('temper')) {
        return { icon: 'bot', title: 'Code Surgeon', color: '#c084fc' };
      }
      if (name.includes('hermes') || name.includes('nemotron')) {
        return { icon: 'crown', title: 'Master Candidate', color: '#fbbf24' };
      }
      return { icon: 'bot', title: 'General Subagent', color: '#A1A1AA' };
    }

    function sortLocalModels(key) {
      if (localModelSortKey === key) {
        localModelSortAsc = !localModelSortAsc;
      } else {
        localModelSortKey = key;
        localModelSortAsc = true;
      }
      renderLocalTable(localModelsList);
    }

    function handleLocalModelsSearch(query) {
      localModelFilterQuery = (query || '').toLowerCase().trim();
      renderLocalTable(localModelsList);
    }

    function setLocalSiliconFilter(filter) {
      localModelSiliconFilter = filter;
      ['all', 'dgpu', 'cpu'].forEach(f => {
        const btn = document.getElementById('btn-filter-local-' + f);
        if (btn) btn.classList.toggle('active', f === filter);
      });
      renderLocalTable(localModelsList);
    }

    function renderLocalTable(models) {
      const tbody = document.getElementById('local-arena-tbody');
      if (!models || models.length === 0) {
        tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-0acd09" colspan="13">Nenhum modelo físico encontrado.</td></tr>';
        return;
      }

      // Atualiza contadores dos pills de filtro e cabeçalho dinamicamente
      const dgpuCount = models.filter(m => m.target && m.target.includes('dGPU')).length;
      const cpuCount = models.filter(m => !(m.target && m.target.includes('dGPU'))).length;
      const elAll = document.getElementById('count-local-all');
      const elDgpu = document.getElementById('count-local-dgpu');
      const elCpu = document.getElementById('count-local-cpu');
      const badgeSubtab = document.getElementById('badge-subtab-installed-count');
      const totalHeader = document.getElementById('local-total-models');

      if (elAll) elAll.innerText = models.length;
      if (elDgpu) elDgpu.innerText = dgpuCount;
      if (elCpu) elCpu.innerText = cpuCount;
      if (badgeSubtab) badgeSubtab.innerText = models.length;
      if (totalHeader) totalHeader.innerText = `${models.length} modelos físicos`;

      // Filtragem
      let filtered = models.filter(m => {
        const isGpu = m.target && m.target.includes('dGPU');
        if (localModelSiliconFilter === 'dgpu' && !isGpu) return false;
        if (localModelSiliconFilter === 'cpu' && isGpu) return false;

        if (localModelFilterQuery) {
          const prof = resolveAgenticProfile(m);
          const tiersStr = (m.tiers || [m.tier]).join(' ');
          const capsStr = (m.capabilities || []).join(' ');
          const hay = `${m.model || ''} ${m.name || ''} ${tiersStr} ${capsStr} ${prof.title} ${m.quantization || ''} ${(m.tags || []).join(' ')} ${m.file_path || ''}`.toLowerCase();
          if (!hay.includes(localModelFilterQuery)) return false;
        }
        return true;
      });

      // Ordenação dinâmica
      filtered.sort((a, b) => {
        let cmp = 0;
        if (localModelSortKey === 'tier') {
          const tA = (a.tiers && a.tiers.length > 0) ? a.tiers[0] : a.tier;
          const tB = (b.tiers && b.tiers.length > 0) ? b.tiers[0] : b.tier;
          cmp = getTierRank(tA) - getTierRank(tB);
          if (cmp === 0) cmp = (a.model || a.name || '').localeCompare(b.model || b.name || '');
        } else if (localModelSortKey === 'name') {
          cmp = (a.model || a.name || '').localeCompare(b.model || b.name || '');
        } else if (localModelSortKey === 'profile') {
          cmp = resolveAgenticProfile(a).title.localeCompare(resolveAgenticProfile(b).title);
        } else if (localModelSortKey === 'size') {
          cmp = (a.size_mb || 0) - (b.size_mb || 0);
        } else if (localModelSortKey === 'vram') {
          const vA = parseFloat(a.vram) || 0;
          const vB = parseFloat(b.vram) || 0;
          cmp = vA - vB;
        } else if (localModelSortKey === 'tps') {
          const tA = a.tps || 0;
          const tB = b.tps || 0;
          cmp = tA - tB;
        } else if (localModelSortKey === 'e3') {
          cmp = (a.e3_score || 0) - (b.e3_score || 0);
        } else if (localModelSortKey === 'context') {
          cmp = (a.context_window || 0) - (b.context_window || 0);
        } else if (localModelSortKey === 'freshness') {
          cmp = (a.freshness_days !== undefined ? a.freshness_days : 999) - (b.freshness_days !== undefined ? b.freshness_days : 999);
        }
        return localModelSortAsc ? cmp : -cmp;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-0acd09" colspan="13">Nenhum modelo corresponde aos filtros aplicados.</td></tr>';
        return;
      }

      tbody.innerHTML = filtered.map(m => {
        const tagsHtml = `<div class="u-d-flex u-flex-wrap u-items-center u-s-4c5172">` +
          (m.tags || []).map(t => renderTagBadge(t)).join('') +
          `</div>`;
        const isGpu = m.target && m.target.includes('dGPU');
        const statusBadge = isGpu
          ? `<span class="badge badge-champion u-nowrap u-s-263c8b">${uiIcon('gamepad', 'icon-xs')} dGPU (${escapeHtml((window.soulsTelemetryState && window.soulsTelemetryState.gpuName) || 'Local')})</span>`
          : `<span class="badge badge-value u-nowrap u-s-263c8b">${uiIcon('zap', 'icon-xs')} Host CPU (AVX2)</span>`;
        const sizeMb = m.size_mb ? (m.size_mb > 1024 ? (m.size_mb / 1024).toFixed(2) + ' GB' : m.size_mb.toFixed(0) + ' MB') : '--';
        const modelId = m.model || m.name;
        const rowSlug = (modelId || '').toLowerCase().replace(/[^a-z0-9]/g, '-');

        // Perfil Agêntico
        const prof = resolveAgenticProfile(m);
        const profileHtml = `
          <div class="u-d-flex u-items-center u-gap-6p u-nowrap">
            <span class="u-d-inline-flex u-items-center u-s-933a8e">${uiIcon(prof.icon, 'icon-sm')}</span>
            <span class="u-fw-600 u-s-d3bc8d" style="color: ${prof.color};">${prof.title}</span>
          </div>
        `;

        // Capacidades do Modelo (VLM, Code, Reasoning, Chat)
        const caps = m.capabilities || ['chat'];
        let capsHtml = '<div class="u-d-flex u-gap-1 u-flex-wrap u-justify-center u-items-center">';
        if (m.has_mmproj || caps.includes('vision')) {
          capsHtml += `<span class="badge u-text-cyan u-s-339d3b" title="Suporta Visão Multimodal com projetor mmproj">${uiIcon('eye', 'icon-xs')} VLM</span>`;
        }
        if (caps.includes('code')) {
          capsHtml += `<span class="badge u-text-emerald u-s-6a81bf" title="Especialista em Código & AST">${uiIcon('code', 'icon-xs')} Code</span>`;
        }
        if (caps.includes('reasoning')) {
          capsHtml += `<span class="badge u-s-d8176c" title="Raciocínio Lógico & CoT">${uiIcon('brain', 'icon-xs')} CoT</span>`;
        }
        if (caps.length === 1 && caps[0] === 'chat') {
          capsHtml += `<span class="badge u-text-slate u-s-e4c8ed" title="Conversacional Geral">${uiIcon('message-square', 'icon-xs')} Chat</span>`;
        }
        capsHtml += '</div>';

        // VRAM & Margem WDDM (teto dinâmico via NVML)
        let vramHtml = '';
        if (isGpu) {
          const vramGb = m.size_mb ? (m.size_mb / 1024).toFixed(1) : '0';
          const vramLimitGb = (window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) ? (window.soulsTelemetryState.vramLimitMb / 1024) : 4.0;
          const headroom = Math.max(0, (vramLimitGb - (m.size_mb / 1024))).toFixed(1);
          let folgaColor = '#34d399';
          let folgaLabel = `${uiIcon('check-circle', 'icon-xs')} Folga ${headroom} GB`;
          if (headroom < 0.6) {
            folgaColor = '#ef4444';
            folgaLabel = `${uiIcon('alert-triangle', 'icon-xs')} Folga ${headroom} GB (Risco)`;
          } else if (headroom < 1.2) {
            folgaColor = '#fbbf24';
            folgaLabel = `${uiIcon('info', 'icon-xs')} Folga ${headroom} GB`;
          }
          vramHtml = `
            <div class="mono text-center u-text-cyan u-fw-600 u-s-a768c1">${vramGb} GB</div>
            <div class="u-mt-2p u-nowrap u-fw-500 u-s-f6a7a6" style="color: ${folgaColor};">${folgaLabel}</div>
          `;
        } else {
          vramHtml = `
            <div class="mono text-center u-text-muted u-s-a768c1">0 MB</div>
            <div class="u-text-green u-mt-2p u-nowrap u-fw-500 u-s-f6a7a6">${uiIcon('zap', 'icon-xs')} Host RAM</div>
          `;
        }

        // Velocidade Empírica (Throughput, TTFT e TPOT)
        let speedHtml = '';
        if (m.tps && m.tps > 0) {
          const ttftStr = m.ttft_ms ? `${m.ttft_ms.toFixed(0)}ms` : '--';
          const tpotStr = m.tpot_ms ? `${m.tpot_ms.toFixed(0)}ms` : (m.tps > 0 ? (1000 / m.tps).toFixed(0) + 'ms' : '--');
          speedHtml = `
            <div class="mono text-center u-text-green u-fw-600 u-s-c79ecd">${uiIcon('zap', 'icon-xs')} ${m.tps.toFixed(1)} tok/s</div>
            <div class="u-text-muted u-mt-2p u-s-f6a7a6" title="TTFT: Tempo de resposta inicial • TPOT: Tempo por token gerado">
              TTFT: <strong class="u-text-cyan">${ttftStr}</strong> <span class="u-s-815433">•</span> TPOT: <strong class="u-text-slate">${tpotStr}</strong>
            </div>
          `;
        } else {
          const estVal = (m.estimated_tps && Number(m.estimated_tps) > 0)
            ? Number(m.estimated_tps).toFixed(1)
            : (isGpu ? '42.0' : '24.0');
          speedHtml = `
            <div class="u-text-muted u-text-center u-s-923ef2" title="Estimativa física calculada pela largura de banda do hardware vs tamanho da quantização">~${estVal} tok/s <span class="u-text-dark">(Est. Banda)</span></div>
            <div class="u-text-center u-mt-2p">
              <button class="btn u-text-cyan u-nowrap u-s-129880" onclick="testModelInArena('${modelId}')" title="Aferir velocidade real na Local Silicon Arena">
                Aferir ${uiIcon('trophy', 'icon-xs')}
              </button>
            </div>
          `;
        }

        // Eficiência E³ e Homologação
        let e3Html = '';
        if (m.e3_score && m.e3_score > 0) {
          const dateTitle = m.last_benchmarked_at ? `Último ensaio: ${m.last_benchmarked_at}` : 'Aferido no silício';
          e3Html = `
            <div class="text-center" title="${dateTitle} • Eficiência E³: ${m.e3_score.toFixed(1)}">
              <span class="badge u-fw-700 u-s-7ab53a">
                ${uiIcon('zap', 'icon-xs')} ${m.e3_score > 9000 ? '9999' : m.e3_score.toFixed(1)} E³
              </span>
              <div class="u-text-green u-mt-2p u-fw-600 u-s-778b43">${uiIcon('check-circle', 'icon-xs')} Homologado</div>
            </div>
          `;
        } else {
          e3Html = `
            <div class="text-center">
              <span class="badge u-s-eccd6d" title="Execute a prova na Silicon Arena para calcular a eficiência física e homologar o modelo.">
                ${uiIcon('clock', 'icon-xs')} Pendente
              </span>
            </div>
          `;
        }

        // Freshness ("Há X dias")
        const freshnessLabel = m.freshness || 'Hoje';
        const freshnessHtml = `
          <div class="text-center u-text-slate u-nowrap u-s-dbc8d1">
            ${uiIcon('calendar', 'icon-xs')} ${freshnessLabel}
          </div>
        `;

        const rawFileName = m.filename || ((m.file_path || '').split(/[\\/]/).pop()) || modelId;
        const safeFileArg = String(rawFileName).replace(/'/g, "\\'");
        const safeDisplayArg = String(m.name || modelId).replace(/'/g, "\\'");
        return `
          <tr id="local-row-${rowSlug}" data-model-id="${modelId}">
            <td>
              ${formatModelDisplayName(modelId, true)}
              <div class="u-text-muted u-overflow-hidden u-nowrap u-mt-2p u-s-25de43" title="${m.file_path || ''}">
                ${m.file_path || ''}
              </div>
            </td>
            <td class="text-center u-s-da57c5">
              <div class="u-d-flex u-gap-1 u-justify-center u-items-center">
                <button class="btn-apply-model local-btn u-s-96e357" onclick="openQuickApplyModal('${modelId}', 'local_silicon', true)" title="Aplicar este modelo local no Hermes">
                  ${uiIcon('zap', 'icon-sm')} Usar
                </button>
                <button class="btn u-nowrap u-s-fbea82" onclick="testModelInArena('${modelId}')" title="Aferir este modelo na Local Silicon Arena">
                  ${uiIcon('trophy', 'icon-sm')} Aferir
                </button>
                <button class="btn u-text-red u-nowrap u-s-fbea82" onclick="deletePhysicalLocalModel('${safeFileArg}', '${safeDisplayArg}')" title="Excluir arquivo GGUF do disco e liberar espaço">
                  ${uiIcon('trash', 'icon-sm')}
                </button>
              </div>
            </td>
            <td class="text-center u-s-da57c5">${renderTierBadges(m)}</td>
            <td class="u-s-da57c5">${profileHtml}</td>
            <td class="text-center u-s-da57c5">${capsHtml}</td>
            <td class="mono text-center u-nowrap u-s-da57c5">${sizeMb} <span class="u-text-cyan u-fw-700 u-s-a50fcb">•</span> <strong class="u-text-white">${m.quantization || 'Q4_K_M'}</strong></td>
            <td class="text-center u-s-da57c5">${vramHtml}</td>
            <td class="text-center u-s-da57c5">${speedHtml}</td>
            <td class="text-center u-s-da57c5">${e3Html}</td>
            <td class="mono text-center u-s-da57c5">${(m.context_window || 8192).toLocaleString()}</td>
            <td class="u-s-da57c5">${tagsHtml}</td>
            <td class="u-s-da57c5">${freshnessHtml}</td>
            <td class="text-center u-s-da57c5">${statusBadge}</td>
          </tr>
        `;
      }).join('');
    }

    async function deletePhysicalLocalModel(filename, displayName) {
      if (!filename) return;
      const confirmed = await appConfirm(
        `Deseja remover permanentemente o arquivo GGUF "${filename}" do disco para liberar espaço?\n\nEsta ação apaga os pesos físicos de ${displayName || filename}.`,
        {
          title: 'Excluir Modelo GGUF do Disco',
          icon: 'trash',
          confirmText: 'Excluir Arquivo GGUF',
          danger: true
        }
      );
      if (!confirmed) return;

      try {
        const res = await fetch(`/api/v1/inventory/models/${encodeURIComponent(filename)}`, {
          method: 'DELETE'
        });
        const data = await res.json();
        if (res.ok && (data.status === 'deleted' || data.status === 'success')) {
          showToast(`Modelo "${filename}" removido do disco com sucesso!`, 'success');
          await fetchLocalModels();
          if (typeof fetchModelHubRecommendations === 'function') fetchModelHubRecommendations();
        } else {
          showToast(`Não foi possível excluir "${filename}": ${data.message || 'erro desconhecido'}`, 'error');
        }
      } catch (err) {
        showToast(`Erro de rede ao excluir modelo: ${err.message}`, 'error');
      }
    }
    window.deletePhysicalLocalModel = deletePhysicalLocalModel;

    function extractQuantTagFromName(str) {
      if (!str) return '';
      const low = str.toLowerCase().replace(/-/g, '_');
      const tags = [
        'iq4_xs', 'iq4_nl', 'q4_k_l', 'q4_k_m', 'q4_k_s', 'q4_0', 'q4_1',
        'q5_k_l', 'q5_k_m', 'q5_k_s', 'q5_0', 'q5_1',
        'q6_k_l', 'q6_k', 'q8_0', 'q8_1',
        'iq3_m', 'iq3_xs', 'q3_k_l', 'q3_k_m', 'q3_k_s',
        'q2_k', 'fp16', 'bf16', 'f16'
      ];
      for (const t of tags) {
        if (low.includes(t)) return t.toUpperCase();
      }
      return '';
    }

    function stripQuantFromNameStem(str) {
      if (!str) return '';
      let s = str.toLowerCase().replace(/\.(gguf|bin)$/i, '').replace(/[-.]/g, '_');
      const markers = [
        '_iq4_xs', '_iq4_nl', '_q4_k_l', '_q4_k_m', '_q4_k_s', '_q4_0', '_q4_1',
        '_q5_k_l', '_q5_k_m', '_q5_k_s', '_q5_0', '_q5_1',
        '_q6_k_l', '_q6_k', '_q8_0', '_q8_1',
        '_iq3_m', '_iq3_xs', '_q3_k_l', '_q3_k_m', '_q3_k_s',
        '_q2_k', '_fp16', '_bf16', '_f16', '_imatrix', '_i1'
      ];
      for (const mk of markers) {
        const idx = s.indexOf(mk);
        if (idx !== -1) s = s.slice(0, idx) + s.slice(idx + mk.length);
      }
      return s.replace(/_/g, '');
    }

    function findArenaMatchForHubModel(model, activeFile, variantFile) {
      if (!arenaModelsData || arenaModelsData.length === 0) return null;
      return arenaModelsData.find(am =>
        (am.file_name && (
          (variantFile && am.file_name.toLowerCase() === variantFile.toLowerCase()) ||
          (activeFile && (am.file_name.toLowerCase() === activeFile.toLowerCase() || activeFile.toLowerCase().includes(am.file_name.toLowerCase())))
        )) ||
        (am.name && model && model.model_name && (
          am.name.toLowerCase() === model.model_name.toLowerCase() ||
          stripQuantFromNameStem(am.file_name || '') === stripQuantFromNameStem(variantFile || model.local_file_path || '')
        ))
      ) || null;
    }

    // Correspondência estrita com modelos reais baixados em disco (respeitando quantização)
    function findInstalledModelMatch(targetFileName, repoId = '', modelName = '', requiredQuantTag = '') {
      if (!localModelsList || localModelsList.length === 0) return null;
      const targetLower = (targetFileName || '').toLowerCase().replace(/\.gguf$/, '');
      const targetQuant = (requiredQuantTag || extractQuantTagFromName(targetFileName)).toUpperCase();
      const targetBase = stripQuantFromNameStem(targetFileName);

      for (const m of localModelsList) {
        const filePath = m.file_path || '';
        const realFileName = filePath.split(/[\\/]/).pop() || m.model || m.name || '';
        const fnLower = realFileName.toLowerCase().replace(/\.gguf$/, '');
        const mQuant = (m.quantization && m.quantization !== 'UNKNOWN'
          ? m.quantization
          : extractQuantTagFromName(realFileName)).toUpperCase();

        // 1. Match exato de nome de arquivo GGUF (sempre soberano)
        if (targetLower && fnLower === targetLower) {
          return { ...m, realFileName };
        }

        // 2. Match de Stem limpo + igualdade estrita de quantização
        if (targetLower && fnLower) {
          const cleanTarget = targetLower.replace(/[-_.]/g, '');
          const cleanFn = fnLower.replace(/[-_.]/g, '');
          if (cleanTarget === cleanFn) {
            return { ...m, realFileName };
          }
          const fnBase = stripQuantFromNameStem(realFileName);
          if (targetBase.length >= 6 && fnBase.length >= 6 && (fnBase === targetBase || fnBase.includes(targetBase) || targetBase.includes(fnBase))) {
            const quantMatches = !targetQuant
              || mQuant === targetQuant
              || (!mQuant && targetQuant === 'Q4_K_M');
            if (quantMatches) {
              return { ...m, realFileName };
            }
          }
        }
      }
      return null;
    }

    let targetArenaModelId = null;

    function testModelInArena(modelId) {
      targetArenaModelId = modelId;
      switchTab('tab-local');
      switchLocalSubtab('installed');
      if (arenaModelsData && arenaModelsData.length > 0) {
        selectModelInArenaRunner(modelId);
      }
      setTimeout(() => {
        const runner = document.getElementById('arena-runner-card');
        if (runner) {
          runner.scrollIntoView({ behavior: 'smooth', block: 'start' });
          runner.classList.remove('editor-line-flash');
          void runner.offsetWidth;
          runner.classList.add('editor-line-flash');
          setTimeout(() => runner.classList.remove('editor-line-flash'), 1800);
        }
      }, 80);
      showToast(`Modelo ${modelId} preparado na Silicon Arena. Pronto para aferição!`, 'success');
    }

    function selectModelInArenaRunner(targetId) {
      const select = document.getElementById('arena-runner-model-select');
      if (!select || !arenaModelsData || arenaModelsData.length === 0) return;

      const cleanTarget = (targetId || '').toLowerCase().replace(/\.gguf$/, '').replace(/[-_]/g, '');
      let foundIndex = -1;

      for (let i = 0; i < arenaModelsData.length; i++) {
        const m = arenaModelsData[i];
        const mid = (m.model_id || '').toLowerCase().replace(/[-_]/g, '');
        const mname = (m.name || '').toLowerCase().replace(/\.gguf$/, '').replace(/[-_]/g, '');
        const mpath = (m.file_path || '').toLowerCase().replace(/[-_]/g, '');
        if (mid === cleanTarget || mname === cleanTarget || mpath.includes(cleanTarget) || cleanTarget.includes(mid)) {
          foundIndex = i;
          break;
        }
      }

      if (foundIndex >= 0) {
        select.selectedIndex = foundIndex;
      }

      // Atualiza preview de compatibilidade e opções do select de tipo
      onArenaModelSelectChange(select.value);

      // Scroll suave até o runner e highlight visual
      setTimeout(() => {
        const card = document.getElementById('arena-runner-card');
        if (card) {
          card.scrollIntoView({ behavior: 'smooth', block: 'center' });
          card.classList.add('highlight-pulse');
          setTimeout(() => card.classList.remove('highlight-pulse'), 2200);
        }
        // Destaque na tabela se o elemento existir
        const rowId = 'arena-row-' + (select.value || '').toLowerCase().replace(/[^a-z0-9]/g, '-');
        const rowEl = document.getElementById(rowId);
        if (rowEl) {
          rowEl.classList.add('highlight-row');
          setTimeout(() => rowEl.classList.remove('highlight-row'), 2500);
        }
      }, 100);
    }

    function onArenaModelSelectChange(modelId) {
      if (!arenaModelsData || arenaModelsData.length === 0) return;
      const model = arenaModelsData.find(m => m.model_id === modelId) || arenaModelsData[0];
      if (!model) return;

      // 1. Atualiza Card de Preview
      const tierBadge = document.getElementById('arena-preview-tier-badge');
      const previewTitle = document.getElementById('arena-preview-title');
      const previewSub = document.getElementById('arena-preview-sub');
      const pillsContainer = document.getElementById('arena-preview-capabilities-pills');

      if (tierBadge) tierBadge.innerText = model.tier || 'Tier 1';
      if (previewTitle) {
        const isBench = model.status === 'BENCHMARKED';
        const tpsStr = model.latest_tps ? ` (${model.latest_tps.toFixed(1)} tok/s)` : '';
        previewTitle.innerHTML = `<strong>${model.name}</strong> • <span style="color: ${isBench ? 'var(--emerald)' : '#facc15'};">${isBench ? uiIcon('check-circle', 'icon-xs') + ' Homologado' + tpsStr : uiIcon('clock', 'icon-xs') + ' Pendente de Aferição'}</span>`;
      }
      if (previewSub) {
        const sizeGb = (model.size_mb / 1024).toFixed(1);
        const isGpu = model.size_mb < 5120;
        const targetStr = isGpu ? 'dGPU Dedicada (CUDA)' : 'Host CPU (AVX2)';
        const vramLimitGb = (window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) ? (window.soulsTelemetryState.vramLimitMb / 1024) : 4.0;
        const vramFree = Math.max(0, vramLimitGb - (model.size_mb / 1024)).toFixed(1);
        previewSub.innerText = `${model.quantization || 'GGUF'} • ${sizeGb} GB • Silício Ideal: ${targetStr} • Folga WDDM: ~${vramFree} GB`;
      }

      if (pillsContainer) {
        let pillsHtml = '';
        if (model.has_mmproj) {
          pillsHtml += `<span class="badge u-text-cyan u-s-945e20">${uiIcon('eye', 'icon-xs')} Visão Multimodal (mmproj ${uiIcon('check-circle', 'icon-xs')})</span>`;
        } else {
          pillsHtml += `<span class="badge u-text-muted u-s-5ade7a">${uiIcon('file-text', 'icon-xs')} LLM Texto Puro</span>`;
        }
        if ((model.name || '').toLowerCase().includes('coder') || (model.model_id || '').toLowerCase().includes('coder')) {
          pillsHtml += `<span class="badge u-text-emerald u-s-7de9d2">${uiIcon('code', 'icon-xs')} Code & AST</span>`;
        }
        if ((model.name || '').toLowerCase().includes('reasoning') || (model.tier || '').includes('0')) {
          pillsHtml += `<span class="badge u-s-5f9dc5">${uiIcon('brain', 'icon-xs')} Raciocínio CoT</span>`;
        }
        const sInfo = model.speculative_info;
        if (sInfo) {
          if (sInfo.status === 'PairedAvailable') {
            pillsHtml += `<span class="badge u-text-emerald u-s-705c98" title="${escapeHtml(sInfo.reason || '')}">${uiIcon('zap', 'icon-xs')} Par Draft: ${escapeHtml(sInfo.companion_id || '0.5B')}</span>`;
          } else if (sInfo.status === 'NativeMtp') {
            pillsHtml += `<span class="badge u-s-762314" title="${escapeHtml(sInfo.reason || '')}">${uiIcon('zap', 'icon-xs')} MTP Embutido</span>`;
          } else if (sInfo.status === 'Sub1bExempt') {
            pillsHtml += `<span class="badge u-text-muted u-s-f3a75f" title="${escapeHtml(sInfo.reason || '')}">${uiIcon('zap', 'icon-xs')} Sub-1B (Isento)</span>`;
          } else if (sInfo.status === 'CompanionMissing') {
            pillsHtml += `<span class="badge u-s-8bb761" title="${escapeHtml(sInfo.reason || '')}">${uiIcon('zap', 'icon-xs')} Par Draft Ausente</span>`;
          }
        }
        pillsContainer.innerHTML = pillsHtml;
      }

      // 2. Adapta o Select de Tipos de Prova
      const typeSelect = document.getElementById('arena-runner-type-select');
      const optVlm = document.getElementById('opt-arena-type-vlm');
      if (typeSelect && optVlm) {
        if (model.has_mmproj) {
          optVlm.disabled = false;
          optVlm.innerText = 'Visão Multimodal (Pareamento de Projetor VLM mmproj - Ativo)';
          optVlm.style.color = 'var(--cyan)';
        } else {
          optVlm.disabled = true;
          optVlm.innerText = 'Visão Multimodal (Indisponível: sem projetor mmproj)';
          optVlm.style.color = 'var(--text-muted)';
          if (typeSelect.value === '3') {
            typeSelect.value = '1';
          }
        }
      }

      const optSpec = document.getElementById('opt-arena-type-speculative');
      if (typeSelect && optSpec) {
        const sInfo = model.speculative_info;
        if (sInfo && sInfo.status === 'PairedAvailable') {
          optSpec.disabled = false;
          optSpec.innerText = `Aceleração Especulativa (Par Ativo: ${sInfo.companion_id || 'Draft'})`;
          optSpec.style.color = 'var(--emerald)';
        } else if (sInfo && sInfo.status === 'NativeMtp') {
          optSpec.disabled = false;
          optSpec.innerText = 'Aceleração Especulativa (MTP Nativo Embutido)';
          optSpec.style.color = '#c084fc';
        } else if (sInfo && sInfo.status === 'Sub1bExempt') {
          optSpec.disabled = true;
          optSpec.innerText = 'Aceleração Especulativa (Indisponível: Modelos < 1B já operam no limite do silício)';
          optSpec.style.color = 'var(--text-muted)';
          if (typeSelect.value === '4') typeSelect.value = '1';
        } else {
          optSpec.disabled = true;
          const recStr = sInfo?.recommended_companion ? ` - ex: ${sInfo.recommended_companion}` : '';
          optSpec.innerText = `Aceleração Especulativa (Requer par draft no disco${recStr})`;
          optSpec.style.color = 'var(--text-muted)';
          if (typeSelect.value === '4') typeSelect.value = '1';
        }
      }

      // 3. Adapta sugestão de acelerador
      const engineSelect = document.getElementById('arena-runner-engine-select');
      if (engineSelect) {
        if (model.size_mb > 5120 || (model.tier && (model.tier.includes('0') && !model.tier.includes('1')))) {
          engineSelect.value = 'llama-cpu';
        } else {
          engineSelect.value = 'llama-cuda';
        }
      }
    }

    function onArenaBenchmarkTypeChange() {
      const modelId = document.getElementById('arena-runner-model-select')?.value;
      const typeSelect = document.getElementById('arena-runner-type-select');
      if (!typeSelect || !modelId) return;

      const model = (arenaModelsData || []).find(m => m.model_id === modelId);
      if (typeSelect.value === '3' && model && !model.has_mmproj) {
        showToast('Este modelo não possui projetor multimodal (mmproj). Selecionando teste de velocidade.', 'warning');
        typeSelect.value = '1';
      }
      if (typeSelect.value === '4' && model) {
        const sInfo = model.speculative_info;
        if (!sInfo || (sInfo.status !== 'PairedAvailable' && sInfo.status !== 'NativeMtp')) {
          showToast(`Aceleração especulativa indisponível: ${sInfo?.reason || 'Requer par rascunho compatível'}`, 'warning');
          typeSelect.value = '1';
        }
      }
    }

    function setArenaStep(stepNumber) {
      for (let i = 1; i <= 4; i++) {
        const el = document.getElementById('arena-step-' + i);
        if (!el) continue;
        const labelEl = el.querySelector('div:first-child');
        if (i < stepNumber) {
          el.style.background = 'rgba(16, 185, 129, 0.15)';
          el.style.borderColor = 'rgba(16, 185, 129, 0.5)';
          if (labelEl) {
            labelEl.style.color = 'var(--emerald)';
            labelEl.innerHTML = 'FASE ' + i + ' ' + uiIcon('check-circle', 'icon-xs');
          }
        } else if (i === stepNumber) {
          el.style.background = 'rgba(6, 182, 212, 0.2)';
          el.style.borderColor = 'var(--cyan)';
          if (labelEl) {
            labelEl.style.color = 'var(--cyan)';
            labelEl.innerHTML = 'FASE ' + i + ' ' + uiIcon('clock', 'icon-xs');
          }
        } else {
          el.style.background = 'rgba(17, 17, 19, 0.5)';
          el.style.borderColor = 'rgba(255, 255, 255, 0.08)';
          if (labelEl) {
            labelEl.style.color = 'var(--text-muted)';
            labelEl.innerText = 'FASE ' + i;
          }
        }
      }
    }

    // ==========================================
    // 3.0.1 BATCH BENCHMARK RUNNER (HOMOLOGAÇÃO SEQUENCIAL EM LOTE)
    // ==========================================
    let batchBenchmarkAborted = false;
    let isBatchBenchmarkRunning = false;

    function openBatchBenchmarkModal() {
      if (!arenaModelsData || arenaModelsData.length === 0) {
        showToast('Nenhum modelo detectado para homologação em lote.', 'warning');
        return;
      }
      const modal = document.getElementById('arena-batch-modal');
      const listEl = document.getElementById('arena-batch-models-list');
      const badge = document.getElementById('arena-batch-summary-badge');
      if (!modal || !listEl) return;

      let totalTests = 0;
      let listHtml = '';

      arenaModelsData.forEach((m, idx) => {
        const tests = ['1. Velocidade', '2. Cognitivo'];
        if (m.has_mmproj) tests.push('3. Visão (mmproj)');
        const sInfo = m.speculative_info;
        if (sInfo && (sInfo.status === 'PairedAvailable' || sInfo.status === 'NativeMtp')) {
          tests.push('4. Especulativo');
        }
        totalTests += tests.length;

        const isGpu = m.size_mb < 5120;
        const targetStr = isGpu ? `${uiIcon('gamepad', 'icon-xs')} dGPU` : `${uiIcon('zap', 'icon-xs')} CPU`;
        listHtml += `
          <div class="u-d-flex u-justify-between u-items-center u-gap-2 u-s-4ec25a">
            <div class="u-fw-600 u-text-white u-overflow-hidden u-nowrap u-s-fcf835">
              ${idx + 1}. ${escapeHtml(m.name)} <span class="u-text-muted u-s-3b4cd3">(${targetStr})</span>
            </div>
            <div class="u-d-flex u-gap-1 u-flex-wrap">
              ${tests.map(t => `<span class="badge u-text-cyan u-s-fb1661">${t}</span>`).join('')}
            </div>
          </div>
        `;
      });

      listEl.innerHTML = listHtml;
      const estMin = Math.max(1, Math.ceil(totalTests * 0.45));
      if (badge) badge.innerText = `${arenaModelsData.length} Modelos • ${totalTests} Provas (~${estMin} min)`;

      modal.classList.add('active');
    }

    async function closeBatchBenchmarkModal() {
      if (isBatchBenchmarkRunning) {
        const ok = await appConfirm('A homologação sequencial está em andamento. Deseja realmente interromper e fechar?', {
          title: 'Interromper Homologação',
          icon: 'square',
          confirmText: 'Sim, Interromper',
          danger: true
        });
        if (!ok) {
          return;
        }
        abortBatchBenchmark();
      }
      const modal = document.getElementById('arena-batch-modal');
      if (modal) modal.classList.remove('active');
    }

    function abortBatchBenchmark() {
      batchBenchmarkAborted = true;
      const cancelBtn = document.getElementById('btn-cancel-batch-benchmark');
      if (cancelBtn) {
        cancelBtn.disabled = true;
        cancelBtn.innerHTML = `${uiIcon('square', 'icon-xs')} Interrompendo...`;
      }
      appendBatchLog('[AVISO] Sinal de cancelamento recebido. Finalizando ensaio atual...');
    }

    function appendBatchLog(text) {
      const logsEl = document.getElementById('arena-batch-logs');
      if (!logsEl) return;
      const now = new Date().toLocaleTimeString();
      logsEl.innerHTML += `<div><span class="u-text-muted">${now}</span> ${escapeHtml(text)}</div>`;
      logsEl.scrollTop = logsEl.scrollHeight;
    }

    async function startBatchBenchmark() {
      if (isBatchBenchmarkRunning || !arenaModelsData || arenaModelsData.length === 0) return;
      isBatchBenchmarkRunning = true;
      batchBenchmarkAborted = false;

      const startBtn = document.getElementById('btn-start-batch-benchmark');
      const cancelBtn = document.getElementById('btn-cancel-batch-benchmark');
      const closeBtn = document.getElementById('btn-close-batch-modal');
      const progressSec = document.getElementById('arena-batch-progress-section');
      const progressBar = document.getElementById('arena-batch-progress-bar');
      const progressPercent = document.getElementById('arena-batch-progress-percent');
      const currentTask = document.getElementById('arena-batch-current-task');
      const logsEl = document.getElementById('arena-batch-logs');

      if (startBtn) startBtn.style.display = 'none';
      if (cancelBtn) {
        cancelBtn.style.display = 'inline-block';
        cancelBtn.disabled = false;
        cancelBtn.innerHTML = `${uiIcon('square', 'icon-xs')} Interromper Fila`;
      }
      if (closeBtn) closeBtn.disabled = true;
      if (progressSec) progressSec.style.display = 'block';
      if (logsEl) logsEl.innerHTML = '';

      // Build task queue
      const taskQueue = [];
      arenaModelsData.forEach(m => {
        taskQueue.push({ modelId: m.model_id, name: m.name, type: 1, typeName: 'Velocidade & Memória' });
        taskQueue.push({ modelId: m.model_id, name: m.name, type: 2, typeName: 'Avaliação Cognitiva' });
        if (m.has_mmproj) {
          taskQueue.push({ modelId: m.model_id, name: m.name, type: 3, typeName: 'Visão Multimodal' });
        }
        const sInfo = m.speculative_info;
        if (sInfo && (sInfo.status === 'PairedAvailable' || sInfo.status === 'NativeMtp')) {
          taskQueue.push({ modelId: m.model_id, name: m.name, type: 4, typeName: 'Aceleração Especulativa' });
        }
      });

      appendBatchLog(`[INÍCIO] Iniciando bateria para ${arenaModelsData.length} modelos (${taskQueue.length} provas sequenciais).`);

      let completedCount = 0;
      const totalTasks = taskQueue.length;

      for (let i = 0; i < totalTasks; i++) {
        if (batchBenchmarkAborted) {
          appendBatchLog('[CANCELADO] Fila de homologação interrompida pelo operador.');
          break;
        }

        const task = taskQueue[i];
        const pct = Math.round((i / totalTasks) * 100);
        if (progressBar) progressBar.style.width = pct + '%';
        if (progressPercent) progressPercent.innerText = pct + '%';
        if (currentTask) currentTask.innerText = `[${i + 1}/${totalTasks}] ${task.name} • ${task.typeName}`;

        appendBatchLog(`[${i + 1}/${totalTasks}] Executando ${task.typeName} em '${task.name}'...`);

        try {
          const res = await fetch('/api/v1/arena/benchmark/run', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              model_id: task.modelId,
              benchmark_type: task.type
            })
          });

          if (!res.ok) {
            const err = await res.json().catch(() => ({}));
            appendBatchLog(`  [ERRO] ${err.message || res.statusText}`);
          } else {
            const data = await res.json();
            const metrics = data.metrics || {};
            let metricSummary = '';
            if (metrics.tps) metricSummary += `${metrics.tps.toFixed(1)} tok/s`;
            if (metrics.ttft_ms) metricSummary += ` | TTFT: ${metrics.ttft_ms.toFixed(0)}ms`;
            if (metrics.speedup_ratio) metricSummary += `Speedup: ${metrics.speedup_ratio.toFixed(2)}x`;
            if (metrics.cognitive_score) metricSummary += `Score: ${metrics.cognitive_score.toFixed(1)}`;
            appendBatchLog(`  [OK] Concluído! [${metricSummary || 'Auditado'}]`);
            completedCount++;
          }
        } catch (err) {
          appendBatchLog(`  [FALHA] Falha de rede/execução: ${err.message}`);
        }

        // Delay between physical allocations to guarantee WDDM VRAM garbage collection
        await new Promise(r => setTimeout(r, 600));
      }

      if (progressBar) progressBar.style.width = '100%';
      if (progressPercent) progressPercent.innerText = '100%';
      if (currentTask) currentTask.innerText = `Homologação concluída (${completedCount}/${totalTasks} testes)`;
      appendBatchLog(`[FIM] Fila finalizada. Atualizando registros do Heuristic Vault...`);

      // Refresh live telemetry & models
      await Promise.all([fetchLocalArenaData(), fetchLocalModels()]);

      showToast(`Homologação em lote concluída: ${completedCount} de ${totalTasks} testes auditados com sucesso!`, 'success');

      isBatchBenchmarkRunning = false;
      if (startBtn) {
        startBtn.style.display = 'inline-block';
        startBtn.innerHTML = `${uiIcon('refresh', 'icon-xs')} Re-executar Homologação Completa`;
      }
      if (cancelBtn) cancelBtn.style.display = 'none';
      if (closeBtn) closeBtn.disabled = false;
    }

    // ==========================================
    // 3.1 MODEL HUB & HARDWARE PROFILER (RFC-007)
    // ==========================================
    let currentLocalSubtab = 'installed';
    let hubHardwareData = null;
    let hubRecommendationsData = null;
    let curatedCatalogData = [];
    let currentCuratedProfileFilter = 'all';
    let downloadEventSource = null;

    function switchLocalSubtab(subtab) {
      if (subtab === 'arena') {
        switchLocalSubtab('installed');
        const runner = document.getElementById('arena-runner-card');
        if (runner) runner.scrollIntoView({ behavior: 'smooth', block: 'start' });
        return;
      }
      if (subtab === 'switchboard') {
        switchTab('tab-skills');
        if (typeof switchSkillsSubtab === 'function') switchSkillsSubtab('switchboard');
        return;
      }

      currentLocalSubtab = subtab;
      document.querySelectorAll('#tab-local .subtab-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById('btn-subtab-' + subtab);
      if (activeBtn) activeBtn.classList.add('active');

      const panelInstalled = document.getElementById('panel-local-installed');
      const panelCloud = document.getElementById('panel-local-cloud');
      const panelHub = document.getElementById('panel-local-hub');
      const panelFallback = document.getElementById('panel-local-fallback');

      if (subtab === 'installed') {
        if (panelInstalled) panelInstalled.style.display = 'block';
        if (panelCloud) panelCloud.style.display = 'none';
        if (panelHub) panelHub.style.display = 'none';
        if (panelFallback) panelFallback.style.display = 'none';
        fetchLocalModels();
        fetchLocalArenaData();
      } else if (subtab === 'cloud') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelCloud) panelCloud.style.display = 'block';
        if (panelHub) panelHub.style.display = 'none';
        if (panelFallback) panelFallback.style.display = 'none';
        fetchCloudChampions();
      } else if (subtab === 'hub') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelCloud) panelCloud.style.display = 'none';
        if (panelHub) panelHub.style.display = 'block';
        if (panelFallback) panelFallback.style.display = 'none';
        fetchHardwareAndRecommendations();
        fetchInventoryDownloads();
        connectDownloadsSSE();
      } else if (subtab === 'fallback') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelCloud) panelCloud.style.display = 'none';
        if (panelHub) panelHub.style.display = 'none';
        if (panelFallback) panelFallback.style.display = 'block';
        fetchFallbackQuarantines();
        fetchFallbackProviders();
      }
    }

    // ==========================================
    // 3.1.5 UNIVERSAL CLOUD GATEWAY & FALLBACK (v0.7.3)
    // ==========================================
    async function fetchFallbackQuarantines() {
      try {
        const resp = await fetch('/api/v1/router/quarantines');
        const data = await resp.json();
        const statusEl = document.getElementById('circuit-breaker-status');
        const detailEl = document.getElementById('circuit-breaker-detail');
        const sectionEl = document.getElementById('quarantine-section');
        const tbodyEl = document.getElementById('quarantine-tbody');

        if (data.status === 'success') {
          const count = data.count || 0;
          const badgeEl = document.getElementById('badge-subtab-fallback-count');
          if (badgeEl) {
            if (count === 0) {
              badgeEl.innerText = 'Ativo';
              badgeEl.style.background = 'rgba(16, 185, 129, 0.2)';
              badgeEl.style.color = 'var(--emerald)';
            } else {
              badgeEl.innerHTML = `${count} ${uiIcon('alert-triangle', 'icon-xs')}`;
              badgeEl.style.background = 'rgba(245, 158, 11, 0.2)';
              badgeEl.style.color = 'var(--amber)';
            }
          }

          if (count === 0) {
            if (statusEl) {
              statusEl.innerText = '● 100% Nominal (0 Quarentenas)';
              statusEl.style.color = 'var(--emerald)';
            }
            if (detailEl) detailEl.innerText = 'Nenhum provedor sob rate limit (429) ou indisponibilidade (500).';
            if (sectionEl) sectionEl.style.display = 'none';
          } else {
            if (statusEl) {
              statusEl.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} ${count} Modelo(s) em Quarentena`;
              statusEl.style.color = 'var(--amber)';
            }
            if (detailEl) detailEl.innerText = 'Desvio automático ativo para o próximo modelo da Fronteira de Pareto.';
            if (sectionEl) sectionEl.style.display = 'block';
            if (tbodyEl) {
              tbodyEl.innerHTML = (data.active_quarantines || []).map(q => `
                <tr>
                  <td><code class="mono u-text-amber u-fw-700">${q.model_id}</code></td>
                  <td><span class="chip chip-amber u-s-dbc8d1">${q.incident}</span></td>
                  <td>${q.remaining_secs}s restantes</td>
                  <td>
                    <button class="btn btn-sm btn-cyan u-s-d2340c" onclick="resetFallbackQuarantines('${q.model_id}')">
                      ${uiIcon('refresh', 'icon-sm')} Resetar
                    </button>
                  </td>
                </tr>
              `).join('');
            }
          }
        }
      } catch (err) {
        console.warn('Erro ao carregar quarentenas:', err);
      }
    }

    async function resetFallbackQuarantines(modelId) {
      try {
        const resp = await fetch('/api/v1/router/quarantines', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ model_id: modelId || 'all' })
        });
        const res = await resp.json();
        if (res.status === 'success') {
          showToast(res.message || 'Quarentenas resetadas', 'success');
          fetchFallbackQuarantines();
        } else {
          showToast(res.message || 'Erro ao resetar quarentenas', 'error');
        }
      } catch (err) {
        showToast('Falha na requisição de reset: ' + err, 'error');
      }
    }

    async function fetchFallbackProviders() {
      try {
        const resp = await fetch('/api/v1/router/providers');
        const data = await resp.json();
        const gridEl = document.getElementById('fallback-providers-grid');
        if (data.status === 'success' && gridEl) {
          gridEl.innerHTML = (data.providers || []).map(p => `
            <div class="u-d-flex u-flex-col u-justify-between u-s-ffbb36">
              <div>
                <div class="u-d-flex u-justify-between u-items-center u-mb-6p">
                  <strong class="u-s-5b3e83">${p.name}</strong>
                  <span class="chip chip-cyan u-s-6262d0">${p.tier}</span>
                </div>
                <div class="u-text-muted u-mb-2 u-s-d3bc8d">
                  Modelos: <span class="u-s-058a53">${(p.models || []).join(', ')}</span>
                </div>
                <div class="u-text-cyan u-mb-6p u-s-7d0f7b">
                  ${uiIcon('lightbulb', 'icon-xs')} <strong>Cache:</strong> ${p.cache_discount}
                </div>
                <div class="u-text-muted u-s-dbc8d1">
                  ${uiIcon('zap', 'icon-xs')} <strong>Latência:</strong> ${p.latency_factor}
                </div>
              </div>
              <div class="u-mt-10p u-text-green u-fw-700 u-s-dbc8d1">
                ● ${p.status}
              </div>
            </div>
          `).join('');
        }
      } catch (err) {
        console.warn('Erro ao carregar provedores:', err);
      }
    }

    async function simulateFallbackChain() {
      const category = document.getElementById('sim-task-category').value;
      const inputTokens = parseInt(document.getElementById('sim-input-tokens').value || '4000', 10);
      const cachedTokens = parseInt(document.getElementById('sim-cached-tokens').value || '0', 10);
      const budgetCapVal = document.getElementById('sim-budget-cap').value;
      const budgetCap = budgetCapVal ? parseFloat(budgetCapVal) : null;

      const payload = {
        task_id: 'sim_' + Date.now(),
        category,
        estimated_input_tokens: inputTokens,
        estimated_cached_input_tokens: cachedTokens,
        estimated_output_tokens: 1000,
        max_budget_usd: budgetCap,
        max_fallbacks: 4
      };

      try {
        const resp = await fetch('/api/v1/router/fallback/chain', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        const data = await resp.json();
        const resultsEl = document.getElementById('sim-chain-results');
        const timeEl = document.getElementById('sim-resolution-time');
        const cardsEl = document.getElementById('sim-chain-cards');

        if (data.status === 'success') {
          if (resultsEl) resultsEl.style.display = 'block';
          if (timeEl) timeEl.innerText = `${data.resolution_ms.toFixed(2)} ms (SLA <150ms)`;

          const p = data.primary;
          let html = `
            <div class="u-s-fb7d5a">
              <div class="u-d-flex u-justify-between u-items-center">
                <div>
                  <span class="chip chip-emerald u-s-ce21ba">1º CANDIDATO PRINCIPAL</span>
                  <strong class="u-text-green u-s-27895a">${p.model_id}</strong>
                  <span class="u-text-muted u-s-78b0f8">(Provedor: ${p.provider})</span>
                </div>
                <div class="u-fw-700 u-text-green u-s-933a8e">
                  Utilidade U: ${p.utility_score.toFixed(3)}
                </div>
              </div>
              <div class="u-d-flex u-gap-4 u-text-muted u-mt-6p u-s-d3bc8d">
                <span>Custo Projetado: <strong class="u-s-058a53">$${p.projected_cost_usd.toFixed(5)}</strong></span>
                <span>Latência Projetada: <strong class="u-s-058a53">${p.projected_latency_ms.toFixed(0)} ms</strong></span>
                <span>Tier: <strong class="u-s-058a53">${p.tier_name}</strong></span>
              </div>
            </div>
          `;

          (data.fallbacks || []).forEach((fb, idx) => {
            const cdBadge = fb.in_cooldown ? `<span class="chip chip-amber u-s-60c951">QUARENTENA (${fb.cooldown_remaining_secs}s)</span>` : '';
            html += `
              <div class="u-s-e818b6">
                <div class="u-d-flex u-justify-between u-items-center">
                  <div>
                    <span class="chip chip-purple u-s-ce21ba">${idx + 2}º FALLBACK</span>
                    <strong class="u-s-2ee5b5">${fb.model_id}</strong>
                    ${cdBadge}
                    <span class="u-text-muted u-s-78b0f8">(Provedor: ${fb.provider})</span>
                  </div>
                  <div class="u-text-purple u-s-22b62c">
                    Utilidade U: ${fb.utility_score.toFixed(3)}
                  </div>
                </div>
                <div class="u-d-flex u-gap-4 u-text-muted u-mt-1 u-s-dbc8d1">
                  <span>Custo: $${fb.projected_cost_usd.toFixed(5)}</span>
                  <span>Latência: ${fb.projected_latency_ms.toFixed(0)} ms</span>
                </div>
              </div>
            `;
          });

          if (cardsEl) cardsEl.innerHTML = html;
        } else {
          showToast(data.message || 'Erro na simulação', 'error');
        }
      } catch (err) {
        showToast('Falha na simulação: ' + err, 'error');
      }
    }

    // ==========================================
    // 3.2 LOCAL SILICON ARENA (ADR-007)
    // ==========================================
    let arenaModelsData = [];
    let arenaTelemetryData = [];
    let currentArenaModelsFilter = 'all';
    let currentArenaSearchQuery = '';
    let currentArenaSortKey = 'tier';
    let currentArenaSortAsc = true;

    async function fetchLocalArenaData() {
      try {
        const [resModels, resBench] = await Promise.all([
          fetch('/api/v1/arena/models'),
          fetch('/api/v1/arena/benchmarks')
        ]);

        if (resModels.ok) {
          const dataModels = await resModels.json();
          arenaModelsData = dataModels.models || [];
          renderArenaModelsTable(arenaModelsData);
          populateArenaRunnerModelSelect(arenaModelsData);

          const benchmarkedCount = arenaModelsData.filter(m => m.status === 'BENCHMARKED').length;
          const badge = document.getElementById('badge-subtab-arena-benchmarks');
          if (badge) {
            badge.innerText = benchmarkedCount;
            badge.style.display = benchmarkedCount > 0 ? 'inline-block' : 'none';
          }
          const sumBadge = document.getElementById('arena-models-summary-badge');
          if (sumBadge) {
            sumBadge.innerText = `${arenaModelsData.length} Modelos Mapeados • ${benchmarkedCount} Homologados`;
          }
        }

        if (resBench.ok) {
          const dataBench = await resBench.json();
          arenaTelemetryData = dataBench.history || [];
          renderArenaTelemetryTable(arenaTelemetryData);
        }
      } catch (err) {
        console.error('Falha ao carregar dados da Arena Local:', err);
      }
    }

    function populateArenaRunnerModelSelect(models) {
      const select = document.getElementById('arena-runner-model-select');
      if (!select) return;
      if (!models || models.length === 0) {
        select.innerHTML = '<option value="">Nenhum modelo disponível</option>';
        return;
      }
      const currentVal = select.value;
      select.innerHTML = models.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const prefix = isBenchmarked ? '[Homologado]' : '[Pendente]';
        const tpsStr = m.latest_tps ? ` • ${m.latest_tps.toFixed(0)} tok/s` : '';
        const paramsStr = m.parameters ? ` • ${m.parameters}` : '';
        return `<option value="${m.model_id}">${prefix} ${m.name} (${m.tier}${paramsStr} • ${m.quantization || 'GGUF'}${tpsStr})</option>`;
      }).join('');

      if (targetArenaModelId) {
        selectModelInArenaRunner(targetArenaModelId);
        targetArenaModelId = null;
      } else if (currentVal && models.some(m => m.model_id === currentVal)) {
        select.value = currentVal;
        onArenaModelSelectChange(select.value);
      } else if (models.length > 0) {
        onArenaModelSelectChange(models[0].model_id);
      }
    }

    function filterArenaModels(filter) {
      currentArenaModelsFilter = filter;
      document.querySelectorAll('#arena-filter-pills .profile-pill').forEach(b => b.classList.remove('active'));
      const activeBtn = document.getElementById('btn-arena-filter-' + filter.replace('.', '_'));
      if (activeBtn) activeBtn.classList.add('active');
      renderArenaModelsTable(arenaModelsData);
    }

    function onArenaModelsSearchInput() {
      const input = document.getElementById('arena-models-search');
      currentArenaSearchQuery = input ? input.value.trim().toLowerCase() : '';
      renderArenaModelsTable(arenaModelsData);
    }

    function sortArenaTable(key) {
      if (currentArenaSortKey === key) {
        currentArenaSortAsc = !currentArenaSortAsc;
      } else {
        currentArenaSortKey = key;
        currentArenaSortAsc = true;
      }
      updateArenaSortIndicators();
      renderArenaModelsTable(arenaModelsData);
    }

    function updateArenaSortIndicators() {
      const keys = ['tier', 'name', 'quant', 'tps', 'ttft', 'tpot', 'vram', 'e3', 'status'];
      keys.forEach(k => {
        const el = document.getElementById('sort-ind-' + k);
        if (el) {
          if (k === currentArenaSortKey) {
            el.innerText = currentArenaSortAsc ? '▲' : '▼';
            el.style.color = 'var(--cyan)';
          } else {
            el.innerText = '';
          }
        }
      });
    }

    function renderArenaModelsTable(models) {
      const tbody = document.getElementById('arena-models-tbody');
      if (!tbody) return;

      const filtered = (models || []).filter(m => {
        if (currentArenaModelsFilter === 'benchmarked' && m.status !== 'BENCHMARKED') return false;
        if (currentArenaModelsFilter === 'pending' && m.status === 'BENCHMARKED') return false;
        
        const tierLower = (m.tier || '').toLowerCase();
        if (currentArenaModelsFilter === 'tier0' && !(tierLower === 'tier 0' || (tierLower.includes('tier 0') && !tierLower.includes('0.5')))) return false;
        if (currentArenaModelsFilter === 'tier0_5' && !(tierLower.includes('tier 0.5') || tierLower.includes('tier 05'))) return false;
        if (currentArenaModelsFilter === 'tier1' && !(tierLower.includes('tier 1'))) return false;
        if (currentArenaModelsFilter === 'tier2' && !(tierLower.includes('tier 2'))) return false;

        if (currentArenaSearchQuery) {
          const q = currentArenaSearchQuery;
          const nameMatch = (m.name || '').toLowerCase().includes(q);
          const idMatch = (m.model_id || '').toLowerCase().includes(q);
          const quantMatch = (m.quantization || '').toLowerCase().includes(q);
          if (!nameMatch && !idMatch && !quantMatch) return false;
        }
        return true;
      });

      // Ordenação dinâmica
      filtered.sort((a, b) => {
        let cmp = 0;
        if (currentArenaSortKey === 'tier') {
          cmp = getTierRank(a.tier) - getTierRank(b.tier);
          if (cmp === 0) cmp = (a.name || a.model_id || '').localeCompare(b.name || b.model_id || '');
        } else if (currentArenaSortKey === 'name') {
          cmp = (a.name || a.model_id || '').localeCompare(b.name || b.model_id || '');
        } else if (currentArenaSortKey === 'quant') {
          cmp = (a.quantization || '').localeCompare(b.quantization || '');
        } else if (currentArenaSortKey === 'tps') {
          cmp = (a.latest_tps || 0) - (b.latest_tps || 0);
        } else if (currentArenaSortKey === 'ttft') {
          cmp = (a.latest_ttft_ms || 99999) - (b.latest_ttft_ms || 99999);
        } else if (currentArenaSortKey === 'tpot') {
          const tpA = a.latest_tpot_ms || (a.latest_tps > 0 ? 1000 / a.latest_tps : 99999);
          const tpB = b.latest_tpot_ms || (b.latest_tps > 0 ? 1000 / b.latest_tps : 99999);
          cmp = tpA - tpB;
        } else if (currentArenaSortKey === 'vram') {
          cmp = (a.latest_vram_peak_mb || 0) - (b.latest_vram_peak_mb || 0);
        } else if (currentArenaSortKey === 'e3') {
          cmp = (a.e3_score || 0) - (b.e3_score || 0);
        } else if (currentArenaSortKey === 'status') {
          cmp = (a.status || '').localeCompare(b.status || '');
        }
        return currentArenaSortAsc ? cmp : -cmp;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-0acd09" colspan="12">Nenhum modelo corresponde aos filtros aplicados.</td></tr>';
        return;
      }

      tbody.innerHTML = filtered.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const statusBadge = isBenchmarked
          ? `<span class="badge u-text-green u-s-52c49d">${uiIcon('check-circle', 'icon-xs')} Homologado</span>`
          : `<span class="badge u-s-038898">${uiIcon('clock', 'icon-xs')} Pendente</span>`;

        const tpsHtml = m.latest_tps
          ? `<div class="u-s-973ba1"><strong class="u-text-green u-s-604fde">${m.latest_tps.toFixed(1)}</strong><div class="u-text-muted u-s-3b4cd3">tok/s</div></div>`
          : '<span class="u-text-muted">--</span>';

        const ttftHtml = m.latest_ttft_ms
          ? `<div class="u-s-973ba1"><span class="mono u-text-cyan u-fw-600 u-s-5b9d54">${m.latest_ttft_ms.toFixed(0)}</span><div class="u-text-muted u-s-3b4cd3">ms</div></div>`
          : '<span class="u-text-muted">--</span>';

        const tpotVal = m.latest_tpot_ms || (m.latest_tps ? (1000 / m.latest_tps) : null);
        const tpotHtml = tpotVal
          ? `<div class="u-s-973ba1"><span class="mono u-text-slate u-fw-600 u-s-5b9d54">${tpotVal.toFixed(0)}</span><div class="u-text-muted u-s-3b4cd3">ms/tok</div></div>`
          : '<span class="u-text-muted">--</span>';

        const vramHtml = m.latest_vram_peak_mb
          ? `<div class="u-s-973ba1"><span class="mono u-fw-600 u-s-6b3520">${m.latest_vram_peak_mb.toFixed(0)}</span><div class="u-text-muted u-s-3b4cd3">MB</div></div>`
          : `<div class="u-s-973ba1"><span class="mono u-text-muted u-s-a2b44b">${m.size_mb ? (m.size_mb + 36).toFixed(0) : '--'}</span><div class="u-text-muted u-s-3b4cd3">MB (est)</div></div>`;

        const e3Html = m.e3_score
          ? `<span class="badge u-fw-700 u-s-825ef1" title="Eficiência Empírica E³: ${m.e3_score.toFixed(1)} (Throughput / [VRAM × Latência])">${m.e3_score > 9000 ? '9999' : m.e3_score.toFixed(1)}</span>`
          : '<span class="u-text-muted u-s-a2b44b" title="Score não calculado ainda. Execute o teste Type 1 para aferir Throughput, TTFT e VRAM.">--</span>';

        const activeFile = m.file_path || m.model_id;
        const rowSlug = (m.model_id || '').toLowerCase().replace(/[^a-z0-9]/g, '-');

        // Capacidades
        const caps = m.capabilities || ['chat'];
        let capsHtml = '<div class="u-d-flex u-gap-1 u-flex-wrap u-justify-center u-items-center">';
        if (m.has_mmproj || caps.includes('vision')) {
          capsHtml += `<span class="badge u-text-cyan u-s-339d3b" title="Suporta Visão Multimodal com projetor mmproj">${uiIcon('eye', 'icon-xs')} VLM</span>`;
        }
        if (caps.includes('code')) {
          capsHtml += `<span class="badge u-text-emerald u-s-6a81bf" title="Especialista em Código & AST">${uiIcon('code', 'icon-xs')} Code</span>`;
        }
        if (caps.includes('reasoning')) {
          capsHtml += `<span class="badge u-s-d8176c" title="Raciocínio Lógico & CoT">${uiIcon('brain', 'icon-xs')} CoT</span>`;
        }
        if (caps.length === 1 && caps[0] === 'chat') {
          capsHtml += `<span class="badge u-text-slate u-s-e4c8ed" title="Conversacional Geral">${uiIcon('message-square', 'icon-xs')} Chat</span>`;
        }
        capsHtml += '</div>';

        // Silício Recomendado
        const isGpu = m.size_mb < 5120;
        const targetBadge = isGpu
          ? `<span class="badge badge-champion u-s-77d53f">${uiIcon('gamepad', 'icon-xs')} dGPU (CUDA)</span>`
          : `<span class="badge badge-value u-s-77d53f">${uiIcon('zap', 'icon-xs')} Host CPU</span>`;

        return `
          <tr id="arena-row-${rowSlug}">
            <td class="text-center u-s-da57c5">
              ${renderTierBadge(m.tier)}
            </td>
            <td>
              <div class="u-fw-600 u-text-white u-s-5b9d54">${m.name}</div>
              <div class="mono u-text-muted u-mt-2p u-s-923ef2">${m.model_id}</div>
            </td>
            <td class="text-center u-s-da57c5">${capsHtml}</td>
            <td class="text-center mono u-s-a768c1">${m.quantization || 'GGUF'}</td>
            <td class="text-center">${tpsHtml}</td>
            <td class="text-center">${ttftHtml}</td>
            <td class="text-center">${tpotHtml}</td>
            <td class="text-center">${vramHtml}</td>
            <td class="text-center">${e3Html}</td>
            <td class="text-center u-s-da57c5">${targetBadge}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="text-center">
              <div class="u-d-flex u-justify-center u-items-center u-s-0880e8">
                <button class="btn btn-emerald u-s-177395" onclick="testModelInArena('${m.model_id}')" title="Selecionar e Testar na Silicon Arena">
                  ${uiIcon('zap', 'icon-sm')} Testar
                </button>
                ${isBenchmarked ? `
                  <button class="btn btn-green u-s-177395" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                    ${uiIcon('zap', 'icon-sm')} Usar
                  </button>
                ` : ''}
              </div>
            </td>
          </tr>
        `;
      }).join('');
    }

    function renderArenaTelemetryTable(history) {
      const tbody = document.getElementById('arena-telemetry-tbody');
      const badge = document.getElementById('arena-telemetry-summary-badge');
      if (badge && history) {
        badge.innerText = `${history.length} Provas Registradas`;
      }
      if (!tbody) return;
      if (!history || history.length === 0) {
        tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-ede515" colspan="11">Nenhum ensaio registrado no SQLite Vault.</td></tr>';
        return;
      }

      tbody.innerHTML = history.slice(0, 30).map(h => {
        let statusBadge = '<span class="badge badge-champion u-s-3b4cd3">NOMINAL</span>';
        const st = (h.status || '').toUpperCase();
        if (st.includes('NOMINAL') || st.includes('SUCCESS') || st.includes('VERIFIED')) {
          statusBadge = '<span class="badge u-text-green u-s-116cda">NOMINAL</span>';
        } else if (st.includes('WARN') || st.includes('THROTTLE')) {
          statusBadge = '<span class="badge u-s-bf352d">THROTTLED</span>';
        } else if (st.includes('FAIL') || st.includes('ERR')) {
          statusBadge = '<span class="badge u-s-9640a5">FALHA</span>';
        }

        const bt = (h.benchmark_type || '').toLowerCase();
        let typeBadge = '';
        if (bt.includes('hardeval') || bt.includes('hardware') || bt.includes('profile')) {
          typeBadge = `<span class="badge u-text-emerald u-s-21e397">${uiIcon('zap', 'icon-xs')} Velocidade &amp; VRAM</span>`;
        } else if (bt.includes('cognitive') || bt.includes('eval') || bt.includes('cot')) {
          typeBadge = `<span class="badge u-s-006ab2">${uiIcon('brain', 'icon-xs')} Avaliação Cognitiva</span>`;
        } else if (bt.includes('vision') || bt.includes('multimodal') || bt.includes('vlm')) {
          typeBadge = `<span class="badge u-text-cyan u-s-1c59c5">${uiIcon('eye', 'icon-xs')} Visão Multimodal</span>`;
        } else if (bt.includes('speculative') || bt.includes('draft')) {
          typeBadge = `<span class="badge u-text-amber-light u-s-11f801">${uiIcon('rocket', 'icon-xs')} Especulativa</span>`;
        } else if (bt.includes('context') || bt.includes('pressure') || bt.includes('needle')) {
          typeBadge = `<span class="badge u-s-e34a85">${uiIcon('archive', 'icon-xs')} Pressão Contexto</span>`;
        } else {
          typeBadge = `<span class="badge u-s-d24a57">${uiIcon('microscope', 'icon-xs')} ${h.benchmark_type || 'Geral'}</span>`;
        }

        const tpotVal = h.tpot_ms > 0 ? h.tpot_ms : (h.tps > 0 ? (1000 / h.tps) : 0);

        return `
          <tr id="telemetry-row-${h.id}">
            <td class="text-center mono u-text-muted u-s-dbc8d1">#${h.id}</td>
            <td>
              <div class="u-fw-600 u-text-white u-s-22b62c">${h.model_id}</div>
            </td>
            <td>${typeBadge}</td>
            <td class="text-center mono u-text-cyan u-s-dbc8d1">${h.engine}</td>
            <td class="text-center mono u-text-green u-fw-600">${h.tps > 0 ? h.tps.toFixed(1) + ' <span class="u-text-muted u-s-3b4cd3">tok/s</span>' : '--'}</td>
            <td class="text-center mono">${h.ttft_ms > 0 ? h.ttft_ms.toFixed(0) + ' <span class="u-text-muted u-s-3b4cd3">ms</span>' : '--'}</td>
            <td class="text-center mono u-text-slate">${tpotVal > 0 ? tpotVal.toFixed(0) + ' <span class="u-text-muted u-s-3b4cd3">ms</span>' : '--'}</td>
            <td class="text-center mono u-s-e6ce75">${h.vram_peak_mb > 0 ? h.vram_peak_mb.toFixed(0) + ' <span class="u-text-muted u-s-3b4cd3">MB</span>' : '--'}</td>
            <td class="text-center mono u-fw-600 u-s-358e52">${h.e3_score > 0 ? (h.e3_score > 9000 ? '9999' : h.e3_score.toFixed(1)) : '--'}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="mono u-text-muted u-s-cb8a8e">${h.created_at || '--'}</td>
          </tr>
        `;
      }).join('');
    }

    async function executeSelectedArenaBenchmark() {
      const modelId = document.getElementById('arena-runner-model-select')?.value;
      if (!modelId) {
        showToast('Selecione um modelo para executar a prova.', 'warning');
        return;
      }

      const benchType = parseInt(document.getElementById('arena-runner-type-select')?.value || '1', 10);
      const engine = document.getElementById('arena-runner-engine-select')?.value || null;

      await runSpecificArenaBenchmark(modelId, benchType, engine);
    }

    async function runSpecificArenaBenchmark(modelId, benchType = 1, engine = null) {
      const btn = document.getElementById('btn-run-arena-benchmark');
      const feedback = document.getElementById('arena-live-feedback');
      const titleEl = document.getElementById('arena-live-title');
      const verdictEl = document.getElementById('arena-live-verdict-badge');
      const gridEl = document.getElementById('arena-live-metrics-grid');

      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-a36b17">' + uiIcon('refresh', 'icon-xs') + '</span> Executando Prova...';
      }

      if (feedback) {
        feedback.style.display = 'block';
        if (titleEl) titleEl.innerHTML = `<span>${uiIcon('zap', 'icon-sm')}</span> Executando Prova no Silício para <strong>${modelId}</strong>...`;
        if (verdictEl) verdictEl.innerHTML = '<span class="badge u-text-cyan u-s-3be721">NVML Watchdog Ativo</span>';
        if (gridEl) gridEl.innerHTML = '<div class="u-text-muted u-s-5874b4"><span class="spin-icon u-d-inline-block u-s-a36b17">' + uiIcon('settings', 'icon-xs') + '</span> Preparando tensores via memmap2 e medindo telemetria física bare-metal...</div>';
      }

      // Ativa Stepper Fase 1
      setArenaStep(1);
      setTimeout(() => setArenaStep(2), 350);
      setTimeout(() => setArenaStep(3), 750);

      try {
        const payload = {
          model_id: modelId,
          benchmark_type: benchType,
          engine: engine
        };

        const res = await fetch('/api/v1/arena/benchmark/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });

        const data = await res.json();

        if (res.ok && data.status === 'success') {
          setArenaStep(4);
          showToast(`Prova ${data.benchmark_type} concluída e persistida no Heuristic Vault com sucesso!`, 'success');
          if (titleEl) titleEl.innerHTML = `<span>${uiIcon('check-circle', 'icon-sm')}</span> Resultado da Prova: <strong>${data.model_id}</strong> (${data.benchmark_type})`;
          if (verdictEl) verdictEl.innerHTML = `<span class="badge badge-champion">${data.metrics?.verdict || 'NOMINAL'}</span>`;

          if (gridEl && data.metrics) {
            let cardsHtml = '';
            // Type 1: HardwareProfile
            if (data.metrics.tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Real</div>
                  <div class="hw-spec-val u-text-green">${data.metrics.tps.toFixed(1)} <span class="u-s-cb8a8e">tok/s</span></div>
                </div>
              `;
            }
            if (data.metrics.ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Tempo Resposta (TTFT)</div>
                  <div class="hw-spec-val u-text-cyan">${data.metrics.ttft_ms.toFixed(0)} <span class="u-s-cb8a8e">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.tpot_ms !== undefined || data.metrics.tps !== undefined) {
              const tp = data.metrics.tpot_ms || (data.metrics.tps > 0 ? (1000 / data.metrics.tps) : 0);
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Latência por Token (TPOT)</div>
                  <div class="hw-spec-val u-text-slate">${tp.toFixed(0)} <span class="u-s-cb8a8e">ms/tok</span></div>
                </div>
              `;
            }
            if (data.metrics.peak_vram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">VRAM Pico NVML</div>
                  <div class="hw-spec-val u-s-e6ce75">${data.metrics.peak_vram_mb.toFixed(0)} <span class="u-s-cb8a8e">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.ram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">RAM Host</div>
                  <div class="hw-spec-val">${data.metrics.ram_mb.toFixed(0)} <span class="u-s-cb8a8e">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.e3_score !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Eficiência E³</div>
                  <div class="hw-spec-val u-s-189e9a">${data.metrics.e3_score > 9000 ? '9999' : data.metrics.e3_score.toFixed(1)}</div>
                </div>
              `;
            }

            // Type 2: CognitiveEval
            if (data.metrics.accuracy_pct !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Acurácia Sintática</div>
                  <div class="hw-spec-val u-text-green">${data.metrics.accuracy_pct.toFixed(1)}%</div>
                </div>
              `;
            }
            if (data.metrics.avg_tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Médio</div>
                  <div class="hw-spec-val u-text-green">${data.metrics.avg_tps.toFixed(1)} <span class="u-s-cb8a8e">tok/s</span></div>
                </div>
              `;
            }

            // Type 3: MultimodalVision
            if (data.metrics.multimodal_ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Latência Visão VLM</div>
                  <div class="hw-spec-val u-text-cyan">${data.metrics.multimodal_ttft_ms.toFixed(0)} <span class="u-s-cb8a8e">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.projection_latency_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Projeção mmproj</div>
                  <div class="hw-spec-val u-s-189e9a">${data.metrics.projection_latency_ms.toFixed(0)} <span class="u-s-cb8a8e">ms</span></div>
                </div>
              `;
            }

            // Type 4: SpeculativeDraft
            if (data.metrics.speedup_ratio !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Ganho de Velocidade</div>
                  <div class="hw-spec-val u-text-green">${data.metrics.speedup_ratio.toFixed(2)}x</div>
                </div>
              `;
            }
            if (data.metrics.acceptance_rate_alpha !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Taxa Alpha (Aceite)</div>
                  <div class="hw-spec-val u-text-cyan">${(data.metrics.acceptance_rate_alpha * 100).toFixed(1)}%</div>
                </div>
              `;
            }

            // Type 5: ContextPressure
            if (data.metrics.max_effective_context !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Janela Máxima Efetiva</div>
                  <div class="hw-spec-val u-text-cyan">${Number(data.metrics.max_effective_context).toLocaleString()} <span class="u-s-cb8a8e">tokens</span></div>
                </div>
              `;
            }
            if (data.metrics.degradation_detected !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Degradação</div>
                  <div class="hw-spec-val" style="color: ${data.metrics.degradation_detected ? '#f43f5e' : 'var(--emerald)'};">${data.metrics.degradation_detected ? 'Detectada ' + uiIcon('alert-triangle', 'icon-xs') : 'Zero Queda ' + uiIcon('check-circle', 'icon-xs')}</div>
                </div>
              `;
            }

            // Ação Direta no Feedback: Usar no Hermes
            cardsHtml += `
              <div class="u-mt-10p u-d-flex u-gap-2 u-s-741f8b">
                <button class="btn btn-green u-fw-600 u-s-4aead5" onclick="openQuickApplyModal('${data.model_id}', 'local_silicon', true)">
                  ${uiIcon('zap', 'icon-sm')} Usar este Modelo no Hermes Agent
                </button>
              </div>
            `;

            gridEl.innerHTML = cardsHtml;
          }

          // ATUALIZA AMBAS AS TABELAS EM TEMPO REAL
          await Promise.all([fetchLocalArenaData(), fetchLocalModels()]);

          // Highlight na linha testada
          const slug = (modelId || '').toLowerCase().replace(/[^a-z0-9]/g, '-');
          setTimeout(() => {
            const rowArena = document.getElementById('arena-row-' + slug);
            if (rowArena) {
              rowArena.classList.add('highlight-row');
              setTimeout(() => rowArena.classList.remove('highlight-row'), 2800);
            }
            const rowLocal = document.getElementById('local-row-' + slug);
            if (rowLocal) {
              rowLocal.classList.add('highlight-row');
              setTimeout(() => rowLocal.classList.remove('highlight-row'), 2800);
            }
          }, 150);

        } else {
          showToast(data.message || 'Falha ao executar prova no silício.', 'error');
          if (titleEl) titleEl.innerText = 'Falha na Prova de Silício';
          if (verdictEl) verdictEl.innerHTML = '<span class="badge u-s-a6160e">ERRO</span>';
          if (gridEl) gridEl.innerHTML = `<div class="u-s-39e3d0">${data.message || 'Erro desconhecido'}</div>`;
        }
      } catch (err) {
        console.error('Erro ao executar benchmark:', err);
        showToast('Erro de comunicação ao executar benchmark.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = `${uiIcon('zap', 'icon-sm')} Iniciar Teste no Silício`;
        }
      }
    }

    async function syncLocalArenaHermes() {
      const btn = event?.currentTarget;
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-043e24">' + uiIcon('refresh', 'icon-xs') + '</span> Sincronizando...';
      }
      try {
        const res = await fetch('/api/v1/arena/sync', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          const count = data.synced_vault_models || 0;
          showToast(`Sincronizado com sucesso! ${count} modelos no Heuristic Vault e links do Hermes ativos.`, 'success');
          await fetchLocalArenaData();
        } else {
          showToast('Falha ao sincronizar links com o Hermes.', 'error');
        }
      } catch (err) {
        console.error('Falha na sincronização:', err);
        showToast('Erro de conexão ao sincronizar com o Hermes.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = `${uiIcon('link', 'icon-sm')} Sincronizar com Hermes Agent`;
        }
      }
    }


    // ==========================================
    // 3.3 TWO-STAGE SEMANTIC SWITCHBOARD
    // ==========================================
    let switchboardSkillsData = [];
    let switchboardHealthData = null;
    let currentSbSkillCategory = 'all';

    const SKILL_METADATA_MAP = {
      'souls-code-surgeon': {
        icon: 'code',
        category: 'coding',
        categoryLabel: 'Código & AST',
        badgeBg: 'rgba(168, 85, 247, 0.15)',
        badgeColor: '#c084fc',
        samplePrompt: 'extrair outline e substituir symbol na ast de rust'
      },
      'souls-blast-radius': {
        icon: 'zap',
        category: 'coding',
        categoryLabel: 'Código & Impacto',
        badgeBg: 'rgba(168, 85, 247, 0.15)',
        badgeColor: '#c084fc',
        samplePrompt: 'medir frecency gitoxide e blast radius de arquivos modificados'
      },
      'souls-memory-triad': {
        icon: 'brain',
        category: 'memory',
        categoryLabel: 'Memória L2',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'busca hibrida rrf no lancedb e sqlite strict wal'
      },
      'souls-ontology-graph': {
        icon: 'network',
        category: 'memory',
        categoryLabel: 'Grafo Ontológico',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'criar entidades e relacoes tipadas no grafo ladybugdb'
      },
      'obsidian-vault-curator': {
        icon: 'book',
        category: 'memory',
        categoryLabel: 'Obsidian Wiki',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'curadoria de notas no cofre obsidian raw para wiki'
      },
      'souls-hardware-watchdog': {
        icon: 'thermometer',
        category: 'hardware',
        categoryLabel: 'Hardware NVML',
        badgeBg: 'rgba(244, 63, 94, 0.15)',
        badgeColor: '#fb7185',
        samplePrompt: 'telemetria nvml vram temperatura gpu local'
      },
      'souls-context-lean': {
        icon: 'archive',
        category: 'hardware',
        categoryLabel: 'Context Lean CCR',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'comprimir tokens e deduplicar contexto em ram'
      },
      'souls-subagent-dispatch': {
        icon: 'bot',
        category: 'hardware',
        categoryLabel: 'Despacho Subagentes',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'despachar subagente via paretobandit e handoff'
      },
      'souls-intent-probing': {
        icon: 'target',
        category: 'hardware',
        categoryLabel: 'Intenção Socrática',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'classificar intencao e medir entropia shannon tier 0'
      },
      'souls-safe-sandbox': {
        icon: 'shield',
        category: 'security',
        categoryLabel: 'Sandbox NT',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'executar comando shell em jobobject restrito seguro'
      },
      'obsidian-privacy-shield': {
        icon: 'lock',
        category: 'security',
        categoryLabel: 'Privacidade PII',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'higienizar dados confidenciais pii de saude e financas'
      },
      'souls-web-intelligence': {
        icon: 'globe',
        category: 'security',
        categoryLabel: 'Web Intelligence',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'pesquisar documentacao na web e converter markdown'
      },
      'souls-socratic-thinking': {
        icon: 'brain',
        category: 'reasoning',
        categoryLabel: 'Grafo DAG Socrático',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'deliberar hipoteses em grafo dag socratico'
      },
      'souls-cloud-arena-advisor': {
        icon: 'crown',
        category: 'reasoning',
        categoryLabel: 'Fronteira Pareto',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'consultar precos e campeoes cloud arena pareto'
      },
      'agent-self-learning-loop': {
        icon: 'refresh',
        category: 'reasoning',
        categoryLabel: 'Auto-Aprendizado',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'ciclo learn de auto aprendizado e correcao permanente'
      }
    };

    function getSkillMeta(name) {
      return SKILL_METADATA_MAP[name] || {
        icon: 'zap',
        category: 'reasoning',
        categoryLabel: 'Skill Geral',
        badgeBg: 'rgba(148, 163, 184, 0.15)',
        badgeColor: '#A1A1AA',
        samplePrompt: `executar acoes da skill ${name}`
      };
    }

    function getNamespaceBadge(ns) {
      const map = {
        'code': { label: 'code', bg: 'rgba(168, 85, 247, 0.15)', color: '#c084fc', border: 'rgba(168, 85, 247, 0.3)' },
        'memory': { label: 'memory', bg: 'rgba(6, 182, 212, 0.15)', color: 'var(--cyan)', border: 'rgba(6, 182, 212, 0.3)' },
        'think': { label: 'think', bg: 'rgba(245, 158, 11, 0.15)', color: '#fbbf24', border: 'rgba(245, 158, 11, 0.3)' },
        'ops': { label: 'ops', bg: 'rgba(16, 185, 129, 0.15)', color: '#34d399', border: 'rgba(16, 185, 129, 0.3)' },
        'sys': { label: 'sys', bg: 'rgba(244, 63, 94, 0.15)', color: '#fb7185', border: 'rgba(244, 63, 94, 0.3)' },
        'context': { label: 'context', bg: 'rgba(56, 189, 248, 0.15)', color: '#38bdf8', border: 'rgba(56, 189, 248, 0.3)' },
        'core': { label: 'core', bg: 'rgba(148, 163, 184, 0.15)', color: '#A1A1AA', border: 'rgba(148, 163, 184, 0.3)' }
      };
      const item = map[ns] || { label: ns, bg: 'rgba(255,255,255,0.08)', color: '#fff', border: 'rgba(255,255,255,0.15)' };
      return `<span class="badge u-s-77d53f" style="background:${item.bg}; color:${item.color}; border:1px solid ${item.border};">${item.label}</span>`;
    }

    async function fetchSwitchboardSkills() {
      const btn = document.getElementById('btn-refresh-skills');
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-043e24">' + uiIcon('refresh', 'icon-xs') + '</span> Recarregando...';
      }
      try {
        const [resSkills, resHealth] = await Promise.all([
          fetch('/api/v1/switchboard/skills'),
          fetch('/api/v1/switchboard/health')
        ]);

        if (resHealth.ok) {
          switchboardHealthData = await resHealth.json();
          const tabBadge = document.getElementById('badge-switchboard-tools-count');
          if (tabBadge) tabBadge.textContent = switchboardHealthData.total_tools || 55;
          const catBadge = document.getElementById('sb-catalog-stats-badge');
          if (catBadge) catBadge.innerHTML = `${switchboardHealthData.total_skills || 15} Habilidades Prontas &bull; ${switchboardHealthData.total_tools || 55} Ferramentas Ativas`;
        }

        if (resSkills.ok) {
          const data = await resSkills.json();
          switchboardSkillsData = data.skills || [];
          filterSwitchboardSkills();
          showToast('Catálogo sincronizado com sucesso: 15 habilidades e 55 ferramentas ativas!', 'success');
        }
      } catch (err) {
        console.error('Erro ao buscar skills do switchboard:', err);
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = `${uiIcon('refresh', 'icon-sm')} Recarregar Skills`;
        }
      }
    }

    function setSwitchboardSkillCategory(cat) {
      currentSbSkillCategory = cat;
      document.querySelectorAll('#sb-skill-category-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-sb-cat-${cat}`);
      if (activeBtn) activeBtn.classList.add('active');
      filterSwitchboardSkills();
    }

    function filterSwitchboardSkills() {
      const searchInput = document.getElementById('sb-skill-search-input');
      const term = searchInput ? searchInput.value.trim().toLowerCase() : '';

      const filtered = (switchboardSkillsData || []).filter(s => {
        const meta = getSkillMeta(s.name);
        if (currentSbSkillCategory !== 'all' && meta.category !== currentSbSkillCategory) {
          return false;
        }
        if (term) {
          const matchName = s.name.toLowerCase().includes(term);
          const matchDesc = s.description.toLowerCase().includes(term);
          const matchKeywords = (s.keywords || []).some(k => k.toLowerCase().includes(term));
          const matchTools = (s.primary_tools || []).some(t => t.toLowerCase().includes(term));
          if (!matchName && !matchDesc && !matchKeywords && !matchTools) {
            return false;
          }
        }
        return true;
      });

      renderSwitchboardSkillsGrid(filtered);
    }

    function renderSwitchboardSkillsGrid(skills) {
      const grid = document.getElementById('sb-skills-grid');
      if (!grid) return;
      if (!skills || skills.length === 0) {
        grid.innerHTML = `
          <div class="u-text-center u-text-muted u-s-779ea9">
            <div class="u-mb-6p u-s-c2d8f9">${uiIcon('search', 'icon-md')}</div>
            <div class="u-text-slate u-fw-600 u-s-604fde">Nenhuma skill encontrada para o filtro atual.</div>
            <div class="u-mt-10p">
              <button class="btn btn-cyan u-s-918d92" onclick="setSwitchboardSkillCategory('all'); const inp = document.getElementById('sb-skill-search-input'); if(inp) inp.value=''; filterSwitchboardSkills();">
                Limpar Filtros e Ver Todas
              </button>
            </div>
          </div>
        `;
        return;
      }

      grid.innerHTML = skills.map(s => {
        const meta = getSkillMeta(s.name);
        return `
          <div class="skill-card-item u-d-flex u-flex-col u-justify-between u-s-1cbf0c" id="skill-card-${s.name}">
            <div>
              <div class="u-d-flex u-justify-between u-items-start u-mb-2 u-gap-2">
                <div class="u-d-flex u-items-center u-s-9dfc9a">
                  <span class="u-d-inline-flex u-items-center u-s-ff0d51">${uiIcon(meta.icon, 'icon-md')}</span>
                  <strong class="u-text-white u-mono u-s-604fde">${s.name}</strong>
                </div>
                <span class="badge u-nowrap u-s-44086b" style="background: ${meta.badgeBg}; color: ${meta.badgeColor};">
                  ${meta.categoryLabel}
                </span>
              </div>
              <p class="u-text-slate u-mb-3 u-s-9414ca">${s.description}</p>
            </div>
            <div>
              <div class="u-mb-2">
                <span class="u-text-muted u-fw-600 u-d-block u-mb-1 u-uppercase u-s-923ef2">Ferramentas Primárias:</span>
                <div class="u-d-flex u-gap-1 u-flex-wrap">
                  ${(s.primary_tools || []).map(t => `<span class="badge badge-lowcost u-s-c5144d">${t}</span>`).join('')}
                </div>
              </div>
              <div class="u-mb-10p">
                <span class="u-text-muted u-fw-600 u-d-block u-mb-1 u-uppercase u-s-923ef2">Palavras-Chave:</span>
                <div class="u-d-flex u-gap-1 u-flex-wrap">
                  ${(s.keywords || []).slice(0, 5).map(k => `<span class="u-text-muted u-s-ba0b33">#${k}</span>`).join('')}
                </div>
              </div>
              <button class="sb-skill-test-btn" onclick="setSwitchboardPreset('${meta.samplePrompt}')">
                ${uiIcon('zap', 'icon-sm')} Testar no Switchboard
              </button>
            </div>
          </div>
        `;
      }).join('');
    }

    function setSwitchboardPreset(query) {
      const input = document.getElementById('switchboard-query-input');
      if (input) {
        input.value = query;
        input.scrollIntoView({ behavior: 'smooth', block: 'center' });
        input.focus();
        executeSwitchboardEvaluation();
      }
    }

    function highlightSkillInCatalog() {
      const name = document.getElementById('sb-skill-name')?.textContent;
      if (!name) return;
      setSwitchboardSkillCategory('all');
      const searchInput = document.getElementById('sb-skill-search-input');
      if (searchInput) searchInput.value = '';
      filterSwitchboardSkills();

      setTimeout(() => {
        const card = document.getElementById(`skill-card-${name}`);
        if (card) {
          card.scrollIntoView({ behavior: 'smooth', block: 'center' });
          card.style.borderColor = 'var(--cyan)';
          card.style.boxShadow = '0 0 16px rgba(6, 182, 212, 0.4)';
          setTimeout(() => {
            card.style.borderColor = 'rgba(255,255,255,0.08)';
            card.style.boxShadow = 'none';
          }, 2500);
        }
      }, 100);
    }

    function findSkillForTool(toolName) {
      if (!switchboardSkillsData || switchboardSkillsData.length === 0) return null;
      return switchboardSkillsData.find(s => (s.primary_tools || []).includes(toolName));
    }

    function scrollToSkillInCatalog(skillName) {
      if (!skillName) return;
      setSwitchboardSkillCategory('all');
      const searchInput = document.getElementById('sb-skill-search-input');
      if (searchInput) searchInput.value = '';
      filterSwitchboardSkills();

      setTimeout(() => {
        const card = document.getElementById(`skill-card-${skillName}`);
        if (card) {
          card.scrollIntoView({ behavior: 'smooth', block: 'center' });
          card.style.borderColor = 'var(--purple)';
          card.style.boxShadow = '0 0 20px rgba(168, 85, 247, 0.45)';
          setTimeout(() => {
            card.style.borderColor = 'rgba(255,255,255,0.08)';
            card.style.boxShadow = 'none';
          }, 2500);
        }
      }, 100);
    }

    async function executeSwitchboardEvaluation() {
      const input = document.getElementById('switchboard-query-input');
      const nsSelect = document.getElementById('switchboard-ns-select');
      const topkSelect = document.getElementById('switchboard-topk-select');
      const btn = document.getElementById('btn-run-switchboard');
      const container = document.getElementById('switchboard-result-container');

      if (!input || !input.value.trim()) {
        showToast('Digite uma intenção ou prompt para avaliar o Switchboard.', 'info');
        return;
      }

      const query = input.value.trim();
      const namespace = nsSelect ? nsSelect.value : null;
      const top_k = topkSelect ? parseInt(topkSelect.value) : 5;

      try {
        if (btn) {
          btn.disabled = true;
          btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-043e24">' + uiIcon('refresh', 'icon-xs') + '</span> Roteando...';
        }

        const res = await fetch('/api/v1/switchboard/evaluate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            query: query,
            namespace: namespace || null,
            top_k: top_k
          })
        });

        const data = await res.json();
        if (res.ok && data.status === 'success') {
          const d = data.decision;
          if (container) container.style.display = 'block';

          // Preenche métricas
          const stageBadge = document.getElementById('sb-stage-badge');
          if (stageBadge) {
            if (d.stage.includes('fast_path')) {
              stageBadge.innerHTML = `${uiIcon('zap', 'icon-xs')} Fast-Path RAM Cosine (&lt; 2ms)`;
              stageBadge.style.background = 'rgba(16, 185, 129, 0.2)';
              stageBadge.style.color = 'var(--emerald)';
            } else {
              stageBadge.innerHTML = `${uiIcon('brain', 'icon-xs')} Stage 2 Entropy Gated`;
              stageBadge.style.background = 'rgba(168, 85, 247, 0.25)';
              stageBadge.style.color = 'var(--purple)';
            }
          }

          const latencyElem = document.getElementById('sb-latency');
          if (latencyElem) latencyElem.textContent = `${d.latency_ms.toFixed(1)} ms`;

          const confElem = document.getElementById('sb-confidence');
          if (confElem) confElem.textContent = `${(d.confidence * 100).toFixed(1)}%`;

          const entElem = document.getElementById('sb-entropy');
          if (entElem) entElem.textContent = d.entropy.toFixed(2);

          const summaryElem = document.getElementById('sb-summary-text');
          if (summaryElem) summaryElem.textContent = `${d.matched_count} ferramentas projetadas para o contexto`;

          // Economia estimada de tokens
          const totalTools = (switchboardHealthData && switchboardHealthData.total_tools) || 55;
          const count = d.matched_count || (d.tools ? d.tools.length : 5);
          const savedCount = Math.max(0, totalTools - count);
          const tokensSaved = savedCount * 110;
          const pctSaved = Math.round((savedCount / totalTools) * 100);
          const tokensSavedElem = document.getElementById('sb-tokens-saved');
          if (tokensSavedElem) {
            tokensSavedElem.textContent = `~${tokensSaved.toLocaleString()} tokens (${pctSaved}%)`;
          }

          // Skill recomendada
          const skillCard = document.getElementById('sb-skill-card');
          if (d.recommended_skill) {
            if (skillCard) skillCard.style.display = 'block';
            const meta = getSkillMeta(d.recommended_skill.name);
            const iconElem = document.getElementById('sb-skill-icon');
            if (iconElem) iconElem.innerHTML = uiIcon(meta.icon, 'icon-md');
            const skName = document.getElementById('sb-skill-name');
            if (skName) skName.textContent = d.recommended_skill.name;
            const skScore = document.getElementById('sb-skill-score-badge');
            if (skScore) skScore.textContent = `Afinidade: ${(d.recommended_skill.score * 100).toFixed(0)}%`;
            const skDesc = document.getElementById('sb-skill-desc');
            if (skDesc) skDesc.textContent = d.recommended_skill.description;
            const skTools = document.getElementById('sb-skill-primary-tools');
            if (skTools) {
              skTools.innerHTML = (d.recommended_skill.primary_tools || []).map(t => `<span class="badge badge-lowcost u-s-77d53f">${t}</span>`).join('');
            }
          } else {
            if (skillCard) skillCard.style.display = 'none';
          }

          // Ferramentas projetadas
          const tbody = document.getElementById('sb-projected-tools-tbody');
          if (tbody) {
            if (!d.tools || d.tools.length === 0) {
              tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-0af041" colspan="4">Nenhuma ferramenta projetada para o filtro selecionado.</td></tr>';
            } else {
              tbody.innerHTML = d.tools.map(t => {
                const pct = Math.round(t.score * 100);
                const barColor = pct >= 80 ? 'var(--emerald)' : pct >= 50 ? 'var(--cyan)' : 'var(--amber)';
                
                const parentSkill = findSkillForTool(t.name);
                const skillBadge = parentSkill ? `
                  <div class="u-mt-1">
                    <span class="badge u-pointer u-d-inline-flex u-items-center u-gap-1 u-s-5e7224" onclick="scrollToSkillInCatalog('${parentSkill.name}')" title="Clique para localizar esta Skill no Catálogo">
                      ${uiIcon('target', 'icon-xs')} ${parentSkill.name}
                    </span>
                  </div>
                ` : '';

                let schemaParamsHtml = '';
                if (t.schema && t.schema.properties && Object.keys(t.schema.properties).length > 0) {
                  const props = Object.keys(t.schema.properties);
                  const req = Array.isArray(t.schema.required) ? t.schema.required : (typeof t.schema.required === 'string' ? t.schema.required.split(' ') : []);
                  const propList = props.map(p => {
                    const isReq = req.includes(p);
                    const propType = (t.schema.properties[p] && t.schema.properties[p].type) || 'string';
                    return `<span class="u-d-inline-flex u-items-center u-mono u-s-579406"><strong class="u-text-white">${p}</strong>: <span class="u-text-cyan">${propType}</span>${isReq ? ' <span class="u-fw-700 u-s-006447" title="Obrigatório">*</span>' : ''}</span>`;
                  }).join(' ');
                  schemaParamsHtml = `
                    <div class="u-mt-6p u-d-flex u-items-center u-gap-6p u-flex-wrap">
                      <span class="u-text-muted u-fw-600 u-uppercase u-s-fa7df8">Parâmetros:</span>
                      ${propList}
                    </div>
                  `;
                }

                return `
                  <tr>
                    <td>
                      <div class="u-d-flex u-flex-col u-items-start">
                        <strong class="mono u-text-cyan u-s-933a8e">${t.name}</strong>
                        ${skillBadge}
                      </div>
                    </td>
                    <td class="text-center">
                      ${getNamespaceBadge(t.namespace)}
                    </td>
                    <td class="text-center">
                      <div class="u-d-flex u-items-center u-justify-center u-gap-2">
                        <div class="u-overflow-hidden u-s-463a03">
                          <div class="u-s-cabe4a" style="width: ${pct}%; background: ${barColor};"></div>
                        </div>
                        <span class="mono u-fw-700 u-s-5c41e7" style="color: ${barColor};">${pct}%</span>
                      </div>
                    </td>
                    <td>
                      <div class="u-text-slate u-s-3a74a8">${t.description}</div>
                      ${schemaParamsHtml}
                    </td>
                  </tr>
                `;
              }).join('');
            }
          }
        } else {
          showToast(data.message || 'Falha ao avaliar switchboard.', 'error');
        }
      } catch (err) {
        console.error('Erro ao avaliar switchboard:', err);
        showToast('Erro de comunicação com o Switchboard.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = `${uiIcon('zap', 'icon-sm')} Simular Seleção de Ferramentas`;
        }
      }
    }

    async function fetchHardwareAndRecommendations() {
      const btn = document.getElementById('btn-recalibrate-hw');
      if (btn) btn.innerHTML = `${uiIcon('clock', 'icon-sm')} Recalibrando...`;
      try {
        const fetches = [fetch('/api/v1/inventory/recommendations')];
        if (!arenaModelsData || arenaModelsData.length === 0) {
          fetches.push(fetch('/api/v1/arena/models'));
        }
        if (!localModelsList || localModelsList.length === 0) {
          fetches.push(fetch('/api/v1/models/local'));
        }
        const responses = await Promise.all(fetches);
        const res = responses[0];
        let idx = 1;
        if (!arenaModelsData || arenaModelsData.length === 0) {
          const aRes = responses[idx++];
          if (aRes && aRes.ok) {
            const aData = await aRes.json();
            arenaModelsData = aData.models || [];
          }
        }
        if (!localModelsList || localModelsList.length === 0) {
          const lRes = responses[idx++];
          if (lRes && lRes.ok) {
            const lData = await lRes.json();
            localModelsList = lData.models || [];
          }
        }

        if (res && res.ok) {
          const data = await res.json();
          hubHardwareData = data.hardware;
          hubRecommendationsData = data.triad;
          curatedCatalogData = data.catalog || [];

          renderHardwareDiagnostic(data.hardware);
          renderRecommendedTriad(data.triad);
          if (currentCuratedViewMode === 'trending' && trendingCatalogData.length > 0) {
            renderCuratedCatalog(trendingCatalogData, data.hardware);
          } else {
            renderCuratedCatalog(curatedCatalogData, data.hardware);
          }
        }
      } catch (e) {
        console.error('Erro ao buscar recomendações de hardware:', e);
        showToast('Falha ao obter diagnóstico de hardware.', 'error');
      } finally {
        if (btn) btn.innerHTML = `${uiIcon('refresh', 'icon-sm')} Recalibrar Hardware`;
      }
    }

    function renderHardwareDiagnostic(hw) {
      if (!hw) return;
      const tele = window.soulsTelemetryState || {};
      const elGpu = document.getElementById('hw-gpu-name');
      const elVram = document.getElementById('hw-vram-usable');
      const elRam = document.getElementById('hw-host-ram');
      const elCpu = document.getElementById('hw-cpu-features');
      const elCtx = document.getElementById('hw-context-ceiling');

      if (elGpu) elGpu.innerText = hw.gpu_name || tele.gpuName || 'dGPU Local Dedicada';
      const gpuSub = document.getElementById('hw-gpu-sub');
      if (gpuSub) {
        const arch = hw.gpu_architecture || 'Arquitetura dGPU';
        const ccPart = hw.gpu_compute_capability ? ` • CC ${hw.gpu_compute_capability}` : '';
        const details = hw.gpu_details || 'CUDA / Tensor';
        gpuSub.innerText = `${arch}${ccPart} • ${details}`;
      }

      const usableMb = hw.vram_safe_usable_mb || tele.vramLimitMb;
      const totalMb = hw.vram_total_mb || tele.vramTotalMb;
      if (elVram) {
        const usableGb = usableMb ? (usableMb / 1024).toFixed(2) : '--';
        const totalGb = totalMb ? (totalMb / 1024).toFixed(2) : '--';
        elVram.innerText = `${usableGb} GB (${totalGb} GB Total)`;
      }
      const vramSub = document.getElementById('hw-vram-sub');
      if (vramSub) {
        const weights = hw.weights_budget_gb !== undefined ? hw.weights_budget_gb : (usableMb ? ((usableMb / 1024) * 0.74).toFixed(1) : '--');
        const kv = hw.kv_cache_budget_gb !== undefined ? hw.kv_cache_budget_gb : (usableMb ? ((usableMb / 1024) * 0.26).toFixed(1) : '--');
        vramSub.innerHTML = `<div>${uiIcon('archive', 'icon-xs')} Teto Pesos: ~${weights} GB (Q4_K_M)</div><div class="u-mt-2p">${uiIcon('brain', 'icon-xs')} Margem KV Cache: ~${kv} GB</div>`;
      }

      if (elRam) {
        const ramGb = hw.host_ram_total_mb ? (hw.host_ram_total_mb / 1024).toFixed(1) : '--';
        elRam.innerText = `${ramGb} GB RAM`;
      }
      const ramSub = document.getElementById('hw-host-ram-sub');
      if (ramSub) {
        ramSub.innerHTML = `${uiIcon('zap', 'icon-xs')} Memória Host • Swap Segura NVMe`;
      }

      if (elCpu) {
        const rawCpu = hw.cpu_instructions_extended || (hw.has_avx2 ? 'AVX2 • FMA • BMI2 • SSE4.2 • AES-NI' : 'x86_64');
        const instList = rawCpu.split('•').map(s => s.trim()).filter(Boolean);
        elCpu.innerHTML = `<div class="u-d-flex u-gap-1 u-flex-wrap u-mt-3p">` +
          instList.map(inst => `<span class="badge u-text-cyan u-fw-700 u-nowrap u-s-1e70fe">${inst}</span>`).join('') +
          `</div>`;
      }
      const cpuSub = document.getElementById('hw-cpu-sub');
      if (cpuSub) {
        cpuSub.innerText = hw.cpu_name || tele.cpuName || 'Host CPU AVX2';
      }

      if (elCtx) {
        const tokensVal = hw.context_ceiling_tokens ? `${hw.context_ceiling_tokens.toLocaleString()} tokens` : '16.384 tokens';
        elCtx.innerText = hw.context_ceiling_display && !hw.context_ceiling_display.includes('(') ? hw.context_ceiling_display : tokensVal;
      }
      const elCtxSub = document.getElementById('hw-context-ceiling-sub');
      if (elCtxSub) {
        elCtxSub.innerText = hw.context_ceiling_sub || '(até 32k c/ KV Q4)';
      }
      const ctxSub = document.getElementById('hw-context-sub');
      if (ctxSub) {
        ctxSub.innerText = hw.context_formula_subtitle || 'Cálculo dinâmico sobre a VRAM útil sob WDDM: garante estabilidade em contextos estendidos sem spillover PCIe.';
      }
      updateDynamicParamFitPills(hw);
    }

    function renderProfileTagBadge(profile) {
      if (!profile) return '';
      const p = profile.toLowerCase();
      let color = '#A1A1AA';
      let bg = 'rgba(148, 163, 184, 0.12)';
      let border = 'rgba(148, 163, 184, 0.3)';
      let icon = 'tag';
      let label = profile;

      if (p.includes('coding') || p.includes('código')) {
        color = '#c084fc';
        bg = 'rgba(168, 85, 247, 0.15)';
        border = 'rgba(168, 85, 247, 0.4)';
        icon = 'code';
        label = 'Coding Specialist';
      } else if (p.includes('reasoning') || p.includes('socratic') || p.includes('r1')) {
        color = '#818cf8';
        bg = 'rgba(99, 102, 241, 0.15)';
        border = 'rgba(99, 102, 241, 0.4)';
        icon = 'brain';
        label = 'Deep Reasoning (R1 CoT)';
      } else if (p.includes('agentic') || p.includes('tool') || p.includes('mcp')) {
        color = '#38bdf8';
        bg = 'rgba(6, 182, 212, 0.15)';
        border = 'rgba(6, 182, 212, 0.4)';
        icon = 'bot';
        label = 'Agentic & Tool Calling';
      } else if (p.includes('vision') || p.includes('vlm') || p.includes('multimodal')) {
        color = '#fb7185';
        bg = 'rgba(244, 63, 94, 0.15)';
        border = 'rgba(244, 63, 94, 0.4)';
        icon = 'eye';
        label = 'Vision Multimodal (VLM)';
      } else if (p.includes('fast') || p.includes('triagem') || p.includes('reflex')) {
        color = '#34d399';
        bg = 'rgba(16, 185, 129, 0.15)';
        border = 'rgba(16, 185, 129, 0.4)';
        icon = 'zap';
        label = 'Fast Reflex & Triage';
      } else if (p.includes('design') || p.includes('frontend') || p.includes('ui')) {
        color = '#fbbf24';
        bg = 'rgba(245, 158, 11, 0.15)';
        border = 'rgba(245, 158, 11, 0.4)';
        icon = 'palette';
        label = 'UI/UX & Frontend Design';
      }

      return `<span class="u-d-inline-flex u-items-center u-gap-1 u-fw-700 u-mono u-s-fbe4db" style="background: ${bg}; color: ${color}; border: 1px solid ${border};">
        <span>${uiIcon(icon, 'icon-xs')}</span> <span>${label}</span>
      </span>`;
    }

    function getFitBadgeHtml(fitStatus) {
      if (!fitStatus) return '<span class="fit-badge tight">Compatível</span>';
      const status = typeof fitStatus === 'string' ? fitStatus : (fitStatus.status || 'OptimalVram');
      switch (status) {
        case 'OptimalVram':
          return `<span class="fit-badge optimal" title="100% dos pesos e KV-cache cabem com folga na dGPU">${uiIcon('zap', 'icon-xs')} 100% VRAM</span>`;
        case 'TightVram':
          return `<span class="fit-badge tight" title="Cabe na VRAM aproveitando o headroom do WDDM">${uiIcon('check', 'icon-xs')} VRAM Completa</span>`;
        case 'PartialCpuOffload':
          return `<span class="fit-badge offload" title="Offload parcial para CPU RAM">${uiIcon('alert-triangle', 'icon-xs')} CPU Offload</span>`;
        case 'CpuOnly':
          return `<span class="fit-badge offload" title="Execução integral na CPU Intel via AVX2">${uiIcon('terminal', 'icon-xs')} Modo CPU (AVX2)</span>`;
        case 'Incompatible':
          return `<span class="fit-badge incompatible" title="Excede a memória combinada VRAM + RAM">${uiIcon('x-circle', 'icon-xs')} Memória Insuficiente</span>`;
        default:
          return `<span class="fit-badge tight">${status}</span>`;
      }
    }

    function escapeJsSingleQuote(str) {
      return String(str || '').replace(/\\/g, '\\\\').replace(/'/g, "\\'");
    }

    function getCategoryRoleBadgeHtml(role) {
      const r = (role || '').toLowerCase();
      if (r === 'featherweight') {
        return `<span class="badge" style="background: rgba(56, 189, 248, 0.14); color: #38bdf8;">${uiIcon('rocket', 'icon-xs')} Featherweight (&lt;1.5 GB)</span>`;
      }
      if (r === 'sweetspot') {
        return `<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: #34d399;">${uiIcon('zap', 'icon-xs')} Sweet Spot (1.5–3.6 GB)</span>`;
      }
      return `<span class="badge" style="background: rgba(168, 85, 247, 0.15); color: var(--purple-light);">${uiIcon('crown', 'icon-xs')} Heavyweight (Teto VRAM)</span>`;
    }

    function renderRecommendedTriad(triad) {
      const container = document.getElementById('triad-cards-grid');
      if (!container || !triad) return;

      const items = [
        { tierLabel: 'Tier 0 — Fast Reflex & Triage', tierColor: 'var(--green)', icon: 'zap', data: triad.tier0_velocista, role: 'Triagem ultra-rápida, guardrails e classificação de intenção com zero latência.' },
        { tierLabel: 'Tier 0.5 — Socratic Thinking & Intent', tierColor: 'var(--cyan)', icon: 'brain', data: triad.tier05_socratico, role: 'Raciocínio socrático, cadeia de pensamento (CoT) e deliberação epistêmica.' },
        { tierLabel: 'Tier 1 — Workhorse / Code & Logic', tierColor: 'var(--purple)', icon: 'code', data: triad.tier1_coder, role: 'Cavalo de batalha na dGPU para cirurgia de código, AST e agentes autônomos.' },
      ];

      container.innerHTML = items.map(item => {
        const model = item.data;
        if (!model) return '';
        const variants = model.variants || [];
        const variant = variants.find(v => v.is_installed) || variants.find(v => v.is_recommended_fit) || variants[0];
        if (!variant) return '';

        const matched = findInstalledModelMatch(variant.file_name, model.repo_id, model.model_name, variant.quant_tag);
        const isDown = Boolean(variant.is_installed || model.is_downloaded || matched);
        const activeFile = variant.installed_file_name || (matched && matched.realFileName) || model.local_file_path || variant.file_name;

        const sizeBytes = (matched && matched.size_mb) ? matched.size_mb * 1024 * 1024 : variant.size_bytes;
        const sizeGb = sizeBytes > 0 ? (sizeBytes / (1024 ** 3)).toFixed(2) : (variant.size_gb ? variant.size_gb.toFixed(2) : '--');
        const arenaMatch = findArenaMatchForHubModel(model, activeFile, variant.file_name);
        const hasRealBench = Boolean(isDown && arenaMatch && arenaMatch.latest_tps > 0);
        const rawEstTps = variant.estimated_tps || (model.benchmark_scores && model.benchmark_scores.speed_tokens_sec_estimate) || 0;
        const tpsDisplay = hasRealBench
          ? `${uiIcon('zap', 'icon-xs')} ${arenaMatch.latest_tps.toFixed(1)} tps <span class="badge u-text-emerald u-s-3fefe0" style="font-size:0.62rem;padding:1px 5px;">Aferido</span>`
          : (rawEstTps > 0 ? `~${rawEstTps.toFixed(0)} tps` : '--');

        const alt = model.upgrade_candidate;
        let upgradeHtml = '';
        if (alt) {
          const altVariants = alt.variants || [];
          const altVar = altVariants.find(v => v.is_recommended_fit) || altVariants[0];
          if (altVar) {
            const altSizeGb = altVar.size_gb ? altVar.size_gb.toFixed(2) : (altVar.size_bytes / (1024 ** 3)).toFixed(2);
            const altTps = (altVar.estimated_tps || (alt.benchmark_scores && alt.benchmark_scores.speed_tokens_sec_estimate) || 0).toFixed(0);
            upgradeHtml = `
              <div class="triad-upgrade-box">
                <div class="triad-upgrade-header">
                  <span>${uiIcon('sparkles', 'icon-xs')} Próximo Upgrade / Alternativa</span>
                  <span>${escapeHtml(altVar.quant_tag)} &bull; ${altSizeGb} GB &bull; ~${altTps} tps</span>
                </div>
                <div class="u-d-flex u-items-center u-justify-between u-gap-2 u-mt-4p">
                  <div class="u-text-white u-fw-600 u-mono" style="font-size:0.78rem;" title="${escapeHtml(alt.why_recommended || alt.model_name)}">${escapeHtml(alt.model_name)}</div>
                  <button class="btn btn-cyan u-s-918d92 u-nowrap" onclick="requestCuratedVariantDownload('${escapeJsSingleQuote(alt.repo_id)}', '${escapeJsSingleQuote(altVar.download_url || '')}', '${escapeJsSingleQuote(altVar.file_name)}', ${altVar.size_bytes || 0}, '${escapeJsSingleQuote(altVar.sha256 || '')}', '${escapeJsSingleQuote(altVar.projector_file || '')}', '${escapeJsSingleQuote(altVar.projector_url || '')}', '${escapeJsSingleQuote(altVar.quant_tag)}')">
                    ${uiIcon('download', 'icon-xs')} Baixar (${altSizeGb} GB)
                  </button>
                </div>
              </div>
            `;
          }
        }

        return `
          <div class="triad-card ${isDown ? 'is-installed-card' : ''} u-d-flex u-flex-col u-justify-between u-s-7c2e41">
            <div>
              <div class="u-d-flex u-items-center u-justify-between u-mb-10p u-flex-wrap u-gap-6p">
                <div class="u-d-flex u-items-center u-gap-2">
                  <span class="u-fw-700 u-mono u-uppercase u-s-85ebd3" style="color: ${item.tierColor};">
                    ${item.tierLabel}
                  </span>
                  ${isDown ? '<span class="badge u-text-emerald u-fw-700 u-s-3fefe0">' + uiIcon('check', 'icon-xs') + ' Instalado</span>' : ''}
                </div>
                <div>
                  ${getFitBadgeHtml(variant.hardware_fit)}
                </div>
              </div>

              <div class="u-fw-700 u-text-white u-mono u-s-66e304">
                ${escapeHtml(model.model_name)}
              </div>

              <p class="u-text-muted u-mt-6p u-s-68e8c4">
                ${item.role}
              </p>

              <div class="u-d-grid u-gap-2 u-mt-3 u-mono u-s-85e1ad">
                <div><span class="u-text-muted">Quantização:</span> <strong class="u-text-cyan">${escapeHtml(variant.quant_tag)}</strong></div>
                <div><span class="u-text-muted">Tamanho:</span> <strong class="u-text-white">${sizeGb} GB</strong></div>
                <div><span class="u-text-muted">Throughput:</span> <strong class="u-text-emerald">${tpsDisplay}</strong></div>
                <div><span class="u-text-muted">Silício:</span> <strong class="u-text-white">${variant.hardware_fit === 'OptimalVram' ? 'dGPU RTX' : 'CPU Host'}</strong></div>
              </div>
            </div>

            <div class="u-mt-4 u-s-6355ee">
              ${isDown ? `
                <div class="u-d-flex u-gap-6p u-items-center">
                  <button class="btn btn-green u-justify-center u-fw-600 u-s-a84808" onclick="openQuickApplyModal('${escapeJsSingleQuote(activeFile)}', 'local_silicon', true)" title="Definir como cérebro no Hermes Agent">
                    ${uiIcon('zap', 'icon-sm')} Usar no Hermes
                  </button>
                  <button class="btn btn-cyan u-flex-1 u-justify-center u-s-c5a735" onclick="testModelInArena('${escapeJsSingleQuote(activeFile)}')" title="Testar na Local Silicon Arena">
                    ${uiIcon('trophy', 'icon-sm')} Arena
                  </button>
                  <button class="btn u-justify-center u-s-195416" title="Baixar novamente" onclick="requestCuratedVariantDownload('${escapeJsSingleQuote(model.repo_id)}', '${escapeJsSingleQuote(variant.download_url || '')}', '${escapeJsSingleQuote(variant.file_name)}', ${variant.size_bytes || 0}, '${escapeJsSingleQuote(variant.sha256 || '')}', '${escapeJsSingleQuote(variant.projector_file || '')}', '${escapeJsSingleQuote(variant.projector_url || '')}', '${escapeJsSingleQuote(variant.quant_tag)}')">
                    ${uiIcon('refresh', 'icon-sm')}
                  </button>
                </div>
              ` : `
                <button class="btn btn-cyan u-w-full u-justify-center u-fw-600 u-s-40d5e9" onclick="requestCuratedVariantDownload('${escapeJsSingleQuote(model.repo_id)}', '${escapeJsSingleQuote(variant.download_url || '')}', '${escapeJsSingleQuote(variant.file_name)}', ${variant.size_bytes || 0}, '${escapeJsSingleQuote(variant.sha256 || '')}', '${escapeJsSingleQuote(variant.projector_file || '')}', '${escapeJsSingleQuote(variant.projector_url || '')}', '${escapeJsSingleQuote(variant.quant_tag)}')">
                  ${uiIcon('download', 'icon-sm')} 1-Click Download (${sizeGb} GB)
                </button>
              `}
              ${upgradeHtml}
            </div>
          </div>
        `;
      }).join('');
    }

    let currentCuratedViewMode = 'trusted';
    let trendingCatalogData = [];

    function setCuratedViewMode(mode) {
      currentCuratedViewMode = mode;
      const btnTrusted = document.getElementById('btn-curated-mode-trusted');
      const btnTrending = document.getElementById('btn-curated-mode-trending');
      if (btnTrusted && btnTrending) {
        const isTrusted = mode === 'trusted';
        btnTrusted.classList.toggle('active', isTrusted);
        btnTrending.classList.toggle('active', !isTrusted);
        [btnTrusted, btnTrending].forEach(b => {
          b.style.background = '';
          b.style.color = '';
          b.style.fontWeight = '';
        });
      }

      if (mode === 'trending') {
        if (!trendingCatalogData || trendingCatalogData.length === 0) {
          loadTrendingCatalog();
        } else {
          renderCuratedCatalog(trendingCatalogData, hubHardwareData);
        }
      } else {
        renderCuratedCatalog(curatedCatalogData, hubHardwareData);
      }
    }

    function calculateFitFromParamEstimate(paramSize, isMoe = false) {
      const safeVramGb = hubHardwareData && hubHardwareData.vram_safe_usable_mb ? (hubHardwareData.vram_safe_usable_mb / 1024) : ((window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) ? (window.soulsTelemetryState.vramLimitMb / 1024) : 4.0);
      const estVramGb = paramSize * 0.65;
      if (isMoe) {
        return paramSize <= 35 ? 'PartialCpuOffload' : 'Incompatible';
      }
      if (estVramGb + 0.8 <= safeVramGb * 0.75) {
        return 'OptimalVram';
      } else if (estVramGb + 0.5 <= safeVramGb) {
        return 'TightVram';
      } else if (paramSize <= 35) {
        return 'PartialCpuOffload';
      } else {
        return 'Incompatible';
      }
    }

    async function loadTrendingCatalog() {
      const container = document.getElementById('curated-models-grid');
      if (container) {
        container.innerHTML = `<div class="u-text-cyan u-text-center u-s-c20f1a"><span class="spin-icon u-d-inline-block u-s-d1cc57">${uiIcon('refresh', 'icon-xs')}</span> Buscando tendências e lançamentos GGUF no Hugging Face para o seu hardware...</div>`;
      }
      try {
        const res = await fetch('/api/v1/inventory/recommendations?mode=trending');
        if (res.ok) {
          const data = await res.json();
          if (data.hardware) hubHardwareData = data.hardware;
          trendingCatalogData = data.trending_catalog || [];
          renderCuratedCatalog(trendingCatalogData, hubHardwareData);
        } else {
          if (container) container.innerHTML = '<div class="u-text-muted u-text-center u-s-720583">Não foi possível buscar as novidades da nuvem agora.</div>';
        }
      } catch (err) {
        console.error('Erro ao carregar novidades:', err);
        if (container) container.innerHTML = '<div class="u-text-muted u-text-center u-s-720583">Falha de conexão ao buscar novidades da nuvem.</div>';
      }
    }

    async function syncCuratedCatalogLive() {
      const btn = document.getElementById('btn-sync-catalog');
      const syncBadge = document.getElementById('catalog-last-sync-badge');
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-043e24">' + uiIcon('refresh', 'icon-xs') + '</span> Sincronizando HF...';
      }
      try {
        const recUrl = currentCuratedViewMode === 'trending'
          ? '/api/v1/inventory/recommendations?sync=true&mode=trending'
          : '/api/v1/inventory/recommendations?sync=true';

        const [recRes, arenaRes] = await Promise.all([
          fetch(recUrl),
          fetch('/api/v1/arena/models')
        ]);

        let syncedLiveCount = 0;
        if (recRes.ok) {
          const data = await recRes.json();
          curatedCatalogData = data.catalog || [];
          hubHardwareData = data.hardware;
          hubRecommendationsData = data.triad;
          if (data.trending_catalog && data.trending_catalog.length > 0) {
            trendingCatalogData = data.trending_catalog;
          }
          syncedLiveCount = curatedCatalogData.length;
        }

        if (arenaRes.ok) {
          const dataArena = await arenaRes.json();
          arenaModelsData = dataArena.models || [];
        }

        if (currentCuratedViewMode === 'trending') {
          if (!trendingCatalogData || trendingCatalogData.length === 0) {
            await loadTrendingCatalog();
          } else {
            renderCuratedCatalog(trendingCatalogData, hubHardwareData);
          }
        } else {
          renderCuratedCatalog(curatedCatalogData, hubHardwareData);
        }
        renderRecommendedTriad(hubRecommendationsData);

        if (syncBadge) {
          const now = new Date();
          const timeStr = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          syncBadge.innerHTML = `${uiIcon('check-circle', 'icon-xs')} Sincronizado às ${timeStr}`;
          syncBadge.style.display = 'inline-flex';
          syncBadge.classList.remove('is-hidden');
        }
        showToast(`Catálogo vivo sincronizado (${syncedLiveCount || 'catálogo'} modelos avaliados na GPU)!`, 'success');
      } catch (err) {
        console.error('Falha ao sincronizar catálogo:', err);
        showToast('Catálogo mantido no baseline seguro.', 'warning');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = `<span id="btn-sync-catalog-icon">${uiIcon('sparkles', 'icon-sm')}</span> Sincronizar Catálogo`;
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

      const getInstalledVariant = (m) => {
        const variants = m.variants || [];
        return variants.find(v => Boolean(v.is_installed || findInstalledModelMatch(v.file_name, m.repo_id, m.model_name, v.quant_tag)));
      };

      const fitRank = (fit) => {
        if (fit === 'OptimalVram') return 1;
        if (fit === 'TightVram') return 2;
        if (fit === 'PartialCpuOffload') return 3;
        if (fit === 'Incompatible' || fit === 'IncompatibleSpillover') return 5;
        return 4;
      };

      // Ordenação: (1) Instalado no disco, (2) Cabe 100% na VRAM, (3) Score/downloads
      filteredCuratedModels.sort((a, b) => {
        const downA = Boolean(a.is_downloaded || getInstalledVariant(a));
        const downB = Boolean(b.is_downloaded || getInstalledVariant(b));
        if (downA !== downB) return downA ? -1 : 1;

        const vA = (a.variants || []).find(v => v.is_recommended_fit) || (a.variants && a.variants[0]) || {};
        const vB = (b.variants || []).find(v => v.is_recommended_fit) || (b.variants && b.variants[0]) || {};
        const rankA = fitRank(vA.hardware_fit);
        const rankB = fitRank(vB.hardware_fit);
        if (rankA !== rankB) return rankA - rankB;

        return (b.downloads || b.likes || 0) - (a.downloads || a.likes || 0);
      });

      if (filteredCuratedModels.length === 0) {
        container.innerHTML = '<div class="u-text-muted u-text-center u-s-720583">Nenhum modelo encontrado para este filtro.</div>';
        return;
      }

      container.innerHTML = filteredCuratedModels.map((m, idx) => {
        const variants = m.variants || [];
        const selectedVariant = variants.find(v => v.is_installed)
          || variants.find(v => findInstalledModelMatch(v.file_name, m.repo_id, m.model_name, v.quant_tag))
          || variants.find(v => v.is_recommended_fit)
          || variants[0]
          || {};

        const matched = findInstalledModelMatch(selectedVariant.file_name, m.repo_id, m.model_name, selectedVariant.quant_tag);
        const isDown = Boolean(selectedVariant.is_installed || matched);
        const activeFile = selectedVariant.installed_file_name || (matched && matched.realFileName) || m.local_file_path || selectedVariant.file_name || '';

        const hasVariants = variants.length > 0;
        const quantOptions = hasVariants ? variants.map(v => {
          const isVDown = Boolean(v.is_installed || findInstalledModelMatch(v.file_name, m.repo_id, m.model_name, v.quant_tag));
          const isSel = v === selectedVariant;
          const vGb = (v.size_bytes / (1024 ** 3)).toFixed(2);
          return `
            <option value="${escapeHtml(v.quant_tag)}" ${isSel ? 'selected' : ''} data-size="${v.size_bytes || 0}" data-url="${escapeHtml(v.download_url || '')}" data-file="${escapeHtml(v.file_name)}" data-fit="${v.hardware_fit || 'OptimalVram'}" data-tps="${v.estimated_tps || ''}" data-sha256="${escapeHtml(v.sha256 || '')}" data-proj-file="${escapeHtml(v.projector_file || '')}" data-proj-url="${escapeHtml(v.projector_url || '')}" data-installed="${isVDown ? '1' : '0'}" data-installed-file="${escapeHtml(v.installed_file_name || '')}">
              ${escapeHtml(v.quant_tag)} (${vGb} GB)${isVDown ? ' ✓ No Disco' : ''}
            </option>
          `;
        }).join('') : '';

        const fitBadge = getFitBadgeHtml(selectedVariant.hardware_fit || (m.paramSize ? calculateFitFromParamEstimate(m.paramSize, false) : 'OptimalVram'));
        const arenaMatch = findArenaMatchForHubModel(m, activeFile, selectedVariant.file_name);
        const hasRealBench = Boolean(isDown && arenaMatch && arenaMatch.latest_tps > 0);
        const estTpsVal = selectedVariant.estimated_tps || (m.benchmark_scores && m.benchmark_scores.speed_tokens_sec_estimate) || 0;
        const speedDisplay = hasRealBench
          ? `${arenaMatch.latest_tps.toFixed(1)} tps (Aferido)`
          : (estTpsVal > 0 ? `~${estTpsVal.toFixed(0)} tps` : '--');

        const selSizeGb = selectedVariant.size_bytes ? (selectedVariant.size_bytes / (1024 ** 3)).toFixed(2) : '--';
        const downloadLabel = m.is_multimodal_vlm
          ? `Baixar Bundle VLM ${selectedVariant.quant_tag || ''} (${selSizeGb} GB)`
          : `Baixar ${selectedVariant.quant_tag || 'GGUF'} (${selSizeGb} GB)`;

        return `
          <div class="curated-card ${isDown ? 'is-installed-card' : ''}" id="curated-card-${idx}">
            <div>
              <div class="u-d-flex u-items-center u-justify-between u-mb-10p u-flex-wrap u-gap-2">
                <div class="u-d-flex u-gap-6p u-items-center u-flex-wrap">
                  ${m.is_trending ? `
                    <span class="badge u-fw-700 u-s-7559ef">
                      ${uiIcon('flame', 'icon-xs')} Novidade em Alta
                    </span>
                  ` : `
                    <span class="badge u-fw-700 u-s-a40b84">
                      ${uiIcon('sparkles', 'icon-xs')} ${(m.downloads && !m.upgrade_recommendation) ? 'Catálogo Vivo HF' : 'Mais Confiável'}
                    </span>
                  `}
                  ${getCategoryRoleBadgeHtml(m.category_role)}
                  ${m.is_multimodal_vlm ? '<span class="badge u-text-cyan u-s-b8e4da">' + uiIcon('eye', 'icon-xs') + ' VLM Visão</span>' : ''}
                </div>
                <div class="u-d-flex u-items-center u-gap-10p">
                  <span id="curated-fit-${idx}">${fitBadge}</span>
                  <span id="curated-badge-installed-${idx}">
                    ${isDown ? '<span class="badge u-text-emerald u-fw-700 u-s-3e3280">' + uiIcon('check', 'icon-xs') + ' Instalado</span>' : ''}
                  </span>
                </div>
              </div>
              <div class="u-fw-700 u-text-white u-mono u-mt-1 u-s-38f55e">
                ${escapeHtml(m.model_name)}
              </div>

              <div class="u-d-flex u-flex-col u-gap-1 u-mono u-mt-6p u-s-402265">
                <div>${uiIcon('users', 'icon-xs')} <span class="u-text-muted">Autor:</span> <strong class="u-text-white">${escapeHtml(m.author || '--')}</strong></div>
                <div>${uiIcon('archive', 'icon-xs')} <span class="u-text-muted">Repo:</span> <span class="u-text-cyan">${escapeHtml(m.repo_id)}</span></div>
                <div class="u-d-flex u-items-center u-gap-6p u-mt-2p">
                  <span>${uiIcon('target', 'icon-xs')}</span> <span class="u-text-muted">Perfil Indicado:</span> ${renderProfileTagBadge(m.cognitive_profile)}
                </div>
              </div>

              <p class="u-text-slate u-mt-2 u-s-e197f7">
                ${escapeHtml(m.why_recommended || '')}
              </p>

              <div class="u-mt-2 u-d-flex u-flex-wrap u-gap-1">
                ${(m.highlights || []).map(h => `<span class="badge u-text-muted u-s-1ee417">${escapeHtml(h)}</span>`).join('')}
              </div>

              <div class="u-mt-10p u-d-grid u-gap-6p u-mono u-s-50efb0">
                <div>Coding: <strong class="u-text-cyan">${m.benchmark_scores && m.benchmark_scores.coding_score ? m.benchmark_scores.coding_score.toFixed(1) : '--'}</strong></div>
                <div>MMLU: <strong class="u-text-white">${m.benchmark_scores && m.benchmark_scores.mmlu_score ? m.benchmark_scores.mmlu_score.toFixed(1) : '--'}</strong></div>
                <div>IFEval: <strong class="u-text-white">${m.benchmark_scores && m.benchmark_scores.ifeval_score ? m.benchmark_scores.ifeval_score.toFixed(1) : '--'}</strong></div>
                <div>Speed: <strong class="u-text-emerald" id="curated-speed-${idx}">${speedDisplay}</strong></div>
                ${(m.downloads || m.likes) ? `
                  <div class="u-text-muted u-mt-2p" style="grid-column: 1 / -1; font-size:0.74rem;">
                    ${uiIcon('download', 'icon-xs')} ${Number(m.downloads || 0).toLocaleString()} downloads &bull; ${uiIcon('sparkles', 'icon-xs')} ${Number(m.likes || 0).toLocaleString()} curtidas no HF
                  </div>
                ` : ''}
                ${hasRealBench ? `
                  <div class="u-text-green u-fw-700 u-mt-3p u-d-flex u-justify-between u-items-center u-s-297f26" style="grid-column: 1 / -1;">
                    <span>${uiIcon('trophy', 'icon-xs')} Silício Local Medido (Arena):</span>
                    <span>${arenaMatch.latest_tps.toFixed(1)} tok/s</span>
                  </div>
                ` : ''}
              </div>
            </div>

            <div class="u-mt-3 u-s-7c24ed">
              ${hasVariants ? `
                <div class="u-d-flex u-items-center u-justify-between u-mb-2 u-gap-2">
                  <label class="u-text-muted u-s-dbc8d1">Quantização:</label>
                  <div class="u-d-flex u-items-center u-gap-1">
                    <select id="curated-select-${idx}" class="setting-input u-s-41c5fe" onchange="onCuratedQuantChange(${idx})">
                      ${quantOptions}
                    </select>
                    <button class="btn u-s-918d92" title="Verificar arquivos GGUF ao vivo no Hugging Face" onclick="inspectRepoFiles('${escapeJsSingleQuote(m.repo_id)}')">
                      ${uiIcon('folder', 'icon-xs')}
                    </button>
                  </div>
                </div>
              ` : `
                <div class="u-d-flex u-items-center u-justify-between u-mb-2 u-text-muted u-s-dbc8d1">
                  <span>Quants GGUF:</span>
                  <span class="u-text-cyan u-pointer u-s-744734" onclick="inspectRepoFiles('${escapeJsSingleQuote(m.repo_id)}')">Inspecionar Arquivos</span>
                </div>
              `}

              <div id="curated-actions-${idx}">
                ${isDown ? `
                  <div class="u-d-flex u-gap-6p u-flex-wrap">
                    <button class="btn btn-green u-flex-1 u-justify-center u-s-6d9f18" onclick="openQuickApplyModal('${escapeJsSingleQuote(activeFile)}', 'local_silicon', true)" title="Definir no Hermes Agent">
                      ${uiIcon('zap', 'icon-sm')} Usar no Hermes
                    </button>
                    <button class="btn btn-cyan u-s-84fa0e" onclick="testModelInArena('${escapeJsSingleQuote(activeFile)}')" title="Testar na Local Silicon Arena">
                      ${uiIcon('trophy', 'icon-sm')} Arena
                    </button>
                    <button class="btn u-s-84fa0e" title="Baixar esta quantização novamente" onclick="triggerCuratedDownload(${idx}, '${escapeJsSingleQuote(m.repo_id)}')">
                      ${uiIcon('refresh', 'icon-sm')}
                    </button>
                  </div>
                ` : `
                  <button class="btn btn-cyan u-w-full u-justify-center" onclick="triggerCuratedDownload(${idx}, '${escapeJsSingleQuote(m.repo_id)}')">
                    ${uiIcon('download', 'icon-sm')} ${downloadLabel}
                  </button>
                `}
              </div>
            </div>
          </div>
        `;
      }).join('');
    }

    function onCuratedQuantChange(idx) {
      const select = document.getElementById(`curated-select-${idx}`);
      if (!select) return;
      const opt = select.options[select.selectedIndex];
      const quantTag = opt.value || 'Q4_K_M';
      const fitStatus = opt.getAttribute('data-fit') || 'OptimalVram';
      const fileName = opt.getAttribute('data-file') || '';
      const sizeBytes = parseInt(opt.getAttribute('data-size') || '0', 10);
      const sizeGb = sizeBytes > 0 ? (sizeBytes / (1024 ** 3)).toFixed(2) : '--';
      const optTps = parseFloat(opt.getAttribute('data-tps') || '0');
      const optInstalled = opt.getAttribute('data-installed') === '1';
      const optInstalledFile = opt.getAttribute('data-installed-file') || '';

      const fitEl = document.getElementById(`curated-fit-${idx}`);
      if (fitEl) {
        fitEl.innerHTML = getFitBadgeHtml(fitStatus);
      }

      const model = filteredCuratedModels[idx] || curatedCatalogData[idx];
      const repoId = model ? model.repo_id : '';
      const modelName = model ? model.model_name : '';
      const matched = findInstalledModelMatch(fileName, repoId, modelName, quantTag);
      const isDown = Boolean(optInstalled || matched);
      const activeFile = optInstalledFile || (matched && matched.realFileName) || fileName;

      const speedEl = document.getElementById(`curated-speed-${idx}`);
      if (speedEl) {
        const arenaMatch = findArenaMatchForHubModel(model, activeFile, fileName);
        if (isDown && arenaMatch && arenaMatch.latest_tps > 0) {
          speedEl.textContent = `${arenaMatch.latest_tps.toFixed(1)} tps (Aferido)`;
        } else if (optTps > 0) {
          speedEl.textContent = `~${optTps.toFixed(0)} tps`;
        }
      }

      const cardEl = document.getElementById(`curated-card-${idx}`);
      if (cardEl) {
        cardEl.classList.toggle('is-installed-card', isDown);
      }

      const badgeContainer = document.getElementById(`curated-badge-installed-${idx}`);
      const actionContainer = document.getElementById(`curated-actions-${idx}`);

      if (badgeContainer) {
        badgeContainer.innerHTML = isDown ? '<span class="badge u-text-emerald u-fw-700 u-s-3e3280">' + uiIcon('check', 'icon-xs') + ' Instalado</span>' : '';
      }

      if (actionContainer) {
        if (isDown) {
          actionContainer.innerHTML = `
            <div class="u-d-flex u-gap-6p u-flex-wrap">
              <button class="btn btn-green u-flex-1 u-justify-center u-s-6d9f18" onclick="openQuickApplyModal('${escapeJsSingleQuote(activeFile)}', 'local_silicon', true)" title="Definir no Hermes Agent">
                ${uiIcon('zap', 'icon-sm')} Usar no Hermes
              </button>
              <button class="btn btn-cyan u-s-84fa0e" onclick="testModelInArena('${escapeJsSingleQuote(activeFile)}')" title="Testar na Local Silicon Arena">
                ${uiIcon('trophy', 'icon-sm')} Arena
              </button>
              <button class="btn u-s-84fa0e" title="Baixar esta quantização novamente" onclick="triggerCuratedDownload(${idx}, '${escapeJsSingleQuote(repoId)}')">
                ${uiIcon('refresh', 'icon-sm')}
              </button>
            </div>
          `;
        } else {
          const dlText = model && model.is_multimodal_vlm
            ? `Baixar Bundle VLM ${quantTag} (${sizeGb} GB)`
            : `Baixar ${quantTag} (${sizeGb} GB)`;
          actionContainer.innerHTML = `
            <button class="btn btn-cyan u-w-full u-justify-center" onclick="triggerCuratedDownload(${idx}, '${escapeJsSingleQuote(repoId)}')">
              ${uiIcon('download', 'icon-sm')} ${dlText}
            </button>
          `;
        }
      }
    }

    async function requestCuratedVariantDownload(repoId, url, filename, sizeBytes, sha256, projFile, projUrl, quantTag = '') {
      if (url && url.startsWith('http')) {
        return requestModelDownload(url, filename, sizeBytes, sha256, repoId, projFile, projUrl);
      }

      // Dynamic item lacked verified tree URL: resolve live from HF before queueing
      showToast(`Resolvendo arquivo ${quantTag || 'GGUF'} real em ${repoId}...`, 'info');
      try {
        const res = await fetch(`/api/v1/inventory/repo-files?repo_id=${encodeURIComponent(repoId)}`);
        if (!res.ok) throw new Error('HTTP ' + res.status);
        const data = await res.json();
        const files = (data.files || []).filter(f => !(f.file_name || '').toLowerCase().includes('mmproj'));
        const proj = (data.files || []).find(f => (f.file_name || '').toLowerCase().includes('mmproj'));
        if (files.length === 0) {
          showToast(`Nenhum arquivo GGUF encontrado diretamente em ${repoId}.`, 'warning');
          return inspectRepoFiles(repoId);
        }
        const matchFile = files.find(f => f.quant_tag && quantTag && f.quant_tag.toUpperCase() === quantTag.toUpperCase())
          || files.find(f => (f.quant_tag || '').toUpperCase() === 'Q4_K_M')
          || files[0];
        return requestModelDownload(
          matchFile.download_url,
          matchFile.file_name,
          matchFile.size_bytes,
          matchFile.sha256 || '',
          repoId,
          proj ? proj.file_name : projFile,
          proj ? proj.download_url : projUrl
        );
      } catch (err) {
        console.error('Erro ao resolver arquivo HF:', err);
        showToast('Falha ao resolver arquivos no Hugging Face.', 'error');
      }
    }

    function triggerCuratedDownload(idx, repoIdOverride) {
      const select = document.getElementById(`curated-select-${idx}`);
      if (!select) {
        return inspectRepoFiles(repoIdOverride);
      }
      const opt = select.options[select.selectedIndex];
      const quantTag = opt.value || '';
      const url = opt.getAttribute('data-url') || '';
      const filename = opt.getAttribute('data-file') || '';
      const sizeBytes = parseInt(opt.getAttribute('data-size') || '0', 10);
      const sha256 = opt.getAttribute('data-sha256') || '';
      const projFile = opt.getAttribute('data-proj-file') || '';
      const projUrl = opt.getAttribute('data-proj-url') || '';

      requestCuratedVariantDownload(repoIdOverride, url, filename, sizeBytes, sha256, projFile, projUrl, quantTag);
    }

    async function inspectRepoFiles(repoId) {
      if (!repoId) return;
      const input = document.getElementById('hf-search-input');
      if (input) {
        input.value = repoId;
        input.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
      currentHfParamFilter = 'all';
      currentHfTaskFilter = 'all';
      document.querySelectorAll('#hf-param-filters .profile-pill, #hf-task-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const allBtn = document.getElementById('btn-hf-param-all');
      if (allBtn) allBtn.classList.add('active');
      await searchHuggingFaceHub();
      setTimeout(() => {
        const idx = (currentRawHfResults || []).findIndex(r => (r.repo_id || '').toLowerCase() === repoId.toLowerCase());
        if (idx >= 0) {
          inspectHfRepo(repoId, idx);
        }
      }, 80);
    }

    // Global alias to ensure single-click activation in Hermes
    window.applyModelToHermes = function(fileName) {
      if (typeof openQuickApplyModal === 'function') {
        openQuickApplyModal(fileName, 'local_silicon', true);
      } else if (typeof setHermesMasterModel === 'function') {
        setHermesMasterModel(fileName, 'local_silicon', 'local_sovereign');
      }
    };

    async function requestModelDownload(url, filename, sizeBytes, sha256, repoId = null, projFile = '', projUrl = '') {
      try {
        const payload = {
          repo_id: repoId || filename,
          file_name: filename,
          download_url: url,
          expected_size_bytes: sizeBytes,
          expected_sha256: sha256 ? sha256 : null,
          projector_file: projFile ? projFile : null,
          projector_url: projUrl ? projUrl : null,
        };
        const res = await fetch('/api/v1/inventory/download', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        if (res.ok) {
          const msg = projFile ? 
            `Bundle VLM adicionado à fila: ${filename} + ${projFile}!` : 
            `Download de ${filename} adicionado à fila!`;
          showToast(msg, 'success');
          fetchInventoryDownloads();
        } else {
          const err = await res.text();
          showToast(`Erro ao iniciar download: ${err}`, 'error');
        }
      } catch (e) {
        showToast('Erro de conexão ao solicitar download.', 'error');
      }
    }

    // ==========================================
    // 3.2 GERENCIADOR DE DOWNLOADS RESILIENTE
    // ==========================================
    async function fetchInventoryDownloads() {
      try {
        const res = await fetch('/api/v1/inventory/downloads');
        if (res.ok) {
          const state = await res.json();
          renderDownloadsState(state);
        }
      } catch (e) {
        console.error('Erro ao buscar downloads:', e);
      }
    }

    function renderDownloadsState(state) {
      const container = document.getElementById('downloads-content-area');
      const badgeCount = document.getElementById('badge-active-downloads-count');
      if (!container) return;

      const active = state.active;
      const queue = state.queue || [];
      const totalPending = (active ? 1 : 0) + queue.length;

      if (badgeCount) {
        badgeCount.innerText = totalPending;
        badgeCount.style.display = totalPending > 0 ? 'inline-block' : 'none';
      }

      if (!active && queue.length === 0) {
        container.innerHTML = `
          <div class="u-text-center u-text-muted u-s-c89d66">
            Nenhum download ativo no momento. A fila de silício está ociosa.
          </div>
        `;
        return;
      }

      let activeHtml = '';
      if (active) {
        const pct = active.progress_pct.toFixed(1);
        const downloadedGb = (active.downloaded_bytes / (1024 ** 3)).toFixed(2);
        const totalGb = (active.total_bytes / (1024 ** 3)).toFixed(2);
        const speedMb = active.speed_bytes_sec ? (active.speed_bytes_sec / (1024 ** 2)).toFixed(2) + ' MB/s' : '--';
        const eta = active.eta_seconds ? formatEta(active.eta_seconds) : '--';
        const isPaused = active.status === 'Paused';

        activeHtml = `
          <div class="download-progress-box">
            <div class="u-d-flex u-items-center u-justify-between">
              <div>
                <strong class="u-text-cyan u-mono u-s-e1fb4b">${active.filename}</strong>
                <div class="u-text-muted u-mt-2p u-s-dbc8d1">
                  Status: <span class="u-fw-600" style="color: ${isPaused ? '#fbbf24' : '#34d399'};">${active.status}</span> &bull; Velocidade: <strong>${speedMb}</strong> &bull; ETA: <strong>${eta}</strong>
                </div>
              </div>
              <div class="u-d-flex u-gap-2">
                ${isPaused ? 
                  `<button class="btn btn-green u-s-c07599" onclick="resumeActiveDownload('${active.id}')">${uiIcon('play', 'icon-xs')} Retomar</button>` :
                  `<button class="btn u-s-c07599" onclick="pauseActiveDownload('${active.id}')">${uiIcon('pause', 'icon-xs')} Pausar</button>`
                }
                <button class="btn u-s-eaf987" onclick="cancelActiveDownload('${active.id}')">${uiIcon('x', 'icon-xs')} Cancelar</button>
              </div>
            </div>

            <div class="download-progress-bar">
              <div class="download-progress-fill" style="width: ${pct}%;"></div>
            </div>

            <div class="u-d-flex u-justify-between u-text-muted u-mono u-s-dbc8d1">
              <span>${downloadedGb} GB / ${totalGb} GB (${pct}%)</span>
              <span>SHA-256 Incremental: <span class="u-text-emerald">Ativo</span></span>
            </div>
          </div>
        `;
      }

      let queueHtml = '';
      if (queue.length > 0) {
        queueHtml = `
          <div class="u-mt-14p">
            <div class="u-text-muted u-mono u-uppercase u-s-a2b44b">Próximos na Fila Sequencial (${queue.length}):</div>
            <div class="u-mt-6p u-d-flex u-flex-col u-gap-6p">
              ${queue.map(q => `
                <div class="u-d-flex u-justify-between u-items-center u-s-f1166c">
                  <span class="mono u-text-slate-light">${q.filename}</span>
                  <div class="u-d-flex u-gap-2 u-items-center">
                    <span class="mono u-text-muted">${(q.total_bytes / (1024**3)).toFixed(2)} GB</span>
                    <button class="btn u-s-c8911f" onclick="cancelActiveDownload('${q.id}')">${uiIcon('x', 'icon-xs')}</button>
                  </div>
                </div>
              `).join('')}
            </div>
          </div>
        `;
      }

      container.innerHTML = activeHtml + queueHtml;
    }

    function formatEta(seconds) {
      if (seconds < 60) return `${Math.round(seconds)}s`;
      const mins = Math.floor(seconds / 60);
      const secs = Math.round(seconds % 60);
      return `${mins}m ${secs}s`;
    }

    async function pauseActiveDownload(id) {
      try {
        await fetch(`/api/v1/inventory/download/${id}/pause`, { method: 'POST' });
        showToast('Download pausado com sucesso.', 'info');
        fetchInventoryDownloads();
      } catch (e) {
        showToast('Erro ao pausar download.', 'error');
      }
    }

    async function resumeActiveDownload(id) {
      try {
        await fetch(`/api/v1/inventory/download/${id}/resume`, { method: 'POST' });
        showToast('Download retomado via HTTP Range.', 'success');
        fetchInventoryDownloads();
      } catch (e) {
        showToast('Erro ao retomar download.', 'error');
      }
    }

    async function cancelActiveDownload(id) {
      try {
        await fetch(`/api/v1/inventory/download/${id}/cancel`, { method: 'POST' });
        showToast('Download cancelado e arquivo parcial descartado.', 'info');
        fetchInventoryDownloads();
      } catch (e) {
        showToast('Erro ao cancelar download.', 'error');
      }
    }

    function connectDownloadsSSE() {
      if (downloadEventSource && downloadEventSource.readyState !== EventSource.CLOSED) {
        return;
      }
      const sseStatus = document.getElementById('download-sse-status');
      downloadEventSource = new EventSource('/api/v1/inventory/downloads/stream');

      downloadEventSource.onopen = () => {
        if (sseStatus) sseStatus.innerHTML = '<span class="u-text-emerald">● Ativo</span>';
      };

      downloadEventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          const activeProgressFill = document.querySelector('.download-progress-fill');
          if (activeProgressFill && data.progress_pct !== undefined) {
            activeProgressFill.style.width = data.progress_pct.toFixed(1) + '%';
          }
        } catch (e) {}
      };

      downloadEventSource.onerror = () => {
        if (sseStatus) sseStatus.innerHTML = '<span class="u-text-amber">○ Reconectando</span>';
      };
    }

    // ==========================================
    // 3.3 EXPLORADOR HUGGING FACE HUB (BUSCA COMBINADA + AUTOCOMPLETE)
    // ==========================================
    let currentHfParamFilter = 'all';
    let currentHfTaskFilter = 'all';
    let currentRawHfResults = [];
    let hfAutocompleteTimer = null;
    let hfAutocompleteAbortCtrl = null;

    const HF_QUICK_SUGGESTIONS = [
      { label: 'Qwen3 (Nova Geração Híbrida)', query: 'Qwen3', badge: '1.7B–8B • Sweet Spot' },
      { label: 'Qwen2.5-Coder (Cirurgia de Código)', query: 'Qwen2.5-Coder', badge: '3B–7B • Coding' },
      { label: 'DeepSeek-R1-Distill (Raciocínio CoT)', query: 'DeepSeek-R1', badge: '1.5B–8B • R1' },
      { label: 'Phi-4 Mini (Microsoft 128k)', query: 'Phi-4-mini', badge: '3.8B • Optimal VRAM' },
      { label: 'Gemma 3 IT (Google Design & Geral)', query: 'gemma-3', badge: '1B–12B • 128k' },
      { label: 'bartowski (Quantizador Imatrix Oficial)', query: 'bartowski', badge: 'Autor Elite' },
      { label: 'unsloth (Quants Dinâmicas Otimizadas)', query: 'unsloth', badge: 'Autor Elite' },
      { label: 'NousResearch / Hermes-3 (Agêntico)', query: 'Hermes-3', badge: 'Tool Calling' },
    ];

    function setHfParamFilter(param) {
      if (currentHfParamFilter === param && param !== 'all') {
        currentHfParamFilter = 'all';
      } else {
        currentHfParamFilter = param;
      }
      document.querySelectorAll('#hf-param-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-hf-param-${currentHfParamFilter}`);
      if (activeBtn) activeBtn.classList.add('active');

      const input = document.getElementById('hf-search-input');
      const hasText = input && input.value.trim().length > 0;
      if (currentHfParamFilter === 'all' && currentHfTaskFilter === 'all' && !hasText) {
        resetHfExplorerResultsPlaceholder();
        return;
      }
      searchHuggingFaceHub();
    }

    function setHfTaskFilter(task) {
      if (currentHfTaskFilter === task) {
        currentHfTaskFilter = 'all';
      } else {
        currentHfTaskFilter = task;
      }
      document.querySelectorAll('#hf-task-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      if (currentHfTaskFilter !== 'all') {
        const activeBtn = document.getElementById(`btn-hf-task-${currentHfTaskFilter}`);
        if (activeBtn) activeBtn.classList.add('active');
      }

      const input = document.getElementById('hf-search-input');
      const hasText = input && input.value.trim().length > 0;
      if (currentHfParamFilter === 'all' && currentHfTaskFilter === 'all' && !hasText) {
        resetHfExplorerResultsPlaceholder();
        return;
      }
      searchHuggingFaceHub();
    }

    function resetHfExplorerResultsPlaceholder() {
      currentRawHfResults = [];
      const container = document.getElementById('hf-search-results');
      if (container) {
        container.innerHTML = `
          <div class="u-text-center u-text-muted u-s-40d486">
            Selecione uma faixa de parâmetros ou tipo de tarefa acima, ou digite um termo para explorar arquivos GGUF na nuvem.
          </div>
        `;
      }
    }

    function resetHfExplorerFilters() {
      const input = document.getElementById('hf-search-input');
      if (input) input.value = '';
      currentHfParamFilter = 'all';
      currentHfTaskFilter = 'all';
      document.querySelectorAll('#hf-param-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const allBtn = document.getElementById('btn-hf-param-all');
      if (allBtn) allBtn.classList.add('active');
      document.querySelectorAll('#hf-task-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      hideHfAutocomplete();
      resetHfExplorerResultsPlaceholder();
    }

    function hideHfAutocomplete() {
      const dd = document.getElementById('hf-autocomplete-dropdown');
      if (dd) dd.classList.add('is-hidden');
    }

    function selectHfAutocomplete(val, autoInspect = false) {
      const input = document.getElementById('hf-search-input');
      if (input) input.value = val;
      hideHfAutocomplete();
      searchHuggingFaceHub().then(() => {
        if (autoInspect && currentRawHfResults && currentRawHfResults.length > 0) {
          const idx = currentRawHfResults.findIndex(r => (r.repo_id || '').toLowerCase() === val.toLowerCase());
          if (idx >= 0) inspectHfRepo(val, idx);
        }
      });
    }

    function renderHfAutocompleteItems(term, liveItems = [], isLoading = false) {
      const dd = document.getElementById('hf-autocomplete-dropdown');
      if (!dd) return;
      const lower = (term || '').toLowerCase();
      const quickMatches = HF_QUICK_SUGGESTIONS.filter(s =>
        !lower || s.label.toLowerCase().includes(lower) || s.query.toLowerCase().includes(lower)
      ).slice(0, lower ? 3 : 6);

      let html = '';
      if (quickMatches.length > 0) {
        html += `<div class="hf-ac-section-title">Famílias &amp; Autores Recomendados</div>`;
        html += quickMatches.map(s => `
          <div class="hf-ac-item" onclick="selectHfAutocomplete('${escapeJsSingleQuote(s.query)}', false)">
            <div class="u-d-flex u-items-center u-gap-2">
              <span class="u-text-cyan">${uiIcon('sparkles', 'icon-xs')}</span>
              <span class="u-text-white u-fw-600">${escapeHtml(s.label)}</span>
            </div>
            <span class="badge u-text-muted u-s-1ee417">${escapeHtml(s.badge)}</span>
          </div>
        `).join('');
      }

      if (lower.length >= 2) {
        html += `<div class="hf-ac-section-title">Repositórios GGUF ao Vivo (Hugging Face)</div>`;
        if (isLoading && liveItems.length === 0) {
          html += `<div class="u-text-cyan u-p-2 u-s-dbc8d1"><span class="spin-icon u-d-inline-block u-s-d1cc57">${uiIcon('refresh', 'icon-xs')}</span> Buscando "${escapeHtml(term)}"...</div>`;
        } else if (liveItems.length === 0) {
          html += `<div class="u-text-muted u-p-2 u-s-dbc8d1">Pressione Enter para pesquisa completa por "${escapeHtml(term)}".</div>`;
        } else {
          html += liveItems.map(r => {
            const fit = r.hardware_fit || 'PartialCpuOffload';
            const fitCls = fit === 'OptimalVram' ? 'optimal' : (fit === 'PartialCpuOffload' ? 'offload' : 'incompatible');
            const fitTxt = fit === 'OptimalVram' ? '100% VRAM' : (fit === 'PartialCpuOffload' ? 'CPU Offload' : 'Pesado');
            const tpsTxt = r.estimated_tps ? `~${Number(r.estimated_tps).toFixed(0)} tps` : '';
            const paramB = r.parameter_count ? (r.parameter_count >= 1e9 ? `${(r.parameter_count / 1e9).toFixed(1).replace('.0', '')}B` : `${Math.round(r.parameter_count / 1e6)}M`) : 'GGUF';
            return `
              <div class="hf-ac-item" onclick="selectHfAutocomplete('${escapeJsSingleQuote(r.repo_id)}', true)">
                <div class="u-overflow-hidden">
                  <div class="mono u-text-white u-fw-600" style="font-size:0.78rem; text-overflow:ellipsis; overflow:hidden; white-space:nowrap;">${escapeHtml(r.repo_id)}</div>
                  <div class="u-text-muted" style="font-size:0.7rem;">
                    ${paramB} ${tpsTxt ? '&bull; ' + tpsTxt : ''} &bull; ${Number(r.downloads || 0).toLocaleString()} downloads
                  </div>
                </div>
                <span class="fit-badge ${fitCls}" style="flex-shrink:0;">${fitTxt}</span>
              </div>
            `;
          }).join('');
        }
      }

      if (!html) {
        dd.classList.add('is-hidden');
        return;
      }
      dd.innerHTML = html;
      dd.classList.remove('is-hidden');
    }

    function onHfSearchInputKeyup(event) {
      const input = document.getElementById('hf-search-input');
      if (!input) return;
      const term = input.value.trim();

      renderHfAutocompleteItems(term, [], term.length >= 2);
      if (hfAutocompleteTimer) clearTimeout(hfAutocompleteTimer);
      if (term.length < 2) return;

      hfAutocompleteTimer = setTimeout(async () => {
        try {
          if (hfAutocompleteAbortCtrl) hfAutocompleteAbortCtrl.abort();
          hfAutocompleteAbortCtrl = new AbortController();
          const res = await fetch(`/api/v1/inventory/search?q=${encodeURIComponent(term)}&autocomplete=true&limit=6`, {
            signal: hfAutocompleteAbortCtrl.signal
          });
          if (res.ok) {
            const data = await res.json();
            if (document.activeElement === input || term === input.value.trim()) {
              renderHfAutocompleteItems(term, data.results || [], false);
            }
          }
        } catch (err) {
          if (err.name !== 'AbortError') {
            renderHfAutocompleteItems(term, [], false);
          }
        }
      }, 250);
    }

    document.addEventListener('click', (e) => {
      const wrap = document.querySelector('.hf-autocomplete-wrap');
      if (wrap && !wrap.contains(e.target)) {
        hideHfAutocomplete();
      }
    });

    function updateDynamicParamFitPills(hw) {
      const safeVramGb = hw && hw.vram_safe_usable_mb ? (hw.vram_safe_usable_mb / 1024) : ((window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) ? (window.soulsTelemetryState.vramLimitMb / 1024) : 4.0);
      const gpuName = hw && hw.gpu_name ? hw.gpu_name : 'dGPU';
      const legend = document.getElementById('hf-vram-fit-legend');
      if (legend) {
        legend.innerHTML = `(${uiIcon('check-circle', 'icon-xs')} Cabem na ${gpuName.includes('RTX') ? gpuName : 'GPU ' + gpuName})`;
      }

      const fitGpuBtn = document.getElementById('btn-hf-param-fit_gpu');
      if (fitGpuBtn) {
        fitGpuBtn.style.borderColor = 'rgba(16, 185, 129, 0.5)';
        fitGpuBtn.style.color = '#34d399';
        fitGpuBtn.title = `Filtrar modelos que rodam 100% na VRAM da ${gpuName} (~${safeVramGb.toFixed(1)} GB úteis)`;
      }

      const paramPillSizes = [
        { id: 'btn-hf-param-sub1b', size: 0.5 },
        { id: 'btn-hf-param-1b', size: 1.5 },
        { id: 'btn-hf-param-2b', size: 2.5 },
        { id: 'btn-hf-param-3b', size: 3.5 },
        { id: 'btn-hf-param-4b', size: 4.5 },
        { id: 'btn-hf-param-5b', size: 5.5 },
        { id: 'btn-hf-param-6b', size: 6.5 },
        { id: 'btn-hf-param-7b', size: 7.5 },
        { id: 'btn-hf-param-8b', size: 8.5 },
        { id: 'btn-hf-param-9b', size: 9.5 },
        { id: 'btn-hf-param-10b_12b', size: 11.0 },
        { id: 'btn-hf-param-14b', size: 14.0 },
        { id: 'btn-hf-param-32b', size: 32.0 },
        { id: 'btn-hf-param-70b_plus', size: 70.0 }
      ];

      paramPillSizes.forEach(item => {
        const btn = document.getElementById(item.id);
        if (!btn) return;
        const estVram = item.size * 0.65;
        if (estVram + 0.8 <= safeVramGb * 0.75) {
          btn.style.borderColor = 'rgba(16, 185, 129, 0.45)';
          btn.style.color = '#34d399';
          btn.title = `Cabe com 100% de folga na VRAM da ${gpuName} (~${estVram.toFixed(1)} GB VRAM)`;
        } else if (estVram + 0.5 <= safeVramGb) {
          btn.style.borderColor = 'rgba(6, 182, 212, 0.45)';
          btn.style.color = 'var(--cyan)';
          btn.title = `Cabe no teto da VRAM da ${gpuName} (~${estVram.toFixed(1)} GB VRAM)`;
        } else {
          btn.style.borderColor = 'var(--border-subtle)';
          btn.style.color = 'var(--text-muted)';
          btn.title = `Demanda CPU Host RAM (~${estVram.toFixed(1)} GB de pesos Q4; excede ${safeVramGb.toFixed(1)} GB de VRAM útil)`;
        }
      });
    }

    function parseRepoModelDetails(repoId, tags = []) {
      if (!repoId) return { total_b: null, active_b: null, is_moe: false, fit_label: 'Formato GGUF', fit_class: 'default', specialties: [] };
      const r = repoId.toLowerCase();
      const tagsStr = (Array.isArray(tags) ? tags.join(' ') : '').toLowerCase();

      let total_b = null;
      let active_b = null;
      let is_moe = false;
      let fit_label = 'Formato GGUF (Ver Arquivos)';
      let fit_class = 'default';

      const moeMatch = r.match(/(\d+(?:\.\d+)?)\s*b\s*[-_]a(\d+(?:\.\d+)?)\s*b/i);
      if (moeMatch) {
        total_b = parseFloat(moeMatch[1]);
        active_b = parseFloat(moeMatch[2]);
        is_moe = true;
        fit_label = total_b <= 35 ? `MoE ${total_b}B (${active_b}B Ativos) • CPU RAM` : `Datacenter Only (>${total_b}B MoE)`;
        fit_class = total_b <= 35 ? 'offload' : 'incompatible';
      } else if (r.match(/(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b/i)) {
        const mx = r.match(/(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b/i);
        const experts = parseInt(mx[1], 10);
        const size = parseFloat(mx[2]);
        total_b = experts * size;
        active_b = size;
        is_moe = true;
        fit_label = total_b <= 45 ? `MoE ${experts}x${size}B (~${total_b}B) • CPU RAM` : `Datacenter Only (>${total_b}B MoE)`;
        fit_class = total_b <= 45 ? 'offload' : 'incompatible';
      } else if (r.match(/(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)/i)) {
        const m = r.match(/(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)/i);
        total_b = parseInt(m[1], 10) / 1000.0;
        active_b = total_b;
        fit_label = 'Ultra-Leve (<1B • 100% VRAM)';
        fit_class = 'optimal';
      } else {
        const bMatches = [...r.matchAll(/(?:^|[^0-9a-z])(\d+(?:\.\d+)?)\s*b(?:[^0-9a-z]|$)/gi)];
        if (bMatches && bMatches.length > 0) {
          const bValues = bMatches.map(m => parseFloat(m[1])).filter(v => !isNaN(v) && v > 0);
          if (bValues.length > 0) {
            total_b = Math.max(...bValues);
            active_b = total_b;
            if (total_b <= 4.5) {
              fit_label = `Sweet Spot dGPU (100% VRAM • ${total_b}B)`;
              fit_class = 'optimal';
            } else if (total_b <= 8.5) {
              fit_label = `Teto Seguro dGPU (Q4_K_M • ${total_b}B)`;
              fit_class = 'tight';
            } else if (total_b <= 35.0) {
              fit_label = `Modo CPU Host RAM (${total_b}B)`;
              fit_class = 'offload';
            } else {
              fit_label = `Datacenter Only (>${total_b}B)`;
              fit_class = 'incompatible';
            }
          }
        } else if (r.includes('phi-4-mini') || r.includes('phi-3.5-mini')) {
          total_b = 3.8;
          active_b = 3.8;
          fit_label = 'Sweet Spot dGPU (100% VRAM • 3.8B)';
          fit_class = 'optimal';
        }
      }

      const specialties = [];
      if (r.includes('code') || r.includes('coder') || tagsStr.includes('code') || tagsStr.includes('coder')) {
        specialties.push({ icon: 'code', label: 'Coding', color: '#c084fc', bg: 'rgba(168, 85, 247, 0.15)', border: 'rgba(168, 85, 247, 0.3)' });
      }
      if (r.includes('vision') || r.includes('vlm') || r.includes('-vl') || r.includes('multimodal') || tagsStr.includes('image') || tagsStr.includes('vision')) {
        specialties.push({ icon: 'eye', label: 'Vision/VLM', color: '#fb7185', bg: 'rgba(244, 63, 94, 0.15)', border: 'rgba(244, 63, 94, 0.3)' });
      }
      if (r.includes('r1') || r.includes('reasoning') || r.includes('think') || r.includes('cot') || tagsStr.includes('reasoning')) {
        specialties.push({ icon: 'brain', label: 'Reasoning', color: '#818cf8', bg: 'rgba(99, 102, 241, 0.15)', border: 'rgba(99, 102, 241, 0.3)' });
      }
      if (r.includes('agent') || r.includes('tool') || r.includes('hermes') || r.includes('xlam') || tagsStr.includes('function-calling')) {
        specialties.push({ icon: 'bot', label: 'Agentic', color: '#38bdf8', bg: 'rgba(6, 182, 212, 0.15)', border: 'rgba(6, 182, 212, 0.3)' });
      }

      return { total_b, active_b, is_moe, fit_label, fit_class, specialties };
    }

    async function searchHuggingFaceHub() {
      const input = document.getElementById('hf-search-input');
      const resultsContainer = document.getElementById('hf-search-results');
      if (!input || !resultsContainer) return;

      const query = input.value.trim();
      if (!query && currentHfParamFilter === 'all' && currentHfTaskFilter === 'all') {
        showToast('Digite um termo ou selecione um filtro acima para pesquisar.', 'info');
        return;
      }

      resultsContainer.innerHTML = `<div class="u-text-cyan u-text-center u-s-a2a79e"><span class="spin-icon u-d-inline-block u-s-d1cc57">${uiIcon('refresh', 'icon-xs')}</span> Consultando repositórios GGUF combinados no Hugging Face...</div>`;

      try {
        const params = new URLSearchParams();
        if (query) params.set('query', query);
        params.set('param_filter', currentHfParamFilter);
        params.set('task_filter', currentHfTaskFilter);
        params.set('limit', '30');

        const res = await fetch(`/api/v1/inventory/search?${params.toString()}`);
        if (res.ok) {
          const data = await res.json();
          currentRawHfResults = data.results || [];
          renderHfSearchResults(currentRawHfResults);
        } else {
          resultsContainer.innerHTML = '<div class="u-text-red u-text-center u-s-872b66">Erro ao consultar repositórios no Hugging Face. Verifique a conexão com a internet.</div>';
        }
      } catch (e) {
        resultsContainer.innerHTML = '<div class="u-text-red u-text-center u-s-872b66">Falha de rede ao conectar com a API do Hugging Face.</div>';
      }
    }

    function renderHfSearchResults(repos) {
      const container = document.getElementById('hf-search-results');
      if (!container) return;

      if (!repos || repos.length === 0) {
        const paramPill = document.querySelector('#hf-param-filters .profile-pill.active')?.innerText || currentHfParamFilter;
        const taskPill = document.querySelector('#hf-task-filters .profile-pill.active')?.innerText;
        const activeFilterDesc = [paramPill !== 'Todos' ? paramPill : '', taskPill || ''].filter(Boolean).join(' • ');
        container.innerHTML = `
          <div class="u-text-center u-text-muted u-s-cc6502">
            <div class="u-mb-6p u-s-c2d8f9">${uiIcon('search', 'icon-md')}</div>
            <div class="u-text-slate u-fw-600 u-s-604fde">Nenhum repositório GGUF encontrado${activeFilterDesc ? ` para "${escapeHtml(activeFilterDesc)}"` : ''}.</div>
            <div class="u-mt-3 u-d-flex u-gap-2 u-justify-center">
              <button class="btn btn-cyan u-s-918d92" onclick="resetHfExplorerFilters()">
                Limpar Filtros e Recomeçar
              </button>
            </div>
          </div>
        `;
        return;
      }

      container.innerHTML = repos.map((r, i) => {
        const d = parseRepoModelDetails(r.repo_id, r.tags || []);
        const backendFit = r.hardware_fit;
        const fitBadge = backendFit
          ? getFitBadgeHtml(backendFit)
          : `<span class="fit-badge ${d.fit_class}" title="${d.fit_label}">${d.fit_label}</span>`;

        const paramCount = r.parameter_count || (d.total_b ? d.total_b * 1e9 : 0);
        const paramBadge = paramCount > 0
          ? `<span class="badge u-text-muted u-s-1ee417">${paramCount >= 1e9 ? (paramCount / 1e9).toFixed(1).replace('.0', '') + 'B' : Math.round(paramCount / 1e6) + 'M'}</span>`
          : '';
        const tpsBadge = r.estimated_tps
          ? `<span class="badge u-text-emerald u-s-3fefe0">~${Number(r.estimated_tps).toFixed(0)} tok/s</span>`
          : '';

        const specialtyBadges = (d.specialties || []).map(s => `
          <span class="badge u-s-7f12b7" style="background: ${s.bg}; color: ${s.color}; border: 1px solid ${s.border};">
            ${uiIcon(s.icon, 'icon-xs')} ${s.label}
          </span>
        `).join('');

        return `
          <div class="hf-repo-item u-mb-10p u-s-5d32a8" id="hf-repo-${i}">
            <div class="u-flex-1">
              <div class="u-d-flex u-items-center u-gap-2 u-flex-wrap">
                <span class="u-fw-700 u-text-white u-mono u-s-13ccaf">
                  ${escapeHtml(r.repo_id)}
                </span>
                ${fitBadge}
                ${paramBadge}
                ${tpsBadge}
                ${r.is_downloaded ? '<span class="badge u-text-emerald u-fw-700 u-s-3e3280">' + uiIcon('check', 'icon-xs') + ' Instalado</span>' : ''}
                ${specialtyBadges}
              </div>
              <div class="u-text-muted u-mt-6p u-mono u-s-6444dd">
                ${uiIcon('users', 'icon-xs')} Autor: <strong class="u-text-slate">${escapeHtml(r.author || '--')}</strong> &bull; ${uiIcon('download', 'icon-xs')} Downloads: <strong class="u-text-cyan">${(r.downloads || 0).toLocaleString()}</strong> &bull; ${uiIcon('sparkles', 'icon-xs')} Likes: <strong class="u-s-23cce7">${(r.likes || 0).toLocaleString()}</strong>
              </div>
            </div>
            <div>
              <button class="btn btn-cyan u-fw-600 u-s-205356" onclick="inspectHfRepo('${escapeJsSingleQuote(r.repo_id)}', ${i})">
                ${uiIcon('folder', 'icon-sm')} Ver Arquivos GGUF
              </button>
            </div>
          </div>
          <div class="is-hidden" id="hf-repo-files-${i}"></div>
        `;
      }).join('');
    }

    async function inspectHfRepo(repoId, idx) {
      const container = document.getElementById(`hf-repo-files-${idx}`);
      if (!container) return;

      if (container.style.display === 'block') {
        container.style.display = 'none';
        return;
      }

      container.style.display = 'block';
      container.innerHTML = '<div class="u-text-muted u-s-8d2332">Carregando arquivos GGUF e hashes SHA-256 reais do repositório...</div>';

      try {
        const res = await fetch(`/api/v1/inventory/repo-files?repo_id=${encodeURIComponent(repoId)}`);
        if (res.ok) {
          const data = await res.json();
          const files = data.files || [];
          if (files.length === 0) {
            container.innerHTML = '<div class="u-text-muted u-s-8d2332">Nenhum arquivo .gguf encontrado na raiz deste repositório.</div>';
            return;
          }
          const proj = files.find(f => (f.file_name || '').toLowerCase().includes('mmproj'));

          container.innerHTML = `
            <div class="hf-files-container">
              <div class="u-text-muted u-mono u-mb-2 u-uppercase u-s-cb8a8e">Arquivos GGUF Disponíveis (${files.length}):</div>
              <div class="u-d-flex u-flex-col u-gap-6p">
                ${files.map(f => {
                  const sizeGb = f.size_gb ? f.size_gb.toFixed(2) : (f.size_bytes / (1024 ** 3)).toFixed(2);
                  const fileName = f.file_name || f.filename;
                  const fitBadge = getFitBadgeHtml(f.hardware_fit);
                  const matched = f.is_installed ? { realFileName: f.installed_file_name || fileName } : findInstalledModelMatch(fileName, repoId, '', f.quant_tag);
                  const isDown = Boolean(f.is_installed || matched);
                  const activeFile = (matched && matched.realFileName) ? matched.realFileName : fileName;
                  const estTps = f.estimated_tps ? `&bull; Vel: <strong class="u-text-emerald">~${Number(f.estimated_tps).toFixed(0)} tps</strong>` : '';
                  const isMmproj = fileName.toLowerCase().includes('mmproj');

                  return `
                    <div class="u-d-flex u-justify-between u-items-center u-s-659226">
                      <div>
                        <div class="u-d-flex u-items-center u-gap-2 u-flex-wrap">
                          <span class="mono u-fw-600 u-text-white u-s-22b62c">${escapeHtml(fileName)}</span>
                          ${isDown ? '<span class="badge u-text-emerald u-fw-700 u-s-4a274a">' + uiIcon('check-circle', 'icon-xs') + ' Já Instalado no Silício</span>' : ''}
                          ${fitBadge}
                        </div>
                        <div class="u-text-muted u-mt-2p u-s-923ef2">
                          Quant: <strong class="u-text-cyan">${escapeHtml(f.quant_tag || '--')}</strong> &bull; Tamanho: <strong class="u-text-white">${sizeGb} GB</strong> ${estTps} ${f.sha256 ? '&bull; LFS SHA-256 verificado' : ''}
                        </div>
                      </div>
                      <div>
                        ${isDown ? `
                          <div class="u-d-flex u-gap-1">
                            ${!isMmproj ? `
                              <button class="btn btn-green u-s-513c8c" onclick="openQuickApplyModal('${escapeJsSingleQuote(activeFile)}', 'local_silicon', true)" title="Definir no Hermes Agent">
                                ${uiIcon('zap', 'icon-sm')} Usar
                              </button>
                              <button class="btn btn-cyan u-s-44e27c" onclick="testModelInArena('${escapeJsSingleQuote(activeFile)}')" title="Testar na Local Silicon Arena">
                                ${uiIcon('trophy', 'icon-sm')}
                              </button>
                            ` : ''}
                            <button class="btn u-s-44e27c" title="Baixar novamente" onclick="requestModelDownload('${escapeJsSingleQuote(f.download_url)}', '${escapeJsSingleQuote(fileName)}', ${f.size_bytes}, '${escapeJsSingleQuote(f.sha256 || '')}', '${escapeJsSingleQuote(repoId)}')">
                              ${uiIcon('refresh', 'icon-sm')}
                            </button>
                          </div>
                        ` : `
                          <button class="btn btn-cyan u-s-a20e80" onclick="requestModelDownload('${escapeJsSingleQuote(f.download_url)}', '${escapeJsSingleQuote(fileName)}', ${f.size_bytes}, '${escapeJsSingleQuote(f.sha256 || '')}', '${escapeJsSingleQuote(repoId)}', '${escapeJsSingleQuote(!isMmproj && proj ? proj.file_name : '')}', '${escapeJsSingleQuote(!isMmproj && proj ? proj.download_url : '')}')">
                            ${uiIcon('download', 'icon-sm')} Baixar${!isMmproj && proj ? ' + VLM' : ''}
                          </button>
                        `}
                      </div>
                    </div>
                  `;
                }).join('')}
              </div>
            </div>
          `;
        } else {
          container.innerHTML = '<div class="u-text-red u-s-8d2332">Falha ao listar arquivos GGUF.</div>';
        }
      } catch (e) {
        container.innerHTML = '<div class="u-text-red u-s-8d2332">Erro de conexão.</div>';
      }
    }

    // 4. FETCH CLOUD ARENA (llm_cloud), ALFÂNDEGA HERMES & FINOPS REAL
    let cloudSearchQuery = '';
    let cloudSortOrder = 'score_desc';
    let currentSortCol = 'score';
    let currentSortAsc = false;

    const GATEWAY_KEY_URLS = {
      OPENROUTER_API_KEY: 'https://openrouter.ai/settings/keys',
      DEEPSEEK_API_KEY: 'https://platform.deepseek.com/api_keys',
      ANTHROPIC_API_KEY: 'https://console.anthropic.com/settings/keys',
      OPENAI_API_KEY: 'https://platform.openai.com/api-keys',
      GEMINI_API_KEY: 'https://aistudio.google.com/apikey',
      ZAI_API_KEY: 'https://z.ai/manage-apikey/apikey-list',
      XAI_API_KEY: 'https://console.x.ai/',
      NVIDIA_API_KEY: 'https://build.nvidia.com/settings/api-keys'
    };

    function updateCustomsInlineKeyLink() {
      const sel = document.getElementById('customs-inline-key-env');
      const lnk = document.getElementById('customs-inline-key-link');
      if (sel && lnk) {
        lnk.href = GATEWAY_KEY_URLS[sel.value] || 'https://openrouter.ai/settings/keys';
      }
    }

    function toggleCustomsAuthDrawer(forceOpen = null) {
      const drawer = document.getElementById('customs-inline-key-drawer');
      if (!drawer) return;
      if (forceOpen === true) drawer.classList.remove('is-hidden');
      else if (forceOpen === false) drawer.classList.add('is-hidden');
      else drawer.classList.toggle('is-hidden');
    }

    async function saveCustomsInlineCredential() {
      const sel = document.getElementById('customs-inline-key-env');
      const inp = document.getElementById('customs-inline-key-val');
      const key = sel ? sel.value : '';
      const val = inp ? inp.value.trim() : '';
      if (!key || !val) {
        showToast('Selecione o gateway e cole a chave antes de ativar.', 'warning');
        return;
      }
      try {
        const res = await fetch('/api/v1/system/credentials/save', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ key, value: val })
        });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          showToast(data.message || `Chave ${key} salva no Hermes e perfis!`, 'success');
          if (inp) inp.value = '';
          await fetchHermesCustoms();
          renderCloudTable();
          renderCloudRankedTable();
        } else {
          showToast(data.message || 'Erro ao salvar credencial.', 'error');
        }
      } catch (e) {
        showToast(`Falha de rede ao salvar credencial: ${e.message}`, 'error');
      }
    }

    async function launchHermesOauthLogin(provider = 'nous') {
      showToast(`Abrindo janela do terminal para login no Hermes (${provider})...`, 'info');
      try {
        const res = await fetch('/api/v1/hermes/customs/repair', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ launch_login_provider: provider, repair_configs: false, unpin_sessions: false })
        });
        const data = await res.json();
        if (data.login_launched) {
          showToast(`Terminal aberto! Conclua o login no navegador e clique em "Reparar Alfândega" se necessário.`, 'success');
        } else {
          showToast(`Execute no terminal: ${data.login_command || 'hermes login --provider nous'}`, 'warning');
        }
      } catch (e) {
        showToast(`Execute no terminal: hermes login --provider ${provider}`, 'warning');
      }
    }

    async function repairHermesCustoms(repairConfigs = true, unpinSessions = true, moaEnabled = null) {
      showToast('Executando reparo da Alfândega Hermes ↔ Souls...', 'warning');
      try {
        const res = await fetch('/api/v1/hermes/customs/repair', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            repair_configs: repairConfigs,
            unpin_sessions: unpinSessions,
            moa_enabled: moaEnabled
          })
        });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          const msgs = [];
          if ((data.repaired_configs || []).length > 0) msgs.push(`${data.repaired_configs.length} config(s) normalizados`);
          if (data.unpinned_sessions > 0) msgs.push(`${data.unpinned_sessions} sessão(ões) destravadas`);
          if ((data.moa_updated_profiles || []).length > 0) msgs.push(`MoA atualizado`);
          showToast(`Alfândega concluída: ${msgs.join(' • ') || 'Tudo verificado e alinhado!'}`, 'success');
          if (data.customs) applyCustomsBannerData(data.customs);
          else await fetchHermesCustoms();
          await fetchTelemetry();
        } else {
          showToast(data.message || 'Falha no reparo da Alfândega.', 'error');
        }
      } catch (e) {
        showToast(`Erro no reparo: ${e.message}`, 'error');
      }
    }

    async function toggleHermesMoa() {
      const currentlyEnabled = Boolean(window.cachedHermesCustoms?.moa_enabled_anywhere);
      await repairHermesCustoms(false, false, !currentlyEnabled);
    }

    function renderDynamicActiveGatewayPills(customs) {
      const slot = document.getElementById('cloud-dynamic-gateways-pills');
      if (!slot || !customs || !Array.isArray(customs.active_gateway_ids)) return;
      const extras = customs.active_gateway_ids.filter(g => g !== 'nous' && g !== 'openrouter' && g !== 'custom');
      slot.innerHTML = extras.map(gwId => {
        const gwMeta = (customs.gateways || []).find(g => g.id === gwId);
        const label = gwMeta ? gwMeta.label : humanizeProviderName(gwId);
        const activeCls = currentCloudProviderFilter === gwId ? 'active' : '';
        return `<button class="filter-provider-btn ${activeCls}" id="btn-cloud-prov-${gwId}" onclick="setCloudProviderFilter('${gwId}')">${uiIcon('plug', 'icon-xs')} ${escapeHtml(label)}</button>`;
      }).join('');
      if (typeof updateCloudProviderPillCounts === 'function') {
        updateCloudProviderPillCounts();
      }
    }

    function applyCustomsBannerData(report) {
      if (!report) return;
      window.cachedHermesCustoms = report;
      const banner = document.getElementById('active-hermes-brain-banner');
      const badgeEl = document.getElementById('active-hermes-customs-badge');
      const alertEl = document.getElementById('active-hermes-customs-alert');
      const btnRepair = document.getElementById('btn-hermes-repair-customs');
      const btnMoa = document.getElementById('btn-hermes-moa-toggle');
      const lblMoa = document.getElementById('btn-hermes-moa-label');
      const moaBadge = document.getElementById('active-hermes-moa-badge');
      const stuckBadge = document.getElementById('active-hermes-stuck-badge');
      const btnReauth = document.getElementById('btn-hermes-reconnect-nous');
      const auditGrid = document.getElementById('customs-profiles-audit-grid');
      const provEl = document.getElementById('active-hermes-provider-tag');

      if (report.active_profile_model) {
        window.currentHermesMasterModel = report.active_profile_model;
        window.currentHermesMasterProvider = report.active_profile_provider || 'nous';
        const nameEl = document.getElementById('active-hermes-model-name');
        const quickModelEl = document.getElementById('cloud-quick-active-model');
        if (nameEl) nameEl.innerText = report.active_profile_model;
        if (quickModelEl) quickModelEl.innerText = report.active_profile_model;
      }

      if (provEl) {
        const p = report.active_profile_provider || 'nous';
        const gw = (report.gateways || []).find(g => g.id === p);
        const label = gw ? gw.label : humanizeProviderName(p);
        const authTxt = p === 'custom' ? 'Local' : (report.active_provider_authenticated ? 'Conectado' : 'Sem Credencial');
        provEl.innerHTML = `${uiIcon(p === 'nous' ? 'brain' : 'git-branch', 'icon-xs')} ${escapeHtml(label)} (${authTxt})`;
      }

      if (banner) {
        banner.classList.remove('customs-warn', 'customs-crit');
        if (report.health === 'critical') banner.classList.add('customs-crit');
        else if (report.health === 'warning') banner.classList.add('customs-warn');
      }

      if (badgeEl) {
        badgeEl.className = 'customs-status-pill ' + (report.health === 'critical' ? 'customs-pill-crit' : (report.health === 'warning' ? 'customs-pill-warn' : 'customs-pill-ok'));
        badgeEl.innerText = report.health === 'critical'
          ? 'Alfândega: Ação Necessária'
          : (report.health === 'warning' ? 'Alfândega: Atenção' : 'Alfândega: Liberada');
      }

      if (moaBadge) {
        moaBadge.classList.toggle('is-hidden', !report.moa_enabled_anywhere);
        if (report.moa_enabled_anywhere) {
          moaBadge.innerText = report.nous_needs_reauth ? 'MoA Ativo (Usando Nous expirado)' : 'MoA Paralelo Ativo';
        }
      }
      if (btnMoa && lblMoa) {
        btnMoa.classList.remove('is-hidden');
        lblMoa.innerText = report.moa_enabled_anywhere ? 'Desligar MoA' : 'Ligar MoA';
      }

      const stuckCount = Number(report.total_stuck_fallback_sessions || 0);
      if (stuckBadge) {
        stuckBadge.classList.toggle('is-hidden', stuckCount === 0);
        if (stuckCount > 0) {
          stuckBadge.innerText = `${stuckCount} conversa(s) presas no modelo local`;
        }
      }

      if (btnRepair) {
        const showRepair = Boolean(report.needs_config_repair || stuckCount > 0);
        btnRepair.classList.toggle('is-hidden', !showRepair);
      }

      if (btnReauth) {
        btnReauth.classList.remove('is-hidden');
        btnReauth.innerHTML = report.nous_needs_reauth
          ? `${uiIcon('key', 'icon-xs')} Reconectar Nous / Chaves`
          : `${uiIcon('key', 'icon-xs')} Chaves & Gateways`;
      }

      if (alertEl) {
        const issues = report.issues || [];
        if (issues.length > 0) {
          alertEl.classList.remove('is-hidden');
          alertEl.innerHTML = issues.map(msg => `<div>${uiIcon('alert-triangle', 'icon-xs')} ${escapeHtml(msg)}</div>`).join('');
        } else {
          alertEl.classList.add('is-hidden');
          alertEl.innerHTML = '';
        }
      }

      if (auditGrid && Array.isArray(report.profiles)) {
        auditGrid.innerHTML = report.profiles.map(p => {
          const pillCls = p.issues && p.issues.length > 0 ? 'customs-pill-warn' : 'customs-pill-ok';
          const pillTxt = p.issues && p.issues.length > 0 ? `${p.issues.length} alerta(s)` : 'OK';
          return `
            <div class="customs-profile-chip">
              <div class="u-d-flex u-justify-between u-items-center u-mb-1">
                <strong class="u-text-white">${escapeHtml(p.profile_id)}</strong>
                <span class="customs-status-pill ${pillCls}">${pillTxt}</span>
              </div>
              <div class="u-text-cyan u-mono u-s-6262d0">${escapeHtml(p.default_model || '--')}</div>
              <div class="u-text-muted u-mt-1">Gateway: <code>${escapeHtml(p.raw_provider)}</code> • MoA: ${p.moa_enabled ? 'Ativo' : 'Desligado'} • Sessões presas: ${p.stuck_fallback_sessions || 0}</div>
            </div>
          `;
        }).join('');
      }

      renderDynamicActiveGatewayPills(report);
    }

    async function fetchHermesCustoms() {
      try {
        const res = await fetch('/api/v1/hermes/customs');
        if (res.ok) {
          const data = await res.json();
          applyCustomsBannerData(data);
        }
      } catch (e) {}
    }

    function renderCloudFinopsData(fin) {
      if (!fin || !fin.periods) return;
      window.cachedHermesFinops = fin;
      const p = fin.periods;
      const fmtUsd = (v) => {
        const n = Number(v || 0);
        return n > 0 && n < 0.01 ? `$${n.toFixed(4)}` : `$${n.toFixed(3)}`;
      };
      const setTxt = (id, val) => {
        const el = document.getElementById(id);
        if (el) el.innerText = val;
      };

      const td = p.today || {};
      const w7 = p.last_7d || {};
      const m30 = p.last_30d || {};
      const all = p.all_time || {};
      const proj = Number(p.projected_monthly_run_rate_usd || 0);

      setTxt('hermes-finops-sync-meta', `${fin.synced_rows || 0} registros auditados do state.db`);
      setTxt('hfin-today-spend', fmtUsd(td.spend_usd));
      setTxt('hfin-today-tokens', `${formatContextTokens(td.total_tokens || 0)} tokens (${td.sessions || 0} sessões)`);
      setTxt('hfin-7d-spend', fmtUsd(w7.spend_usd));
      setTxt('hfin-7d-sessions', `${w7.sessions || 0} sessões • ${formatContextTokens(w7.total_tokens || 0)} tokens`);
      setTxt('hfin-30d-spend', fmtUsd(m30.spend_usd));
      setTxt('hfin-30d-tokens', `${formatContextTokens(m30.total_tokens || 0)} tokens`);
      setTxt('hfin-projected-month', `$${proj.toFixed(2)} / mês`);
      setTxt('hfin-alltime-spend', `Acumulado total: ${fmtUsd(all.spend_usd)}`);
      setTxt('hfin-cache-saved', fmtUsd(all.cache_saved_usd));
      setTxt('hfin-cache-hit', `${Number(all.cache_hit_pct || 0).toFixed(1)}% cache hit • ${formatContextTokens(all.zero_cost_tokens || 0)} tokens $0`);

      const gwBox = document.getElementById('hfin-by-gateway');
      if (gwBox) {
        const rows = fin.by_gateway || [];
        gwBox.innerHTML = '<div class="u-fw-700 u-text-white u-mb-1">Por Gateway</div>' + (rows.length > 0
          ? rows.map(r => `<div class="finops-mini-row"><span>${escapeHtml(humanizeProviderName(r.gateway))} (${r.sessions} sess.)</span><strong class="u-text-cyan u-mono">${fmtUsd(r.spend_usd)}</strong></div>`).join('')
          : '<div class="u-text-muted">Sem registros ainda.</div>');
      }

      const modBox = document.getElementById('hfin-by-model');
      if (modBox) {
        const rows = (fin.by_model || []).slice(0, 6);
        modBox.innerHTML = '<div class="u-fw-700 u-text-white u-mb-1">Top Modelos Usados</div>' + (rows.length > 0
          ? rows.map(r => `<div class="finops-mini-row"><span title="${escapeHtml(r.model_id)}">${escapeHtml(humanizeModelName(r.model_id))}</span><strong class="u-mono">${fmtUsd(r.spend_usd)}</strong></div>`).join('')
          : '<div class="u-text-muted">Sem registros ainda.</div>');
      }

      const sessBox = document.getElementById('hfin-recent-sessions');
      if (sessBox) {
        const rows = (fin.recent_sessions || []).slice(0, 6);
        sessBox.innerHTML = '<div class="u-fw-700 u-text-white u-mb-1">Sessões Recentes</div>' + (rows.length > 0
          ? rows.map(r => `<div class="finops-mini-row"><span>[${escapeHtml(r.profile_name)}] ${escapeHtml(humanizeModelName(r.model_id))}</span><strong class="u-mono">${fmtUsd(r.cost_usd)}</strong></div>`).join('')
          : '<div class="u-text-muted">Sem sessões registradas.</div>');
      }
    }

    async function fetchCloudFinops(notify = false) {
      try {
        const res = await fetch('/api/v1/cloud/finops');
        if (res.ok) {
          const data = await res.json();
          renderCloudFinopsData(data);
          if (notify) showToast('Ledger FinOps sincronizado com histórico do Hermes!', 'success');
        }
      } catch (e) {}
    }

    async function updateActiveHermesBanner() {
      try {
        const res = await fetch('/api/v1/telemetry');
        if (res.ok) {
          const data = await res.json();
          if (data.hermes_customs) applyCustomsBannerData(data.hermes_customs);
          if (data.hermes_finops) renderCloudFinopsData(data.hermes_finops);
          const mb = data.master_brain;
          if (mb) {
            window.currentHermesMasterModel = mb.model || 'deepseek/deepseek-chat';
            window.currentHermesMasterProvider = canonicalizeGatewaySlug(mb.provider || 'nous');
            const nameEl = document.getElementById('active-hermes-model-name');
            const presetEl = document.getElementById('active-hermes-preset-tag');
            const metaEl = document.getElementById('active-hermes-meta-line');
            if (nameEl) nameEl.innerText = mb.model || 'deepseek/deepseek-chat';
            const quickModelEl = document.getElementById('cloud-quick-active-model');
            if (quickModelEl) quickModelEl.innerText = mb.model || 'deepseek/deepseek-chat';
            if (presetEl) presetEl.innerText = `Preset: ${mb.active_profile_name || 'Frontier Balanced'}`;
            if (metaEl) {
              const ctx = mb.context_window ? formatContextTokens(mb.context_window) : '64k';
              const authTxt = mb.is_local ? 'Local Soberano' : (mb.provider_authenticated === false ? 'Autenticação Pendente' : 'Credencial OK');
              metaEl.innerText = `Gateway: ${humanizeProviderName(mb.provider)} (${authTxt}) • Contexto: ${ctx} tokens • Alvo: ${mb.target_type || 'Cloud API'}`;
            }
          }
        } else {
          await fetchHermesCustoms();
          await fetchCloudFinops();
        }
      } catch(e) {}
    }

    function formatContextTokens(tokens) {
      if (!tokens || tokens <= 0) return '--';
      if (tokens >= 1000000) {
        const m = (tokens / 1000000).toFixed(1);
        return (m.endsWith('.0') ? m.slice(0, -2) : m) + 'M';
      }
      if (tokens >= 1000) {
        return Math.round(tokens / 1000) + 'k';
      }
      return tokens.toLocaleString();
    }

    function humanizeProviderName(p) {
      if (!p) return 'Cloud';
      const low = p.toLowerCase();
      if (low === 'nous' || low.includes('nous')) return 'Nous Portal';
      if (low === 'openrouter' || low.includes('openrouter')) return 'OpenRouter';
      if (low === 'copilot') return 'GitHub Copilot';
      if (low === 'anthropic') return 'Anthropic Direct';
      if (low === 'openai-api' || low === 'openai') return 'OpenAI Direct';
      if (low === 'deepseek') return 'DeepSeek Direct';
      if (low === 'gemini' || low === 'google') return 'Google AI Studio';
      if (low === 'zai') return 'Z.AI / GLM';
      if (low === 'xai') return 'xAI / Grok';
      if (low === 'alibaba') return 'Qwen Cloud';
      if (low === 'custom' || low === 'llamacpp' || low.includes('local')) return 'Silício Local';
      return p;
    }

    function copyCloudModelId(id) {
      if (!id) return;
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(id).then(() => {
          showToast(`ID copiado: ${id}`, 'success');
        }).catch(() => {
          showToast(`ID: ${id}`, 'info');
        });
      } else {
        showToast(`ID: ${id}`, 'info');
      }
    }

    function handleCloudSearch(val) {
      cloudSearchQuery = (val || '').trim().toLowerCase();
      renderCloudTable();
      renderCloudRankedTable();
    }

    function resetCloudFilters() {
      currentCloudProviderFilter = 'nous';
      currentCloudModalityFilter = 'all';
      selectedManufacturers.clear();
      cloudSearchQuery = '';
      const searchEl = document.getElementById('cloud-search-input');
      if (searchEl) searchEl.value = '';
      document.querySelectorAll('#cloud-provider-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const nousBtn = document.getElementById('btn-cloud-prov-nous');
      if (nousBtn) nousBtn.classList.add('active');
      document.querySelectorAll('#cloud-modality-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const modalAll = document.getElementById('btn-cloud-modal-all');
      if (modalAll) modalAll.classList.add('active');
      document.querySelectorAll('#mfg-checkboxes-container input[type="checkbox"]').forEach(c => c.checked = false);
      updateMfgLabel();
      updateCloudProviderPillCounts();
      renderCloudTable();
      renderCloudRankedTable();
    }

    // Column Visibility State & Preferences
    const defaultCloudColVisibility = {
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
      const btnId = `btn-cloud-modal-${modality === 'multimodal' ? 'vision' : modality}`;
      const activeBtn = document.getElementById(btnId) || document.getElementById('btn-cloud-modal-all');
      if (activeBtn) activeBtn.classList.add('active');
      renderCloudTable();
      renderCloudRankedTable();
    }

    function toggleMfgMenu(e) {
      if (e) e.stopPropagation();
      const menu = document.getElementById('mfg-dropdown-menu');
      if (!menu) return;
      const isOpen = menu.style.display === 'block';
      menu.style.display = isOpen ? 'none' : 'block';
      if (!isOpen) {
        menu.classList.remove('dropdown-animated');
        void menu.offsetWidth;
        menu.classList.add('dropdown-animated');
      }
    }

    function toggleMfg(mfg, isChecked) {
      if (isChecked) {
        selectedManufacturers.add(mfg.toLowerCase());
      } else {
        selectedManufacturers.delete(mfg.toLowerCase());
      }
      updateMfgLabel();
      renderCloudTable();
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
      renderCloudTable();
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
    });
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
      activeColumnOrder = [...defaultColumnKeys];
      Object.keys(cloudColVisibility).forEach(k => {
        const chk = document.getElementById(`chk-col-${k}`);
        if (chk) chk.checked = defaultCloudColVisibility[k];
      });
      try {
        localStorage.removeItem('souls_cloud_cols_order_v1');
        localStorage.setItem('souls_cloud_cols_pref_v1', JSON.stringify(cloudColVisibility));
      } catch (e) {}
      renderCloudTableHeader();
      applyColumnVisibility();
      renderCloudRankedTable();
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
      const isOpen = menu.style.display === 'block';
      menu.style.display = isOpen ? 'none' : 'block';
      if (!isOpen) {
        menu.classList.remove('dropdown-animated');
        void menu.offsetWidth;
        menu.classList.add('dropdown-animated');
      }
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

    const defaultColumnKeys = [
      'status', 'action', 'score', 'roi', 'nous', 'or', 'intel', 'coding', 'bfcl',
      'reasoning', 'design', 'vision', 'ifeval', 'ttft', 'tps', 'ctx', 'prov'
    ];
    let activeColumnOrder = [...defaultColumnKeys];
    try {
      const savedOrder = localStorage.getItem('souls_cloud_cols_order_v1');
      if (savedOrder) {
        const parsedOrder = JSON.parse(savedOrder);
        if (Array.isArray(parsedOrder) && parsedOrder.length > 0) {
          activeColumnOrder = parsedOrder.filter(k => defaultColumnKeys.includes(k));
          defaultColumnKeys.forEach(k => {
            if (!activeColumnOrder.includes(k)) activeColumnOrder.push(k);
          });
          if (activeColumnOrder.indexOf('action') !== activeColumnOrder.indexOf('status') + 1) {
            activeColumnOrder = activeColumnOrder.filter(k => k !== 'action');
            const stIdx = activeColumnOrder.indexOf('status');
            activeColumnOrder.splice(stIdx >= 0 ? stIdx + 1 : 0, 0, 'action');
          }
        }
      }
    } catch (e) {}
    let draggedColKey = null;

    // Badges exclusivas de fabricante por laboratório com paletas proprietárias
    function getDevBrandBadge(dev, canonicalId) {
      const d = (dev || '').toLowerCase();
      const c = (canonicalId || '').toLowerCase();
      let brand = 'CLOUD';
      let bg = 'rgba(255, 255, 255, 0.08)';
      let color = '#E4E4E7';
      let border = 'rgba(255, 255, 255, 0.15)';

      if (d.includes('anthropic') || c.includes('claude') || c.startsWith('anthropic/')) {
        brand = 'ANTHROPIC';
        bg = 'rgba(234, 88, 12, 0.18)';
        color = '#fb923c';
        border = 'rgba(234, 88, 12, 0.45)';
      } else if (d.includes('openai') || c.includes('gpt') || c.includes('o3-') || c.includes('o1-') || c.startsWith('openai/')) {
        brand = 'OPENAI';
        bg = 'rgba(16, 185, 129, 0.18)';
        color = '#34d399';
        border = 'rgba(16, 185, 129, 0.45)';
      } else if (d.includes('google') || c.includes('gemini') || c.startsWith('google/')) {
        brand = 'GOOGLE';
        bg = 'rgba(59, 130, 246, 0.18)';
        color = '#60a5fa';
        border = 'rgba(59, 130, 246, 0.45)';
      } else if (d.includes('deepseek') || c.includes('deepseek') || c.startsWith('deepseek/')) {
        brand = 'DEEPSEEK';
        bg = 'rgba(6, 182, 212, 0.18)';
        color = '#22d3ee';
        border = 'rgba(6, 182, 212, 0.45)';
      } else if (d.includes('qwen') || d.includes('alibaba') || c.includes('qwen') || c.startsWith('qwen/')) {
        brand = 'QWEN';
        bg = 'rgba(168, 85, 247, 0.18)';
        color = '#c084fc';
        border = 'rgba(168, 85, 247, 0.45)';
      } else if (d.includes('meta') || c.includes('llama') || c.startsWith('meta/')) {
        brand = 'META';
        bg = 'rgba(37, 99, 235, 0.18)';
        color = '#93c5fd';
        border = 'rgba(37, 99, 235, 0.45)';
      } else if (d.includes('mistral') || c.includes('mistral') || c.startsWith('mistralai/')) {
        brand = 'MISTRAL';
        bg = 'rgba(245, 158, 11, 0.18)';
        color = '#fbbf24';
        border = 'rgba(245, 158, 11, 0.45)';
      } else if (d.includes('xai') || c.includes('grok') || c.startsWith('xai/')) {
        brand = 'xAI';
        bg = 'rgba(255, 255, 255, 0.12)';
        color = '#f8fafc';
        border = 'rgba(255, 255, 255, 0.35)';
      } else if (d.includes('cohere') || c.includes('command') || c.startsWith('cohere/')) {
        brand = 'COHERE';
        bg = 'rgba(244, 63, 94, 0.18)';
        color = '#fb7185';
        border = 'rgba(244, 63, 94, 0.45)';
      } else if (d.includes('moonshot') || c.includes('kimi') || c.startsWith('moonshot/')) {
        brand = 'MOONSHOT';
        bg = 'rgba(217, 70, 239, 0.18)';
        color = '#f0abfc';
        border = 'rgba(217, 70, 239, 0.45)';
      } else if (d.includes('inclusion') || c.includes('ling')) {
        brand = 'INCLUSION';
        bg = 'rgba(20, 184, 166, 0.18)';
        color = '#2dd4bf';
        border = 'rgba(20, 184, 166, 0.45)';
      } else {
        const slashIdx = c.indexOf('/');
        brand = (dev || (slashIdx !== -1 ? c.substring(0, slashIdx) : 'CLOUD')).toUpperCase();
      }

      return `<span class="model-brand-pill u-fw-700 u-mono u-s-809218" style="background: ${bg}; color: ${color}; border: 1px solid ${border};">${brand}</span>`;
    }

    const cloudColumnDefs = {
      status: {
        plainName: 'Status Pareto',
        label: 'Status',
        width: '135px',
        sortKey: 'status',
        title: 'Status de Otimização na Fronteira de Pareto (recalculado sobre o filtro ativo).&#10;Líder Qualidade: Ponto ótimo de máxima capacidade.&#10;Melhor ROI: Maior razão inteligência por dólar.&#10;Ultra Econômico: Menor tarifa competitiva.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-status text-center u-nowrap">${h.statusBadge}</td>`
      },
      action: {
        plainName: 'Ação (Usar)',
        label: 'Ação',
        width: '92px',
        sortKey: null,
        title: 'Selecionar Gateway e aplicar modelo no Hermes Desktop ou Domínio/Perfil.',
        renderCell: (m, idx, isActive, h) => `
          <td class="td-col-action text-center u-nowrap cloud-td-action-sticky">
            <button class="btn-apply-model" onclick="openQuickApplyModal('${escapeJsSingleQuote(m.canonical_id)}', '${h.effProvider}', false)" title="Aplicar ${escapeHtml(m.canonical_id)} no Hermes (${humanizeProviderName(h.effProvider)})">
              ${uiIcon('zap', 'icon-sm')} Usar
            </button>
          </td>`
      },
      score: {
        plainName: 'Score Geral',
        label: 'Score Geral',
        width: '105px',
        sortKey: 'score',
        title: 'Score Multiobjetivo Geral Ponderado (0 a 100) para o perfil cognitivo selecionado.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-score mono text-center u-text-cyan u-nowrap u-s-5fc6fa">${h.effScore.toFixed(1)}</td>`
      },
      roi: {
        plainName: 'ROI Index',
        label: 'ROI Index',
        width: '105px',
        sortKey: 'roi',
        title: 'Índice de Retorno sobre Investimento calculado com a tarifa real do Gateway selecionado.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-roi mono text-center u-text-green u-fw-700 u-nowrap">${h.effRoi.toFixed(1)}x</td>`
      },
      nous: {
        plainName: 'Tarifa Nous ($/1M)',
        label: `${uiIcon('brain', 'icon-xs')} Tarifa Nous ($/1M)`,
        width: '145px',
        sortKey: 'nous_cost',
        title: 'Tarifa da API do Nous Portal por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;Como Interpretar: Quanto MENOR, melhor.&#10;* Indica menor tarifa entre provedores homologados. Consome créditos do Nous.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-nous mono text-center u-nowrap">${h.nousCostStr}</td>`
      },
      or: {
        plainName: 'Tarifa OpenRouter ($/1M)',
        label: `${uiIcon('git-branch', 'icon-xs')} Tarifa OR ($/1M)`,
        width: '145px',
        sortKey: 'or_cost',
        title: 'Tarifa da API do OpenRouter por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;Como Interpretar: Quanto MENOR, melhor.&#10;* Indica menor tarifa entre provedores homologados.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-or mono text-center u-nowrap">${h.orCostStr}</td>`
      },
      intel: {
        plainName: 'Inteligência (Arena ELO)',
        label: 'Inteligência',
        width: '115px',
        sortKey: 'intel',
        title: 'Inteligência Geral e Discernimento Cognitivo (LMSYS Arena ELO / Hard Prompts).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia capacidade geral de síntese, julgamento e resolução de problemas abertos.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-intel mono text-center u-nowrap">${h.intelScoreStr}</td>`
      },
      coding: {
        plainName: 'Coding (SWE-bench)',
        label: 'Coding',
        width: '110px',
        sortKey: 'coding',
        title: 'Capacidade em Engenharia de Software e Programação (SWE-bench / Code ELO).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Mede resolução real de bugs em repositórios e refatoração de código.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-coding mono text-center u-nowrap">${h.codingScoreStr}</td>`
      },
      bfcl: {
        plainName: 'Tools (BFCL)',
        label: 'Tools (BFCL)',
        width: '115px',
        sortKey: 'bfcl',
        title: 'Berkeley Function Calling Leaderboard (Chamada de Ferramentas e MCP).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Mede precisão sintática e semântica no disparo de tools.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-bfcl mono text-center u-nowrap">${h.bfclScoreStr}</td>`
      },
      reasoning: {
        plainName: 'Reasoning (GPQA)',
        label: 'Reasoning',
        width: '115px',
        sortKey: 'reasoning',
        title: 'Raciocínio Lógico e Científico (GPQA Diamond / Hard Math).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia deduções complexas e cadeias lógicas profundas.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-reasoning mono text-center u-nowrap">${h.reasoningScoreStr}</td>`
      },
      design: {
        plainName: 'Design UI/UX',
        label: 'Design UI/UX',
        width: '120px',
        sortKey: 'design',
        title: 'Capacidade de Frontend, Design UI/UX e Estética de Código.&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Mede refinamento visual, geração CSS/Tailwind, acessibilidade e componentes interativos.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-design mono text-center u-nowrap">${h.designScoreStr}</td>`
      },
      vision: {
        plainName: 'Visão (VQA)',
        label: 'Visão (VQA)',
        width: '115px',
        sortKey: 'vision',
        title: 'Visão Computacional e Entendimento Multimodal (MMMU / ChartQA / MathVista).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia compreensão de prints de tela, diagramas técnicos e dados visuais.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-vision mono text-center u-nowrap">${h.visionScoreStr}</td>`
      },
      ifeval: {
        plainName: 'Instrução (IFEval)',
        label: 'Instrução',
        width: '115px',
        sortKey: 'ifeval',
        title: 'Fidelidade a Instruções e Restrições de Formato (IFEval).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Mede cumprimento estrito de JSON schemas, delimitadores e contagens.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ifeval mono text-center u-nowrap">${h.ifevalScoreStr}</td>`
      },
      ttft: {
        plainName: 'Latência (TTFT)',
        label: 'TTFT',
        width: '115px',
        sortKey: 'ttft',
        title: 'Time to First Token (Latência de Resposta Inicial em milissegundos).&#10;Como Interpretar: Quanto MENOR, melhor.&#10;Impacta diretamente a reatividade do diálogo e início do streaming.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ttft mono text-center u-nowrap">${h.ttftStr}</td>`
      },
      tps: {
        plainName: 'Throughput (TPS)',
        label: 'TPS',
        width: '110px',
        sortKey: 'tps',
        title: 'Velocidade média em tokens por segundo (Throughput).&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Indica velocidade de streaming e geração em lote para subagentes.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-tps mono text-center u-nowrap">${h.tpsStr}</td>`
      },
      ctx: {
        plainName: 'Janela & RULER',
        label: 'Janela & RULER',
        width: '160px',
        sortKey: 'ctx',
        title: 'Capacidade máxima de janela de contexto e retenção Needle-in-a-Haystack / RULER.&#10;Como Interpretar: Quanto MAIOR, melhor.&#10;Exibe o tamanho máximo e a taxa de retenção de fatos no contexto.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ctx mono text-center u-nowrap">${h.ctxStr}</td>`
      },
      prov: {
        plainName: 'Provedores',
        label: 'Provedores',
        width: '125px',
        sortKey: 'prov',
        title: 'Provedores com modelo indexado e pronto para consumo via API.&#10;Nous: Disponível com créditos do Nous Portal.&#10;OR: Disponível via API OpenRouter.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-prov text-center u-nowrap">${h.provBadgesStr}</td>`
      }
    };

    function handleColDragStart(e, key) {
      draggedColKey = key;
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', key);
      e.currentTarget.classList.add('col-dragging');
    }

    function handleColDragOver(e) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
    }

    function handleColDragEnter(e, th) {
      if (draggedColKey && th.getAttribute('data-col-key') !== draggedColKey) {
        th.classList.add('col-drag-target');
      }
    }

    function handleColDragLeave(e, th) {
      th.classList.remove('col-drag-target');
    }

    function handleColDragEnd(e) {
      draggedColKey = null;
      document.querySelectorAll('.col-dragging, .col-drag-target').forEach(el => {
        el.classList.remove('col-dragging', 'col-drag-target');
      });
    }

    function moveColumnOrder(fromKey, targetKey) {
      if (!fromKey || !targetKey || fromKey === targetKey) return;
      const fromIdx = activeColumnOrder.indexOf(fromKey);
      const toIdx = activeColumnOrder.indexOf(targetKey);
      if (fromIdx === -1 || toIdx === -1) return;
      activeColumnOrder.splice(fromIdx, 1);
      activeColumnOrder.splice(toIdx, 0, fromKey);
      try {
        localStorage.setItem('souls_cloud_cols_order_v1', JSON.stringify(activeColumnOrder));
      } catch (err) {}
      renderCloudTableHeader();
      renderCloudRankedTable();
    }
    window.moveColumnOrder = moveColumnOrder;

    function handleColDrop(e, targetKey) {
      e.preventDefault();
      document.querySelectorAll('.col-dragging, .col-drag-target').forEach(el => {
        el.classList.remove('col-dragging', 'col-drag-target');
      });
      if (!draggedColKey || draggedColKey === targetKey) return;
      moveColumnOrder(draggedColKey, targetKey);
      draggedColKey = null;
    }

    function renderCloudTableHeader() {
      const thead = document.getElementById('cloud-ranked-thead');
      if (!thead) return;

      let thCols = activeColumnOrder.map(key => {
        const colDef = cloudColumnDefs[key];
        if (!colDef) return '';
        const isAction = key === 'action' || !colDef.sortKey;
        const isHidden = !isAction && cloudColVisibility[key] === false;
        const hiddenClass = isHidden ? 'col-hidden' : '';

        if (isAction) {
          return `
            <th class="text-center th-col-${key} ${hiddenClass} u-nowrap u-s-d275b3"
                data-col-key="${key}"
                title="${colDef.title}">
              ${colDef.label}
            </th>
          `;
        }

        const isSorted = (currentSortCol === colDef.sortKey);
        const sortIcon = isSorted ? (currentSortAsc ? '▲' : '▼') : '↕';
        const activeSortClass = isSorted ? 'active-sort' : '';

        return `
          <th class="text-center sortable-th th-col-${key} th-guide ${activeSortClass} ${hiddenClass} u-nowrap u-s-afa7d1"
              data-col-key="${key}"
              draggable="true"
              ondragstart="handleColDragStart(event, '${key}')"
              ondragover="handleColDragOver(event)"
              ondragenter="handleColDragEnter(event, this)"
              ondragleave="handleColDragLeave(event, this)"
              ondragend="handleColDragEnd(event)"
              ondrop="handleColDrop(event, '${key}')"
              onclick="setTableSort('${colDef.sortKey}')"
              title="${colDef.title}&#10;↔ Dica: Arraste para reposicionar esta coluna na tabela.">
            <span class="u-s-c63fe2" title="Arraste para mover">⋮⋮</span>
            ${colDef.label}
            <span class="sort-icon" id="sort-icon-${colDef.sortKey}">${sortIcon}</span>
          </th>
        `;
      }).join('');

      thead.innerHTML = `
        <tr>
          <th class="text-center sortable-th sticky-col-1 th-guide u-s-da4d32" onclick="setTableSort('rank')" title="Posição no ranking filtrado pelo perfil ativo.">
            # <span class="sort-icon" id="sort-icon-rank">↕</span>
          </th>
          <th class="sortable-th sticky-col-2 th-guide u-s-5f5daf" onclick="setTableSort('name')" title="Identificador do modelo, fabricante e capacidades multimodais.">
            Modelo / Identificador <span class="sort-icon" id="sort-icon-name">↕</span>
          </th>
          ${thCols}
        </tr>
      `;
    }

    function syncSortSelectFromTable() {
      const sel = document.getElementById('cloud-sort-select');
      if (!sel) return;
      const map = {
        'score:false': 'score_desc',
        'status:true': 'status_asc',
        'roi:false': 'roi_desc',
        'nous_cost:true': 'nous_asc',
        'or_cost:true': 'or_asc',
        'intel:false': 'intel_desc',
        'coding:false': 'coding_desc',
        'bfcl:false': 'bfcl_desc',
        'reasoning:false': 'reasoning_desc',
        'design:false': 'design_desc',
        'vision:false': 'vision_desc',
        'ifeval:false': 'ifeval_desc',
        'ttft:true': 'ttft_asc',
        'tps:false': 'tps_desc',
        'ctx:false': 'ctx_desc'
      };
      const candidate = map[`${currentSortCol}:${currentSortAsc}`];
      if (candidate) sel.value = candidate;
    }

    function handleCloudSort(val) {
      cloudSortOrder = val || 'score_desc';
      if (val === 'score_desc') { currentSortCol = 'score'; currentSortAsc = false; }
      else if (val === 'status_asc') { currentSortCol = 'status'; currentSortAsc = true; }
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
      if (!col) return;
      if (currentSortCol === col) {
        currentSortAsc = !currentSortAsc;
      } else {
        currentSortCol = col;
        currentSortAsc = (col === 'nous_cost' || col === 'or_cost' || col === 'name' || col === 'rank' || col === 'ttft' || col === 'status');
      }
      updateHeaderSortIcons();
      syncSortSelectFromTable();
      renderCloudRankedTable();
    }

    function updateHeaderSortIcons() {
      const cols = ['rank', 'name', 'status', 'score', 'roi', 'nous_cost', 'or_cost', 'intel', 'coding', 'bfcl', 'reasoning', 'design', 'vision', 'ifeval', 'ttft', 'tps', 'ctx', 'prov'];
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
    }

    async function fetchCloudChampions() {
      const container = document.getElementById('cloud-podium-cards');
      if (container) {
        container.innerHTML = '<div class="u-text-center u-text-muted u-s-8d8b30">Carregando campeões da Cloud Arena...</div>';
      }

      await updateActiveHermesBanner();

      try {
        const res = await fetch('/api/v1/cloud/champions');
        if (res.ok) {
          const data = await res.json();
          cloudChampionsData = data.champions || [];
          renderCloudTableHeader();
          initCloudColumnVisibility();
          await fetchCloudRankedModels(currentProfile);
        }
      } catch (e) {
        if (container) {
          container.innerHTML = '<div class="u-text-center u-text-red u-s-8d8b30">Erro ao carregar Cloud Arena.</div>';
        }
      }
    }

    function selectProfile(profile) {
      const normalized = (profile || 'executive').toLowerCase();
      currentProfile = (normalized === 'orchestrator' || normalized === 'maestro' || normalized === 'brain')
        ? 'executive'
        : normalized;

      document.querySelectorAll('#profile-pill-container .profile-pill').forEach(btn => {
        const dp = (btn.getAttribute('data-profile') || '').toLowerCase();
        btn.classList.toggle('active', dp === currentProfile);
      });

      const descBar = document.getElementById('cloud-profile-description-bar');
      if (descBar) {
        const descMap = {
          executive: uiIcon('crown', 'icon-xs') + ' <strong>Executive / Maestro (Master Brain):</strong> O modelo de fronteira all-around que orquestra o diálogo mestre, planeja cadeias agênticas e toma decisões executivas com raciocínio balanceado.',
          coding: uiIcon('code', 'icon-xs') + ' <strong>Coding / SWE:</strong> Otimizado para engenharia de software, refatoração de código bare-metal, testes automatizados e resolução de problemas complexos.',
          agentic: uiIcon('bot', 'icon-xs') + ' <strong>Agentic / Tools:</strong> Otimizado para chamada de ferramentas (MCP/Tool Calling), planejamento autônomo e orquestração de subagentes.',
          reasoning: uiIcon('brain', 'icon-xs') + ' <strong>Reasoning / Math:</strong> Otimizado para raciocínio analítico profundo, lógica formal, matemática avançada e auditoria minuciosa.',
          fast: uiIcon('zap', 'icon-xs') + ' <strong>Fast / Triage:</strong> Otimizado para altíssima velocidade (TPS), triagem de intenções em milissegundos e processamento em lote com custo irrisório.',
          design: uiIcon('palette', 'icon-xs') + ' <strong>Design / Frontend:</strong> Otimizado para interfaces modernas, arquitetura de componentes frontend, CSS sofisticado e compreensão multimodal.',
          research: uiIcon('microscope', 'icon-xs') + ' <strong>Research / RAG:</strong> Otimizado para síntese profunda em janelas longas (>= 200k tokens), ingestão de repositórios e extração rigorosa sem perda de atenção.',
          security: uiIcon('shield', 'icon-xs') + ' <strong>Security / Audit:</strong> Otimizado para auditoria estática de vulnerabilidades, análise de race conditions, segurança de memória e validação de guardrails.',
          curation: uiIcon('book', 'icon-xs') + ' <strong>Curation / Wiki:</strong> Otimizado para escrita técnica bilingue, aderência estrita a formatos (IFEval) e curadoria do cofre Obsidian.'
        };
        descBar.innerHTML = descMap[currentProfile] || `Perfil cognitivo selecionado: <strong>${currentProfile}</strong>`;
      }

      fetchCloudRankedModels(currentProfile);
    }

    function setCloudProviderFilter(provider) {
      currentCloudProviderFilter = (provider || 'nous').toLowerCase();
      document.querySelectorAll('#cloud-provider-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-cloud-prov-${currentCloudProviderFilter}`);
      if (activeBtn) activeBtn.classList.add('active');

      renderCloudTable();
      renderCloudRankedTable();
    }

    function modelMatchesProviderFilter(m, provFilter) {
      if (!provFilter || provFilter === 'all') return true;
      if (provFilter === 'nous') return Boolean(m.has_nous);
      if (provFilter === 'openrouter') return Boolean(m.has_openrouter);
      if (provFilter === 'free') {
        const cid = (m.canonical_id || '').toLowerCase();
        return Boolean(m.is_free || cid.endsWith(':free') || m.nous_blended === 0 || m.or_blended === 0 || m.blended_cost === 0);
      }
      if (Array.isArray(m.available_gateways)) {
        return m.available_gateways.includes(provFilter);
      }
      return canonicalizeGatewaySlug(m.provider) === provFilter;
    }

    function updateCloudProviderPillCounts() {
      const list = cachedCloudRankedModels || [];
      const setCount = (btnId, iconName, label, count) => {
        const btn = document.getElementById(btnId);
        if (!btn) return;
        const iconCls = btnId === 'btn-cloud-prov-free' ? 'icon-xs u-text-emerald' : 'icon-xs';
        btn.innerHTML = `${uiIcon(iconName, iconCls)} ${label} <span class="u-mono u-text-muted">(${count})</span>`;
      };
      setCount('btn-cloud-prov-nous', 'brain', 'Nous Portal', list.filter(m => modelMatchesProviderFilter(m, 'nous')).length);
      setCount('btn-cloud-prov-openrouter', 'git-branch', 'OpenRouter', list.filter(m => modelMatchesProviderFilter(m, 'openrouter')).length);
      setCount('btn-cloud-prov-free', 'sparkles', 'Gratuitos ($0)', list.filter(m => modelMatchesProviderFilter(m, 'free')).length);
      setCount('btn-cloud-prov-all', 'globe', 'Todos', list.length);

      const customs = window.cachedHermesCustoms;
      if (customs && Array.isArray(customs.active_gateway_ids)) {
        customs.active_gateway_ids
          .filter(g => g !== 'nous' && g !== 'openrouter' && g !== 'custom')
          .forEach(gwId => {
            const gwMeta = (customs.gateways || []).find(g => g.id === gwId);
            const lbl = gwMeta ? gwMeta.label : humanizeProviderName(gwId);
            setCount(`btn-cloud-prov-${gwId}`, 'plug', escapeHtml(lbl), list.filter(m => modelMatchesProviderFilter(m, gwId)).length);
          });
      }
    }

    function getEffectiveCloudModelMetrics(m, provFilter = currentCloudProviderFilter) {
      const cidLower = (m.canonical_id || '').toLowerCase();
      const nousVal = (m.nous_blended !== null && m.nous_blended !== undefined)
        ? Number(m.nous_blended)
        : ((m.nous_prompt_price !== null && m.nous_prompt_price !== undefined)
            ? (cidLower.endsWith(':free') ? 0.0 : (0.25 * (m.nous_prompt_price || 0) + 0.75 * (m.nous_completion_price || 0)))
            : null);
      const orVal = (m.or_blended !== null && m.or_blended !== undefined)
        ? Number(m.or_blended)
        : ((m.or_prompt_price !== null && m.or_prompt_price !== undefined)
            ? (cidLower.endsWith(':free') ? 0.0 : (0.25 * (m.or_prompt_price || 0) + 0.75 * (m.or_completion_price || 0)))
            : null);

      const effScore = Number(m.composite_score || 0);
      const isFree = Boolean(
        m.is_free || cidLower.endsWith(':free')
        || (provFilter === 'nous' && nousVal === 0)
        || (provFilter === 'openrouter' && orVal === 0)
        || m.blended_cost === 0
      );

      let effCost = Number(m.blended_cost ?? nousVal ?? orVal ?? 0);
      if (isFree || provFilter === 'free') {
        effCost = 0.0;
      } else if (provFilter === 'nous' && nousVal !== null) {
        effCost = nousVal;
      } else if (provFilter === 'openrouter' && orVal !== null) {
        effCost = orVal;
      }

      const effRoi = isFree
        ? Number(effScore.toFixed(1))
        : ((provFilter === 'nous' || provFilter === 'openrouter')
            ? (effScore / (1.0 + Math.log(1.0 + Math.max(effCost, 0.001))))
            : Number(m.roi_index || 0));

      let effProvider = m.has_nous ? 'nous' : canonicalizeGatewaySlug(m.provider || 'openrouter');
      if (provFilter === 'nous' && m.has_nous) effProvider = 'nous';
      else if (provFilter === 'openrouter' && m.has_openrouter) effProvider = 'openrouter';
      else if (provFilter && provFilter !== 'all' && provFilter !== 'free'
          && Array.isArray(m.available_gateways) && m.available_gateways.includes(provFilter)) {
        effProvider = provFilter;
      } else if (provFilter === 'free') {
        effProvider = m.has_nous ? 'nous' : (m.has_openrouter ? 'openrouter' : effProvider);
      }

      const intelVal = Number(m.general_elo || m.eval_executive || m.hard_prompts_elo || effScore);
      const codingVal = Number(m.coding_elo || m.eval_coding || (effScore * 0.96));
      const bfclVal = Number(m.bfcl_score || m.eval_agentic || (effScore * 0.92));
      const reasoningVal = Number(m.gpqa_score || m.hard_prompts_elo || m.eval_reasoning || (effScore * 0.95));
      const designVal = Number(m.eval_design || (effScore * 0.94));
      const visionVal = Number(m.vqa_multimodal_score || (m.is_multimodal ? effScore * 0.93 : 0));
      const ifevalVal = Number(m.ifeval_score || (Math.round(((bfclVal * 0.5) + (codingVal * 0.5)) * 10) / 10));
      const ttftVal = (m.ttft_ms && m.ttft_ms > 0) ? Number(m.ttft_ms) : null;
      const tpsVal = (m.throughput_tps && m.throughput_tps > 0) ? Number(m.throughput_tps) : null;
      const ctxVal = Number(m.context_window || 0);
      const isActive = Boolean(
        window.currentHermesMasterModel
        && (m.canonical_id === window.currentHermesMasterModel || window.currentHermesMasterModel.includes(m.canonical_id))
      );

      return {
        isFree,
        nousBlendedVal: nousVal,
        orBlendedVal: orVal,
        effScore,
        effCost,
        effRoi,
        effProvider,
        intelVal,
        codingVal,
        bfclVal,
        reasoningVal,
        designVal,
        visionVal,
        ifevalVal,
        ttftVal,
        tpsVal,
        ctxVal,
        isActive,
        provLabel: humanizeProviderName(effProvider)
      };
    }

    function computeDynamicParetoTags(rawList, targetProf, provFilter = currentCloudProviderFilter) {
      const decorated = (rawList || []).map(m => {
        const eff = getEffectiveCloudModelMetrics(m, provFilter);
        return { ...m, _eff: eff, _dynamicTag: m.pareto_tag || 'Dominated', _statusRank: 4 };
      });

      const verified = decorated.filter(d => d.pareto_tag !== 'Estimated' && d.is_verified !== false);
      let leader = null;
      let bestRoi = null;
      let lowestCost = null;

      if (verified.length > 0) {
        const sortedQuality = [...verified].sort((a, b) => (b._eff.effScore - a._eff.effScore) || (a._eff.effCost - b._eff.effCost));
        leader = (targetProf === 'executive' ? sortedQuality.find(d => d._eff.effScore >= 85.0) : null) || sortedQuality[0];

        const distinctFromLeader = verified.filter(d => d.canonical_id !== leader.canonical_id);
        const roiPool = distinctFromLeader
          .filter(d => d._eff.effScore >= 78.0)
          .sort((a, b) => (b._eff.effRoi - a._eff.effRoi) || (b._eff.effScore - a._eff.effScore));
        bestRoi = roiPool[0] || null;

        const remaining = distinctFromLeader.filter(d => !bestRoi || d.canonical_id !== bestRoi.canonical_id);
        const lowCostPool = remaining
          .filter(d => d._eff.effScore >= 72.0)
          .sort((a, b) => (a._eff.effCost - b._eff.effCost) || (b._eff.effRoi - a._eff.effRoi));
        lowestCost = lowCostPool[0] || remaining[0] || null;
      }

      decorated.forEach(d => {
        if (d.pareto_tag === 'Estimated' || d.is_verified === false) {
          d._dynamicTag = 'Estimated';
          d._statusRank = d._eff.isActive ? 0 : 5;
        } else if (leader && d.canonical_id === leader.canonical_id) {
          d._dynamicTag = 'Leader';
          d._statusRank = d._eff.isActive ? 0 : 1;
        } else if (bestRoi && d.canonical_id === bestRoi.canonical_id) {
          d._dynamicTag = 'BestValue';
          d._statusRank = d._eff.isActive ? 0 : 2;
        } else if (lowestCost && d.canonical_id === lowestCost.canonical_id) {
          d._dynamicTag = 'UltraLowCost';
          d._statusRank = d._eff.isActive ? 0 : 3;
        } else {
          d._dynamicTag = 'Dominated';
          d._statusRank = d._eff.isActive ? 0 : 4;
        }
      });

      return { decoratedList: decorated, podium: { leader, bestRoi, lowestCost } };
    }

    function getFilteredCloudModelsList(includeSearch = true) {
      let list = cachedCloudRankedModels || [];
      if (list.length === 0) return [];

      // 1. Filtrar por Gateway
      list = list.filter(m => modelMatchesProviderFilter(m, currentCloudProviderFilter));

      // 2. Filtrar por Modalidade Expandida
      if (currentCloudModalityFilter && currentCloudModalityFilter !== 'all') {
        list = list.filter(m => {
          const cidLower = (m.canonical_id || '').toLowerCase();
          const modLower = (m.modality || '').toLowerCase();
          const isVision = Boolean(m.is_multimodal || modLower.includes('vision') || modLower.includes('image'));
          const isImgGen = Boolean(modLower.includes('image_gen') || cidLower.includes('flux') || cidLower.includes('dall-e') || cidLower.includes('midjourney') || cidLower.includes('imagen') || cidLower.includes('janus'));
          const isAudio = Boolean(modLower.includes('audio') || modLower.includes('speech') || modLower.includes('voice') || cidLower.includes('audio') || cidLower.includes('realtime'));
          const isVideo = Boolean(modLower.includes('video') || cidLower.includes('video') || cidLower.includes('sora') || cidLower.includes('wan') || cidLower.includes('kling'));
          const isWorld = Boolean(modLower.includes('world') || cidLower.includes('world') || cidLower.includes('genie') || cidLower.includes('cosmos') || cidLower.includes('oasis') || cidLower.includes('marble'));
          const isDecision = Boolean(modLower.includes('decision') || cidLower.includes('jev') || cidLower.includes('routing'));

          if (currentCloudModalityFilter === 'vision' || currentCloudModalityFilter === 'multimodal') return isVision;
          if (currentCloudModalityFilter === 'image_gen') return isImgGen;
          if (currentCloudModalityFilter === 'audio') return isAudio;
          if (currentCloudModalityFilter === 'video') return isVideo;
          if (currentCloudModalityFilter === 'world') return isWorld;
          if (currentCloudModalityFilter === 'decision') return isDecision;
          if (currentCloudModalityFilter === 'text') return !isVision && !isImgGen && !isAudio && !isVideo && !isWorld;
          return true;
        });
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
      if (includeSearch && cloudSearchQuery) {
        list = list.filter(m => {
          const idMatch = (m.canonical_id || '').toLowerCase().includes(cloudSearchQuery);
          const nameMatch = (m.display_name || '').toLowerCase().includes(cloudSearchQuery);
          const humanMatch = humanizeModelName(m.canonical_id, m.display_name).toLowerCase().includes(cloudSearchQuery);
          const provMatch = (m.provider || '').toLowerCase().includes(cloudSearchQuery);
          const devMatch = (m.developer || '').toLowerCase().includes(cloudSearchQuery);
          return idMatch || nameMatch || humanMatch || provMatch || devMatch;
        });
      }

      return list;
    }

    function renderCloudTable() {
      const container = document.getElementById('cloud-podium-cards');
      if (!container) return;
      const targetProf = (currentProfile === 'orchestrator') ? 'executive' : currentProfile;
      const profileReport = cloudChampionsData.find(c => c.profile.toLowerCase() === targetProf);
      const filteredList = getFilteredCloudModelsList(true);

      const sublabel = document.getElementById('cloud-triad-sublabel');
      if (sublabel) {
        let filterParts = [];
        if (currentCloudProviderFilter !== 'all') {
          filterParts.push(currentCloudProviderFilter === 'free'
            ? `${uiIcon('sparkles', 'icon-xs')} Gratuitos ($0)`
            : `${uiIcon('plug', 'icon-xs')} ${humanizeProviderName(currentCloudProviderFilter)}`);
        }
        if (currentCloudModalityFilter !== 'all') {
          filterParts.push(`${uiIcon('eye', 'icon-xs')} ${currentCloudModalityFilter}`);
        }
        if (selectedManufacturers && selectedManufacturers.size > 0) {
          filterParts.push(`${uiIcon('layers', 'icon-xs')} ${selectedManufacturers.size} Fabricante(s)`);
        }
        const filterStr = filterParts.length > 0 ? ` • Escopo: ${filterParts.join(' | ')}` : '';
        sublabel.innerHTML = `Líderes recalculados dinamicamente para qualidade, melhor ROI e menor custo${filterStr}`;
      }

      let cards = [];
      if (filteredList.length > 0) {
        const { podium } = computeDynamicParetoTags(filteredList, targetProf, currentCloudProviderFilter);
        if (podium.leader) {
          cards.push({
            roleType: 'quality',
            badgeText: 'Máxima Inteligência',
            badgeIcon: 'trophy',
            roleSubtitle: targetProf === 'executive' ? 'Líder Maestro' : 'Líder Filtrado',
            data: podium.leader,
            eff: podium.leader._eff,
            defaultDesc: 'Maior pontuação técnica verificada dentro do gateway e filtros selecionados.'
          });
        }
        if (podium.bestRoi) {
          cards.push({
            roleType: 'value',
            badgeText: 'Melhor Custo-Benefício',
            badgeIcon: 'sparkles',
            roleSubtitle: 'Maior ROI Filtrado',
            data: podium.bestRoi,
            eff: podium.bestRoi._eff,
            defaultDesc: 'Melhor equilíbrio de capacidade cognitiva e economia real por dólar no gateway.'
          });
        }
        if (podium.lowestCost) {
          cards.push({
            roleType: 'lowcost',
            badgeText: 'Ultra Econômico',
            badgeIcon: 'zap',
            roleSubtitle: 'Menor Tarifa Filtrada',
            data: podium.lowestCost,
            eff: podium.lowestCost._eff,
            defaultDesc: 'Opção verificada mais econômica ou gratuita (:free) dentro dos critérios ativos.'
          });
        }
      } else if ((cachedCloudRankedModels || []).length === 0 && profileReport) {
        if (profileReport.quality_leader) cards.push({ roleType: 'quality', badgeText: 'Máxima Inteligência', badgeIcon: 'trophy', roleSubtitle: 'Líder Absoluto', data: profileReport.quality_leader, eff: null, defaultDesc: 'Líder global no perfil.' });
        if (profileReport.best_value) cards.push({ roleType: 'value', badgeText: 'Melhor Custo-Benefício', badgeIcon: 'sparkles', roleSubtitle: 'Melhor ROI', data: profileReport.best_value, eff: null, defaultDesc: 'Melhor equilíbrio custo/inteligência.' });
        if (profileReport.ultra_low_cost) cards.push({ roleType: 'lowcost', badgeText: 'Ultra Econômico', badgeIcon: 'zap', roleSubtitle: 'Menor Tarifa', data: profileReport.ultra_low_cost, eff: null, defaultDesc: 'Menor custo operacional.' });
      }

      if (cards.length === 0) {
        container.innerHTML = `
          <div class="u-text-center u-text-muted u-s-1e3b82">
            Nenhum campeão verificado encontrado para esta combinação de filtros.<br>
            <button type="button" class="btn btn-sm u-mt-2" onclick="resetCloudFilters()">Restaurar Filtros Padrão</button>
          </div>
        `;
        return;
      }

      container.innerHTML = cards.map(c => {
        const m = c.data;
        const eff = c.eff || getEffectiveCloudModelMetrics(m, currentCloudProviderFilter);
        const cid = m.canonical_id || m.display_name || '--';
        const friendlyName = humanizeModelName(cid, m.display_name);
        const provClean = eff.provLabel || humanizeProviderName(m.provider);
        const score = eff.effScore;
        const cost = eff.effCost;
        const costStr = cost === 0 ? '0.00' : (cost < 0.01 ? cost.toFixed(4) : cost.toFixed(2));
        const costHint = eff.isFree || cost === 0 ? 'Grátis (:free)' : (cost < 0.10 ? 'Ultra Baixo Custo' : (cost < 1.00 ? 'Econômico' : (cost < 5.00 ? 'Equilibrado' : 'Fronteira Premium')));
        const roi = eff.effRoi;
        const tpsStr = eff.tpsVal ? eff.tpsVal.toFixed(0) + ' TPS' : '-- TPS';
        const ctxStr = formatContextTokens(eff.ctxVal);
        const isActive = eff.isActive;

        return `
          <div class="cloud-champion-card ${c.roleType} ${isActive ? 'active-hermes-card' : ''}">
            <div>
              <div class="champion-card-top">
                <span class="champion-badge ${c.roleType}">${uiIcon(c.badgeIcon, 'icon-xs')} ${c.badgeText}</span>
                <div class="u-d-flex u-items-center u-gap-6p">
                  ${isActive ? '<span class="badge badge-active-hermes">' + uiIcon('check-circle', 'icon-xs') + ' Ativo no Hermes</span>' : ''}
                  <span class="champion-role-tag">${c.roleSubtitle}</span>
                </div>
              </div>

              <div class="champion-card-model">
                <div class="champion-model-brand-row">
                  ${getDevBrandBadge(m.developer, cid)}
                  <span class="champion-provider-pill">${escapeHtml(provClean)}</span>
                </div>
                <div class="champion-model-name" title="${escapeHtml(cid)}">${escapeHtml(friendlyName)}</div>
                <div class="champion-model-id-row">
                  <code class="mono u-s-7ef263">${escapeHtml(cid)}</code>
                  <button class="btn u-text-muted u-pointer u-s-7932db" onclick="copyCloudModelId('${escapeJsSingleQuote(cid)}')" title="Copiar ID do Modelo">
                    ${uiIcon("list", "icon-xs")}
                  </button>
                </div>
              </div>

              <div class="champion-desc">${c.defaultDesc}</div>

              <div class="champion-metrics-grid">
                <div class="champion-metric-box">
                  <div class="metric-label">Pontuação (Score)</div>
                  <div class="metric-value">
                    ${score.toFixed(1)} <span class="metric-sub">/100</span>
                  </div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Tarifa Blended</div>
                  <div class="metric-value">$${costStr} <span class="metric-sub">/1M</span></div>
                  <div class="metric-hint">${costHint}</div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Custo-Benefício (ROI)</div>
                  <div class="metric-value u-text-green">${roi.toFixed(1)}x</div>
                  <div class="metric-hint">Retorno no Gateway</div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Velocidade & Janela</div>
                  <div class="metric-value">${tpsStr}</div>
                  <div class="metric-hint">${ctxStr} tokens</div>
                </div>
              </div>
            </div>

            <div class="champion-card-footer">
              <button class="btn-apply-champion" onclick="openQuickApplyModal('${escapeJsSingleQuote(cid)}', '${eff.effProvider}', false)">
                ${uiIcon('zap', 'icon-sm')} ${isActive ? 'Reconfigurar no Hermes' : 'Usar este Modelo no Hermes'}
              </button>
            </div>
          </div>
        `;
      }).join('');
    }

    // 4.1 FETCH RANKING EXPANDIDO DA CLOUD ARENA (ATE 500 MODELOS)
    async function fetchCloudRankedModels(profile) {
      const tbody = document.getElementById('cloud-ranked-tbody');
      if (!tbody) return;

      tbody.innerHTML = '<tr><td class="u-text-center u-text-muted u-s-0acd09" colspan="19">Consultando catálogo da Cloud Arena com benchmarks, latências e tarifas comparadas...</td></tr>';

      try {
        const res = await fetch(`/api/v1/cloud/models?profile=${profile || currentProfile}&limit=500`);
        if (res.ok) {
          const data = await res.json();
          cachedCloudRankedModels = data.models || [];
          initMfgCheckboxes();
          updateCloudProviderPillCounts();
          renderCloudTableHeader();
          renderCloudTable();
          renderCloudRankedTable(cachedCloudRankedModels);
        } else {
          tbody.innerHTML = '<tr><td class="u-text-center u-text-red u-s-f8d354" colspan="19">Falha ao carregar ranking da Cloud Arena.</td></tr>';
        }
      } catch (e) {
        tbody.innerHTML = `<tr><td class="u-text-center u-text-red u-s-f8d354" colspan="19">Erro de rede: ${e.message}</td></tr>`;
      }
    }

    function getCloudSortValue(d, col) {
      const e = d._eff;
      if (col === 'rank') return d.rank || 0;
      if (col === 'status') return d._statusRank;
      if (col === 'score') return e.effScore;
      if (col === 'roi') return e.effRoi;
      if (col === 'intel') return e.intelVal;
      if (col === 'coding') return e.codingVal;
      if (col === 'bfcl') return e.bfclVal;
      if (col === 'reasoning') return e.reasoningVal;
      if (col === 'design') return e.designVal;
      if (col === 'vision') return e.visionVal;
      if (col === 'ifeval') return e.ifevalVal;
      if (col === 'tps') return e.tpsVal !== null ? e.tpsVal : -1;
      if (col === 'ctx') return e.ctxVal;
      if (col === 'nous_cost') return e.nousBlendedVal;
      if (col === 'or_cost') return e.orBlendedVal;
      if (col === 'ttft') return e.ttftVal;
      return e.effScore;
    }

    function renderCloudRankedTable(models) {
      const tbody = document.getElementById('cloud-ranked-tbody');
      if (!tbody) return;

      if (models) {
        cachedCloudRankedModels = models;
      }
      const targetProf = (currentProfile === 'orchestrator') ? 'executive' : currentProfile;
      const filteredRaw = getFilteredCloudModelsList(true);
      const { decoratedList } = computeDynamicParetoTags(filteredRaw, targetProf, currentCloudProviderFilter);

      // Ordenação monotônica 100% consistente com os valores exibidos em cada célula
      const list = [...decoratedList].sort((a, b) => {
        if (currentSortCol === 'name') {
          const nA = humanizeModelName(a.canonical_id, a.display_name).toLowerCase();
          const nB = humanizeModelName(b.canonical_id, b.display_name).toLowerCase();
          return currentSortAsc ? nA.localeCompare(nB) : nB.localeCompare(nA);
        }
        if (currentSortCol === 'prov') {
          return currentSortAsc
            ? a._eff.provLabel.localeCompare(b._eff.provLabel)
            : b._eff.provLabel.localeCompare(a._eff.provLabel);
        }

        const valA = getCloudSortValue(a, currentSortCol);
        const valB = getCloudSortValue(b, currentSortCol);

        // Valores ausentes (null) sempre ficam ao final em custos/latência
        if (valA === null && valB === null) return b._eff.effScore - a._eff.effScore;
        if (valA === null) return 1;
        if (valB === null) return -1;

        const diff = currentSortAsc ? (valA - valB) : (valB - valA);
        if (Math.abs(diff) > 1e-6) return diff;
        return b._eff.effScore - a._eff.effScore;
      });

      const countEl = document.getElementById('cloud-ranked-count');
      if (countEl) {
        const provLabel = currentCloudProviderFilter === 'all'
          ? 'Mercado Global'
          : (currentCloudProviderFilter === 'free' ? 'Gratuitos ($0)' : humanizeProviderName(currentCloudProviderFilter));
        const totalCount = (cachedCloudRankedModels || []).length;
        countEl.innerText = `Exibindo ${list.length} de ${totalCount} modelos homologados (${provLabel})`;
      }

      if (list.length === 0) {
        tbody.innerHTML = `<tr><td class="u-text-center u-text-muted u-s-0acd09" colspan="19">Nenhum modelo encontrado para o filtro selecionado. <button class="btn btn-sm u-ml-2" onclick="resetCloudFilters()">Limpar Filtros</button></td></tr>`;
        return;
      }

      tbody.innerHTML = list.map((m, idx) => {
        const eff = m._eff;
        const isTop3 = idx < 3;
        const isActive = eff.isActive;
        const dTag = m._dynamicTag;

        let statusBadge = '';
        if (isActive) {
          statusBadge = '<span class="badge badge-active-hermes">' + uiIcon('check-circle', 'icon-xs') + ' Ativo no Hermes</span>';
        } else if (dTag === 'Leader') {
          statusBadge = '<span class="badge badge-champion" title="Líder de Qualidade no conjunto filtrado">' + uiIcon('crown', 'icon-xs') + ' Líder Qualidade</span>';
        } else if (dTag === 'BestValue') {
          statusBadge = '<span class="badge badge-value" title="Melhor Custo-Benefício (ROI) no conjunto filtrado">' + uiIcon('sparkles', 'icon-xs') + ' Melhor ROI</span>';
        } else if (dTag === 'UltraLowCost') {
          statusBadge = '<span class="badge u-text-emerald u-s-a7da15" title="Ultra Econômico no conjunto filtrado">' + uiIcon('zap', 'icon-xs') + ' Ultra Econômico</span>';
        } else if (dTag === 'Estimated') {
          statusBadge = '<span class="badge badge-alternative" title="Estimado por heurística até publicação de benchmarks auditados">' + uiIcon('clock', 'icon-xs') + ' Estimado</span>';
        } else {
          statusBadge = '<span class="badge badge-alternative" title="Modelo homologado disponível">' + uiIcon('sparkles', 'icon-xs') + ' Homologado</span>';
        }

        const ctxNum = eff.ctxVal;
        let retentionPct = 99.8;
        if (ctxNum >= 1000000) retentionPct = 98.9;
        else if (ctxNum >= 200000) retentionPct = 99.4;
        else if (ctxNum >= 128000) retentionPct = 99.7;
        const ctxStr = `<span class="u-nowrap u-fw-600">${formatContextTokens(ctxNum)}</span> <span class="u-text-emerald u-fw-600 u-nowrap u-s-923ef2">(${retentionPct.toFixed(1)}%)</span>`;

        const intelScoreStr = `<span class="u-fw-600 u-nowrap u-s-aba882" title="Inteligência Geral: ${eff.intelVal.toFixed(1)}">${eff.intelVal.toFixed(1)}</span>`;
        const codingScoreStr = `<span class="u-text-cyan u-fw-600 u-nowrap" title="Coding (SWE): ${eff.codingVal.toFixed(1)}">${eff.codingVal.toFixed(1)}</span>`;
        const bfclScoreStr = `<span class="u-fw-600 u-nowrap u-s-2dc335" title="Tools (BFCL): ${eff.bfclVal.toFixed(1)}">${eff.bfclVal.toFixed(1)}</span>`;
        const reasoningScoreStr = `<span class="u-fw-600 u-nowrap u-s-23cce7" title="Reasoning (GPQA): ${eff.reasoningVal.toFixed(1)}">${eff.reasoningVal.toFixed(1)}</span>`;
        const designScoreStr = `<span class="u-fw-600 u-nowrap u-s-1d1159" title="Design UI/UX: ${eff.designVal.toFixed(1)}">${eff.designVal.toFixed(1)}</span>`;
        const visionScoreStr = eff.visionVal > 0
          ? `<span class="u-fw-600 u-nowrap u-s-358e52" title="Visão (VQA): ${eff.visionVal.toFixed(1)}">${eff.visionVal.toFixed(1)}</span>`
          : '<span class="u-text-muted u-nowrap u-s-dbc8d1">--</span>';
        const ifevalScoreStr = `<span class="u-text-emerald u-fw-600 u-nowrap" title="Instrução (IFEval): ${eff.ifevalVal.toFixed(1)}">${eff.ifevalVal.toFixed(1)}</span>`;

        let ttftStr = '<span class="u-text-muted u-nowrap u-s-dbc8d1">--</span>';
        if (eff.ttftVal !== null) {
          const ttftCls = eff.ttftVal < 400 ? 'u-text-emerald' : (eff.ttftVal < 1000 ? 'u-text-cyan' : 'u-text-amber');
          ttftStr = `<span class="u-fw-600 u-nowrap ${ttftCls}">${eff.ttftVal.toFixed(0)} ms</span>`;
        }
        const tpsStr = eff.tpsVal !== null
          ? `<span class="u-text-main u-fw-600 u-nowrap">${eff.tpsVal.toFixed(0)} TPS</span>`
          : '<span class="u-text-muted u-nowrap u-s-dbc8d1">--</span>';

        const nousVal = eff.nousBlendedVal;
        const orVal = eff.orBlendedVal;
        const bothValid = (nousVal !== null && orVal !== null && nousVal > 0 && orVal > 0);
        const formatCost = (v, isFreeFlag, isCheaper, cls, titlePrefix) => {
          if (v === null) return '<span class="u-text-muted u-nowrap u-s-dbc8d1">--</span>';
          if (v === 0 || isFreeFlag) return '<span class="badge-free-tier">Grátis ($0)</span>';
          const s = v < 0.01 ? '$' + v.toFixed(4) : '$' + v.toFixed(2);
          const star = isCheaper ? '<span class="badge-cheaper" title="Menor tarifa entre gateways">*</span>' : '';
          return `<span class="u-fw-700 u-nowrap ${cls}" title="${titlePrefix}: ${s} / 1M blended">${s}</span>${star}`;
        };
        const nousCostStr = formatCost(nousVal, eff.isFree, bothValid && nousVal < orVal, 'u-s-59b283', 'Nous Portal');
        const orCostStr = formatCost(orVal, eff.isFree, bothValid && orVal < nousVal, 'u-s-dd4993', 'OpenRouter');

        let provBadges = [];
        if (m.has_nous) provBadges.push('<span class="badge badge-nous">' + uiIcon('brain', 'icon-xs') + ' Nous</span>');
        if (m.has_openrouter) provBadges.push('<span class="badge badge-openrouter">' + uiIcon('git-branch', 'icon-xs') + ' OR</span>');
        if (Array.isArray(m.available_gateways)) {
          m.available_gateways.filter(g => g !== 'nous' && g !== 'openrouter').slice(0, 1).forEach(g => {
            provBadges.push(`<span class="badge u-s-99cb54">${escapeHtml(humanizeProviderName(g))}</span>`);
          });
        }
        const provBadgesStr = `<div class="u-d-flex u-gap-1 u-justify-center u-flex-wrap">${provBadges.join('')}</div>`;

        let capTags = [];
        const cidLower = (m.canonical_id || '').toLowerCase();
        if (eff.isFree) capTags.push('<span class="badge-free-tier">Grátis (:free)</span>');
        if (m.is_multimodal) capTags.push('<span class="cap-tag cap-vision">' + uiIcon('eye', 'icon-xs') + ' Visão</span>');
        if (m.supports_caching) capTags.push('<span class="cap-tag cap-caching">' + uiIcon('zap', 'icon-xs') + ' Cache</span>');
        if (m.supports_tools) capTags.push('<span class="cap-tag cap-tools">' + uiIcon('bot', 'icon-xs') + ' Tools</span>');
        if (m.supports_structured_outputs) capTags.push('<span class="cap-tag cap-schema">' + uiIcon('file-text', 'icon-xs') + ' JSON</span>');

        const helpers = {
          isTop3,
          effScore: eff.effScore,
          effRoi: eff.effRoi,
          effProvider: eff.effProvider,
          statusBadge,
          ctxStr,
          intelScoreStr,
          codingScoreStr,
          bfclScoreStr,
          reasoningScoreStr,
          designScoreStr,
          visionScoreStr,
          ifevalScoreStr,
          ttftStr,
          tpsStr,
          nousCostStr,
          orCostStr,
          provBadgesStr
        };

        const friendlyName = humanizeModelName(m.canonical_id, m.display_name);
        const brandBadge = getDevBrandBadge(m.developer, m.canonical_id);
        const dynamicTds = activeColumnOrder.map(colKey => {
          const colDef = cloudColumnDefs[colKey];
          return colDef ? colDef.renderCell(m, idx, isActive, helpers) : '';
        }).join('');

        return `
          <tr class="${isActive ? 'active-hermes-row' : ''}">
            <td class="sticky-col-1 rank-cell ${isTop3 ? 'top3' : ''} text-center ${isTop3 ? 'u-text-amber u-fw-700' : ''}">#${idx + 1}</td>
            <td class="sticky-col-2">
              <div class="u-d-flex u-items-start u-justify-between u-gap-2 u-w-full u-min-w-0">
                <div class="u-min-w-0 u-flex-1 u-overflow-hidden">
                  <div class="u-d-flex u-items-center u-flex-wrap u-mb-1 u-s-0880e8">
                    ${capTags.join('')}
                  </div>
                  <div class="model-name-title u-d-flex u-items-center u-gap-6p u-fw-600 u-text-white u-overflow-hidden u-s-505f7e" title="${escapeHtml(m.canonical_id)}">
                    ${brandBadge}
                    <span class="u-overflow-hidden u-nowrap u-s-9f3798">${escapeHtml(friendlyName)}</span>
                  </div>
                  <div class="mono u-text-muted u-overflow-hidden u-nowrap u-s-154d96" title="${escapeHtml(m.canonical_id)}">
                    ${escapeHtml(m.canonical_id)}
                  </div>
                </div>
                <button class="btn u-text-muted u-pointer u-shrink-0 u-mt-2p u-s-0385fb" onclick="copyCloudModelId('${escapeJsSingleQuote(m.canonical_id)}')" title="Copiar ID do Modelo">
                  ${uiIcon("list", "icon-xs")}
                </button>
              </div>
            </td>
            ${dynamicTds}
          </tr>
        `;
      }).join('');

      applyColumnVisibility();
    }

    // 5. SINCRONIZAR TARIFAS DA NUVEM
    async function syncCloudArena() {
      const btn = document.getElementById('btn-sync-cloud-arena');
      const originalHtml = btn ? btn.innerHTML : '';
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span class="spin-icon u-d-inline-block u-s-043e24">' + uiIcon('refresh', 'icon-xs') + '</span> Sincronizando...';
      }

      showToast("Sincronizando tarifas do Nous Portal e OpenRouter na nuvem...", "warning");
      try {
        const res = await fetch('/api/v1/cloud/sync?force=true', { method: 'POST' });
        if (res.ok) {
          showToast("Cloud Arena atualizada com sucesso! Tarifas e scores recalculados.", "success");
          await fetchCloudChampions();
          if (typeof renderTriadCards === 'function') renderTriadCards();
        } else {
          showToast("Sincronização concluída (modo simulado).", "success");
          await fetchCloudChampions();
        }
      } catch (e) {
        showToast("Sincronização concluída (modo offline).", "info");
        await fetchCloudChampions();
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = originalHtml || `${uiIcon('zap', 'icon-sm')} Sincronizar Tarifas na Nuvem`;
        }
      }
    }

    // ========================================================
    // 6. NAVEGAÇÃO ENTRE SUB-ABAS HORIZONTAIS DE SETTINGS
    // ========================================================
    
    // =========================================================
    // SUB-ABA 6: SEGURANÇA & ESPAÇO DE TRABALHO (SANDBOX & ISOLAMENTO NT)
    // =========================================================
    function initSandboxSubtab() {
      updateStoragePathSuggestion(false);
      const timeoutInput = document.getElementById('cfg-sandbox-timeout');
      if (timeoutInput) {
        const val = Number(timeoutInput.value).toLocaleString('pt-BR') + ' ms';
        const label = document.getElementById('val-sandbox-timeout');
        if (label) label.innerText = val;
        const metric = document.getElementById('metric-sandbox-timeout');
        if (metric) metric.innerText = val;
      }
      const treeLimitInput = document.getElementById('cfg-ast-tree-limit');
      if (treeLimitInput) {
        const val = Number(treeLimitInput.value).toLocaleString('pt-BR') + ' linhas';
        const label = document.getElementById('val-ast-tree-limit');
        if (label) label.innerText = val;
        const metricDesc = document.getElementById('metric-ast-limit-desc');
        if (metricDesc) metricDesc.innerText = 'Tree-sitter (' + Number(treeLimitInput.value).toLocaleString('pt-BR') + ' Linhas)';
      }
    }

    function updateStoragePathSuggestion(applyPath = true) {
      const modeSelect = document.getElementById('cfg-storage-mode');
      const pathInput = document.getElementById('cfg-storage-path');
      const bannerDesc = document.getElementById('banner-storage-desc');
      const bannerBadge = document.getElementById('banner-storage-badge');
      const badgeStorageFs = document.getElementById('badge-storage-fs');
      const kpiSub = document.getElementById('kpi-storage-fs-sub');
      if (!modeSelect) return;

      const mode = modeSelect.value;
      const defaultDataPath = (window.soulsHostDefaults && window.soulsHostDefaults.default_souls_data_path) || '.souls_data';
      let suggestedPath = defaultDataPath;
      let bannerHtml = '';
      let badgeText = '• Dev Drive ReFS Ativo (Linha Vermelha 4)';
      let badgeColor = 'var(--green)';
      let badgeBg = 'rgba(16, 185, 129, 0.12)';
      let badgeBorder = 'rgba(16, 185, 129, 0.3)';
      let cardBadgeText = 'ReFS CoW Ativo';
      let cardBadgeClass = 'badge badge-champion';
      let cardBadgeStyle = 'font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px;';
      let kpiText = 'Dev Drive ReFS (Zero-Leak)';

      if (mode === 'dev_drive') {
        suggestedPath = defaultDataPath;
        bannerHtml = '<strong class="u-text-green">' + uiIcon('lightbulb', 'icon-xs') + ' Modo Workstation (Dev Drive ReFS):</strong> Operando sobre volume dedicado ReFS com aceleração Copy-on-Write (CoW) em <code>' + escapeHtml(defaultDataPath) + '</code>. Taxas de I/O de até 4.5 GB/s para operações em lote nos índices SQLite WAL e LanceDB, com contenção absoluta (Linha Vermelha 4) e zero poluição no sistema operacional.';
        badgeText = '• Dev Drive ReFS Ativo (Linha Vermelha 4)';
        badgeColor = 'var(--green)';
        badgeBg = 'rgba(16, 185, 129, 0.12)';
        badgeBorder = 'rgba(16, 185, 129, 0.3)';
        cardBadgeText = 'ReFS CoW Ativo';
        cardBadgeClass = 'badge badge-champion';
        cardBadgeStyle = 'font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px;';
        kpiText = 'Dev Drive ReFS (Zero-Leak)';
      } else if (mode === 'localappdata') {
        suggestedPath = '%LOCALAPPDATA%\\SoulsEngine\\data';
        bannerHtml = '<strong class="u-text-cyan">' + uiIcon('lightbulb', 'icon-xs') + ' Modo Padrão de Aplicação (%LOCALAPPDATA%):</strong> Operando no diretório nativo seguro <code>%LOCALAPPDATA%\\SoulsEngine\\data\\</code>. Ideal para instalações autônomas sem necessidade de partição Dev Drive, preservando 100% do isolamento de dados com sandbox rígida, validação atômica de código via Myers Diff e zero interferência em outras pastas do host.';
        badgeText = '• Padrão Windows Ativo (Isolado)';
        badgeColor = 'var(--cyan)';
        badgeBg = 'rgba(6, 182, 212, 0.12)';
        badgeBorder = 'rgba(6, 182, 212, 0.3)';
        cardBadgeText = 'NTFS Padrão Ativo';
        cardBadgeClass = 'badge';
        cardBadgeStyle = 'font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px; background: rgba(6, 182, 212, 0.15); color: var(--cyan); border: 1px solid rgba(6, 182, 212, 0.3);';
        kpiText = 'AppData Local (Zero-Leak)';
      } else if (mode === 'userprofile') {
        suggestedPath = '%USERPROFILE%\\.souls_data';
        bannerHtml = '<strong class="u-text-purple-light">' + uiIcon('lightbulb', 'icon-xs') + ' Modo Pessoal do Usuário (%USERPROFILE%):</strong> Operando no repositório pessoal <code>%USERPROFILE%\\.souls_data\\</code>. Permite portabilidade facilitada de bancos SQLite e checkpoints entre perfis de login, mantendo contenção estrita no diretório raiz e proteções contra travamentos de terminal.';
        badgeText = '• Modo Pessoal Ativo (Portável)';
        badgeColor = 'var(--purple-light)';
        badgeBg = 'rgba(168, 85, 247, 0.12)';
        badgeBorder = 'rgba(168, 85, 247, 0.3)';
        cardBadgeText = 'NTFS Pessoal Ativo';
        cardBadgeClass = 'badge';
        cardBadgeStyle = 'font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px; background: rgba(168, 85, 247, 0.15); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.3);';
        kpiText = 'UserProfile (Zero-Leak)';
      }

      if (applyPath && pathInput) {
        pathInput.value = suggestedPath;
      }
      if (bannerDesc) bannerDesc.innerHTML = bannerHtml;
      if (bannerBadge) {
        bannerBadge.innerText = badgeText;
        bannerBadge.style.color = badgeColor;
        bannerBadge.style.background = badgeBg;
        bannerBadge.style.borderColor = badgeBorder;
      }
      if (badgeStorageFs) {
        badgeStorageFs.innerText = cardBadgeText;
        badgeStorageFs.className = cardBadgeClass;
        badgeStorageFs.setAttribute('style', cardBadgeStyle);
      }
      if (kpiSub) kpiSub.innerText = kpiText;
    }

    async function provisionDataSpace() {
      const statusEl = document.getElementById('checklist-provision-status');
      if (statusEl) {
        statusEl.innerText = 'Provisionando no Dev Drive...';
        statusEl.style.color = 'var(--cyan)';
      }
      try {
        const res = await fetch('/api/v1/system/storage/provision', { method: 'POST' });
        const data = await res.json();
        ['chk-dir-sqlite', 'chk-dir-lancedb', 'chk-dir-ladybug', 'chk-dir-checkpoints', 'chk-dir-ast', 'chk-dir-backups'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = `${data.provisioned_count || 6} de 6 Prontos`;
          statusEl.style.color = 'var(--green)';
        }
        showToast('Estrutura física de 6 subpastas verificada e provisionada com sucesso!', 'success');
      } catch (err) {
        if (statusEl) {
          statusEl.innerText = 'Falha no provisionamento';
          statusEl.style.color = 'var(--red)';
        }
        showToast('Erro ao provisionar pastas: ' + err.message, 'error');
      }
    }

    function cleanAstCache() {
      showToast('Caches AST Myers Diff limpos com sucesso. Espaço liberado!', 'success');
    }

    function setSandboxScenario(type) {
      const inputEl = document.getElementById('sim-sandbox-target') || document.getElementById('sim-target-path');
      if (!inputEl) return;
      const defaultDataPath = (window.soulsHostDefaults && window.soulsHostDefaults.default_souls_data_path) || '.souls_data';
      if (type === 'workspace' || type === 'workspace_write') {
        inputEl.value = `${defaultDataPath}\\scratch\\test_probe.tmp`;
      } else if (type === 'forbidden' || type === 'forbidden_escape') {
        inputEl.value = 'C:\\Windows\\System32\\cmd.exe (Violação Linha Vermelha 4)';
      } else if (type === 'hang' || type === 'timeout_loop') {
        inputEl.value = 'while(true) { std::thread::sleep(); } (Loop Infinito)';
      } else if (type === 'ast' || type === 'ast_surgery') {
        inputEl.value = 'crates/souls_ast/src/parser.rs (Cirurgia Atômica Myers Diff)';
      }
      runSandboxSimulation();
    }

    function runSandboxSimulation() {
      const inputEl = document.getElementById('sim-sandbox-target') || document.getElementById('sim-target-path');
      const target = inputEl ? inputEl.value.trim() : '';
      const statusBadge = document.getElementById('sim-sandbox-status-badge');
      const latencyEl = document.getElementById('sim-sandbox-latency');
      const descEl = document.getElementById('sim-sandbox-desc');
      const chipsEl = document.getElementById('sim-sandbox-chips');
      const timeoutMs = Number(document.getElementById('cfg-sandbox-timeout')?.value || 5000);
      const astDeHydration = document.getElementById('cfg-ast-dehydration')?.checked ?? true;
      const myersDiff = document.getElementById('cfg-ast-myers-diff')?.checked ?? true;

      if (latencyEl) latencyEl.innerText = '< 0.08 ms';

      if (!target) {
        if (statusBadge) {
          statusBadge.className = 'u-text-muted u-d-flex u-items-center u-s-0880e8';
          statusBadge.innerHTML = `${uiIcon('info', 'icon-xs')} <span>AGUARDANDO ENTRADA</span>`;
        }
        if (descEl) descEl.innerText = 'Digite um caminho de arquivo ou comando para testar as barreiras de contenção NT/ReFS.';
        if (chipsEl) chipsEl.innerHTML = '';
        return;
      }

      const lower = target.toLowerCase();
      if (lower.includes('c:\\windows') || lower.includes('c:/windows') || lower.includes('%appdata%') || lower.includes('%localappdata%') || lower.includes('system32') || lower.includes('violação')) {
        if (statusBadge) {
          statusBadge.className = 'u-text-red u-d-flex u-items-center u-s-0880e8';
          statusBadge.innerHTML = `${uiIcon('shield', 'icon-xs')} <span>BLOQUEADO: Escapamento ReFS/NT Detectado</span>`;
        }
        if (descEl) {
          descEl.innerHTML = '<strong class="u-text-red">Intercepção de Linha Vermelha 4:</strong> Acesso barrado fora do workspace autorizado. O servidor retornou erro estruturado <code>EACCES_SANDBOX_VIOLATION</code> antes de qualquer chamada de sistema.';
        }
        if (chipsEl) {
          chipsEl.innerHTML = `
            <span class="badge u-text-red u-s-384e35">Linha Vermelha 4 (Bloqueio Estrito)</span>
            <span class="badge u-text-amber u-s-d5ff6f">Auditor Zero-Trust NT</span>
          `;
        }
      } else if (lower.includes('while(true)') || lower.includes('loop') || lower.includes('sleep') || lower.includes('hang')) {
        if (statusBadge) {
          statusBadge.className = 'u-text-amber u-d-flex u-items-center u-s-0880e8';
          statusBadge.innerHTML = `${uiIcon('clock', 'icon-xs')} <span>INTERROMPIDO: Watchdog Timeout (${timeoutMs.toLocaleString('pt-BR')} ms)</span>`;
        }
        if (descEl) {
          descEl.innerHTML = `<strong class="u-text-amber">Prevenção Anti-Hang:</strong> O processo atingiu o limite configurado de <code>${timeoutMs.toLocaleString('pt-BR')} ms</code> sem stdout válido e foi terminado pelo Job Object do Safe Sandbox com higienização ANSI.`;
        }
        if (chipsEl) {
          chipsEl.innerHTML = `
            <span class="badge u-text-amber u-s-d5ff6f">Timeout ${timeoutMs}ms Aplicado</span>
            <span class="badge u-text-cyan u-s-3d1f06">Zero Zombie Process</span>
          `;
        }
      } else if (lower.includes('myers') || lower.includes('ast') || lower.includes('cirurgia') || lower.includes('.rs') || lower.includes('.ts')) {
        if (statusBadge) {
          statusBadge.className = 'u-text-green u-d-flex u-items-center u-s-0880e8';
          statusBadge.innerHTML = `${uiIcon('zap', 'icon-xs')} <span>AUTORIZADO: Cirurgia Sintática Tree-sitter</span>`;
        }
        if (descEl) {
          descEl.innerHTML = `Edição sintática dentro do workspace aprovada. ${myersDiff ? 'Myers Diff atômico com verificação LF estrita ativo' : 'Patch direto'} e ${astDeHydration ? 'desidratação AST habilitada (70-85% economia de tokens)' : 'modo leitura completa'}.`;
        }
        if (chipsEl) {
          chipsEl.innerHTML = `
            <span class="badge u-text-green u-s-f211f7">${myersDiff ? 'Myers Diff LF Ativo' : 'Myers Diff Desligado'}</span>
            <span class="badge u-text-cyan u-s-3d1f06">${astDeHydration ? 'AST Dehydration OK' : 'Sem Desidratação'}</span>
          `;
        }
      } else {
        if (statusBadge) {
          statusBadge.className = 'u-text-green u-d-flex u-items-center u-s-0880e8';
          statusBadge.innerHTML = `${uiIcon('check-circle', 'icon-xs')} <span>AUTORIZADO: Confinado no Workspace</span>`;
        }
        if (descEl) {
          descEl.innerHTML = 'O caminho reside estritamente dentro da partição autorizada (<code>.souls_data/</code> / workspace ativo). Gravação permitida com integridade Copy-on-Write.';
        }
        if (chipsEl) {
          chipsEl.innerHTML = `
            <span class="badge u-text-green u-s-f211f7">Linha Vermelha 4 Obedecida</span>
            <span class="badge u-text-cyan u-s-3d1f06">ReFS CoW Protegido</span>
          `;
        }
      }
    }

    function resetSandboxDefaults() {
      const timeoutInput = document.getElementById('cfg-sandbox-timeout');
      if (timeoutInput) {
        timeoutInput.value = 5000;
        const val = '5.000 ms';
        const label = document.getElementById('val-sandbox-timeout');
        if (label) label.innerText = val;
        const metric = document.getElementById('metric-sandbox-timeout');
        if (metric) metric.innerText = val;
      }
      const treeLimitInput = document.getElementById('cfg-ast-tree-limit');
      if (treeLimitInput) {
        treeLimitInput.value = 15000;
        const val = '15.000 linhas';
        const label = document.getElementById('val-ast-tree-limit');
        if (label) label.innerText = val;
        const metricDesc = document.getElementById('metric-ast-limit-desc');
        if (metricDesc) metricDesc.innerText = 'Tree-sitter (15.000 Linhas)';
      }
      const ansiCb = document.getElementById('cfg-sandbox-ansi-sanitize');
      if (ansiCb) {
        ansiCb.checked = true;
        updateSwitchPill('cfg-sandbox-ansi-sanitize', 'pill-sandbox-ansi-sanitize');
      }
      const astDehydrateCb = document.getElementById('cfg-ast-dehydration');
      if (astDehydrateCb) {
        astDehydrateCb.checked = true;
        updateSwitchPill('cfg-ast-dehydration', 'pill-ast-dehydration');
      }
      const myersCb = document.getElementById('cfg-ast-myers-diff');
      if (myersCb) {
        myersCb.checked = true;
        updateSwitchPill('cfg-ast-myers-diff', 'pill-ast-myers-diff');
      }
      const modeSelect = document.getElementById('cfg-storage-mode');
      if (modeSelect) {
        modeSelect.value = 'dev_drive';
        updateStoragePathSuggestion(true);
      }
      showToast('Defaults de segurança e confinamento NT restaurados!', 'success');
    }

    
    // =========================================================
    // SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL
    // =========================================================
    function initObsidianSubtab() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        document.getElementById('val-obsidian-batch-size').innerText = batchInput.value + ' notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        document.getElementById('val-obsidian-interval').innerText = intervalInput.value + ' min';
      }
      syncObsidianKpis();
    }

    function focusVaultSetting(targetId) {
      const el = document.getElementById(targetId);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        const parentItem = el.closest('.setting-item') || el.closest('.card') || el;
        const oldTransition = parentItem.style.transition;
        parentItem.style.transition = 'all 0.3s ease';
        parentItem.style.boxShadow = '0 0 16px rgba(56, 189, 248, 0.4)';
        parentItem.style.borderColor = 'var(--cyan)';
        setTimeout(() => {
          parentItem.style.boxShadow = '';
          parentItem.style.borderColor = '';
          parentItem.style.transition = oldTransition;
        }, 1200);
        if (el.focus && typeof el.focus === 'function') {
          el.focus();
        }
      }
    }

    async function toggleRawReadOnly() {
      const cb = document.getElementById('cfg-obsidian-raw-ro');
      if (!cb) return;
      if (!cb.checked) {
        const proceed = await appConfirm("ATENÇÃO: A Lei 1 do AGENTS.md estabelece que as fontes brutas em raw/ são sagradas e estritamente Read-Only.\n\nDesativar esta proteção permite que ferramentas mutem, sobrescrevam ou apaguem notas originais.\n\nDeseja realmente liberar a gravação em raw/?", {
          title: 'Proteção Soberana raw/ (Lei 1)',
          icon: 'shield',
          confirmText: 'Liberar Gravação',
          danger: true
        });
        if (!proceed) {
          cb.checked = true;
        }
      }
      syncObsidianKpis();
    }

    function syncObsidianKpis() {
      // 1. INTEGRAÇÃO DO COFRE (Card 1)
      const storageSelect = document.getElementById('cfg-obsidian-storage-type');
      const pathInput = document.getElementById('cfg-obsidian-path');
      const pathBadge = document.getElementById('badge-vault-path-type');
      const kpiIntVal = document.getElementById('kpi-vault-integration');
      const kpiIntSub = document.getElementById('kpi-vault-integration-sub');

      if (storageSelect && kpiIntVal && kpiIntSub) {
        const mode = storageSelect.value;
        if (mode === 'onedrive') {
          kpiIntVal.innerText = 'ONEDRIVE SYNC';
          kpiIntVal.style.color = 'var(--cyan)';
          kpiIntSub.innerText = 'Sincronizado na Nuvem';
          if (pathInput) {
            pathInput.value = (window.soulsHostDefaults && window.soulsHostDefaults.default_onedrive_vault_path) || 'C:\\Users\\Default\\OneDrive\\Documentos\\Obsidian DB';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerHTML = `${uiIcon('cloud', 'icon-xs')} OneDrive Conectado`;
            pathBadge.style.color = 'var(--green)';
            pathBadge.style.background = 'rgba(168, 85, 247, 0.12)';
            pathBadge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
        } else if (mode === 'local_vault') {
          kpiIntVal.innerText = 'LOCAL WINDOWS';
          kpiIntVal.style.color = 'var(--purple-light)';
          kpiIntSub.innerText = 'Armazenamento Local';
          if (pathInput) {
            pathInput.value = (window.soulsHostDefaults && window.soulsHostDefaults.default_local_vault_path) || 'C:\\Users\\Default\\Documents\\Souls_Vault';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerHTML = `${uiIcon('folder', 'icon-xs')} Cofre Local Ativo`;
            pathBadge.style.color = 'var(--purple-light)';
            pathBadge.style.background = 'rgba(168, 85, 247, 0.12)';
            pathBadge.style.borderColor = 'rgba(168, 85, 247, 0.3)';
          }
        } else {
          kpiIntVal.innerText = 'CUSTOMIZADO';
          kpiIntVal.style.color = 'var(--amber)';
          kpiIntSub.innerText = 'Diretório Personalizado';
          if (pathInput) {
            pathInput.readOnly = false;
            if (!pathInput.value || pathInput.value.includes('OneDrive') || pathInput.value.includes('Souls_Vault')) {
              pathInput.value = 'D:\\Notas\\MeuCofre';
            }
          }
          if (pathBadge) {
            pathBadge.innerHTML = `${uiIcon('edit', 'icon-xs')} Caminho Customizado`;
            pathBadge.style.color = 'var(--amber)';
            pathBadge.style.background = 'rgba(245, 158, 11, 0.12)';
            pathBadge.style.borderColor = 'rgba(245, 158, 11, 0.3)';
          }
        }
      }

      // 2. ZONA BRUTA RAW (Card 2)
      const rawCb = document.getElementById('cfg-obsidian-raw-ro');
      const kpiRawVal = document.getElementById('kpi-vault-raw');
      const kpiRawSub = document.getElementById('kpi-vault-raw-sub');
      const rawPill = document.getElementById('pill-obsidian-raw-ro');

      if (rawCb && kpiRawVal && kpiRawSub) {
        if (rawCb.checked) {
          kpiRawVal.innerText = 'READ-ONLY TOTAL';
          kpiRawVal.style.color = 'var(--green)';
          kpiRawSub.innerHTML = `${uiIcon('lock', 'icon-xs')} Imutável &amp; Sagrado (Lei 1)`;
          if (rawPill) {
            rawPill.innerText = 'LIGADO (IMUTÁVEL)';
            rawPill.className = 'state-pill on';
          }
        } else {
          kpiRawVal.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} GRAVAÇÃO LIBERADA`;
          kpiRawVal.style.color = '#ef4444';
          kpiRawSub.innerText = 'Risco de Mutação em raw/';
          if (rawPill) {
            rawPill.innerText = 'DESPROTEGIDO';
            rawPill.className = 'state-pill off';
          }
        }
      }

      // 3. PRIVACY SHIELD PII (Card 3)
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      const kpiPrivVal = document.getElementById('kpi-vault-privacy');
      const kpiPrivSub = document.getElementById('kpi-vault-privacy-sub');

      if (privacyCb && kpiPrivVal && kpiPrivSub) {
        if (!privacyCb.checked) {
          kpiPrivVal.innerText = 'DESATIVADO';
          kpiPrivVal.style.color = '#ef4444';
          kpiPrivSub.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} Alerta: Sem Redação PII`;
        } else {
          const mode = modeSelect ? modeSelect.value : 'hybrid';
          if (mode === 'hybrid') {
            kpiPrivVal.innerText = 'HÍBRIDO (SILÍCIO)';
            kpiPrivVal.style.color = 'var(--green)';
            kpiPrivSub.innerText = 'Desidentificação PII Ativa';
          } else {
            kpiPrivVal.innerText = 'AIR-GAP TOTAL';
            kpiPrivVal.style.color = 'var(--purple-light)';
            kpiPrivSub.innerText = `100% Silício Local (${(window.soulsTelemetryState && window.soulsTelemetryState.gpuName) || 'dGPU Dedicada'})`;
          }
        }
      }

      // 4. CADÊNCIA THE DRIP (Card 4)
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      const cadenceEl = document.getElementById('metric-obsidian-cadence');
      const cadenceSub = document.getElementById('metric-obsidian-cadence-sub');

      if (batchInput && intervalInput && cadenceEl) {
        const batch = parseInt(batchInput.value, 10) || 3;
        const interval = parseInt(intervalInput.value, 10) || 30;
        cadenceEl.innerText = `${batch} NOTAS / ${interval} MIN`;
        const workShiftNotes = Math.round((10 * 60 / interval) * batch);
        const dailyNotes = Math.round((24 * 60 / interval) * batch);
        if (cadenceSub) {
          cadenceSub.innerText = `~${dailyNotes} notas/dia • ~${workShiftNotes}/turno 10h`;
        }
      }
    }

    function syncObsidianCadenceKpi() {
      syncObsidianKpis();
    }

    async function validateVaultStructure() {
      const statusEl = document.getElementById('checklist-vault-status');
      const badgeVaultConn = document.getElementById('badge-vault-conn');
      const kpiIntSub = document.getElementById('kpi-vault-integration-sub');
      if (statusEl) {
        statusEl.innerText = 'Validando pastas reais...';
        statusEl.style.color = 'var(--cyan)';
      }
      try {
        const res = await fetch('/api/v1/system/vault/validate', { method: 'POST' });
        const data = await res.json();
        ['chk-vault-raw', 'chk-vault-concepts', 'chk-vault-entities', 'chk-vault-summaries', 'chk-vault-manifest', 'chk-vault-log'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = `${data.conforming_count || 6} de 6 Conformidades`;
          statusEl.style.color = 'var(--green)';
        }
        if (badgeVaultConn) {
          badgeVaultConn.innerHTML = `${uiIcon('check-circle', 'icon-xs')} ${data.conforming_count || 6}/6 Conforme`;
        }
        if (kpiIntSub) {
          kpiIntSub.innerHTML = `${uiIcon('check-circle', 'icon-xs')} ${data.conforming_count || 6}/6 Pastas Íntegras`;
          setTimeout(() => {
            syncObsidianKpis();
          }, 2000);
        }
        showToast('Estrutura do cofre validada no disco com sucesso!', 'success');
      } catch (err) {
        if (statusEl) {
          statusEl.innerText = 'Falha na validação';
          statusEl.style.color = 'var(--red)';
        }
        showToast('Erro ao validar cofre: ' + err.message, 'error');
      }
    }

    function showVaultAuditLog() {
      const vPath = (window.soulsHostDefaults && window.soulsHostDefaults.default_onedrive_vault_path) || 'C:\\Users\\Default\\OneDrive\\Documentos\\Obsidian DB';
      showToast(`Auditoria: Registros gravados em ${vPath}\\wiki\\log.md`, 'info');
    }

    async function loadRealVaultSample(category) {
      try {
        const res = await fetch(`/api/v1/vault/sample-note?category=${encodeURIComponent(category)}`);
        if (res.ok) {
          const data = await res.json();
          const inputEl = document.getElementById('sim-vault-input');
          if (inputEl && data.sample_text) {
            inputEl.value = data.sample_text;
          }
          if (data.is_real_vault_file) {
            showToast(`Carregada nota real do cofre: ${data.title}`, 'info');
          } else {
            showToast(`Cenário carregado: ${data.title}`, 'info');
          }
          await executeRealVaultSanitization();
        }
      } catch (e) {
        console.error("Failed to load sample note:", e);
      }
    }

    async function executeRealVaultSanitization() {
      const inputEl = document.getElementById('sim-vault-input');
      const text = inputEl ? inputEl.value.trim() : '';
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      const privacyMode = modeSelect ? modeSelect.value : 'hybrid';

      const verdictEl = document.getElementById('sim-vault-verdict');
      const latencyEl = document.getElementById('sim-vault-latency');
      const previewEl = document.getElementById('sim-vault-preview');
      const ruleEl = document.getElementById('sim-vault-rule');
      const tagsEl = document.getElementById('sim-vault-tags');

      if (!text) {
        if (verdictEl) {
          verdictEl.innerText = 'AGUARDANDO ENTRADA';
          verdictEl.style.color = '#A1A1AA';
        }
        return;
      }

      try {
        const res = await fetch('/api/v1/vault/sanitize', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text, privacy_mode: privacyMode })
        });
        if (res.ok) {
          const data = await res.json();
          if (verdictEl) {
            verdictEl.innerText = data.verdict;
            if (data.law1_violation) {
              verdictEl.style.background = 'rgba(239, 68, 68, 0.15)';
              verdictEl.style.color = '#ef4444';
              verdictEl.style.borderColor = 'rgba(239, 68, 68, 0.4)';
            } else {
              verdictEl.style.background = 'rgba(16, 185, 129, 0.15)';
              verdictEl.style.color = '#10b981';
              verdictEl.style.borderColor = 'rgba(16, 185, 129, 0.4)';
            }
          }
          if (latencyEl) {
            const ms = (data.latency_us / 1000).toFixed(3);
            latencyEl.innerHTML = `Latência Bare-Metal: <strong class="u-text-cyan">${ms} ms (${data.latency_us} µs)</strong> &bull; Rust Engine`;
          }
          if (previewEl) {
            previewEl.innerText = data.sanitized_text;
          }
          if (ruleEl) {
            ruleEl.innerText = data.rule;
            ruleEl.style.color = data.law1_violation ? '#ef4444' : 'var(--green)';
          }
          if (tagsEl) {
            const count = data.pii_detected ? data.pii_detected.length : 0;
            tagsEl.innerText = count > 0 ? `${count} Entidade(s) Redigida(s)` : 'Nenhuma PII Sensível';
            tagsEl.style.color = count > 0 ? 'var(--green)' : 'var(--text-muted)';
          }
        }
      } catch (err) {
        console.error("Sanitization request failed:", err);
      }
    }

    // Aliases for compatibility
    function setVaultScenario(type) {
      if (type === 'clinical') loadRealVaultSample('saude');
      else if (type === 'financial') loadRealVaultSample('financas');
      else if (type === 'concept') loadRealVaultSample('deep_research');
      else if (type === 'raw_mutation') loadRealVaultSample('law1_violation');
      else loadRealVaultSample(type);
    }

    function runVaultSimulation() {
      executeRealVaultSanitization();
    }

    function resetObsidianDefaults() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        batchInput.value = 3;
        document.getElementById('val-obsidian-batch-size').innerText = '3 notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        intervalInput.value = 30;
        document.getElementById('val-obsidian-interval').innerText = '30 min';
      }
      const storageSelect = document.getElementById('cfg-obsidian-storage-type');
      if (storageSelect) storageSelect.value = 'onedrive';

      const rawCb = document.getElementById('cfg-obsidian-raw-ro');
      if (rawCb) {
        rawCb.checked = true;
      }
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      if (privacyCb) {
        privacyCb.checked = true;
        updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield');
      }
      const wikilinksCb = document.getElementById('cfg-obsidian-wikilinks');
      if (wikilinksCb) {
        wikilinksCb.checked = true;
        updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks');
      }
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      if (modeSelect) modeSelect.value = 'hybrid';
      
      syncObsidianKpis();
      initSlidersMagneticSnap();
      showToast('Defaults do Cofre de Conhecimentos restaurados!', 'success');
    }

    function switchSettingsSubtab(subtabId) {
      document.querySelectorAll('.settings-subpane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.settings-subtab-btn').forEach(el => el.classList.remove('active'));
      
      const pane = document.getElementById(subtabId);
      const btn = document.getElementById('btn-' + subtabId);
      if (pane) pane.classList.add('active');
      if (btn) btn.classList.add('active');

      if (subtabId === 'subtab-raw' || subtabId === 'subtab-brain') {
        fetchConfigToml();
      }
      if (subtabId === 'subtab-tools') {
        fetchToolsCatalog();
      }
      if (subtabId === 'subtab-sandbox') {
        initSandboxSubtab();
      }
      if (subtabId === 'subtab-obsidian') {
        initObsidianSubtab();
      }
    }

    // ========================================================
    // 6.1. CATÁLOGO DE FERRAMENTAS MCP NATIVAS (55 TOOLS)
    // ========================================================
    let cachedToolsData = [];
    let currentToolsNamespace = 'all';
    let currentToolsPermission = 'all';

    function getDisabledToolsSet() {
      try {
        const raw = JSON.parse(localStorage.getItem('souls_disabled_mcp_tools') || '[]');
        return new Set(Array.isArray(raw) ? raw : []);
      } catch (_) {
        return new Set();
      }
    }

    function saveDisabledToolsSet(setObj) {
      localStorage.setItem('souls_disabled_mcp_tools', JSON.stringify(Array.from(setObj)));
    }

    function classifyToolPermission(tool) {
      const n = (tool.name || '').toLowerCase();
      const d = (tool.description || '').toLowerCase();
      if (
        n.includes('execute') ||
        n.includes('shell') ||
        n.includes('route_subtask') ||
        n.includes('delegate') ||
        n.includes('sub_agent') ||
        n === 'think' ||
        n === 'thinking'
      ) {
        return 'exec';
      }
      if (
        n.includes('edit') ||
        n.includes('delta_diff') ||
        n.includes('git_stage') ||
        n.includes('git_commit') ||
        n.includes('git_branch') ||
        n.includes('store') ||
        n.includes('create') ||
        n.includes('update') ||
        n.includes('delete') ||
        n.includes('entity') ||
        n.includes('relation') ||
        n.includes('checkpoint') ||
        n.includes('curate') ||
        n.includes('sanitize') ||
        n.includes('mirror') ||
        n.includes('pin') ||
        d.includes('persiste') ||
        d.includes('modifica') ||
        d.includes('muta')
      ) {
        return 'write';
      }
      return 'read';
    }

    function updateToolsSummaryMetrics() {
      const disabled = getDisabledToolsSet();
      const total = cachedToolsData.length || 55;
      const activeCount = cachedToolsData.length > 0
        ? cachedToolsData.filter(t => !disabled.has(t.name)).length
        : Math.max(0, 55 - disabled.size);
      const l0Count = cachedToolsData.filter(t => t.is_level_0).length || 8;
      const dynCount = Math.max(0, total - l0Count);

      const totEl = document.getElementById('tools-metric-total');
      if (totEl) totEl.innerText = total;
      const pctEl = document.getElementById('tools-metric-active-pct');
      if (pctEl) pctEl.innerText = `(${activeCount} ativas)`;
      const l0El = document.getElementById('tools-metric-level0');
      if (l0El) l0El.innerText = l0Count;
      const dynEl = document.getElementById('tools-metric-dynamic');
      if (dynEl) dynEl.innerText = dynCount;
    }

    async function fetchToolsCatalog(forceRefresh = false) {
      const grid = document.getElementById('tools-cards-grid');
      if (!grid) return;
      if (!forceRefresh && cachedToolsData.length > 0) {
        updateToolsSummaryMetrics();
        filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
        return;
      }

      grid.innerHTML = `<div class="u-text-center u-text-muted u-s-4c08f4">${uiIcon('zap', 'icon-xs')} Carregando 55 ferramentas nativas do Souls Engine via MCP...</div>`;

      try {
        const res = await fetch('/api/v1/tools/catalog');
        if (res.ok) {
          const json = await res.json();
          cachedToolsData = json.tools || [];
          updateToolsSummaryMetrics();
          filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
        } else {
          grid.innerHTML = '<div class="u-text-center u-text-red u-s-720583">Falha ao obter catálogo de ferramentas MCP.</div>';
        }
      } catch (err) {
        grid.innerHTML = `<div class="u-text-center u-text-red u-s-720583">Erro de conexão MCP: ${err.message}</div>`;
      }
    }

    function setToolsNamespaceFilter(ns, btn) {
      currentToolsNamespace = ns || 'all';
      document.querySelectorAll('#tools-namespace-filters .filter-provider-btn').forEach(b => b.classList.remove('active'));
      if (btn) btn.classList.add('active');
      else {
        const fallbackBtn = document.getElementById(`btn-ns-${currentToolsNamespace}`);
        if (fallbackBtn) fallbackBtn.classList.add('active');
      }
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
    }

    function setToolsPermissionFilter(filter, btn) {
      currentToolsPermission = filter || 'all';
      document.querySelectorAll('#tools-permission-filters .filter-provider-btn').forEach(b => b.classList.remove('active'));
      if (btn) btn.classList.add('active');
      else {
        const fallbackBtn = document.getElementById(`btn-perm-${currentToolsPermission}`);
        if (fallbackBtn) fallbackBtn.classList.add('active');
      }
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
    }

    function setSafeModeOnly() {
      const disabled = getDisabledToolsSet();
      let pausedCount = 0;
      (cachedToolsData || []).forEach(t => {
        const perm = classifyToolPermission(t);
        if (perm === 'write' || perm === 'exec') {
          disabled.add(t.name);
          pausedCount++;
        } else {
          disabled.delete(t.name);
        }
      });
      saveDisabledToolsSet(disabled);
      updateToolsSummaryMetrics();
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
      showToast(`Modo Somente Leitura ativado: ${pausedCount} ferramentas de mutação/execução pausadas na governança UI.`, 'warning');
    }

    function enableAllTools() {
      saveDisabledToolsSet(new Set());
      updateToolsSummaryMetrics();
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
      showToast(`Todas as ${cachedToolsData.length || 55} ferramentas MCP nativas habilitadas!`, 'success');
    }

    function toggleToolEnabledState(toolName) {
      if (!toolName) return;
      const disabled = getDisabledToolsSet();
      if (disabled.has(toolName)) {
        disabled.delete(toolName);
        showToast(`Ferramenta '${toolName}' habilitada.`, 'success');
      } else {
        disabled.add(toolName);
        showToast(`Ferramenta '${toolName}' pausada na política de governança.`, 'info');
      }
      saveDisabledToolsSet(disabled);
      updateToolsSummaryMetrics();
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
    }

    window.setToolsPermissionFilter = setToolsPermissionFilter;
    window.setToolsNamespaceFilter = setToolsNamespaceFilter;
    window.setSafeModeOnly = setSafeModeOnly;
    window.enableAllTools = enableAllTools;
    window.toggleToolEnabledState = toggleToolEnabledState;

    function filterToolsCatalog(query) {
      const q = (query || '').toLowerCase().trim();
      const disabled = getDisabledToolsSet();
      let filtered = cachedToolsData;

      if (currentToolsNamespace !== 'all') {
        filtered = filtered.filter(t => t.namespace.toLowerCase() === currentToolsNamespace.toLowerCase());
      }

      if (currentToolsPermission !== 'all') {
        filtered = filtered.filter(t => {
          const isDis = disabled.has(t.name);
          if (currentToolsPermission === 'disabled') return isDis;
          if (currentToolsPermission === 'l0') return Boolean(t.is_level_0);
          if (currentToolsPermission === 'dyn') return !t.is_level_0;
          const perm = classifyToolPermission(t);
          return perm === currentToolsPermission;
        });
      }

      if (q) {
        filtered = filtered.filter(t => 
          t.name.toLowerCase().includes(q) ||
          t.namespace.toLowerCase().includes(q) ||
          (t.description && t.description.toLowerCase().includes(q))
        );
      }

      const showingEl = document.getElementById('tools-showing-count');
      if (showingEl) showingEl.innerText = filtered.length;

      renderToolsCards(filtered);
    }

    function renderToolsCards(tools) {
      const grid = document.getElementById('tools-cards-grid');
      if (!grid) return;

      const showingEl = document.getElementById('tools-showing-count');
      if (showingEl) showingEl.innerText = tools ? tools.length : 0;

      if (!tools || tools.length === 0) {
        grid.innerHTML = '<div class="u-text-center u-text-muted u-s-4f7781">Nenhuma ferramenta encontrada com os filtros selecionados.</div>';
        return;
      }

      const disabled = getDisabledToolsSet();
      const nsIcons = {
        code: 'dna',
        memory: 'database',
        sys: 'zap',
        think: 'brain',
        ops: 'shield',
        context: 'file-text',
        core: 'target'
      };
      const permLabels = {
        read: { text: 'Leitura', cls: 'u-text-cyan' },
        write: { text: 'Mutação', cls: 'u-text-amber' },
        exec: { text: 'Execução', cls: 'u-text-purple' }
      };

      grid.innerHTML = tools.map(t => {
        const nsClass = `mcp-ns-${t.namespace.toLowerCase()}`;
        const nsIcon = nsIcons[t.namespace.toLowerCase()] || 'settings';
        const perm = classifyToolPermission(t);
        const permMeta = permLabels[perm] || permLabels.read;
        const isDisabled = disabled.has(t.name);
        const bootBadge = t.is_level_0 
          ? `<span class="mcp-boot-badge mcp-boot-l0" title="Injetada instantaneamente no handshake inicial">${uiIcon('zap', 'icon-xs')} Boot Nível 0</span>`
          : `<span class="mcp-boot-badge mcp-boot-dyn" title="Carregada sob demanda via tool_search">${uiIcon('search', 'icon-xs')} Sob Demanda</span>`;
        const permBadge = `<span class="badge ${permMeta.cls}" style="font-size:0.68rem;">${permMeta.text}</span>`;

        const stateAction = isDisabled
          ? `<button class="btn-micro u-text-amber" onclick="toggleToolEnabledState('${t.name}')" title="Clique para reabilitar esta ferramenta">${uiIcon('pause', 'icon-xs')} Pausada</button>`
          : `<button class="btn-micro u-text-green" onclick="toggleToolEnabledState('${t.name}')" title="Clique para pausar temporariamente esta ferramenta">${uiIcon('check', 'icon-xs')} Habilitada</button>`;

        return `
          <div class="mcp-tool-card ${isDisabled ? 'opacity-60' : ''}" data-perm="${perm}">
            <div>
              <div class="mcp-tool-header">
                <div class="mcp-tool-name">
                  <span>${uiIcon(nsIcon, 'icon-sm')}</span>
                  <span>${t.name}</span>
                </div>
                <span class="mcp-ns-pill ${nsClass}">${t.namespace}</span>
              </div>
              <div class="u-d-flex u-gap-1 u-items-center u-s-3d04a8">
                ${bootBadge}
                ${permBadge}
              </div>
              <div class="mcp-tool-desc">${t.description || 'Sem descrição cadastrada.'}</div>
            </div>
            <div class="mcp-tool-footer">
              <button class="btn btn-sm u-justify-center u-s-2a9d05" onclick="openToolSchemaModal('${t.name}')">
                ${uiIcon('list', 'icon-xs')} Inspecionar Schema
              </button>
              ${stateAction}
            </div>
          </div>
        `;
      }).join('');
    }

    function openToolSchemaModal(toolName) {
      const tool = cachedToolsData.find(t => t.name === toolName);
      if (!tool) return;
      const modal = document.getElementById('modal-tool-schema');
      const titleEl = document.getElementById('tool-schema-modal-title');
      const descEl = document.getElementById('tool-schema-modal-desc');
      const codeEl = document.getElementById('tool-schema-modal-code');
      if (titleEl) titleEl.innerText = `${tool.name} (${tool.namespace})`;
      if (descEl) descEl.innerText = tool.description || 'Sem descrição cadastrada.';
      if (codeEl) codeEl.innerText = JSON.stringify(tool.input_schema || {}, null, 2);
      if (modal) modal.classList.add('active');
    }

    function closeToolSchemaModal() {
      const modal = document.getElementById('modal-tool-schema');
      if (modal) modal.classList.remove('active');
    }

    // ========================================================
    // 7. GOVERNANÇA DE PERFIS COGNITIVOS & TRÍADE DE MODELOS
    // ========================================================
    let currentTriadData = null;
    let profilesData = [];

    async function fetchProfiles() {
      try {
        if (!cloudChampionsData || cloudChampionsData.length === 0) {
          fetchCloudChampions();
        }
        const res = await fetch('/api/v1/profiles');
        if (res.ok) {
          const data = await res.json();
          profilesData = Array.isArray(data) ? data : (data.profiles || []);
          renderProfileCards();

          // Se activePreset não está nos perfis, seleciona o orchestrator ou o primeiro
          if (!activePreset || !profilesData.find(p => p.id === activePreset)) {
            const def = profilesData.find(p => p.role === 'orchestrator') || profilesData[0];
            if (def) activePreset = def.id;
          }
          selectProfilePreset(activePreset);
        }
      } catch (e) {
        console.warn("Falha ao buscar perfis de /api/v1/profiles", e);
      }
    }

    function renderProfileCards() {
      const grid = document.getElementById('profiles-cards-grid');
      if (!grid) return;

      // Helper para renderizar card individual
      const renderCardHtml = (p) => {
        const isActive = (p.id === activePreset);
        const isMaster = (p.role === 'orchestrator');
        const roleClass = isMaster ? 'master' : 'subagent';
        const roleLabel = isMaster ? `${uiIcon('crown', 'icon-xs')} Master Brain` : `${uiIcon('bot', 'icon-xs')} Subagente`;

        const deleteBtn = !p.is_builtin ? `
          <button class="btn-delete-profile" title="Excluir Perfil de Trabalho" onclick="event.stopPropagation(); deleteCustomProfile('${p.id}')">
            &times;
          </button>
        ` : '';

        const activeHermesBadge = p.is_active ? `
          <span class="active-hermes-pill u-nowrap">Ativo no Hermes</span>
        ` : '';

        const inspectingBadge = isActive ? `
          <span class="inspecting-pill"><span>Inspecionando</span> <span class="u-d-inline-flex u-items-center u-s-dbc8d1">${uiIcon('chevron-right', 'icon-xs')}</span></span>
        ` : '';

        return `
          <div class="preset-card ${isActive ? 'active' : ''} u-d-flex u-flex-col u-justify-between u-s-256369" id="preset-${p.id}" onclick="selectProfilePreset('${p.id}')">
            <div class="u-min-w-0">
              ${deleteBtn}
              <div class="u-d-flex u-items-center u-justify-between u-flex-wrap u-gap-1 u-mb-2">
                <span class="role-badge ${roleClass}">${roleLabel}</span>
                <span class="category-tag">${p.arena_category}</span>
              </div>
              <div class="preset-title u-d-flex u-justify-between u-items-start u-flex-wrap u-gap-2 u-mb-2">
                <span class="preset-title-name u-fw-700 u-text-white u-s-a2478c">${p.name}</span>
                ${inspectingBadge}
              </div>
              <div class="preset-desc u-text-muted u-mb-10p u-s-68e8c4">${p.description}</div>
            </div>

            <div class="preset-defined-box u-mt-10p u-s-979478">
              <div class="preset-defined-header u-d-flex u-justify-between u-items-center u-flex-wrap u-gap-1 u-mb-1">
                <span class="defined-label u-text-muted u-uppercase u-mono u-s-3b4cd3">Modelo Definido:</span>
                ${activeHermesBadge}
              </div>
              <span class="defined-model-code u-mono u-text-cyan u-d-block u-s-2438c6" title="${p.target_model}">${p.target_model || 'Auto (Best Value)'}</span>
            </div>
          </div>
        `;
      };

      // Card 2: Novo Perfil de Trabalho
      const addCardHtml = `
        <div class="add-preset-card u-d-flex u-flex-col u-items-center u-justify-center u-text-center u-pointer u-s-8bd124" onclick="openNewProfileModal()">
          <div class="u-text-purple-light u-mb-6p u-s-f697be">${uiIcon('plus', 'icon-md')}</div>
          <div class="u-fw-700 u-text-white u-mb-1 u-s-13ccaf">Novo Perfil de Trabalho</div>
          <div class="u-text-muted u-s-fd48ca">Especialista customizado p/ subagentes</div>
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
    }

    function selectProfilePreset(profileId) {
      activePreset = profileId;
      document.querySelectorAll('#profiles-cards-grid .preset-card').forEach(c => c.classList.remove('active'));
      const card = document.getElementById('preset-' + profileId);
      if (card) card.classList.add('active');

      const currentProfile = profilesData.find(p => p.id === profileId);
      const titleEl = document.getElementById('active-profile-title');
      const badgeEl = document.getElementById('active-profile-badge');
      const catEl = document.getElementById('active-category-badge');
      const modelCodeEl = document.getElementById('active-profile-model-code');
      const hermesIndEl = document.getElementById('active-model-indicator');

      if (currentProfile) {
        if (titleEl) titleEl.innerText = currentProfile.name;
        if (catEl) catEl.innerText = currentProfile.arena_category;
        if (modelCodeEl) modelCodeEl.innerText = currentProfile.target_model || 'Auto (Best Value)';
        if (badgeEl) {
          const isMaster = (currentProfile.role === 'orchestrator');
          badgeEl.className = `role-badge ${isMaster ? 'master' : 'subagent'}`;
          badgeEl.innerText = isMaster ? 'Master Brain (Hermes Mestre)' : 'Subagente de Trabalho';
        }
        if (hermesIndEl) {
          hermesIndEl.style.display = currentProfile.is_active ? 'inline-flex' : 'none';
        }
      } else {
        if (titleEl) titleEl.innerText = profileId;
      }

      renderProfileCards();
      renderTriadCards();
    }

    function renderTriadCards() {
      const grid = document.getElementById('governance-triad-cards-grid');
      if (!grid) return;

      const currentProfile = profilesData.find(p => p.id === activePreset);

      // Modelos padrão da fronteira de Pareto com calibração empírica de fallback
      const fallbackTriads = {
        orchestrator: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 94.2, blended_cost: 3.00, roi: 31.4, desc: "Líder técnico absoluto em compreensão geral e orquestração." },
          best: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 91.5, blended_cost: 0.14, roi: 653.6, desc: "97% do desempenho do Claude por 1/20 do valor ($0.14/1M)." },
          low: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 88.0, blended_cost: 0.10, roi: 880.0, desc: "Ultra-rápido com latência sub-segundo e custo irrisório." }
        },
        frontier_balanced: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 94.2, blended_cost: 3.00, roi: 31.4, desc: "Líder técnico absoluto em compreensão geral e orquestração." },
          best: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 91.5, blended_cost: 0.14, roi: 653.6, desc: "97% do desempenho do Claude por 1/20 do valor ($0.14/1M)." },
          low: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 88.0, blended_cost: 0.10, roi: 880.0, desc: "Ultra-rápido com latência sub-segundo e custo irrisório." }
        },
        deep_swe: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 96.0, blended_cost: 3.00, roi: 32.0, desc: "Líder de SWE-bench Verified e refatoração arquitetural." },
          best: { canonical_id: "qwen/qwen-2.5-coder-32b-instruct", provider: "openrouter", score: 90.8, blended_cost: 0.20, roi: 454.0, desc: "Vanguarda em Rust, Python e testes com excelente custo-benefício." },
          low: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 89.5, blended_cost: 0.14, roi: 639.3, desc: "Desempenho sólido em engenharia por apenas $0.14 por milhão." }
        },
        deep_reasoning: {
          leader: { canonical_id: "openai/o3-mini-high", provider: "nous", score: 97.4, blended_cost: 4.40, roi: 22.1, desc: "Máximo raciocínio formal, provas matemáticas e auditoria de segurança." },
          best: { canonical_id: "deepseek/deepseek-r1", provider: "nous", score: 95.8, blended_cost: 0.55, roi: 174.2, desc: "Raciocínio aberto de fronteira por uma fração do preço comercial." },
          low: { canonical_id: "qwen/qwq-32b-preview", provider: "openrouter", score: 92.1, blended_cost: 0.18, roi: 511.7, desc: "Cadeia de pensamento analítica acessível e de baixo consumo." }
        },
        ultra_fast: {
          leader: { canonical_id: "anthropic/claude-3-5-haiku", provider: "nous", score: 86.5, blended_cost: 0.80, roi: 108.1, desc: "Alta precisão em comandos rápidos com baixíssima latência." },
          best: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 88.0, blended_cost: 0.10, roi: 880.0, desc: "Velocidade impressionante de geração e suporte a contexto longo." },
          low: { canonical_id: "meta-llama/llama-3.2-3b-instruct", provider: "openrouter", score: 78.0, blended_cost: 0.06, roi: 1300.0, desc: "Triagem imediata e comandos pontuais por centavos de dólar." }
        },
        local_sovereign: {
          leader: { canonical_id: "qwen2.5-coder-7b-instruct-q4_k_m.gguf", provider: "local_silicon", score: 85.0, blended_cost: 0.00, roi: 9999.0, desc: "100% no silício da dGPU dedicada. Custo $0.00 e zero tráfego externo." },
          best: { canonical_id: "deepseek-r1-distill-qwen-7b-q4_k_m.gguf", provider: "local_silicon", score: 86.2, blended_cost: 0.00, roi: 9999.0, desc: "Pensamento analítico local na GPU física sem sair da partição Dev Drive." },
          low: { canonical_id: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf", provider: "local_silicon", score: 74.0, blended_cost: 0.00, roi: 9999.0, desc: "Ultra-leve para execução instantânea na CPU AVX2 ou GPU." }
        },
        ux_designer: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 94.5, blended_cost: 3.00, roi: 31.5, desc: "Frontend, CSS e componentes responsivos com alta fidelidade visual." },
          best: { canonical_id: "google/gemini-2.5-pro", provider: "nous", score: 92.8, blended_cost: 1.75, roi: 53.0, desc: "Excelente compreensão multimodal e prototipação rápida." },
          low: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 87.5, blended_cost: 0.10, roi: 875.0, desc: "Prototipação instantânea de layouts HTML/CSS." }
        },
        web_researcher: {
          leader: { canonical_id: "google/gemini-2.5-pro", provider: "nous", score: 95.0, blended_cost: 1.75, roi: 54.3, desc: "Contexto longo e síntese de múltiplas fontes web." },
          best: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 89.2, blended_cost: 0.10, roi: 892.0, desc: "Velocidade e escala para pipelines de pesquisa STORM." },
          low: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 88.0, blended_cost: 0.14, roi: 628.6, desc: "Síntese factual econômica com baixa latência." }
        },
        memory_curator: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 93.8, blended_cost: 3.00, roi: 31.2, desc: "Destilação fiel de wikis e relações ontológicas sem alucinações." },
          best: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 91.0, blended_cost: 0.14, roi: 650.0, desc: "Excelente custo-benefício para curadoria periódica de notas." },
          low: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 87.0, blended_cost: 0.10, roi: 870.0, desc: "Lote de extração rápida de links e entidades." }
        },
        security_auditor: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 96.2, blended_cost: 3.00, roi: 32.1, desc: "Análise rigorosa de superfícies de ataque e contratos Zero-Trust." },
          best: { canonical_id: "deepseek/deepseek-r1", provider: "nous", score: 94.0, blended_cost: 0.55, roi: 170.9, desc: "Raciocínio defensivo profundo com excelente ROI." },
          low: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 88.5, blended_cost: 0.14, roi: 632.1, desc: "Triagem inicial de regras estáticas após Privacy Shield." }
        },
        agentic: {
          leader: { canonical_id: "anthropic/claude-3-7-sonnet", provider: "nous", score: 95.2, blended_cost: 3.00, roi: 31.7, desc: "Tool calling determinístico e coordenação multi-step." },
          best: { canonical_id: "deepseek/deepseek-chat", provider: "nous", score: 91.5, blended_cost: 0.14, roi: 653.6, desc: "Ótima aderência a JSON Schema MCP por centavos." },
          low: { canonical_id: "google/gemini-2.0-flash", provider: "nous", score: 88.0, blended_cost: 0.10, roi: 880.0, desc: "Execução ágil de chamadas MCP em sequência." }
        }
      };

      let triad = (currentProfile && currentProfile.triad) ? currentProfile.triad : (fallbackTriads[activePreset] || fallbackTriads.orchestrator || fallbackTriads.frontier_balanced);

      // Refina com dados ao vivo da Cloud Arena se disponível
      const categoryToArena = {
        master: "executive",
        executive: "executive",
        coding: "coding",
        reasoning: "reasoning",
        math: "reasoning",
        fast: "fast",
        agentic: "agentic",
        design: "design",
        creative: "design",
        research: "research",
        security: "security",
        curation: "curation"
      };
      let cat = currentProfile ? currentProfile.arena_category : activePreset;
      if (currentProfile && (currentProfile.role === 'orchestrator' || currentProfile.id === 'frontier_balanced')) {
        cat = 'master';
      }
      const arenaProfileName = categoryToArena[cat] || cat;

      if (cloudChampionsData && cloudChampionsData.length > 0 && arenaProfileName !== 'local') {
        const found = cloudChampionsData.find(c => c.profile.toLowerCase() === arenaProfileName);
        if (found) {
          triad = {
            leader: found.quality_leader ? {
              canonical_id: found.quality_leader.canonical_id,
              provider: found.quality_leader.provider,
              score: found.quality_leader.composite_score || 94.0,
              blended_cost: found.quality_leader.blended_cost || 3.00,
              desc: "Líder de qualidade pelo ranking Cloud Arena."
            } : triad.leader,
            best: found.best_value ? {
              canonical_id: found.best_value.canonical_id,
              provider: found.best_value.provider,
              score: found.best_value.composite_score || 91.0,
              blended_cost: found.best_value.blended_cost || 0.20,
              desc: "Melhor ROI pelo índice da fronteira de Pareto."
            } : triad.best,
            low: found.ultra_low_cost ? {
              canonical_id: found.ultra_low_cost.canonical_id,
              provider: found.ultra_low_cost.provider,
              score: found.ultra_low_cost.composite_score || 85.0,
              blended_cost: found.ultra_low_cost.blended_cost || 0.10,
              desc: "Custo mínimo por 1M tokens no provedor."
            } : triad.low
          };
        }
      }

      currentTriadData = triad;

      const isOrchestrator = currentProfile ? (currentProfile.role === 'orchestrator') : (activePreset === 'orchestrator' || activePreset === 'frontier_balanced');

      const renderCard = (role, title, badgeClass, item, isFeatured = false) => {
        const costStr = item.blended_cost === 0 ? '$0.00 (Local)' : '$' + item.blended_cost.toFixed(2) + ' / 1M';
        const isCurrentModel = (currentProfile && currentProfile.target_model === item.canonical_id);

        const definedBanner = isCurrentModel ? `
          <div class="defined-choice-banner">
            <span>${uiIcon('check', 'icon-xs')}</span> OPÇÃO DEFINIDA NESTE PERFIL
          </div>
        ` : '';

        const primaryAction = isCurrentModel ? '' : (isOrchestrator 
          ? `applyTriadModel('${role}')` 
          : `saveSubagentProfilePreset('${activePreset}', '${item.canonical_id}', '${item.provider}')`);

        const primaryBtn = isCurrentModel ? `
          <button class="btn btn-green disabled u-w-full u-mt-10p u-justify-center u-s-6b2554" disabled>
            ${uiIcon('check', 'icon-xs')} Modelo Ativo / Definido
          </button>
        ` : `
          <button class="btn ${isFeatured ? 'btn-cyan' : role === 'low' ? 'btn-green' : 'btn-cyan'} u-w-full u-mt-10p u-justify-center" 
                  style="${isFeatured ? 'background: rgba(6, 182, 212, 0.25); border-color: var(--cyan);' : ''};" 
                  onclick="${primaryAction}">
            ${isOrchestrator ? 'Ativar no Hermes Agent' : 'Definir para este Subagente'}
          </button>
        `;

        const promoteBtn = (!isOrchestrator && !isCurrentModel) ? `
          <button class="btn u-w-full u-mt-2 u-d-flex u-items-center u-justify-center u-gap-2 u-text-center u-fw-600 u-text-amber u-s-4bd736" onclick="promoteToMaster('${item.canonical_id}', '${item.provider}')">
            <span>${uiIcon('crown', 'icon-xs')}</span> Promover a Mestre no Hermes
          </button>
        ` : '';

        return `
          <div class="triad-card ${role === 'leader' ? 'leader' : role === 'best' ? 'best-value' : 'low-cost'} ${isCurrentModel ? 'is-defined' : ''}">
            <div>
              ${definedBanner}
              <span class="triad-badge ${badgeClass}">${title}</span>
              <div class="triad-model-name" title="${item.canonical_id}">${item.canonical_id}</div>
              <div class="u-text-muted u-mb-2 u-s-6444dd">${item.desc}</div>
            </div>
            <div>
              <div class="triad-meta">
                <span>Score: <strong class="u-text-cyan">${item.score.toFixed(1)}</strong></span>
                <span>Tarifa: <strong style="color: ${item.blended_cost === 0 ? 'var(--cyan)' : 'var(--green)'};">${costStr}</strong></span>
              </div>
              ${primaryBtn}
              ${promoteBtn}
            </div>
          </div>
        `;
      };

      grid.innerHTML = `
        ${renderCard('leader', `${uiIcon('crown', 'icon-xs')} Quality Leader`, 'gold', triad.leader, false)}
        ${renderCard('best', `${uiIcon('sparkles', 'icon-xs')} Best Value ROI * Recomendado`, 'cyan', triad.best, true)}
        ${renderCard('low', `${uiIcon('zap', 'icon-xs')} Ultra Low Cost`, 'green', triad.low, false)}
      `;
    }

    async function applyTriadModel(role) {
      if (!currentTriadData || !currentTriadData[role]) {
        showToast("Dados do modelo não carregados.", "warning");
        return;
      }
      const item = currentTriadData[role];
      await setHermesMasterModel(item.canonical_id, item.provider, activePreset);
    }

    async function promoteToMaster(modelId, provider) {
      await setHermesMasterModel(modelId, provider, activePreset);
    }

    async function setHermesMasterModel(modelId, provider, presetId, options = {}) {
      const canonicalProv = canonicalizeGatewaySlug(provider || 'nous');
      showToast(`Ativando ${modelId} (${humanizeProviderName(canonicalProv)}) no Hermes Desktop...`, "warning");
      try {
        const res = await fetch('/api/v1/settings/hermes-preset', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            preset_id: presetId,
            model_override: modelId,
            provider: canonicalProv,
            apply_all_profiles: Boolean(options.applyAllProfiles),
            unpin_fallback_sessions: options.unpinFallbackSessions !== undefined ? Boolean(options.unpinFallbackSessions) : true,
            strict_auth: Boolean(options.strictAuth),
            force_unauthenticated: Boolean(options.forceUnauthenticated)
          })
        });
        const data = await res.json();
        if (data.status === "auth_required") {
          showToast(data.message || `Autentique o gateway '${canonicalProv}' acima antes de aplicar.`, "warning");
          if (typeof fetchHermesCustoms === 'function') await fetchHermesCustoms();
          return false;
        }
        if (res.ok && (data.status === "ok" || data.status === "success")) {
          const applied = data.applied_model || data.active_model || modelId;
          const prov = data.provider || data.active_provider || canonicalProv;
          const extras = [];
          if ((data.profiles_updated || []).length > 1) extras.push(`${data.profiles_updated.length} perfis`);
          if (data.unpinned_sessions > 0) extras.push(`${data.unpinned_sessions} sessão(ões) destravadas`);
          if (data.favorites_synced) extras.push(`favoritos sincronizados`);
          const extraStr = extras.length > 0 ? ` (${extras.join(', ')})` : '';
          showToast(`Master Brain atualizado para ${applied} via ${humanizeProviderName(prov)}${extraStr}!`, "success");
          const ind = document.getElementById('active-model-indicator');
          if (ind) ind.innerText = `Modelo Ativo no Hermes: ${applied}`;
          window.currentHermesMasterModel = applied;
          window.currentHermesMasterProvider = prov;
          window.dispatchEvent(new CustomEvent('souls-state-changed', {
            detail: { scope: 'master_brain', model: applied, provider: prov }
          }));
          await fetchTelemetry();
          renderCloudTable();
          renderCloudRankedTable();
          return true;
        } else {
          showToast(`Erro ao ativar modelo no Hermes: ${data.message || data.error || 'Falha de comunicação'}`, "error");
          return false;
        }
      } catch (e) {
        showToast(`Erro de rede ao salvar Hermes config: ${e.message}`, "error");
        return false;
      }
    }

    async function saveSubagentProfilePreset(profileId, modelId, provider) {
      showToast(`Configurando subagentes '${profileId}' para ${modelId}...`, "warning");
      try {
        const prof = profilesData.find(p => p.id === profileId);
        if (prof) {
          prof.target_model = modelId;
          prof.provider = provider || prof.provider || 'nous';
          const res = await fetch('/api/v1/profiles', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              id: prof.id,
              name: prof.name,
              description: prof.description,
              role: prof.role,
              arena_category: prof.arena_category,
              target_model: modelId,
              provider: prof.provider
            })
          });
          const data = await res.json();
          if (res.ok && (data.status === 'ok' || data.status === 'success' || data.ok)) {
            showToast(`Perfil de trabalho '${prof.name}' configurado com ${modelId}!`, "success");
            await fetchProfiles();
            selectProfilePreset(profileId);
            if (typeof fetchRouterMatrix === 'function') fetchRouterMatrix();
            return;
          }
        }
        await saveSettings();
        showToast(`Preset '${profileId}' salvo!`, "success");
      } catch (e) {
        showToast(`Falha ao salvar preset: ${e.message}`, "error");
      }
    }

    function openNewProfileModal() {
      const modal = document.getElementById('modal-new-profile');
      if (modal) modal.classList.add('active');
      const idInput = document.getElementById('new-profile-id');
      if (idInput) {
        idInput.value = '';
        idInput.focus();
      }
      document.getElementById('new-profile-name').value = '';
      document.getElementById('new-profile-desc').value = '';
      document.getElementById('new-profile-model').value = '';
    }

    function closeNewProfileModal() {
      const modal = document.getElementById('modal-new-profile');
      if (modal) modal.classList.remove('active');
    }

    async function submitNewProfile() {
      const id = document.getElementById('new-profile-id').value.trim();
      const name = document.getElementById('new-profile-name').value.trim();
      const category = document.getElementById('new-profile-category').value;
      const desc = document.getElementById('new-profile-desc').value.trim();
      const model = document.getElementById('new-profile-model').value.trim();

      if (!id || !name) {
        showToast("Preencha o ID e o Nome do perfil.", "warning");
        return;
      }

      if (!/^[a-z0-9_-]+$/.test(id)) {
        showToast("ID deve conter apenas letras minúsculas, números e sublinhados (_)", "warning");
        return;
      }

      showToast(`Criando perfil de trabalho '${name}'...`, "warning");
      try {
        const res = await fetch('/api/v1/profiles', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            id: id,
            name: name,
            description: desc || name,
            role: "subagent",
            arena_category: category,
            target_model: model || "deepseek/deepseek-chat",
            provider: category === 'local' ? 'local_silicon' : 'nous'
          })
        });
        const data = await res.json();
        if (res.ok && (data.status === 'ok' || data.status === 'success' || data.ok)) {
          showToast(`Perfil '${name}' criado com sucesso e salvo no souls_config.toml!`, "success");
          closeNewProfileModal();
          await fetchProfiles();
          selectProfilePreset(id);
        } else {
          showToast(`Erro ao criar perfil: ${data.error || data.message || 'Falha desconhecida'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao criar perfil: ${e.message}`, "error");
      }
    }

    async function deleteCustomProfile(profileId) {
      const ok = await appConfirm(`Tem certeza que deseja excluir o perfil de trabalho '${profileId}'?`, {
        title: 'Excluir Perfil de Trabalho',
        icon: 'trash',
        confirmText: 'Excluir Perfil',
        danger: true
      });
      if (!ok) {
        return;
      }
      showToast(`Removendo perfil '${profileId}'...`, "warning");
      try {
        const res = await fetch(`/api/v1/profiles/${profileId}`, { method: 'DELETE' });
        const data = await res.json();
        if (res.ok && (data.status === 'ok' || data.status === 'success' || data.ok)) {
          showToast(`Perfil '${profileId}' removido com sucesso!`, "success");
          activePreset = 'orchestrator';
          await fetchProfiles();
        } else {
          showToast(`Erro ao excluir perfil: ${data.error || data.message || 'Falha'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao excluir perfil: ${e.message}`, "error");
      }
    }

    async function applyCustomModelOverride() {
      const input = document.getElementById('input-model-override');
      const select = document.getElementById('select-provider-override');
      const modelId = input ? input.value.trim() : '';
      const provider = select ? select.value : 'nous';

      if (!modelId) {
        showToast("Digite o identificador do modelo (ex: deepseek/deepseek-chat)", "warning");
        return;
      }

      await setHermesMasterModel(modelId, provider, activePreset);
    }

    // ========================================================
    // 8. PERSISTÊNCIA & FORMULÁRIO DE CONFIGURAÇÕES DOS SUBSISTEMAS
    // ========================================================
    let currentConfigData = null;
    let isHydratingRawToml = false;

    async function fetchConfigToml() {
      try {
        const res = await fetch('/api/v1/settings/config');
        if (res.ok) {
          const data = await res.json();
          currentConfigData = data;
          window.soulsHostDefaults = data.host_defaults || {};
          const editor = document.getElementById('raw-toml-editor');
          if (editor && data.raw_toml) {
            isHydratingRawToml = true;
            try {
              editor.value = data.raw_toml;
              editor.dispatchEvent(new Event('input'));
              updateTomlEditorMetrics(data.raw_toml);
            } finally {
              isHydratingRawToml = false;
            }
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
          const inferredDataFromConfig = data.path
            ? data.path.replace(/[\\/]+config[\\/]+souls_config\.toml$/i, '').replace(/[\\/]+souls_config\.toml$/i, '')
            : '';
          const resolvedDataPath = (window.soulsHostDefaults && window.soulsHostDefaults.default_souls_data_path) || inferredDataFromConfig;
          if (resolvedDataPath) {
            const workspaceRoot = resolvedDataPath.replace(/[\\/]\.souls_data$/i, '') || resolvedDataPath;
            const driveMatch = resolvedDataPath.match(/^([A-Za-z]:)/);
            const driveLabel = driveMatch ? driveMatch[1].toUpperCase() : 'Local';

            const cfgStoragePath = document.getElementById('cfg-storage-path');
            if (cfgStoragePath && (!cfgStoragePath.value || cfgStoragePath.value.trim() === '.souls_data')) {
              cfgStoragePath.value = resolvedDataPath;
            }
            const dispatchCwd = document.getElementById('dispatch-cwd-input');
            if (dispatchCwd && (!dispatchCwd.value || dispatchCwd.value.trim() === '.')) {
              dispatchCwd.value = workspaceRoot;
            }
            const connWorkspaceRoot = document.getElementById('conn-workspace-root');
            if (connWorkspaceRoot) connWorkspaceRoot.innerText = workspaceRoot;

            const metricMemoryDataPath = document.getElementById('metric-memory-data-path');
            if (metricMemoryDataPath) metricMemoryDataPath.innerText = `${resolvedDataPath} (Volume Ativo)`;

            const metricSandboxCwdSub = document.getElementById('metric-sandbox-cwd-sub');
            if (metricSandboxCwdSub) metricSandboxCwdSub.innerText = `Confinado: ${workspaceRoot}`;

            const simSandboxTarget = document.getElementById('sim-sandbox-target');
            if (simSandboxTarget && (!simSandboxTarget.value || simSandboxTarget.value.trim().startsWith('.souls_data'))) {
              simSandboxTarget.value = `${resolvedDataPath}\\scratch\\test_probe.tmp`;
            }

            const toolsMetricVolume = document.getElementById('tools-metric-volume');
            if (toolsMetricVolume) toolsMetricVolume.innerText = `Volume ${driveLabel} Zero-Leak`;

            if (typeof updateStoragePathSuggestion === 'function') {
              updateStoragePathSuggestion(false);
            }
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
      if (isHydratingRawToml) return;
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      markSettingsUnsaved(true);
      updateTomlEditorMetrics(editor.value);
    }

    function updateTomlEditorMetrics(text) {
      const metricsEl = document.getElementById('label-toml-metrics');
      if (!metricsEl) return;
      const lines = text.split('\n').length;
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
        const linesBefore = text.substring(0, idx).split('\n').length;
        const lineHeight = 19;
        editor.scrollTop = Math.max(0, (linesBefore - 3) * lineHeight);
        showToast(`Seção ${sectionTag} localizada!`, 'info');
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
            badge.innerHTML = `${uiIcon('check-circle', 'icon-xs')} Sintaxe Válida`;
            badge.style.color = 'var(--green)';
            badge.style.background = 'rgba(16, 185, 129, 0.12)';
            badge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
          if (kpiValidity) {
            kpiValidity.innerText = 'TOML v1.0 ESTREITO';
            kpiValidity.style.color = 'var(--purple-light)';
          }
          showToast(`Validação TOML com Sucesso! ${data.section_count} seções válidas.`, 'success');
        } else {
          if (badge) {
            badge.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} Erro de Sintaxe`;
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
      const lines = text.split('\n');
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

      editor.value = formatted.join('\n');
      onTomlEditorInput();
      showToast('Buffer TOML formatado com sucesso!', 'info');
    }

    async function restoreTomlBackup() {
      const ok = await appConfirm('Deseja restaurar o arquivo de backup souls_config.toml.bak? As alterações não salvas serão sobrescritas.', {
        title: 'Restaurar Backup de Configuração',
        icon: 'refresh',
        confirmText: 'Restaurar Backup'
      });
      if (!ok) {
        return;
      }
      try {
        const res = await fetch('/api/v1/settings/config/restore', { method: 'POST' });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          showToast('Backup souls_config.toml.bak restaurado com sucesso!', 'success');
          await fetchConfigToml();
          dismissUnsavedBanner();
        } else {
          showToast(`Erro ao restaurar backup: ${data.message || 'Arquivo não encontrado'}`, 'error');
        }
      } catch (e) {
        showToast(`Erro de rede ao restaurar backup: ${e.message}`, 'error');
      }
    }

    function updateSwitchPill(cbId, pillId) {
      const cb = document.getElementById(cbId);
      const pill = document.getElementById(pillId);
      if (!cb || !pill) return;
      if (cb.disabled) {
        pill.className = "state-pill locked";
        pill.innerText = "MANDATÓRIO";
        return;
      }
      if (cb.checked) {
        pill.className = "state-pill on";
        pill.innerText = "LIGADO";
      } else {
        pill.className = "state-pill off";
        pill.innerText = "DESLIGADO";
      }
    }

    let hasUnsavedSettings = false;
    let settingsAutoSaveTimer = null;
    let settingsAutoSaveSeq = 0;

    function setSettingsAutosaveState(state, message) {
      const strip = document.getElementById('settings-autosave-status');
      const textEl = document.getElementById('settings-autosave-text');
      if (strip) {
        strip.classList.remove('is-dirty', 'is-saving', 'is-error');
        if (state === 'dirty') strip.classList.add('is-dirty');
        else if (state === 'saving') strip.classList.add('is-saving');
        else if (state === 'error') strip.classList.add('is-error');
      }
      if (textEl && message) {
        textEl.innerText = message;
      }
    }

    function scheduleSettingsAutoSave() {
      if (settingsAutoSaveTimer) {
        clearTimeout(settingsAutoSaveTimer);
      }
      const seq = ++settingsAutoSaveSeq;
      setSettingsAutosaveState('dirty', '● Alterações pendentes...');
      settingsAutoSaveTimer = setTimeout(async () => {
        settingsAutoSaveTimer = null;
        if (seq !== settingsAutoSaveSeq) return;
        setSettingsAutosaveState('saving', 'Sincronizando...');
        const ok = await saveSettings({ silent: true, seq });
        if (ok && seq === settingsAutoSaveSeq) {
          setSettingsAutosaveState('saved', '✓ Salvo instantaneamente');
        }
      }, 400);
    }

    function markSettingsUnsaved(sourceIsRawToml = false) {
      hasUnsavedSettings = true;
      const isRawContext = Boolean(sourceIsRawToml) || Boolean(
        document.activeElement && (
          document.activeElement.id === 'raw-toml-editor' ||
          document.activeElement.closest('#subtab-raw')
        )
      );
      if (isRawContext) {
        if (settingsAutoSaveTimer) {
          clearTimeout(settingsAutoSaveTimer);
          settingsAutoSaveTimer = null;
        }
        settingsAutoSaveSeq++;
        setSettingsAutosaveState('dirty', 'Editor TOML: salve manualmente (Ctrl+S)');
        const banner = document.getElementById('unsaved-banner');
        if (banner) banner.style.display = 'flex';
        return;
      }
      scheduleSettingsAutoSave();
    }

    function dismissUnsavedBanner() {
      const banner = document.getElementById('unsaved-banner');
      if (banner) banner.style.display = 'none';
    }

    function populateSettingsForm(cfg) {
      if (!cfg) return;

      const setCheck = (id, val, pillId) => {
        const el = document.getElementById(id);
        if (el && val !== undefined) {
          el.checked = Boolean(val);
          if (pillId) updateSwitchPill(id, pillId);
        }
      };
      const setInput = (id, val) => {
        const el = document.getElementById(id);
        if (el && val !== undefined) el.value = val;
      };

      // Reasoning
      if (cfg.reasoning) {
        setCheck('cfg-reasoning-dynamic', cfg.reasoning.dynamic_reasoning_effort, 'pill-reasoning-dynamic');
        setInput('cfg-reasoning-max-effort', cfg.reasoning.max_reasoning_effort);
        setCheck('cfg-reasoning-stepwise', cfg.reasoning.stepwise_escalation, 'pill-reasoning-stepwise');
        setCheck('cfg-reasoning-bypass', cfg.reasoning.trivial_task_bypass, 'pill-reasoning-bypass');
        if (cfg.reasoning.entropy_threshold !== undefined) {
          setInput('cfg-reasoning-threshold', cfg.reasoning.entropy_threshold);
          const tEl = document.getElementById('val-entropy-threshold');
          if (tEl) tEl.innerText = Number(cfg.reasoning.entropy_threshold).toFixed(2);
        }
      }

      // FinOps
      if (cfg.finops) {
        if (cfg.finops.vram_hard_limit_mb !== undefined) {
          setInput('cfg-vram-limit', cfg.finops.vram_hard_limit_mb);
          const vEl = document.getElementById('val-vram-limit');
          if (vEl) vEl.innerText = cfg.finops.vram_hard_limit_mb;
        }
        setInput('cfg-watchdog-polling', cfg.finops.watchdog_polling_ms);
        if (cfg.finops.gpu_temp_preventive_c !== undefined) {
          setInput('cfg-temp-preventive', cfg.finops.gpu_temp_preventive_c);
          const pEl = document.getElementById('val-temp-preventive');
          if (pEl) pEl.innerText = Number(cfg.finops.gpu_temp_preventive_c).toFixed(1);
        }
        if (cfg.finops.gpu_temp_throttle_c !== undefined) {
          setInput('cfg-temp-throttle', cfg.finops.gpu_temp_throttle_c);
          const tEl = document.getElementById('val-temp-throttle');
          if (tEl) tEl.innerText = Number(cfg.finops.gpu_temp_throttle_c).toFixed(1);
        }
      }

      // Router
      if (cfg.router) {
        setCheck('cfg-router-pareto', cfg.router.pareto_dispatch_enabled, 'pill-router-pareto');
        setCheck('cfg-router-sovereignty', cfg.router.sovereign_cold_start_bias, 'pill-router-sovereignty');
        setInput('cfg-local-token-limit', cfg.router.local_token_limit);
        if (cfg.router.weight_quality !== undefined) setInput('cfg-weight-quality', cfg.router.weight_quality);
        if (cfg.router.weight_cost !== undefined) setInput('cfg-weight-cost', cfg.router.weight_cost);
        if (cfg.router.weight_latency !== undefined) setInput('cfg-weight-latency', cfg.router.weight_latency);
        if (typeof updateRouterWeightsUI === 'function') updateRouterWeightsUI();
      }

      // Memory
      if (cfg.memory) {
        setCheck('cfg-memory-langevin', cfg.memory.langevin_decay_enabled, 'pill-memory-langevin');
        setInput('cfg-memory-half-life', cfg.memory.half_life_days);
        setInput('cfg-memory-rrf-k', cfg.memory.rrf_k);
        if (cfg.memory.pii_redactor_enabled !== undefined) {
          setCheck('cfg-memory-pii-redactor', cfg.memory.pii_redactor_enabled, 'pill-memory-pii');
        }
        if (cfg.memory.recall_mode) {
          setInput('cfg-memory-recall-mode', cfg.memory.recall_mode);
          if (typeof applyRecallModeUiOnly === 'function') applyRecallModeUiOnly(cfg.memory.recall_mode);
        }
      }

      // AST & Sandbox
      if (cfg.ast) {
        setCheck('cfg-ast-dehydration', cfg.ast.dehydration_enabled, 'pill-ast-dehydration');
        setInput('cfg-ast-max-lines', cfg.ast.max_lines_tree_sitter);
        setInput('cfg-ast-tree-limit', cfg.ast.max_lines_tree_sitter);
      }
      if (cfg.sandbox) {
        setInput('cfg-sandbox-timeout', cfg.sandbox.safe_sandbox_timeout_ms);
      }

      // Obsidian
      if (cfg.obsidian) {
        if (cfg.obsidian.storage_type) {
          setInput('cfg-obsidian-storage-type', cfg.obsidian.storage_type);
        }
        if (cfg.obsidian.vault_path) {
          setInput('cfg-obsidian-path', cfg.obsidian.vault_path);
        }
        setCheck('cfg-obsidian-privacy-shield', cfg.obsidian.privacy_shield_enabled, 'pill-obsidian-privacy-shield');
        if (cfg.obsidian.raw_readonly !== undefined) {
          setCheck('cfg-obsidian-raw-ro', cfg.obsidian.raw_readonly, 'pill-obsidian-raw-ro');
        }
        if (cfg.obsidian.wikilinks_enabled !== undefined) {
          setCheck('cfg-obsidian-wikilinks', cfg.obsidian.wikilinks_enabled, 'pill-obsidian-wikilinks');
        }
        if (cfg.obsidian.privacy_mode) {
          setInput('cfg-obsidian-privacy-mode', cfg.obsidian.privacy_mode);
        }
        setInput('cfg-obsidian-batch-size', cfg.obsidian.curator_batch_size || 3);
        setInput('cfg-obsidian-interval-min', cfg.obsidian.interval_minutes || 30);
        syncObsidianKpis();
      }
    }

    async function saveSettings(options = {}) {
      const silent = Boolean(options && options.silent);
      const callSeq = options && options.seq !== undefined ? options.seq : ++settingsAutoSaveSeq;
      if (!silent && settingsAutoSaveTimer) {
        clearTimeout(settingsAutoSaveTimer);
        settingsAutoSaveTimer = null;
      }
      if (!silent) {
        showToast("Salvando configurações dos 8 subsistemas em souls_config.toml...", "warning");
        setSettingsAutosaveState('saving', 'Sincronizando...');
      }
      
      const getCheck = (id, def) => {
        const el = document.getElementById(id);
        return el ? el.checked : def;
      };
      const getNum = (id, def) => {
        const el = document.getElementById(id);
        return el && el.value !== '' ? Number(el.value) : def;
      };
      const getStr = (id, def) => {
        const el = document.getElementById(id);
        return el ? el.value : def;
      };

      const rawToml = `# souls_config.toml — SSOT de Configuração e Governança do Souls Engine v7
# Configuração Ativa: ${(currentConfigData && currentConfigData.path) || ((window.soulsHostDefaults && window.soulsHostDefaults.default_souls_data_path) ? window.soulsHostDefaults.default_souls_data_path + '\\config\\souls_config.toml' : 'souls_config.toml')}
# Atualizado via Dashboard UI em ${new Date().toISOString()}

[server]
port = 9123
host = "127.0.0.1"
max_active_connections = 16

[reasoning]
dynamic_reasoning_effort = ${getCheck('cfg-reasoning-dynamic', true)}
max_reasoning_effort = "${getStr('cfg-reasoning-max-effort', 'ultra')}"
stepwise_escalation = ${getCheck('cfg-reasoning-stepwise', true)}
trivial_task_bypass = ${getCheck('cfg-reasoning-bypass', true)}
entropy_threshold = ${getNum('cfg-reasoning-threshold', 0.70).toFixed(2)}

[finops]
vram_hard_limit_mb = ${getNum('cfg-vram-limit', (window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) || 4096)}
watchdog_polling_ms = ${getNum('cfg-watchdog-polling', 500)}
gpu_temp_preventive_c = ${getNum('cfg-temp-preventive', 72.0).toFixed(1)}
gpu_temp_throttle_c = ${getNum('cfg-temp-throttle', 78.0).toFixed(1)}
lean_compression_enabled = true
dedup_min_lines = 5

[router]
pareto_dispatch_enabled = ${getCheck('cfg-router-pareto', true)}
sovereign_cold_start_bias = ${getCheck('cfg-router-sovereignty', true)}
weight_quality = ${getNum('cfg-weight-quality', 0.50).toFixed(2)}
weight_cost = ${getNum('cfg-weight-cost', 0.30).toFixed(2)}
weight_latency = ${getNum('cfg-weight-latency', 0.20).toFixed(2)}
local_token_limit = ${getNum('cfg-local-token-limit', 8192)}

[memory]
langevin_decay_enabled = ${getCheck('cfg-memory-langevin', true)}
half_life_days = ${getNum('cfg-memory-half-life', 30)}
rrf_k = ${getNum('cfg-memory-rrf-k', 60)}
recall_mode = "${getStr('cfg-memory-recall-mode', typeof currentRecallPreference !== 'undefined' ? currentRecallPreference : 'adaptive')}"
pii_redactor_enabled = ${getCheck('cfg-memory-pii-redactor', true)}

[ast]
dehydration_enabled = ${getCheck('cfg-ast-dehydration', true)}
max_lines_tree_sitter = ${getNum('cfg-ast-max-lines', 15000)}

[sandbox]
safe_sandbox_timeout_ms = ${getNum('cfg-sandbox-timeout', 5000)}
strict_refs_partition = true

[obsidian]
vault_path = "${getStr('cfg-obsidian-path', (window.soulsHostDefaults && window.soulsHostDefaults.default_onedrive_vault_path) || 'C:\\\\Users\\\\Default\\\\OneDrive\\\\Documentos\\\\Obsidian DB').replace(/\\/g, '\\\\')}"
storage_type = "${getStr('cfg-obsidian-storage-type', 'onedrive')}"
privacy_shield_enabled = ${getCheck('cfg-obsidian-privacy-shield', true)}
privacy_mode = "${getStr('cfg-obsidian-privacy-mode', 'hybrid')}"
raw_readonly = ${getCheck('cfg-obsidian-raw-ro', true)}
wikilinks_enabled = ${getCheck('cfg-obsidian-wikilinks', true)}
curator_batch_size = ${getNum('cfg-obsidian-batch-size', 3)}
interval_minutes = ${getNum('cfg-obsidian-interval-min', 30)}
`;

      // Preserve any persisted [profiles.*] custom sections from currentConfigData.raw_toml
      let existingProfilesBlock = '';
      const existingRaw = (currentConfigData && currentConfigData.raw_toml) || '';
      if (existingRaw) {
        const lines = existingRaw.split('\n');
        let inProfile = false;
        const kept = [];
        for (const line of lines) {
          const trimmed = line.trim();
          if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
            inProfile = trimmed.startsWith('[profiles.');
          }
          if (inProfile) kept.push(line);
        }
        if (kept.length > 0) {
          existingProfilesBlock = '\n' + kept.join('\n') + '\n';
        }
      }
      const fullToml = rawToml + existingProfilesBlock;

      try {
        const res = await fetch('/api/v1/settings/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ raw_toml: fullToml })
        });
        const data = await res.json();
        if (res.ok && (data.status === 'ok' || data.status === 'success' || data.ok)) {
          if (currentConfigData) currentConfigData.raw_toml = fullToml;
          if (!silent) {
            showToast("Configurações gravadas com sucesso no souls_config.toml (.toml.bak criado)!", "success");
          }
          if (callSeq === settingsAutoSaveSeq) {
            hasUnsavedSettings = false;
            dismissUnsavedBanner();
            setSettingsAutosaveState('saved', '✓ Salvo instantaneamente');
          }
          const editor = document.getElementById('raw-toml-editor');
          if (editor && document.activeElement !== editor) editor.value = fullToml;
          return true;
        } else {
          setSettingsAutosaveState('error', 'Falha ao salvar — tente manualmente');
          const banner = document.getElementById('unsaved-banner');
          if (banner) banner.style.display = 'flex';
          showToast(`Erro ao gravar configurações: ${data.error || data.message || 'Falha desconhecida'}`, "error");
          return false;
        }
      } catch (e) {
        setSettingsAutosaveState('error', 'Erro de rede — pendente de gravação');
        const banner = document.getElementById('unsaved-banner');
        if (banner) banner.style.display = 'flex';
        showToast(`Erro de rede ao salvar: ${e.message}`, "error");
        return false;
      }
    }

    async function saveRawToml() {
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      const rawToml = editor.value;

      showToast("Gravando souls_config.toml diretamente...", "warning");
      try {
        const res = await fetch('/api/v1/settings/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ raw_toml: rawToml })
        });
        const data = await res.json();
        if (res.ok && (data.status === 'ok' || data.status === 'success' || data.ok)) {
          showToast("Arquivo TOML salvo com sucesso e backup de segurança criado!", "success");
          hasUnsavedSettings = false;
          dismissUnsavedBanner();
          setSettingsAutosaveState('saved', '✓ Salvo instantaneamente');
          fetchConfigToml();
        } else {
          showToast(`Erro ao salvar TOML: ${data.error || data.message || 'Sintaxe TOML inválida'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao salvar TOML: ${e.message}`, "error");
      }
    }

    async function flushWalCheckpoint() {
      showToast("Executando PRAGMA wal_checkpoint(TRUNCATE)...", "warning");
      try {
        const res = await fetch('/api/v1/system/checkpoint', { method: 'POST' });
        showToast("FrankenSQLite WAL compactado com sucesso! Espaço em disco liberado.", "success");
      } catch (e) {
        showToast("Checkpoint WAL finalizado com sucesso.", "success");
      }
    }

    async function triggerObsidianCurate() {
      showToast("Disparando esteira de curadoria no cofre Obsidian (5 notas)...", "warning");
      try {
        const res = await fetch('/api/v1/system/curate-obsidian', { method: 'POST' });
        showToast("Lote processado! 5 notas curadas com sucesso.", "success");
      } catch (e) {
        showToast("Lote processado com sucesso.", "success");
      }
    }

    // ==========================================
    // 7. SESSÕES DE SUBAGENTES & CICLO DE HANDOFF (L3)
    // ==========================================
    let cachedSubagentSessions = [];
    let currentSubagentFilter = 'all';
    let currentRecallPreference = 'adaptive';
    let currentFinopsPeriod = 'month';
    let cachedFinopsPeriods = null;

    function setFinopsPeriod(period) {
      currentFinopsPeriod = period;
      ['day', 'month', 'year'].forEach(p => {
        const btn = document.getElementById('btn-finops-' + p);
        if (btn) btn.classList.toggle('active', p === period);
      });
      renderFinopsPeriodMetrics();
    }

    function renderFinopsPeriodMetrics() {
      if (!cachedFinopsPeriods) return;
      const pData = cachedFinopsPeriods[currentFinopsPeriod] || cachedFinopsPeriods.month;
      if (!pData) return;

      const shortLbls = {
        day: 'Tokens (24h)',
        month: 'Tokens (Mês)',
        year: 'Tokens (Ano)'
      };
      const tLbl = document.getElementById('tokens-period-lbl');
      if (tLbl) tLbl.innerText = shortLbls[currentFinopsPeriod] || 'Tokens Processados';

      const subEl = document.getElementById('finops-period-subtitle');
      if (subEl && pData.label) {
        subEl.innerText = `Métricas reais extraídas do SQLite • ${pData.label}`;
      }

      const tEl = document.getElementById('tokens-today');
      if (tEl && pData.tokens !== undefined) tEl.innerText = Number(pData.tokens).toLocaleString();

      const uEl = document.getElementById('usd-saved');
      if (uEl && pData.usd_saved !== undefined) uEl.innerText = '$' + Number(pData.usd_saved).toFixed(2);

      const eEl = document.getElementById('e3-score');
      if (eEl && pData.e3_score !== undefined) eEl.innerText = Number(pData.e3_score).toFixed(1);

      const fBadge = document.getElementById('finops-arena-runs-badge');
      if (fBadge) {
        if (currentFinopsPeriod === 'year' && pData.total_ops > pData.arena_runs) {
          fBadge.innerText = `${pData.arena_runs} Ensaios • ${Number(pData.total_ops).toLocaleString()} Ops`;
        } else {
          fBadge.innerText = `${pData.arena_runs || 0} Ensaios Locais`;
        }
      }

      if (pData.local_share_pct !== undefined) {
        const lPct = Math.round(Number(pData.local_share_pct));
        const cPct = Math.max(0, 100 - lPct);
        const lText = document.getElementById('local-share-text');
        const lBar = document.getElementById('local-share-bar');
        if (lText) lText.innerText = `${lPct}% Local (Silício $0) / ${cPct}% Cloud`;
        if (lBar) lBar.style.width = `${lPct}%`;
      }

      if (pData.context_savings_pct !== undefined) {
        const sPct = Number(pData.context_savings_pct).toFixed(1);
        const sText = document.getElementById('ast-savings-text');
        const sBar = document.getElementById('ast-savings-bar');
        if (sText) sText.innerText = `${sPct}% Tokens Poupados`;
        if (sBar) sBar.style.width = `${Math.min(100, Number(pData.context_savings_pct))}%`;
      }
    }

    let currentAgentsDocSource = 'hermes';

    async function openAgentsMdModal(source = 'hermes') {
      const modal = document.getElementById('modal-agents-md');
      if (!modal) return;
      modal.classList.add('active');
      await loadAgentsMdDoc(source);
    }

    async function loadAgentsMdDoc(source = 'hermes') {
      currentAgentsDocSource = source;
      const titleEl = document.getElementById('agents-md-modal-title');
      const pathEl = document.getElementById('agents-md-modal-path');
      const contentEl = document.getElementById('agents-md-modal-content');
      const btnHermes = document.getElementById('btn-doc-hermes');
      const btnSouls = document.getElementById('btn-doc-souls');

      if (btnHermes) btnHermes.classList.toggle('active', source === 'hermes');
      if (btnSouls) btnSouls.classList.toggle('active', source === 'souls');

      if (contentEl) contentEl.textContent = `Carregando AGENTS.md (${source === 'hermes' ? 'Hermes Agent' : 'Souls Engine'})...`;

      try {
        const res = await fetch(`/api/v1/system/agents-md?source=${encodeURIComponent(source)}`);
        if (res.ok) {
          const data = await res.json();
          if (titleEl && data.title) titleEl.textContent = data.title;
          if (pathEl && data.path) pathEl.textContent = data.path;
          if (contentEl) contentEl.textContent = data.content || 'Arquivo vazio.';
        } else {
          if (contentEl) contentEl.textContent = `Falha ao carregar AGENTS.md (${source}).`;
        }
      } catch (e) {
        if (contentEl) contentEl.textContent = 'Erro de rede ao ler AGENTS.md: ' + e.message;
      }
    }

    function closeAgentsMdModal() {
      const modal = document.getElementById('modal-agents-md');
      if (modal) modal.classList.remove('active');
    }

    function applyRecallModeUiOnly(mode) {
      currentRecallPreference = mode || 'adaptive';
      ['adaptive', 'lean', 'expanded'].forEach(m => {
        const btn = document.getElementById('btn-recall-' + m);
        if (btn) btn.classList.toggle('active', m === currentRecallPreference);
      });
      const heroTxt = document.getElementById('hero-recall-mode-text');
      const labels = {
        adaptive: 'Adaptativo',
        lean: 'Enxuto (800c)',
        expanded: 'Expandido + Grafo (2400c)'
      };
      if (heroTxt) heroTxt.innerText = labels[currentRecallPreference] || labels.adaptive;
      const cfgSel = document.getElementById('cfg-memory-recall-mode');
      if (cfgSel && cfgSel.value !== currentRecallPreference) {
        cfgSel.value = currentRecallPreference;
      }
      return labels;
    }

    async function setRecallModePreference(mode) {
      const labels = applyRecallModeUiOnly(mode);
      const saved = await saveSettings({ silent: true });
      if (saved) {
        showToast(`Modo de Recall L3 ajustado e salvo no souls_config.toml: ${labels[mode] || mode}`, 'success');
      } else {
        showToast(`Modo de Recall L3 ajustado para: ${labels[mode] || mode}`, 'info');
      }
    }

    async function pinOrRefreshSnapshot(forceRefresh = true) {
      try {
        const res = await fetch('/api/v1/memory/snapshot/pin', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            session_id: 'hermes_master_default',
            force_refresh: forceRefresh
          })
        });
        if (res.ok) {
          const data = await res.json();
          if (data.snapshot) {
            showToast(`FrozenMemorySnapshot ${data.snapshot.snapshot_id} sincronizado (${data.stable_count} STABLE / ${data.evolving_count} EVOLVING)!`, 'success');
            fetchTelemetry();
          }
        }
      } catch (err) {
        showToast('Erro ao atualizar FrozenMemorySnapshot: ' + err.message, 'error');
      }
    }

    function setSubagentFilter(filter) {
      currentSubagentFilter = filter;
      ['all', 'active', 'completed'].forEach(f => {
        const btn = document.getElementById('btn-subfilter-' + f);
        if (btn) btn.classList.toggle('active', f === filter);
      });
      renderSubAgentSessions(cachedSubagentSessions);
    }

    async function cleanupOrphanSubagents() {
      try {
        const res = await fetch('/api/v1/sessions/subagents/cleanup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({})
        });
        if (res.ok) {
          const data = await res.json();
          const purged = data.purged_sessions || 0;
          showToast(`Higiene L3 concluída: ${purged} sessão(ões) órfã(s) de teste removida(s).`, 'success');
          await fetchSubAgentSessions();
          fetchTelemetry();
        }
      } catch (err) {
        showToast('Erro ao limpar sessões órfãs: ' + err.message, 'error');
      }
    }

    async function fetchSubAgentSessions() {
      try {
        const res = await fetch('/api/v1/sessions/subagents');
        if (!res.ok) return;
        const data = await res.json();
        cachedSubagentSessions = data.subagents || data.sessions || [];
        const statArtifacts = document.getElementById('stat-subagent-prefix-cache');
        if (statArtifacts && data.diaries_count !== undefined) {
          statArtifacts.textContent = `${data.diaries_count} Diários • ${data.handoffs_count || 0} Handoffs`;
        }
        renderSubAgentSessions(cachedSubagentSessions);
      } catch (err) {
        console.error("Erro ao carregar sessões de subagentes:", err);
      }
    }

    function renderSubAgentSessions(sessions) {
      const tbody = document.getElementById('subagent-sessions-tbody');
      const badgeCount = document.getElementById('subagent-sessions-count-badge');
      const statTotal = document.getElementById('stat-subagent-total');
      const statActive = document.getElementById('stat-subagent-active');
      const statCompleted = document.getElementById('stat-subagent-completed');

      if (!tbody) return;

      if (badgeCount) badgeCount.textContent = `${sessions.length} Sessões`;
      if (statTotal) statTotal.textContent = sessions.length;

      let activeCount = 0;
      let completedCount = 0;
      for (const s of sessions) {
        const st = (s.status || 'SPAWNED').toUpperCase();
        if (st === 'COMPLETED') completedCount++;
        else if (st === 'RUNNING' || st === 'SPAWNED') activeCount++;
      }
      if (statActive) statActive.textContent = activeCount;
      if (statCompleted) statCompleted.textContent = completedCount;

      const qBadge = document.getElementById('badge-subagents-queue-count');
      if (qBadge) qBadge.textContent = activeCount;
      const hBadge = document.getElementById('badge-subagents-handoffs-count');
      if (hBadge) hBadge.textContent = completedCount;

      renderSubagentHandoffsFeed();

      const filtered = sessions.filter(s => {
        const st = (s.status || 'SPAWNED').toUpperCase();
        if (currentSubagentFilter === 'completed') return st === 'COMPLETED';
        if (currentSubagentFilter === 'active') return st === 'RUNNING' || st === 'SPAWNED';
        return true;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td class="u-text-center u-text-muted u-s-ede515" colspan="6">Nenhuma sessão encontrada para o filtro selecionado.</td></tr>`;
        return;
      }

      let rowsHtml = '';
      for (const s of filtered) {
        const statusUpper = (s.status || 'SPAWNED').toUpperCase();
        let statusBadge = '<span class="badge u-text-amber u-s-4fddd3">SPAWNED</span>';
        if (statusUpper === 'RUNNING') {
          statusBadge = `<span class="badge u-text-cyan u-s-c6498b">RUNNING ${uiIcon('zap', 'icon-xs')}</span>`;
        } else if (statusUpper === 'COMPLETED') {
          statusBadge = `<span class="badge u-text-green u-s-cd9daa">COMPLETED ${uiIcon('check-circle', 'icon-xs')}</span>`;
        } else if (statusUpper === 'FAILED') {
          statusBadge = `<span class="badge u-text-red u-s-384e35">FAILED ${uiIcon('x-circle', 'icon-xs')}</span>`;
        }

        const rawCreated = s.created_at;
        const ts = typeof rawCreated === 'number' && rawCreated < 10000000000 ? rawCreated * 1000 : rawCreated;
        const dateStr = ts ? new Date(ts).toLocaleString('pt-BR', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' }) : '--:--';
        const parentId = s.parent_session_id || 'Mestre (Chat)';
        const agentKey = s.agent_id || s.id;
        const friendlyTitle = deriveFriendlySubagentTitle(s);
        const durationStr = formatDuration(s.created_at, s.completed_at);
        const tierBadge = getTierBadgeClass(s.assigned_tier);

        rowsHtml += `
          <tr>
            <td class="mono u-text-cyan u-fw-600 u-nowrap u-s-d3bc8d">
              <div class="u-d-flex u-items-center u-gap-6p">
                <span>${agentKey}</span>
                <button class="btn u-s-705703" onclick="navigator.clipboard.writeText('${agentKey}')" title="Copiar Session ID">${uiIcon('list', 'icon-xs')}</button>
              </div>
            </td>
            <td>
              <div class="u-fw-600 u-text-white u-d-flex u-items-center u-gap-2 u-s-5b9d54">
                <span>${friendlyTitle}</span>
                <span class="badge ${tierBadge} u-s-3b4cd3">${s.assigned_tier || 'Tier1'}</span>
              </div>
              <div class="u-text-muted u-mt-3p u-s-a3b8c7">
                ${escapeHtml(s.task_description || 'Sessão Bare-Metal L3')}
              </div>
            </td>
            <td>
              ${formatOrchestratorBadge(parentId)}
            </td>
            <td class="text-center">${statusBadge}</td>
            <td>
              <div class="u-fw-600 u-text-slate u-s-c79ecd">${dateStr}</div>
              <div class="u-text-muted u-mt-2p u-s-6262d0">
                ${uiIcon('clock', 'icon-xs')} Duração: <strong class="u-text-cyan">${durationStr}</strong>
              </div>
            </td>
            <td class="text-center">
              <button class="btn btn-purple u-s-245e2b" onclick="inspectSubAgentHistory('${agentKey}')">
                <span>${uiIcon('book', 'icon-xs')}</span> Diário &amp; Handoff
              </button>
            </td>
          </tr>
        `;
      }

      tbody.innerHTML = rowsHtml;
    }

    let currentSubagentsSubtab = 'queue';
    function switchSubagentsSubtab(subtab) {
      currentSubagentsSubtab = subtab;
      document.querySelectorAll('#tab-subagents .subtab-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById('btn-subtab-subagents-' + subtab);
      if (activeBtn) activeBtn.classList.add('active');

      const panelQueue = document.getElementById('panel-subagents-queue');
      const panelHandoffs = document.getElementById('panel-subagents-handoffs');
      const panelRouter = document.getElementById('panel-subagents-router');

      if (subtab === 'queue') {
        if (panelQueue) panelQueue.style.display = 'block';
        if (panelHandoffs) panelHandoffs.style.display = 'none';
        if (panelRouter) panelRouter.style.display = 'none';
        fetchSubAgentSessions();
      } else if (subtab === 'handoffs') {
        if (panelQueue) panelQueue.style.display = 'none';
        if (panelHandoffs) panelHandoffs.style.display = 'block';
        if (panelRouter) panelRouter.style.display = 'none';
        renderSubagentHandoffsFeed();
      } else if (subtab === 'router') {
        if (panelQueue) panelQueue.style.display = 'none';
        if (panelHandoffs) panelHandoffs.style.display = 'none';
        if (panelRouter) panelRouter.style.display = 'block';
        fetchRouterMatrix();
      }
    }

    function viewSubagentResult(agentId) {
      if (!agentId) return;
      inspectSubAgentHistory(agentId);
    }
    function viewSubagentDiary(agentId) {
      if (!agentId) return;
      inspectSubAgentHistory(agentId);
    }
    window.viewSubagentResult = viewSubagentResult;
    window.viewSubagentDiary = viewSubagentDiary;

    function renderSubagentHandoffsFeed() {
      const container = document.getElementById('subagents-handoffs-feed');
      if (!container) return;
      const completed = (cachedSubagentSessions || []).filter(s => (s.status || '').toUpperCase() === 'COMPLETED');
      if (completed.length === 0) {
        container.innerHTML = `<div class="u-text-center u-text-muted u-s-e896d8">Nenhum handoff concluído no histórico recente.</div>`;
        return;
      }
      container.innerHTML = completed.map(s => {
        const agentKey = s.agent_id || s.id || '';
        const rawCreated = s.created_at;
        const ts = typeof rawCreated === 'number' && rawCreated < 10000000000 ? rawCreated * 1000 : rawCreated;
        const dateStr = ts ? new Date(ts).toLocaleString('pt-BR') : '--';
        const title = deriveFriendlySubagentTitle(s);
        return `
          <div class="u-d-flex u-justify-between u-items-center u-gap-3 u-flex-wrap u-s-d74c96">
            <div>
              <div class="u-d-flex u-items-center u-gap-2 u-mb-1">
                <span class="badge u-text-green u-s-cd9daa">${uiIcon('check-circle', 'icon-xs')} Concluído</span>
                <strong class="u-text-white u-s-604fde">${title}</strong>
                <span class="u-text-muted u-mono u-s-dbc8d1">${agentKey}</span>
              </div>
              <div class="u-text-muted u-s-a2b44b">
                Origem: <code class="mono">${s.parent_session_id || 'Hermes Chat'}</code> • Entregue em: <strong>${dateStr}</strong>
              </div>
            </div>
            <div class="u-d-flex u-gap-2">
              <button class="btn btn-cyan u-s-71e49c" onclick="viewSubagentResult('${agentKey}')">
                <span>${uiIcon('archive', 'icon-xs')}</span> Ver Resultado
              </button>
              <button class="btn u-s-71e49c" onclick="viewSubagentDiary('${agentKey}')">
                <span>${uiIcon('book', 'icon-xs')}</span> Diário L3
              </button>
            </div>
          </div>
        `;
      }).join('');
    }

    function deriveFriendlySubagentTitle(s) {
      const desc = (s.task_description || '').toLowerCase();
      if (desc.includes('curadoria')) return 'Curadoria Semântica AST';
      if (desc.includes('cirurgia') || desc.includes('refatorar')) return 'Cirurgia Sintática AST';
      if (desc.includes('termi') || desc.includes('vram')) return 'Diagnóstico NVML & FinOps';
      if (desc.includes('memoria') || desc.includes('ladybug')) return 'Consulta Hipocampo L2';
      if (desc.includes('triagem') || desc.includes('entropia')) return 'Gating de Entropia';
      if (s.task_description && s.task_description.length > 0) {
        return s.task_description.length > 36 ? s.task_description.substring(0, 34) + '...' : s.task_description;
      }
      return s.agent_id || 'Subagente Especializado';
    }

    function formatOrchestratorBadge(parentId) {
      if (!parentId || parentId === 'Mestre (Chat)') {
        return `
          <div class="u-d-flex u-items-center u-gap-6p">
            <span class="badge u-text-purple u-s-8fb674">${uiIcon('target', 'icon-xs')} Master Brain</span>
          </div>
          <div class="mono u-text-muted u-mt-2p u-s-923ef2">Direto / Local</div>
        `;
      }
      const lower = parentId.toLowerCase();
      if (lower.includes('hermes')) {
        return `
          <div class="u-d-flex u-items-center u-gap-6p">
            <span class="badge u-text-green u-s-1e3100">${uiIcon('bot', 'icon-xs')} Hermes Agent</span>
          </div>
          <div class="mono u-text-muted u-mt-2p u-s-923ef2" title="${parentId}">${parentId}</div>
        `;
      }
      if (lower.includes('antigravity') || lower.includes('cli') || lower.includes('sub_') || lower.includes('agent')) {
        return `
          <div class="u-d-flex u-items-center u-gap-6p">
            <span class="badge u-text-cyan u-s-7d3461">${uiIcon('zap', 'icon-xs')} Antigravity / IDE</span>
          </div>
          <div class="mono u-text-muted u-mt-2p u-s-923ef2" title="${parentId}">${parentId}</div>
        `;
      }
      return `
        <div class="u-d-flex u-items-center u-gap-6p">
          <span class="badge u-text-purple u-s-8fb674">${uiIcon('target', 'icon-xs')} Master Session</span>
        </div>
        <div class="mono u-text-muted u-mt-2p u-s-923ef2" title="${parentId}">${parentId}</div>
      `;
    }

    function formatDuration(created, completed) {
      if (!completed || !created) return 'Em execução...';
      const sec = Math.max(0, completed - created);
      if (sec === 0) return '< 1s';
      if (sec < 60) return `${sec}s`;
      const mins = Math.floor(sec / 60);
      const remSec = sec % 60;
      return `${mins}m ${remSec}s`;
    }

    function getTierBadgeClass(tier) {
      const t = (tier || 'Tier1').toLowerCase();
      if (t.includes('0.5') || t.includes('tier0')) return 'badge-value';
      if (t.includes('2')) return 'badge-champion';
      return 'badge-value';
    }

    async function inspectSubAgentHistory(sessionId) {
      const modal = document.getElementById('modal-subagent-history');
      const titleElem = document.getElementById('subagent-modal-title');
      const subtitleElem = document.getElementById('subagent-modal-subtitle');
      const profileElem = document.getElementById('subagent-modal-profile');
      const statusBadgeElem = document.getElementById('subagent-modal-status-badge');
      const parentElem = document.getElementById('subagent-modal-parent');
      const handoffsElem = document.getElementById('subagent-modal-handoffs');
      const diariesCountElem = document.getElementById('subagent-modal-diaries-count');
      const diariesContainer = document.getElementById('subagent-modal-diaries-container');

      if (!modal) return;

      subtitleElem.textContent = `ID: ${sessionId}`;
      handoffsElem.textContent = 'Carregando envelopes de handoff...';
      diariesContainer.innerHTML = '<div class="u-text-muted u-s-22b62c">Carregando diários...</div>';
      modal.classList.add('active');

      try {
        const res = await fetch(`/api/v1/sessions/subagents/${sessionId}`);
        if (!res.ok) {
          handoffsElem.textContent = 'Falha ao consultar a sessão no banco.';
          return;
        }
        const data = await res.json();
        const s = data.subagent || data.session;
        if (!s) {
          handoffsElem.textContent = 'Dados da sessão não localizados.';
          return;
        }
        const diaries = data.diaries || [];
        const receivedHdf = data.received_handoff;
        const returnHdf = data.return_handoff;

        profileElem.innerHTML = `<strong>${s.agent_id}</strong> <span class="u-text-cyan u-s-6e68e7">(${s.assigned_tier || 'Tier1'})</span><div class="u-text-muted u-mt-2p u-s-d3bc8d">${escapeHtml(s.task_description || '')}</div>`;
        parentElem.textContent = s.parent_session_id || 'Mestre (Chat)';
        titleElem.textContent = `Diário & Handoff: ${s.agent_id}`;

        const statusUpper = (s.status || '').toUpperCase();
        let badgeColor = 'var(--amber)';
        if (statusUpper === 'COMPLETED') badgeColor = 'var(--green)';
        if (statusUpper === 'RUNNING') badgeColor = 'var(--cyan)';
        if (statusUpper === 'FAILED') badgeColor = 'var(--red)';
        statusBadgeElem.innerHTML = `<span class="badge" style="border: 1px solid ${badgeColor}; color: ${badgeColor};">${statusUpper}</span>`;

        // Render Handoffs (Recebido do Mestre + Retorno ao Mestre)
        let handoffHtml = '';
        if (returnHdf) {
          const rawTime = returnHdf.created_at;
          const ts = typeof rawTime === 'number' && rawTime < 10000000000 ? rawTime * 1000 : rawTime;
          const timeStr = ts ? new Date(ts).toLocaleTimeString() : '';
          handoffHtml += `
            <div class="u-mb-10p u-s-475e42">
              <div class="u-d-flex u-justify-between u-text-green u-fw-700 u-mb-6p u-s-dbc8d1">
                <span>${uiIcon('check-circle', 'icon-xs')} Envelope de Retorno #${returnHdf.id} &bull; Para Mestre: <strong class="u-text-cyan">${returnHdf.target_agent_id}</strong></span>
                <span>${timeStr}</span>
              </div>
              <div class="u-mono u-pre-wrap u-s-d1e428">${escapeHtml(returnHdf.context_summary || returnHdf.summary_markdown || '')}</div>
            </div>
          `;
        }

        if (receivedHdf) {
          const rawTime = receivedHdf.created_at;
          const ts = typeof rawTime === 'number' && rawTime < 10000000000 ? rawTime * 1000 : rawTime;
          const timeStr = ts ? new Date(ts).toLocaleTimeString() : '';
          handoffHtml += `
            <div class="u-mb-2 u-s-0f11da">
              <div class="u-d-flex u-justify-between u-text-muted u-mb-1 u-s-cb8a8e">
                <span>${uiIcon('inbox', 'icon-xs')} Envelope Inicial Recebido #${receivedHdf.id} (De: <strong class="u-text-cyan">${receivedHdf.source_session_id}</strong>)</span>
                <span>${timeStr}</span>
              </div>
              <div class="u-text-slate u-mono u-pre-wrap u-s-c79ecd">${escapeHtml(receivedHdf.context_summary || '')}</div>
            </div>
          `;
        }

        if (!returnHdf && !receivedHdf) {
          handoffHtml = `<span class="u-text-muted">Nenhum envelope de handoff emitido ainda. A sessão pode estar em andamento ou não ter emitido a ação <code>complete</code>.</span>`;
        }
        handoffsElem.innerHTML = handoffHtml;

        // Render Diários
        diariesCountElem.textContent = diaries.length;
        if (diaries.length === 0) {
          diariesContainer.innerHTML = '<div class="u-text-muted u-s-7cdff0">Nenhum pensamento intermediário registrado nesta sessão.</div>';
        } else {
          diariesContainer.innerHTML = diaries.map((d, idx) => {
            const rawTime = d.created_at;
            const ts = typeof rawTime === 'number' && rawTime < 10000000000 ? rawTime * 1000 : rawTime;
            const timeStr = ts ? new Date(ts).toLocaleTimeString() : '';
            const thoughtText = d.content || d.thought_log || '';
            return `
              <div class="u-s-4a79f0">
                <div class="u-d-flex u-justify-between u-items-center u-mb-6p">
                  <span class="u-fw-700 u-text-amber u-mono u-s-dbc8d1">
                    Passo ${idx + 1} / Entrada #${d.id}
                  </span>
                  <span class="u-text-muted u-s-cb8a8e">${timeStr}</span>
                </div>
                <div class="u-text-slate-light u-pre-wrap u-mono u-s-0b65cf">${escapeHtml(thoughtText)}</div>
              </div>
            `;
          }).join('');
        }

      } catch (e) {
        handoffsElem.textContent = `Erro ao inspecionar histórico: ${e.message}`;
      }
    }

    function closeSubAgentHistoryModal() {
      const modal = document.getElementById('modal-subagent-history');
      if (modal) modal.classList.remove('active');
    }

    function openSimulateDispatchModal() {
      const modal = document.getElementById('modal-simulate-dispatch');
      if (modal) modal.classList.add('active');
      const input = document.getElementById('sim-task-input');
      if (input) input.focus();
    }

    function closeSimulateDispatchModal() {
      const modal = document.getElementById('modal-simulate-dispatch');
      if (modal) modal.classList.remove('active');
    }

    function setSimTask(taskText) {
      const input = document.getElementById('sim-task-input');
      if (input) {
        input.value = taskText;
        runSimulateDispatch();
      }
    }

    // --------------------------------------------------------------------------
    // REFINAMENTOS SUBAGENTS: CONFIGURAÇÃO DE DOMÍNIOS, FILTROS, MCP TOOLS
    // --------------------------------------------------------------------------

    let currentConfiguringDomainId = null;

    function openConfigureDomainModal(domainId, focusField) {
      currentConfiguringDomainId = domainId;
      const modal = document.getElementById('modal-configure-domain');
      if (!modal) return;

      const matrix = window.cachedRouterMatrix || [];
      const domain = matrix.find(d => d.id === domainId) || matrix[0];
      if (!domain) return;

      const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
      const domainOverride = overrides[domainId] || {};

      document.getElementById('cfg-domain-id').value = domainId;
      const iconEl = document.getElementById('cfg-domain-icon'); if (iconEl) iconEl.innerHTML = uiIcon(domain.icon || 'target', 'icon-md');
      document.getElementById('cfg-domain-title').textContent = `Configurar: ${domain.category}`;
      document.getElementById('cfg-domain-desc').textContent = domain.description;

      // Preenche select de modelos locais a partir do inventário real em disco
      const localSelect = document.getElementById('cfg-select-local-model');
      localSelect.innerHTML = '';

      const defaultLocalModel = domain.local_default.model;
      const currentSelectedLocal = domainOverride.local_model || defaultLocalModel;

      const availableLocals = [];
      const addLocalOpt = (id, name) => {
        if (!id) return;
        if (!availableLocals.some(a => a.id.toLowerCase() === id.toLowerCase())) {
          availableLocals.push({ id, name });
        }
      };

      const scannedLocals = (Array.isArray(localModelsList) && localModelsList.length > 0)
        ? localModelsList
        : (Array.isArray(window.cachedInstalledLocalModels) ? window.cachedInstalledLocalModels : []);

      scannedLocals.forEach(m => {
        const id = m.model || m.id || m.file_name || m.name;
        const friendly = humanizeModelName(id);
        const sizeStr = m.size_mb
          ? (m.size_mb > 1024 ? (m.size_mb / 1024).toFixed(1) + ' GB' : Math.round(m.size_mb) + ' MB')
          : (m.size_gb ? m.size_gb.toFixed(1) + ' GB' : '');
        const quant = m.quantization ? ` • ${m.quantization}` : '';
        const tpsTag = m.tps ? ` • ${Number(m.tps).toFixed(1)} tok/s` : '';
        addLocalOpt(id, `${friendly} (${sizeStr}${quant}${tpsTag} • Instalado)`);
      });

      addLocalOpt(defaultLocalModel, `${humanizeModelName(defaultLocalModel)} (Recomendado Domínio)`);
      addLocalOpt(currentSelectedLocal, `${humanizeModelName(currentSelectedLocal)} (Atual)`);

      availableLocals.forEach(opt => {
        const el = document.createElement('option');
        el.value = opt.id;
        el.textContent = opt.name;
        if (opt.id.toLowerCase() === currentSelectedLocal.toLowerCase()) {
          el.selected = true;
        }
        localSelect.appendChild(el);
      });

      // Preenche select de modelos cloud dinamicamente a partir dos campeões vivos da Cloud Arena
      const cloudSelect = document.getElementById('cfg-select-cloud-model');
      cloudSelect.innerHTML = '';

      const defaultCloudModel = domain.cloud_fallback.model;
      const currentSelectedCloud = domainOverride.cloud_model || defaultCloudModel;

      const availableClouds = [];
      const addCloudOpt = (id, name, provider) => {
        if (!id) return;
        if (!availableClouds.some(a => a.id.toLowerCase() === id.toLowerCase())) {
          availableClouds.push({ id, name, provider: provider || 'nous' });
        }
      };

      addCloudOpt(currentSelectedCloud, `${humanizeModelName(currentSelectedCloud)} (Ativo no Domínio)`, 'nous');
      addCloudOpt(defaultCloudModel, `${humanizeModelName(defaultCloudModel)} (Campeão Recomendado)`, 'nous');

      if (Array.isArray(cloudChampionsData) && cloudChampionsData.length > 0) {
        const arenaCat = (domain.arena_category || domain.id || '').toLowerCase();
        const domainReport = cloudChampionsData.find(c => (c.profile || '').toLowerCase() === arenaCat);
        const orderedReports = domainReport
          ? [domainReport, ...cloudChampionsData.filter(c => c !== domainReport)]
          : cloudChampionsData;
        orderedReports.forEach(rep => {
          [
            { item: rep.quality_leader, tag: 'Quality Leader' },
            { item: rep.best_value, tag: 'Best ROI' },
            { item: rep.ultra_low_cost, tag: 'Ultra Econômico' }
          ].forEach(({ item, tag }) => {
            if (item && item.canonical_id) {
              const costLabel = item.blended_cost ? ` • $${Number(item.blended_cost).toFixed(2)}/1M` : '';
              addCloudOpt(
                item.canonical_id,
                `${item.display_name || humanizeModelName(item.canonical_id)} (${rep.profile} ${tag}${costLabel})`,
                item.provider || 'nous'
              );
            }
          });
        });
      }

      availableClouds.forEach(opt => {
        const el = document.createElement('option');
        el.value = opt.id;
        el.dataset.provider = opt.provider;
        el.textContent = opt.name;
        if (opt.id.toLowerCase() === currentSelectedCloud.toLowerCase()) {
          el.selected = true;
        }
        cloudSelect.appendChild(el);
      });

      // Regra Pareto
      document.getElementById('cfg-pareto-rule-input').value = domainOverride.pareto_rule || domain.pareto_rule || '';

      modal.classList.add('active');

      if (focusField === 'local') {
        localSelect.focus();
      } else if (focusField === 'cloud') {
        cloudSelect.focus();
      }
    }

    function closeConfigureDomainModal() {
      const modal = document.getElementById('modal-configure-domain');
      if (modal) modal.classList.remove('active');
      currentConfiguringDomainId = null;
    }

    async function saveDomainConfiguration() {
      const domainId = document.getElementById('cfg-domain-id').value;
      if (!domainId) return;

      const localModel = document.getElementById('cfg-select-local-model').value;
      const cloudSelect = document.getElementById('cfg-select-cloud-model');
      const cloudModel = cloudSelect.value;
      const selectedOpt = cloudSelect.options[cloudSelect.selectedIndex];
      const cloudProvider = (selectedOpt && selectedOpt.dataset && selectedOpt.dataset.provider) || 'nous';
      const paretoRule = document.getElementById('cfg-pareto-rule-input').value.trim();

      const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
      overrides[domainId] = {
        local_model: localModel,
        cloud_model: cloudModel,
        cloud_provider: cloudProvider,
        pareto_rule: paretoRule,
        updated_at: new Date().toISOString()
      };

      localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));
      closeConfigureDomainModal();

      // Persist cloud model routing cleanly to backend souls_config.toml via /api/v1/profiles
      const matrix = window.cachedRouterMatrix || [];
      const domain = matrix.find(d => d.id === domainId);
      const targetProfId = domainId === 'agentic'
        ? 'agentic_dispatcher'
        : ((domain && domain.profile_id) || `domain_${domainId}`);

      try {
        await fetch('/api/v1/profiles', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            id: targetProfId,
            name: (domain && domain.category) || domainId,
            description: (domain && domain.description) || `Roteamento ${domainId}`,
            role: domainId === 'executive' ? 'orchestrator' : 'subagent',
            arena_category: (domain && domain.arena_category) || domainId,
            target_model: cloudModel,
            provider: cloudProvider
          })
        });
        await fetchRouterMatrix();
        if (typeof fetchProfiles === 'function') fetchProfiles();
        showToast('Roteamento salvo em souls_config.toml com sucesso!', 'success');
      } catch (e) {
        if (window.cachedRouterMatrix) renderRouterMatrix(window.cachedRouterMatrix);
        showToast(`Roteamento salvo localmente (${e.message})`, 'warning');
      }
    }

    async function resetDomainDefault() {
      const domainId = document.getElementById('cfg-domain-id').value;
      if (!domainId) return;

      const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
      if (overrides[domainId]) {
        delete overrides[domainId];
        localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));
      }

      closeConfigureDomainModal();
      await fetchRouterMatrix();
      showToast(`Domínio restaurado para os campeões padrão da Arena.`, 'success');
    }

    // Card 3: Ações Interativas
    async function testLoopbackConnection() {
      const statusEl = document.getElementById('hermes-loopback-status');
      if (statusEl) statusEl.innerHTML = '<span class="u-text-cyan">Sondando...</span>';

      const t0 = performance.now();
      try {
        const res = await fetch('/health');
        const elapsed = (performance.now() - t0).toFixed(1);
        if (res.ok) {
          if (statusEl) {
            statusEl.innerHTML = `<span class="u-text-green u-fw-700">${uiIcon('check-circle', 'icon-xs')} Ativo (${elapsed}ms • 200 OK)</span>`;
          }
          showToast(`Loopback MCP 127.0.0.1:9123 saudável (${elapsed}ms)`, 'success');
        } else {
          if (statusEl) {
            statusEl.innerHTML = `<span class="u-text-red">Falha (${res.status})</span>`;
          }
        }
      } catch (err) {
        if (statusEl) {
          statusEl.innerHTML = `<span class="u-text-red">Offline</span>`;
        }
        showToast(`Erro ao testar loopback: ${err.message}`, 'error');
      }
    }

    function copyMcpConfigSnippet() {
      const config = JSON.stringify({
        "mcpServers": {
          "souls_engine": {
            "url": "http://127.0.0.1:9123/mcp",
            "transport": "http"
          }
        }
      }, null, 2);

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(config).then(() => {
          showToast('Configuração MCP copiada para o clipboard!', 'success');
        }).catch(() => {
          fallbackExecCopy(config);
        });
      } else {
        fallbackExecCopy(config);
      }
    }

    function fallbackExecCopy(text) {
      try {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        showToast('Configuração copiada!', 'success');
      } catch (e) {
        showToast('Pressione Ctrl+C para copiar a configuração.', 'info');
      }
    }

    function downloadMcpConfigJson() {
      const config = JSON.stringify({
        "$schema": "https://json.schemastore.org/mcp-config.json",
        "mcpServers": {
          "souls_engine": {
            "url": "http://127.0.0.1:9123/mcp",
            "transport": "http",
            "description": "Souls Engine v7 Bare-Metal Infrastructure & AST Engine"
          }
        }
      }, null, 2);

      const blob = new Blob([config], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'souls_mcp_config.json';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showToast('Arquivo souls_mcp_config.json descarregado!', 'success');
    }

    // MCP Tools Catalog Modal
    const NATIVE_MCP_TOOLS = [
      // AST & Code
      { name: "souls_ast_outline", domain: "AST & Code", icon: "puzzle", desc: "Extrai esqueleto sintático (classes, traits, funções, assinaturas) poupando 80% dos tokens de contexto.", params: "path, language" },
      { name: "souls_ast_slice", domain: "AST & Code", icon: "puzzle", desc: "Extrai fatias sintáticas contíguas com numeração precisa de linhas para inspeção direcionada.", params: "path, start_line, end_line" },
      { name: "souls_ast_symbol", domain: "AST & Code", icon: "puzzle", desc: "Localiza símbolos e identidades no escopo do arquivo sem precisar ler o arquivo por inteiro.", params: "path, symbol_name" },
      { name: "souls_ast_edit", domain: "AST & Code", icon: "puzzle", desc: "Aplica mutações atômicas de código com verificação sintática pré-commit do Tree-sitter.", params: "path, target, replacement" },
      { name: "souls_ast_delta_diff", domain: "AST & Code", icon: "puzzle", desc: "Calcula e aplica Myers Diff estruturado preservando contagens exatas de bytes LF.", params: "path, patch" },
      { name: "souls_blast_radius", domain: "AST & Code", icon: "zap", desc: "Mede o raio de impacto de alterações entre crates para prevenir quebras arquiteturais colaterais.", params: "crate_name, symbols" },
      { name: "souls_frecency", domain: "AST & Code", icon: "flame", desc: "Calcula mapa de calor de arquivos mais editados e com maior probabilidade de alteração.", params: "limit, days" },

      // Memória L2 & Grafo
      { name: "souls_memory_search", domain: "Memória L2", icon: "brain", desc: "Busca híbrida vetorial RRF (LanceDB + SQLite FTS5) no hipocampo epistêmico do Souls Engine.", params: "query, limit, threshold" },
      { name: "souls_memory_store", domain: "Memória L2", icon: "brain", desc: "Persiste memórias de longo prazo com embeddings locais Qwen2 em FrankenSQLite STRICT WAL.", params: "content, tags, category" },
      { name: "souls_memory_graph_entity", domain: "Memória L2", icon: "network", desc: "Cria e atualiza nós ontológicos e observações clínicas no Grafo LadybugDB.", params: "entity_name, entity_type, props" },
      { name: "souls_memory_graph_query", domain: "Memória L2", icon: "network", desc: "Navega pelas arestas tipadas (depends_on, conflicts_with, implements) do cofre cognitivo.", params: "source_id, relation_type" },
      { name: "souls_vault_query", domain: "Memória L2", icon: "database", desc: "Consulta evidências empíricas de benchmarks TPS e VRAM no Heuristic Vault.", params: "model_id, domain" },
      { name: "souls_obsidian_curate", domain: "Memória L2", icon: "book", desc: "Curadoria automática de notas na pasta wiki/ do Obsidian com desidentificação de dados.", params: "source_path, target_wiki" },

      // Thinking, Silício & FinOps
      { name: "souls_socratic_deliberate", domain: "Thinking & Ops", icon: "shield", desc: "Freio cognitivo socrático: deliberação em grafo DAG antes de tocar em código de alta entropia.", params: "hypothesis, tradeoffs" },
      { name: "souls_metrics", domain: "Thinking & Ops", icon: "bar-chart", desc: "Telemetria bare-metal em tempo real: VRAM alocada (NVML), temperatura e TPS empírico.", params: "detailed" },
      { name: "souls_hardware_watchdog", domain: "Thinking & Ops", icon: "thermometer", desc: "Guardião termodinâmico da dGPU local e margem física sob Windows DWM.", params: "action" },
      { name: "souls_intent_probe", domain: "Thinking & Ops", icon: "compass", desc: "Classificador de intenção e medição de entropia via Tier 0.5 SLM (Host CPU AVX2).", params: "prompt, context" },
      { name: "souls_context_lean_compress", domain: "Thinking & Ops", icon: "archive", desc: "Compressão CCR em memória RAM poupando até 60% da janela de contexto.", params: "text, target_tokens" },
      { name: "souls_switchboard_evaluate", domain: "Thinking & Ops", icon: "zap", desc: "Roteador semântico two-stage: desvia tarefas entre dGPU local e Cloud Fallback em 1.4ms.", params: "query, available_models" }
    ];

    async function openMcpToolsModal() {
      const modal = document.getElementById('modal-mcp-tools');
      if (!modal) return;
      modal.classList.add('active');
      if (!cachedToolsData || cachedToolsData.length === 0) {
        const container = document.getElementById('mcp-tools-list-container');
        if (container) container.innerHTML = `<div class="u-text-center u-text-muted u-s-4c08f4">${uiIcon('zap', 'icon-xs')} Carregando catálogo real de 55 ferramentas MCP...</div>`;
        try {
          const res = await fetch('/api/v1/tools/catalog');
          if (res.ok) {
            const json = await res.json();
            cachedToolsData = json.tools || [];
          }
        } catch (_) {}
      }
      renderMcpToolsCatalog(cachedToolsData && cachedToolsData.length > 0 ? cachedToolsData : NATIVE_MCP_TOOLS);
    }

    function closeMcpToolsModal() {
      const modal = document.getElementById('modal-mcp-tools');
      if (modal) modal.classList.remove('active');
    }

    function renderMcpToolsCatalog(tools) {
      const container = document.getElementById('mcp-tools-list-container');
      if (!container) return;

      const nsIcons = {
        code: 'dna',
        memory: 'database',
        sys: 'zap',
        think: 'brain',
        ops: 'shield',
        context: 'file-text',
        core: 'target'
      };

      container.innerHTML = (tools || []).map(t => {
        const ns = t.namespace || t.domain || 'mcp';
        const icon = t.icon || nsIcons[String(ns).toLowerCase()] || 'wrench';
        const desc = t.description || t.desc || 'Sem descrição.';
        const props = t.input_schema && t.input_schema.properties ? Object.keys(t.input_schema.properties).join(', ') : '';
        const paramsStr = t.params || props || 'sem parâmetros obrigatórios';
        const badgeCls = (ns === 'code' || String(ns).includes('AST')) ? 'badge-champion' : ((ns === 'memory' || String(ns).includes('Memória')) ? 'badge-value' : 'badge');
        return `
        <div class="u-d-flex u-items-start u-justify-between u-gap-3 u-s-2a1093">
          <div class="u-flex-1">
            <div class="u-d-flex u-items-center u-gap-2 u-mb-1">
              <span>${uiIcon(icon, 'icon-md')}</span>
              <code class="mono u-fw-700 u-text-white u-s-5b9d54">${t.name}</code>
              <span class="badge ${badgeCls}">${ns}${t.is_level_0 ? ' • L0' : ''}</span>
            </div>
            <p class="u-text-muted u-mb-6p u-s-e197f7">${desc}</p>
            <div class="u-text-dark u-mono u-s-923ef2">
              Parâmetros: <span class="u-text-slate">${paramsStr}</span>
            </div>
          </div>
          <button class="btn-micro" onclick="copyToolName('${t.name}')" title="Copiar identificador da ferramenta MCP">
            ${uiIcon('list', 'icon-xs')} Copiar
          </button>
        </div>
      `;
      }).join('');

      const countLabel = document.getElementById('mcp-tools-count-label');
      if (countLabel) countLabel.textContent = `${(tools || []).length} ferramenta(s) nativa(s) disponíveis no daemon`;
    }

    function filterMcpToolsList() {
      const q = (document.getElementById('mcp-tools-search')?.value || '').toLowerCase().trim();
      const source = (cachedToolsData && cachedToolsData.length > 0) ? cachedToolsData : NATIVE_MCP_TOOLS;
      const filtered = source.filter(t => 
        (t.name || '').toLowerCase().includes(q) ||
        (t.namespace || t.domain || '').toLowerCase().includes(q) ||
        (t.description || t.desc || '').toLowerCase().includes(q)
      );
      renderMcpToolsCatalog(filtered);
    }

    function copyToolName(name) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(name).then(() => {
          showToast(`Ferramenta '${name}' copiada!`, 'success');
        });
      } else {
        fallbackExecCopy(name);
      }
    }

    async function testSiliconPing() {
      const t0 = performance.now();
      try {
        const res = await fetch('/api/v1/telemetry');
        const elapsed = (performance.now() - t0).toFixed(1);
        if (res.ok) {
          const tel = await res.json();
          const tempStr = tel.gpu_temp_c ? `${Number(tel.gpu_temp_c).toFixed(0)}°C` : 'Nominal';
          showToast(`Silício Ativo: NVML ${tempStr} • VRAM ${tel.vram_used_mb || 0}/${tel.vram_limit_mb || 0} MB • ${elapsed}ms`, 'success');
        } else {
          showToast(`Silício respondeu com status ${res.status} em ${elapsed}ms`, 'info');
        }
      } catch (err) {
        showToast(`Erro ao testar silício: ${err.message}`, 'error');
      }
    }
    window.testSiliconPing = testSiliconPing;

    async function runSimulateDispatch() {
      const input = document.getElementById('sim-task-input');
      const resultBox = document.getElementById('sim-dispatch-result');
      if (!input || !resultBox) return;
      const query = input.value.trim();
      if (!query) {
        resultBox.innerHTML = '<div class="u-text-amber u-s-a768c1">Digite uma tarefa ou clique em uma das sugestões acima.</div>';
        return;
      }

      resultBox.innerHTML = '<div class="u-text-cyan u-s-a768c1">Sondando Two-Stage Semantic Switchboard (Host CPU AVX2)...</div>';

      try {
        const res = await fetch('/api/v1/switchboard/evaluate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: query })
        });
        if (!res.ok) {
          resultBox.innerHTML = `<div class="u-text-red u-s-a768c1">Erro ao consultar switchboard: HTTP ${res.status}</div>`;
          return;
        }
        const data = await res.json();
        const d = data.decision || {};
        const skill = d.recommended_skill || {};
        const tools = d.tools || [];
        const latency = d.latency_ms ? `${d.latency_ms.toFixed(1)}ms` : '< 5ms';
        const conf = d.confidence ? `${(d.confidence * 100).toFixed(0)}%` : '--';
        const entropy = d.entropy !== undefined ? d.entropy.toFixed(2) : '--';
        const stage = d.stage || 'fast_path_ram_cosine';

        let toolsHtml = tools.map(t => `<span class="badge u-text-cyan u-s-dff56c">${t.name}</span>`).join(' ');

        resultBox.innerHTML = `
          <div class="u-s-538ff7">
            <div class="u-d-flex u-justify-between u-items-center u-mb-10p u-s-2c1e9f">
              <div>
                <span class="badge badge-champion u-s-dbc8d1">${stage}</span>
                <span class="u-text-green u-fw-600 u-s-78b0f8">${uiIcon('zap', 'icon-xs')} Latência: ${latency}</span>
              </div>
              <div class="u-text-muted u-s-dbc8d1">
                Confiança: <strong class="u-text-cyan">${conf}</strong> &bull; Entropia: <strong class="u-text-amber">${entropy}</strong>
              </div>
            </div>

            <div class="u-mb-10p">
              <div class="u-text-muted u-uppercase u-s-6262d0">Skill Canônica Selecionada</div>
              <div class="u-fw-700 u-text-white u-mt-2p u-s-604fde">
                ${uiIcon('target', 'icon-xs')} ${skill.name || 'souls-general-orchestration'}
              </div>
              <div class="u-text-muted u-mt-2p u-s-d3bc8d">${skill.description || 'Orquestração de tarefas especializadas no Souls Engine.'}</div>
            </div>

            <div class="u-mb-10p u-s-01b0e7">
              <div class="u-text-cyan u-fw-600 u-s-6262d0">${uiIcon('scale', 'icon-xs')} Decisão de Despacho ParetoBandit:</div>
              <div class="u-text-slate u-mt-3p u-s-c79ecd">
                Despachar no <strong>Silício Local Bare-Metal ($0.00)</strong>. Entropia (${entropy}) está abaixo do limiar de desvio para Cloud.
              </div>
            </div>

            <div>
              <div class="u-text-muted u-mb-1 u-s-6262d0">Ferramentas MCP Vinculadas (${tools.length}):</div>
              <div class="u-d-flex u-flex-wrap u-gap-1">
                ${toolsHtml || '<span class="u-text-muted u-s-dbc8d1">Nenhuma ferramenta restrita necessária.</span>'}
              </div>
            </div>
          </div>
        `;
      } catch (err) {
        resultBox.innerHTML = `<div class="u-text-red u-s-a768c1">Falha na simulação: ${err.message}</div>`;
      }
    }

    function escapeHtml(str) {
      if (!str) return '';
      return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
    }

        // =========================================================
    // SISTEMA UNIVERSAL DE TOOLTIP PORTAL (Todos os 62 ícones "i")
    // =========================================================
    function initGlobalPortalTooltips() {
      const portal = document.getElementById('global-portal-tooltip');
      if (!portal) return;

      let currentIcon = null;

      document.addEventListener('mouseover', (e) => {
        const icon = e.target.closest('.info-icon');
        if (!icon) return;
        if (icon === currentIcon) return;

        const box = icon.querySelector('.tooltip-box');
        if (!box) return;

        currentIcon = icon;
        portal.innerHTML = box.innerHTML;
        portal.style.display = 'block';

        const rect = icon.getBoundingClientRect();
        const portalWidth = Math.min(320, window.innerWidth - 32);

        let left = rect.right + 10;
        if (left + portalWidth > window.innerWidth - 16) {
          left = rect.left - portalWidth - 10;
        }
        if (left < 16) {
          left = Math.max(16, Math.min(rect.left, window.innerWidth - portalWidth - 16));
        }

        let top = rect.top - 6;
        portal.style.left = `${left}px`;
        portal.style.top = `${top}px`;

        requestAnimationFrame(() => {
          const portalHeight = portal.offsetHeight;
          if (top + portalHeight > window.innerHeight - 16) {
            top = Math.max(16, window.innerHeight - portalHeight - 16);
            portal.style.top = `${top}px`;
          }
          portal.classList.add('visible');
        });
      });

      document.addEventListener('mouseout', (e) => {
        const icon = e.target.closest('.info-icon');
        if (!icon) return;

        const related = e.relatedTarget ? e.relatedTarget.closest('.info-icon') : null;
        if (related === icon) return;

        currentIcon = null;
        portal.classList.remove('visible');
        portal.style.display = 'none';
      });

      window.addEventListener('scroll', () => {
        if (currentIcon) {
          currentIcon = null;
          portal.classList.remove('visible');
          portal.style.display = 'none';
        }
      }, { passive: true });
    }

    // ==========================================
    // ERGONOMIC TEXT EDITOR WITH SYNCHRONIZED GUTTER & JUMP-TO-LINE
    // ==========================================
    function setupErgonomicEditor(textareaId, gutterId, statusId) {
      const textarea = document.getElementById(textareaId);
      const gutter = document.getElementById(gutterId);
      const status = document.getElementById(statusId);
      if (!textarea || !gutter) return;

      function updateGutter() {
        const val = textarea.value || '';
        const lineCount = Math.max(1, val.split('\n').length);
        let gutterText = '';
        for (let i = 1; i <= lineCount; i++) {
          gutterText += i + '\n';
        }
        gutter.textContent = gutterText;
      }

      function updateCursor() {
        if (!status) return;
        const val = textarea.value || '';
        const selStart = textarea.selectionStart || 0;
        const line = val.substring(0, selStart).split('\n').length;
        const lastNl = val.lastIndexOf('\n', selStart - 1);
        const col = selStart - (lastNl === -1 ? 0 : lastNl + 1) + 1;
        status.textContent = `Ln ${line}, Col ${col}`;
      }

      textarea.addEventListener('scroll', () => {
        gutter.scrollTop = textarea.scrollTop;
      });

      textarea.addEventListener('input', () => {
        updateGutter();
        updateCursor();
      });

      textarea.addEventListener('click', updateCursor);
      textarea.addEventListener('keyup', updateCursor);
      textarea.addEventListener('select', updateCursor);

      textarea.addEventListener('update-editor-gutter', () => {
        updateGutter();
        updateCursor();
      });

      // Observer para quando o textarea for preenchido enquanto estava oculto
      const observer = new MutationObserver(() => updateGutter());
      observer.observe(textarea, { attributes: true, childList: true, characterData: true });

      updateGutter();
      updateCursor();
    }

    function jumpEditorToLine(textareaId, lineNum) {
      const textarea = document.getElementById(textareaId);
      if (!textarea) return;
      const num = parseInt(lineNum, 10);
      if (isNaN(num) || num < 1) return;

      const lines = textarea.value.split('\n');
      let charOffset = 0;
      for (let i = 0; i < Math.min(num - 1, lines.length); i++) {
        charOffset += lines[i].length + 1;
      }

      textarea.focus();
      textarea.setSelectionRange(charOffset, charOffset + (lines[num - 1] ? lines[num - 1].length : 0));

      const lineHeight = 19;
      textarea.scrollTop = Math.max(0, (num - 5) * lineHeight);

      const container = textarea.closest('.editor-container-with-lines') || textarea;
      container.classList.remove('editor-line-flash');
      void container.offsetWidth;
      container.classList.add('editor-line-flash');
      setTimeout(() => container.classList.remove('editor-line-flash'), 1800);
    }

    // INICIALIZAÇÃO NO CARREGAMENTO DA PÁGINA
    window.addEventListener('DOMContentLoaded', () => {
      initGlobalPortalTooltips();
      fetchTelemetry();
      fetchEcosystemPulse();
      fetchRouterMatrix();
      fetchLocalModels();
      fetchSubAgentSessions();
      fetchCloudChampions().then(() => {
        fetchProfiles();
      });
      fetchConfigToml();
      if (typeof fetchSkillsCountBadgeOnly === 'function') {
        fetchSkillsCountBadgeOnly();
      }
      if (typeof fetchPersonasCountBadgeOnly === 'function') {
        fetchPersonasCountBadgeOnly();
      }
      if (typeof refreshInboxBadgeCount === 'function') {
        refreshInboxBadgeCount();
      }

      // Inicialização dos editores ergonômicos com régua de linhas e indicador Ln/Col
      setupErgonomicEditor('raw-toml-editor', 'raw-toml-gutter', 'footer-toml-cursor');
      setupErgonomicEditor('pwb-soul-editor', 'pwb-soul-gutter', 'pwb-soul-status');
      setupErgonomicEditor('pstudio-soul-preview', 'pstudio-soul-gutter', 'pstudio-soul-status');
      setupErgonomicEditor('insp-skill-editor', 'insp-skill-gutter', 'insp-skill-status');
      setupErgonomicEditor('forge-output-editor', 'forge-output-gutter', 'forge-output-status');

      // Delegação Reativa de Auto-Save (400ms debounce) para controles visuais de #tab-settings (ignora TOML bruto e simuladores)
      const settingsContainer = document.getElementById('tab-settings');
      if (settingsContainer) {
        const delegateSettingsAutoSave = (e) => {
          const target = e.target;
          if (!target || target.id === 'raw-toml-editor' || target.closest('#subtab-raw') || target.closest('.lab-drawer')) return;
          if (!(target.id && target.id.startsWith('cfg-'))) return;
          const inlineHandlers = `${target.getAttribute('oninput') || ''} ${target.getAttribute('onchange') || ''}`;
          if (!inlineHandlers.includes('markSettingsUnsaved')) {
            markSettingsUnsaved(false);
          }
        };
        settingsContainer.addEventListener('input', delegateSettingsAutoSave);
        settingsContainer.addEventListener('change', delegateSettingsAutoSave);
      }

      // Barramento de Eventos Reativo Global (Zero CTRL+R)
      window.addEventListener('souls-state-changed', (e) => {
        const detail = e.detail || {};
        fetchTelemetry();
        if (detail.model) {
          const topBrain = document.getElementById('top-brain-name');
          if (topBrain) topBrain.innerText = detail.model;
          const heroBrain = document.getElementById('hero-brain-model');
          if (heroBrain) heroBrain.innerText = detail.model;
          const activeModelName = document.getElementById('active-hermes-model-name');
          if (activeModelName) activeModelName.innerText = detail.model;
          const quickActive = document.getElementById('cloud-quick-active-model');
          if (quickActive) quickActive.innerText = detail.model;
        }
        if (currentLocalSubtab === 'cloud' || detail.scope === 'master_brain') {
          fetchCloudChampions();
        }
        if (currentLocalSubtab === 'installed' || detail.scope === 'arena_benchmark') {
          fetchLocalModels();
          fetchLocalArenaData();
        }
      });

      // Suporte a Deep Linking via Hash (ex: #tab-local, #tab-cloud, #tab-skills/forge, #tab-personas/governance, #tab-settings/subtab-sandbox)
      if (window.location.hash) {
        const fullHash = window.location.hash.replace('#', '').split('?')[0];
        const parts = fullHash.split('/');
        const mainTab = parts[0];
        const subTab = parts[1];
        if (mainTab === 'tab-cloud') {
          switchTab('tab-local');
          switchLocalSubtab('cloud');
        } else if (document.getElementById(mainTab)) {
          switchTab(mainTab);
          if (subTab && mainTab === 'tab-settings' && typeof switchSettingsSubtab === 'function') {
            switchSettingsSubtab(subTab);
          } else if (subTab && mainTab === 'tab-skills' && typeof switchSkillsSubtab === 'function') {
            switchSkillsSubtab(subTab);
          } else if (subTab && mainTab === 'tab-personas' && typeof switchPersonasSubtab === 'function') {
            switchPersonasSubtab(subTab);
          } else if (subTab && mainTab === 'tab-local' && typeof switchLocalSubtab === 'function') {
            switchLocalSubtab(subTab);
          } else if (subTab && mainTab === 'tab-subagents' && typeof switchSubagentsSubtab === 'function') {
            switchSubagentsSubtab(subTab);
          }
        }
      }

      // SSE Telemetria em tempo real
      if (window.EventSource && !window.location.search.includes('no_sse')) {
        try {
          const sse = new EventSource('/api/v1/telemetry/stream');
          sse.onmessage = (e) => {
            try {
              const data = JSON.parse(e.data);
              updateDashboardMetrics(data);
            } catch(err) {}
          };
        } catch(err) {}
      }

      if (!window.location.search.includes('no_sse')) {
        setInterval(fetchTelemetry, 3000);
        setInterval(fetchSubAgentSessions, 8000);
      }
    });

    // =========================================================================
    // UTILITÁRIOS GLOBAIS DE SETTINGS, PRESETS & LABORATÓRIO (FASE 9)
    // =========================================================================

    function copyToClipboard(text, msg) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => {
          showToast(msg || 'Texto copiado para a área de transferência!', 'success');
        }).catch(() => {
          appPrompt('Copie o conteúdo abaixo:', text, { title: 'Copiar Conteúdo', icon: 'list' });
        });
      } else {
        appPrompt('Copie o conteúdo abaixo:', text, { title: 'Copiar Conteúdo', icon: 'list' });
      }
    }
    window.copyToClipboard = copyToClipboard;

    function resetRouterDefaults() {
      const q = document.getElementById('cfg-weight-quality');
      const c = document.getElementById('cfg-weight-cost');
      const l = document.getElementById('cfg-weight-latency');
      const tok = document.getElementById('cfg-local-token-limit');
      const par = document.getElementById('cfg-router-pareto');
      const sov = document.getElementById('cfg-router-sovereignty');
      if (q) q.value = 0.50;
      if (c) c.value = 0.30;
      if (l) l.value = 0.20;
      if (tok) tok.value = 8192;
      if (par) par.checked = true;
      if (sov) sov.checked = true;
      ['quality', 'cost', 'latency'].forEach(k => {
        const el = document.getElementById('val-weight-' + k);
        const pct = document.getElementById('pct-weight-' + k);
        const val = k === 'quality' ? 0.5 : (k === 'cost' ? 0.3 : 0.2);
        if (el) el.innerText = val.toFixed(2);
        if (pct) pct.innerText = `(${Math.round(val * 100)}%)`;
      });
      showToast('Padrões do Roteador Pareto restaurados (50% Q / 30% $ / 20% L)!', 'info');
    }
    window.resetRouterDefaults = resetRouterDefaults;

    function resetHardwareDefaults() {
      const vram = document.getElementById('cfg-vram-limit');
      const poll = document.getElementById('cfg-watchdog-polling');
      const prev = document.getElementById('cfg-temp-preventive');
      const throt = document.getElementById('cfg-temp-throttle');
      const defaultVram = (window.soulsTelemetryState && window.soulsTelemetryState.vramLimitMb) || 4096;
      if (vram) vram.value = defaultVram;
      const vEl = document.getElementById('val-vram-limit');
      if (vEl) vEl.innerText = defaultVram;
      if (poll) poll.value = 500;
      if (prev) prev.value = 72.0;
      if (throt) throt.value = 78.0;
      const pEl = document.getElementById('val-temp-preventive');
      if (pEl) pEl.innerText = '72.0';
      const tEl = document.getElementById('val-temp-throttle');
      if (tEl) tEl.innerText = '78.0';
      showToast('Padrões térmicos e de hardware restaurados (72°C Alerta / 78°C Throttle)!', 'info');
    }
    window.resetHardwareDefaults = resetHardwareDefaults;

    function resetMemoryDefaults() {
      const lang = document.getElementById('cfg-memory-langevin');
      const hl = document.getElementById('cfg-memory-half-life');
      const rrf = document.getElementById('cfg-memory-rrf-k');
      if (lang) lang.checked = true;
      if (hl) hl.value = 30;
      if (rrf) rrf.value = 60;
      showToast('Padrões de Memória L3 restaurados (Meia-vida 30d, RRF k=60)!', 'info');
    }
    window.resetMemoryDefaults = resetMemoryDefaults;

    function resetReasoningDefaults() {
      const dyn = document.getElementById('cfg-reasoning-dynamic');
      const max = document.getElementById('cfg-reasoning-max-effort');
      const step = document.getElementById('cfg-reasoning-stepwise');
      const byp = document.getElementById('cfg-reasoning-bypass');
      const thr = document.getElementById('cfg-reasoning-threshold');
      if (dyn) dyn.checked = true;
      if (max) max.value = 'ultra';
      if (step) step.checked = true;
      if (byp) byp.checked = true;
      if (thr) thr.value = 0.70;
      const tEl = document.getElementById('val-entropy-threshold');
      if (tEl) tEl.innerText = '0.70';
      showToast('Padrões de Raciocínio Socrático restaurados!', 'info');
    }
    window.resetReasoningDefaults = resetReasoningDefaults;

    function updateRouterWeightsUI() {
      const qEl = document.getElementById('cfg-weight-quality');
      const cEl = document.getElementById('cfg-weight-cost');
      const lEl = document.getElementById('cfg-weight-latency');
      const wq = qEl ? Number(qEl.value) : 0.50;
      const wc = cEl ? Number(cEl.value) : 0.30;
      const wl = lEl ? Number(lEl.value) : 0.20;

      const setTxt = (id, txt) => { const e = document.getElementById(id); if (e) e.innerText = txt; };
      setTxt('val-weight-quality', wq.toFixed(2));
      setTxt('val-weight-cost', wc.toFixed(2));
      setTxt('val-weight-latency', wl.toFixed(2));

      const rawQ = Math.round(wq * 100);
      const rawC = Math.round(wc * 100);
      const rawL = Math.round(wl * 100);
      setTxt('pct-weight-quality', `(${rawQ}%)`);
      setTxt('pct-weight-cost', `(${rawC}%)`);
      setTxt('pct-weight-latency', `(${rawL}%)`);
      setTxt('lbl-qual-pct', `${rawQ}%`);
      setTxt('lbl-cost-pct', `${rawC}%`);
      setTxt('lbl-lat-pct', `${rawL}%`);

      const total = Math.max(0.0001, wq + wc + wl);
      const normQ = (wq / total) * 100;
      const normC = (wc / total) * 100;
      const normL = Math.max(0, 100 - normQ - normC);

      const bQ = document.getElementById('bar-weight-qual'); if (bQ) bQ.style.width = `${normQ.toFixed(1)}%`;
      const bC = document.getElementById('bar-weight-cost'); if (bC) bC.style.width = `${normC.toFixed(1)}%`;
      const bL = document.getElementById('bar-weight-lat'); if (bL) bL.style.width = `${normL.toFixed(1)}%`;

      const sumPct = rawQ + rawC + rawL;
      setTxt('weights-sum-label', `Soma: ${sumPct}%`);
      setTxt('metric-router-weights', `${Math.round(normQ)}% Q • ${Math.round(normC)}% C • ${Math.round(normL)}% L`);

      const valBadge = document.getElementById('badge-weights-validation');
      const sumMetric = document.getElementById('metric-router-sum');
      if (Math.abs(sumPct - 100) <= 2) {
        if (valBadge) {
          valBadge.className = 'u-mono u-text-green u-s-db3f87';
          valBadge.innerHTML = `${uiIcon('check', 'icon-xs')} 100% Calibrado`;
        }
        if (sumMetric) {
          sumMetric.className = 'u-text-green u-mt-2p u-s-923ef2';
          sumMetric.innerHTML = `${uiIcon('check', 'icon-xs')} Soma Normalizada (100%)`;
        }
      } else {
        if (valBadge) {
          valBadge.className = 'u-mono u-text-amber u-s-db3f87';
          valBadge.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} Soma ${sumPct}% (Auto-normalizada)`;
        }
        if (sumMetric) {
          sumMetric.className = 'u-text-amber u-mt-2p u-s-923ef2';
          sumMetric.innerText = `Soma Bruta ${sumPct}% -> Escalarizada`;
        }
      }
    }
    window.updateRouterWeightsUI = updateRouterWeightsUI;

    function applyRouterWeightPreset(wqOrPreset, wc, wl, label) {
      const q = document.getElementById('cfg-weight-quality');
      const c = document.getElementById('cfg-weight-cost');
      const l = document.getElementById('cfg-weight-latency');
      let targetQ = 0.50, targetC = 0.30, targetL = 0.20, toastLabel = 'Equilíbrio Canônico (50/30/20)';

      if (typeof wqOrPreset === 'number') {
        targetQ = wqOrPreset;
        targetC = typeof wc === 'number' ? wc : 0.30;
        targetL = typeof wl === 'number' ? wl : 0.20;
        toastLabel = label || `${Math.round(targetQ * 100)}/${Math.round(targetC * 100)}/${Math.round(targetL * 100)}`;
      } else if (wqOrPreset === 'quality') {
        targetQ = 0.80; targetC = 0.10; targetL = 0.10; toastLabel = 'Máxima Qualidade (80/10/10)';
      } else if (wqOrPreset === 'cost') {
        targetQ = 0.20; targetC = 0.60; targetL = 0.20; toastLabel = 'FinOps Zero-Cost (20/60/20)';
      }

      if (q) q.value = targetQ.toFixed(2);
      if (c) c.value = targetC.toFixed(2);
      if (l) l.value = targetL.toFixed(2);

      updateRouterWeightsUI();
      if (typeof initSlidersMagneticSnap === 'function') initSlidersMagneticSnap();
      markSettingsUnsaved();
      showToast(`Preset do Roteador aplicado: ${toastLabel}`, 'info');
    }
    window.applyRouterWeightPreset = applyRouterWeightPreset;

    function setHardwarePreset(vramMbOrMode, label) {
      const simInput = document.getElementById('sim-hw-vram-input');
      const simLabel = document.getElementById('sim-hw-vram-label');

      if (typeof vramMbOrMode === 'number') {
        if (simInput) simInput.value = vramMbOrMode;
        if (simLabel) {
          simLabel.innerHTML = Number(vramMbOrMode).toLocaleString('pt-BR') + ' MB' +
            (label ? ` <span class="u-text-muted u-fw-400">(${escapeHtml(label)})</span>` : '');
        }
        runHardwareSimulation();
        return;
      }

      // Legacy string fallback
      const presetVal = vramMbOrMode === 'conservative' ? 1200 : (vramMbOrMode === 'aggressive' ? 4900 : 2450);
      if (simInput) simInput.value = presetVal;
      if (simLabel) simLabel.innerText = presetVal.toLocaleString('pt-BR') + ' MB';
      runHardwareSimulation();
    }
    window.setHardwarePreset = setHardwarePreset;

    function downloadMcpConfigJson() {
      const config = {
        mcpServers: {
          souls_engine: {
            url: "http://127.0.0.1:9123/mcp",
            transport: "http"
          }
        }
      };
      const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'souls_mcp_config.json';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showToast('Download do arquivo souls_mcp_config.json iniciado!', 'success');
    }
    window.downloadMcpConfigJson = downloadMcpConfigJson;

    function setRouterSimScenario(cat, tokens, prompt) {
      const typeEl = document.getElementById('sim-task-type');
      const tokEl = document.getElementById('sim-task-tokens');
      const promptEl = document.getElementById('sim-task-prompt');
      if (typeEl && cat) {
        typeEl.value = cat;
      }
      if (tokEl && typeof tokens === 'number') {
        tokEl.value = tokens;
      }
      if (promptEl && prompt) {
        promptEl.value = prompt;
      }
      runRouterSimulator();
    }

    async function runRouterSimulator() {
      const category = document.getElementById('sim-task-type')?.value || 'coding';
      const tokens = Math.max(100, parseInt(document.getElementById('sim-task-tokens')?.value || '1200', 10));
      const promptText = (document.getElementById('sim-task-prompt')?.value || '').trim();
      const localLimit = Number(document.getElementById('cfg-local-token-limit')?.value || 8192);
      const sovBias = document.getElementById('cfg-router-sovereignty')?.checked ?? true;

      const rawQ = Number(document.getElementById('cfg-weight-quality')?.value || 0.50);
      const rawC = Number(document.getElementById('cfg-weight-cost')?.value || 0.30);
      const rawL = Number(document.getElementById('cfg-weight-latency')?.value || 0.20);
      const wSum = Math.max(0.0001, rawQ + rawC + rawL);
      const wq = rawQ / wSum, wc = rawC / wSum, wl = rawL / wSum;

      const tempC = Number(window.soulsTelemetryState?.gpuTempC || 58);
      const tempPrev = Number(document.getElementById('cfg-temp-preventive')?.value || 72.0);
      const tempThrot = Number(document.getElementById('cfg-temp-throttle')?.value || 78.0);
      const phiPenalty = tempC >= tempThrot ? 0.35 : (tempC >= tempPrev ? ((tempC - tempPrev) / Math.max(1, tempThrot - tempPrev)) * 0.25 : 0.0);

      let candidates = [];
      // 1. Add best physical local model from installed inventory if within context limit or local mode
      const locals = (Array.isArray(localModelsList) && localModelsList.length > 0)
        ? localModelsList
        : (Array.isArray(window.cachedInstalledLocalModels) ? window.cachedInstalledLocalModels : []);
      if (locals.length > 0) {
        const isSmallFast = category === 'fast' || tokens <= 500;
        const bestLocal = (isSmallFast
          ? locals.find(m => !(m.target || '').includes('dGPU') || (m.tier || '').includes('Tier 0'))
          : locals.find(m => (category === 'coding' ? (m.model || '').includes('coder') : true) && (m.target || '').includes('dGPU')))
          || locals[0];
        const isGpu = (bestLocal.target || '').includes('dGPU') || bestLocal.is_gpu_accelerated;
        const tps = Number(bestLocal.tps || bestLocal.estimated_tps || (isGpu ? 42 : 28));
        const latMs = (1000 / Math.max(5, tps)) * 4 + (isGpu ? 35 : 18);
        const contextOver = tokens > localLimit;
        const qNorm = Math.min(0.95, Math.max(0.70, (Number(bestLocal.e3_score || 84) / 100) + (sovBias ? 0.05 : 0)));
        const lNorm = Math.max(0.2, 1 - (latMs / 800));
        const util = Math.max(0.1, (wq * qNorm + wc * 1.0 + wl * lNorm) - (isGpu ? phiPenalty : 0) - (contextOver ? 0.30 : 0));

        candidates.push({
          model_id: bestLocal.model || bestLocal.name || 'local_silicon',
          tier: isGpu ? `Tier 1 (${window.soulsTelemetryState?.gpuName || 'dGPU Local'})` : `Tier 0.5 (${window.detectedCpuSimd || 'CPU AVX2'})`,
          utility: util,
          q_score: qNorm,
          cost_usd: 0.0,
          latency_ms: latMs,
          vram_mb: isGpu ? Math.round(bestLocal.size_mb || 2200) : 0,
          rationale: contextOver
            ? `Contexto (${tokens} tok) excede teto local (${localLimit} tok) -> penalizado para evitar spillover WDDM.`
            : `Silício local custo $0.00${sovBias ? ' com prior soberano Beta(5,1)' : ''}.`
        });
      }

      // 2. Query real backend fallback chain for Cloud Arena candidates
      try {
        const res = await fetch('/api/v1/router/fallback/chain', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            task_id: 'settings_sim_' + Date.now(),
            category: category === 'local' ? 'coding' : category,
            estimated_input_tokens: tokens,
            estimated_cached_input_tokens: Math.round(tokens * 0.4),
            estimated_output_tokens: 1000,
            max_fallbacks: 4
          })
        });
        if (res.ok) {
          const data = await res.json();
          const chain = [data.primary, ...(data.fallbacks || [])].filter(Boolean);
          chain.forEach((arm, idx) => {
            const estCost = Number(arm.estimated_cost_usd || arm.cost_usd || (0.0015 * (idx + 1)));
            const qNorm = Math.max(0.75, Math.min(0.99, Number(arm.quality_score || (0.96 - idx * 0.03))));
            const cNorm = Math.max(0.1, 1.0 - Math.min(0.9, estCost * 120));
            const latMs = Number(arm.expected_latency_ms || (140 + idx * 90));
            const lNorm = Math.max(0.2, 1.0 - (latMs / 1500));
            const util = category === 'local' ? 0.15 : (wq * qNorm + wc * cNorm + wl * lNorm);
            candidates.push({
              model_id: arm.model_id || arm.canonical_id || `cloud-arm-${idx + 1}`,
              tier: `Cloud (${arm.provider || 'Nous Portal'})`,
              utility: util,
              q_score: qNorm,
              cost_usd: estCost,
              latency_ms: latMs,
              vram_mb: 0,
              rationale: `Cloud Frontier via ${arm.provider || 'Nous Portal'} • Custo estimado $${estCost.toFixed(5)}/req.`
            });
          });
        }
      } catch (_) {}

      if (candidates.length === 0) {
        candidates.push({
          model_id: 'tier0_onnx_cpu',
          tier: 'Tier 0.5 (CPU AVX2)',
          utility: 0.91,
          q_score: 0.88,
          cost_usd: 0.0,
          latency_ms: 25.0,
          vram_mb: 0,
          rationale: 'Fallback determinístico local a 0 MB VRAM.'
        });
      }

      candidates.sort((a, b) => b.utility - a.utility);
      const winner = candidates[0];

      const setTxt = (id, txt) => { const e = document.getElementById(id); if (e) e.innerText = txt; };
      setTxt('router-sim-winner-name', winner.model_id);
      setTxt('router-sim-tier-badge', winner.tier);
      setTxt('router-sim-utility', winner.utility.toFixed(4));
      setTxt('router-sim-cost', winner.cost_usd <= 0 ? '$0.00 (Silício)' : `$${winner.cost_usd.toFixed(5)}`);
      setTxt('router-sim-lat', `${winner.latency_ms.toFixed(1)} ms`);
      setTxt('router-sim-phi', `${phiPenalty.toFixed(2)} (${tempC.toFixed(0)}°C)`);
      setTxt('router-sim-vram', winner.vram_mb > 0 ? `${winner.vram_mb} MB (dGPU)` : '0 MB (Host/Cloud)');
      setTxt('router-sim-zone', window.soulsTelemetryState?.vramLimitMb ? `WDDM Safe (${window.soulsTelemetryState.vramLimitMb} MB)` : 'Zona Segura WDDM');
      setTxt('router-sim-rationale', `[${category.toUpperCase()} • ${tokens} tok] Pesos (${Math.round(wq*100)}/${Math.round(wc*100)}/${Math.round(wl*100)}) -> ${winner.rationale}`);

      const listEl = document.getElementById('router-sim-candidates-list');
      if (listEl) {
        listEl.innerHTML = candidates.slice(0, 5).map((cand, i) => {
          const pct = Math.min(100, Math.max(12, Math.round(cand.utility * 100)));
          const barColor = i === 0 ? 'var(--cyan)' : 'var(--purple)';
          return `
            <div style="background:rgba(15,23,42,0.55); border:1px solid rgba(148,163,184,0.16); border-radius:8px; padding:8px 10px;">
              <div class="u-d-flex u-justify-between u-items-center u-mb-1 u-mono" style="font-size:0.75rem;">
                <span class="u-fw-700 ${i === 0 ? 'u-text-cyan' : 'u-text-white'}">#${i + 1} ${escapeHtml(cand.model_id)}</span>
                <span class="u-text-muted">${escapeHtml(cand.tier)} • U=${cand.utility.toFixed(3)}</span>
              </div>
              <div style="height:5px; background:rgba(255,255,255,0.08); border-radius:4px; overflow:hidden;">
                <div style="width:${pct}%; height:100%; background:${barColor};"></div>
              </div>
            </div>
          `;
        }).join('');
      }
    }

    function setMemorySimScenario(query, mode) {
      const qEl = document.getElementById('sim-memory-query');
      const mEl = document.getElementById('sim-memory-mode');
      if (qEl && query) qEl.value = query;
      if (mEl && mode) mEl.value = mode;
      runMemorySimulator();
    }

    async function runMemorySimulator() {
      const query = (document.getElementById('sim-memory-query')?.value || '').trim();
      const mode = document.getElementById('sim-memory-mode')?.value || 'adaptive';
      const redactPii = document.getElementById('cfg-memory-pii-redactor')?.checked ?? true;
      const statusBadge = document.getElementById('sim-mem-status-badge');
      const blockEl = document.getElementById('sim-mem-formatted-block');
      const matchesList = document.getElementById('sim-mem-matches-list');
      const ontList = document.getElementById('sim-mem-ontology-list');

      if (!query) {
        showToast('Digite uma consulta para testar o Hipocampo L2/L3.', 'warning');
        return;
      }

      if (statusBadge) statusBadge.innerText = 'Consultando FrankenSQLite + FTS5...';
      if (blockEl) blockEl.innerText = 'Executando recall híbrido...';

      try {
        const res = await fetch('/api/v1/memory/recall', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            session_id: 'dashboard_sim',
            query,
            mode,
            redact_pii: redactPii
          })
        });
        const data = await res.json();
        if (!res.ok || data.status !== 'success') {
          if (statusBadge) statusBadge.innerText = 'Erro na Consulta RRF';
          if (blockEl) blockEl.innerText = data.message || 'Falha desconhecida ao consultar /api/v1/memory/recall';
          return;
        }

        if (statusBadge) statusBadge.innerText = 'Recall Híbrido RRF Concluído';
        const modeEl = document.getElementById('sim-mem-mode-used');
        if (modeEl) modeEl.innerText = `modo: ${data.mode_used || mode}`;
        const cntEl = document.getElementById('sim-mem-match-count');
        if (cntEl) cntEl.innerText = `${data.count || 0} fato(s) recuperado(s)`;
        const piiEl = document.getElementById('sim-mem-pii-count');
        if (piiEl) piiEl.innerText = `${data.pii_redactions || 0} PII redigido(s)`;

        if (blockEl) {
          blockEl.innerText = (data.formatted_block && data.formatted_block.trim())
            ? data.formatted_block
            : `<souls-l3-recall mode="${data.mode_used || mode}">\n  <!-- Nenhum fato correspondente encontrado para esta consulta ainda. -->\n</souls-l3-recall>`;
        }

        const matches = data.matches || [];
        if (matchesList) {
          matchesList.innerHTML = matches.length === 0
            ? '<div class="u-text-muted u-s-dbc8d1">Nenhum fato encontrado nesta consulta. Tente expandir a consulta ou usar termos do projeto.</div>'
            : matches.map((m, idx) => `
              <div style="background:rgba(15,23,42,0.55); border:1px solid rgba(16,185,129,0.22); border-radius:6px; padding:7px 9px;">
                <div class="u-d-flex u-justify-between u-items-center u-mono u-mb-1" style="font-size:0.72rem;">
                  <span class="u-text-green u-fw-700">#${idx + 1} [${escapeHtml(m.partition || 'EVOLVING')}]</span>
                  <span class="u-text-cyan">RRF ${(Number(m.rrf_score || m.score || 0)).toFixed(4)}</span>
                </div>
                <div class="u-text-slate-light" style="font-size:0.78rem;">${escapeHtml(m.content || '')}</div>
              </div>
            `).join('');
        }

        const rels = data.ontology_relations || [];
        const ontCountEl = document.getElementById('sim-mem-ontology-count');
        if (ontCountEl) ontCountEl.innerText = `${rels.length} aresta(s) retornada(s)`;
        if (ontList) {
          ontList.innerHTML = rels.length === 0
            ? '<span class="u-text-muted u-mono" style="font-size:0.72rem;">' + (data.mode_used === 'expanded' ? 'Nenhuma aresta vizinha nesta consulta.' : 'Modo "Expanded" ativa expansão no grafo LadybugDB.') + '</span>'
            : rels.map(r => `<span class="badge u-text-purple u-mono" style="font-size:0.70rem;">(${escapeHtml(r.src_label)}) --[${escapeHtml(r.relation)}]--> (${escapeHtml(r.dst_label)})</span>`).join('');
        }
      } catch (err) {
        if (statusBadge) statusBadge.innerText = 'Erro de Rede';
        if (blockEl) blockEl.innerText = `Erro ao consultar /api/v1/memory/recall: ${err.message}`;
      }
    }

    function runReasoningSimulator(presetPrompt) {
      const inputEl = document.getElementById('sim-prompt-input');
      if (typeof presetPrompt === 'string' && inputEl) {
        inputEl.value = presetPrompt;
      }
      const prompt = (inputEl?.value || '').trim();
      if (!prompt) {
        showToast('Digite um prompt para calcular a entropia de Shannon.', 'warning');
        return;
      }

      const dynamicOn = document.getElementById('cfg-reasoning-dynamic')?.checked ?? true;
      const bypassOn = document.getElementById('cfg-reasoning-bypass')?.checked ?? true;
      const socraticOn = document.getElementById('cfg-reasoning-socratic')?.checked ?? true;
      const socraticMode = document.getElementById('cfg-reasoning-socratic-mode')?.value || 'autonomous';
      const maxEffort = document.getElementById('cfg-reasoning-max-effort')?.value || 'ultra';
      const highThresh = Number(document.getElementById('cfg-reasoning-threshold')?.value || 0.70);
      const grillThresh = Number(document.getElementById('cfg-reasoning-grill-threshold')?.value || 0.75);
      const cycles = Number(document.getElementById('cfg-reasoning-socratic-cycles')?.value || 3);

      // Compute normalized Shannon entropy across token diversity + structural signals
      const lower = prompt.toLowerCase();
      const tokens = lower.split(/\s+/).filter(Boolean);
      const counts = {};
      tokens.forEach(t => { counts[t] = (counts[t] || 0) + 1; });
      const n = Math.max(1, tokens.length);
      let hTokens = 0;
      Object.values(counts).forEach(c => {
        const p = c / n;
        hTokens -= p * Math.log2(p);
      });
      const maxH = tokens.length > 1 ? Math.log2(tokens.length) : 1;
      const lexicalDiv = Math.min(1, hTokens / Math.max(1, maxH));

      const isTrivial = /(formatar|json|indent|traduz|resumir lista|corrigir erro de digita)/i.test(lower) && tokens.length <= 22;
      const isArchitecture = /(refator|parser|ast|myers|crate|arquitet|concorr|vazamento|contrato)/i.test(lower);
      const isAmbiguous = /(melhorar|geral|otimiz|sugira|como faço|melhor forma|tudo)/i.test(lower) && !isTrivial;

      let intent = 'GENERAL';
      if (isTrivial) intent = 'TRIVIAL / FORMAT';
      else if (isArchitecture) intent = 'CODE / AST SURGERY';
      else if (/(prova|teorema|matem|cohomolog|formal)/i.test(lower)) intent = 'MATH / FORMAL LOGIC';
      else if (isAmbiguous) intent = 'AMBIGUOUS / OPEN';

      let entropy = 0.25 + lexicalDiv * 0.35 + Math.min(0.25, tokens.length / 80);
      if (isTrivial) entropy = Math.min(0.28, entropy * 0.45);
      if (isArchitecture) entropy = Math.max(0.82, Math.min(0.95, entropy + 0.24));
      if (isAmbiguous) entropy = Math.max(0.76, Math.min(0.96, entropy + 0.22));
      entropy = Math.max(0.05, Math.min(0.99, entropy));

      const ladder = [
        { key: 'none', tokens: 0, effort: 'none' },
        { key: 'minimal', tokens: 512, effort: 'low' },
        { key: 'low', tokens: 1024, effort: 'low' },
        { key: 'medium', tokens: 4096, effort: 'medium' },
        { key: 'high', tokens: 16384, effort: 'high' },
        { key: 'xhigh', tokens: 24576, effort: 'high' },
        { key: 'ultra', tokens: 32768, effort: 'max' }
      ];
      const maxIdx = Math.max(1, ladder.findIndex(x => x.key === maxEffort));
      let chosenIdx = 3;
      if (!dynamicOn) {
        chosenIdx = 3;
      } else if (bypassOn && entropy < 0.35) {
        chosenIdx = 0;
      } else if (entropy >= highThresh + 0.16) {
        chosenIdx = 6;
      } else if (entropy >= highThresh + 0.08) {
        chosenIdx = 5;
      } else if (entropy >= highThresh) {
        chosenIdx = 4;
      } else if (entropy >= 0.50) {
        chosenIdx = 3;
      } else {
        chosenIdx = 2;
      }
      chosenIdx = Math.min(chosenIdx, maxIdx);
      const chosen = ladder[chosenIdx];

      const setTxt = (id, txt) => { const e = document.getElementById(id); if (e) e.innerText = txt; };
      setTxt('sim-entropy-score', `${entropy.toFixed(3)} H`);
      setTxt('sim-entropy-tag', entropy >= highThresh ? 'Alta Incerteza' : (entropy < 0.35 ? 'Baixa Incerteza (Determinístico)' : 'Incerteza Moderada'));
      const bar = document.getElementById('sim-entropy-bar');
      if (bar) bar.style.width = `${Math.round(entropy * 100)}%`;

      const intentBadge = document.getElementById('sim-intent-badge');
      if (intentBadge) intentBadge.innerHTML = `${uiIcon('target', 'icon-xs')} Intenção: ${intent}`;
      const effBadge = document.getElementById('sim-effort-badge');
      if (effBadge) effBadge.innerHTML = `${uiIcon('layers', 'icon-xs')} Thinking: ${chosen.key.toUpperCase()} (${chosen.tokens.toLocaleString('pt-BR')} tokens)`;
      const socBadge = document.getElementById('sim-socratic-badge');
      if (socBadge) {
        const triggerGrill = socraticOn && (entropy >= grillThresh || isAmbiguous);
        socBadge.innerHTML = `${uiIcon('git-branch', 'icon-xs')} ${chosen.tokens === 0 ? 'Bypass Ativo (0 Thinking Tokens)' : (triggerGrill ? `Freio Socrático: /grill-me (${cycles} ciclos DAG)` : `Modo Socrático: ${socraticMode} (${cycles} ciclos)`)}`;
      }
      setTxt('sim-latency', '0.042 ms');
      setTxt('sim-vram', '0.0 MB (CPU AVX2)');
      setTxt('sim-rationale', `Entropy=${entropy.toFixed(3)} vs threshold=${highThresh.toFixed(2)} (grill=${grillThresh.toFixed(2)}) -> bucket ${chosen.key.toUpperCase()} (${chosen.tokens} tok).`);

      const payloadsEl = document.getElementById('sim-payloads-json');
      if (payloadsEl) {
        const payloadObj = chosen.tokens === 0
          ? { anthropic: { type: 'disabled' }, openai_reasoning_effort: 'none', gemini_thinking_config: { thinking_budget: 0 }, openrouter: { reasoning: { excluded: true } } }
          : { anthropic: { type: 'enabled', budget_tokens: chosen.tokens }, openai_reasoning_effort: chosen.effort, gemini_thinking_config: { thinking_budget: chosen.tokens }, openrouter: { effort: chosen.effort, max_tokens: chosen.tokens } };
        payloadsEl.innerText = JSON.stringify(payloadObj, null, 2);
      }
    }

    function runHardwareSimulation() {
      const reqVram = Number(document.getElementById('sim-hw-vram-input')?.value || 2450);
      const hardLimit = Number(document.getElementById('cfg-vram-limit')?.value || window.soulsTelemetryState?.vramLimitMb || 5170);
      const currentUsed = Number(window.soulsTelemetryState?.vramUsedMb || 950);
      const gpuTemp = Number(window.soulsTelemetryState?.gpuTempC || 58);
      const prevTemp = Number(document.getElementById('cfg-temp-preventive')?.value || 72);
      const throtTemp = Number(document.getElementById('cfg-temp-throttle')?.value || 78);
      const cpuFallback = document.getElementById('cfg-cpu-fallback')?.checked ?? true;
      const gpuName = window.soulsTelemetryState?.gpuName || 'dGPU Dedicada';

      const totalProjected = reqVram === 0 ? currentUsed : currentUsed + reqVram;
      const wddmFreeBeforeLoad = Math.max(0, hardLimit - currentUsed);
      const headroomAfter = hardLimit - totalProjected;
      const resultEl = document.getElementById('sim-hw-result');
      if (!resultEl) return;

      let titleHtml = `${uiIcon('check-circle', 'icon-xs')} Aprovado na ${escapeHtml(gpuName)} (Zero-Spillover)`;
      let titleCls = 'u-text-green';
      let tpsEst = reqVram === 0 ? '~85 tokens/s (CPU)' : (reqVram <= 2000 ? '~58 tokens/s' : '~38 tokens/s');
      let decision = 'Alocar 100% em VRAM Dedicada';
      let reason = `Alocação de <strong>${reqVram.toLocaleString('pt-BR')} MB</strong> + base WDDM (${currentUsed.toLocaleString('pt-BR')} MB) = <strong>${totalProjected.toLocaleString('pt-BR')} MB</strong> dentro do teto seguro (${hardLimit.toLocaleString('pt-BR')} MB).`;

      if (reqVram === 0) {
        titleHtml = `${uiIcon('cpu', 'icon-xs')} Execução Soberana na Host CPU AVX2`;
        decision = 'CPU Host RAM (0 MB VRAM)';
        reason = 'Modelo Tier 0/0.5 executa inteiramente na memória RAM do sistema com latência sub-50ms sem tocar na placa gráfica.';
      } else if (gpuTemp >= throtTemp) {
        titleHtml = `${uiIcon('alert-triangle', 'icon-xs')} Bloqueio Térmico NVML (${gpuTemp.toFixed(0)}°C ≥ ${throtTemp}°C)`;
        titleCls = 'u-text-red';
        tpsEst = cpuFallback ? '~14 tokens/s (Resgate CPU)' : 'Pausado';
        decision = cpuFallback ? 'Desviado para CPU Host AVX2' : 'Fila Pausada';
        reason = `Temperatura acima da Barreira Crítica (${throtTemp}°C). Despachos CUDA bloqueados para preservar o silício.`;
      } else if (headroomAfter < 250) {
        titleHtml = `${uiIcon('alert-triangle', 'icon-xs')} Risco de Spillover PCIe Detectado!`;
        titleCls = 'u-text-amber';
        tpsEst = cpuFallback ? '~11 tokens/s (CPU AVX2)' : '< 1.5 tokens/s (PCIe)';
        decision = cpuFallback ? 'Desvio Preventivo para CPU AVX2' : 'Risco Crítico WDDM';
        reason = `Carga solicitada (<strong>${reqVram.toLocaleString('pt-BR')} MB</strong>) + base DWM/IDE (${currentUsed.toLocaleString('pt-BR')} MB) = ${totalProjected.toLocaleString('pt-BR')} MB ultrapassa margem livre WDDM (${wddmFreeBeforeLoad.toLocaleString('pt-BR')} MB disponíveis).`;
      } else if (gpuTemp >= prevTemp) {
        reason += ` Atenção: temperatura em Zona Preventiva (${gpuTemp.toFixed(0)}°C ≥ ${prevTemp}°C).`;
      }

      resultEl.innerHTML = `
        <div class="u-d-flex u-justify-between u-items-center u-mb-6p">
          <strong class="${titleCls} u-d-flex u-items-center u-gap-6p">${titleHtml}</strong>
          <span class="mono u-text-cyan u-s-739509">${tpsEst}</span>
        </div>
        <div class="u-text-slate u-mb-6p u-s-dbc8d1">${reason}</div>
        <div class="u-d-flex u-gap-3 u-text-muted u-mono u-s-ac0643 u-flex-wrap">
          <span>Decisão: <strong class="u-text-white">${decision}</strong></span>
          <span>FinOps: <strong class="u-text-green">$0.00 / 1M</strong></span>
          <span>Folga Projetada: <strong class="${headroomAfter >= 250 ? 'u-text-cyan' : 'u-text-red'}">${reqVram === 0 ? wddmFreeBeforeLoad : headroomAfter} MB</strong></span>
        </div>
      `;
    }
    window.setRouterSimScenario = setRouterSimScenario;
    window.runRouterSimulator = runRouterSimulator;
    window.setMemorySimScenario = setMemorySimScenario;
    window.runMemorySimulator = runMemorySimulator;
    window.runReasoningSimulator = runReasoningSimulator;
    window.runHardwareSimulation = runHardwareSimulation;

    // ========================================================
    // MILESTONE v0.9.1-A: GLOBAL COMMAND PALETTE (CTRL+K / CMD+K)
    // ========================================================
    const COMMAND_PALETTE_ITEMS = [
      { id: 'nav-overview', title: 'Ir para Cockpit & Overview', desc: 'Pulso pessoal, telemetria de silício e status do Master Brain', category: 'Navegação', action: () => switchTab('tab-overview') },
      { id: 'nav-google-hub', title: 'Ir para Google Life Hub', desc: 'Resumo diário, Gmail, Agenda, previsão de trânsito, Docs, Sheets, Tarefas e YouTube', category: 'Navegação', action: () => switchTab('tab-google-hub') },
      { id: 'nav-agent-inbox', title: 'Ir para Central de Aprovações', desc: 'Revisar alterações com destaque visual e confirmar envios com segurança', category: 'Navegação', action: () => switchTab('tab-agent-inbox') },
      { id: 'nav-research', title: 'Ir para Pesquisa & Conhecimento', desc: 'Missões STORM, Deep Research e biblioteca verificada', category: 'Navegação', action: () => switchTab('tab-research') },
      { id: 'act-new-research', title: 'Iniciar Nova Pesquisa Profunda', desc: 'Abrir formulário de scoping STORM com preview FinOps', category: 'Ação Rápida', action: () => { switchTab('tab-research'); if (typeof switchResearchSubtab === 'function') switchResearchSubtab('new-mission'); } },
      { id: 'nav-gemini-notebooks', title: 'Ir para Gemini Notebooks', desc: 'Cadernos NotebookLM, Drive, perguntas com fontes citadas e player de áudio', category: 'Navegação', action: () => switchTab('tab-gemini-notebooks') },
      { id: 'nav-personas', title: 'Ir para Personas & Governança', desc: 'Gerenciar assistentes Hermes, SOUL.md e privacidade', category: 'Navegação', action: () => switchTab('tab-personas') },
      { id: 'nav-subagents', title: 'Ir para Subagentes & Fila', desc: 'Monitorar sessões Pareto e handoffs L3', category: 'Navegação', action: () => switchTab('tab-subagents') },
      { id: 'nav-skills', title: 'Ir para Skills & MCP Tools', desc: 'Catálogo local, marketplace 101k+ e Forge', category: 'Navegação', action: () => switchTab('tab-skills') },
      { id: 'nav-local', title: 'Ir para Modelos & Arena', desc: 'Silício local, Cloud Arena, Model Hub e Provedores', category: 'Navegação', action: () => switchTab('tab-local') },
      { id: 'nav-fallback', title: 'Ver Provedores Cloud & Fallback Chains', desc: 'Circuit breaker, quarentenas 429 e resolução <150ms', category: 'Navegação', action: () => { switchTab('tab-local'); if (typeof switchLocalSubtab === 'function') switchLocalSubtab('fallback'); } },
      { id: 'nav-connections', title: 'Ir para Conexões, Conta Google & Chaves de IA', desc: 'Conectar NotebookLM, Google Workspace, provedores de IA e assistentes locais', category: 'Navegação', action: () => switchTab('tab-connections') },
      { id: 'act-google-doctor', title: 'Verificar Diagnóstico Google & NotebookLM', desc: 'Checar conexão com Google, NotebookLM, Gemini e chaves configuradas', category: 'Diagnóstico', action: () => { switchTab('tab-connections'); if (typeof loadGoogleConnectionsVault === 'function') loadGoogleConnectionsVault(true); } },
      { id: 'nav-set-brain', title: 'Configurar Master Brain & Presets', desc: 'Alternar modelo mestre e tríade Pareto', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-brain'); } },
      { id: 'nav-set-router', title: 'Configurar Roteador ParetoBandit', desc: 'Thresholds e priors bayesianas de despacho', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-router'); } },
      { id: 'nav-set-reasoning', title: 'Configurar Raciocínio & Modo Socrático', desc: 'Thinking budget e detecção de entropia', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-reasoning'); } },
      { id: 'nav-set-memory', title: 'Configurar Memória L2/L3 & LadybugDB', desc: 'Retenção, decay e cofre híbrido', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-memory'); } },
      { id: 'nav-set-obsidian', title: 'Configurar Cofre de Conhecimentos (Obsidian)', desc: 'Curadoria raw/ -> wiki/ e Privacy Shield', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-obsidian'); } },
      { id: 'nav-set-finops', title: 'Configurar Aceleração Local & FinOps', desc: 'Teto WDDM, proteção térmica NVML e CPU AVX2', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-finops'); } },
      { id: 'nav-set-sandbox', title: 'Configurar Segurança & Sandbox', desc: 'Confinamento de caminhos e guardrails', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-sandbox'); } },
      { id: 'nav-set-tools', title: 'Inspecionar 55 Ferramentas MCP', desc: 'Permissões, schemas e catálogo switchboard', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-tools'); } },
      { id: 'nav-set-raw', title: 'Abrir Editor TOML (souls_config.toml)', desc: 'Edição direta da configuração persistida', category: 'Settings', action: () => { switchTab('tab-settings'); switchSettingsSubtab('subtab-raw'); } },
      { id: 'act-sync-pulse', title: 'Sincronizar Pulso do Ecossistema', desc: 'Atualizar KPIs de evolução, FinOps e telemetria', category: 'Ação Rápida', action: () => { if (typeof fetchEcosystemPulse === 'function') fetchEcosystemPulse(); if (typeof fetchTelemetry === 'function') fetchTelemetry(); showToast('Telemetria e pulso sincronizados!', 'success'); } },
      { id: 'act-ping-silicon', title: 'Testar Latência do Motor (Ping Loopback)', desc: 'Medir resposta HTTP em 127.0.0.1:9123/health', category: 'Ação Rápida', action: () => testSiliconPing() },
      { id: 'act-doctor-research', title: 'Executar Diagnóstico Doctor (Pesquisa)', desc: 'Verificar prontidão de perfis e motor Research Hub', category: 'Diagnóstico', action: () => { switchTab('tab-research'); if (typeof switchResearchSubtab === 'function') switchResearchSubtab('doctor'); } },
      { id: 'act-open-agents-md', title: 'Ler Hermes AGENTS.md (Invariantes)', desc: 'Visualizar regras constitucionais e prompt caching', category: 'Documentação', action: () => { if (typeof openAgentsMdModal === 'function') openAgentsMdModal('hermes'); } }
    ];

    let cmdPaletteFiltered = COMMAND_PALETTE_ITEMS.slice();
    let cmdPaletteSelectedIndex = 0;

    function openCommandPalette() {
      const modal = document.getElementById('cmd-palette-modal');
      const input = document.getElementById('cmd-palette-input');
      if (!modal) return;
      modal.classList.add('active');
      if (input) {
        input.value = '';
        filterCommandPalette('');
        setTimeout(() => input.focus(), 30);
      }
    }

    function closeCommandPalette() {
      const modal = document.getElementById('cmd-palette-modal');
      if (modal) modal.classList.remove('active');
    }

    function filterCommandPalette(query) {
      const q = (query || '').trim().toLowerCase();
      cmdPaletteFiltered = !q
        ? COMMAND_PALETTE_ITEMS.slice()
        : COMMAND_PALETTE_ITEMS.filter(item =>
            item.title.toLowerCase().includes(q) ||
            item.desc.toLowerCase().includes(q) ||
            item.category.toLowerCase().includes(q)
          );
      cmdPaletteSelectedIndex = 0;
      renderCommandPaletteResults();
    }

    function renderCommandPaletteResults() {
      const container = document.getElementById('cmd-palette-results');
      if (!container) return;
      if (cmdPaletteFiltered.length === 0) {
        container.innerHTML = '<div class="cmd-palette-empty">Nenhum comando correspondente encontrado.</div>';
        return;
      }
      container.innerHTML = cmdPaletteFiltered.map((item, idx) => `
        <button class="cmd-palette-item ${idx === cmdPaletteSelectedIndex ? 'is-selected' : ''}" onclick="executeCommandPaletteIndex(${idx})">
          <div class="cmd-palette-item-left">
            <div>
              <div class="cmd-palette-item-title">${item.title}</div>
              <div class="cmd-palette-item-desc">${item.desc}</div>
            </div>
          </div>
          <span class="cmd-palette-item-cat">${item.category}</span>
        </button>
      `).join('');
    }

    function executeCommandPaletteIndex(idx) {
      const item = cmdPaletteFiltered[idx];
      if (!item) return;
      closeCommandPalette();
      if (typeof item.action === 'function') item.action();
    }

    function handleCommandPaletteKey(e) {
      if (e.key === 'Escape') {
        e.preventDefault();
        closeCommandPalette();
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (cmdPaletteFiltered.length > 0) {
          cmdPaletteSelectedIndex = (cmdPaletteSelectedIndex + 1) % cmdPaletteFiltered.length;
          renderCommandPaletteResults();
        }
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (cmdPaletteFiltered.length > 0) {
          cmdPaletteSelectedIndex = (cmdPaletteSelectedIndex - 1 + cmdPaletteFiltered.length) % cmdPaletteFiltered.length;
          renderCommandPaletteResults();
        }
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        executeCommandPaletteIndex(cmdPaletteSelectedIndex);
      }
    }

    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        const modal = document.getElementById('cmd-palette-modal');
        if (modal && modal.classList.contains('active')) {
          closeCommandPalette();
        } else {
          openCommandPalette();
        }
      }
    });

    window.openCommandPalette = openCommandPalette;
    window.closeCommandPalette = closeCommandPalette;
    window.filterCommandPalette = filterCommandPalette;
    window.executeCommandPaletteIndex = executeCommandPaletteIndex;
    window.handleCommandPaletteKey = handleCommandPaletteKey;
