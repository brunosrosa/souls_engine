// evolutive_harness.js — Auto-Evolução & Aprendizado (v0.9.4 / ADR-034)

let _lastHarnessOverview = null;

function harnessEsc(str) {
  return String(str ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

async function loadEvolutiveHarnessTab() {
  try {
    const resp = await fetch('/api/v1/harness/overview');
    const data = await resp.json();
    _lastHarnessOverview = data;
    renderEvolutiveHarnessOverview(data);
  } catch (err) {
    const gateEl = document.getElementById('harness-catnap-gate-reason');
    if (gateEl) {
      gateEl.textContent = 'Não foi possível carregar o status do aprendizado contínuo: ' + err.message;
    }
  }
}

function renderEvolutiveHarnessOverview(data) {
  const sum = data?.summary || {};
  const pins = Array.isArray(data?.constitutional_pins) ? data.constitutional_pins : [];
  const fibers = Array.isArray(data?.cordis?.fibers) ? data.cordis.fibers : [];
  const tombstones = Array.isArray(data?.tombstones) ? data.tombstones : [];
  const candidates = Array.isArray(data?.ralph_candidates) ? data.ralph_candidates : [];
  const catnap = data?.catnap || {};
  const cfg = catnap.config || {};

  const chipPins = document.getElementById('harness-chip-pins');
  if (chipPins) {
    const ok = !!sum.pins_verified;
    chipPins.classList.toggle('badge-value', ok);
    chipPins.classList.toggle('badge-champion', !ok);
    const label = chipPins.querySelector('span');
    if (label) {
      label.textContent = ok
        ? `Identidade protegida (${pins.length} regra(s))`
        : 'ALERTA: violação nas regras centrais';
    }
  }

  const chipOpt = document.getElementById('harness-chip-optimizations');
  if (chipOpt) {
    const label = chipOpt.querySelector('span');
    if (label) {
      label.textContent = `Otimizações ativas: ${sum.active_optimizations ?? 0}`;
    }
  }

  const chipCatnap = document.getElementById('harness-chip-catnap');
  if (chipCatnap) {
    const label = chipCatnap.querySelector('span');
    if (label) {
      label.textContent = sum.catnap_gate_allowed
        ? 'Tempo ocioso: pronto'
        : `Tempo ocioso: ${catnap.idle_elapsed_secs ?? 0}s`;
    }
  }

  const badgeAlwaysOn = document.getElementById('harness-badge-always-on');
  if (badgeAlwaysOn) {
    badgeAlwaysOn.textContent = `${sum.always_on_chars_total ?? 0} caracteres em contexto`;
  }

  const badgeBudget = document.getElementById('harness-badge-budget-today');
  if (badgeBudget) {
    const spent = Number(catnap.spent_today_usd ?? 0).toFixed(2);
    badgeBudget.textContent = `Gasto hoje: $${spent}`;
  }

  const gateReason = document.getElementById('harness-catnap-gate-reason');
  if (gateReason) {
    gateReason.textContent = sum.catnap_gate_reason || 'Monitor em espera cooperativa (< 1 ms preempção).';
  }

  const setChecked = (id, val) => {
    const el = document.getElementById(id);
    if (el) el.checked = !!val;
  };
  const setValue = (id, val) => {
    const el = document.getElementById(id);
    if (el && val !== undefined && val !== null) el.value = val;
  };

  setChecked('harness-cfg-enabled', cfg.enabled ?? true);
  setValue('harness-cfg-idle-secs', cfg.min_idle_secs ?? 180);
  setValue('harness-cfg-gpu-temp', cfg.max_gpu_temp_c ?? 62);
  setValue('harness-cfg-budget', Number(cfg.max_daily_budget_usd ?? 0).toFixed(2));
  setChecked('harness-cfg-free-cloud', cfg.allow_free_cloud_improver ?? true);
  setChecked('harness-cfg-quiet-enabled', cfg.quiet_hours_enabled ?? true);
  setValue('harness-cfg-quiet-start', cfg.quiet_hours_start_hour ?? 22);
  setValue('harness-cfg-quiet-end', cfg.quiet_hours_end_hour ?? 8);

  renderHarnessTombstones(tombstones);
  renderHarnessFibers(fibers);
  renderHarnessRalphCandidates(candidates);
  renderHarnessLabDrawer(data);
}

function renderHarnessTombstones(items) {
  const container = document.getElementById('harness-tombstones-list');
  if (!container) return;

  if (!items.length) {
    container.innerHTML =
      '<div class="gh-subpanel u-text-muted gh-text-xs">Nenhuma contradição de memória registrada até o momento. Suas lembranças estão coerentes.</div>';
    return;
  }

  container.innerHTML = items
    .map((t) => {
      const isSuperseded = t.status === 'superseded';
      const badgeClass = isSuperseded ? 'badge-champion' : 'badge-value';
      const badgeLabel = isSuperseded ? 'Substituída automaticamente' : 'Restaurada pelo operador';
      const actionHtml = isSuperseded
        ? `<button type="button" class="btn btn-sm btn-secondary" onclick="restoreHarnessTombstone('${harnessEsc(t.tombstone_id)}')">Restaurar versão anterior</button>`
        : `<span class="gh-text-xs u-text-green">Ativa novamente nas buscas</span>`;

      return `
        <div class="gh-subpanel">
          <div class="u-d-flex u-justify-between u-items-center u-gap-2 u-mb-2">
            <span class="badge ${badgeClass}">${harnessEsc(badgeLabel)}</span>
            ${actionHtml}
          </div>
          <div class="gh-text-xs u-text-muted u-mb-1"><strong>Motivo:</strong> ${harnessEsc(t.reason_pt)}</div>
          <div class="gh-text-xs u-text-amber u-mb-1"><strong>Versão antiga:</strong> ${harnessEsc(t.superseded_content_preview)}</div>
          <div class="gh-text-xs u-text-cyan"><strong>Versão atual:</strong> ${harnessEsc(t.new_content_preview)}</div>
        </div>
      `;
    })
    .join('');
}

function renderHarnessFibers(fibers) {
  const container = document.getElementById('harness-fibers-list');
  if (!container) return;

  if (!fibers.length) {
    container.innerHTML =
      '<div class="gh-subpanel u-text-muted gh-text-xs">Nenhuma otimização tática montada nesta sessão.</div>';
    return;
  }

  container.innerHTML = fibers
    .map((f) => {
      let stateText = 'Ativa';
      let stateClass = 'badge-value';
      if (!f.active) {
        stateText = 'Desfeita';
        stateClass = 'badge-champion';
      } else if (f.paused_by_coeffect) {
        stateText = 'Pausada (dependência indisponível)';
        stateClass = 'badge-champion';
      }

      const actionBtn = f.active
        ? `<button type="button" class="btn btn-sm btn-secondary" onclick="disposeHarnessFiber('${harnessEsc(f.fiber_id)}')">Desfazer otimização</button>`
        : `<button type="button" class="btn btn-sm btn-primary" onclick="reactivateHarnessFiber('${harnessEsc(f.fiber_id)}')">Reativar otimização</button>`;

      return `
        <div class="gh-subpanel">
          <div class="u-d-flex u-justify-between u-items-center u-gap-2 u-mb-1">
            <span class="u-fw-600 u-text-white">${harnessEsc(f.title_pt)}</span>
            <span class="badge ${stateClass}">${harnessEsc(stateText)}</span>
          </div>
          <div class="gh-text-xs u-text-muted u-mb-2">${harnessEsc(f.benefit_summary_pt)}</div>
          <div class="u-d-flex u-justify-between u-items-center u-gap-2">
            <span class="gh-text-xs u-text-cyan">Pontuação ELO: ${Number(f.elo_rating ?? 1200).toFixed(0)}</span>
            ${actionBtn}
          </div>
        </div>
      `;
    })
    .join('');
}

function renderHarnessRalphCandidates(candidates) {
  const badge = document.getElementById('harness-badge-quarantine-count');
  if (badge) {
    const pendingCount = candidates.filter((c) => c.status === 'quarantined').length;
    badge.textContent = `${pendingCount} aguardando`;
  }

  const container = document.getElementById('harness-ralph-list');
  if (!container) return;

  if (!candidates.length) {
    container.innerHTML =
      '<div class="gh-subpanel u-text-muted gh-text-xs">Ainda não há novas habilidades aprendidas aguardando aprovação. Quando uma tarefa falhar e for resolvida com sucesso, ela aparecerá aqui.</div>';
    return;
  }

  container.innerHTML = candidates
    .map((c) => {
      const isApproved = c.status === 'approved';
      const approveBtn = isApproved
        ? '<span class="badge badge-value">Ativada no sistema</span>'
        : `<button type="button" class="btn btn-sm btn-primary" onclick="approveHarnessRalphSkill('${harnessEsc(c.slug)}')">Aprovar e Ativar</button>`;
      const reviewBtn = isApproved
        ? ''
        : `<button type="button" class="btn btn-sm btn-secondary" onclick="openHarnessSkillInQuarantine('${harnessEsc(c.slug)}')">Revisar na Quarentena</button>`;

      return `
        <div class="gh-subpanel">
          <div class="u-d-flex u-justify-between u-items-center u-gap-2 u-mb-1">
            <span class="u-fw-600 u-text-white">${harnessEsc(c.title_pt)}</span>
            <span class="badge badge-value">Qualidade ${(Number(c.e3_score ?? 0) * 100).toFixed(0)}/100</span>
          </div>
          <div class="gh-text-xs u-text-muted u-mb-1">${harnessEsc(c.problem_pattern_pt)}</div>
          <div class="gh-text-xs u-text-cyan u-mb-2">${harnessEsc(c.diff_preview)}</div>
          <div class="u-d-flex u-justify-between u-items-center u-flex-wrap u-gap-2">
            <span class="gh-text-xs u-text-muted">${harnessEsc(c.solution_summary_pt)}</span>
            <div class="u-d-flex u-items-center u-gap-2">
              ${reviewBtn}
              ${approveBtn}
            </div>
          </div>
        </div>
      `;
    })
    .join('');
}

function renderHarnessLabDrawer(data) {
  const pinsPre = document.getElementById('harness-lab-pins');
  if (pinsPre) {
    const pins = Array.isArray(data?.constitutional_pins) ? data.constitutional_pins : [];
    pinsPre.textContent = pins.length
      ? pins
          .map(
            (p) =>
              `[${p.verified ? 'OK' : 'FAIL-CLOSED'}] ${p.file_path_posix}\n  expected: ${p.expected_sha256}\n  actual:   ${p.actual_sha256}`
          )
          .join('\n\n')
      : 'Sem arquivos c1 registrados.';
  }

  const slotsPre = document.getElementById('harness-lab-slots');
  if (slotsPre) {
    const slots = Array.isArray(data?.slots_catalog) ? data.slots_catalog : [];
    const coeffects = data?.cordis?.coeffects || {};
    const slotLines = slots.map(
      (s) =>
        `${s.code} (${s.pinned_readonly ? 'c1 read-only' : 'mutable'}) | ${s.label_pt} | fibras=${s.active_fibers}`
    );
    const coeffLines = Object.entries(coeffects).map(([k, v]) => `coeffect[${k}] = ${v}`);
    slotsPre.textContent = [...slotLines, '', 'Coefeitos Cordis:', ...coeffLines].join('\n');
  }

  const evalPre = document.getElementById('harness-lab-weevolve-catnap');
  if (evalPre) {
    const ratings = Array.isArray(data?.weevolve_ratings) ? data.weevolve_ratings : [];
    const slices = Array.isArray(data?.catnap_slices) ? data.catnap_slices : [];
    const rLines = ratings.map(
      (r) =>
        `${r.target_id} [${r.target_kind}] ELO=${Number(r.elo_rating).toFixed(1)} EMA=${Number(r.ema_score).toFixed(3)} matches=${r.total_matches} (${r.last_signal})`
    );
    const sLines = slices.map(
      (s) =>
        `${s.slice_id} [${s.status}] ${s.slice_kind} | ${Number(s.duration_ms).toFixed(2)}ms | $${Number(s.cost_usd).toFixed(2)} | fp=${String(s.runmark_fingerprint).slice(0, 12)}`
    );
    evalPre.textContent = [
      '=== WeEvolve Bradley-Terry Ratings ===',
      ...(rLines.length ? rLines : ['(nenhuma avaliação registrada)']),
      '',
      '=== Catnap Consolidation Ledger ===',
      ...(sLines.length ? sLines : ['(nenhuma fatia executada hoje)']),
    ].join('\n');
  }

  const indexPre = document.getElementById('harness-lab-index-md');
  if (indexPre) {
    indexPre.textContent = data?.experience_library?.index_markdown || '# Domain Experience Library Index\n(vazio)';
  }
}

async function saveHarnessCatnapConfig() {
  const statusEl = document.getElementById('harness-cfg-save-status');
  if (statusEl) statusEl.textContent = 'Salvando preferências...';

  const payload = {
    enabled: !!document.getElementById('harness-cfg-enabled')?.checked,
    min_idle_secs: Number(document.getElementById('harness-cfg-idle-secs')?.value || 180),
    max_gpu_temp_c: Number(document.getElementById('harness-cfg-gpu-temp')?.value || 62),
    max_daily_budget_usd: Number(document.getElementById('harness-cfg-budget')?.value || 0),
    allow_free_cloud_improver: !!document.getElementById('harness-cfg-free-cloud')?.checked,
    quiet_hours_enabled: !!document.getElementById('harness-cfg-quiet-enabled')?.checked,
    quiet_hours_start_hour: Number(document.getElementById('harness-cfg-quiet-start')?.value || 22),
    quiet_hours_end_hour: Number(document.getElementById('harness-cfg-quiet-end')?.value || 8),
  };

  try {
    const resp = await fetch('/api/v1/harness/catnap/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const res = await resp.json();
    if (statusEl) {
      statusEl.textContent = res.message || res.error || 'Preferências atualizadas.';
    }
    await loadEvolutiveHarnessTab();
  } catch (err) {
    if (statusEl) statusEl.textContent = 'Erro ao salvar: ' + err.message;
  }
}

async function triggerHarnessCatnapCycle() {
  const btn = document.getElementById('btn-harness-run-catnap');
  if (btn) btn.disabled = true;
  try {
    // Omitting slice_kind cycles cleanly in round-robin across all 5 maintenance slices
    await fetch('/api/v1/harness/catnap/trigger', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ force_manual: true }),
    });
    await loadEvolutiveHarnessTab();
  } finally {
    if (btn) btn.disabled = false;
  }
}

async function scanHarnessTombstones() {
  await fetch('/api/v1/harness/tombstones/resolve', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({}),
  });
  await loadEvolutiveHarnessTab();
}

async function restoreHarnessTombstone(tombstoneId) {
  await fetch('/api/v1/harness/tombstones/restore', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tombstone_id: tombstoneId }),
  });
  await loadEvolutiveHarnessTab();
}

async function disposeHarnessFiber(fiberId) {
  await fetch('/api/v1/harness/cordis/dispose', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ fiber_id: fiberId }),
  });
  await loadEvolutiveHarnessTab();
}

async function reactivateHarnessFiber(fiberId) {
  await fetch('/api/v1/harness/cordis/reactivate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ fiber_id: fiberId }),
  });
  await loadEvolutiveHarnessTab();
}

async function mountHarnessFiberFromUi() {
  const titleEl = document.getElementById('harness-cordis-title');
  const cueEl = document.getElementById('harness-cordis-cue');
  const scopeEl = document.getElementById('harness-cordis-scope');
  const statusEl = document.getElementById('harness-cordis-create-status');

  const title_pt = titleEl?.value?.trim() || '';
  const ephemeral_cue_text = cueEl?.value?.trim() || '';
  const rawScope = scopeEl?.value || 'all';
  const target_task_type = rawScope === 'all' ? null : rawScope;

  if (!title_pt || !ephemeral_cue_text) {
    if (statusEl) statusEl.textContent = 'Informe o nome e a instrução prática da orientação.';
    return;
  }
  if (statusEl) statusEl.textContent = 'Ativando ajuste reversível...';

  try {
    const resp = await fetch('/api/v1/harness/cordis/mount', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        slot: 'c5',
        title_pt,
        benefit_summary_pt: 'Orientação criada manualmente pelo operador com reversão em 1 clique.',
        ephemeral_cue_text,
        target_task_type,
      }),
    });
    const res = await resp.json();
    if (res.status === 'ok') {
      if (titleEl) titleEl.value = '';
      if (cueEl) cueEl.value = '';
      if (statusEl) statusEl.textContent = 'Ajuste ativado com sucesso!';
      await loadEvolutiveHarnessTab();
    } else if (statusEl) {
      statusEl.textContent = res.error || 'Falha ao ativar ajuste.';
    }
  } catch (err) {
    if (statusEl) statusEl.textContent = 'Erro: ' + err.message;
  }
}

async function crystallizeHarnessSkillFromUi() {
  const titleEl = document.getElementById('harness-ralph-title');
  const probEl = document.getElementById('harness-ralph-problem');
  const playEl = document.getElementById('harness-ralph-playbook');
  const statusEl = document.getElementById('harness-ralph-create-status');

  const title_pt = titleEl?.value?.trim() || '';
  const problem_pattern_pt = probEl?.value?.trim() || '';
  const resolution_playbook = playEl?.value?.trim() || '';

  if (!title_pt || !problem_pattern_pt || !resolution_playbook) {
    if (statusEl) statusEl.textContent = 'Preencha título, problema e passos da solução.';
    return;
  }
  if (statusEl) statusEl.textContent = 'Gerando candidato na Quarentena...';

  try {
    const resp = await fetch('/api/v1/harness/ralph/crystallize', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title_pt,
        domain: 'ops',
        problem_pattern_pt,
        resolution_playbook,
      }),
    });
    const res = await resp.json();
    if (res.status === 'ok') {
      if (titleEl) titleEl.value = '';
      if (probEl) probEl.value = '';
      if (playEl) playEl.value = '';
      if (statusEl) statusEl.textContent = 'Enviada para Quarentena do Skills Hub!';
      await loadEvolutiveHarnessTab();
    } else if (statusEl) {
      statusEl.textContent = res.error || 'Erro ao gerar habilidade.';
    }
  } catch (err) {
    if (statusEl) statusEl.textContent = 'Erro: ' + err.message;
  }
}

async function approveHarnessRalphSkill(slug) {
  const resp = await fetch('/api/v1/harness/ralph/approve', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ slug }),
  });
  const res = await resp.json();
  if (res.status === 'ok' && typeof showToast === 'function') {
    showToast(`Habilidade '${slug}' ativada com sucesso!`, 'success');
  }
  await loadEvolutiveHarnessTab();
}

function openHarnessSkillInQuarantine(slug) {
  if (typeof switchTab === 'function') {
    switchTab('tab-skills');
  }
  if (typeof switchSkillsSubtab === 'function') {
    switchSkillsSubtab('explorer');
  }
  if (typeof fetchQuarantineList === 'function') {
    setTimeout(() => {
      fetchQuarantineList(slug);
    }, 150);
  }
}
