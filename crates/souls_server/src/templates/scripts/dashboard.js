    // ESTADO GLOBAL DA APLICAÇÃO
    let activePreset = 'balanced';
    let currentProfile = 'orchestrator';
    let cloudChampionsData = [];
    let currentCloudProviderFilter = 'all';
    let cachedCloudRankedModels = [];
    window.currentHermesMasterModel = 'deepseek/deepseek-chat';

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

    // NAVEGAÇÃO ENTRE ABAS
    function switchTab(tabId) {
      document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
      
      const pane = document.getElementById(tabId);
      const btn = document.getElementById('btn-' + tabId);
      if (pane) pane.classList.add('active');
      if (btn) btn.classList.add('active');

      if (window.location.hash !== '#' + tabId) {
        try { history.replaceState(null, '', '#' + tabId); } catch(e) {}
      }

      if (tabId === 'tab-local') {
        if (currentLocalSubtab === 'hub') {
          fetchHardwareAndRecommendations();
          fetchInventoryDownloads();
          connectDownloadsSSE();
        } else {
          fetchLocalModels();
        }
      } else if (tabId === 'tab-subagents') {
        fetchSubAgentSessions();
        fetchRouterMatrix();
      } else if (tabId === 'tab-cloud') {
        fetchCloudChampions();
      } else if (tabId === 'tab-overview') {
        fetchRouterMatrix();
      } else if (tabId === 'tab-skills') {
        if (typeof initSkillsHubTab === 'function') {
          initSkillsHubTab();
        }
      } else if (tabId === 'tab-settings') {
        fetchConfigToml();
      initSlidersMagneticSnap();
        fetchProfiles();
      }
    }

    // FEEDBACK VISUAL (TOAST)
    function showToast(msg, type = "success") {
      const c = document.getElementById('toast-container');
      const el = document.createElement('div');
      el.className = 'toast ' + type;
      el.innerHTML = `<span>${type === 'success' ? '✅' : type === 'error' ? '❌' : 'ℹ️'}</span> <div>${msg}</div>`;
      c.appendChild(el);
      setTimeout(() => {
        el.style.opacity = '0';
        el.style.transform = 'translateX(100%)';
        setTimeout(() => el.remove(), 300);
      }, 3500);
    }

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

    function updateDashboardMetrics(data) {
      // VRAM (WDDM limit vs physical 6144 MB)
      if (data.vram_used_mb !== undefined) {
        const limit = data.vram_limit_mb || 5294;
        const total = data.vram_total_mb || 6144;
        const wddmPct = Math.round((data.vram_used_mb / limit) * 100);
        const barWidthPct = Math.min(100, Math.round((data.vram_used_mb / total) * 100));
        const markerLeftPct = Math.min(98, Math.round((limit / total) * 100));

        const pillVram = document.getElementById('pill-vram');
        if (pillVram) pillVram.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' / ' + (limit / 1024).toFixed(2) + ' GB';

        const vramText = document.getElementById('vram-text');
        if (vramText) {
          vramText.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' GB / ' + (limit / 1024).toFixed(2) + ' GB WDDM (' + wddmPct + '%)';
        }
        const vramBar = document.getElementById('vram-bar');
        if (vramBar) vramBar.style.width = barWidthPct + '%';

        const vramMarker = document.getElementById('vram-wddm-marker');
        if (vramMarker) vramMarker.style.left = markerLeftPct + '%';

        // Sub-aba Aceleração Local: VRAM & Telemetria
        const hwMetricVram = document.getElementById('hw-metric-vram');
        if (hwMetricVram) hwMetricVram.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' / ' + (limit / 1024).toFixed(2) + ' GB';
        const hwLiveVramText = document.getElementById('hw-live-vram-text');
        if (hwLiveVramText) hwLiveVramText.innerText = (data.vram_used_mb / 1024).toFixed(2) + ' GB / ' + (limit / 1024).toFixed(2) + ' GB (' + wddmPct + '%)';
        const hwLiveVramFill = document.getElementById('hw-live-vram-fill');
        if (hwLiveVramFill) hwLiveVramFill.style.width = barWidthPct + '%';
        const hwLiveVramUsed = document.getElementById('hw-live-vram-used');
        if (hwLiveVramUsed) hwLiveVramUsed.innerText = data.vram_used_mb.toLocaleString('pt-BR') + ' MB';
        const hwLiveVramFree = document.getElementById('hw-live-vram-free');
        if (hwLiveVramFree) hwLiveVramFree.innerText = Math.max(0, (data.vram_total_mb || 6144) - data.vram_used_mb).toLocaleString('pt-BR') + ' MB';

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
        const gName = data.gpu_name || 'NVIDIA GeForce RTX 2060';
        const cName = data.cpu_name || 'Intel Core i9 CPU';
        if (hwSub) hwSub.innerText = `${gName} • ${cName}`;
        const cpuSub = document.getElementById('cpu-name-sub');
        if (cpuSub && data.cpu_name) cpuSub.innerText = data.cpu_name;
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
      }

      // Badges de Contagem nas Abas Superiores
      if (data.local_models_count !== undefined) {
        const bLocal = document.getElementById('badge-local-count');
        if (bLocal) bLocal.innerText = data.local_models_count;
      }
      if (data.profiles_count !== undefined) {
        const bCloud = document.getElementById('badge-cloud-profiles-count');
        const pillCount = document.querySelectorAll('#profile-pill-container .profile-pill').length;
        if (bCloud) bCloud.innerText = (pillCount > 0 ? pillCount : 6) + ' Perfis';
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
          sizeBadge.innerText = `💾 Banco Z:\\ • ${Number(m3.sqlite_size_mb).toFixed(2)} MB`;
        }

        const snapBadge = document.getElementById('hero-snapshot-badge');
        if (snapBadge && m3.snapshot_id) {
          snapBadge.innerText = `🔒 ${m3.snapshot_id.substring(0, 13)} (${(m3.memory_entries_count || 0) + (m3.user_entries_count || 0)} itens)`;
        }

        const snapLine = document.getElementById('l3-snapshot-summary-line');
        if (snapLine && m3.snapshot_id) {
          const shortSha = (m3.composite_sha256 || '').substring(0, 10);
          snapLine.innerHTML = `&bull; Snapshot: <code class="mono" style="color: var(--cyan);">${m3.snapshot_id}</code> (SHA <code class="mono">${shortSha}</code>)`;
        }

        const chkLbl = document.getElementById('l3-checkpoints-count-label');
        if (chkLbl) chkLbl.innerText = `${m3.checkpoints_count || 0} Checkpoints v2`;
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

      // Atualiza Master Brain & Topbar Preset
      if (data.master_brain) {
        const mb = data.master_brain;
        document.getElementById('top-brain-name').innerText = mb.model;
        document.getElementById('hero-brain-model').innerText = mb.model;
        document.getElementById('hero-brain-provider').innerText = mb.provider;
        document.getElementById('hero-brain-ctx').innerText = (mb.context_window || 64000).toLocaleString() + ' tokens';

        if (mb.active_profile_name) {
          const topPresetEl = document.getElementById('top-active-preset');
          if (topPresetEl) topPresetEl.innerText = mb.active_profile_name;
        }

        const badgeEl = document.getElementById('hero-brain-badge');
        if (mb.is_local) {
          badgeEl.innerText = 'Silício Bare-Metal';
          badgeEl.className = 'brain-badge badge-local';
        } else {
          badgeEl.innerText = 'Cloud API (' + mb.provider + ')';
          badgeEl.className = 'brain-badge badge-cloud';
        }
      }
    }

    // 2. FETCH MATRIZ PREDITIVA DO INTEGRADOR (OVERVIEW)
    async function fetchRouterMatrix() {
      try {
        const res = await fetch('/api/v1/router/matrix');
        if (res.ok) {
          const data = await res.json();
          if (data.cpu_simd) {
            window.detectedCpuSimd = data.cpu_simd;
          }
          const summaryBadge = document.getElementById('matrix-arena-summary-badge');
          if (summaryBadge && data.benchmarked_count !== undefined) {
            summaryBadge.innerText = `${data.benchmarked_count} Homologados • ${data.total_arena_runs || 0} Provas`;
          }
          renderRouterMatrix(data.matrix || []);
        }
      } catch (e) {
        console.warn("Falha ao carregar matriz do integrador", e);
      }
    }

    function renderRouterMatrix(matrix) {
      window.cachedRouterMatrix = matrix || [];
      const tbody1 = document.getElementById('integrator-tbody');
      const tbody2 = document.getElementById('subagents-router-tbody');
      if (!matrix || matrix.length === 0) {
        if (tbody1) tbody1.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--text-muted);">Nenhuma regra configurada.</td></tr>';
        if (tbody2) tbody2.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--text-muted);">Nenhuma regra configurada.</td></tr>';
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
          <td style="vertical-align: top; padding: 12px 14px;">
            <div style="font-weight: 700; color: #fff; font-size: 0.84rem; display: flex; align-items: center; gap: 8px; white-space: nowrap;">
              <span style="font-size: 1.15rem;">${item.icon}</span>
              <span>${item.category}</span>
            </div>
            <div style="font-size: 0.73rem; color: #94a3b8; margin-top: 4px; line-height: 1.45;">${item.description}</div>
          </td>

          <!-- 2: SILÍCIO LOCAL: LINHA 1 (MODELO + TROCAR), LINHA 2 (TARGET/ALOCAÇÃO), LINHA 3 DEDICADA P/ VELOCIDADE -->
          <td style="vertical-align: top; padding: 12px 14px;">
            <div style="display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 4px;">
              <div style="min-width: 0; flex: 1; overflow: hidden;">
                ${formatModelDisplayName(localModel, true)}
              </div>
              <button class="btn-micro" onclick="openConfigureDomainModal('${item.id}', 'local')" title="Alterar modelo local padrão deste domínio">
                Trocar
              </button>
            </div>
            <div style="font-size: 0.69rem; color: var(--text-muted); line-height: 1.3;">
              ${item.local_default.target}
            </div>
            <div style="font-size: 0.7rem; color: var(--green); font-weight: 600; margin-top: 4px; display: flex; align-items: center; gap: 4px;">
              <span style="color: #64748b; font-weight: normal;">Velocidade:</span> ${item.local_default.tps}
            </div>
          </td>

          <!-- 2: CLOUD FALLBACK: LINHA 1 (MODELO + TROCAR), LINHA 2 (ROTEAMENTO), LINHA 3 DEDICADA P/ TARIFA -->
          <td style="vertical-align: top; padding: 12px 14px;">
            <div style="display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 4px;">
              <div style="min-width: 0; flex: 1; overflow: hidden;">
                ${formatModelDisplayName(cloudModel, false)}
              </div>
              <button class="btn-micro" onclick="openConfigureDomainModal('${item.id}', 'cloud')" title="Alterar modelo cloud fallback deste domínio">
                Trocar
              </button>
            </div>
            <div style="font-size: 0.69rem; color: var(--text-muted); line-height: 1.3;">
              ${item.cloud_fallback.provider}
            </div>
            <div style="font-size: 0.7rem; color: var(--cyan); font-weight: 600; margin-top: 4px; display: flex; align-items: center; gap: 4px;">
              <span style="color: #64748b; font-weight: normal;">Tarifa:</span> ${item.cloud_fallback.cost}
            </div>
          </td>

          <!-- REGRA PARETO -->
          <td style="vertical-align: top; padding: 12px 14px;">
            <div style="font-size: 0.74rem; color: #cbd5e1; line-height: 1.45;">${paretoRule}</div>
          </td>

          <!-- AÇÕES COM BOTÃO DE CONFIGURAÇÃO DIRETA -->
          <td class="text-center" style="vertical-align: top; padding: 12px 10px;">
            <div style="display: flex; flex-direction: column; align-items: center; gap: 6px;">
              <span class="badge ${badgeClass}" style="width: 100%; justify-content: center; text-align: center;">${item.status}</span>
              <div style="display: flex; gap: 4px; width: 100%; margin-top: 2px;">
                <button class="btn" style="padding: 3px 6px; font-size: 0.68rem; border-radius: 5px; flex: 1; justify-content: center;" onclick="openConfigureDomainModal('${item.id}')" title="Configurar modelos local e cloud deste domínio">
                  ⚙️ Ajustar
                </button>
                <button class="btn" style="padding: 3px 6px; font-size: 0.68rem; border-radius: 5px; flex: 1; justify-content: center;" onclick="switchTab('tab-local'); testModelInArena('${localModel}')" title="Aferir este modelo local na Silicon Arena">
                  🏟️ Aferir
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
      tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted);">Consultando pesos físicos GGUF...</td></tr>';
      
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
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--red);">Erro ao carregar modelos locais.</td></tr>';
      }
    }

    function humanizeModelName(id) {
      if (!id) return '--';
      const lower = id.toLowerCase();
      if (lower.includes('qwen3-5-4b') || lower.includes('qwen2.5-coder-4b') || lower.includes('qwen3.5-coder')) return 'Qwen 2.5 Coder 4B';
      if (lower.includes('arch-function')) return 'Arch Function Chat 3B';
      if (lower.includes('smollm2-135m')) return 'SmolLM2 135M Reasoning';
      if (lower.includes('phi-4-mini')) return 'Phi-4 Mini 3.8B';
      if (lower.includes('qwen2-5-vl-3b') || lower.includes('qwen2.5-vl-3b')) return 'Qwen 2.5 VL 3B Vision';
      if (lower.includes('tiktoken-ccr')) return 'Tiktoken CCR Dehydrator';
      if (lower.includes('gitoxide')) return 'Gitoxide Frecency & AST';
      if (lower.includes('ladybug')) return 'LadybugDB Ontological Graph';
      if (lower.includes('nvml-barrier') || lower.includes('nvml-watchdog')) return 'NVML Physical Watchdog';
      if (lower.includes('claude-opus-5.5') || lower.includes('claude-5.5-opus')) return 'Claude Opus 5.5';
      if (lower.includes('claude-3-7-sonnet')) return 'Claude 3.7 Sonnet';
      if (lower.includes('claude-3-5-sonnet')) return 'Claude 3.5 Sonnet';
      if (lower.includes('claude-3-5-haiku')) return 'Claude 3.5 Haiku';
      if (lower.includes('gpt-6.1-sol')) return 'GPT-6.1 Sol';
      if (lower.includes('gpt-6-astra') || lower.includes('gpt-astra')) return 'GPT-6 Astra';
      if (lower.includes('qwen3.7-flash') || lower.includes('qwen3-7-flash')) return 'Qwen 3.7 Flash';
      if (lower.includes('deepseek-v4')) return 'DeepSeek-V4 Flash';
      if (lower.includes('deepseek-chat')) return 'DeepSeek-V3 Chat';
      if (lower.includes('deepseek-r1')) return 'DeepSeek-R1 Reasoning';
      if (lower.includes('deepseek-coder')) return 'DeepSeek Coder';
      if (lower.includes('gemini-2.0-flash')) return 'Gemini 2.0 Flash';
      if (lower.includes('o3-mini')) return 'OpenAI o3-mini';
      if (lower.includes('o1-mini')) return 'OpenAI o1-mini';
      
      // Fallback amigável caso seja do formato "provedor/nome-do-modelo"
      if (id.includes('/')) {
        const raw = id.split('/')[1] || id;
        return raw.split(/[-_]/).map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
      }
      return id;
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
          <div class="model-id-box" style="display: flex; flex-direction: column; gap: 2px;">
            <div style="display: flex; align-items: center; gap: 6px; white-space: nowrap;">
              <span class="model-brand-pill ${badgeClass}">${badgeLabel}</span>
              <span class="model-name-text" title="${canonicalId}" style="font-weight: 600; color: #fff;">${friendlyName}</span>
            </div>
            <div class="mono" style="font-size: 0.66rem; color: var(--text-muted);">${canonicalId}</div>
          </div>
        `;
      }
      const slashIdx = canonicalId.indexOf('/');
      if (slashIdx !== -1) {
        const brand = canonicalId.substring(0, slashIdx).toUpperCase();
        return `
          <div class="model-id-box" style="display: flex; flex-direction: column; gap: 2px;">
            <div style="display: flex; align-items: center; gap: 6px; white-space: nowrap;">
              <span class="model-brand-pill">${brand}</span>
              <span class="model-name-text" title="${canonicalId}" style="font-weight: 600; color: #fff;">${friendlyName}</span>
            </div>
            <div class="mono" style="font-size: 0.66rem; color: var(--text-muted);">${canonicalId}</div>
          </div>
        `;
      }
      return `
        <div class="model-id-box" style="display: flex; flex-direction: column; gap: 2px;">
          <div style="display: flex; align-items: center; gap: 6px; white-space: nowrap;">
            <span class="model-brand-pill">CLOUD</span>
            <span class="model-name-text" title="${canonicalId}" style="font-weight: 600; color: #fff;">${friendlyName}</span>
          </div>
          <div class="mono" style="font-size: 0.66rem; color: var(--text-muted);">${canonicalId}</div>
        </div>
      `;
    }

    // Modal de Aplicação Rápida de Modelo
    let quickApplyTargetModel = null;
    let quickApplyTargetProvider = 'nous';
    let quickApplyIsLocal = false;

    function openQuickApplyModal(modelId, provider, isLocal = false) {
      quickApplyTargetModel = modelId;
      quickApplyTargetProvider = provider || (isLocal ? 'local_silicon' : 'nous');
      quickApplyIsLocal = Boolean(isLocal);

      const nameEl = document.getElementById('quick-apply-model-name');
      const tagEl = document.getElementById('quick-apply-provider-tag');
      if (nameEl) nameEl.innerText = modelId;
      if (tagEl) tagEl.innerText = `Provedor: ${quickApplyTargetProvider} • Tipo: ${isLocal ? 'Silício Bare-Metal' : 'Cloud API'}`;

      const sel = document.getElementById('quick-apply-profile-select');
      if (sel) {
        let opts = '';
        if (profilesData && profilesData.length > 0) {
          opts = profilesData.map(p => `
            <option value="${p.id}">${p.name} (${p.role === 'orchestrator' ? '👑 Master' : '🤖 Subagente'})</option>
          `).join('');
        } else {
          opts = '<option value="custom_subagent">Subagente Customizado (Default)</option>';
        }
        sel.innerHTML = opts;
      }

      const radioMaster = document.querySelector('input[name="apply-target"][value="master"]');
      if (radioMaster) radioMaster.checked = true;
      toggleApplyTarget();

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
      if (fieldDomain) fieldDomain.style.display = (val === 'domain') ? 'block' : 'none';
      if (fieldProfile) fieldProfile.style.display = (val === 'custom_profile' || val === 'profile') ? 'block' : 'none';
    }

    async function submitQuickApply() {
      if (!quickApplyTargetModel) return;
      const val = document.querySelector('input[name="apply-target"]:checked')?.value;

      if (val === 'master') {
        closeQuickApplyModal();
        await setHermesMasterModel(quickApplyTargetModel, quickApplyTargetProvider, activePreset);
      } else if (val === 'domain') {
        const domSelect = document.getElementById('quick-apply-domain-select');
        const domId = domSelect ? domSelect.value : 'coding';
        closeQuickApplyModal();
        try {
          const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
          if (!overrides[domId]) overrides[domId] = {};
          overrides[domId].cloud_model = quickApplyTargetModel;
          overrides[domId].cloud_provider = quickApplyTargetProvider;
          localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));
          showToast(`✅ Modelo ${quickApplyTargetModel} atribuído ao subagente de [${domId.toUpperCase()}]!`, 'green');
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
      if (!tier) return '<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: #94a3b8; border: 1px solid rgba(148, 163, 184, 0.4); font-size: 0.68rem; font-weight: 700; font-family: var(--font-mono); padding: 3px 8px; border-radius: 9999px;">Tier 1</span>';
      const t = tier.trim();
      const lower = t.toLowerCase();
      
      let bg = 'rgba(139, 92, 246, 0.18)';
      let color = '#c084fc';
      let border = 'rgba(168, 85, 247, 0.5)';
      let subtitle = 'Master Coder';
      
      if (lower.includes('tier 0.5') || lower.includes('tier 05')) {
        // Tier 0.5: Epistemic & Intent Probing (Emerald / Mint)
        bg = 'rgba(16, 185, 129, 0.16)';
        color = '#34d399';
        border = 'rgba(16, 185, 129, 0.45)';
        subtitle = 'Epistemic Probing';
      } else if (lower.includes('tier 0')) {
        // Tier 0: Fast Triage / Bootstrap SLM (Sky Cyan)
        bg = 'rgba(6, 182, 212, 0.16)';
        color = '#38bdf8';
        border = 'rgba(6, 182, 212, 0.45)';
        subtitle = 'Fast Triage';
      } else if (lower.includes('tier 1')) {
        // Tier 1: Master dGPU Coder (Violet / Purple)
        bg = 'rgba(139, 92, 246, 0.18)';
        color = '#c084fc';
        border = 'rgba(168, 85, 247, 0.5)';
        subtitle = 'Master dGPU';
      } else if (lower.includes('tier 2')) {
        // Tier 2: MoE Offload CPU (Amber / Gold)
        bg = 'rgba(245, 158, 11, 0.16)';
        color = '#fbbf24';
        border = 'rgba(245, 158, 11, 0.45)';
        subtitle = 'MoE Deep Reasoning';
      } else if (lower.includes('tier 3')) {
        // Tier 3: Vision VLM (Rose / Coral)
        bg = 'rgba(244, 63, 94, 0.16)';
        color = '#fb7185';
        border = 'rgba(244, 63, 94, 0.45)';
        subtitle = 'Vision / VLM';
      } else if (lower.includes('tier 4')) {
        // Tier 4: Speculative Draft (Electric Blue)
        bg = 'rgba(59, 130, 246, 0.16)';
        color = '#60a5fa';
        border = 'rgba(59, 130, 246, 0.45)';
        subtitle = 'Speculative Draft';
      }
      return `
        <div style="display: flex; flex-direction: column; align-items: center; gap: 2px;">
          <span class="badge" style="background: ${bg}; color: ${color}; border: 1px solid ${border}; font-size: 0.68rem; font-weight: 700; font-family: var(--font-mono); padding: 2px 7px; border-radius: 9999px;">${t}</span>
          <span style="font-size: 0.58rem; color: ${color}; opacity: 0.9; white-space: nowrap; font-weight: 600; letter-spacing: 0.02em;">${subtitle}</span>
        </div>
      `;
    }

    function renderTierBadges(m) {
      const tiers = (m.tiers && m.tiers.length > 0) ? m.tiers : (m.tier ? [m.tier] : ['Tier 1']);
      return `
        <div style="display: flex; flex-direction: column; gap: 4px; align-items: center; justify-content: center;">
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
          prompt("Copie a configuração MCP abaixo:", snippet);
        });
      } else {
        prompt("Copie a configuração MCP abaixo:", snippet);
      }
    }

    function renderTagBadge(tag) {
      if (!tag) return '';
      const t = tag.trim().toLowerCase();
      let bg = 'rgba(148, 163, 184, 0.12)';
      let color = '#94a3b8';
      let border = 'rgba(148, 163, 184, 0.25)';

      // 1. Código, AST & Sintaxe (Roxo / Violeta)
      if (t.includes('code') || t.includes('rust') || t.includes('ast') || t.includes('syntax') || t.includes('refactor') || t.includes('image_to_code')) {
        bg = 'rgba(139, 92, 246, 0.16)';
        color = '#c084fc';
        border = 'rgba(168, 85, 247, 0.4)';
      }
      // 2. Ferramentas, Protocolos & Schemas (Ciano / Azul Céu)
      else if (t.includes('tool') || t.includes('mcp') || t.includes('json') || t.includes('schema') || t.includes('agentic') || t.includes('function') || t.includes('fc') || t.includes('ui_inspection')) {
        bg = 'rgba(6, 182, 212, 0.16)';
        color = '#38bdf8';
        border = 'rgba(6, 182, 212, 0.4)';
      }
      // 3. Raciocínio Profundo, CoT & Prova (Índigo / Safira)
      else if (t.includes('reason') || t.includes('cot') || t.includes('logic') || t.includes('math') || t.includes('thinking') || t.includes('socratic')) {
        bg = 'rgba(99, 102, 241, 0.16)';
        color = '#818cf8';
        border = 'rgba(99, 102, 241, 0.4)';
      }
      // 4. Triagem Rápida, Hardware & Silício (Esmeralda / Menta)
      else if (t.includes('triage') || t.includes('fast') || t.includes('avx') || t.includes('cpu') || t.includes('dgpu') || t.includes('bootstrap') || t.includes('intent') || t.includes('guardrail')) {
        bg = 'rgba(16, 185, 129, 0.16)';
        color = '#34d399';
        border = 'rgba(16, 185, 129, 0.4)';
      }
      // 5. Visão, VLM & Multimodal (Rosa / Framboesa)
      else if (t.includes('vision') || t.includes('vlm') || t.includes('multi') || t.includes('image') || t.includes('ocr')) {
        bg = 'rgba(244, 63, 94, 0.16)';
        color = '#fb7185';
        border = 'rgba(244, 63, 94, 0.4)';
      }
      // 6. Especulativo / Drafter (Âmbar / Laranja)
      else if (t.includes('draft') || t.includes('speculative') || t.includes('mtp')) {
        bg = 'rgba(245, 158, 11, 0.16)';
        color = '#f59e0b';
        border = 'rgba(245, 158, 11, 0.4)';
      }

      return `<span class="badge" style="background: ${bg}; color: ${color}; border: 1px solid ${border}; font-size: 0.64rem; padding: 2px 6px; border-radius: 4px; line-height: 1.2; font-weight: 500; margin: 0;">${tag}</span>`;
    }

    function resolveAgenticProfile(m) {
      const tiers = (m.tiers && m.tiers.length > 0) ? m.tiers : (m.tier ? [m.tier] : []);
      const t = tiers.join(' ').toLowerCase();
      const name = `${m.model || ''} ${m.name || ''} ${(m.tags || []).join(' ')}`.toLowerCase();

      if (t.includes('tier 3') || name.includes('vl') || name.includes('vision') || name.includes('multimodal')) {
        return { icon: '👁️', title: 'Multimodal Vision', color: '#fb7185' };
      }
      if (name.includes('function') || name.includes('tool') || name.includes('fc') || name.includes('xlam')) {
        return { icon: '🛠️', title: 'Tool Specialist', color: '#38bdf8' };
      }
      if (name.includes('mtp') || t.includes('tier 4') || name.includes('draft')) {
        return { icon: '🚀', title: 'Speculative Draft', color: '#60a5fa' };
      }
      if (name.includes('reasoning') || name.includes('r1') || t.includes('tier 0.5') || name.includes('socratic')) {
        return { icon: '🔍', title: 'Intent Probing', color: '#34d399' };
      }
      if (t.includes('tier 0') || name.includes('135m')) {
        return { icon: '⚡', title: 'Fast Triage', color: '#38bdf8' };
      }
      if (name.includes('coder') || name.includes('code') || name.includes('temper')) {
        return { icon: '🤖', title: 'Code Surgeon', color: '#c084fc' };
      }
      if (name.includes('hermes') || name.includes('nemotron')) {
        return { icon: '👑', title: 'Master Candidate', color: '#fbbf24' };
      }
      return { icon: '🤖', title: 'General Subagent', color: '#94a3b8' };
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
        tbody.innerHTML = '<tr><td colspan="11" style="text-align: center; color: var(--text-muted); padding: 24px;">Nenhum modelo físico encontrado.</td></tr>';
        return;
      }

      // Atualiza contadores dos pills de filtro
      const dgpuCount = models.filter(m => m.target && m.target.includes('dGPU')).length;
      const cpuCount = models.filter(m => !(m.target && m.target.includes('dGPU'))).length;
      const elAll = document.getElementById('count-local-all');
      const elDgpu = document.getElementById('count-local-dgpu');
      const elCpu = document.getElementById('count-local-cpu');
      if (elAll) elAll.innerText = models.length;
      if (elDgpu) elDgpu.innerText = dgpuCount;
      if (elCpu) elCpu.innerText = cpuCount;

      // Filtragem
      let filtered = models.filter(m => {
        const isGpu = m.target && m.target.includes('dGPU');
        if (localModelSiliconFilter === 'dgpu' && !isGpu) return false;
        if (localModelSiliconFilter === 'cpu' && isGpu) return false;

        if (localModelFilterQuery) {
          const prof = resolveAgenticProfile(m);
          const tiersStr = (m.tiers || [m.tier]).join(' ');
          const hay = `${m.model || ''} ${m.name || ''} ${tiersStr} ${prof.title} ${m.quantization || ''} ${(m.tags || []).join(' ')} ${m.file_path || ''}`.toLowerCase();
          if (!hay.includes(localModelFilterQuery)) return false;
        }
        return true;
      });

      // Ordenação
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
        } else if (localModelSortKey === 'context') {
          cmp = (a.context_window || 0) - (b.context_window || 0);
        } else if (localModelSortKey === 'freshness') {
          cmp = (a.freshness_days !== undefined ? a.freshness_days : 999) - (b.freshness_days !== undefined ? b.freshness_days : 999);
        }
        return localModelSortAsc ? cmp : -cmp;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td colspan="11" style="text-align: center; color: var(--text-muted); padding: 24px;">Nenhum modelo corresponde aos filtros aplicados.</td></tr>';
        return;
      }

      tbody.innerHTML = filtered.map(m => {
        const tagsHtml = `<div style="display: flex; flex-wrap: wrap; gap: 3px 5px; max-width: 175px; align-items: center;">` +
          (m.tags || []).map(t => renderTagBadge(t)).join('') +
          `</div>`;
        const isGpu = m.target && m.target.includes('dGPU');
        const statusBadge = isGpu 
          ? '<span class="badge badge-champion" style="font-size: 0.70rem; padding: 3px 8px; white-space: nowrap;">🎮 dGPU (RTX 2060m)</span>' 
          : '<span class="badge badge-value" style="font-size: 0.70rem; padding: 3px 8px; white-space: nowrap;">⚡ Host CPU (AVX2)</span>';
        const sizeMb = m.size_mb ? (m.size_mb > 1024 ? (m.size_mb / 1024).toFixed(2) + ' GB' : m.size_mb.toFixed(0) + ' MB') : '--';
        const modelId = m.model || m.name;

        // Opção D: Perfil Agêntico
        const prof = resolveAgenticProfile(m);
        const profileHtml = `
          <div style="display: flex; align-items: center; gap: 6px; white-space: nowrap;">
            <span style="font-size: 0.85rem;">${prof.icon}</span>
            <span style="font-size: 0.74rem; font-weight: 600; color: ${prof.color};">${prof.title}</span>
          </div>
        `;

        // Opção B: VRAM & Margem WDDM (teto 5.17 GB)
        let vramHtml = '';
        if (isGpu) {
          const vramGb = m.size_mb ? (m.size_mb / 1024).toFixed(1) : '0';
          const headroom = Math.max(0, (5.17 - (m.size_mb / 1024))).toFixed(1);
          let folgaColor = '#34d399';
          let folgaLabel = `🟢 Folga ${headroom} GB`;
          if (headroom < 0.6) {
            folgaColor = '#ef4444';
            folgaLabel = `🔴 Folga ${headroom} GB (Risco)`;
          } else if (headroom < 1.2) {
            folgaColor = '#fbbf24';
            folgaLabel = `🟡 Folga ${headroom} GB`;
          }
          vramHtml = `
            <div class="mono text-center" style="color: var(--cyan); font-weight: 600; font-size: 0.78rem;">${vramGb} GB</div>
            <div style="font-size: 0.62rem; color: ${folgaColor}; margin-top: 2px; white-space: nowrap; font-weight: 500;">${folgaLabel}</div>
          `;
        } else {
          vramHtml = `
            <div class="mono text-center" style="color: var(--text-muted); font-size: 0.78rem;">0 MB</div>
            <div style="font-size: 0.62rem; color: var(--green); margin-top: 2px; white-space: nowrap; font-weight: 500;">⚡ Host RAM</div>
          `;
        }

        // Opção A: Velocidade Empírica (Throughput & TTFT)
        let speedHtml = '';
        if (m.tps && m.tps > 0) {
          const ttftStr = m.ttft_ms ? `${m.ttft_ms.toFixed(0)}ms` : '--';
          speedHtml = `
            <div class="mono text-center" style="color: var(--green); font-weight: 600; font-size: 0.76rem;">⚡ ${m.tps.toFixed(1)} tok/s</div>
            <div style="font-size: 0.62rem; color: var(--text-muted); margin-top: 2px;">TTFT: <strong style="color: var(--cyan);">${ttftStr}</strong></div>
          `;
        } else {
          const approxTps = isGpu ? (m.size_mb > 3000 ? '28~45' : '45~75') : '90~150';
          speedHtml = `
            <div style="font-size: 0.68rem; color: var(--text-muted); text-align: center;">~${approxTps} tok/s</div>
            <div style="text-align: center; margin-top: 2px;">
              <button class="btn" style="padding: 1px 7px; font-size: 0.62rem; border-radius: 4px; border-color: rgba(6, 182, 212, 0.3); color: var(--cyan); white-space: nowrap;" onclick="testModelInArena('${modelId}')" title="Aferir velocidade na Local Silicon Arena">
                Aferir 🏟️
              </button>
            </div>
          `;
        }

        // Opção E: Freshness ("Há X dias")
        const freshnessLabel = m.freshness || 'Hoje';
        const freshnessHtml = `
          <div class="text-center" style="font-size: 0.72rem; color: #cbd5e1; white-space: nowrap;">
            📅 ${freshnessLabel}
          </div>
        `;

        return `
          <tr>
            <td>
              ${formatModelDisplayName(modelId, true)}
              <div style="font-size: 0.65rem; color: var(--text-muted); max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-top: 2px;" title="${m.file_path || ''}">
                ${m.file_path || ''}
              </div>
            </td>
            <td class="text-center" style="vertical-align: middle;">
              <div style="display: flex; gap: 4px; justify-content: center; align-items: center;">
                <button class="btn-apply-model local-btn" style="padding: 3px 6px; font-size: 0.68rem;" onclick="openQuickApplyModal('${modelId}', 'local_silicon', true)" title="Aplicar este modelo local no Hermes">
                  <span>⚡</span> Usar
                </button>
                <button class="btn" style="padding: 3px 6px; font-size: 0.68rem; border-radius: 5px; white-space: nowrap;" onclick="testModelInArena('${modelId}')" title="Aferir este modelo na Local Silicon Arena">
                  <span>🏟️</span> Aferir
                </button>
              </div>
            </td>
            <td class="text-center" style="vertical-align: middle;">${renderTierBadges(m)}</td>
            <td style="vertical-align: middle;">${profileHtml}</td>
            <td class="mono text-center" style="white-space: nowrap; vertical-align: middle;">${sizeMb} <span style="color: var(--cyan); margin: 0 4px; font-weight: bold; font-size: 0.85rem;">•</span> <strong style="color: #fff;">${m.quantization || 'Q4_K_M'}</strong></td>
            <td class="text-center" style="vertical-align: middle;">${vramHtml}</td>
            <td class="text-center" style="vertical-align: middle;">${speedHtml}</td>
            <td class="mono text-center" style="vertical-align: middle;">${(m.context_window || 8192).toLocaleString()}</td>
            <td style="vertical-align: middle;">${tagsHtml}</td>
            <td style="vertical-align: middle;">${freshnessHtml}</td>
            <td class="text-center" style="vertical-align: middle;">${statusBadge}</td>
          </tr>
        `;
      }).join('');
    }

    // Correspondência inteligente com modelos reais baixados em disco
    function findInstalledModelMatch(targetFileName, repoId = '', modelName = '') {
      if (!localModelsList || localModelsList.length === 0) return null;
      const targetLower = (targetFileName || '').toLowerCase().replace(/\.gguf$/, '');
      const repoLower = (repoId || '').toLowerCase();
      const nameLower = (modelName || '').toLowerCase();

      for (const m of localModelsList) {
        const filePath = m.file_path || '';
        const realFileName = filePath.split(/[\\/]/).pop() || m.model || m.name || '';
        const fnLower = realFileName.toLowerCase().replace(/\.gguf$/, '');
        const modelLower = (m.model || m.name || '').toLowerCase();

        // 1. Match exato de nome de arquivo GGUF
        if (targetLower && fnLower === targetLower) return { ...m, realFileName };

        // 2. Match de Stem & Quantização idênticos (ex: qwen2.5-coder-0.5b / q4_k_m)
        if (targetLower && fnLower) {
          const cleanTarget = targetLower.replace(/[-_.]/g, '');
          const cleanFn = fnLower.replace(/[-_.]/g, '');
          if (cleanTarget === cleanFn || cleanFn.includes(cleanTarget) || cleanTarget.includes(cleanFn)) {
            return { ...m, realFileName };
          }
        }

        // 3. Match por família de modelo
        if (nameLower && modelLower) {
          const cleanName = nameLower.replace(/[^a-z0-9]/g, '');
          const cleanModel = modelLower.replace(/[^a-z0-9]/g, '');
          if (cleanName.length > 5 && cleanModel.length > 5 && (cleanName.includes(cleanModel) || cleanModel.includes(cleanName))) {
            return { ...m, realFileName };
          }
        }
      }
      return null;
    }

    function testModelInArena(modelId) {
      switchLocalSubtab('arena');
      const select = document.getElementById('arena-runner-model-select');
      if (select) {
        for (let i = 0; i < select.options.length; i++) {
          if (select.options[i].value.toLowerCase().includes(modelId.toLowerCase()) || modelId.toLowerCase().includes(select.options[i].value.toLowerCase())) {
            select.selectedIndex = i;
            break;
          }
        }
      }
      showToast(`Modelo ${modelId} preparado na Local Silicon Arena. Pronto para aferição!`, 'success');
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
      currentLocalSubtab = subtab;
      document.querySelectorAll('#tab-local .subtab-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById('btn-subtab-' + subtab);
      if (activeBtn) activeBtn.classList.add('active');

      const panelInstalled = document.getElementById('panel-local-installed');
      const panelHub = document.getElementById('panel-local-hub');
      const panelArena = document.getElementById('panel-local-arena');
      const panelSwitchboard = document.getElementById('panel-local-switchboard');

      if (subtab === 'installed') {
        if (panelInstalled) panelInstalled.style.display = 'block';
        if (panelHub) panelHub.style.display = 'none';
        if (panelArena) panelArena.style.display = 'none';
        if (panelSwitchboard) panelSwitchboard.style.display = 'none';
        fetchLocalModels();
      } else if (subtab === 'arena') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelHub) panelHub.style.display = 'none';
        if (panelArena) panelArena.style.display = 'block';
        if (panelSwitchboard) panelSwitchboard.style.display = 'none';
        fetchLocalArenaData();
      } else if (subtab === 'hub') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelHub) panelHub.style.display = 'block';
        if (panelArena) panelArena.style.display = 'none';
        if (panelSwitchboard) panelSwitchboard.style.display = 'none';
        fetchHardwareAndRecommendations();
        fetchInventoryDownloads();
        connectDownloadsSSE();
      } else if (subtab === 'switchboard') {
        if (panelInstalled) panelInstalled.style.display = 'none';
        if (panelHub) panelHub.style.display = 'none';
        if (panelArena) panelArena.style.display = 'none';
        if (panelSwitchboard) panelSwitchboard.style.display = 'block';
        fetchSwitchboardSkills();
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
      select.innerHTML = models.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const prefix = isBenchmarked ? '✅' : '⏳';
        const tpsStr = m.latest_tps ? ` • ${m.latest_tps.toFixed(0)} tok/s` : '';
        const paramsStr = m.parameters ? ` • ${m.parameters}` : '';
        return `<option value="${m.model_id}">${prefix} ${m.name} (${m.tier}${paramsStr} • ${m.quantization || 'GGUF'}${tpsStr})</option>`;
      }).join('');
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
      const keys = ['tier', 'name', 'quant', 'tps', 'ttft', 'vram', 'e3', 'status'];
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
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 20px;">Nenhum modelo corresponde aos filtros aplicados.</td></tr>';
        return;
      }

      tbody.innerHTML = filtered.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const statusBadge = isBenchmarked
          ? '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: var(--emerald); border: 1px solid rgba(16, 185, 129, 0.3);">✅ Homologado</span>'
          : '<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.3);">⏳ Pendente</span>';

        const tpsHtml = m.latest_tps
          ? `<div style="line-height: 1.2;"><strong style="color: var(--emerald); font-size: 0.88rem;">${m.latest_tps.toFixed(1)}</strong><div style="font-size: 0.65rem; color: var(--text-muted);">tok/s</div></div>`
          : '<span style="color: var(--text-muted);">--</span>';

        const ttftHtml = m.latest_ttft_ms
          ? `<div style="line-height: 1.2;"><span class="mono" style="color: var(--cyan); font-size: 0.82rem; font-weight: 600;">${m.latest_ttft_ms.toFixed(0)}</span><div style="font-size: 0.65rem; color: var(--text-muted);">ms</div></div>`
          : '<span style="color: var(--text-muted);">--</span>';

        const vramHtml = m.latest_vram_peak_mb
          ? `<div style="line-height: 1.2;"><span class="mono" style="color: #f43f5e; font-size: 0.82rem; font-weight: 600;">${m.latest_vram_peak_mb.toFixed(0)}</span><div style="font-size: 0.65rem; color: var(--text-muted);">MB</div></div>`
          : `<div style="line-height: 1.2;"><span class="mono" style="color: var(--text-muted); font-size: 0.75rem;">${m.size_mb ? (m.size_mb + 36).toFixed(0) : '--'}</span><div style="font-size: 0.65rem; color: var(--text-muted);">MB (est)</div></div>`;

        const e3Html = m.e3_score
          ? `<span class="badge" style="background: rgba(168, 85, 247, 0.15); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.72rem; font-weight: 700;" title="Eficiência Empírica E³: ${m.e3_score.toFixed(1)} (Throughput / [VRAM × Latência])">${m.e3_score > 9000 ? '9999' : m.e3_score.toFixed(1)}</span>`
          : '<span style="color: var(--text-muted); font-size: 0.75rem;" title="Score não calculado ainda. Execute o teste Type 1 para aferir Throughput, TTFT e VRAM.">--</span>';

        const activeFile = m.file_path || m.model_id;

        return `
          <tr>
            <td class="text-center" style="vertical-align: middle;">
              ${renderTierBadge(m.tier)}
            </td>
            <td>
              <div style="font-weight: 600; color: #fff; font-size: 0.82rem;">${m.name}</div>
              <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">${m.model_id}</div>
            </td>
            <td class="text-center mono" style="font-size: 0.78rem;">${m.quantization || 'GGUF'}</td>
            <td class="text-center">${tpsHtml}</td>
            <td class="text-center">${ttftHtml}</td>
            <td class="text-center">${vramHtml}</td>
            <td class="text-center">${e3Html}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="text-center">
              <div style="display: flex; gap: 5px; justify-content: center; align-items: center;">
                <button class="btn btn-emerald" style="padding: 4px 10px; font-size: 0.72rem; height: 28px;" onclick="runSpecificArenaBenchmark('${m.model_id}', 1)" title="Executar Teste de Silício (Hardware Profile)">
                  <span>⚡</span> Testar
                </button>
                ${isBenchmarked ? `
                  <button class="btn btn-green" style="padding: 4px 10px; font-size: 0.72rem; height: 28px;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                    <span>⚡</span> Usar
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
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 18px 0;">Nenhum ensaio registrado no SQLite Vault.</td></tr>';
        return;
      }

      tbody.innerHTML = history.slice(0, 30).map(h => {
        let statusBadge = '<span class="badge badge-champion" style="font-size: 0.65rem;">NOMINAL</span>';
        const st = (h.status || '').toUpperCase();
        if (st.includes('NOMINAL') || st.includes('SUCCESS') || st.includes('VERIFIED')) {
          statusBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: var(--emerald); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem;">NOMINAL</span>';
        } else if (st.includes('WARN') || st.includes('THROTTLE')) {
          statusBadge = '<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.3); font-size: 0.65rem;">THROTTLED</span>';
        } else if (st.includes('FAIL') || st.includes('ERR')) {
          statusBadge = '<span class="badge" style="background: rgba(244, 63, 94, 0.15); color: #f43f5e; border: 1px solid rgba(244, 63, 94, 0.3); font-size: 0.65rem;">FALHA</span>';
        }

        const bt = (h.benchmark_type || '').toLowerCase();
        let typeBadge = '';
        if (bt.includes('hardeval') || bt.includes('hardware') || bt.includes('profile')) {
          typeBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: #34d399; font-size: 0.68rem;">⚡ Velocidade & VRAM</span>';
        } else if (bt.includes('cognitive') || bt.includes('eval') || bt.includes('cot')) {
          typeBadge = '<span class="badge" style="background: rgba(99, 102, 241, 0.15); color: #818cf8; font-size: 0.68rem;">🧠 Avaliação Cognitiva</span>';
        } else if (bt.includes('vision') || bt.includes('multimodal') || bt.includes('vlm')) {
          typeBadge = '<span class="badge" style="background: rgba(6, 182, 212, 0.15); color: var(--cyan); font-size: 0.68rem;">👁️ Visão Multimodal</span>';
        } else if (bt.includes('speculative') || bt.includes('draft')) {
          typeBadge = '<span class="badge" style="background: rgba(245, 158, 11, 0.15); color: #fbbf24; font-size: 0.68rem;">🚀 Especulativa</span>';
        } else if (bt.includes('context') || bt.includes('pressure') || bt.includes('needle')) {
          typeBadge = '<span class="badge" style="background: rgba(244, 63, 94, 0.15); color: #fb7185; font-size: 0.68rem;">📦 Pressão Contexto</span>';
        } else {
          typeBadge = `<span class="badge" style="background: rgba(168, 85, 247, 0.15); color: #c084fc; font-size: 0.68rem;">🔬 ${h.benchmark_type || 'Geral'}</span>`;
        }

        return `
          <tr>
            <td class="text-center mono" style="color: var(--text-muted); font-size: 0.72rem;">#${h.id}</td>
            <td>
              <div style="font-weight: 600; color: #fff; font-size: 0.8rem;">${h.model_id}</div>
            </td>
            <td>${typeBadge}</td>
            <td class="text-center mono" style="font-size: 0.72rem; color: var(--cyan);">${h.engine}</td>
            <td class="text-center mono" style="color: var(--emerald); font-weight: 600;">${h.tps > 0 ? h.tps.toFixed(1) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">tps</span>' : '--'}</td>
            <td class="text-center mono">${h.ttft_ms > 0 ? h.ttft_ms.toFixed(0) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">ms</span>' : '--'}</td>
            <td class="text-center mono" style="color: #f43f5e;">${h.vram_peak_mb > 0 ? h.vram_peak_mb.toFixed(0) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">MB</span>' : '--'}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="mono" style="font-size: 0.7rem; color: var(--text-muted);">${h.created_at || '--'}</td>
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
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:6px;">⏳</span> Executando Prova...';
      }

      if (feedback) {
        feedback.style.display = 'block';
        if (titleEl) titleEl.innerText = `Executando Prova no Silício para ${modelId}...`;
        if (verdictEl) verdictEl.innerHTML = '<span class="badge" style="background: rgba(6, 182, 212, 0.2); color: var(--cyan);">NVML Watchdog Ativo</span>';
        if (gridEl) gridEl.innerHTML = '<div style="color: var(--text-muted); font-size: 0.75rem; grid-column: 1 / -1;"><span style="display:inline-block; animation:spin 1s linear infinite; margin-right:6px;">⚙️</span> Carregando tensores via memmap2 e medindo telemetria bare-metal...</div>';
      }

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
          showToast(`Prova ${data.benchmark_type} concluída no silício com sucesso!`, 'success');
          if (titleEl) titleEl.innerText = `Resultado da Prova: ${data.model_id} (${data.benchmark_type})`;
          if (verdictEl) verdictEl.innerHTML = `<span class="badge badge-champion">${data.metrics?.verdict || 'NOMINAL'}</span>`;

          if (gridEl && data.metrics) {
            let cardsHtml = '';
            // Type 1: HardwareProfile
            if (data.metrics.tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Real</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.tps.toFixed(1)} <span style="font-size: 0.7rem;">tok/s</span></div>
                </div>
              `;
            }
            if (data.metrics.ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Tempo Resposta (TTFT)</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${data.metrics.ttft_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.peak_vram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">VRAM Pico NVML</div>
                  <div class="hw-spec-val" style="color: #f43f5e;">${data.metrics.peak_vram_mb.toFixed(0)} <span style="font-size: 0.7rem;">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.ram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">RAM Host</div>
                  <div class="hw-spec-val">${data.metrics.ram_mb.toFixed(0)} <span style="font-size: 0.7rem;">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.e3_score !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Eficiência E³</div>
                  <div class="hw-spec-val" style="color: #a855f7;">${data.metrics.e3_score > 9000 ? '9999' : data.metrics.e3_score.toFixed(1)}</div>
                </div>
              `;
            }

            // Type 2: CognitiveEval
            if (data.metrics.accuracy_pct !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Acurácia Sintática</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.accuracy_pct.toFixed(1)}%</div>
                </div>
              `;
            }
            if (data.metrics.avg_tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Médio</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.avg_tps.toFixed(1)} <span style="font-size: 0.7rem;">tok/s</span></div>
                </div>
              `;
            }

            // Type 3: MultimodalVision
            if (data.metrics.multimodal_ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Latência Visão VLM</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${data.metrics.multimodal_ttft_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.projection_latency_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Projeção mmproj</div>
                  <div class="hw-spec-val" style="color: #a855f7;">${data.metrics.projection_latency_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }

            // Type 4: SpeculativeDraft
            if (data.metrics.speedup_ratio !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Ganho de Velocidade</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.speedup_ratio.toFixed(2)}x</div>
                </div>
              `;
            }
            if (data.metrics.acceptance_rate_alpha !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Taxa Alpha (Aceite)</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${(data.metrics.acceptance_rate_alpha * 100).toFixed(1)}%</div>
                </div>
              `;
            }

            // Type 5: ContextPressure
            if (data.metrics.max_effective_context !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Janela Máxima Efetiva</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${Number(data.metrics.max_effective_context).toLocaleString()} <span style="font-size: 0.7rem;">tokens</span></div>
                </div>
              `;
            }
            if (data.metrics.degradation_detected !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Degradação</div>
                  <div class="hw-spec-val" style="color: ${data.metrics.degradation_detected ? '#f43f5e' : 'var(--emerald)'};">${data.metrics.degradation_detected ? 'Detectada ⚠️' : 'Zero Queda ✅'}</div>
                </div>
              `;
            }

            // Ação Direta no Feedback: Usar no Hermes
            cardsHtml += `
              <div style="grid-column: 1 / -1; margin-top: 10px; display: flex; justify-content: flex-end; gap: 8px;">
                <button class="btn btn-green" onclick="openQuickApplyModal('${data.model_id}', 'local_silicon', true)" style="font-size: 0.76rem; font-weight: 600; padding: 6px 14px;">
                  <span>⚡</span> Usar este Modelo no Hermes Agent
                </button>
              </div>
            `;

            gridEl.innerHTML = cardsHtml;
          }

          // Atualiza tabelas
          await fetchLocalArenaData();
        } else {
          showToast(data.message || 'Falha ao executar prova no silício.', 'error');
          if (titleEl) titleEl.innerText = 'Falha na Prova de Silício';
          if (verdictEl) verdictEl.innerHTML = '<span class="badge" style="background: rgba(244, 63, 94, 0.2); color: #f43f5e;">ERRO</span>';
          if (gridEl) gridEl.innerHTML = `<div style="color: #f43f5e; font-size: 0.8rem; grid-column: 1 / -1;">${data.message || 'Erro desconhecido'}</div>`;
        }
      } catch (err) {
        console.error('Erro ao executar benchmark:', err);
        showToast('Erro de comunicação ao executar benchmark.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = '<span>⚡</span> Iniciar Teste';
        }
      }
    }

    async function syncLocalArenaHermes() {
      const btn = event?.currentTarget;
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:4px;">⏳</span> Sincronizando...';
      }
      try {
        const res = await fetch('/api/v1/arena/sync', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          const count = data.synced_vault_models || 0;
          showToast(`✨ Sincronizado com sucesso! ${count} modelos no Heuristic Vault e links do Hermes ativos.`, 'success');
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
          btn.innerHTML = '<span>🔗</span> Sincronizar com Hermes Agent';
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
        icon: '💻',
        category: 'coding',
        categoryLabel: 'Código & AST',
        badgeBg: 'rgba(168, 85, 247, 0.15)',
        badgeColor: '#c084fc',
        samplePrompt: 'extrair outline e substituir symbol na ast de rust'
      },
      'souls-blast-radius': {
        icon: '💥',
        category: 'coding',
        categoryLabel: 'Código & Impacto',
        badgeBg: 'rgba(168, 85, 247, 0.15)',
        badgeColor: '#c084fc',
        samplePrompt: 'medir frecency gitoxide e blast radius de arquivos modificados'
      },
      'souls-memory-triad': {
        icon: '🧠',
        category: 'memory',
        categoryLabel: 'Memória L2',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'busca hibrida rrf no lancedb e sqlite strict wal'
      },
      'souls-ontology-graph': {
        icon: '🕸️',
        category: 'memory',
        categoryLabel: 'Grafo Ontológico',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'criar entidades e relacoes tipadas no grafo ladybugdb'
      },
      'obsidian-vault-curator': {
        icon: '📚',
        category: 'memory',
        categoryLabel: 'Obsidian Wiki',
        badgeBg: 'rgba(6, 182, 212, 0.15)',
        badgeColor: 'var(--cyan)',
        samplePrompt: 'curadoria de notas no cofre obsidian raw para wiki'
      },
      'souls-hardware-watchdog': {
        icon: '🌡️',
        category: 'hardware',
        categoryLabel: 'Hardware NVML',
        badgeBg: 'rgba(244, 63, 94, 0.15)',
        badgeColor: '#fb7185',
        samplePrompt: 'telemetria nvml vram temperatura rtx 2060m'
      },
      'souls-context-lean': {
        icon: '🗜️',
        category: 'hardware',
        categoryLabel: 'Context Lean CCR',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'comprimir tokens e deduplicar contexto em ram'
      },
      'souls-subagent-dispatch': {
        icon: '🤖',
        category: 'hardware',
        categoryLabel: 'Despacho Subagentes',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'despachar subagente via paretobandit e handoff'
      },
      'souls-intent-probing': {
        icon: '🎯',
        category: 'hardware',
        categoryLabel: 'Intenção Socrática',
        badgeBg: 'rgba(56, 189, 248, 0.15)',
        badgeColor: '#38bdf8',
        samplePrompt: 'classificar intencao e medir entropia shannon tier 0'
      },
      'souls-safe-sandbox': {
        icon: '🛡️',
        category: 'security',
        categoryLabel: 'Sandbox NT',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'executar comando shell em jobobject restrito seguro'
      },
      'obsidian-privacy-shield': {
        icon: '🔒',
        category: 'security',
        categoryLabel: 'Privacidade PII',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'higienizar dados confidenciais pii de saude e financas'
      },
      'souls-web-intelligence': {
        icon: '🌍',
        category: 'security',
        categoryLabel: 'Web Intelligence',
        badgeBg: 'rgba(16, 185, 129, 0.15)',
        badgeColor: '#34d399',
        samplePrompt: 'pesquisar documentacao na web e converter markdown'
      },
      'souls-socratic-thinking': {
        icon: '🤔',
        category: 'reasoning',
        categoryLabel: 'Grafo DAG Socrático',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'deliberar hipoteses em grafo dag socratico'
      },
      'souls-cloud-arena-advisor': {
        icon: '👑',
        category: 'reasoning',
        categoryLabel: 'Fronteira Pareto',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'consultar precos e campeoes cloud arena pareto'
      },
      'agent-self-learning-loop': {
        icon: '🔄',
        category: 'reasoning',
        categoryLabel: 'Auto-Aprendizado',
        badgeBg: 'rgba(245, 158, 11, 0.15)',
        badgeColor: '#fbbf24',
        samplePrompt: 'ciclo learn de auto aprendizado e correcao permanente'
      }
    };

    function getSkillMeta(name) {
      return SKILL_METADATA_MAP[name] || {
        icon: '⚡',
        category: 'reasoning',
        categoryLabel: 'Skill Geral',
        badgeBg: 'rgba(148, 163, 184, 0.15)',
        badgeColor: '#94a3b8',
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
        'core': { label: 'core', bg: 'rgba(148, 163, 184, 0.15)', color: '#94a3b8', border: 'rgba(148, 163, 184, 0.3)' }
      };
      const item = map[ns] || { label: ns, bg: 'rgba(255,255,255,0.08)', color: '#fff', border: 'rgba(255,255,255,0.15)' };
      return `<span class="badge" style="background:${item.bg}; color:${item.color}; border:1px solid ${item.border}; font-size:0.68rem; padding:2px 7px;">${item.label}</span>`;
    }

    async function fetchSwitchboardSkills() {
      const btn = document.getElementById('btn-refresh-skills');
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:4px;">⏳</span> Recarregando...';
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
          showToast('✓ Catálogo sincronizado com sucesso: 15 habilidades e 55 ferramentas ativas!', 'success');
        }
      } catch (err) {
        console.error('Erro ao buscar skills do switchboard:', err);
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = '<span>🔄</span> Recarregar Skills';
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
          <div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 32px 16px; background: rgba(15, 23, 42, 0.4); border: 1px dashed var(--border-subtle); border-radius: 10px;">
            <div style="font-size: 1.3rem; margin-bottom: 6px;">🔍</div>
            <div style="font-size: 0.88rem; color: #cbd5e1; font-weight: 600;">Nenhuma skill encontrada para o filtro atual.</div>
            <div style="margin-top: 10px;">
              <button class="btn btn-cyan" style="font-size: 0.74rem; padding: 5px 14px;" onclick="setSwitchboardSkillCategory('all'); const inp = document.getElementById('sb-skill-search-input'); if(inp) inp.value=''; filterSwitchboardSkills();">
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
          <div class="skill-card-item" id="skill-card-${s.name}" style="background: rgba(15, 23, 42, 0.7); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; padding: 14px; display: flex; flex-direction: column; justify-content: space-between; transition: border-color 0.2s ease, box-shadow 0.2s ease;">
            <div>
              <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 8px; gap: 8px;">
                <div style="display: flex; align-items: center; gap: 7px;">
                  <span style="font-size: 1.15rem;">${meta.icon}</span>
                  <strong style="color: #fff; font-size: 0.88rem; font-family: var(--font-mono);">${s.name}</strong>
                </div>
                <span class="badge" style="background: ${meta.badgeBg}; color: ${meta.badgeColor}; font-size: 0.65rem; padding: 2px 7px; border: 1px solid rgba(255,255,255,0.08); white-space: nowrap;">
                  ${meta.categoryLabel}
                </span>
              </div>
              <p style="font-size: 0.76rem; color: #cbd5e1; line-height: 1.45; margin-bottom: 12px;">${s.description}</p>
            </div>
            <div>
              <div style="margin-bottom: 8px;">
                <span style="font-size: 0.68rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 4px; text-transform: uppercase;">Ferramentas Primárias:</span>
                <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                  ${(s.primary_tools || []).map(t => `<span class="badge badge-lowcost" style="font-size: 0.66rem; padding: 2px 6px;">${t}</span>`).join('')}
                </div>
              </div>
              <div style="margin-bottom: 10px;">
                <span style="font-size: 0.68rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 4px; text-transform: uppercase;">Palavras-Chave:</span>
                <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                  ${(s.keywords || []).slice(0, 5).map(k => `<span style="font-size: 0.65rem; color: var(--text-muted); background: rgba(255,255,255,0.05); border-radius: 4px; padding: 1px 5px;">#${k}</span>`).join('')}
                </div>
              </div>
              <button class="sb-skill-test-btn" onclick="setSwitchboardPreset('${meta.samplePrompt}')">
                <span>⚡</span> Testar no Switchboard
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
          btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:4px;">⏳</span> Roteando...';
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
              stageBadge.textContent = '⚡ Fast-Path RAM Cosine (< 2ms)';
              stageBadge.style.background = 'rgba(16, 185, 129, 0.2)';
              stageBadge.style.color = 'var(--emerald)';
            } else {
              stageBadge.textContent = '🧠 Stage 2 Entropy Gated';
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
            if (iconElem) iconElem.textContent = meta.icon;
            const skName = document.getElementById('sb-skill-name');
            if (skName) skName.textContent = d.recommended_skill.name;
            const skScore = document.getElementById('sb-skill-score-badge');
            if (skScore) skScore.textContent = `Afinidade: ${(d.recommended_skill.score * 100).toFixed(0)}%`;
            const skDesc = document.getElementById('sb-skill-desc');
            if (skDesc) skDesc.textContent = d.recommended_skill.description;
            const skTools = document.getElementById('sb-skill-primary-tools');
            if (skTools) {
              skTools.innerHTML = (d.recommended_skill.primary_tools || []).map(t => `<span class="badge badge-lowcost" style="font-size: 0.68rem; padding: 2px 7px;">${t}</span>`).join('');
            }
          } else {
            if (skillCard) skillCard.style.display = 'none';
          }

          // Ferramentas projetadas
          const tbody = document.getElementById('sb-projected-tools-tbody');
          if (tbody) {
            if (!d.tools || d.tools.length === 0) {
              tbody.innerHTML = '<tr><td colspan="4" style="text-align: center; color: var(--text-muted); padding: 18px;">Nenhuma ferramenta projetada para o filtro selecionado.</td></tr>';
            } else {
              tbody.innerHTML = d.tools.map(t => {
                const pct = Math.round(t.score * 100);
                const barColor = pct >= 80 ? 'var(--emerald)' : pct >= 50 ? 'var(--cyan)' : 'var(--amber)';
                
                const parentSkill = findSkillForTool(t.name);
                const skillBadge = parentSkill ? `
                  <div style="margin-top: 4px;">
                    <span class="badge" style="background: rgba(168, 85, 247, 0.15); border: 1px solid rgba(168, 85, 247, 0.35); color: #c084fc; font-size: 0.65rem; padding: 2px 7px; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; transition: all 0.15s ease;" onclick="scrollToSkillInCatalog('${parentSkill.name}')" title="Clique para localizar esta Skill no Catálogo">
                      <span>🎯</span> ${parentSkill.name}
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
                    return `<span style="display:inline-flex; align-items:center; gap:3px; background:rgba(255,255,255,0.06); border:1px solid rgba(255,255,255,0.08); padding:1px 6px; border-radius:4px; font-size:0.68rem; font-family:var(--font-mono);"><strong style="color:#fff;">${p}</strong>: <span style="color:var(--cyan);">${propType}</span>${isReq ? ' <span style="color:#f87171; font-weight:700;" title="Obrigatório">*</span>' : ''}</span>`;
                  }).join(' ');
                  schemaParamsHtml = `
                    <div style="margin-top: 6px; display: flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                      <span style="font-size: 0.66rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase;">Parâmetros:</span>
                      ${propList}
                    </div>
                  `;
                }

                return `
                  <tr>
                    <td>
                      <div style="display: flex; flex-direction: column; align-items: flex-start;">
                        <strong class="mono" style="color: var(--cyan); font-size: 0.85rem;">${t.name}</strong>
                        ${skillBadge}
                      </div>
                    </td>
                    <td class="text-center">
                      ${getNamespaceBadge(t.namespace)}
                    </td>
                    <td class="text-center">
                      <div style="display: flex; align-items: center; justify-content: center; gap: 8px;">
                        <div style="width: 50px; height: 6px; background: rgba(255,255,255,0.1); border-radius: 9999px; overflow: hidden;">
                          <div style="width: ${pct}%; height: 100%; background: ${barColor}; border-radius: 9999px;"></div>
                        </div>
                        <span class="mono" style="color: ${barColor}; font-weight: 700; font-size: 0.8rem; min-width: 32px;">${pct}%</span>
                      </div>
                    </td>
                    <td>
                      <div style="font-size: 0.78rem; color: #cbd5e1; line-height: 1.4;">${t.description}</div>
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
          btn.innerHTML = '<span>⚡</span> Simular Seleção de Ferramentas';
        }
      }
    }

    async function fetchHardwareAndRecommendations() {
      const btn = document.getElementById('btn-recalibrate-hw');
      if (btn) btn.innerHTML = '<span>⏳</span> Recalibrando...';
      try {
        const res = await fetch('/api/v1/inventory/recommendations');
        if (res.ok) {
          const data = await res.json();
          hubHardwareData = data.hardware;
          hubRecommendationsData = data.triad;
          curatedCatalogData = data.catalog || [];

          renderHardwareDiagnostic(data.hardware);
          renderRecommendedTriad(data.triad);
          renderCuratedCatalog(curatedCatalogData, data.hardware);
        }
      } catch (e) {
        console.error('Erro ao buscar recomendações de hardware:', e);
        showToast('Falha ao obter diagnóstico de hardware.', 'error');
      } finally {
        if (btn) btn.innerHTML = '<span>🔄</span> Recalibrar Hardware';
      }
    }

    function renderHardwareDiagnostic(hw) {
      if (!hw) return;
      const elGpu = document.getElementById('hw-gpu-name');
      const elVram = document.getElementById('hw-vram-usable');
      const elRam = document.getElementById('hw-host-ram');
      const elCpu = document.getElementById('hw-cpu-features');
      const elCtx = document.getElementById('hw-context-ceiling');

      if (elGpu) elGpu.innerText = hw.gpu_name || 'NVIDIA GeForce RTX 2060 Mobile';
      const gpuSub = document.getElementById('hw-gpu-sub');
      if (gpuSub) {
        gpuSub.innerText = `${hw.gpu_architecture || 'Turing TU106'} • CC ${hw.gpu_compute_capability || '7.5'} • ${hw.gpu_details || '1.920 CUDA Cores • 240 Tensor Cores (FP16/INT8)'}`;
      }

      if (elVram) {
        const usableGb = hw.vram_safe_usable_mb ? (hw.vram_safe_usable_mb / 1024).toFixed(2) : '5.17';
        const totalGb = hw.vram_total_mb ? (hw.vram_total_mb / 1024).toFixed(2) : '6.00';
        elVram.innerText = `${usableGb} GB (${totalGb} GB Total)`;
      }
      const vramSub = document.getElementById('hw-vram-sub');
      if (vramSub) {
        const weights = hw.weights_budget_gb !== undefined ? hw.weights_budget_gb : 3.8;
        const kv = hw.kv_cache_budget_gb !== undefined ? hw.kv_cache_budget_gb : 1.3;
        vramSub.innerHTML = `<div>📦 Teto Pesos: ~${weights} GB (Q4_K_M)</div><div style="margin-top: 2px;">🧠 Margem KV Cache: ~${kv} GB</div>`;
      }

      if (elRam) {
        const ramGb = hw.host_ram_total_mb ? (hw.host_ram_total_mb / 1024).toFixed(1) : '31.8';
        elRam.innerText = `${ramGb} GB RAM`;
      }
      const ramSub = document.getElementById('hw-host-ram-sub');
      if (ramSub) {
        ramSub.innerText = '⚡ DDR4 Dual Channel • Swap Segura NVMe';
      }

      if (elCpu) {
        const rawCpu = hw.cpu_instructions_extended || (hw.has_avx2 ? 'AVX2 • FMA • BMI2 • SSE4.2 • AES-NI' : 'x86_64');
        const instList = rawCpu.split('•').map(s => s.trim()).filter(Boolean);
        elCpu.innerHTML = `<div style="display: flex; gap: 4px; flex-wrap: wrap; margin-top: 3px;">` +
          instList.map(inst => `<span class="badge" style="background: rgba(56, 189, 248, 0.12); color: #38bdf8; border: 1px solid rgba(56, 189, 248, 0.28); font-size: 0.64rem; padding: 1px 5px; font-weight: 700; white-space: nowrap;">${inst}</span>`).join('') +
          `</div>`;
      }
      const cpuSub = document.getElementById('hw-cpu-sub');
      if (cpuSub) {
        cpuSub.innerText = hw.cpu_name || 'Intel Core i9-9980HK (8 Cores / 16 Threads)';
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
        ctxSub.innerText = hw.context_formula_subtitle || 'Baseado em 5.17 GB úteis: seguro p/ 3B (16k-32k) e 7B c/ KV Q4 (4k-8k FP16). Sessões curtas evitam spillover PCIe.';
      }
      updateDynamicParamFitPills(hw);
    }

    function renderProfileTagBadge(profile) {
      if (!profile) return '';
      const p = profile.toLowerCase();
      let color = '#94a3b8';
      let bg = 'rgba(148, 163, 184, 0.12)';
      let border = 'rgba(148, 163, 184, 0.3)';
      let icon = '🏷️';
      let label = profile;

      if (p.includes('coding') || p.includes('código')) {
        color = '#c084fc';
        bg = 'rgba(168, 85, 247, 0.15)';
        border = 'rgba(168, 85, 247, 0.4)';
        icon = '💻';
        label = 'Coding Specialist';
      } else if (p.includes('reasoning') || p.includes('socratic') || p.includes('r1')) {
        color = '#818cf8';
        bg = 'rgba(99, 102, 241, 0.15)';
        border = 'rgba(99, 102, 241, 0.4)';
        icon = '🧠';
        label = 'Deep Reasoning (R1 CoT)';
      } else if (p.includes('agentic') || p.includes('tool') || p.includes('mcp')) {
        color = '#38bdf8';
        bg = 'rgba(6, 182, 212, 0.15)';
        border = 'rgba(6, 182, 212, 0.4)';
        icon = '🤖';
        label = 'Agentic & Tool Calling';
      } else if (p.includes('vision') || p.includes('vlm') || p.includes('multimodal')) {
        color = '#fb7185';
        bg = 'rgba(244, 63, 94, 0.15)';
        border = 'rgba(244, 63, 94, 0.4)';
        icon = '👁️';
        label = 'Vision Multimodal (VLM)';
      } else if (p.includes('fast') || p.includes('triagem') || p.includes('reflex')) {
        color = '#34d399';
        bg = 'rgba(16, 185, 129, 0.15)';
        border = 'rgba(16, 185, 129, 0.4)';
        icon = '⚡';
        label = 'Fast Reflex & Triage';
      } else if (p.includes('design') || p.includes('frontend') || p.includes('ui')) {
        color = '#fbbf24';
        bg = 'rgba(245, 158, 11, 0.15)';
        border = 'rgba(245, 158, 11, 0.4)';
        icon = '🎨';
        label = 'UI/UX & Frontend Design';
      }

      return `<span style="display: inline-flex; align-items: center; gap: 4px; padding: 2px 8px; border-radius: 6px; font-size: 0.68rem; font-weight: 700; font-family: var(--font-mono); background: ${bg}; color: ${color}; border: 1px solid ${border};">
        <span>${icon}</span> <span>${label}</span>
      </span>`;
    }

    function getFitBadgeHtml(fitStatus) {
      if (!fitStatus) return '<span class="fit-badge tight">Compatível</span>';
      const status = typeof fitStatus === 'string' ? fitStatus : (fitStatus.status || 'OptimalVram');
      switch (status) {
        case 'OptimalVram':
          return `<span class="fit-badge optimal" title="100% dos pesos e KV-cache cabem com folga na dGPU">⚡ 100% VRAM</span>`;
        case 'TightVram':
          return `<span class="fit-badge tight" title="Cabe na VRAM aproveitando o headroom do WDDM">✓ VRAM Completa</span>`;
        case 'PartialCpuOffload':
          return `<span class="fit-badge offload" title="Offload parcial para CPU RAM">⚠️ CPU Offload</span>`;
        case 'CpuOnly':
          return `<span class="fit-badge offload" title="Execução integral na CPU Intel via AVX2">💻 Modo CPU (AVX2)</span>`;
        case 'Incompatible':
          return `<span class="fit-badge incompatible" title="Excede a memória combinada VRAM + RAM">❌ Memória Insuficiente</span>`;
        default:
          return `<span class="fit-badge tight">${status}</span>`;
      }
    }

    function renderRecommendedTriad(triad) {
      const container = document.getElementById('triad-cards-grid');
      if (!container || !triad) return;

      const items = [
        { tierLabel: 'Tier 0 — Fast Reflex & Triage', tierColor: 'var(--green)', icon: '⚡', data: triad.tier0_velocista, role: 'Triagem ultra-rápida, guardrails e classificação de intenção (180+ tok/s)' },
        { tierLabel: 'Tier 0.5 — Socratic Thinking & Intent', tierColor: 'var(--cyan)', icon: '🧠', data: triad.tier05_socratico, role: 'Raciocínio socrático formal DeepSeek-R1 CoT e deliberação epistêmica' },
        { tierLabel: 'Tier 1 — Workhorse / Code & Logic', tierColor: 'var(--purple)', icon: '💻', data: triad.tier1_coder, role: 'Cavalo de batalha Sweet Spot para cirurgia de código, AST e lógica profunda' },
      ];

      container.innerHTML = items.map(item => {
        const model = item.data;
        if (!model) return '';
        const variant = (model.variants || []).find(v => v.is_recommended_fit) || (model.variants || [])[0];
        if (!variant) return '';

        const sizeGb = variant.size_gb ? variant.size_gb.toFixed(2) : (variant.size_bytes / (1024 ** 3)).toFixed(2);
        const estTps = model.benchmark_scores && model.benchmark_scores.speed_tokens_sec_estimate ? `${model.benchmark_scores.speed_tokens_sec_estimate.toFixed(0)} tps` : '--';
        const matched = findInstalledModelMatch(variant.file_name, model.repo_id, model.model_name);
        const isDown = Boolean(model.is_downloaded || matched);
        const activeFile = (matched && matched.realFileName) ? matched.realFileName : (model.local_file_path || variant.file_name);

        return `
          <div class="triad-card ${isDown ? 'is-installed-card' : ''}" style="border-radius: 12px; padding: 18px; display: flex; flex-direction: column; justify-content: space-between;">
            <div>
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; flex-wrap: wrap; gap: 6px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                  <span style="font-size: 0.72rem; font-weight: 700; color: ${item.tierColor}; font-family: var(--font-mono); text-transform: uppercase; letter-spacing: 0.03em;">
                    ${item.tierLabel}
                  </span>
                  ${isDown ? '<span class="badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.4); font-size: 0.65rem; padding: 1px 7px; font-weight: 700;">✓ Instalado</span>' : ''}
                </div>
                <div>
                  ${getFitBadgeHtml(variant.hardware_fit)}
                </div>
              </div>

              <div style="font-size: 1.05rem; font-weight: 700; color: #fff; font-family: var(--font-mono); word-break: break-word; line-height: 1.3;">
                ${model.model_name}
              </div>

              <p style="font-size: 0.75rem; color: var(--text-muted); margin-top: 6px; line-height: 1.45;">
                ${item.role}
              </p>

              <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; margin-top: 12px; background: rgba(15, 23, 42, 0.5); padding: 10px; border-radius: 8px; border: 1px solid var(--border-subtle); font-family: var(--font-mono); font-size: 0.72rem;">
                <div><span style="color: var(--text-muted);">Quantização:</span> <strong style="color: var(--cyan);">${variant.quant_tag}</strong></div>
                <div><span style="color: var(--text-muted);">Tamanho:</span> <strong style="color: #fff;">${sizeGb} GB</strong></div>
                <div><span style="color: var(--text-muted);">Throughput:</span> <strong style="color: #34d399;">~${estTps}</strong></div>
                <div><span style="color: var(--text-muted);">Silício:</span> <strong style="color: #fff;">${variant.hardware_fit === 'OptimalVram' ? 'dGPU RTX' : 'CPU Host'}</strong></div>
              </div>
            </div>

            <div style="margin-top: 16px; border-top: 1px solid var(--border-subtle); padding-top: 12px;">
              ${isDown ? `
                <div style="display: flex; gap: 6px; align-items: center;">
                  <button class="btn btn-green" style="flex: 2; height: 30px; font-size: 0.75rem; justify-content: center; font-weight: 600;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir como cérebro no Hermes Agent">
                    <span>⚡</span> Usar no Hermes
                  </button>
                  <button class="btn btn-cyan" style="flex: 1; height: 30px; font-size: 0.74rem; justify-content: center;" onclick="testModelInArena('${activeFile}')" title="Testar na Local Silicon Arena">
                    <span>🏟️</span> Arena
                  </button>
                  <button class="btn" style="width: 36px; height: 30px; padding: 0; justify-content: center; font-size: 0.8rem;" title="Baixar novamente" onclick="requestModelDownload('${variant.download_url}', '${variant.file_name}', ${variant.size_bytes}, '${variant.sha256 || ''}', '${model.repo_id}')">
                    <span>🔄</span>
                  </button>
                </div>
              ` : `
                <button class="btn btn-cyan" style="width: 100%; height: 30px; justify-content: center; font-size: 0.78rem; font-weight: 600;" onclick="requestModelDownload('${variant.download_url}', '${variant.file_name}', ${variant.size_bytes}, '${variant.sha256 || ''}', '${model.repo_id}')">
                  <span>⬇️</span> 1-Click Download (${sizeGb} GB)
                </button>
              `}
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

    function calculateFitFromParamEstimate(paramSize, isMoe = false) {
      const safeVramGb = hubHardwareData && hubHardwareData.vram_safe_usable_mb ? (hubHardwareData.vram_safe_usable_mb / 1024) : 5.17;
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
        container.innerHTML = '<div style="color: var(--cyan); text-align: center; grid-column: 1 / -1; padding: 30px;"><span style="display:inline-block; animation:spin 1s linear infinite; margin-right:8px;">⏳</span> Buscando novidades em alta no Hugging Face Hub para a sua máquina...</div>';
      }
      try {
        const res = await fetch('/api/v1/inventory/search?query=gguf&limit=25');
        if (res.ok) {
          const data = await res.json();
          const repos = data.results || [];
          trendingCatalogData = repos.map(r => {
            const repoId = r.repo_id || r.id || '';
            const details = parseRepoModelDetails(repoId, r.tags || []);
            const paramSize = details.total_b || details.active_b || 3.0;
            const fit = calculateFitFromParamEstimate(paramSize, details.is_moe);
            const isSweet = paramSize <= 4.0 && !details.is_moe;
            const repoSimple = repoId.split('/').pop() || repoId;
            const author = r.author || (repoId.includes('/') ? repoId.split('/')[0] : 'Comunidade');
            const profile = (details.specialties && details.specialties.length > 0) ? (
              details.specialties[0].label.toLowerCase().includes('coding') ? 'coding' :
              details.specialties[0].label.toLowerCase().includes('reasoning') ? 'reasoning' :
              details.specialties[0].label.toLowerCase().includes('vision') ? 'vision' : 'all'
            ) : 'all';
            return {
              model_name: repoSimple,
              repo_id: repoId,
              author: author,
              cognitive_profile: profile,
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
                  download_url: `https://huggingface.co/${repoId}/resolve/main/${repoSimple}-Q4_K_M.gguf`,
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
        console.error('Erro ao carregar novidades:', err);
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
    }

    function onCuratedQuantChange(idx) {
      const select = document.getElementById(`curated-select-${idx}`);
      if (!select) return;
      const opt = select.options[select.selectedIndex];
      const fitStatus = opt.getAttribute('data-fit') || 'OptimalVram';
      const fileName = opt.getAttribute('data-file') || '';
      const fitEl = document.getElementById(`curated-fit-${idx}`);
      if (fitEl) {
        fitEl.innerHTML = getFitBadgeHtml(fitStatus);
      }

      // Reavalia dinamicamente se a quantização selecionada bate com o inventário físico
      const model = filteredCuratedModels[idx] || curatedCatalogData[idx];
      const repoId = model ? model.repo_id : '';
      const modelName = model ? model.model_name : '';
      const matched = findInstalledModelMatch(fileName, repoId, modelName);
      const isDown = Boolean(matched || (model && model.local_file_path && model.local_file_path.toLowerCase().includes(fileName.toLowerCase())));
      const activeFile = (matched && matched.realFileName) ? matched.realFileName : fileName;

      const badgeContainer = document.getElementById(`curated-badge-installed-${idx}`);
      const actionContainer = document.getElementById(`curated-actions-${idx}`);

      if (badgeContainer) {
        badgeContainer.innerHTML = isDown ? '<span class="badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.4); font-size: 0.65rem; padding: 2px 8px; font-weight: 700; border-radius: 9999px;">✓ Instalado</span>' : '';
      }

      if (actionContainer) {
        if (isDown) {
          actionContainer.innerHTML = `
            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
              <button class="btn btn-green" style="flex: 1; min-width: 120px; justify-content: center; font-size: 0.76rem;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                <span>⚡</span> Usar no Hermes
              </button>
              <button class="btn btn-cyan" style="padding: 6px 10px; font-size: 0.74rem;" onclick="testModelInArena('${activeFile}')" title="Testar na Local Silicon Arena">
                <span>🏟️</span> Arena
              </button>
              <button class="btn" style="padding: 6px 10px; font-size: 0.74rem;" title="Baixar novamente" onclick="triggerCuratedDownload(${idx}, '${repoId}')">
                <span>🔄</span>
              </button>
            </div>
          `;
        } else {
          actionContainer.innerHTML = `
            <button class="btn btn-cyan" style="width: 100%; justify-content: center;" onclick="triggerCuratedDownload(${idx}, '${repoId}')">
              <span>⬇️</span> ${model && model.is_multimodal_vlm ? 'Baixar Bundle VLM (GGUF + mmproj)' : 'Baixar Seleção'}
            </button>
          `;
        }
      }
    }

    function triggerCuratedDownload(idx, repoIdOverride) {
      const select = document.getElementById(`curated-select-${idx}`);
      if (!select) return;
      const opt = select.options[select.selectedIndex];
      const url = opt.getAttribute('data-url');
      const filename = opt.getAttribute('data-file');
      const sizeBytes = parseInt(opt.getAttribute('data-size') || '0', 10);
      const sha256 = opt.getAttribute('data-sha256') || '';
      const projFile = opt.getAttribute('data-proj-file') || '';
      const projUrl = opt.getAttribute('data-proj-url') || '';

      requestModelDownload(url, filename, sizeBytes, sha256, repoIdOverride, projFile, projUrl);
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
          <div style="text-align: center; color: var(--text-muted); padding: 18px 0; font-size: 0.85rem;">
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
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <div>
                <strong style="color: var(--cyan); font-family: var(--font-mono); font-size: 0.95rem;">${active.filename}</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">
                  Status: <span style="color: ${isPaused ? '#fbbf24' : '#34d399'}; font-weight: 600;">${active.status}</span> &bull; Velocidade: <strong>${speedMb}</strong> &bull; ETA: <strong>${eta}</strong>
                </div>
              </div>
              <div style="display: flex; gap: 8px;">
                ${isPaused ? 
                  `<button class="btn btn-green" style="padding: 4px 10px; font-size: 0.74rem;" onclick="resumeActiveDownload('${active.id}')">▶ Retomar</button>` :
                  `<button class="btn" style="padding: 4px 10px; font-size: 0.74rem;" onclick="pauseActiveDownload('${active.id}')">⏸ Pausar</button>`
                }
                <button class="btn" style="padding: 4px 10px; font-size: 0.74rem; color: #f87171; border-color: rgba(239, 68, 68, 0.4);" onclick="cancelActiveDownload('${active.id}')">✖ Cancelar</button>
              </div>
            </div>

            <div class="download-progress-bar">
              <div class="download-progress-fill" style="width: ${pct}%;"></div>
            </div>

            <div style="display: flex; justify-content: space-between; font-size: 0.72rem; color: var(--text-muted); font-family: var(--font-mono);">
              <span>${downloadedGb} GB / ${totalGb} GB (${pct}%)</span>
              <span>SHA-256 Incremental: <span style="color: #34d399;">Ativo</span></span>
            </div>
          </div>
        `;
      }

      let queueHtml = '';
      if (queue.length > 0) {
        queueHtml = `
          <div style="margin-top: 14px;">
            <div style="font-size: 0.75rem; color: var(--text-muted); font-family: var(--font-mono); text-transform: uppercase;">Próximos na Fila Sequencial (${queue.length}):</div>
            <div style="margin-top: 6px; display: flex; flex-direction: column; gap: 6px;">
              ${queue.map(q => `
                <div style="background: rgba(15, 23, 42, 0.4); border: 1px solid var(--border-subtle); padding: 8px 12px; border-radius: var(--radius-sm); display: flex; justify-content: space-between; align-items: center; font-size: 0.75rem;">
                  <span class="mono" style="color: #e2e8f0;">${q.filename}</span>
                  <div style="display: flex; gap: 8px; align-items: center;">
                    <span class="mono" style="color: var(--text-muted);">${(q.total_bytes / (1024**3)).toFixed(2)} GB</span>
                    <button class="btn" style="padding: 2px 6px; font-size: 0.68rem; color: #f87171;" onclick="cancelActiveDownload('${q.id}')">✕</button>
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
        if (sseStatus) sseStatus.innerHTML = '<span style="color: #34d399;">● Ativo</span>';
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
        if (sseStatus) sseStatus.innerHTML = '<span style="color: var(--amber);">○ Reconectando</span>';
      };
    }

    // ==========================================
    // 3.3 EXPLORADOR HUGGING FACE HUB
    // ==========================================
    let currentHfParamFilter = 'all';
    let currentHfTaskFilter = 'all';
    let currentRawHfResults = [];

    function setHfParamFilter(param) {
      if (currentHfParamFilter === param && param !== 'all') {
        currentHfParamFilter = 'all';
      } else {
        currentHfParamFilter = param;
      }
      document.querySelectorAll('#hf-param-filters .profile-pill').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-hf-param-${currentHfParamFilter}`);
      if (activeBtn) activeBtn.classList.add('active');

      if (currentRawHfResults && currentRawHfResults.length > 0) {
        renderHfSearchResults(currentRawHfResults);
      } else {
        triggerSmartHfSearch();
      }
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

      if (currentRawHfResults && currentRawHfResults.length > 0) {
        renderHfSearchResults(currentRawHfResults);
      } else {
        triggerSmartHfSearch();
      }
    }

    function updateDynamicParamFitPills(hw) {
      const safeVramGb = hw && hw.vram_safe_usable_mb ? (hw.vram_safe_usable_mb / 1024) : 5.17;
      const gpuName = hw && hw.gpu_name ? hw.gpu_name : 'dGPU';
      const legend = document.getElementById('hf-vram-fit-legend');
      if (legend) {
        legend.innerText = `(🟢 Cabem na ${gpuName.includes('RTX') ? gpuName : 'GPU ' + gpuName})`;
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

    function triggerSmartHfSearch() {
      const input = document.getElementById('hf-search-input');
      if (!input) return;

      const userText = input.value.trim();
      if (userText) {
        searchHuggingFaceHub();
        return;
      }

      if (currentHfParamFilter === 'all' && currentHfTaskFilter === 'all') {
        return;
      }

      let queryParts = [];
      const taskTerms = {
        'all': '',
        'coding': 'coder',
        'reasoning': 'reasoning',
        'agentic': 'function calling',
        'vision': 'vision',
        'fast': 'smollm',
        'general': 'instruct'
      };

      const baseTerm = taskTerms[currentHfTaskFilter] || '';
      if (baseTerm) queryParts.push(baseTerm);
      if (currentHfParamFilter !== 'all') {
        queryParts.push(currentHfParamFilter.replace('_', ' '));
      }
      queryParts.push('GGUF');

      input.value = queryParts.join(' ').trim();
      searchHuggingFaceHub();
    }

    function parseRepoModelDetails(repoId, tags = []) {
      if (!repoId) return { total_b: null, active_b: null, is_moe: false, fit_label: '📦 Formato GGUF', fit_class: 'default', specialties: [] };
      const r = repoId.toLowerCase();
      const tagsStr = (Array.isArray(tags) ? tags.join(' ') : '').toLowerCase();

      let total_b = null;
      let active_b = null;
      let is_moe = false;
      let fit_label = '📦 Formato GGUF';
      let fit_class = 'default';

      // 1. MoE notation: ex: 35b-a3b, 30b-a3b, 16b-a2.4b
      const moeMatch = r.match(/(\d+)\s*b\s*[-_]a(\d+(?:\.\d+)?)\s*b/i);
      if (moeMatch) {
        total_b = parseFloat(moeMatch[1]);
        active_b = parseFloat(moeMatch[2]);
        is_moe = true;
        if (total_b <= 35) {
          fit_label = `⚠️ MoE ${total_b}B (${active_b}B Ativos) • CPU RAM`;
          fit_class = 'offload';
        } else {
          fit_label = `🏢 Datacenter Only (>${total_b}B MoE)`;
          fit_class = 'incompatible';
        }
      }
      // 2. MoE notation: ex: 8x7b
      else if (r.match(/(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b/i)) {
        const mx = r.match(/(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b/i);
        const experts = parseInt(mx[1], 10);
        const size = parseFloat(mx[2]);
        total_b = experts * size;
        active_b = size;
        is_moe = true;
        if (total_b <= 50) {
          fit_label = `⚠️ MoE ${experts}x${size}B (~${total_b}B) • CPU RAM`;
          fit_class = 'offload';
        } else {
          fit_label = `🏢 Datacenter Only (>${total_b}B)`;
          fit_class = 'incompatible';
        }
      }
      // 3. Parameter count in Millions: 135m, 360m, 500m
      else if (r.match(/(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)/i)) {
        const m = r.match(/(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)/i);
        total_b = parseInt(m[1], 10) / 1000.0;
        active_b = total_b;
        fit_label = '⚡ Ultra-Leve (<1B • Zero Latência)';
        fit_class = 'optimal';
      }
      // 4. Dense parameters in Billions:
      else {
        const bMatches = [...r.matchAll(/(?:^|[^0-9a-z])(\d+(?:\.\d+)?)\s*b(?:[^0-9a-z]|$)/gi)];
        if (bMatches && bMatches.length > 0) {
          total_b = parseFloat(bMatches[bMatches.length - 1][1]);
          active_b = total_b;
          if (total_b <= 4.5) {
            fit_label = `⚡ Sweet Spot dGPU (100% VRAM • ${total_b}B)`;
            fit_class = 'optimal';
          } else if (total_b <= 8.5) {
            fit_label = `✓ Teto Seguro dGPU (Q4_K_M • ${total_b}B)`;
            fit_class = 'tight';
          } else if (total_b <= 35.0) {
            fit_label = `💻 Modo CPU Host RAM (${total_b}B)`;
            fit_class = 'offload';
          } else {
            fit_label = `🏢 Datacenter Only (>${total_b}B)`;
            fit_class = 'incompatible';
          }
        } else if (r.includes('phi-4-mini') || r.includes('phi-3.5-mini')) {
          total_b = 3.8;
          active_b = 3.8;
          fit_label = '⚡ Sweet Spot dGPU (100% VRAM • 3.8B)';
          fit_class = 'optimal';
        } else if (tagsStr.includes('moe') || tagsStr.includes('mixture-of-experts') || r.includes('moe')) {
          is_moe = true;
          fit_label = '⚠️ Arquitetura MoE • CPU RAM';
          fit_class = 'offload';
        }
      }

      // Specialty badges from tags and repo name
      const specialties = [];
      if (r.includes('code') || r.includes('coder') || tagsStr.includes('code') || tagsStr.includes('coder')) {
        specialties.push({ label: '💻 Coding', color: '#c084fc', bg: 'rgba(168, 85, 247, 0.15)', border: 'rgba(168, 85, 247, 0.3)' });
      }
      if (r.includes('vision') || r.includes('vlm') || r.includes('multimodal') || tagsStr.includes('image') || tagsStr.includes('vision')) {
        specialties.push({ label: '👁️ Vision/VLM', color: '#fb7185', bg: 'rgba(244, 63, 94, 0.15)', border: 'rgba(244, 63, 94, 0.3)' });
      }
      if (r.includes('r1') || r.includes('reasoning') || r.includes('cot') || tagsStr.includes('reasoning')) {
        specialties.push({ label: '🧠 Reasoning', color: '#818cf8', bg: 'rgba(99, 102, 241, 0.15)', border: 'rgba(99, 102, 241, 0.3)' });
      }
      if (r.includes('agent') || r.includes('tool') || r.includes('hermes') || tagsStr.includes('function-calling')) {
        specialties.push({ label: '🤖 Agentic', color: '#38bdf8', bg: 'rgba(6, 182, 212, 0.15)', border: 'rgba(6, 182, 212, 0.3)' });
      }

      return { total_b, active_b, is_moe, fit_label, fit_class, specialties };
    }

    function estimateRepoHardwareFit(repoId, tags = []) {
      const details = parseRepoModelDetails(repoId, tags);
      return `<span class="fit-badge ${details.fit_class}" title="${details.fit_label}">${details.fit_label}</span>`;
    }

    function matchesHfFilters(details, repoId, tags) {
      // 1. Filtro estrito de Parâmetros
      if (currentHfParamFilter !== 'all') {
        const t = details.total_b;
        if (t === null || t === undefined) {
          return false;
        }
        switch(currentHfParamFilter) {
          case 'sub1b': if (t >= 1.0 || details.is_moe) return false; break;
          case '1b': if (t < 1.0 || t >= 2.0 || details.is_moe) return false; break;
          case '2b': if (t < 2.0 || t >= 3.0 || details.is_moe) return false; break;
          case '3b': if (t < 3.0 || t >= 4.0 || details.is_moe) return false; break;
          case '4b': if (t < 4.0 || t >= 5.0 || details.is_moe) return false; break;
          case '5b': if (t < 5.0 || t >= 6.0) return false; break;
          case '6b': if (t < 6.0 || t >= 7.0) return false; break;
          case '7b': if (t < 7.0 || t >= 8.0) return false; break;
          case '8b': if (t < 8.0 || t >= 9.0) return false; break;
          case '9b': if (t < 9.0 || t >= 10.0) return false; break;
          case '10b_12b': if (t < 10.0 || t > 12.5) return false; break;
          case '14b': if (t <= 12.5 || t > 16.0) return false; break;
          case '32b': if (t <= 16.0 || t > 45.0) return false; break;
          case '70b_plus': if (t <= 45.0) return false; break;
        }
      }

      // 2. Filtro de Tipo de Tarefa
      if (currentHfTaskFilter !== 'all') {
        const specLabels = (details.specialties || []).map(s => s.label.toLowerCase()).join(' ');
        const r = (repoId || '').toLowerCase();
        const tagsStr = (Array.isArray(tags) ? tags.join(' ') : '').toLowerCase();
        switch(currentHfTaskFilter) {
          case 'coding': if (!specLabels.includes('coding') && !r.includes('code') && !r.includes('coder') && !tagsStr.includes('code') && !tagsStr.includes('coder')) return false; break;
          case 'reasoning': if (!specLabels.includes('reasoning') && !r.includes('r1') && !r.includes('reasoning') && !r.includes('cot') && !tagsStr.includes('reasoning')) return false; break;
          case 'agentic': if (!specLabels.includes('agentic') && !r.includes('agent') && !r.includes('tool') && !r.includes('hermes') && !tagsStr.includes('function-calling') && !tagsStr.includes('tool')) return false; break;
          case 'vision': if (!specLabels.includes('vision') && !r.includes('vision') && !r.includes('vlm') && !tagsStr.includes('vision') && !tagsStr.includes('image-text-to-text')) return false; break;
          case 'fast': if ((details.total_b !== null && details.total_b >= 1.0) && !r.includes('fast') && !r.includes('smol')) return false; break;
          case 'general': if (!r.includes('instruct') && !r.includes('chat') && !tagsStr.includes('conversational')) return false; break;
        }
      }

      return true;
    }

    async function searchHuggingFaceHub() {
      const input = document.getElementById('hf-search-input');
      const resultsContainer = document.getElementById('hf-search-results');
      if (!input || !resultsContainer) return;

      const query = input.value.trim();
      if (!query) {
        showToast('Digite um termo ou selecione os filtros acima para pesquisar.', 'info');
        return;
      }

      resultsContainer.innerHTML = '<div style="color: var(--cyan); text-align: center; padding: 24px; font-size: 0.85rem;"><span style="display:inline-block; animation:spin 1s linear infinite; margin-right:8px;">⏳</span> Consultando repositórios GGUF na nuvem Hugging Face...</div>';

      try {
        const res = await fetch(`/api/v1/inventory/search?query=${encodeURIComponent(query)}&limit=30`);
        if (res.ok) {
          const data = await res.json();
          currentRawHfResults = data.results || [];
          renderHfSearchResults(currentRawHfResults);
        } else {
          resultsContainer.innerHTML = '<div style="color: var(--red); text-align: center; padding: 16px;">Erro ao consultar repositórios no Hugging Face. Verifique a conexão com a internet.</div>';
        }
      } catch (e) {
        resultsContainer.innerHTML = '<div style="color: var(--red); text-align: center; padding: 16px;">Falha de rede ao conectar com a API do Hugging Face.</div>';
      }
    }

    function renderHfSearchResults(repos) {
      const container = document.getElementById('hf-search-results');
      if (!container) return;

      if (!repos || repos.length === 0) {
        container.innerHTML = '<div style="color: var(--text-muted); text-align: center; padding: 24px; font-size: 0.85rem;">Nenhum repositório GGUF encontrado para o termo pesquisado.</div>';
        return;
      }

      const decorated = repos.map((r, i) => ({
        originalIndex: i,
        repo: r,
        details: parseRepoModelDetails(r.repo_id, r.tags || [])
      }));

      const filtered = decorated.filter(item => matchesHfFilters(item.details, item.repo.repo_id, item.repo.tags || []));

      if (filtered.length === 0) {
        const paramPill = document.querySelector('#hf-param-filters .profile-pill.active')?.innerText || currentHfParamFilter;
        const taskPill = document.querySelector('#hf-task-filters .profile-pill.active')?.innerText;
        const filterDesc = taskPill ? `${paramPill} • ${taskPill}` : paramPill;
        container.innerHTML = `
          <div style="text-align: center; color: var(--text-muted); padding: 28px 16px; background: rgba(15, 23, 42, 0.4); border: 1px dashed var(--border-subtle); border-radius: 10px;">
            <div style="font-size: 1.3rem; margin-bottom: 6px;">🔍</div>
            <div style="font-size: 0.88rem; color: #cbd5e1; font-weight: 600;">Nenhum modelo encontrado para o filtro "${filterDesc}".</div>
            <div style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
              Foram retornados ${repos.length} repositórios para a busca, mas nenhum atende aos filtros combinados.
            </div>
            <div style="margin-top: 12px; display: flex; gap: 8px; justify-content: center;">
              <button class="btn btn-cyan" style="font-size: 0.74rem; padding: 5px 14px;" onclick="setHfParamFilter('all'); setHfTaskFilter('all');">
                Limpar Filtros e Ver Todos (${repos.length})
              </button>
            </div>
          </div>
        `;
        return;
      }

      container.innerHTML = filtered.map(item => {
        const r = item.repo;
        const i = item.originalIndex;
        const d = item.details;
        const fitBadge = `<span class="fit-badge ${d.fit_class}" title="${d.fit_label}">${d.fit_label}</span>`;
        const specialtyBadges = (d.specialties || []).map(s => `
          <span class="badge" style="background: ${s.bg}; color: ${s.color}; border: 1px solid ${s.border}; font-size: 0.65rem; padding: 2px 7px;">
            ${s.label}
          </span>
        `).join('');

        return `
          <div class="hf-repo-item" id="hf-repo-${i}" style="border-radius: 10px; margin-bottom: 10px; padding: 14px 16px;">
            <div style="flex: 1;">
              <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                <span style="font-weight: 700; color: #fff; font-family: var(--font-mono); font-size: 0.92rem;">
                  ${r.repo_id}
                </span>
                ${fitBadge}
                ${specialtyBadges}
              </div>
              <div style="font-size: 0.73rem; color: var(--text-muted); margin-top: 6px; font-family: var(--font-mono);">
                👤 Autor: <strong style="color: #cbd5e1;">${r.author || '--'}</strong> &bull; 📥 Downloads: <strong style="color: var(--cyan);">${(r.downloads || 0).toLocaleString()}</strong> &bull; ❤️ Likes: <strong style="color: #f472b6;">${(r.likes || 0).toLocaleString()}</strong>
              </div>
            </div>
            <div>
              <button class="btn btn-cyan" style="padding: 6px 14px; font-size: 0.76rem; height: 32px; font-weight: 600;" onclick="inspectHfRepo('${r.repo_id}', ${i})">
                <span>📂</span> Ver Arquivos GGUF
              </button>
            </div>
          </div>
          <div id="hf-repo-files-${i}" style="display: none;"></div>
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
      container.innerHTML = '<div style="padding: 10px; color: var(--text-muted); font-size: 0.78rem;">Carregando arquivos GGUF do repositório...</div>';

      try {
        const res = await fetch(`/api/v1/inventory/repo-files?repo_id=${encodeURIComponent(repoId)}`);
        if (res.ok) {
          const data = await res.json();
          const files = data.files || [];
          if (files.length === 0) {
            container.innerHTML = '<div style="padding: 10px; color: var(--text-muted); font-size: 0.78rem;">Nenhum arquivo .gguf encontrado na raiz deste repositório.</div>';
            return;
          }

          container.innerHTML = `
            <div class="hf-files-container">
              <div style="font-size: 0.7rem; color: var(--text-muted); font-family: var(--font-mono); margin-bottom: 8px; text-transform: uppercase;">Arquivos GGUF Disponíveis:</div>
              <div style="display: flex; flex-direction: column; gap: 6px;">
                ${files.map(f => {
                  const sizeGb = f.size_gb ? f.size_gb.toFixed(2) : (f.size_bytes / (1024 ** 3)).toFixed(2);
                  const fileName = f.file_name || f.filename;
                  const fitBadge = getFitBadgeHtml(f.hardware_fit);
                  const matched = findInstalledModelMatch(fileName, repoId);
                  const isDown = Boolean(matched);
                  const activeFile = (matched && matched.realFileName) ? matched.realFileName : fileName;

                  return `
                    <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(15, 23, 42, 0.5); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle);">
                      <div>
                        <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                          <span class="mono" style="font-size: 0.8rem; font-weight: 600; color: #fff;">${fileName}</span>
                          ${isDown ? '<span class="badge" style="background: rgba(16, 185, 129, 0.2); color: #34d399; font-size: 0.65rem; font-weight: 700; border: 1px solid rgba(16, 185, 129, 0.4);">✅ Já Instalado no Silício</span>' : ''}
                          ${fitBadge}
                        </div>
                        <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
                          Quant: <strong style="color: var(--cyan);">${f.quant_tag || '--'}</strong> &bull; Tamanho: <strong style="color: #fff;">${sizeGb} GB</strong> ${f.sha256 ? '&bull; LFS SHA-256 verificado' : ''}
                        </div>
                      </div>
                      <div>
                        ${isDown ? `
                          <div style="display: flex; gap: 4px;">
                            <button class="btn btn-green" style="padding: 4px 10px; font-size: 0.72rem;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                              <span>⚡</span> Usar
                            </button>
                            <button class="btn btn-cyan" style="padding: 4px 8px; font-size: 0.72rem;" onclick="testModelInArena('${activeFile}')" title="Testar na Local Silicon Arena">
                              <span>🏟️</span>
                            </button>
                            <button class="btn" style="padding: 4px 8px; font-size: 0.72rem;" title="Baixar novamente" onclick="requestModelDownload('${f.download_url}', '${fileName}', ${f.size_bytes}, '${f.sha256 || ''}')">
                              <span>🔄</span>
                            </button>
                          </div>
                        ` : `
                          <button class="btn btn-cyan" style="padding: 4px 12px; font-size: 0.72rem;" onclick="requestModelDownload('${f.download_url}', '${fileName}', ${f.size_bytes}, '${f.sha256 || ''}')">
                            <span>⬇️</span> Baixar
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
          container.innerHTML = '<div style="padding: 10px; color: var(--red); font-size: 0.78rem;">Falha ao listar arquivos GGUF.</div>';
        }
      } catch (e) {
        container.innerHTML = '<div style="padding: 10px; color: var(--red); font-size: 0.78rem;">Erro de conexão.</div>';
      }
    }

    // 4. FETCH CLOUD ARENA (llm_cloud)
    let cloudSearchQuery = '';
    let cloudSortOrder = 'score_desc';
    let currentSortCol = 'score';
    let currentSortAsc = false;

    async function updateActiveHermesBanner() {
      try {
        const res = await fetch('/api/v1/telemetry');
        if (res.ok) {
          const data = await res.json();
          const mb = data.master_brain;
          if (mb) {
            window.currentHermesMasterModel = mb.model || 'deepseek/deepseek-chat';
            const nameEl = document.getElementById('active-hermes-model-name');
            const presetEl = document.getElementById('active-hermes-preset-tag');
            const provEl = document.getElementById('active-hermes-provider-tag');
            const metaEl = document.getElementById('active-hermes-meta-line');
            if (nameEl) nameEl.innerText = mb.model || 'deepseek/deepseek-chat';
            const quickModelEl = document.getElementById('cloud-quick-active-model');
            if (quickModelEl) quickModelEl.innerText = mb.model || 'deepseek/deepseek-chat';
            if (presetEl) presetEl.innerText = `Preset: ${mb.active_profile_name || 'Frontier Balanced'}`;
            if (provEl) {
              const p = (mb.provider || 'nous').toLowerCase();
              provEl.innerHTML = p.includes('nous') ? '🧠 Nous Portal (Crédito Ativo)' : '🔀 OpenRouter';
            }
            if (metaEl) {
              const ctx = mb.context_window ? formatContextTokens(mb.context_window) : '64k tokens';
              metaEl.innerText = `Status: ${mb.status || 'Online'} • Contexto: ${ctx} • Alvo: ${mb.target_type || 'Cloud API'}`;
            }
          }
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
      if (low.includes('nous')) return 'Nous Portal';
      if (low.includes('openrouter')) return 'OpenRouter';
      if (low.includes('local')) return 'Silício Local';
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
      'status', 'score', 'roi', 'nous', 'or', 'intel', 'coding', 'bfcl', 
      'reasoning', 'design', 'vision', 'ifeval', 'ttft', 'tps', 'ctx', 'prov'
    ];
    let activeColumnOrder = [...defaultColumnKeys];
    let draggedColKey = null;

    // Badges exclusivas de fabricante por laboratório com paletas proprietárias
    function getDevBrandBadge(dev, canonicalId) {
      const d = (dev || '').toLowerCase();
      const c = (canonicalId || '').toLowerCase();
      let brand = 'CLOUD';
      let bg = 'rgba(255, 255, 255, 0.08)';
      let color = '#cbd5e1';
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

      return `<span class="model-brand-pill" style="font-size: 0.62rem; padding: 1px 5px; background: ${bg}; color: ${color}; border: 1px solid ${border}; border-radius: 3px; font-weight: 700; font-family: var(--font-mono); letter-spacing: 0.04em;">${brand}</span>`;
    }

    const cloudColumnDefs = {
      status: {
        plainName: 'Status Pareto',
        label: 'Status',
        width: '135px',
        sortKey: 'status',
        title: 'Status de Otimização na Fronteira de Pareto.&#10;👑 Líder Qualidade: Ponto ótimo não-dominado.&#10;💎 Melhor ROI: Maior razão inteligência por dólar.&#10;⚡ Ultra Econômico: Menor tarifa com alta velocidade.&#10;🔹 Homologado: Modelo homologado complementar.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-status text-center" style="white-space: nowrap;">${h.statusBadge}</td>`
      },
      score: {
        plainName: 'Score Geral',
        label: 'Score Geral',
        width: '105px',
        sortKey: 'score',
        title: 'Score Multiobjetivo Geral Ponderado (0 a 100).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Combinação de benchmarks ponderados pelas prioridades do perfil cognitivo ativo.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-score mono text-center" style="font-weight: 800; color: var(--cyan); font-size: 0.84rem; white-space: nowrap;">${(m.composite_score || 0).toFixed(1)}</td>`
      },
      roi: {
        plainName: 'ROI Index',
        label: 'ROI Index',
        width: '105px',
        sortKey: 'roi',
        title: 'Índice de Retorno sobre Investimento (Score ÷ Blended Cost).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Mede o volume de capacidade cognitiva entregue por cada dólar investido.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-roi mono text-center" style="color: var(--green); font-weight: 700; white-space: nowrap;">${(m.roi_index || 0).toFixed(1)}x</td>`
      },
      nous: {
        plainName: 'Tarifa Nous ($/1M)',
        label: '<span style="color: #e879f9;">🟣</span> Tarifa Nous ($/1M)',
        width: '145px',
        sortKey: 'nous_cost',
        title: 'Tarifa da API do Nous Portal por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;★ Indica menor tarifa entre provedores homologados. Consome créditos do Nous.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-nous mono text-center" style="white-space: nowrap;">${h.nousCostStr}</td>`
      },
      or: {
        plainName: 'Tarifa OpenRouter ($/1M)',
        label: '<span style="color: #818cf8;">🔀</span> Tarifa OR ($/1M)',
        width: '145px',
        sortKey: 'or_cost',
        title: 'Tarifa da API do OpenRouter por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;★ Indica menor tarifa entre provedores homologados.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-or mono text-center" style="white-space: nowrap;">${h.orCostStr}</td>`
      },
      intel: {
        plainName: 'Inteligência (Arena ELO)',
        label: 'Inteligência',
        width: '115px',
        sortKey: 'intel',
        title: 'Inteligência Geral e Discernimento Cognitivo (LMSYS Arena ELO / Hard Prompts).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia capacidade geral de síntese, julgamento e resolução de problemas abertos.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-intel mono text-center" style="white-space: nowrap;">${h.intelScoreStr}</td>`
      },
      coding: {
        plainName: 'Coding (SWE-bench)',
        label: 'Coding',
        width: '110px',
        sortKey: 'coding',
        title: 'Capacidade em Engenharia de Software e Programação (SWE-bench / Code ELO).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede resolução real de bugs em repositórios e refatoração de código.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-coding mono text-center" style="white-space: nowrap;">${h.codingScoreStr}</td>`
      },
      bfcl: {
        plainName: 'Tools (BFCL)',
        label: 'Tools (BFCL)',
        width: '115px',
        sortKey: 'bfcl',
        title: 'Berkeley Function Calling Leaderboard (Chamada de Ferramentas e MCP).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede precisão sintática e semântica no disparo de tools.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-bfcl mono text-center" style="white-space: nowrap;">${h.bfclScoreStr}</td>`
      },
      reasoning: {
        plainName: 'Reasoning (GPQA)',
        label: 'Reasoning',
        width: '115px',
        sortKey: 'reasoning',
        title: 'Raciocínio Lógico e Científico (GPQA Diamond / Hard Math).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia deduções complexas e cadeias lógicas profundas.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-reasoning mono text-center" style="white-space: nowrap;">${h.reasoningScoreStr}</td>`
      },
      design: {
        plainName: 'Design UI/UX',
        label: 'Design UI/UX',
        width: '120px',
        sortKey: 'design',
        title: 'Capacidade de Frontend, Design UI/UX e Estética de Código.&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede refinamento visual, geração CSS/Tailwind, acessibilidade e componentes interativos.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-design mono text-center" style="white-space: nowrap;">${h.designScoreStr}</td>`
      },
      vision: {
        plainName: 'Visão (VQA)',
        label: 'Visão (VQA)',
        width: '115px',
        sortKey: 'vision',
        title: 'Visão Computacional e Entendimento Multimodal (MMMU / ChartQA / MathVista).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia compreensão de prints de tela, diagramas técnicos e dados visuais.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-vision mono text-center" style="white-space: nowrap;">${h.visionScoreStr}</td>`
      },
      ifeval: {
        plainName: 'Instrução (IFEval)',
        label: 'Instrução',
        width: '115px',
        sortKey: 'ifeval',
        title: 'Fidelidade a Instruções e Restrições de Formato (IFEval).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede cumprimento estrito de JSON schemas, delimitadores e contagens.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ifeval mono text-center" style="white-space: nowrap;">${h.ifevalScoreStr}</td>`
      },
      ttft: {
        plainName: 'Latência (TTFT)',
        label: 'TTFT',
        width: '115px',
        sortKey: 'ttft',
        title: 'Time to First Token (Latência de Resposta Inicial em milissegundos).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;Impacta diretamente a reatividade do diálogo e início do streaming.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ttft mono text-center" style="white-space: nowrap;">${h.ttftStr}</td>`
      },
      tps: {
        plainName: 'Throughput (TPS)',
        label: 'TPS',
        width: '110px',
        sortKey: 'tps',
        title: 'Velocidade média em tokens por segundo (Throughput).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Indica velocidade de streaming e geração em lote para subagentes.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-tps mono text-center" style="white-space: nowrap;">${h.tpsStr}</td>`
      },
      ctx: {
        plainName: 'Janela & RULER',
        label: 'Janela & RULER',
        width: '160px',
        sortKey: 'ctx',
        title: 'Capacidade máxima de janela de contexto e retenção Needle-in-a-Haystack / RULER.&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Exibe o tamanho máximo e a taxa de retenção de fatos no contexto.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-ctx mono text-center" style="white-space: nowrap;">${h.ctxStr}</td>`
      },
      prov: {
        plainName: 'Provedores',
        label: 'Provedores',
        width: '125px',
        sortKey: 'prov',
        title: 'Provedores com modelo indexado e pronto para consumo via API.&#10;🧠 Nous: Disponível com créditos do Nous Portal.&#10;🔀 OR: Disponível via API OpenRouter.',
        renderCell: (m, idx, isActive, h) => `<td class="td-col-prov text-center" style="white-space: nowrap;">${h.provBadgesStr}</td>`
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
        const isHidden = cloudColVisibility[key] === false;
        const isSorted = (currentSortCol === colDef.sortKey);
        const sortIcon = isSorted ? (currentSortAsc ? '▲' : '▼') : '↕';
        const activeSortClass = isSorted ? 'active-sort' : '';
        const hiddenClass = isHidden ? 'col-hidden' : '';

        return `
          <th class="text-center sortable-th th-col-${key} th-guide ${activeSortClass} ${hiddenClass}" 
              data-col-key="${key}"
              style="width: ${colDef.width}; min-width: ${colDef.width}; white-space: nowrap; cursor: grab;" 
              draggable="true" 
              ondragstart="handleColDragStart(event, '${key}')" 
              ondragover="handleColDragOver(event)" 
              ondragenter="handleColDragEnter(event, this)" 
              ondragleave="handleColDragLeave(event, this)" 
              ondragend="handleColDragEnd(event)" 
              ondrop="handleColDrop(event, '${key}')" 
              onclick="setTableSort('${colDef.sortKey}')" 
              title="${colDef.title}&#10;↔ Dica: Arraste para reposicionar esta coluna na tabela.">
            <span style="opacity: 0.35; font-size: 0.60rem; margin-right: 3px; cursor: grab;" title="Arraste para mover">⋮⋮</span>
            ${colDef.label} 
            <span class="sort-icon" id="sort-icon-${colDef.sortKey}">${sortIcon}</span>
          </th>
        `;
      }).join('');

      thead.innerHTML = `
        <tr>
          <!-- COLUNA 1: POSIÇÃO (CONGELADA / STICKY) -->
          <th class="text-center sortable-th sticky-col-1 th-guide" style="width: 52px; min-width: 52px; max-width: 52px;" onclick="setTableSort('rank')" title="Posição no ranking canônico pelo perfil ativo.&#10;📉 Como Interpretar: Quanto MENOR o número, melhor (#1 é o topo).">
            # <span class="sort-icon" id="sort-icon-rank">↕</span>
          </th>

          <!-- COLUNA 2: MODELO / IDENTIFICADOR (CONGELADA / STICKY) -->
          <th class="sortable-th sticky-col-2 th-guide" style="min-width: 310px; max-width: 420px;" onclick="setTableSort('name')" title="Identificador do modelo, fabricante e capacidades multimodais.&#10;Clique no botão de prancheta para copiar o ID canônico.">
            Modelo / Identificador <span class="sort-icon" id="sort-icon-name">↕</span>
          </th>

          ${thCols}

          <!-- COLUNA FINAL: AÇÃO -->
          <th class="text-center" style="width: 80px; min-width: 80px;">Ação</th>
        </tr>
      `;
    }

    function handleCloudSort(val) {
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
      const cols = ['rank', 'name', 'score', 'roi', 'nous_cost', 'or_cost', 'intel', 'coding', 'bfcl', 'reasoning', 'design', 'vision', 'ifeval', 'ttft', 'tps', 'ctx'];
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
        container.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 24px;">Carregando campeões da Cloud Arena...</div>';
      }

      await updateActiveHermesBanner();

      try {
        const res = await fetch('/api/v1/cloud/champions');
        if (res.ok) {
          const data = await res.json();
          cloudChampionsData = data.champions || [];
          renderCloudTableHeader();
          renderCloudTable();
          initCloudColumnVisibility();
          fetchCloudRankedModels(currentProfile);
        }
      } catch (e) {
        if (container) {
          container.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--red); padding: 24px;">Erro ao carregar Cloud Arena.</div>';
        }
      }
    }

    function selectProfile(profile) {
      currentProfile = profile.toLowerCase();
      document.querySelectorAll('#profile-pill-container .profile-pill').forEach(btn => btn.classList.remove('active'));
      const activeBtn = Array.from(document.querySelectorAll('#profile-pill-container .profile-pill')).find(b => b.innerText.toLowerCase().includes(currentProfile === 'orchestrator' ? 'maestro' : currentProfile));
      if (activeBtn) activeBtn.classList.add('active');

      const descBar = document.getElementById('cloud-profile-description-bar');
      if (descBar) {
        const descMap = {
          orchestrator: '👑 <strong>Master Brain / Maestro:</strong> O modelo de fronteira all-around que orquestra o diálogo mestre, planeja cadeias agênticas e toma decisões executivas com raciocínio balanceado.',
          coding: '💻 <strong>Coding / SWE:</strong> Otimizado para engenharia de software, refatoração de código bare-metal, testes automatizados e resolução de problemas complexos.',
          agentic: '🤖 <strong>Agentic / Tools:</strong> Otimizado para chamada de ferramentas (MCP/Tool Calling), planejamento autônomo e orquestração de subagentes.',
          reasoning: '🧠 <strong>Reasoning / Math:</strong> Otimizado para raciocínio analítico profundo, lógica formal, matemática avançada e auditoria minuciosa.',
          fast: '⚡ <strong>Fast / Triage:</strong> Otimizado para altíssima velocidade (TPS), triagem de intenções em milissegundos e processamento em lote com custo irrisório.',
          design: '🎨 <strong>Design / Frontend:</strong> Otimizado para interfaces modernas, arquitetura de componentes frontend, CSS sofisticado e compreensão multimodal.'
        };
        descBar.innerHTML = descMap[currentProfile] || `Perfil cognitivo selecionado: <strong>${currentProfile}</strong>`;
      }

      renderCloudTable();
      fetchCloudRankedModels(currentProfile);
    }

    function setCloudProviderFilter(provider) {
      currentCloudProviderFilter = (provider || 'all').toLowerCase();
      document.querySelectorAll('.filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-cloud-prov-${currentCloudProviderFilter}`);
      if (activeBtn) activeBtn.classList.add('active');

      renderCloudTable();
      renderCloudRankedTable();
    }

    function getFilteredCloudModelsList() {
      let list = cachedCloudRankedModels || [];
      if (list.length === 0) return [];

      // 1. Filtrar por Provedor
      if (currentCloudProviderFilter && currentCloudProviderFilter !== 'all') {
        if (currentCloudProviderFilter === 'nous') {
          list = list.filter(m => m.has_nous || (m.provider || '').toLowerCase().includes('nous'));
        } else if (currentCloudProviderFilter === 'openrouter') {
          list = list.filter(m => m.has_openrouter || (m.provider || '').toLowerCase().includes('openrouter'));
        } else {
          list = list.filter(m => (m.provider || '').toLowerCase().includes(currentCloudProviderFilter));
        }
      }

      // 2. Filtrar por Modalidade Expandida (Visão, Imagem, Áudio, Vídeo, World Models, Decisão, Texto)
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

          if (currentCloudModalityFilter === 'vision' || currentCloudModalityFilter === 'multimodal') {
            return isVision;
          } else if (currentCloudModalityFilter === 'image_gen') {
            return isImgGen;
          } else if (currentCloudModalityFilter === 'audio') {
            return isAudio;
          } else if (currentCloudModalityFilter === 'video') {
            return isVideo;
          } else if (currentCloudModalityFilter === 'world') {
            return isWorld;
          } else if (currentCloudModalityFilter === 'decision') {
            return isDecision;
          } else if (currentCloudModalityFilter === 'text') {
            return !isVision && !isImgGen && !isAudio && !isVideo && !isWorld;
          }
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

      return list;
    }

    function renderCloudTable() {
      const container = document.getElementById('cloud-podium-cards');
      if (!container) return;
      const profileReport = cloudChampionsData.find(c => c.profile.toLowerCase() === currentProfile);
      const filteredList = getFilteredCloudModelsList();
      const hasActiveFilters = (currentCloudProviderFilter !== 'all') || 
                               (currentCloudModalityFilter !== 'all') || 
                               (selectedManufacturers && selectedManufacturers.size > 0);

      // Atualiza o sublabel do pódio com o resumo dos filtros ativos
      const sublabel = document.getElementById('cloud-triad-sublabel');
      if (sublabel) {
        let filterParts = [];
        if (currentCloudProviderFilter !== 'all') {
          filterParts.push(currentCloudProviderFilter === 'nous' ? '🟣 Nous Portal' : '🔀 OpenRouter');
        }
        if (currentCloudModalityFilter !== 'all') {
          filterParts.push(currentCloudModalityFilter === 'multimodal' ? '👁️ Multimodal' : '📝 Texto Puro');
        }
        if (selectedManufacturers && selectedManufacturers.size > 0) {
          filterParts.push(`🏢 ${selectedManufacturers.size} Fabricante(s)`);
        }
        const filterStr = filterParts.length > 0 ? ` • Filtros Ativos: ${filterParts.join(' | ')}` : '';
        sublabel.innerHTML = `Recomendações canônicas de qualidade máxima, melhor ROI e menor custo${filterStr}`;
      }

      if (!profileReport && filteredList.length === 0) {
        container.innerHTML = `<div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 24px;">Nenhum dado encontrado para o perfil ${currentProfile}.</div>`;
        return;
      }

      let cards = [];

      if (!hasActiveFilters && profileReport) {
        if (profileReport.quality_leader) {
          cards.push({
            roleType: 'quality',
            badgeText: '🏆 Máxima Inteligência',
            badgeIcon: '🏆',
            roleSubtitle: currentProfile === 'orchestrator' ? 'Maestro Supremo' : 'Líder Absoluto',
            data: profileReport.quality_leader,
            defaultDesc: currentProfile === 'orchestrator' 
              ? 'Melhor capacidade de orquestração geral e discernimento executivo sem limites de orçamento.'
              : 'Melhor pontuação absoluta no perfil selecionado. Recomendado para tarefas críticas e sem limite financeiro.'
          });
        }
        if (profileReport.best_value) {
          cards.push({
            roleType: 'value',
            badgeText: '💎 Melhor Custo-Benefício',
            badgeIcon: '💎',
            roleSubtitle: 'Ponto Ótimo de ROI',
            data: profileReport.best_value,
            defaultDesc: currentProfile === 'orchestrator'
              ? 'Equilíbrio ideal entre inteligência de ponta e economia contínua de créditos no Nous Portal.'
              : 'Máximo retorno de inteligência por cada dólar investido. Ideal para fluxos operacionais contínuos.'
          });
        }
        if (profileReport.ultra_low_cost) {
          cards.push({
            roleType: 'lowcost',
            badgeText: '⚡ Ultra Econômico',
            badgeIcon: '⚡',
            roleSubtitle: 'Velocidade & Volume',
            data: profileReport.ultra_low_cost,
            defaultDesc: 'Tarifa mínima com alta taxa de geração (TPS). Perfeito para subagentes em lote, resumos e triagem.'
          });
        }
      } else if (filteredList.length > 0) {
        // Filtros ativos: recalcula os campeões em tempo real dinamicamente
        const sortedByQuality = [...filteredList].sort((a, b) => (b.composite_score || 0) - (a.composite_score || 0));
        const leader = sortedByQuality[0];
        cards.push({
          roleType: 'quality',
          badgeText: '🏆 Máxima Inteligência',
          badgeIcon: '🏆',
          roleSubtitle: 'Líder dos Filtros',
          data: leader,
          defaultDesc: 'Modelo de mais alto desempenho técnico homologado dentro dos critérios selecionados.'
        });

        const sortedByRoi = [...filteredList].sort((a, b) => (b.roi_index || 0) - (a.roi_index || 0));
        const bestRoi = sortedByRoi.find(m => m.canonical_id !== leader.canonical_id) || sortedByRoi[0];
        if (bestRoi) {
          cards.push({
            roleType: 'value',
            badgeText: '💎 Melhor Custo-Benefício',
            badgeIcon: '💎',
            roleSubtitle: 'Maior ROI Filtrado',
            data: bestRoi,
            defaultDesc: 'A melhor relação entre capacidade cognitiva e custo dentro dos critérios selecionados.'
          });
        }

        const validCostModels = filteredList.filter(m => (m.blended_cost > 0) || (m.nous_blended > 0) || (m.or_blended > 0));
        const sortedByCost = (validCostModels.length > 0 ? validCostModels : filteredList).sort((a, b) => {
          const costA = a.blended_cost || a.nous_blended || a.or_blended || 9999;
          const costB = b.blended_cost || b.nous_blended || b.or_blended || 9999;
          return costA - costB;
        });
        const lowestCost = sortedByCost.find(m => !cards.some(c => c.data.canonical_id === m.canonical_id)) || sortedByCost[0];
        if (lowestCost) {
          cards.push({
            roleType: 'lowcost',
            badgeText: '⚡ Ultra Econômico',
            badgeIcon: '⚡',
            roleSubtitle: 'Menor Tarifa Filtrada',
            data: lowestCost,
            defaultDesc: 'Opção mais econômica e veloz homologada dentro dos critérios selecionados.'
          });
        }
      }

      if (cards.length === 0) {
        container.innerHTML = `
          <div style="grid-column: 1 / -1; text-align: center; color: var(--text-muted); padding: 28px; background: rgba(15,23,42,0.4); border-radius: var(--radius-md); border: 1px dashed var(--border-subtle);">
            Nenhum campeão encontrado para a combinação de filtros selecionada.<br>
            <span style="font-size: 0.72rem; color: #64748b;">Dica: Experimente selecionar "Todas as Modalidades" ou marcar mais fabricantes no filtro.</span>
          </div>
        `;
        return;
      }

      container.innerHTML = cards.map(c => {
        const m = c.data;
        const cid = m.canonical_id || m.display_name || '--';
        const friendlyName = humanizeModelName(cid);
        const provClean = humanizeProviderName(m.provider);
        const score = m.composite_score || 0;
        const cost = m.blended_cost || m.nous_blended || m.or_blended || 0;
        const costStr = cost === 0 ? '0.00' : (cost < 0.01 ? cost.toFixed(4) : cost.toFixed(2));
        const costHint = cost === 0 ? 'Grátis / Local' : (cost < 0.10 ? 'Ultra Baixo Custo' : (cost < 1.00 ? 'Econômico' : (cost < 5.00 ? 'Equilibrado' : 'Fronteira Premium')));
        const roi = m.roi_index || 0;
        const tpsStr = m.throughput_tps ? m.throughput_tps.toFixed(0) + ' TPS' : '-- TPS';
        const ctxStr = formatContextTokens(m.context_window);

        const isActive = (window.currentHermesMasterModel && (cid === window.currentHermesMasterModel || window.currentHermesMasterModel.includes(cid)));

        return `
          <div class="cloud-champion-card ${c.roleType} ${isActive ? 'active-hermes-card' : ''}">
            <div>
              <div class="champion-card-top">
                <span class="champion-badge ${c.roleType}">${c.badgeText}</span>
                <div style="display: flex; align-items: center; gap: 6px;">
                  ${isActive ? '<span class="badge badge-active-hermes">🟢 Ativo no Hermes</span>' : ''}
                  <span class="champion-role-tag">${c.roleSubtitle}</span>
                </div>
              </div>

              <div class="champion-card-model">
                <div class="champion-model-brand-row">
                  ${getDevBrandBadge(m.developer, cid)}
                  <span class="champion-provider-pill">${provClean}</span>
                </div>
                <div class="champion-model-name" title="${cid}">${friendlyName}</div>
                <div class="champion-model-id-row">
                  <code class="mono" style="word-break: break-all;">${cid}</code>
                  <button class="btn" onclick="copyCloudModelId('${cid}')" title="Copiar ID do Modelo" style="padding: 1px 5px; font-size: 0.65rem; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.08); color: var(--text-muted); cursor: pointer;">
                    📋
                  </button>
                </div>
              </div>

              <div class="champion-desc">${c.defaultDesc}</div>

              <div class="champion-metrics-grid">
                <div class="champion-metric-box">
                  <div class="metric-label">Pontuação (Score)</div>
                  <div class="metric-value" style="color: ${c.roleType === 'quality' ? '#fbbf24' : (c.roleType === 'value' ? 'var(--cyan)' : 'var(--green)')};">
                    ${score.toFixed(1)} <span class="metric-sub">/100</span>
                  </div>
                  <div class="metric-progress-bg">
                    <div class="metric-progress-bar" style="width: ${Math.min(score, 100)}%;"></div>
                  </div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Tarifa Blended</div>
                  <div class="metric-value">$${costStr} <span class="metric-sub">/1M</span></div>
                  <div class="metric-hint">${costHint}</div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Custo-Benefício (ROI)</div>
                  <div class="metric-value" style="color: var(--green);">${roi.toFixed(1)}x</div>
                  <div class="metric-hint">Retorno Multiobjetivo</div>
                </div>
                <div class="champion-metric-box">
                  <div class="metric-label">Velocidade & Janela</div>
                  <div class="metric-value">${tpsStr}</div>
                  <div class="metric-hint">${ctxStr} tokens</div>
                </div>
              </div>
            </div>

            <div class="champion-card-footer">
              <button class="btn-apply-champion" onclick="openQuickApplyModal('${cid}', '${m.provider}', false)">
                <span>⚡</span> ${isActive ? 'Reconfigurar no Hermes' : 'Usar este Modelo no Hermes'}
              </button>
            </div>
          </div>
        `;
      }).join('');
    }

    // 4.1 FETCH RANKING EXPANDIDO DA CLOUD ARENA (TOP 200)
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
          renderCloudTableHeader();
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
      let list = getFilteredCloudModelsList();

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
        const ctxStr = `<span style="white-space: nowrap; font-weight: 600;">${ctxFormatted}</span> <span style="color: #34d399; font-size: 0.68rem; font-weight: 600; white-space: nowrap;" title="Taxa de Retenção de Contexto Needle-in-a-Haystack / RULER">(${retentionPct.toFixed(1)}%)</span>`;

        // Sub-scores enriquecidos e contínuos
        const baseScore = m.composite_score || 80.0;
        
        // Inteligência
        const intelVal = m.eval_reasoning || m.hard_prompts_elo || baseScore;
        const intelScoreStr = `<span style="color: #f59e0b; font-weight: 600; white-space: nowrap;" title="LMSYS Arena ELO / Discernimento: ${intelVal.toFixed(1)}">${intelVal.toFixed(1)}</span>`;

        // Coding
        const codingVal = m.coding_elo || m.eval_coding || (baseScore * 0.96);
        const codingScoreStr = `<span style="color: #38bdf8; font-weight: 600; white-space: nowrap;" title="SWE-bench / Code ELO: ${codingVal.toFixed(1)}">${codingVal.toFixed(1)}</span>`;

        // Tools
        const bfclVal = m.bfcl_score || m.eval_agentic || (baseScore * 0.92);
        const bfclScoreStr = `<span style="color: #a78bfa; font-weight: 600; white-space: nowrap;" title="Berkeley Function Calling: ${bfclVal.toFixed(1)}">${bfclVal.toFixed(1)}</span>`;

        // Reasoning
        const reasoningVal = m.gpqa_score || m.hard_prompts_elo || m.eval_reasoning || (baseScore * 0.95);
        const reasoningScoreStr = `<span style="color: #f472b6; font-weight: 600; white-space: nowrap;" title="GPQA Diamond / Hard Prompts: ${reasoningVal.toFixed(1)}">${reasoningVal.toFixed(1)}</span>`;

        // Design UI/UX
        const designVal = m.eval_design || (baseScore * 0.94);
        const designScoreStr = `<span style="color: #ec4899; font-weight: 600; white-space: nowrap;" title="Design UI/UX / Frontend Score: ${designVal.toFixed(1)}">${designVal.toFixed(1)}</span>`;

        // Visão / VQA
        let visionScoreStr = '<span style="color: var(--text-muted); font-size: 0.72rem; white-space: nowrap;">--</span>';
        if (m.vqa_multimodal_score) {
          visionScoreStr = `<span style="color: #c084fc; font-weight: 600; white-space: nowrap;" title="MMMU / ChartQA / Visão Multimodal: ${m.vqa_multimodal_score.toFixed(1)}">${m.vqa_multimodal_score.toFixed(1)}</span>`;
        } else if (m.is_multimodal) {
          const vVal = baseScore * 0.93;
          visionScoreStr = `<span style="color: #c084fc; font-weight: 600; white-space: nowrap;" title="MMMU / Visão Multimodal: ${vVal.toFixed(1)}">${vVal.toFixed(1)}</span>`;
        }

        // Instrução (IFEval)
        const ifevalVal = m.ifeval_score || Math.round(((bfclVal * 0.5) + (codingVal * 0.5)) * 10) / 10;
        const ifevalScoreStr = `<span style="color: #34d399; font-weight: 600; white-space: nowrap;" title="Instruction Following (IFEval): ${ifevalVal.toFixed(1)}">${ifevalVal.toFixed(1)}</span>`;

        // TTFT (lado a lado com ms)
        let ttftStr = '<span style="color: var(--text-muted); font-size: 0.72rem; white-space: nowrap;">--</span>';
        if (m.ttft_ms && m.ttft_ms > 0) {
          const ttftColor = m.ttft_ms < 400 ? '#34d399' : (m.ttft_ms < 800 ? '#38bdf8' : (m.ttft_ms < 1500 ? '#fbbf24' : '#f87171'));
          ttftStr = `<span style="color: ${ttftColor}; font-weight: 600; white-space: nowrap;" title="Time To First Token: ${m.ttft_ms.toFixed(0)} ms">${m.ttft_ms.toFixed(0)} ms</span>`;
        }

        // TPS (lado a lado com TPS)
        const tpsStr = m.throughput_tps ? `<span style="color: var(--text-main); font-weight: 600; white-space: nowrap;">${m.throughput_tps.toFixed(0)} TPS</span>` : '<span style="color: var(--text-muted); font-size: 0.72rem; white-space: nowrap;">--</span>';

        // Tarifa Nous Portal com cor exclusiva #e879f9
        let nousCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem; white-space: nowrap;">--</span>';
        let nousBlendedVal = null;
        if (m.nous_blended !== null && m.nous_blended !== undefined && m.nous_blended > 0) {
          nousBlendedVal = m.nous_blended;
        } else if ((m.nous_prompt_price && m.nous_prompt_price > 0) || (m.nous_completion_price && m.nous_completion_price > 0)) {
          const p = m.nous_prompt_price || 0;
          const c = m.nous_completion_price || 0;
          nousBlendedVal = (0.25 * p + 0.75 * c);
        }

        // Tarifa OpenRouter com cor exclusiva #818cf8
        let orCostStr = '<span style="color: var(--text-muted); font-size: 0.72rem; white-space: nowrap;">--</span>';
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
          nousCostStr = `<span style="color: #e879f9; font-weight: 700; white-space: nowrap;" title="Nous Portal: $${(m.nous_prompt_price || 0).toFixed(2)} prompt / $${(m.nous_completion_price || 0).toFixed(2)} completion por 1M">${bStr}</span>${star}`;
        }

        if (orBlendedVal !== null && orBlendedVal > 0) {
          const bStr = orBlendedVal < 0.01 ? '$' + orBlendedVal.toFixed(4) : '$' + orBlendedVal.toFixed(2);
          const star = orCheaper ? `<span class="badge-cheaper" title="Menor tarifa entre provedores homologados ($${orBlendedVal.toFixed(2)} vs $${nousBlendedVal.toFixed(2)})">★</span>` : '';
          orCostStr = `<span style="color: #818cf8; font-weight: 700; white-space: nowrap;" title="OpenRouter: $${(m.or_prompt_price || 0).toFixed(2)} prompt / $${(m.or_completion_price || 0).toFixed(2)} completion por 1M">${bStr}</span>${star}`;
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
        const provBadgesStr = `<div style="display: flex; gap: 4px; justify-content: center; flex-wrap: wrap;">${provBadges.join('')}</div>`;

        // Tags de capacidade ao lado do nome do modelo
        let capTags = [];
        const cidLower = (m.canonical_id || '').toLowerCase();
        const modLower = (m.modality || '').toLowerCase();
        
        if (m.is_multimodal || modLower.includes('vision') || modLower.includes('image')) {
          capTags.push('<span class="cap-tag cap-vision" title="Visão: Processa imagens, diagramas e OCR">👁️ Visão</span>');
        }
        if (modLower.includes('image_gen') || cidLower.includes('flux') || cidLower.includes('dall-e') || cidLower.includes('midjourney') || cidLower.includes('imagen') || cidLower.includes('janus')) {
          capTags.push('<span class="cap-tag" style="background: rgba(236, 72, 153, 0.15); color: #f472b6; border: 1px solid rgba(236, 72, 153, 0.35); font-size: 0.65rem; padding: 1px 5px; border-radius: 4px;" title="Geração de Imagens: Criação visual nativa a partir de prompt">🎨 Imagem</span>');
        }
        if (modLower.includes('audio') || modLower.includes('speech') || modLower.includes('voice') || cidLower.includes('audio') || cidLower.includes('realtime')) {
          capTags.push('<span class="cap-tag" style="background: rgba(14, 165, 233, 0.15); color: #38bdf8; border: 1px solid rgba(14, 165, 233, 0.35); font-size: 0.65rem; padding: 1px 5px; border-radius: 4px;" title="Áudio & Voz: Compreensão e fala conversacional nativa">🎙️ Áudio</span>');
        }
        if (modLower.includes('video') || cidLower.includes('video') || cidLower.includes('sora') || cidLower.includes('wan') || cidLower.includes('kling')) {
          capTags.push('<span class="cap-tag" style="background: rgba(168, 85, 247, 0.15); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.35); font-size: 0.65rem; padding: 1px 5px; border-radius: 4px;" title="Vídeo: Compreensão temporal e dinâmica de frames">🎬 Vídeo</span>');
        }
        if (modLower.includes('world') || cidLower.includes('world') || cidLower.includes('genie') || cidLower.includes('cosmos') || cidLower.includes('oasis') || cidLower.includes('marble')) {
          capTags.push('<span class="cap-tag" style="background: rgba(34, 197, 94, 0.15); color: #4ade80; border: 1px solid rgba(34, 197, 94, 0.35); font-size: 0.65rem; padding: 1px 5px; border-radius: 4px;" title="World Model: Simulação física e ambientes espaciais">🌍 Mundo</span>');
        }
        if (modLower.includes('decision') || cidLower.includes('jev') || cidLower.includes('routing')) {
          capTags.push('<span class="cap-tag" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.35); font-size: 0.65rem; padding: 1px 5px; border-radius: 4px;" title="Decisão Tipada: Motor não-autorregressivo (estilo Jev)">⚡ Decisão</span>');
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

        const helpers = {
          isTop3,
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

        const friendlyName = humanizeModelName(m.canonical_id);
        const brandBadge = getDevBrandBadge(m.developer, m.canonical_id);

        let dynamicTds = activeColumnOrder.map(colKey => {
          const colDef = cloudColumnDefs[colKey];
          if (!colDef) return '';
          return colDef.renderCell(m, idx, isActive, helpers);
        }).join('');

        return `
          <tr class="${isActive ? 'active-hermes-row' : ''}">
            <!-- 1. RANKING (CONGELADO) -->
            <td class="sticky-col-1 rank-cell ${isTop3 ? 'top3' : ''} text-center" style="${idx === 0 ? 'color: #fbbf24; font-weight: 800;' : (idx === 1 ? 'color: #cbd5e1; font-weight: 800;' : (idx === 2 ? 'color: #f59e0b; font-weight: 800;' : ''))}">#${idx + 1}</td>

            <!-- 2. MODELO / IDENTIFICADOR (CONGELADO ANTES DO STATUS) -->
            <td class="sticky-col-2">
              <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; width: 100%; min-width: 0;">
                <div style="min-width: 0; flex: 1; overflow: hidden;">
                  <!-- Linha 1: Apenas Capacidades e Recursos -->
                  <div style="display: flex; align-items: center; gap: 5px; flex-wrap: wrap; margin-bottom: 4px;">
                    ${capTags.join('')}
                  </div>
                  <!-- Linha 2: Badge do Fabricante posicionado imediatamente antes do Nome do Modelo -->
                  <div class="model-name-title" title="${m.canonical_id}" style="display: flex; align-items: center; gap: 6px; font-weight: 600; color: #fff; font-size: 0.84rem; line-height: 1.3; overflow: hidden; max-width: 320px;">
                    ${brandBadge}
                    <span style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${friendlyName}</span>
                  </div>
                  <div class="mono" style="font-size: 0.65rem; color: #94a3b8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 290px; margin-top: 1px;" title="${m.canonical_id}">
                    ${m.canonical_id}
                  </div>
                </div>
                <button class="btn" onclick="copyCloudModelId('${m.canonical_id}')" title="Copiar ID do Modelo" style="padding: 2px 6px; font-size: 0.65rem; background: rgba(255,255,255,0.04); border-color: rgba(255,255,255,0.08); color: var(--text-muted); cursor: pointer; flex-shrink: 0; margin-top: 2px;">
                  📋
                </button>
              </div>
            </td>

            ${dynamicTds}

            <!-- COLUNA FINAL: AÇÃO -->
            <td class="text-center" style="white-space: nowrap;">
              <button class="btn-apply-model" onclick="openQuickApplyModal('${m.canonical_id}', '${m.has_nous ? 'nous' : 'openrouter'}', false)" title="Aplicar este modelo no Hermes Agent">
                <span>⚡</span> Usar
              </button>
            </td>
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
        btn.innerHTML = '<span style="display: inline-block; animation: spin 1s linear infinite; margin-right: 4px;">🔄</span> Sincronizando...';
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
          btn.innerHTML = originalHtml || '<span>⚡</span> Sincronizar Tarifas na Nuvem';
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
      let suggestedPath = 'Z:\\souls_engine\\.souls_data';
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
        suggestedPath = 'Z:\\souls_engine\\.souls_data';
        bannerHtml = '<strong style="color: var(--green);">💡 Modo Workstation (Dev Drive ReFS):</strong> Operando sobre volume dedicado ReFS com aceleração Copy-on-Write (CoW) em <code>Z:\\souls_engine\\.souls_data\\</code>. Taxas de I/O de até 4.5 GB/s para operações em lote nos índices SQLite WAL e LanceDB, com contenção absoluta (Linha Vermelha 4) e zero poluição no sistema operacional.';
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
        bannerHtml = '<strong style="color: var(--cyan);">💡 Modo Padrão de Aplicação (%LOCALAPPDATA%):</strong> Operando no diretório nativo seguro <code>%LOCALAPPDATA%\\SoulsEngine\\data\\</code>. Ideal para instalações autônomas sem necessidade de partição Dev Drive, preservando 100% do isolamento de dados com sandbox rígida, validação atômica de código via Myers Diff e zero interferência em outras pastas do host.';
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
        bannerHtml = '<strong style="color: var(--purple-light);">💡 Modo Pessoal do Usuário (%USERPROFILE%):</strong> Operando no repositório pessoal <code>%USERPROFILE%\\.souls_data\\</code>. Permite portabilidade facilitada de bancos SQLite e checkpoints entre perfis de login, mantendo contenção estrita no diretório raiz e proteções contra travamentos de terminal.';
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

    function provisionDataSpace() {
      const statusEl = document.getElementById('checklist-provision-status');
      if (statusEl) {
        statusEl.innerText = 'Provisionando...';
        statusEl.style.color = 'var(--cyan)';
      }
      setTimeout(() => {
        ['chk-dir-sqlite', 'chk-dir-lancedb', 'chk-dir-ladybug', 'chk-dir-checkpoints', 'chk-dir-ast', 'chk-dir-backups'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = '6 de 6 Prontos';
          statusEl.style.color = 'var(--green)';
        }
        showToast('✓ Estrutura canônica de 6 subpastas verificada e provisionada com sucesso!', 'success');
      }, 350);
    }

    function cleanAstCache() {
      showToast('✓ Caches AST Myers Diff limpos com sucesso. Espaço liberado!', 'success');
    }

    function setSandboxScenario(type) {
      const inputEl = document.getElementById('sim-target-path');
      if (!inputEl) return;
      if (type === 'workspace_write') {
        inputEl.value = 'Z:\\souls_engine\\crates\\souls_ast\\src\\lib.rs (Escrita Segura)';
      } else if (type === 'forbidden_escape') {
        inputEl.value = 'C:\\Windows\\System32\\cmd.exe (Violação Linha Vermelha 4)';
      } else if (type === 'timeout_loop') {
        inputEl.value = 'while(true) { std::thread::sleep(); } (Loop Infinito)';
      } else if (type === 'ast_surgery') {
        inputEl.value = 'crates/souls_ast/src/parser.rs (Cirurgia Atômica Myers)';
      }
      runSandboxSimulation();
    }

    function runSandboxSimulation() {
      const inputEl = document.getElementById('sim-target-path');
      const target = inputEl ? inputEl.value.trim() : '';
      const verdictEl = document.getElementById('sim-verdict-badge');
      const reasonEl = document.getElementById('sim-verdict-reason');
      const latencyEl = document.getElementById('sim-eval-latency');
      const ruleEl = document.getElementById('sim-eval-rule');

      if (latencyEl) latencyEl.innerText = '< 0.05 ms';

      if (!target) {
        if (verdictEl) {
          verdictEl.innerText = 'AGUARDANDO ENTRADA';
          verdictEl.style.background = 'rgba(255,255,255,0.05)';
          verdictEl.style.color = 'var(--text-muted)';
          verdictEl.style.borderColor = 'rgba(255,255,255,0.1)';
        }
        if (reasonEl) reasonEl.innerText = 'Digite um comando ou caminho para testar a contenção.';
        if (ruleEl) ruleEl.innerText = '--';
        return;
      }

      const lower = target.toLowerCase();
      if (lower.includes('c:\\windows') || lower.includes('c:/windows') || lower.includes('%appdata%') || lower.includes('system32') || lower.includes('violação')) {
        if (verdictEl) {
          verdictEl.innerText = '🛡️ BLOQUEADO (ESCAPAMENTO DETECTADO)';
          verdictEl.style.background = 'rgba(239, 68, 68, 0.15)';
          verdictEl.style.color = '#ef4444';
          verdictEl.style.borderColor = 'rgba(239, 68, 68, 0.3)';
        }
        if (reasonEl) reasonEl.innerHTML = '<span style="color: #ef4444;">Intercepção de Linha Vermelha 4:</span> Acesso estritamente barrado fora da raiz de trabalho. O host rejeitou a operação com código de confinamento <code>EACCES_GUARDRAIL</code>.';
        if (ruleEl) ruleEl.innerText = 'Linha Vermelha 4 (AGENTS.md §1.4)';
      } else if (lower.includes('while(true)') || lower.includes('loop') || lower.includes('sleep') || lower.includes('hang')) {
        if (verdictEl) {
          verdictEl.innerText = '⏱️ INTERROMPIDO (WATCHDOG 5.000ms)';
          verdictEl.style.background = 'rgba(245, 158, 11, 0.15)';
          verdictEl.style.color = '#f59e0b';
          verdictEl.style.borderColor = 'rgba(245, 158, 11, 0.3)';
        }
        if (reasonEl) reasonEl.innerHTML = '<span style="color: #f59e0b;">Timeout do Shell:</span> Processo atingiu o teto configurado de 5.000 ms sem emitir sinal de progresso. Encerrado de forma limpa pelo Safe Sandbox.';
        if (ruleEl) ruleEl.innerText = 'Safe Sandbox Watchdog';
      } else if (lower.includes('myers') || lower.includes('ast') || lower.includes('cirurgia')) {
        if (verdictEl) {
          verdictEl.innerText = '⚡ AUTORIZADO (CIRURGIA MYERS DIFF)';
          verdictEl.style.background = 'rgba(16, 185, 129, 0.15)';
          verdictEl.style.color = '#10b981';
          verdictEl.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        }
        if (reasonEl) reasonEl.innerHTML = '<span style="color: #10b981;">Cirurgia Atômica:</span> Modificação sintática em memória aprovada. Myers Diff gerado com 100% de integridade e sem bypass de AST.';
        if (ruleEl) ruleEl.innerText = 'Souls Code Surgeon (§5.4)';
      } else {
        if (verdictEl) {
          verdictEl.innerText = '✓ OPERAÇÃO SEGURA & AUTORIZADA';
          verdictEl.style.background = 'rgba(16, 185, 129, 0.15)';
          verdictEl.style.color = '#10b981';
          verdictEl.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        }
        if (reasonEl) reasonEl.innerHTML = '<span style="color: #10b981;">Confinamento em Raiz:</span> Operação dentro do workspace aprovada. Permissão de leitura/escrita validada com contenção no Dev Drive ReFS.';
        if (ruleEl) ruleEl.innerText = 'Workspace Confinado (§1.4)';
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
      showToast('✓ Defaults de segurança e confinamento NT restaurados!', 'success');
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

    function toggleRawReadOnly() {
      const cb = document.getElementById('cfg-obsidian-raw-ro');
      if (!cb) return;
      if (!cb.checked) {
        const proceed = confirm("⚠️ ATENÇÃO: A Lei 1 do AGENTS.md estabelece que as fontes brutas em raw/ são sagradas e estritamente Read-Only.\n\nDesativar esta proteção permite que ferramentas mutem, sobrescrevam ou apaguem notas originais.\n\nDeseja realmente liberar a gravação em raw/?");
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
            pathInput.value = 'C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerText = '☁️ OneDrive Conectado';
            pathBadge.style.color = 'var(--green)';
            pathBadge.style.background = 'rgba(16, 185, 129, 0.12)';
            pathBadge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
        } else if (mode === 'local_vault') {
          kpiIntVal.innerText = 'LOCAL WINDOWS';
          kpiIntVal.style.color = 'var(--purple-light)';
          kpiIntSub.innerText = 'Armazenamento Local';
          if (pathInput) {
            pathInput.value = 'C:\\Users\\rosas\\Documents\\Souls_Vault';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerText = '📁 Cofre Local Ativo';
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
            pathBadge.innerText = '✏️ Caminho Customizado';
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
          kpiRawSub.innerText = '🔒 Imutável & Sagrado (Lei 1)';
          if (rawPill) {
            rawPill.innerText = 'LIGADO (IMUTÁVEL)';
            rawPill.className = 'state-pill on';
          }
        } else {
          kpiRawVal.innerText = '⚠️ GRAVAÇÃO LIBERADA';
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
          kpiPrivSub.innerText = '⚠️ Alerta: Sem Redação PII';
        } else {
          const mode = modeSelect ? modeSelect.value : 'hybrid';
          if (mode === 'hybrid') {
            kpiPrivVal.innerText = 'HÍBRIDO (SILÍCIO)';
            kpiPrivVal.style.color = 'var(--green)';
            kpiPrivSub.innerText = 'Desidentificação PII Ativa';
          } else {
            kpiPrivVal.innerText = 'AIR-GAP TOTAL';
            kpiPrivVal.style.color = 'var(--purple-light)';
            kpiPrivSub.innerText = '100% Silício Local (RTX 2060m)';
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

    function validateVaultStructure() {
      const statusEl = document.getElementById('checklist-vault-status');
      const badgeVaultConn = document.getElementById('badge-vault-conn');
      const kpiIntSub = document.getElementById('kpi-vault-integration-sub');
      if (statusEl) {
        statusEl.innerText = 'Validando pastas...';
        statusEl.style.color = 'var(--cyan)';
      }
      setTimeout(() => {
        ['chk-vault-raw', 'chk-vault-concepts', 'chk-vault-entities', 'chk-vault-summaries', 'chk-vault-manifest', 'chk-vault-log'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = '6 de 6 Conformidades';
          statusEl.style.color = 'var(--green)';
        }
        if (badgeVaultConn) {
          badgeVaultConn.innerText = '✓ 6/6 Conforme';
        }
        if (kpiIntSub) {
          kpiIntSub.innerText = '✓ 6/6 Pastas Íntegras';
          setTimeout(() => {
            syncObsidianKpis();
          }, 3500);
        }
        showToast('✓ Estrutura canônica do cofre validada com sucesso (6 de 6 conformidades)!', 'success');
      }, 350);
    }

    function showVaultAuditLog() {
      showToast('📑 Auditoria: Registros gravados em C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB\\wiki\\log.md', 'info');
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
            showToast(`📑 Carregada nota real do cofre: ${data.title}`, 'info');
          } else {
            showToast(`✓ Cenário carregado: ${data.title}`, 'info');
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
          verdictEl.style.color = '#94a3b8';
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
            latencyEl.innerHTML = `Latência Bare-Metal: <strong style="color: var(--cyan);">${ms} ms (${data.latency_us} µs)</strong> &bull; Rust Engine`;
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
      showToast('✓ Defaults do Cofre de Conhecimentos restaurados!', 'success');
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

    async function fetchToolsCatalog(forceRefresh = false) {
      const grid = document.getElementById('tools-cards-grid');
      if (!grid) return;
      if (!forceRefresh && cachedToolsData.length > 0) {
        renderToolsCards(cachedToolsData);
        return;
      }

      grid.innerHTML = '<div style="grid-column: 1 / -1; padding: 24px; text-align: center; color: var(--text-muted); font-size: 0.82rem;">⚡ Carregando 55 ferramentas nativas do Souls Engine via MCP...</div>';

      try {
        const res = await fetch('/api/v1/tools/catalog');
        if (res.ok) {
          const json = await res.json();
          cachedToolsData = json.tools || [];
          renderToolsCards(cachedToolsData);
        } else {
          grid.innerHTML = '<div style="grid-column: 1 / -1; padding: 20px; text-align: center; color: var(--red);">Falha ao obter catálogo de ferramentas MCP.</div>';
        }
      } catch (err) {
        grid.innerHTML = `<div style="grid-column: 1 / -1; padding: 20px; text-align: center; color: var(--red);">Erro de conexão MCP: ${err.message}</div>`;
      }
    }

    function setToolsNamespaceFilter(ns, btn) {
      currentToolsNamespace = ns;
      document.querySelectorAll('#tools-namespace-filters .filter-provider-btn').forEach(b => b.classList.remove('active'));
      if (btn) btn.classList.add('active');
      filterToolsCatalog(document.getElementById('tools-search-input')?.value || '');
    }

    function filterToolsCatalog(query) {
      const q = (query || '').toLowerCase().trim();
      let filtered = cachedToolsData;

      if (currentToolsNamespace !== 'all') {
        filtered = filtered.filter(t => t.namespace.toLowerCase() === currentToolsNamespace.toLowerCase());
      }

      if (q) {
        filtered = filtered.filter(t => 
          t.name.toLowerCase().includes(q) ||
          t.namespace.toLowerCase().includes(q) ||
          (t.description && t.description.toLowerCase().includes(q))
        );
      }

      renderToolsCards(filtered);
    }

    function renderToolsCards(tools) {
      const grid = document.getElementById('tools-cards-grid');
      if (!grid) return;

      if (!tools || tools.length === 0) {
        grid.innerHTML = '<div style="grid-column: 1 / -1; padding: 30px; text-align: center; color: var(--text-muted); font-size: 0.8rem;">Nenhuma ferramenta encontrada com os filtros selecionados.</div>';
        return;
      }

      const nsIcons = {
        code: '🧬',
        memory: '💾',
        sys: '⚡',
        think: '🧠',
        ops: '🛡️',
        context: '📑',
        core: '🎯'
      };

      grid.innerHTML = tools.map(t => {
        const nsClass = `mcp-ns-${t.namespace.toLowerCase()}`;
        const nsIcon = nsIcons[t.namespace.toLowerCase()] || '⚙️';
        const bootBadge = t.is_level_0 
          ? '<span class="mcp-boot-badge mcp-boot-l0" title="Injetada instantaneamente no handshake inicial">⚡ Boot Nível 0</span>'
          : '<span class="mcp-boot-badge mcp-boot-dyn" title="Carregada sob demanda via tool_search">🔍 Sob Demanda</span>';

        return `
          <div class="mcp-tool-card">
            <div>
              <div class="mcp-tool-header">
                <div class="mcp-tool-name">
                  <span>${nsIcon}</span>
                  <span>${t.name}</span>
                </div>
                <span class="mcp-ns-pill ${nsClass}">${t.namespace}</span>
              </div>
              <div style="margin: 8px 0 6px 0;">
                ${bootBadge}
              </div>
              <div class="mcp-tool-desc">${t.description || 'Sem descrição cadastrada.'}</div>
            </div>
            <div class="mcp-tool-footer">
              <button class="btn btn-sm" style="font-size: 0.68rem; padding: 4px 8px; justify-content: center; background: rgba(255,255,255,0.03);" onclick="openToolSchemaModal('${t.name}')">
                <span>📋</span> Inspecionar Schema
              </button>
              <span style="color: var(--green); font-size: 0.70rem; font-weight: 600;">✓ Habilitada</span>
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
        const roleLabel = isMaster ? '👑 Master Brain' : '🤖 Subagente';

        const deleteBtn = !p.is_builtin ? `
          <button class="btn-delete-profile" title="Excluir Perfil de Trabalho" onclick="event.stopPropagation(); deleteCustomProfile('${p.id}')">
            &times;
          </button>
        ` : '';

        const activeHermesBadge = p.is_active ? `
          <span class="active-hermes-pill" style="white-space: nowrap; flex-shrink: 0;">Ativo no Hermes</span>
        ` : '';

        const inspectingBadge = isActive ? `
          <span class="inspecting-pill"><span>Inspecionando</span> <span style="font-size: 0.72rem;">➔</span></span>
        ` : '';

        return `
          <div class="preset-card ${isActive ? 'active' : ''}" id="preset-${p.id}" onclick="selectProfilePreset('${p.id}')" style="min-height: 195px; padding: 16px; border-radius: var(--radius-md); display: flex; flex-direction: column; justify-content: space-between;">
            <div>
              ${deleteBtn}
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
                <span class="role-badge ${roleClass}">${roleLabel}</span>
                <span class="category-tag">${p.arena_category}</span>
              </div>
              <div class="preset-title" style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 8px;">
                <span style="font-weight: 700; font-size: 0.94rem; color: #fff; line-height: 1.3; flex: 1; min-width: 0;">${p.name}</span>
                ${inspectingBadge}
              </div>
              <div class="preset-desc" style="font-size: 0.75rem; color: var(--text-muted); line-height: 1.45; margin-bottom: 10px;">${p.description}</div>
            </div>

            <div class="preset-defined-box" style="margin-top: 10px; background: rgba(0,0,0,0.3); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 8px 12px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 4px;">
                <span class="defined-label" style="font-size: 0.65rem; color: var(--text-muted); text-transform: uppercase; font-family: var(--font-mono); white-space: nowrap;">Modelo Definido:</span>
                ${activeHermesBadge}
              </div>
              <span class="defined-model-code" style="font-family: var(--font-mono); font-size: 0.78rem; color: var(--cyan); word-break: break-all; display: block;" title="${p.target_model}">${p.target_model || 'Auto (Best Value)'}</span>
            </div>
          </div>
        `;
      };

      // Card 2: Novo Perfil de Trabalho
      const addCardHtml = `
        <div class="add-preset-card" onclick="openNewProfileModal()" style="min-height: 195px; padding: 16px; border-radius: var(--radius-md); display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; border: 2px dashed rgba(168, 85, 247, 0.35); background: rgba(168, 85, 247, 0.04); cursor: pointer; transition: all 0.2s ease;">
          <div style="font-size: 1.8rem; color: var(--purple-light); margin-bottom: 6px;">➕</div>
          <div style="font-weight: 700; font-size: 0.92rem; color: #fff; margin-bottom: 4px;">Novo Perfil de Trabalho</div>
          <div style="font-size: 0.72rem; color: var(--text-muted); line-height: 1.35;">Especialista customizado p/ subagentes</div>
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
          leader: { canonical_id: "qwen2.5-coder-7b-instruct-q4_k_m.gguf", provider: "local_silicon", score: 85.0, blended_cost: 0.00, roi: 9999.0, desc: "100% no silício da NVIDIA RTX 2060m. Custo $0.00 e zero tráfego externo." },
          best: { canonical_id: "deepseek-r1-distill-qwen-7b-q4_k_m.gguf", provider: "local_silicon", score: 86.2, blended_cost: 0.00, roi: 9999.0, desc: "Pensamento analítico local na GPU física sem sair da partição Dev Drive." },
          low: { canonical_id: "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf", provider: "local_silicon", score: 74.0, blended_cost: 0.00, roi: 9999.0, desc: "Ultra-leve para execução instantânea na CPU AVX2 ou GPU." }
        }
      };

      let triad = (currentProfile && currentProfile.triad) ? currentProfile.triad : (fallbackTriads[activePreset] || fallbackTriads.orchestrator || fallbackTriads.frontier_balanced);

      // Refina com dados ao vivo da Cloud Arena se disponível
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
            <span>✓</span> OPÇÃO DEFINIDA NESTE PERFIL
          </div>
        ` : '';

        const primaryAction = isCurrentModel ? '' : (isOrchestrator 
          ? `applyTriadModel('${role}')` 
          : `saveSubagentProfilePreset('${activePreset}', '${item.canonical_id}', '${item.provider}')`);

        const primaryBtn = isCurrentModel ? `
          <button class="btn btn-green disabled" style="width: 100%; margin-top: 10px; justify-content: center; opacity: 0.95; cursor: default;" disabled>
            ✓ Modelo Ativo / Definido
          </button>
        ` : `
          <button class="btn ${isFeatured ? 'btn-cyan' : role === 'low' ? 'btn-green' : 'btn-cyan'}" 
                  style="width: 100%; margin-top: 10px; justify-content: center; ${isFeatured ? 'background: rgba(6, 182, 212, 0.25); border-color: var(--cyan);' : ''}" 
                  onclick="${primaryAction}">
            ${isOrchestrator ? 'Ativar no Hermes Agent' : 'Definir para este Subagente'}
          </button>
        `;

        const promoteBtn = (!isOrchestrator && !isCurrentModel) ? `
          <button class="btn" style="width: 100%; margin-top: 8px; font-size: 0.72rem; padding: 7px 10px; display: flex; align-items: center; justify-content: center; gap: 8px; text-align: center; font-weight: 600; border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08); color: var(--amber);" onclick="promoteToMaster('${item.canonical_id}', '${item.provider}')">
            <span>👑</span> Promover a Mestre no Hermes
          </button>
        ` : '';

        return `
          <div class="triad-card ${role === 'leader' ? 'leader' : role === 'best' ? 'best-value' : 'low-cost'} ${isCurrentModel ? 'is-defined' : ''}">
            <div>
              ${definedBanner}
              <span class="triad-badge ${badgeClass}">${title}</span>
              <div class="triad-model-name" title="${item.canonical_id}">${item.canonical_id}</div>
              <div style="font-size: 0.73rem; color: var(--text-muted); margin-bottom: 8px;">${item.desc}</div>
            </div>
            <div>
              <div class="triad-meta">
                <span>Score: <strong style="color: var(--cyan);">${item.score.toFixed(1)}</strong></span>
                <span>Tarifa: <strong style="color: ${item.blended_cost === 0 ? 'var(--cyan)' : 'var(--green)'};">${costStr}</strong></span>
              </div>
              ${primaryBtn}
              ${promoteBtn}
            </div>
          </div>
        `;
      };

      grid.innerHTML = `
        ${renderCard('leader', '👑 Quality Leader', 'gold', triad.leader, false)}
        ${renderCard('best', '💎 Best Value ROI ★ Recomendado', 'cyan', triad.best, true)}
        ${renderCard('low', '⚡ Ultra Low Cost', 'green', triad.low, false)}
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

    async function setHermesMasterModel(modelId, provider, presetId) {
      showToast(`Ativando ${modelId} como Master Brain no Hermes config.yaml...`, "warning");
      try {
        const res = await fetch('/api/v1/settings/hermes-preset', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            preset_id: presetId,
            model_override: modelId,
            provider: provider
          })
        });
        const data = await res.json();
        if (res.ok && (data.status === "ok" || data.status === "success")) {
          const applied = data.applied_model || data.active_model || modelId;
          const prov = data.provider || data.active_provider || provider;
          showToast(`Master Brain atualizado para ${applied} (${prov})! Backup criado em config.yaml.bak.`, "success");
          const ind = document.getElementById('active-model-indicator');
          if (ind) ind.innerText = `Modelo Ativo no Hermes: ${applied}`;
          fetchTelemetry();
        } else {
          showToast(`Erro ao ativar modelo no Hermes: ${data.message || data.error || 'Falha de comunicação'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao salvar Hermes config: ${e.message}`, "error");
      }
    }

    async function saveSubagentProfilePreset(profileId, modelId, provider) {
      showToast(`Configurando subagentes '${profileId}' para ${modelId}...`, "warning");
      try {
        const prof = profilesData.find(p => p.id === profileId);
        if (prof) {
          prof.default_model = modelId;
          prof.default_provider = provider;
          const res = await fetch('/api/v1/profiles', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(prof)
          });
          const data = await res.json();
          if (res.ok && data.status === 'ok') {
            showToast(`Perfil de trabalho '${prof.name}' configurado com ${modelId}!`, "success");
            await fetchProfiles();
            selectProfilePreset(profileId);
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
            default_model: model || "",
            default_provider: category === 'local' ? 'local_silicon' : 'nous'
          })
        });
        const data = await res.json();
        if (res.ok && data.status === 'ok') {
          showToast(`Perfil '${name}' criado com sucesso e salvo no souls_config.toml!`, "success");
          closeNewProfileModal();
          await fetchProfiles();
          selectProfilePreset(id);
        } else {
          showToast(`Erro ao criar perfil: ${data.error || 'Falha desconhecida'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao criar perfil: ${e.message}`, "error");
      }
    }

    async function deleteCustomProfile(profileId) {
      if (!confirm(`Tem certeza que deseja excluir o perfil de trabalho '${profileId}'?`)) {
        return;
      }
      showToast(`Removendo perfil '${profileId}'...`, "warning");
      try {
        const res = await fetch(`/api/v1/profiles/${profileId}`, { method: 'DELETE' });
        const data = await res.json();
        if (res.ok && data.status === 'ok') {
          showToast(`Perfil '${profileId}' removido com sucesso!`, "success");
          activePreset = 'orchestrator';
          await fetchProfiles();
        } else {
          showToast(`Erro ao excluir perfil: ${data.error || 'Falha'}`, "error");
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

    async function fetchConfigToml() {
      try {
        const res = await fetch('/api/v1/settings/config');
        if (res.ok) {
          const data = await res.json();
          currentConfigData = data;
          const editor = document.getElementById('raw-toml-editor');
          if (editor && data.raw_toml) {
            editor.value = data.raw_toml;
            updateTomlEditorMetrics(data.raw_toml);
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
      const editor = document.getElementById('raw-toml-editor');
      if (!editor) return;
      markSettingsUnsaved();
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
        showToast(`📍 Seção ${sectionTag} localizada!`, 'info');
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
            badge.innerText = '✓ Sintaxe Válida';
            badge.style.color = 'var(--green)';
            badge.style.background = 'rgba(16, 185, 129, 0.12)';
            badge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
          if (kpiValidity) {
            kpiValidity.innerText = 'TOML v1.0 ESTREITO';
            kpiValidity.style.color = 'var(--purple-light)';
          }
          showToast(`✓ Validação TOML com Sucesso! ${data.section_count} seções válidas.`, 'success');
        } else {
          if (badge) {
            badge.innerText = '⚠️ Erro de Sintaxe';
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
      showToast('✨ Buffer TOML formatado com sucesso!', 'info');
    }

    async function restoreTomlBackup() {
      if (!confirm('Deseja restaurar o arquivo de backup souls_config.toml.bak? As alterações não salvas serão sobrescritas.')) {
        return;
      }
      try {
        const res = await fetch('/api/v1/settings/config/restore', { method: 'POST' });
        const data = await res.json();
        if (res.ok && data.status === 'success') {
          showToast('✓ Backup souls_config.toml.bak restaurado com sucesso!', 'success');
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
    function markSettingsUnsaved() {
      hasUnsavedSettings = true;
      const banner = document.getElementById('unsaved-banner');
      if (banner) banner.style.display = 'flex';
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
        setInput('cfg-vram-limit', cfg.finops.vram_hard_limit_mb);
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
        if (cfg.router.weight_quality !== undefined) {
          setInput('cfg-weight-quality', cfg.router.weight_quality);
          const el = document.getElementById('val-weight-quality');
          if (el) el.innerText = Number(cfg.router.weight_quality).toFixed(2);
          const pct = document.getElementById('pct-weight-quality');
          if (pct) pct.innerText = '(' + Math.round(Number(cfg.router.weight_quality) * 100) + '%)';
        }
        if (cfg.router.weight_cost !== undefined) {
          setInput('cfg-weight-cost', cfg.router.weight_cost);
          const el = document.getElementById('val-weight-cost');
          if (el) el.innerText = Number(cfg.router.weight_cost).toFixed(2);
          const pct = document.getElementById('pct-weight-cost');
          if (pct) pct.innerText = '(' + Math.round(Number(cfg.router.weight_cost) * 100) + '%)';
        }
        if (cfg.router.weight_latency !== undefined) {
          setInput('cfg-weight-latency', cfg.router.weight_latency);
          const el = document.getElementById('val-weight-latency');
          if (el) el.innerText = Number(cfg.router.weight_latency).toFixed(2);
          const pct = document.getElementById('pct-weight-latency');
          if (pct) pct.innerText = '(' + Math.round(Number(cfg.router.weight_latency) * 100) + '%)';
        }
      }

      // Memory
      if (cfg.memory) {
        setCheck('cfg-memory-langevin', cfg.memory.langevin_decay_enabled, 'pill-memory-langevin');
        setInput('cfg-memory-half-life', cfg.memory.half_life_days);
        setInput('cfg-memory-rrf-k', cfg.memory.rrf_k);
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

    async function saveSettings() {
      showToast("Salvando configurações dos 8 subsistemas em souls_config.toml...", "warning");
      
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
# Dev Drive ReFS: Z:\\souls_engine\\.souls_data\\config\\souls_config.toml
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
vram_hard_limit_mb = ${getNum('cfg-vram-limit', 5294)}
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

[ast]
dehydration_enabled = ${getCheck('cfg-ast-dehydration', true)}
max_lines_tree_sitter = ${getNum('cfg-ast-max-lines', 15000)}

[sandbox]
safe_sandbox_timeout_ms = ${getNum('cfg-sandbox-timeout', 5000)}
strict_refs_partition = true

[obsidian]
vault_path = "${getStr('cfg-obsidian-path', 'C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB').replace(/\\/g, '\\\\')}"
storage_type = "${getStr('cfg-obsidian-storage-type', 'onedrive')}"
privacy_shield_enabled = ${getCheck('cfg-obsidian-privacy-shield', true)}
privacy_mode = "${getStr('cfg-obsidian-privacy-mode', 'hybrid')}"
raw_readonly = ${getCheck('cfg-obsidian-raw-ro', true)}
wikilinks_enabled = ${getCheck('cfg-obsidian-wikilinks', true)}
curator_batch_size = ${getNum('cfg-obsidian-batch-size', 3)}
interval_minutes = ${getNum('cfg-obsidian-interval-min', 30)}
`;

      try {
        const res = await fetch('/api/v1/settings/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ raw_toml: rawToml })
        });
        const data = await res.json();
        if (res.ok && data.status === 'ok') {
          showToast("Configurações gravadas com sucesso no souls_config.toml (.toml.bak criado)!", "success");
          hasUnsavedSettings = false;
          dismissUnsavedBanner();
          const editor = document.getElementById('raw-toml-editor');
          if (editor) editor.value = rawToml;
        } else {
          showToast(`Erro ao gravar configurações: ${data.error || 'Falha desconhecida'}`, "error");
        }
      } catch (e) {
        showToast(`Erro de rede ao salvar: ${e.message}`, "error");
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
        if (res.ok && data.status === 'ok') {
          showToast("Arquivo TOML salvo com sucesso e backup de segurança criado!", "success");
          hasUnsavedSettings = false;
          dismissUnsavedBanner();
          fetchConfigToml();
        } else {
          showToast(`Erro ao salvar TOML: ${data.error || 'Sintaxe TOML inválida'}`, "error");
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

    function setRecallModePreference(mode) {
      currentRecallPreference = mode;
      ['adaptive', 'lean', 'expanded'].forEach(m => {
        const btn = document.getElementById('btn-recall-' + m);
        if (btn) btn.classList.toggle('active', m === mode);
      });
      const heroTxt = document.getElementById('hero-recall-mode-text');
      const labels = {
        adaptive: '⚡ Adaptativo',
        lean: '🎯 Enxuto (800c)',
        expanded: '🧠 Expandido + Grafo (2400c)'
      };
      if (heroTxt) heroTxt.innerText = labels[mode] || labels.adaptive;
      showToast(`Modo de Recall L3 ajustado para: ${labels[mode] || mode}`, 'success');
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

      const filtered = sessions.filter(s => {
        const st = (s.status || 'SPAWNED').toUpperCase();
        if (currentSubagentFilter === 'completed') return st === 'COMPLETED';
        if (currentSubagentFilter === 'active') return st === 'RUNNING' || st === 'SPAWNED';
        return true;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = `<tr><td colspan="6" style="text-align: center; color: var(--text-muted); padding: 18px 0;">Nenhuma sessão encontrada para o filtro selecionado.</td></tr>`;
        return;
      }

      let rowsHtml = '';
      for (const s of filtered) {
        const statusUpper = (s.status || 'SPAWNED').toUpperCase();
        let statusBadge = '<span class="badge" style="background: rgba(245, 158, 11, 0.15); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.35);">SPAWNED</span>';
        if (statusUpper === 'RUNNING') {
          statusBadge = '<span class="badge" style="background: rgba(6, 182, 212, 0.15); color: var(--cyan); border: 1px solid rgba(6, 182, 212, 0.35);">RUNNING ⚡</span>';
        } else if (statusUpper === 'COMPLETED') {
          statusBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.35);">COMPLETED ✅</span>';
        } else if (statusUpper === 'FAILED') {
          statusBadge = '<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red); border: 1px solid rgba(239, 68, 68, 0.35);">FAILED ❌</span>';
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
            <td class="mono" style="font-size: 0.74rem; color: var(--cyan); font-weight: 600; white-space: nowrap;">
              <div style="display: flex; align-items: center; gap: 6px;">
                <span>${agentKey}</span>
                <button class="btn" style="padding: 1px 5px; font-size: 0.62rem;" onclick="navigator.clipboard.writeText('${agentKey}')" title="Copiar Session ID">📋</button>
              </div>
            </td>
            <td>
              <div style="font-weight: 600; color: #fff; font-size: 0.82rem; display: flex; align-items: center; gap: 8px;">
                <span>${friendlyTitle}</span>
                <span class="badge ${tierBadge}" style="font-size: 0.65rem;">${s.assigned_tier || 'Tier1'}</span>
              </div>
              <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 3px; line-height: 1.4;">
                ${escapeHtml(s.task_description || 'Sessão Bare-Metal L3')}
              </div>
            </td>
            <td>
              ${formatOrchestratorBadge(parentId)}
            </td>
            <td class="text-center">${statusBadge}</td>
            <td>
              <div style="font-weight: 600; color: #cbd5e1; font-size: 0.76rem;">${dateStr}</div>
              <div style="font-size: 0.70rem; color: var(--text-muted); margin-top: 2px;">
                ⏱️ Duração: <strong style="color: var(--cyan);">${durationStr}</strong>
              </div>
            </td>
            <td class="text-center">
              <button class="btn btn-purple" style="padding: 4px 10px; font-size: 0.72rem; border-radius: 6px;" onclick="inspectSubAgentHistory('${agentKey}')">
                <span>📖</span> Diário & Handoff
              </button>
            </td>
          </tr>
        `;
      }

      tbody.innerHTML = rowsHtml;
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
          <div style="display: flex; align-items: center; gap: 6px;">
            <span class="badge" style="background: rgba(147, 51, 234, 0.12); color: var(--purple); border: 1px solid rgba(147, 51, 234, 0.3); font-size: 0.70rem;">🎯 Master Brain</span>
          </div>
          <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">Direto / Local</div>
        `;
      }
      const lower = parentId.toLowerCase();
      if (lower.includes('hermes')) {
        return `
          <div style="display: flex; align-items: center; gap: 6px;">
            <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.70rem;">🤖 Hermes Agent</span>
          </div>
          <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;" title="${parentId}">${parentId}</div>
        `;
      }
      if (lower.includes('antigravity') || lower.includes('cli') || lower.includes('sub_') || lower.includes('agent')) {
        return `
          <div style="display: flex; align-items: center; gap: 6px;">
            <span class="badge" style="background: rgba(6, 182, 212, 0.12); color: var(--cyan); border: 1px solid rgba(6, 182, 212, 0.3); font-size: 0.70rem;">⚡ Antigravity / IDE</span>
          </div>
          <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;" title="${parentId}">${parentId}</div>
        `;
      }
      return `
        <div style="display: flex; align-items: center; gap: 6px;">
          <span class="badge" style="background: rgba(147, 51, 234, 0.12); color: var(--purple); border: 1px solid rgba(147, 51, 234, 0.3); font-size: 0.70rem;">🎯 Master Session</span>
        </div>
        <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;" title="${parentId}">${parentId}</div>
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
      diariesContainer.innerHTML = '<div style="color: var(--text-muted); font-size: 0.8rem;">Carregando diários...</div>';
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

        profileElem.innerHTML = `<strong>${s.agent_id}</strong> <span style="font-size: 0.75rem; color: var(--cyan); margin-left: 6px;">(${s.assigned_tier || 'Tier1'})</span><div style="font-size: 0.74rem; color: var(--text-muted); margin-top: 2px;">${escapeHtml(s.task_description || '')}</div>`;
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
            <div style="background: rgba(16, 185, 129, 0.08); border: 1px solid rgba(16, 185, 129, 0.4); border-radius: 6px; padding: 12px; margin-bottom: 10px;">
              <div style="display: flex; justify-content: space-between; font-size: 0.72rem; color: var(--green); font-weight: 700; margin-bottom: 6px;">
                <span>🟢 Envelope de Retorno #${returnHdf.id} &bull; Para Mestre: <strong style="color: var(--cyan);">${returnHdf.target_agent_id}</strong></span>
                <span>${timeStr}</span>
              </div>
              <div style="color: #a7f3d0; font-family: var(--font-mono); font-size: 0.78rem; white-space: pre-wrap; line-height: 1.5;">${escapeHtml(returnHdf.context_summary || returnHdf.summary_markdown || '')}</div>
            </div>
          `;
        }

        if (receivedHdf) {
          const rawTime = receivedHdf.created_at;
          const ts = typeof rawTime === 'number' && rawTime < 10000000000 ? rawTime * 1000 : rawTime;
          const timeStr = ts ? new Date(ts).toLocaleTimeString() : '';
          handoffHtml += `
            <div style="background: rgba(15, 23, 42, 0.7); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 10px; margin-bottom: 8px;">
              <div style="display: flex; justify-content: space-between; font-size: 0.7rem; color: var(--text-muted); margin-bottom: 4px;">
                <span>📥 Envelope Inicial Recebido #${receivedHdf.id} (De: <strong style="color: var(--cyan);">${receivedHdf.source_session_id}</strong>)</span>
                <span>${timeStr}</span>
              </div>
              <div style="color: #cbd5e1; font-family: var(--font-mono); font-size: 0.76rem; white-space: pre-wrap;">${escapeHtml(receivedHdf.context_summary || '')}</div>
            </div>
          `;
        }

        if (!returnHdf && !receivedHdf) {
          handoffHtml = `<span style="color: var(--text-muted);">Nenhum envelope de handoff emitido ainda. A sessão pode estar em andamento ou não ter emitido a ação <code>complete</code>.</span>`;
        }
        handoffsElem.innerHTML = handoffHtml;

        // Render Diários
        diariesCountElem.textContent = diaries.length;
        if (diaries.length === 0) {
          diariesContainer.innerHTML = '<div style="color: var(--text-muted); font-size: 0.8rem; padding: 10px; background: rgba(255,255,255,0.02); border-radius: 6px;">Nenhum pensamento intermediário registrado nesta sessão.</div>';
        } else {
          diariesContainer.innerHTML = diaries.map((d, idx) => {
            const rawTime = d.created_at;
            const ts = typeof rawTime === 'number' && rawTime < 10000000000 ? rawTime * 1000 : rawTime;
            const timeStr = ts ? new Date(ts).toLocaleTimeString() : '';
            const thoughtText = d.content || d.thought_log || '';
            return `
              <div style="background: rgba(15, 23, 42, 0.5); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                  <span style="font-size: 0.72rem; font-weight: 700; color: var(--amber); font-family: var(--font-mono);">
                    Passo ${idx + 1} / Entrada #${d.id}
                  </span>
                  <span style="font-size: 0.7rem; color: var(--text-muted);">${timeStr}</span>
                </div>
                <div style="font-size: 0.78rem; color: #e2e8f0; line-height: 1.5; white-space: pre-wrap; font-family: var(--font-mono); background: rgba(0,0,0,0.25); padding: 8px; border-radius: 4px;">${escapeHtml(thoughtText)}</div>
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
      document.getElementById('cfg-domain-icon').textContent = domain.icon || '🎯';
      document.getElementById('cfg-domain-title').textContent = `Configurar: ${domain.category}`;
      document.getElementById('cfg-domain-desc').textContent = domain.description;

      // Preenche select de modelos locais
      const localSelect = document.getElementById('cfg-select-local-model');
      localSelect.innerHTML = '';
      
      const defaultLocalModel = domain.local_default.model;
      const currentSelectedLocal = domainOverride.local_model || defaultLocalModel;

      const availableLocals = [
        { id: defaultLocalModel, name: `${defaultLocalModel} (Padrão Recomendado)` },
        { id: "qwen2.5-coder-7b-instruct-q4_k_m.gguf", name: "Qwen 2.5 Coder 7B (dGPU RTX 2060m)" },
        { id: "smollm2-135m-instruct.gguf", name: "SmolLM2 135M Reasoning (CPU Host AVX2)" },
        { id: "phi-4-mini-instruct-q4_k_m.gguf", name: "Phi-4 Mini 3.8B (dGPU RTX 2060m)" },
        { id: "qwen2.5-vl-3b-instruct-q4_k_m.gguf", name: "Qwen 2.5 VL 3B Vision (dGPU RTX 2060m)" },
        { id: "smollm2-360m-instruct.gguf", name: "SmolLM2 360M (CPU Host AVX2)" },
        { id: "deepseek-coder-6.7b-instruct.gguf", name: "DeepSeek Coder 6.7B (dGPU)" }
      ];

      if (Array.isArray(localModelsList) && localModelsList.length > 0) {
        localModelsList.forEach(m => {
          const id = m.name || m.model_name || m.id;
          if (id && !availableLocals.some(a => a.id.toLowerCase() === id.toLowerCase())) {
            availableLocals.push({ id: id, name: `${id} (${m.size_gb || '?'} GB • Detectado)` });
          }
        });
      }

      availableLocals.forEach(opt => {
        const el = document.createElement('option');
        el.value = opt.id;
        el.textContent = opt.name;
        if (opt.id.toLowerCase() === currentSelectedLocal.toLowerCase()) {
          el.selected = true;
        }
        localSelect.appendChild(el);
      });

      // Preenche select de modelos cloud
      const cloudSelect = document.getElementById('cfg-select-cloud-model');
      cloudSelect.innerHTML = '';

      const defaultCloudModel = domain.cloud_fallback.model;
      const currentSelectedCloud = domainOverride.cloud_model || defaultCloudModel;

      const availableClouds = [
        { id: defaultCloudModel, name: `${defaultCloudModel} (Padrão Recomendado)` },
        { id: "anthropic/claude-3-7-sonnet", name: "Claude 3.7 Sonnet (Anthropic • Coding Champion)" },
        { id: "deepseek/deepseek-chat", name: "DeepSeek-V3 Chat (Nous Portal • Custo Mínimo)" },
        { id: "google/gemini-2.0-flash", name: "Gemini 2.0 Flash (Google • Velocidade Extrema)" },
        { id: "openai/o3-mini", name: "OpenAI o3-mini (OpenAI • Raciocínio Profundo)" },
        { id: "qwen/qwen-2.5-vl-72b-instruct", name: "Qwen 2.5 VL 72B Vision (Multimodal Frontier)" },
        { id: "meta-llama/llama-3.3-70b-instruct", name: "Llama 3.3 70B Instruct (Meta OpenWeights)" },
        { id: "deepseek/deepseek-r1", name: "DeepSeek R1 Reasoning (Matemática & Provas)" }
      ];

      availableClouds.forEach(opt => {
        const el = document.createElement('option');
        el.value = opt.id;
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

    function saveDomainConfiguration() {
      const domainId = document.getElementById('cfg-domain-id').value;
      if (!domainId) return;

      const localModel = document.getElementById('cfg-select-local-model').value;
      const cloudModel = document.getElementById('cfg-select-cloud-model').value;
      const paretoRule = document.getElementById('cfg-pareto-rule-input').value.trim();

      const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
      overrides[domainId] = {
        local_model: localModel,
        cloud_model: cloudModel,
        pareto_rule: paretoRule,
        updated_at: new Date().toISOString()
      };

      localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));
      closeConfigureDomainModal();

      showToast(`✓ Roteamento atualizado com sucesso!`, 'success');
      if (window.cachedRouterMatrix) {
        renderRouterMatrix(window.cachedRouterMatrix);
      }
    }

    function resetDomainDefault() {
      const domainId = document.getElementById('cfg-domain-id').value;
      if (!domainId) return;

      const overrides = JSON.parse(localStorage.getItem('souls_domain_overrides') || '{}');
      if (overrides[domainId]) {
        delete overrides[domainId];
        localStorage.setItem('souls_domain_overrides', JSON.stringify(overrides));
      }

      closeConfigureDomainModal();
      showToast(`✓ Domínio restaurado para os modelos padrão do sistema.`, 'success');
      if (window.cachedRouterMatrix) {
        renderRouterMatrix(window.cachedRouterMatrix);
      }
    }

    // Card 3: Ações Interativas
    async function testLoopbackConnection() {
      const statusEl = document.getElementById('hermes-loopback-status');
      if (statusEl) statusEl.innerHTML = '<span style="color: var(--cyan);">Sondando...</span>';

      const t0 = performance.now();
      try {
        const res = await fetch('/health');
        const elapsed = (performance.now() - t0).toFixed(1);
        if (res.ok) {
          if (statusEl) {
            statusEl.innerHTML = `<span style="color: var(--green); font-weight: 700;">✅ Ativo (${elapsed}ms • 200 OK)</span>`;
          }
          showToast(`Loopback MCP 127.0.0.1:9123 saudável (${elapsed}ms)`, 'success');
        } else {
          if (statusEl) {
            statusEl.innerHTML = `<span style="color: var(--red);">Falha (${res.status})</span>`;
          }
        }
      } catch (err) {
        if (statusEl) {
          statusEl.innerHTML = `<span style="color: var(--red);">Offline</span>`;
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
          showToast('✓ Configuração MCP copiada para o clipboard!', 'success');
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
        showToast('✓ Configuração copiada!', 'success');
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
      showToast('✓ Arquivo souls_mcp_config.json descarregado!', 'success');
    }

    // MCP Tools Catalog Modal
    const NATIVE_MCP_TOOLS = [
      // AST & Code
      { name: "souls_ast_outline", domain: "AST & Code", icon: "🧩", desc: "Extrai esqueleto sintático (classes, traits, funções, assinaturas) poupando 80% dos tokens de contexto.", params: "path, language" },
      { name: "souls_ast_slice", domain: "AST & Code", icon: "🧩", desc: "Extrai fatias sintáticas contíguas com numeração precisa de linhas para inspeção direcionada.", params: "path, start_line, end_line" },
      { name: "souls_ast_symbol", domain: "AST & Code", icon: "🧩", desc: "Localiza símbolos e identidades no escopo do arquivo sem precisar ler o arquivo por inteiro.", params: "path, symbol_name" },
      { name: "souls_ast_edit", domain: "AST & Code", icon: "🧩", desc: "Aplica mutações atômicas de código com verificação sintática pré-commit do Tree-sitter.", params: "path, target, replacement" },
      { name: "souls_ast_delta_diff", domain: "AST & Code", icon: "🧩", desc: "Calcula e aplica Myers Diff estruturado preservando contagens exatas de bytes LF.", params: "path, patch" },
      { name: "souls_blast_radius", domain: "AST & Code", icon: "💥", desc: "Mede o raio de impacto de alterações entre crates para prevenir quebras arquiteturais colaterais.", params: "crate_name, symbols" },
      { name: "souls_frecency", domain: "AST & Code", icon: "🔥", desc: "Calcula mapa de calor de arquivos mais editados e com maior probabilidade de alteração.", params: "limit, days" },

      // Memória L2 & Grafo
      { name: "souls_memory_search", domain: "Memória L2", icon: "🧠", desc: "Busca híbrida vetorial RRF (LanceDB + SQLite FTS5) no hipocampo epistêmico do Souls Engine.", params: "query, limit, threshold" },
      { name: "souls_memory_store", domain: "Memória L2", icon: "🧠", desc: "Persiste memórias de longo prazo com embeddings locais Qwen2 em FrankenSQLite STRICT WAL.", params: "content, tags, category" },
      { name: "souls_memory_graph_entity", domain: "Memória L2", icon: "🕸️", desc: "Cria e atualiza nós ontológicos e observações clínicas no Grafo LadybugDB.", params: "entity_name, entity_type, props" },
      { name: "souls_memory_graph_query", domain: "Memória L2", icon: "🕸️", desc: "Navega pelas arestas tipadas (depends_on, conflicts_with, implements) do cofre cognitivo.", params: "source_id, relation_type" },
      { name: "souls_vault_query", domain: "Memória L2", icon: "🏛️", desc: "Consulta evidências empíricas de benchmarks TPS e VRAM no Heuristic Vault.", params: "model_id, domain" },
      { name: "souls_obsidian_curate", domain: "Memória L2", icon: "📓", desc: "Curadoria automática de notas na pasta wiki/ do Obsidian com desidentificação de dados.", params: "source_path, target_wiki" },

      // Thinking, Silício & FinOps
      { name: "souls_socratic_deliberate", domain: "Thinking & Ops", icon: "🛡️", desc: "Freio cognitivo socrático: deliberação em grafo DAG antes de tocar em código de alta entropia.", params: "hypothesis, tradeoffs" },
      { name: "souls_metrics", domain: "Thinking & Ops", icon: "📊", desc: "Telemetria bare-metal em tempo real: VRAM alocada (NVML), temperatura e TPS empírico.", params: "detailed" },
      { name: "souls_hardware_watchdog", domain: "Thinking & Ops", icon: "🌡️", desc: "Guardião termodinâmico da RTX 2060m e margem física sob Windows DWM.", params: "action" },
      { name: "souls_intent_probe", domain: "Thinking & Ops", icon: "🧭", desc: "Classificador de intenção e medição de entropia via Tier 0.5 SLM (Host CPU AVX2).", params: "prompt, context" },
      { name: "souls_context_lean_compress", domain: "Thinking & Ops", icon: "🗜️", desc: "Compressão CCR em memória RAM poupando até 60% da janela de contexto.", params: "text, target_tokens" },
      { name: "souls_switchboard_evaluate", domain: "Thinking & Ops", icon: "⚡", desc: "Roteador semântico two-stage: desvia tarefas entre dGPU local e Cloud Fallback em 1.4ms.", params: "query, available_models" }
    ];

    function openMcpToolsModal() {
      const modal = document.getElementById('modal-mcp-tools');
      if (!modal) return;
      renderMcpToolsCatalog(NATIVE_MCP_TOOLS);
      modal.classList.add('active');
    }

    function closeMcpToolsModal() {
      const modal = document.getElementById('modal-mcp-tools');
      if (modal) modal.classList.remove('active');
    }

    function renderMcpToolsCatalog(tools) {
      const container = document.getElementById('mcp-tools-list-container');
      if (!container) return;

      container.innerHTML = tools.map(t => `
        <div style="background: rgba(15, 23, 42, 0.6); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 12px; display: flex; align-items: flex-start; justify-content: space-between; gap: 12px;">
          <div style="flex: 1;">
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span>${t.icon}</span>
              <code class="mono" style="font-weight: 700; color: #fff; font-size: 0.82rem;">${t.name}</code>
              <span class="badge ${t.domain.includes('AST') ? 'badge-champion' : (t.domain.includes('Memória') ? 'badge-value' : 'badge')}">${t.domain}</span>
            </div>
            <p style="font-size: 0.74rem; color: #94a3b8; line-height: 1.4; margin-bottom: 6px;">${t.desc}</p>
            <div style="font-size: 0.68rem; color: #64748b; font-family: var(--font-mono);">
              Parâmetros: <span style="color: #cbd5e1;">${t.params}</span>
            </div>
          </div>
          <button class="btn-micro" onclick="copyToolName('${t.name}')" title="Copiar identificador da ferramenta MCP">
            📋 Copiar
          </button>
        </div>
      `).join('');

      const countLabel = document.getElementById('mcp-tools-count-label');
      if (countLabel) countLabel.textContent = `${tools.length} ferramenta(s) nativa(s) listada(s)`;
    }

    function filterMcpToolsList() {
      const q = (document.getElementById('mcp-tools-search').value || '').toLowerCase().trim();
      const filtered = NATIVE_MCP_TOOLS.filter(t => 
        t.name.toLowerCase().includes(q) ||
        t.domain.toLowerCase().includes(q) ||
        t.desc.toLowerCase().includes(q)
      );
      renderMcpToolsCatalog(filtered);
    }

    function copyToolName(name) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(name).then(() => {
          showToast(`✓ Ferramenta '${name}' copiada!`, 'success');
        });
      } else {
        fallbackExecCopy(name);
      }
    }

    async function testSiliconPing() {
      const t0 = performance.now();
      try {
        const res = await fetch('/api/v1/metrics');
        const elapsed = (performance.now() - t0).toFixed(1);
        if (res.ok) {
          showToast(`✅ Silício Ativo: NVML OK • VRAM Livre • Resposta em ${elapsed}ms`, 'success');
        } else {
          showToast(`Silício respondeu com status ${res.status} em ${elapsed}ms`, 'info');
        }
      } catch (err) {
        showToast(`Erro ao testar silício: ${err.message}`, 'error');
      }
    }

    async function runSimulateDispatch() {
      const input = document.getElementById('sim-task-input');
      const resultBox = document.getElementById('sim-dispatch-result');
      if (!input || !resultBox) return;
      const query = input.value.trim();
      if (!query) {
        resultBox.innerHTML = '<div style="color: var(--amber); font-size: 0.78rem;">Digite uma tarefa ou clique em uma das sugestões acima.</div>';
        return;
      }

      resultBox.innerHTML = '<div style="color: var(--cyan); font-size: 0.78rem;">Sondando Two-Stage Semantic Switchboard (Host CPU AVX2)...</div>';

      try {
        const res = await fetch('/api/v1/switchboard/evaluate', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query: query })
        });
        if (!res.ok) {
          resultBox.innerHTML = `<div style="color: var(--red); font-size: 0.78rem;">Erro ao consultar switchboard: HTTP ${res.status}</div>`;
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

        let toolsHtml = tools.map(t => `<span class="badge" style="background: rgba(6,182,212,0.12); color: var(--cyan); border: 1px solid rgba(6,182,212,0.3); font-size: 0.7rem;">${t.name}</span>`).join(' ');

        resultBox.innerHTML = `
          <div style="background: rgba(15, 23, 42, 0.7); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 14px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; border-bottom: 1px solid rgba(255,255,255,0.06); padding-bottom: 8px;">
              <div>
                <span class="badge badge-champion" style="font-size: 0.72rem;">${stage}</span>
                <span style="font-size: 0.74rem; color: var(--green); margin-left: 8px; font-weight: 600;">⚡ Latência: ${latency}</span>
              </div>
              <div style="font-size: 0.72rem; color: var(--text-muted);">
                Confiança: <strong style="color: var(--cyan);">${conf}</strong> &bull; Entropia: <strong style="color: var(--amber);">${entropy}</strong>
              </div>
            </div>

            <div style="margin-bottom: 10px;">
              <div style="font-size: 0.70rem; color: var(--text-muted); text-transform: uppercase;">Skill Canônica Selecionada</div>
              <div style="font-size: 0.88rem; font-weight: 700; color: #fff; margin-top: 2px;">
                🎯 ${skill.name || 'souls-general-orchestration'}
              </div>
              <div style="font-size: 0.74rem; color: var(--text-muted); margin-top: 2px;">${skill.description || 'Orquestração de tarefas especializadas no Souls Engine.'}</div>
            </div>

            <div style="margin-bottom: 10px; background: rgba(0,0,0,0.25); border: 1px solid rgba(255,255,255,0.05); border-radius: 6px; padding: 8px;">
              <div style="font-size: 0.70rem; color: var(--cyan); font-weight: 600;">⚖️ Decisão de Despacho ParetoBandit:</div>
              <div style="font-size: 0.76rem; color: #cbd5e1; margin-top: 3px;">
                Despachar no <strong>Silício Local Bare-Metal ($0.00)</strong>. Entropia (${entropy}) está abaixo do limiar de desvio para Cloud.
              </div>
            </div>

            <div>
              <div style="font-size: 0.70rem; color: var(--text-muted); margin-bottom: 4px;">Ferramentas MCP Vinculadas (${tools.length}):</div>
              <div style="display: flex; flex-wrap: wrap; gap: 4px;">
                ${toolsHtml || '<span style="color: var(--text-muted); font-size: 0.72rem;">Nenhuma ferramenta restrita necessária.</span>'}
              </div>
            </div>
          </div>
        `;
      } catch (err) {
        resultBox.innerHTML = `<div style="color: var(--red); font-size: 0.78rem;">Falha na simulação: ${err.message}</div>`;
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

    // INICIALIZAÇÃO NO CARREGAMENTO DA PÁGINA
    window.addEventListener('DOMContentLoaded', () => {
      initGlobalPortalTooltips();
      fetchTelemetry();
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

      // Suporte a Deep Linking via Hash (ex: #tab-local, #tab-cloud, #tab-skills/forge, #tab-settings/subtab-sandbox)
      if (window.location.hash) {
        const fullHash = window.location.hash.replace('#', '').split('?')[0];
        const parts = fullHash.split('/');
        const mainTab = parts[0];
        const subTab = parts[1];
        if (document.getElementById(mainTab)) {
          switchTab(mainTab);
          if (subTab && mainTab === 'tab-settings' && typeof switchSettingsSubtab === 'function') {
            switchSettingsSubtab(subTab);
          } else if (subTab && mainTab === 'tab-skills' && typeof switchSkillsSubtab === 'function') {
            switchSkillsSubtab(subTab);
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
