/**
 * Research Hub & Deep Research Mission Control Client (v0.7.0)
 *
 * Handles:
 * - Subtab navigation within "🔬 Pesquisa Profunda"
 * - Dynamic scoping, STORM perspective drafting and FinOps pre-flight estimation
 * - Real-time SSE telemetry stream (/api/v1/research/stream)
 * - Mission lifecycle operations (enqueue, draft, cancel, archive)
 * - Ecosystem Doctor readiness inspector
 */

let currentResearchMissions = [];
let selectedMissionDetail = null;
let researchSseSource = null;

// ============================================================================
// SUBTAB SWITCHER
// ============================================================================

function switchResearchSubtab(subtabId) {
  const panels = [
    { id: 'new-mission', btn: 'btn-research-subtab-new', panel: 'panel-research-new' },
    { id: 'active', btn: 'btn-research-subtab-active', panel: 'panel-research-active' },
    { id: 'library', btn: 'btn-research-subtab-library', panel: 'panel-research-library' },
    { id: 'quality', btn: 'btn-research-subtab-quality', panel: 'panel-research-quality' },
    { id: 'doctor', btn: 'btn-research-subtab-doctor', panel: 'panel-research-doctor' }
  ];

  panels.forEach(item => {
    const btn = document.getElementById(item.btn);
    const panel = document.getElementById(item.panel);
    if (!btn || !panel) return;

    if (item.id === subtabId) {
      btn.classList.add('active');
      panel.style.display = 'block';
    } else {
      btn.classList.remove('active');
      panel.style.display = 'none';
    }
  });

  if (subtabId === 'active') {
    loadResearchHubMissions();
  } else if (subtabId === 'library') {
    loadResearchHubMissions();
  } else if (subtabId === 'doctor') {
    loadResearchDoctorReport();
  }
}

// ============================================================================
// TEMPLATES & PLAN GENERATOR (TIER 0)
// ============================================================================

function applyResearchTemplate(tplKey) {
  const qInput = document.getElementById('input-research-question');
  if (!qInput) return;

  if (tplKey === 'tech_benchmark') {
    qInput.value = 'Compare a arquitetura do Tokio vs Glommio para I/O bare-metal no Windows, analisando latência, escalabilidade em threads e compatibilidade com IOCP.';
  } else if (tplKey === 'market_review') {
    qInput.value = 'Revisão aprofundada do mercado e maturidade técnica de agentes autônomos de código aberto com memória de longo prazo e MCP.';
  }
  generateResearchDraftPlan();
}

async function generateResearchDraftPlan() {
  const qInput = document.getElementById('input-research-question');
  if (!qInput || !qInput.value.trim()) {
    alert('Por favor, informe a pergunta central da pesquisa.');
    return;
  }

  const q = qInput.value.trim();

  try {
    const res = await fetch('/api/v1/research/plan', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question: q })
    });
    const data = await res.json();
    if (data.status === 'success') {
      renderEditableBrief(data.brief, data.finops);
    } else {
      alert('Erro ao gerar plano: ' + (data.message || 'Desconhecido'));
    }
  } catch (err) {
    console.error('Falha ao gerar plano de pesquisa:', err);
    alert('Erro de comunicação com o souls_server.');
  }
}

function renderEditableBrief(brief, finops) {
  const box = document.getElementById('box-editable-brief');
  if (box) box.style.display = 'block';

  // Badge
  const effortBadge = document.getElementById('label-effort-badge');
  if (effortBadge) {
    let effText = 'L2 Comparativo';
    if (brief.effort === 'l1_fact_finding') effText = 'L1 Fact-Finding';
    if (brief.effort === 'l3_exhaustive') effText = 'L3 Exaustivo';
    effortBadge.innerText = effText;
  }

  // Perspectives
  const pList = document.getElementById('list-brief-perspectives');
  if (pList) {
    pList.innerHTML = brief.perspectives.map((p, idx) => `
      <div style="display:flex; gap:6px; align-items:center;">
        <span style="color:#38bdf8; font-size:0.75rem;">#${idx+1}</span>
        <input type="text" class="input-base brief-perspective-item" value="${escapeHtml(p)}" style="flex:1; font-size:0.8rem; padding:4px 8px;">
      </div>
    `).join('');
  }

  // Subtopics
  const sList = document.getElementById('list-brief-subtopics');
  if (sList) {
    sList.innerHTML = brief.subtopics.map((s, idx) => `
      <div style="display:flex; gap:6px; align-items:center;">
        <span style="color:#10b981; font-size:0.75rem;">•</span>
        <input type="text" class="input-base brief-subtopic-item" value="${escapeHtml(s)}" style="flex:1; font-size:0.8rem; padding:4px 8px;">
      </div>
    `).join('');
  }

  // Exclusions
  const eList = document.getElementById('list-brief-exclusions');
  if (eList) {
    eList.innerHTML = brief.exclusions.map((e, idx) => `
      <div style="display:flex; gap:6px; align-items:center;">
        <span style="color:#ef4444; font-size:0.75rem;">✕</span>
        <input type="text" class="input-base brief-exclusion-item" value="${escapeHtml(e)}" style="flex:1; font-size:0.8rem; padding:4px 8px;">
      </div>
    `).join('');
  }

  // Numbers
  const bInp = document.getElementById('input-brief-breadth');
  if (bInp) bInp.value = brief.breadth;
  const dInp = document.getElementById('input-brief-depth');
  if (dInp) dInp.value = brief.depth;
  const cInp = document.getElementById('input-brief-cap');
  if (cInp) cInp.value = finops.cap_usd.toFixed(2);

  // FinOps Preview Box
  const effSpan = document.getElementById('finops-effort');
  if (effSpan) effSpan.innerText = brief.effort.toUpperCase();
  const turnsSpan = document.getElementById('finops-turns');
  if (turnsSpan) turnsSpan.innerText = `~${finops.est_turns} turnos`;
  const t0Span = document.getElementById('finops-tokens-t0');
  if (t0Span) t0Span.innerText = `${(finops.est_tokens_tier0 / 1000).toFixed(0)}k tokens`;
  const t2Span = document.getElementById('finops-tokens-t2');
  if (t2Span) t2Span.innerText = `${(finops.est_tokens_tier2 / 1000).toFixed(0)}k tokens`;
  const costSpan = document.getElementById('finops-cost-range');
  if (costSpan) costSpan.innerText = `$${finops.est_usd_low.toFixed(2)} - $${finops.est_usd_high.toFixed(2)}`;
  const hardCap = document.getElementById('finops-hard-cap');
  if (hardCap) hardCap.innerText = `$${finops.cap_usd.toFixed(2)}`;
}

// ============================================================================
// MISSION CREATION & LAUNCH
// ============================================================================

async function submitNewResearchMission(queueImmediately) {
  const qInput = document.getElementById('input-research-question');
  if (!qInput || !qInput.value.trim()) {
    alert('Informe a pergunta da pesquisa.');
    return;
  }

  const question = qInput.value.trim();

  // Collect perspectives
  const perspectives = Array.from(document.querySelectorAll('.brief-perspective-item'))
    .map(el => el.value.trim())
    .filter(Boolean);

  // Collect subtopics
  const subtopics = Array.from(document.querySelectorAll('.brief-subtopic-item'))
    .map(el => el.value.trim())
    .filter(Boolean);

  // Collect exclusions
  const exclusions = Array.from(document.querySelectorAll('.brief-exclusion-item'))
    .map(el => el.value.trim())
    .filter(Boolean);

  const breadth = parseInt(document.getElementById('input-brief-breadth')?.value || '3', 10);
  const depth = parseInt(document.getElementById('input-brief-depth')?.value || '2', 10);
  const cap_usd = parseFloat(document.getElementById('input-brief-cap')?.value || '1.50');

  const payload = {
    question,
    perspectives: perspectives.length > 0 ? perspectives : undefined,
    subtopics: subtopics.length > 0 ? subtopics : undefined,
    exclusions: exclusions.length > 0 ? exclusions : undefined,
    breadth,
    depth,
    cap_usd,
    queue_immediately: queueImmediately
  };

  try {
    const res = await fetch('/api/v1/research/missions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.status === 'success') {
      const mission = data.mission;
      // Show command box
      const cmdBox = document.getElementById('box-copy-command');
      const cmdText = document.getElementById('text-copy-command');
      if (cmdBox && cmdText) {
        cmdBox.style.display = 'block';
        cmdText.innerText = data.command_hint || `/deep-research ${mission.run_id}`;
      }

      alert(queueImmediately ? 'Missão enfileirada com sucesso para o Hermes Agent!' : 'Rascunho salvo com sucesso!');
      loadResearchHubMissions();
      if (queueImmediately) {
        switchResearchSubtab('active');
      }
    } else {
      alert('Erro ao criar missão: ' + (data.message || 'Desconhecido'));
    }
  } catch (err) {
    console.error('Falha ao submeter missão:', err);
    alert('Erro de comunicação com o souls_server.');
  }
}

function copyResearchCommand() {
  const cmdText = document.getElementById('text-copy-command');
  if (cmdText && cmdText.innerText) {
    navigator.clipboard.writeText(cmdText.innerText);
    alert('Comando copiado para a área de transferência! Cole no chat do Hermes Agent.');
  }
}

// ============================================================================
// MISSION LISTING & SSE TELEMETRY
// ============================================================================

async function loadResearchHubMissions() {
  try {
    const res = await fetch('/api/v1/research/missions');
    const data = await res.json();
    if (data.status === 'success') {
      currentResearchMissions = data.missions || [];
      updateMissionCountBadges();
      renderActiveMissions();
      renderLibraryMissions();
    }
  } catch (err) {
    console.error('Falha ao carregar missões:', err);
  }
}

function updateMissionCountBadges() {
  const activeCount = currentResearchMissions.filter(m =>
    m.status === 'queued' || m.status === 'claimed' || m.status === 'researching' || m.status === 'synthesizing' || m.status === 'verifying'
  ).length;

  const libraryCount = currentResearchMissions.filter(m =>
    m.status === 'confirmed' || m.status === 'draft' || m.status === 'blocked' || m.status === 'failed' || m.status === 'cancelled'
  ).length;

  const topBadge = document.getElementById('badge-research-count');
  if (topBadge) topBadge.innerText = activeCount;

  const activeBadge = document.getElementById('badge-research-active-count');
  if (activeBadge) activeBadge.innerText = activeCount;

  const libBadge = document.getElementById('badge-research-library-count');
  if (libBadge) libBadge.innerText = libraryCount;
}

function renderActiveMissions() {
  const container = document.getElementById('container-active-missions');
  if (!container) return;

  const activeList = currentResearchMissions.filter(m =>
    m.status === 'queued' || m.status === 'claimed' || m.status === 'researching' || m.status === 'synthesizing' || m.status === 'verifying'
  );

  if (activeList.length === 0) {
    container.innerHTML = `
      <div class="card" style="text-align: center; padding: 40px; color: #64748b;">
        <span>📡 Nenhuma missão de pesquisa em andamento no momento.</span>
        <div style="margin-top: 10px;">
          <button class="btn btn-sm btn-primary" onclick="switchResearchSubtab('new-mission')">Criar Nova Missão</button>
        </div>
      </div>
    `;
    return;
  }

  container.innerHTML = activeList.map(m => {
    const turnsPct = Math.min(100, ((m.turns_used || 0) / 90) * 100);
    const costPct = Math.min(100, ((m.spent_usd || 0) / (m.cap_usd || 1.50)) * 100);

    return `
      <div class="card" style="padding: 18px; border-left: 4px solid var(--cyan, #38bdf8);">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 10px;">
          <div>
            <div style="display: flex; gap: 8px; align-items: center; margin-bottom: 4px;">
              <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: #38bdf8;">${m.status.toUpperCase()}</span>
              <span style="font-size: 0.75rem; color: #64748b; font-family: monospace;">${m.run_id}</span>
            </div>
            <h3 style="font-size: 1rem; font-weight: 600; color: #f8fafc; margin: 0;">${escapeHtml(m.question)}</h3>
          </div>
          <div style="display: flex; gap: 6px;">
            <button class="btn btn-xs btn-outline" onclick="openMissionSteeringModal('${m.run_id}')">🧭 Steering</button>
            <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${m.run_id}')">Inspecionar</button>
            <button class="btn btn-xs btn-danger" onclick="cancelResearchMission('${m.run_id}')">Cancelar</button>
          </div>
        </div>

        <!-- Telemetry Gauges -->
        <div class="grid grid-3" style="gap: 12px; margin-top: 14px; background: rgba(15, 23, 42, 0.6); padding: 12px; border-radius: 6px; border: 1px solid #1e293b;">
          <div>
            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; margin-bottom: 4px;">
              <span style="color: #94a3b8;">Turnos do Orçamento:</span>
              <strong style="color: #f8fafc;">${m.turns_used || 0} / 90</strong>
            </div>
            <div style="height: 6px; background: #334155; border-radius: 9999px; overflow: hidden;">
              <div style="width: ${turnsPct}%; height: 100%; background: ${turnsPct > 80 ? '#ef4444' : '#38bdf8'};"></div>
            </div>
          </div>
          <div>
            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; margin-bottom: 4px;">
              <span style="color: #94a3b8;">Despesa Acumulada:</span>
              <strong style="color: #facc15;">$${(m.spent_usd || 0).toFixed(2)} / $${(m.cap_usd || 1.50).toFixed(2)}</strong>
            </div>
            <div style="height: 6px; background: #334155; border-radius: 9999px; overflow: hidden;">
              <div style="width: ${costPct}%; height: 100%; background: ${costPct > 85 ? '#ef4444' : '#10b981'};"></div>
            </div>
          </div>
          <div>
            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; margin-bottom: 4px;">
              <span style="color: #94a3b8;">Ronda IterResearch:</span>
              <strong style="color: #cbd5e1;">Ronda #${m.round || 1}</strong>
            </div>
            <span style="font-size: 0.7rem; color: #10b981;">● Agente Ativo em Execução</span>
          </div>
        </div>
      </div>
    `;
  }).join('');
}

function renderLibraryMissions() {
  const listEl = document.getElementById('list-library-items');
  if (!listEl) return;

  if (currentResearchMissions.length === 0) {
    listEl.innerHTML = '<div style="color:#64748b; font-size:0.75rem; text-align:center; padding:20px;">Nenhuma pesquisa encontrada.</div>';
    return;
  }

  listEl.innerHTML = currentResearchMissions.map(m => `
    <div onclick="inspectMissionDetail('${m.run_id}')" style="background:#0f172a; border:1px solid #1e293b; padding:10px; border-radius:6px; cursor:pointer; transition:border-color 0.15s;" onmouseover="this.style.borderColor='#38bdf8'" onmouseout="this.style.borderColor='#1e293b'">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:4px;">
        <span class="badge" style="font-size:0.65rem; padding:1px 6px;">${m.status}</span>
        <span style="font-size:0.65rem; color:#64748b;">${(m.created_at || '').substring(0, 10)}</span>
      </div>
      <div style="font-size:0.8rem; font-weight:600; color:#f8fafc; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">
        ${escapeHtml(m.question)}
      </div>
      <div style="font-size:0.7rem; color:#94a3b8; margin-top:4px; display:flex; justify-content:space-between;">
        <span>$${(m.spent_usd || 0).toFixed(2)}</span>
        <span>${m.turns_used || 0} turnos</span>
      </div>
    </div>
  `).join('');
}

async function inspectMissionDetail(run_id) {
  try {
    const res = await fetch(`/api/v1/research/mission?run_id=${encodeURIComponent(run_id)}`);
    const data = await res.json();
    if (data.status === 'success') {
      selectedMissionDetail = data;
      renderMissionViewer(data);
    }
  } catch (err) {
    console.error('Falha ao inspecionar missão:', err);
  }
}

function renderMissionViewer(data) {
  const viewer = document.getElementById('box-library-viewer');
  if (!viewer) return;

  const m = data.mission;
  const events = data.events || [];
  const evolvingReport = data.evolving_report || '*(Relatório central ainda não iniciado pelo orquestrador)*';

  viewer.innerHTML = `
    <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:16px;">
      <div>
        <div style="display:flex; gap:8px; align-items:center; margin-bottom:4px;">
          <span class="badge" style="background:rgba(56,189,248,0.15); color:#38bdf8;">${m.status.toUpperCase()}</span>
          <span style="font-family:monospace; font-size:0.75rem; color:#64748b;">${m.run_id}</span>
        </div>
        <h2 style="font-size:1.15rem; font-weight:700; color:#fff; margin:0;">${escapeHtml(m.brief.question)}</h2>
      </div>
      <div style="display:flex; gap:6px; flex-wrap:wrap;">
        <button class="btn btn-xs btn-outline" onclick="inspectEpistemicReport('${m.run_id}')">🏷️ Leitor Epistêmico</button>
        <button class="btn btn-xs btn-outline" onclick="openRefreshDeltaModal('${m.run_id}')">♻️ Atualizar (Delta)</button>
        <button class="btn btn-xs btn-outline" onclick="openBranchSectionModal('${m.run_id}')">🌿 Ramificar</button>
        <button class="btn btn-xs btn-outline" onclick="openDiffModal('${m.run_id}')">🔍 Diff</button>
        <button class="btn btn-xs btn-primary" onclick="promoteReportModal('${m.run_id}')">📓 Promover</button>
        <button class="btn btn-xs btn-outline" onclick="copyReportMarkdown()">📥 Copiar .md</button>
        <button class="btn btn-xs btn-danger" onclick="archiveResearchMission('${m.run_id}')">Arquivar</button>
      </div>
    </div>

    <!-- Timeline Resumida -->
    <div style="margin-bottom:20px; background:#0f172a; padding:12px; border-radius:6px; border:1px solid #1e293b;">
      <span style="font-size:0.75rem; color:#94a3b8; font-weight:600; display:block; margin-bottom:8px;">TIMELINE DE EVENTOS (${events.length}):</span>
      <div style="max-height:120px; overflow-y:auto; display:flex; flex-direction:column; gap:4px; font-size:0.75rem; font-family:monospace;">
        ${events.length === 0 ? '<span style="color:#64748b;">Nenhum evento registrado ainda.</span>' : events.map(e => `
          <div><span style="color:#64748b;">[${(e.timestamp || '').substring(11, 19)}]</span> <strong style="color:#38bdf8;">${e.phase}</strong> (Ronda #${e.round}): <span style="color:#cbd5e1;">${escapeHtml(e.summary)}</span></div>
        `).join('')}
      </div>
    </div>

    <!-- Evolving Report Markdown -->
    <div style="background:#020617; border:1px solid #1e293b; border-radius:8px; padding:16px;">
      <span style="font-size:0.75rem; color:#94a3b8; font-weight:600; display:block; margin-bottom:10px;">RELATÓRIO CENTRAL EM EVOLUÇÃO (01_evolving_report.md):</span>
      <div id="text-report-markdown" style="font-size:0.85rem; color:#e2e8f0; line-height:1.6; white-space:pre-wrap; max-height:400px; overflow-y:auto;">${escapeHtml(evolvingReport)}</div>
    </div>
  `;
}

// ============================================================================
// FASE 3: RELATÓRIOS VIVOS (DELTA), RAMIFICAÇÃO, DIFF & PROMOÇÃO OBSIDIAN
// ============================================================================

async function openRefreshDeltaModal(run_id) {
  if (!confirm(`Deseja lançar uma nova versão viva (Delta v2+) para a pesquisa ${run_id}?\nO Hermes reaproveitará o acervo de fontes e buscará atualizações e fatos recentes.`)) return;

  try {
    const res = await fetch('/api/v1/research/mission/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ parent_run_id: run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Versão Delta criada e enfileirada: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao criar delta'));
    }
  } catch (err) {
    console.error('Falha ao atualizar delta:', err);
  }
}

async function openBranchSectionModal(run_id) {
  const heading = prompt(`[RAMIFICAR SEÇÃO — ${run_id}]\nDigite o título ou heading do tópico que deseja aprofundar de forma autônoma:\n(Ex: "Mapeamento empírico de latência no ReFS")`);
  if (!heading || !heading.trim()) return;

  const customQ = prompt(`Pergunta específica para este subagente (ou deixe em branco para derivar automaticamente):`);

  try {
    const res = await fetch('/api/v1/research/mission/branch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parent_run_id: run_id,
        heading_title: heading.trim(),
        custom_question: customQ && customQ.trim() ? customQ.trim() : undefined
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Missão ramificada criada e enfileirada com sucesso: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao ramificar missão'));
    }
  } catch (err) {
    console.error('Falha ao ramificar:', err);
  }
}

async function openDiffModal(run_id) {
  const viewer = document.getElementById('box-library-viewer');
  if (!viewer) return;

  try {
    const res = await fetch(`/api/v1/research/diff?run_id=${encodeURIComponent(run_id)}`);
    const data = await res.json();
    if (data.status === 'success') {
      const d = data.diff;
      viewer.innerHTML = `
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px;">
          <div>
            <h2 style="font-size:1.15rem; font-weight:700; color:#fff; margin:0;">🔍 Comparador Myers Diff de Versões</h2>
            <span style="font-size:0.75rem; color:#94a3b8;">Missão: ${run_id} (Adições: +${d.additions}, Remoções: -${d.deletions})</span>
          </div>
          <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Voltar ao Relatório</button>
        </div>
        <div style="background:#020617; border:1px solid #1e293b; border-radius:8px; padding:12px; font-family:monospace; font-size:0.75rem; max-height:450px; overflow-y:auto;">
          ${d.diff_lines.length === 0 ? '<span style="color:#64748b;">Nenhuma alteração entre as versões baseline e atual.</span>' : d.diff_lines.map(line => {
            if (line.startsWith('+')) {
              return `<div style="background:rgba(16,185,129,0.15); color:#10b981;">${escapeHtml(line)}</div>`;
            } else if (line.startsWith('-')) {
              return `<div style="background:rgba(239,68,68,0.15); color:#ef4444;">${escapeHtml(line)}</div>`;
            } else {
              return `<div style="color:#94a3b8;">${escapeHtml(line)}</div>`;
            }
          }).join('')}
        </div>
      `;
    } else {
      alert('Erro ao calcular diff: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao carregar diff:', err);
  }
}

async function promoteReportModal(run_id) {
  if (!confirm(`Deseja promover este relatório verificado para o Cofre Obsidian e Hipocampo L3?\nO Privacy Shield higienizará localmente quaisquer dados sensíveis (PII) antes de persistir.`)) return;

  try {
    const res = await fetch('/api/v1/research/promote', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        run_id,
        targets: ['obsidian', 'hippocampus'],
        dry_run: false
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      const p = data.promotion;
      alert(`Relatório promovido com sucesso!\n- Obsidian: ${p.obsidian_file_path || 'raw/research/'}\n- PIIs Higienizados: ${p.pii_redactions_count}\n- Hipocampo L3: Espelhado`);
      inspectMissionDetail(run_id);
    } else {
      alert('Erro na promoção: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao promover relatório:', err);
  }
}


function copyReportMarkdown() {
  const el = document.getElementById('text-report-markdown');
  if (el) {
    navigator.clipboard.writeText(el.innerText);
    alert('Relatório copiado em formato Markdown!');
  }
}

async function cancelResearchMission(run_id) {
  if (!confirm(`Deseja realmente cancelar a missão ${run_id}?`)) return;
  try {
    const res = await fetch('/api/v1/research/mission/cancel', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert('Missão cancelada.');
      loadResearchHubMissions();
    } else {
      alert('Erro: ' + data.message);
    }
  } catch (err) {
    console.error('Falha ao cancelar missão:', err);
  }
}

async function archiveResearchMission(run_id) {
  if (!confirm(`Deseja arquivar com segurança a missão ${run_id} para .souls_data/research/_archive/?`)) return;
  try {
    const res = await fetch('/api/v1/research/mission/archive', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert('Missão arquivada.');
      loadResearchHubMissions();
      const viewer = document.getElementById('box-library-viewer');
      if (viewer) viewer.innerHTML = '<div style="text-align:center; padding:60px; color:#64748b;">Missão arquivada.</div>';
    } else {
      alert('Erro: ' + data.message);
    }
  } catch (err) {
    console.error('Falha ao arquivar missão:', err);
  }
}

// ============================================================================
// DOCTOR ENGINE
// ============================================================================

async function loadResearchDoctorReport() {
  const listEl = document.getElementById('list-doctor-checks');
  if (!listEl) return;
  listEl.innerHTML = '<div style="color:#64748b; font-size:0.8rem; padding:10px;">Executando diagnóstico de prontidão...</div>';

  try {
    const res = await fetch('/api/v1/research/doctor');
    const data = await res.json();
    if (data.status === 'success') {
      const doc = data.doctor;
      const statusBadge = document.getElementById('badge-research-doctor-status');
      if (statusBadge) {
        if (doc.overall_status === 'ready') {
          statusBadge.innerText = 'Pronto';
          statusBadge.style.color = '#10b981';
        } else if (doc.overall_status === 'needs_provisioning') {
          statusBadge.innerText = 'Pendente';
          statusBadge.style.color = '#facc15';
        } else {
          statusBadge.innerText = 'Incompleto';
          statusBadge.style.color = '#ef4444';
        }
      }

      listEl.innerHTML = doc.checks.map(c => `
        <div style="display:flex; justify-content:space-between; align-items:center; background:#0f172a; border:1px solid #1e293b; padding:10px 14px; border-radius:6px;">
          <div>
            <div style="display:flex; gap:8px; align-items:center;">
              <span style="font-size:0.85rem; font-weight:600; color:#f8fafc;">${escapeHtml(c.label)}</span>
              <span class="badge" style="font-size:0.65rem;">${c.category}</span>
            </div>
            <div style="font-size:0.75rem; color:#94a3b8; margin-top:2px;">${escapeHtml(c.details)}</div>
          </div>
          <div>
            ${c.status === 'ok'
              ? '<span style="color:#10b981; font-weight:700; font-size:0.85rem;">✓ OK</span>'
              : c.status === 'warn'
              ? '<span style="color:#facc15; font-weight:700; font-size:0.85rem;">⚠️ Atenção</span>'
              : '<span style="color:#ef4444; font-weight:700; font-size:0.85rem;">✕ Erro</span>'}
          </div>
        </div>
      `).join('');
    }
  } catch (err) {
    console.error('Falha no diagnóstico doctor:', err);
    listEl.innerHTML = '<div style="color:#ef4444; font-size:0.8rem; padding:10px;">Falha ao carregar relatório do Doctor.</div>';
  }
}

// ============================================================================
// SSE STREAM INITIALIZATION
// ============================================================================

function initResearchSseStream() {
  if (researchSseSource) return;
  try {
    researchSseSource = new EventSource('/api/v1/research/stream');
    researchSseSource.onmessage = function (event) {
      try {
        const payload = JSON.parse(event.data);
        handleResearchSseEvent(payload);
      } catch (e) {
        // Ignora keep-alive
      }
    };
    researchSseSource.onerror = function () {
      researchSseSource.close();
      researchSseSource = null;
      setTimeout(initResearchSseStream, 5000);
    };
  } catch (err) {
    console.error('SSE connection error:', err);
  }
}

function handleResearchSseEvent(event) {
  // Recarrega missões quando houver mudança de estado ou progresso
  loadResearchHubMissions();
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

// Boot on DOM ready
document.addEventListener('DOMContentLoaded', () => {
  initResearchSseStream();
});

// ============================================================================
// IN-FLIGHT STEERING & OPERATOR INTERVENTION (Fase 2)
// ============================================================================

function openMissionSteeringModal(run_id) {
  const directive = prompt(`[STEERING EM VOO — ${run_id}]\nDigite a diretriz a ser injetada para os subagentes nesta ronda:\n(Ex: "Ignore discussões sobre Mac/Linux; foque estritamente em Windows 11 ReFS")`);
  if (directive === null) return;

  sendMissionSteering(run_id, { directive: directive.trim() || undefined });
}

async function sendMissionSteering(run_id, options = {}) {
  try {
    const res = await fetch('/api/v1/research/mission/steer', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        run_id,
        directive: options.directive,
        pause: options.pause,
        force_synthesis: options.force_synthesis,
        cancel: options.cancel
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert('Comando de steering enviado com sucesso!');
      loadResearchHubMissions();
      if (selectedMissionDetail && selectedMissionDetail.mission.run_id === run_id) {
        inspectMissionDetail(run_id);
      }
    } else {
      alert('Erro ao enviar steering: ' + (data.message || 'Desconhecido'));
    }
  } catch (err) {
    console.error('Falha ao enviar steering:', err);
  }
}

// ============================================================================
// EPISTEMIC REPORT VIEWER (Fase 2)
// ============================================================================

async function inspectEpistemicReport(run_id) {
  try {
    const res = await fetch(`/api/v1/research/report?run_id=${encodeURIComponent(run_id)}`);
    const data = await res.json();
    if (data.status === 'success') {
      renderEpistemicViewer(data.report, run_id);
    } else {
      alert('Relatório ainda não finalizado: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao obter relatório epistêmico:', err);
  }
}

function renderEpistemicViewer(rep, run_id) {
  const viewer = document.getElementById('box-library-viewer');
  if (!viewer) return;

  const tc = rep.tag_counts;
  const citations = rep.citations || [];

  viewer.innerHTML = `
    <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:14px;">
      <div>
        <div style="display:flex; gap:8px; align-items:center; margin-bottom:4px;">
          <span class="badge" style="background:${rep.is_verified ? 'rgba(16,185,129,0.2)' : 'rgba(234,179,8,0.2)'}; color:${rep.is_verified ? '#10b981' : '#facc15'};">
            ${rep.is_verified ? '✓ VERIFICADO READ-ONLY' : '⚡ RASCUNHO EM PROGRESSO'}
          </span>
          <span style="font-family:monospace; font-size:0.75rem; color:#64748b;">${run_id}</span>
        </div>
        <h2 style="font-size:1.15rem; font-weight:700; color:#fff; margin:0;">Leitor Epistêmico de Evidências</h2>
      </div>
      <div style="display:flex; gap:6px;">
        <button class="btn btn-xs btn-outline" onclick="copyReportMarkdown()">📥 Copiar .md</button>
        <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Timeline</button>
      </div>
    </div>

    <!-- Chips Epistêmicos -->
    <div style="display:flex; gap:10px; flex-wrap:wrap; margin-bottom:14px;">
      <span class="badge" style="background:rgba(16,185,129,0.15); color:#10b981; border:1px solid rgba(16,185,129,0.3); font-size:0.75rem; padding:4px 10px;">
        [VERIFIED]: <strong>${tc.verified}</strong>
      </span>
      <span class="badge" style="background:rgba(56,189,248,0.15); color:#38bdf8; border:1px solid rgba(56,189,248,0.3); font-size:0.75rem; padding:4px 10px;">
        [SOURCED]: <strong>${tc.sourced}</strong>
      </span>
      <span class="badge" style="background:rgba(234,179,8,0.15); color:#facc15; border:1px solid rgba(234,179,8,0.3); font-size:0.75rem; padding:4px 10px;">
        [REASONED]: <strong>${tc.reasoned}</strong>
      </span>
      <span class="badge" style="background:rgba(148,163,184,0.15); color:#cbd5e1; border:1px solid rgba(148,163,184,0.3); font-size:0.75rem; padding:4px 10px;">
        [ESTIMATED]: <strong>${tc.estimated}</strong>
      </span>
      ${rep.contradictions_count > 0 ? `
        <span class="badge" style="background:rgba(239,68,68,0.15); color:#ef4444; border:1px solid rgba(239,68,68,0.3); font-size:0.75rem; padding:4px 10px;">
          Contradições Mapeadas: <strong>${rep.contradictions_count}</strong>
        </span>
      ` : ''}
    </div>

    <!-- Fontes & Citações Extraídas -->
    ${citations.length > 0 ? `
      <div style="margin-bottom:14px; background:#0f172a; padding:10px 12px; border-radius:6px; border:1px solid #1e293b;">
        <span style="font-size:0.75rem; color:#94a3b8; font-weight:600; display:block; margin-bottom:6px;">FONTES IDENTIFICADAS (${citations.length}):</span>
        <div style="display:flex; flex-wrap:wrap; gap:6px; max-height:80px; overflow-y:auto;">
          ${citations.map(c => `
            <a href="${c.url}" target="_blank" rel="noopener" style="font-size:0.7rem; background:#1e293b; color:#38bdf8; padding:2px 8px; border-radius:4px; text-decoration:none; display:inline-flex; align-items:center; gap:4px;">
              <span>[${c.citation_num}]</span> ${escapeHtml(c.domain)}
              <span style="color:#64748b; font-size:0.65rem;">(${c.tier})</span>
            </a>
          `).join('')}
        </div>
      </div>
    ` : ''}

    <!-- Visualizador de Markdown com Renderização de Tags -->
    <div style="background:#020617; border:1px solid #1e293b; border-radius:8px; padding:16px;">
      <div id="text-report-markdown" style="font-size:0.85rem; color:#e2e8f0; line-height:1.6; white-space:pre-wrap; max-height:450px; overflow-y:auto;">${renderHighlightedEpistemicReport(rep.raw_markdown)}</div>
    </div>
  `;
}

// ============================================================================
// FASE 3: RELATÓRIOS VIVOS (DELTA), RAMIFICAÇÃO, DIFF & PROMOÇÃO OBSIDIAN
// ============================================================================

async function openRefreshDeltaModal(run_id) {
  if (!confirm(`Deseja lançar uma nova versão viva (Delta v2+) para a pesquisa ${run_id}?\nO Hermes reaproveitará o acervo de fontes e buscará atualizações e fatos recentes.`)) return;

  try {
    const res = await fetch('/api/v1/research/mission/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ parent_run_id: run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Versão Delta criada e enfileirada: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao criar delta'));
    }
  } catch (err) {
    console.error('Falha ao atualizar delta:', err);
  }
}

async function openBranchSectionModal(run_id) {
  const heading = prompt(`[RAMIFICAR SEÇÃO — ${run_id}]\nDigite o título ou heading do tópico que deseja aprofundar de forma autônoma:\n(Ex: "Mapeamento empírico de latência no ReFS")`);
  if (!heading || !heading.trim()) return;

  const customQ = prompt(`Pergunta específica para este subagente (ou deixe em branco para derivar automaticamente):`);

  try {
    const res = await fetch('/api/v1/research/mission/branch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parent_run_id: run_id,
        heading_title: heading.trim(),
        custom_question: customQ && customQ.trim() ? customQ.trim() : undefined
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Missão ramificada criada e enfileirada com sucesso: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao ramificar missão'));
    }
  } catch (err) {
    console.error('Falha ao ramificar:', err);
  }
}

async function openDiffModal(run_id) {
  const viewer = document.getElementById('box-library-viewer');
  if (!viewer) return;

  try {
    const res = await fetch(`/api/v1/research/diff?run_id=${encodeURIComponent(run_id)}`);
    const data = await res.json();
    if (data.status === 'success') {
      const d = data.diff;
      viewer.innerHTML = `
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px;">
          <div>
            <h2 style="font-size:1.15rem; font-weight:700; color:#fff; margin:0;">🔍 Comparador Myers Diff de Versões</h2>
            <span style="font-size:0.75rem; color:#94a3b8;">Missão: ${run_id} (Adições: +${d.additions}, Remoções: -${d.deletions})</span>
          </div>
          <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Voltar ao Relatório</button>
        </div>
        <div style="background:#020617; border:1px solid #1e293b; border-radius:8px; padding:12px; font-family:monospace; font-size:0.75rem; max-height:450px; overflow-y:auto;">
          ${d.diff_lines.length === 0 ? '<span style="color:#64748b;">Nenhuma alteração entre as versões baseline e atual.</span>' : d.diff_lines.map(line => {
            if (line.startsWith('+')) {
              return `<div style="background:rgba(16,185,129,0.15); color:#10b981;">${escapeHtml(line)}</div>`;
            } else if (line.startsWith('-')) {
              return `<div style="background:rgba(239,68,68,0.15); color:#ef4444;">${escapeHtml(line)}</div>`;
            } else {
              return `<div style="color:#94a3b8;">${escapeHtml(line)}</div>`;
            }
          }).join('')}
        </div>
      `;
    } else {
      alert('Erro ao calcular diff: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao carregar diff:', err);
  }
}

async function promoteReportModal(run_id) {
  if (!confirm(`Deseja promover este relatório verificado para o Cofre Obsidian e Hipocampo L3?\nO Privacy Shield higienizará localmente quaisquer dados sensíveis (PII) antes de persistir.`)) return;

  try {
    const res = await fetch('/api/v1/research/promote', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        run_id,
        targets: ['obsidian', 'hippocampus'],
        dry_run: false
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      const p = data.promotion;
      alert(`Relatório promovido com sucesso!\n- Obsidian: ${p.obsidian_file_path || 'raw/research/'}\n- PIIs Higienizados: ${p.pii_redactions_count}\n- Hipocampo L3: Espelhado`);
      inspectMissionDetail(run_id);
    } else {
      alert('Erro na promoção: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao promover relatório:', err);
  }
}


function renderHighlightedEpistemicReport(raw) {
  return escapeHtml(raw)
    .replace(/\[VERIFIED\]/g, '<span style="background:rgba(16,185,129,0.25); color:#10b981; font-weight:700; padding:1px 6px; border-radius:3px;">[VERIFIED]</span>')
    .replace(/\[SOURCED\]/g, '<span style="background:rgba(56,189,248,0.25); color:#38bdf8; font-weight:700; padding:1px 6px; border-radius:3px;">[SOURCED]</span>')
    .replace(/\[REASONED\]/g, '<span style="background:rgba(234,179,8,0.25); color:#facc15; font-weight:700; padding:1px 6px; border-radius:3px;">[REASONED]</span>')
    .replace(/\[ESTIMATED\]/g, '<span style="background:rgba(148,163,184,0.25); color:#cbd5e1; font-weight:700; padding:1px 6px; border-radius:3px;">[ESTIMATED]</span>');
}

// ============================================================================
// 1-CLICK PROVISIONING FLOW (Fase 2)
// ============================================================================

async function triggerProvisioningFlow(dryRun) {
  const box = document.getElementById('box-provisioning-diff');
  if (box) {
    box.style.display = 'block';
    box.innerHTML = '<div style="color:#38bdf8; font-size:0.8rem;">Calculando modificações necessárias no ecossistema Hermes...</div>';
  }

  try {
    const res = await fetch('/api/v1/research/provision', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ dry_run: dryRun })
    });
    const data = await res.json();
    if (data.status === 'success') {
      renderProvisioningBox(data.result);
      if (!dryRun) {
        loadResearchDoctorReport();
      }
    } else {
      if (box) box.innerHTML = `<div style="color:#ef4444; font-size:0.8rem;">Erro no provisionamento: ${escapeHtml(data.message)}</div>`;
    }
  } catch (err) {
    console.error('Falha no provisionamento:', err);
  }
}

function renderProvisioningBox(result) {
  const box = document.getElementById('box-provisioning-diff');
  if (!box) return;

  const changes = result.changes || [];
  if (changes.length === 0) {
    box.innerHTML = `
      <div style="display:flex; justify-content:space-between; align-items:center;">
        <span style="color:#10b981; font-size:0.85rem; font-weight:600;">✓ Ecossistema 100% alinhado! Nenhuma modificação pendente.</span>
        <button class="btn btn-xs btn-outline" onclick="this.parentElement.parentElement.style.display='none'">Fechar</button>
      </div>
    `;
    return;
  }

  box.innerHTML = `
    <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:10px;">
      <div>
        <strong style="font-size:0.9rem; color:#f8fafc;">${result.dry_run ? '🔍 Prévia de Provisionamento' : '✅ Provisionamento Aplicado'}</strong>
        <span style="font-size:0.75rem; color:#94a3b8; display:block;">${escapeHtml(result.message)}</span>
      </div>
      <div style="display:flex; gap:6px;">
        ${result.dry_run ? `
          <button class="btn btn-xs btn-success" onclick="triggerProvisioningFlow(false)">🚀 Aplicar Modificações Agora</button>
        ` : ''}
        <button class="btn btn-xs btn-outline" onclick="document.getElementById('box-provisioning-diff').style.display='none'">Fechar</button>
      </div>
    </div>
    <div style="display:flex; flex-direction:column; gap:8px; max-height:220px; overflow-y:auto;">
      ${changes.map(c => `
        <div style="background:#020617; border:1px solid #1e293b; padding:8px 10px; border-radius:4px; font-size:0.75rem;">
          <div style="display:flex; justify-content:space-between; margin-bottom:4px;">
            <strong style="color:#38bdf8;">${escapeHtml(c.target_file)}</strong>
            <span class="badge" style="font-size:0.65rem;">${c.change_type}</span>
          </div>
          <div style="color:#94a3b8; font-size:0.7rem;">${escapeHtml(c.explanation)}</div>
        </div>
      `).join('')}
    </div>
  `;
}

// ============================================================================
// FASE 3: RELATÓRIOS VIVOS (DELTA), RAMIFICAÇÃO, DIFF & PROMOÇÃO OBSIDIAN
// ============================================================================

async function openRefreshDeltaModal(run_id) {
  if (!confirm(`Deseja lançar uma nova versão viva (Delta v2+) para a pesquisa ${run_id}?\nO Hermes reaproveitará o acervo de fontes e buscará atualizações e fatos recentes.`)) return;

  try {
    const res = await fetch('/api/v1/research/mission/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ parent_run_id: run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Versão Delta criada e enfileirada: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao criar delta'));
    }
  } catch (err) {
    console.error('Falha ao atualizar delta:', err);
  }
}

async function openBranchSectionModal(run_id) {
  const heading = prompt(`[RAMIFICAR SEÇÃO — ${run_id}]\nDigite o título ou heading do tópico que deseja aprofundar de forma autônoma:\n(Ex: "Mapeamento empírico de latência no ReFS")`);
  if (!heading || !heading.trim()) return;

  const customQ = prompt(`Pergunta específica para este subagente (ou deixe em branco para derivar automaticamente):`);

  try {
    const res = await fetch('/api/v1/research/mission/branch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        parent_run_id: run_id,
        heading_title: heading.trim(),
        custom_question: customQ && customQ.trim() ? customQ.trim() : undefined
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      alert(`Missão ramificada criada e enfileirada com sucesso: ${data.mission.run_id}`);
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      alert('Erro: ' + (data.message || 'Falha ao ramificar missão'));
    }
  } catch (err) {
    console.error('Falha ao ramificar:', err);
  }
}

async function openDiffModal(run_id) {
  const viewer = document.getElementById('box-library-viewer');
  if (!viewer) return;

  try {
    const res = await fetch(`/api/v1/research/diff?run_id=${encodeURIComponent(run_id)}`);
    const data = await res.json();
    if (data.status === 'success') {
      const d = data.diff;
      viewer.innerHTML = `
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px;">
          <div>
            <h2 style="font-size:1.15rem; font-weight:700; color:#fff; margin:0;">🔍 Comparador Myers Diff de Versões</h2>
            <span style="font-size:0.75rem; color:#94a3b8;">Missão: ${run_id} (Adições: +${d.additions}, Remoções: -${d.deletions})</span>
          </div>
          <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Voltar ao Relatório</button>
        </div>
        <div style="background:#020617; border:1px solid #1e293b; border-radius:8px; padding:12px; font-family:monospace; font-size:0.75rem; max-height:450px; overflow-y:auto;">
          ${d.diff_lines.length === 0 ? '<span style="color:#64748b;">Nenhuma alteração entre as versões baseline e atual.</span>' : d.diff_lines.map(line => {
            if (line.startsWith('+')) {
              return `<div style="background:rgba(16,185,129,0.15); color:#10b981;">${escapeHtml(line)}</div>`;
            } else if (line.startsWith('-')) {
              return `<div style="background:rgba(239,68,68,0.15); color:#ef4444;">${escapeHtml(line)}</div>`;
            } else {
              return `<div style="color:#94a3b8;">${escapeHtml(line)}</div>`;
            }
          }).join('')}
        </div>
      `;
    } else {
      alert('Erro ao calcular diff: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao carregar diff:', err);
  }
}

async function promoteReportModal(run_id) {
  if (!confirm(`Deseja promover este relatório verificado para o Cofre Obsidian e Hipocampo L3?\nO Privacy Shield higienizará localmente quaisquer dados sensíveis (PII) antes de persistir.`)) return;

  try {
    const res = await fetch('/api/v1/research/promote', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        run_id,
        targets: ['obsidian', 'hippocampus'],
        dry_run: false
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      const p = data.promotion;
      alert(`Relatório promovido com sucesso!\n- Obsidian: ${p.obsidian_file_path || 'raw/research/'}\n- PIIs Higienizados: ${p.pii_redactions_count}\n- Hipocampo L3: Espelhado`);
      inspectMissionDetail(run_id);
    } else {
      alert('Erro na promoção: ' + (data.message || ''));
    }
  } catch (err) {
    console.error('Falha ao promover relatório:', err);
  }
}



