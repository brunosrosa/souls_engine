// connections.js — Logica da Aba de Conexoes Globais e Harnesses Agenticos (v0.8.1)

async function fetchConnectionsData() {
  try {
    const [clisRes, provRes] = await Promise.all([
      fetch('/api/v1/connections/clis'),
      fetch('/api/v1/connections/providers')
    ]);

    if (clisRes.ok) {
      const data = await clisRes.json();
      renderClis(data.clis || []);
    }

    if (provRes.ok) {
      const data = await provRes.json();
      renderProviders(data.providers || []);
    }

    checkGoogleSession();
    if (typeof loadGoogleConnectionsVault === 'function') {
      loadGoogleConnectionsVault();
    }
  } catch (err) {
    console.error('Falha ao carregar conexões:', err);
  }
}

function connEsc(raw) {
  if (typeof window.escapeHtml === 'function') return window.escapeHtml(raw);
  return String(raw ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function renderClis(clis) {
  const container = document.getElementById('clis-grid');
  const badgeTop = document.getElementById('badge-clis-count');
  const badgeSection = document.getElementById('clis-count-badge');

  if (!container) return;

  const installedCount = clis.filter(c => c.is_installed).length;
  if (badgeTop) badgeTop.textContent = `${installedCount}/${clis.length}`;
  if (badgeSection) badgeSection.textContent = `${installedCount} de ${clis.length} Instalados`;

  if (clis.length === 0) {
    container.innerHTML = '<div class="u-text-muted u-s-801160">Nenhum assistente local catalogado.</div>';
    return;
  }

  container.innerHTML = clis.map(cli => {
    const isInst = cli.is_installed;
    const statusText = isInst ? 'Pronto no PC' : 'Não Instalado';
    const badgeCls = isInst ? 'badge-value' : 'badge-dominated';
    const versionDisplay = cli.version ? `v${connEsc(cli.version)}` : (isInst ? 'Instalado' : '--');
    const pathDisplay = cli.resolved_path ? connEsc(cli.resolved_path) : 'Não encontrado no PATH do Windows';

    const modesBadges = (cli.supported_modes || []).map(m =>
      `<span class="u-text-slate u-mono u-s-e34342">${connEsc(m)}</span>`
    ).join(' ');

    return `
      <div class="conn-cli-card ${isInst ? 'is-ready' : ''}">
        <div class="conn-cli-head">
          <div>
            <h4 class="conn-cli-title u-m-0">${connEsc(cli.display_name)}</h4>
            <div class="u-mono u-text-cyan gh-text-xs">${connEsc(cli.binary_name)}</div>
          </div>
          <span class="badge ${badgeCls}">${statusText}</span>
        </div>

        <div class="u-mb-2 u-d-flex u-justify-between gh-text-xs">
          <span class="u-text-muted">Versão:</span>
          <strong class="u-text-white">${versionDisplay}</strong>
        </div>

        <div class="u-mb-2 conn-cli-meta u-mono">${pathDisplay}</div>

        <div class="u-text-muted u-mb-1 gh-text-xs">Recursos:</div>
        <div class="u-d-flex u-gap-6p u-flex-wrap">
          ${modesBadges}
        </div>
      </div>
    `;
  }).join('');
}

let cachedProviders = [];
let activeProviderFilter = 'all';
let activeProviderSearch = '';

function setProviderFilter(filter, btn) {
  activeProviderFilter = filter || 'all';
  document.querySelectorAll('#providers-filter-bar .filter-provider-btn').forEach((el) => el.classList.remove('active'));
  if (btn) btn.classList.add('active');
  renderProviders(cachedProviders);
}
window.setProviderFilter = setProviderFilter;

function setProviderSearch(query) {
  activeProviderSearch = String(query || '').trim().toLowerCase();
  renderProviders(cachedProviders);
}
window.setProviderSearch = setProviderSearch;

async function saveProviderInlineKey(slug, envKey) {
  const inputId = `prov-key-${slug}`;
  if (typeof window.saveInlineApiKey === 'function') {
    await window.saveInlineApiKey(envKey, inputId);
    return;
  }
  const input = document.getElementById(inputId);
  const val = input?.value?.trim() || '';
  if (!val || val.length < 6) {
    if (typeof showToast === 'function') showToast(`Cole uma chave válida para ${envKey}.`, 'warning');
    return;
  }
  try {
    const res = await fetch('/api/v1/system/credentials/save', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key: envKey, value: val })
    });
    const data = await res.json();
    if (res.ok) {
      if (input) input.value = '';
      if (typeof showToast === 'function') showToast(`${envKey} salva e ativada (${data.masked || '***'})!`, 'success');
      fetchConnectionsData();
    } else if (typeof showToast === 'function') {
      showToast(data.message || `Falha ao salvar ${envKey}.`, 'error');
    }
  } catch (err) {
    if (typeof showToast === 'function') showToast(`Erro ao salvar ${envKey}: ${err.message}`, 'error');
  }
}
window.saveProviderInlineKey = saveProviderInlineKey;

function renderProviders(providers) {
  cachedProviders = Array.isArray(providers) ? providers : [];
  const container = document.getElementById('providers-grid');
  const badgeSection = document.getElementById('providers-count-badge');

  if (!container) return;

  const configuredCount = cachedProviders.filter((p) => p.is_configured).length;
  if (badgeSection) {
    badgeSection.textContent = `${configuredCount} de ${cachedProviders.length} Conectados`;
  }

  const filtered = cachedProviders.filter((p) => {
    if (activeProviderFilter === 'connected' && !p.is_configured) return false;
    if (activeProviderFilter === 'free' && !p.has_free_tier) return false;
    if (activeProviderFilter === 'missing' && p.is_configured) return false;
    if (activeProviderSearch) {
      const hay = `${p.name || ''} ${p.slug || ''} ${p.purpose_summary || ''} ${(p.models || []).map((m) => m.model_id || '').join(' ')}`.toLowerCase();
      if (!hay.includes(activeProviderSearch)) return false;
    }
    return true;
  });

  if (filtered.length === 0) {
    container.innerHTML = '<div class="gh-empty-state">Nenhum provedor encontrado para este filtro.</div>';
    return;
  }

  container.innerHTML = filtered.map(p => {
    const slug = connEsc(p.slug);
    const envKey = connEsc(p.auth_header_env || '');
    const cachePct = Math.round((p.default_cache_discount || 0) * 100);
    const cacheText = cachePct > 0 ? `${cachePct}% economia em cache` : 'Tarifa padrão';

    const modelsList = (p.models || []).slice(0, 3).map(m => `
      <div class="u-d-flex u-justify-between u-items-center u-s-69f3f1">
        <span class="u-text-slate-light u-mono">${connEsc(m.model_id)}</span>
        <span class="u-text-green">$${Number(m.cost_input_per_m || 0).toFixed(2)} / $${Number(m.cost_output_per_m || 0).toFixed(2)}</span>
      </div>
    `).join('');

    return `
      <div class="conn-provider-card ${p.is_configured ? 'is-configured' : ''}">
        <div>
          <div class="conn-provider-top">
            <div>
              <div class="conn-provider-name">${connEsc(p.name)}</div>
              <div class="conn-provider-badges">
                ${p.has_free_tier ? '<span class="badge-free-tier">Plano Gratuito</span>' : ''}
                ${p.is_configured && p.masked_key ? `<span class="u-mono gh-text-xs u-text-cyan">${connEsc(p.masked_key)}</span>` : ''}
              </div>
            </div>
            <span class="badge ${p.is_configured ? 'badge-value' : 'badge-lowcost'}">
              ${p.is_configured ? 'Conectado' : 'Falta Chave'}
            </span>
          </div>

          <p class="conn-provider-purpose u-mt-2 u-mb-2">${connEsc(p.purpose_summary || 'Provedor de inteligência artificial em nuvem.')}</p>

          <div class="conn-provider-stats u-mb-2">
            <span>${p.models_count || 0} modelos disponíveis</span>
            <span>&bull;</span>
            <span>${cacheText}</span>
          </div>

          ${modelsList ? `<div class="u-mb-2">${modelsList}</div>` : ''}
        </div>

        <div class="conn-provider-footer">
          <div class="conn-provider-links">
            ${p.console_url ? `<a class="conn-external-link" href="${connEsc(p.console_url)}" target="_blank" rel="noopener noreferrer">Obter chave &rarr;</a>` : `<span class="u-text-muted">${envKey}</span>`}
            <div class="u-d-flex u-items-center u-gap-2">
              <span class="gh-item-meta" id="ping-res-${slug}">Latência: --</span>
              <button type="button" class="btn btn-xs btn-secondary" onclick="testProviderPing('${slug}', this)">
                ${uiIcon('zap', 'icon-xs')} Testar
              </button>
            </div>
          </div>
          <div class="conn-key-inline">
            <input id="prov-key-${slug}" type="password" class="form-control conn-key-input" placeholder="${p.is_configured ? `Atualizar chave (${envKey})` : `Cole sua chave (${envKey})`}" />
            <button type="button" class="btn btn-xs btn-primary" onclick="saveProviderInlineKey('${slug}', '${envKey}')">
              Salvar e Ativar
            </button>
          </div>
        </div>
      </div>
    `;
  }).join('');
}

async function testProviderPing(slug, btn) {
  const label = document.getElementById(`ping-res-${slug}`);
  if (btn) btn.disabled = true;
  if (label) {
    label.textContent = 'Testando...';
    label.style.color = 'var(--cyan)';
  }

  try {
    const res = await fetch('/api/v1/connections/test', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ provider: slug })
    });

    const data = await res.json();
    if (label) {
      if (data.status === 'success') {
        label.textContent = `HTTP ${data.http_status} (${data.latency_ms}ms)`;
        label.style.color = '#22c55e';
      } else {
        label.textContent = `${data.message || 'Falha'} (${data.latency_ms}ms)`;
        label.style.color = '#f59e0b';
      }
    }
  } catch (err) {
    if (label) {
      label.textContent = `Erro: ${err.message}`;
      label.style.color = '#ef4444';
    }
  } finally {
    if (btn) btn.disabled = false;
  }
}

async function dispatchCliTask() {
  const cliSelect = document.getElementById('dispatch-cli-select');
  const cwdInput = document.getElementById('dispatch-cwd-input');
  const promptInput = document.getElementById('dispatch-prompt-textarea');
  const timeoutSelect = document.getElementById('dispatch-timeout-select');
  const restoreCheck = document.getElementById('dispatch-restore-check');
  const btn = document.getElementById('btn-dispatch-cli');

  const outputBox = document.getElementById('dispatch-output-box');
  const statusLabel = document.getElementById('dispatch-status-label');
  const metricsLabel = document.getElementById('dispatch-metrics-label');
  const outputPre = document.getElementById('dispatch-output-pre');

  const prompt = promptInput ? promptInput.value.trim() : '';
  if (!prompt) {
    if (window.showToast) window.showToast('Por favor informe o prompt da tarefa agêntica antes de despachar.', 'warning');
    return;
  }

  const cli = cliSelect ? cliSelect.value : 'agy';
  const defaultRoot = (window.soulsHostDefaults && window.soulsHostDefaults.default_souls_data_path)
    ? window.soulsHostDefaults.default_souls_data_path.replace(/[\\/]\.souls_data$/i, '')
    : '.';
  const cwd = cwdInput && cwdInput.value.trim() ? cwdInput.value.trim() : defaultRoot;
  const timeout = timeoutSelect ? parseInt(timeoutSelect.value, 10) : 300;
  const autoRestore = restoreCheck ? restoreCheck.checked : true;

  if (btn) btn.disabled = true;
  if (outputBox) outputBox.style.display = 'block';
  if (statusLabel) {
    statusLabel.textContent = `Executando ${cli}...`;
    statusLabel.style.color = 'var(--cyan)';
  }
  if (metricsLabel) metricsLabel.textContent = 'Aguardando subprocesso...';
  if (outputPre) outputPre.textContent = 'Iniciando processo e capturando stdout/stderr via streaming NDJSON...\n';

  const startTime = Date.now();

  try {
    const res = await fetch('/api/v1/connections/cli/dispatch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        cli: cli,
        prompt: prompt,
        working_dir: cwd,
        timeout_secs: timeout,
        auto_restore_on_failure: autoRestore
      })
    });

    const data = await res.json();
    const elapsed = Math.round((Date.now() - startTime) / 1000);

    if (data.status === 'success' && data.output) {
      const out = data.output;
      const success = out.success;
      if (statusLabel) {
        statusLabel.textContent = success ? `Concluído com Sucesso (Exit Code: 0)` : `Finalizado com Erro (Exit Code: ${out.exit_code})`;
        statusLabel.style.color = success ? '#22c55e' : '#ef4444';
      }
      if (metricsLabel) {
        metricsLabel.textContent = `Duração: ${out.duration_ms}ms | Eventos: ${out.events.length} | Restauração: ${out.restored_files.length} arquivos`;
      }

      let content = '';
      if (out.restored_files.length > 0) {
        content += `[AVISO] Arquivos revertidos via git restore devido ao exit code != 0:\n - ${out.restored_files.join('\n - ')}\n\n`;
      }
      content += out.stdout || '(Stdout vazio)';
      if (out.stderr) {
        content += `\n\n--- STDERR ---\n${out.stderr}`;
      }
      if (outputPre) outputPre.textContent = content;
    } else {
      if (statusLabel) {
        statusLabel.textContent = 'Falha na Execução';
        statusLabel.style.color = '#ef4444';
      }
      if (outputPre) outputPre.textContent = data.message || 'Erro desconhecido ao despachar CLI.';
    }
  } catch (err) {
    if (statusLabel) {
      statusLabel.textContent = 'Erro de Rede / Timeout';
      statusLabel.style.color = '#ef4444';
    }
    if (outputPre) outputPre.textContent = `Exceção: ${err.message}`;
  } finally {
    if (btn) btn.disabled = false;
  }
}

// ==========================================
// GOOGLE UNIVERSE & NOTEBOOKLM BRIDGE (v0.8.2)
// ==========================================

async function checkGoogleSession() {
  try {
    const res = await fetch('/api/v1/google/session');
    if (!res.ok) return;
    const data = await res.json();

    const badge = document.getElementById('google-session-badge');
    const emailEl = document.getElementById('google-account-email');
    const psidEl = document.getElementById('google-masked-psid');
    const csrfEl = document.getElementById('google-csrf-token');

    if (badge) {
      if (data.is_authenticated && !data.is_expired) {
        badge.className = 'badge badge-value';
        badge.innerHTML = `${uiIcon('check-circle', 'icon-xs')} <span>Sessão Google Ativa</span>`;
      } else if (data.is_expired) {
        badge.className = 'badge badge-lowcost';
        badge.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} <span>Sessão Expirada</span>`;
      }
    }

    if (emailEl) emailEl.textContent = data.account_email || 'Nenhuma detectada';
    if (psidEl) psidEl.textContent = data.masked_psid || (data.is_authenticated ? 'Configurado' : 'Não configurado');
    if (csrfEl) csrfEl.textContent = data.csrf_token || 'SNlM0e';
  } catch (err) {
    console.error('Falha ao verificar sessão Google:', err);
  }
}

async function saveGoogleSession() {
  const psid = document.getElementById('google-psid-input')?.value?.trim();
  const psidts = document.getElementById('google-psidts-input')?.value?.trim();
  const sapisid = document.getElementById('google-sapisid-input')?.value?.trim();
  const email = document.getElementById('google-email-input')?.value?.trim();

  if (!psid) {
    if (window.showToast) window.showToast('Por favor informe ao menos o cookie __Secure-1PSID.', 'warning');
    return;
  }

  try {
    const res = await fetch('/api/v1/google/session', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        psid: psid || null,
        psidts: psidts || null,
        sapisid: sapisid || null,
        account_email: email || null
      })
    });

    const data = await res.json();
    if (res.ok) {
      if (window.showToast) window.showToast('Credenciais salvas com sucesso no cofre encriptado!', 'success');
      checkGoogleSession();
    if (typeof loadGoogleConnectionsVault === 'function') {
      loadGoogleConnectionsVault();
    }
    } else {
      if (window.showToast) window.showToast('Erro ao salvar credenciais: ' + (data.message || 'Falha desconhecida'), 'error');
    }
  } catch (err) {
    if (window.showToast) window.showToast('Erro de rede: ' + err.message, 'error');
  }
}

async function dispatchGoogleRpc() {
  const rpcId = document.getElementById('google-rpc-select')?.value || 'QA9ei';
  const argsRaw = document.getElementById('google-rpc-args-input')?.value || '[]';
  const box = document.getElementById('google-bridge-output-box');
  const pre = document.getElementById('google-bridge-output-pre');
  const label = document.getElementById('google-bridge-status-label');

  let argsJson = [];
  try {
    argsJson = JSON.parse(argsRaw);
  } catch (e) {
    if (window.showToast) window.showToast('Argumentos devem ser um JSON válido (ex: ["valor"]).', 'error');
    return;
  }

  if (box) {
    box.classList.remove('is-hidden');
    box.style.display = 'block';
  }
  if (label) {
    label.textContent = 'Executando ação...';
    label.style.color = 'var(--purple)';
  }
  if (pre) pre.textContent = 'Enviando comando ao NotebookLM...';

  try {
    const res = await fetch('/api/v1/google/notebooklm/rpc', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        rpc_id: rpcId,
        arguments: argsJson
      })
    });

    const data = await res.json();
    if (res.ok && data.status === 'success') {
      if (label) {
        label.textContent = 'Ação concluída com sucesso';
        label.style.color = '#22c55e';
      }
      if (pre) pre.textContent = JSON.stringify(data.data || data, null, 2);
    } else {
      if (label) {
        label.textContent = 'Não foi possível concluir a ação';
        label.style.color = '#ef4444';
      }
      if (pre) pre.textContent = data.message || JSON.stringify(data, null, 2);
    }
  } catch (err) {
    if (label) {
      label.textContent = 'Erro de conexão';
      label.style.color = '#ef4444';
    }
    if (pre) pre.textContent = err.message;
  }
}

async function testJulesMigration() {
  const cmd = document.getElementById('jules-cmd-input')?.value || 'jules apply';
  const box = document.getElementById('google-bridge-output-box');
  const pre = document.getElementById('google-bridge-output-pre');
  const label = document.getElementById('google-bridge-status-label');

  if (box) {
    box.classList.remove('is-hidden');
    box.style.display = 'block';
  }
  if (label) {
    label.textContent = 'Roteador de Migração Jules -> agy';
    label.style.color = 'var(--cyan)';
  }

  try {
    const res = await fetch('/api/v1/google/jules/migrate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: cmd })
    });
    const data = await res.json();
    if (pre) pre.textContent = JSON.stringify(data, null, 2);
  } catch (err) {
    if (pre) pre.textContent = `Erro ao testar migração: ${err.message}`;
  }
}

// Hook de ativação ao alternar para a aba
document.addEventListener('DOMContentLoaded', () => {
  // Inicialização passiva
});
