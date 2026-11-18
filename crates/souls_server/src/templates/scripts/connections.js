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
  } catch (err) {
    console.error('Falha ao carregar conexões:', err);
  }
}

function renderClis(clis) {
  const container = document.getElementById('clis-grid');
  const badgeTop = document.getElementById('badge-clis-count');
  const badgeSection = document.getElementById('clis-count-badge');

  if (!container) return;

  const installedCount = clis.filter(c => c.is_installed).length;
  if (badgeTop) badgeTop.textContent = `${installedCount}/${clis.length}`;
  if (badgeSection) badgeSection.textContent = `${installedCount} de ${clis.length} Instaladas`;

  if (clis.length === 0) {
    container.innerHTML = '<div class="u-text-muted u-s-801160">Nenhuma CLI catalogada.</div>';
    return;
  }

  container.innerHTML = clis.map(cli => {
    const isInst = cli.is_installed;
    const statusColor = isInst ? '#22c55e' : '#71717A';
    const statusText = isInst ? 'Disponível no Host' : 'Não Instalada';
    const borderStyle = isInst ? 'border-color: rgba(34, 197, 94, 0.3);' : 'border-color: rgba(255, 255, 255, 0.08); opacity: 0.8;';
    const versionDisplay = cli.version ? `v${cli.version}` : (isInst ? 'v1.x' : '--');
    const pathDisplay = cli.resolved_path ? cli.resolved_path : 'Não encontrada no PATH';

    const modesBadges = (cli.supported_modes || []).map(m =>
      `<span class="u-text-slate u-mono u-s-e34342">${m}</span>`
    ).join(' ');

    return `
      <div class="u-s-907208" style="${borderStyle} transition: transform 0.15s ease;">
        <div class="u-d-flex u-justify-between u-items-start u-mb-10p">
          <div>
            <h4 class="u-text-white u-s-c6b43a">${cli.display_name}</h4>
            <div class="u-mono u-text-cyan u-s-a768c1">${cli.binary_name}</div>
          </div>
          <span class="u-fw-600 u-s-463ae6" style="background: ${isInst ? 'rgba(34, 197, 94, 0.15)' : 'rgba(100, 116, 139, 0.15)'}; color: ${statusColor}; border: 1px solid ${statusColor};">
            ${statusText}
          </span>
        </div>

        <div class="u-mb-2 u-d-flex u-justify-between u-s-5b9d54">
          <span class="u-text-muted">Versão:</span>
          <strong class="u-text-white">${versionDisplay}</strong>
        </div>

        <div class="u-mb-10p u-text-muted u-mono u-s-6244d7">
          ${pathDisplay}
        </div>

        <div class="u-text-muted u-mb-1 u-s-22b62c">Modos Suportados:</div>
        <div class="u-d-flex u-gap-6p u-flex-wrap">
          ${modesBadges}
        </div>
      </div>
    `;
  }).join('');
}

function renderProviders(providers) {
  const container = document.getElementById('providers-grid');
  const badgeSection = document.getElementById('providers-count-badge');

  if (!container) return;

  if (badgeSection) badgeSection.textContent = `${providers.length} Provedores Homologados`;

  if (providers.length === 0) {
    container.innerHTML = '<div class="u-text-muted u-s-801160">Nenhum provedor retornado.</div>';
    return;
  }

  container.innerHTML = providers.map(p => {
    const isConfig = p.is_configured;
    const statusColor = isConfig ? '#22c55e' : '#f59e0b';
    const statusLabel = isConfig ? 'Ativo (Chave Detectada)' : 'Aguardando Chave';
    const keyInfo = p.masked_key ? `<code class="u-text-cyan">${p.masked_key}</code>` : `<code class="u-s-aba882">$${p.auth_header_env}</code>`;
    const cachePct = Math.round(p.default_cache_discount * 100);

    const modelsList = (p.models || []).slice(0, 3).map(m => `
      <div class="u-d-flex u-justify-between u-items-center u-s-69f3f1">
        <span class="u-text-slate-light u-mono">${m.model_id}</span>
        <span class="u-text-green">$${m.cost_input_per_m.toFixed(2)} / $${m.cost_output_per_m.toFixed(2)}</span>
      </div>
    `).join('');

    return `
      <div class="u-d-flex u-flex-col u-justify-between u-s-907208">
        <div>
          <div class="u-d-flex u-justify-between u-items-start u-mb-2">
            <div>
              <h4 class="u-text-white u-s-d6b5d4">${p.name}</h4>
              <div class="u-mono u-text-muted u-s-ecd7f5">${p.base_url}</div>
            </div>
            <span class="u-fw-600 u-nowrap u-s-deb014" style="background: ${isConfig ? 'rgba(34, 197, 94, 0.15)' : 'rgba(245, 158, 11, 0.15)'}; color: ${statusColor}; border: 1px solid ${statusColor};">
              ${statusLabel}
            </span>
          </div>

          <div class="u-s-45aaa6">
            <div class="u-d-flex u-justify-between u-mb-1">
              <span class="u-text-muted">Credencial (${p.auth_header_env}):</span>
              <span>${keyInfo}</span>
            </div>
            <div class="u-d-flex u-justify-between">
              <span class="u-text-muted">Desconto Prompt Cache:</span>
              <strong class="u-text-cyan">${cachePct}% OFF</strong>
            </div>
          </div>

          <div class="u-mb-2 u-text-muted u-s-22b62c">
            Modelos Canônicos (${p.models_count}):
          </div>
          <div class="u-mb-3">
            ${modelsList}
          </div>
        </div>

        <div class="u-d-flex u-justify-between u-items-center u-mt-6p u-s-2f8642">
          <span class="u-text-muted u-s-a768c1" id="ping-res-${p.slug}">Latência: --</span>
          <button class="btn btn-secondary u-s-7a00b7" onclick="testProviderPing('${p.slug}', this)">
            ${uiIcon('zap', 'icon-xs')} Testar Ping
          </button>
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
        badge.innerHTML = `${uiIcon('check-circle', 'icon-xs')} Sessão Ativa`;
        badge.style.background = 'rgba(34, 197, 94, 0.15)';
        badge.style.color = '#22c55e';
        badge.style.borderColor = '#22c55e';
      } else if (data.is_expired) {
        badge.innerHTML = `${uiIcon('alert-triangle', 'icon-xs')} Sessão Expirada`;
        badge.style.background = 'rgba(234, 179, 8, 0.15)';
        badge.style.color = '#eab308';
        badge.style.borderColor = '#eab308';
      } else {
        badge.innerHTML = `${uiIcon('lock', 'icon-xs')} Desconectado`;
        badge.style.background = 'rgba(239, 68, 68, 0.15)';
        badge.style.color = '#ef4444';
        badge.style.borderColor = '#ef4444';
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
      if (window.showToast) window.showToast('Credenciais salvas com sucesso no cofre encriptado GV1!', 'success');
      checkGoogleSession();
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

  if (box) box.style.display = 'block';
  if (label) {
    label.textContent = `Executando RPC ${rpcId}...`;
    label.style.color = 'var(--purple)';
  }
  if (pre) pre.textContent = 'Enviando payload batchexecute ao backend do NotebookLM...';

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
        label.textContent = `RPC ${rpcId} Concluído com Sucesso`;
        label.style.color = '#22c55e';
      }
      if (pre) pre.textContent = JSON.stringify(data.data || data, null, 2);
    } else {
      if (label) {
        label.textContent = `Falha no RPC ${rpcId}`;
        label.style.color = '#ef4444';
      }
      if (pre) pre.textContent = data.message || JSON.stringify(data, null, 2);
    }
  } catch (err) {
    if (label) {
      label.textContent = `Exceção de Rede no RPC`;
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

  if (box) box.style.display = 'block';
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
