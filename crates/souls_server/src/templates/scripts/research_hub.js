/**
 * Research Hub & Deep Research Mission Control Client (v0.7.0)
 *
 * Handles:
 * - Subtab navigation within "Pesquisa Profunda"
 * - Dynamic scoping, STORM perspective drafting and FinOps pre-flight estimation
 * - Real-time SSE telemetry stream (/api/v1/research/stream)
 * - Mission lifecycle operations (enqueue, draft, cancel, archive)
 * - Ecosystem Doctor readiness inspector
 */

let currentResearchMissions = [];
let selectedMissionDetail = null;
let researchSseSource = null;

const toast = (msg, type = 'info') => {
  if (window.showToast) window.showToast(msg, type);
  else console.log(`[${type}] ${msg}`);
};

// appConfirm and appPrompt are provided globally by dashboard.js (window.appConfirm, window.appPrompt)
const getAppConfirm = () => window.appConfirm || window.confirm;
const getAppPrompt = () => window.appPrompt || window.prompt;

function filterLibraryMissions() {
  const inputEl = document.getElementById('input-search-library') || document.getElementById('input-filter-library');
  const query = (inputEl?.value || '').toLowerCase().trim();
  const rows = document.querySelectorAll('.library-mission-row');
  rows.forEach(r => {
    const text = ((r.getAttribute('data-search-text') || '') + ' ' + (r.textContent || '')).toLowerCase();
    r.style.display = (!query || text.includes(query)) ? '' : 'none';
  });
}
window.filterLibraryMissions = filterLibraryMissions;

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
  } else if (subtabId === 'quality') {
    loadResearchScorecard();
    loadDomainTrustLedger();
  } else if (subtabId === 'doctor') {
    loadResearchDoctorReport();
    loadResearchSettings();
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
    toast('Por favor, informe a pergunta central da pesquisa.', 'warning');
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
      toast('Erro ao gerar plano: ' + (data.message || 'Desconhecido'), 'error');
    }
  } catch (err) {
    console.error('Falha ao gerar plano de pesquisa:', err);
    toast('Erro de comunicação com o souls_server.', 'error');
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
      <div class="u-d-flex u-gap-6p u-items-center">
        <span class="u-text-cyan u-s-a2b44b">#${idx+1}</span>
        <input type="text" class="input-base brief-perspective-item u-flex-1 u-s-6682a1" value="${escapeHtml(p)}">
      </div>
    `).join('');
  }

  // Subtopics
  const sList = document.getElementById('list-brief-subtopics');
  if (sList) {
    sList.innerHTML = brief.subtopics.map((s, idx) => `
      <div class="u-d-flex u-gap-6p u-items-center">
        <span class="u-text-green u-s-a2b44b">•</span>
        <input type="text" class="input-base brief-subtopic-item u-flex-1 u-s-6682a1" value="${escapeHtml(s)}">
      </div>
    `).join('');
  }

  // Exclusions
  const eList = document.getElementById('list-brief-exclusions');
  if (eList) {
    eList.innerHTML = brief.exclusions.map((e, idx) => `
      <div class="u-d-flex u-gap-6p u-items-center">
        <span class="u-text-red u-s-a2b44b">${uiIcon('x', 'icon-xs')}</span>
        <input type="text" class="input-base brief-exclusion-item u-flex-1 u-s-6682a1" value="${escapeHtml(e)}">
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
    toast('Informe a pergunta da pesquisa.', 'warning');
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

      toast(queueImmediately ? 'Missão enfileirada com sucesso para o Hermes Agent!' : 'Rascunho salvo com sucesso!', 'success');
      loadResearchHubMissions();
      if (queueImmediately) {
        switchResearchSubtab('active');
      }
    } else {
      toast('Erro ao criar missão: ' + (data.message || 'Desconhecido'), 'error');
    }
  } catch (err) {
    console.error('Falha ao submeter missão:', err);
    toast('Erro de comunicação com o souls_server.', 'error');
  }
}

function copyResearchCommand() {
  const cmdText = document.getElementById('text-copy-command');
  if (cmdText && cmdText.innerText) {
    navigator.clipboard.writeText(cmdText.innerText);
    toast('Comando copiado para a área de transferência! Cole no chat do Hermes Agent.', 'success');
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
      <div class="card u-text-center u-text-dark u-s-73fc9c">
        <span>${uiIcon('radio', 'icon-sm')} Nenhuma missão de pesquisa em andamento no momento.</span>
        <div class="u-mt-10p">
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
      <div class="card u-s-a12eea">
        <div class="u-d-flex u-justify-between u-items-start u-mb-10p">
          <div>
            <div class="u-d-flex u-gap-2 u-items-center u-mb-1">
              <span class="badge u-text-cyan u-s-866a31">${m.status.toUpperCase()}</span>
              <span class="u-text-dark u-mono u-s-a2b44b">${m.run_id}</span>
            </div>
            <h3 class="u-fw-600 u-m-0 u-s-c73cff">${escapeHtml(m.question)}</h3>
          </div>
          <div class="u-d-flex u-gap-6p">
            <button class="btn btn-xs btn-outline" onclick="openMissionSteeringModal('${m.run_id}')">${uiIcon('compass', 'icon-xs')} Steering</button>
            <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${m.run_id}')">Inspecionar</button>
            <button class="btn btn-xs btn-danger" onclick="cancelResearchMission('${m.run_id}')">Cancelar</button>
          </div>
        </div>

        <!-- Telemetry Gauges -->
        <div class="grid grid-3 u-gap-3 u-mt-14p u-s-b89f5e">
          <div>
            <div class="u-d-flex u-justify-between u-mb-1 u-s-a2b44b">
              <span class="u-text-muted">Turnos do Orçamento:</span>
              <strong class="u-s-221324">${m.turns_used || 0} / 90</strong>
            </div>
            <div class="u-overflow-hidden u-s-ac784c">
              <div class="u-s-5e11b4" style="width: ${turnsPct}%; background: ${turnsPct > 80 ? '#ef4444' : '#38bdf8'};"></div>
            </div>
          </div>
          <div>
            <div class="u-d-flex u-justify-between u-mb-1 u-s-a2b44b">
              <span class="u-text-muted">Despesa Acumulada:</span>
              <strong class="u-s-0e5a67">$${(m.spent_usd || 0).toFixed(2)} / $${(m.cap_usd || 1.50).toFixed(2)}</strong>
            </div>
            <div class="u-overflow-hidden u-s-ac784c">
              <div class="u-s-5e11b4" style="width: ${costPct}%; background: ${costPct > 85 ? '#ef4444' : '#10b981'};"></div>
            </div>
          </div>
          <div>
            <div class="u-d-flex u-justify-between u-mb-1 u-s-a2b44b">
              <span class="u-text-muted">Ronda IterResearch:</span>
              <strong class="u-text-slate">Ronda #${m.round || 1}</strong>
            </div>
            <span class="u-text-green u-s-cb8a8e">● Agente Ativo em Execução</span>
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
    listEl.innerHTML = '<div class="u-text-dark u-text-center u-s-850f5e">Nenhuma pesquisa encontrada.</div>';
    return;
  }

  listEl.innerHTML = currentResearchMissions.map(m => {
    const searchBlob = `${m.run_id || ''} ${m.status || ''} ${m.question || ''}`.replace(/"/g, '&quot;');
    return `
    <div class="library-mission-row u-pointer u-s-7c4854" data-search-text="${searchBlob}" onclick="inspectMissionDetail('${m.run_id}')" onmouseover="this.style.borderColor='#38bdf8'" onmouseout="this.style.borderColor='#18181B'">
      <div class="u-d-flex u-justify-between u-items-center u-mb-1">
        <span class="badge u-s-9c7a43">${m.status}</span>
        <span class="u-text-dark u-s-3b4cd3">${(m.created_at || '').substring(0, 10)}</span>
      </div>
      <div class="u-fw-600 u-nowrap u-overflow-hidden u-s-7452c9">
        ${escapeHtml(m.question)}
      </div>
      <div class="u-text-muted u-mt-1 u-d-flex u-justify-between u-s-cb8a8e">
        <span>$${(m.spent_usd || 0).toFixed(2)}</span>
        <span>${m.turns_used || 0} turnos</span>
      </div>
    </div>
  `;
  }).join('');
  filterLibraryMissions();
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
    <div class="u-d-flex u-justify-between u-items-start u-mb-4">
      <div>
        <div class="u-d-flex u-gap-2 u-items-center u-mb-1">
          <span class="badge u-text-cyan u-s-1e6344">${m.status.toUpperCase()}</span>
          <span class="u-mono u-text-dark u-s-a2b44b">${m.run_id}</span>
        </div>
        <h2 class="u-fw-700 u-text-white u-m-0 u-s-ff0d51">${escapeHtml(m.brief.question)}</h2>
      </div>
      <div class="u-d-flex u-gap-6p u-flex-wrap">
        <button class="btn btn-xs btn-outline" onclick="inspectEpistemicReport('${m.run_id}')">${uiIcon('tag', 'icon-xs')} Leitor Epistêmico</button>
        <button class="btn btn-xs btn-outline" onclick="openRefreshDeltaModal('${m.run_id}')">${uiIcon('refresh', 'icon-xs')} Atualizar (Delta)</button>
        <button class="btn btn-xs btn-outline" onclick="openBranchSectionModal('${m.run_id}')">${uiIcon('git-branch', 'icon-xs')} Ramificar</button>
        <button class="btn btn-xs btn-outline" onclick="openDiffModal('${m.run_id}')">${uiIcon('search', 'icon-xs')} Diff</button>
        <button class="btn btn-xs btn-primary" onclick="promoteReportModal('${m.run_id}')">${uiIcon('book', 'icon-xs')} Promover</button>
        <button class="btn btn-xs btn-outline" onclick="copyReportMarkdown()">${uiIcon('download', 'icon-xs')} Copiar .md</button>
        <button class="btn btn-xs btn-danger" onclick="archiveResearchMission('${m.run_id}')">Arquivar</button>
      </div>
    </div>

    <!-- Timeline Resumida -->
    <div class="u-mb-5 u-s-3955e5">
      <span class="u-text-muted u-fw-600 u-d-block u-mb-2 u-s-a2b44b">TIMELINE DE EVENTOS (${events.length}):</span>
      <div class="u-scroll-y u-d-flex u-flex-col u-gap-1 u-mono u-s-bc7c7e">
        ${events.length === 0 ? '<span class="u-text-dark">Nenhum evento registrado ainda.</span>' : events.map(e => `
          <div><span class="u-text-dark">[${(e.timestamp || '').substring(11, 19)}]</span> <strong class="u-text-cyan">${e.phase}</strong> (Ronda #${e.round}): <span class="u-text-slate">${escapeHtml(e.summary)}</span></div>
        `).join('')}
      </div>
    </div>

    <!-- Evolving Report Markdown -->
    <div class="u-s-789053">
      <span class="u-text-muted u-fw-600 u-d-block u-mb-10p u-s-a2b44b">RELATÓRIO CENTRAL EM EVOLUÇÃO (01_evolving_report.md):</span>
      <div class="u-text-slate-light u-pre-wrap u-scroll-y u-s-105f86" id="text-report-markdown">${escapeHtml(evolvingReport)}</div>
    </div>
  `;
}

function copyReportMarkdown() {
  const el = document.getElementById('text-report-markdown');
  if (el) {
    navigator.clipboard.writeText(el.innerText);
    toast('Relatório copiado em formato Markdown!', 'success');
  }
}

async function cancelResearchMission(run_id) {
  const ok = await appConfirm(`Deseja realmente cancelar a missão ${run_id}?`, {
    title: 'Cancelar Missão',
    icon: 'square',
    confirmText: 'Sim, Cancelar',
    danger: true
  });
  if (!ok) return;
  try {
    const res = await fetch('/api/v1/research/mission/cancel', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast('Missão cancelada com sucesso.', 'info');
      loadResearchHubMissions();
    } else {
      toast('Erro: ' + data.message, 'error');
    }
  } catch (err) {
    console.error('Falha ao cancelar missão:', err);
  }
}

async function archiveResearchMission(run_id) {
  const ok = await appConfirm(`Deseja arquivar com segurança a missão ${run_id} para .souls_data/research/_archive/?`, {
    title: 'Arquivar Missão de Pesquisa',
    icon: 'archive',
    confirmText: 'Arquivar com Segurança'
  });
  if (!ok) return;
  try {
    const res = await fetch('/api/v1/research/mission/archive', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast('Missão arquivada com sucesso.', 'success');
      loadResearchHubMissions();
      const viewer = document.getElementById('box-library-viewer');
      if (viewer) viewer.innerHTML = '<div class="u-text-center u-text-dark u-s-9f5153">Missão arquivada.</div>';
    } else {
      toast('Erro: ' + data.message, 'error');
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
  listEl.innerHTML = '<div class="u-text-dark u-s-76ce24">Executando diagnóstico de prontidão...</div>';

  try {
    const res = await fetch('/api/v1/research/doctor');
    const data = await res.json();
    if (data.status === 'success') {
      const doc = data.doctor;
      const statusBadge = document.getElementById('badge-research-doctor-status');
      if (statusBadge) {
        if (doc.overall_status === 'ready') {
          statusBadge.innerText = 'Pronto';
          statusBadge.style.color = '#34D399';
          statusBadge.style.background = 'rgba(16, 185, 129, 0.14)';
          statusBadge.style.borderColor = 'rgba(16, 185, 129, 0.38)';
        } else if (doc.overall_status === 'needs_provisioning') {
          statusBadge.innerText = 'Pendente';
          statusBadge.style.color = '#FBBF24';
          statusBadge.style.background = 'rgba(245, 158, 11, 0.16)';
          statusBadge.style.borderColor = 'rgba(245, 158, 11, 0.44)';
        } else {
          statusBadge.innerText = 'Incompleto';
          statusBadge.style.color = '#F87171';
          statusBadge.style.background = 'rgba(239, 68, 68, 0.16)';
          statusBadge.style.borderColor = 'rgba(239, 68, 68, 0.44)';
        }
      }

      listEl.innerHTML = doc.checks.map((c, idx) => {
        const hasEnv = Boolean(c.env_var);
        const catStr = String(c.category || '').trim();
        const pricingStr = String(c.pricing_tag || '').trim();
        const metaParts = [catStr, pricingStr && pricingStr.toLowerCase() !== catStr.toLowerCase() ? pricingStr : ''].filter(Boolean);
        const metaLine = metaParts.length > 0 ? `<span class="u-text-dark" style="font-size:0.7rem;">&middot; ${metaParts.map(escapeHtml).join(' &middot; ')}</span>` : '';
        const dashBtn = c.dashboard_url ? `<a href="${c.dashboard_url}" target="_blank" rel="noopener noreferrer" class="btn u-d-inline-flex u-items-center u-s-01205c" title="Abrir painel para gerar a chave">Obter Chave</a>` : '';
        const configBtn = hasEnv ? `<button class="btn u-d-inline-flex u-items-center u-s-321376" onclick="toggleDoctorKeyInput(${idx})">${uiIcon('key', 'icon-xs')} ${c.status === 'ok' ? 'Alterar' : 'Configurar'}</button>` : '';

        return `
          <div class="u-mb-1 u-s-3779eb">
            <div class="u-d-flex u-justify-between u-items-center u-flex-wrap u-gap-2">
              <div>
                <div class="u-d-flex u-gap-2 u-items-center u-flex-wrap">
                  <span class="u-fw-600 u-s-560c42">${escapeHtml(c.label)}</span>
                  ${metaLine}
                </div>
                <div class="u-text-muted u-mt-3p u-s-a2b44b">${escapeHtml(c.details)}</div>
              </div>
              <div class="u-d-flex u-items-center u-gap-2">
                ${dashBtn}
                ${configBtn}
                <div class="u-d-inline-flex u-items-center u-gap-1" style="font-size:0.78rem; font-weight:600;">
                  ${c.status === 'ok'
                    ? `<span class="u-d-inline-flex u-items-center u-gap-1 u-text-muted"><span class="status-dot status-ok"></span> Ativo</span>`
                    : c.status === 'warn'
                    ? `<span class="u-d-inline-flex u-items-center u-gap-1 u-text-amber-light"><span class="status-dot status-warn"></span> Opcional</span>`
                    : `<span class="u-d-inline-flex u-items-center u-gap-1 u-text-red"><span class="status-dot status-danger"></span> Pendente</span>`}
                </div>
              </div>
            </div>
            ${hasEnv ? `
              <div class="u-mt-10p u-gap-6p u-items-center u-s-4eb8b1 is-hidden" id="doctor-key-row-${idx}">
                <input type="password" id="doctor-key-val-${idx}" class="setting-input u-flex-1 u-s-87da7d" placeholder="Cole aqui sua chave (${c.env_var})...">
                <button class="btn btn-green u-s-7f445b" onclick="saveDoctorKey('${c.env_var}', ${idx})">Salvar Chave</button>
                <button class="btn u-s-4d5576" onclick="toggleDoctorKeyInput(${idx})">Cancelar</button>
              </div>
            ` : ''}
          </div>
        `;
      }).join('');
    }
  } catch (err) {
    console.error('Falha no diagnóstico doctor:', err);
    listEl.innerHTML = '<div class="u-text-red u-s-76ce24">Falha ao carregar relatório do Doctor.</div>';
  }
}

function toggleDoctorKeyInput(idx) {
  const row = document.getElementById(`doctor-key-row-${idx}`);
  if (row) {
    const isHidden = row.style.display === 'none' || !row.style.display;
    row.style.display = isHidden ? 'flex' : 'none';
    if (isHidden) {
      const input = document.getElementById(`doctor-key-val-${idx}`);
      if (input) input.focus();
    }
  }
}

async function saveDoctorKey(keyName, idx) {
  const input = document.getElementById(`doctor-key-val-${idx}`);
  if (!input) return;
  const val = input.value.trim();
  if (!val) {
    toast('Digite ou cole a chave de API antes de salvar.', 'warning');
    return;
  }

  try {
    const res = await fetch('/api/v1/system/credentials/save', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key: keyName, value: val })
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast(`Chave para '${keyName}' configurada e ativada com sucesso!`, 'success');
      await loadResearchDoctorReport();
    } else {
      toast(data.message || 'Falha ao salvar credencial.', 'error');
    }
  } catch (e) {
    console.error('Erro ao salvar credencial:', e);
    toast('Erro de conexão ao salvar credencial.', 'error');
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

async function openMissionSteeringModal(run_id) {
  const directive = await appPrompt(
    `[STEERING EM VOO — ${run_id}]\nDigite a diretriz a ser injetada para os subagentes nesta ronda:\n(Ex: "Ignore discussões sobre Mac/Linux; foque estritamente em Windows 11 ReFS")`,
    '',
    { title: 'Injetar Diretriz (Steering)', icon: 'compass', submitText: 'Injetar Diretriz' }
  );
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
      toast('Comando de steering enviado com sucesso!', 'success');
      loadResearchHubMissions();
      if (selectedMissionDetail && selectedMissionDetail.mission.run_id === run_id) {
        inspectMissionDetail(run_id);
      }
    } else {
      toast('Erro ao enviar steering: ' + (data.message || 'Desconhecido'), 'error');
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
      toast('Relatório ainda não finalizado: ' + (data.message || ''), 'warning');
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
    <div class="u-d-flex u-justify-between u-items-start u-mb-14p">
      <div>
        <div class="u-d-flex u-gap-2 u-items-center u-mb-1">
          <span class="badge" style="background:${rep.is_verified ? 'rgba(16,185,129,0.2)' : 'rgba(234,179,8,0.2)'}; color:${rep.is_verified ? '#10b981' : '#facc15'};">
            ${rep.is_verified ? `${uiIcon('check-circle', 'icon-xs')} VERIFICADO READ-ONLY` : `${uiIcon('zap', 'icon-xs')} RASCUNHO EM PROGRESSO`}
          </span>
          <span class="u-mono u-text-dark u-s-a2b44b">${run_id}</span>
        </div>
        <h2 class="u-fw-700 u-text-white u-m-0 u-s-ff0d51">Leitor Epistêmico de Evidências</h2>
      </div>
      <div class="u-d-flex u-gap-6p">
        <button class="btn btn-xs btn-primary" onclick="openClaimFeedbackModal('${run_id}')">${uiIcon('sparkles', 'icon-xs')} Avaliar Fatos</button>
        <button class="btn btn-xs btn-outline" onclick="copyReportMarkdown()">${uiIcon('download', 'icon-xs')} Copiar .md</button>
        <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Timeline</button>
      </div>
    </div>

    <!-- Chips Epistêmicos -->
    <div class="u-d-flex u-gap-10p u-flex-wrap u-mb-14p">
      <span class="badge u-text-green u-s-a7f54a">
        [VERIFIED]: <strong>${tc.verified}</strong>
      </span>
      <span class="badge u-text-cyan u-s-7405b3">
        [SOURCED]: <strong>${tc.sourced}</strong>
      </span>
      <span class="badge u-s-41e0bc">
        [REASONED]: <strong>${tc.reasoned}</strong>
      </span>
      <span class="badge u-text-slate u-s-5e03fe">
        [ESTIMATED]: <strong>${tc.estimated}</strong>
      </span>
      ${rep.contradictions_count > 0 ? `
        <span class="badge u-text-red u-s-d77f14">
          Contradições Mapeadas: <strong>${rep.contradictions_count}</strong>
        </span>
      ` : ''}
    </div>

    <!-- Fontes & Citações Extraídas -->
    ${citations.length > 0 ? `
      <div class="u-mb-14p u-s-0c09a3">
        <span class="u-text-muted u-fw-600 u-d-block u-mb-6p u-s-a2b44b">FONTES IDENTIFICADAS (${citations.length}):</span>
        <div class="u-d-flex u-flex-wrap u-gap-6p u-scroll-y u-s-05d62e">
          ${citations.map(c => `
            <a class="u-text-cyan u-d-inline-flex u-items-center u-gap-1 u-s-2680c3" href="${c.url}" target="_blank" rel="noopener">
              <span>[${c.citation_num}]</span> ${escapeHtml(c.domain)}
              <span class="u-text-dark u-s-3b4cd3">(${c.tier})</span>
            </a>
          `).join('')}
        </div>
      </div>
    ` : ''}

    <!-- Visualizador de Markdown com Renderização de Tags -->
    <div class="u-s-789053">
      <div class="u-text-slate-light u-pre-wrap u-scroll-y u-s-7b13c7" id="text-report-markdown">${renderHighlightedEpistemicReport(rep.raw_markdown)}</div>
    </div>
  `;
}

// ============================================================================
// 1-CLICK PROVISIONING FLOW (Fase 2)
// ============================================================================

async function triggerProvisioningFlow(dryRun) {
  const box = document.getElementById('box-provisioning-diff');
  if (box) {
    box.style.display = 'block';
    box.innerHTML = '<div class="u-text-cyan u-s-22b62c">Calculando modificações necessárias no ecossistema Hermes...</div>';
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
      if (box) box.innerHTML = `<div class="u-text-red u-s-22b62c">Erro no provisionamento: ${escapeHtml(data.message)}</div>`;
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
      <div class="u-d-flex u-justify-between u-items-center">
        <span class="u-text-green u-fw-600 u-s-933a8e">${uiIcon('check-circle', 'icon-xs')} Ecossistema 100% alinhado! Nenhuma modificação pendente.</span>
        <button class="btn btn-xs btn-outline" onclick="this.parentElement.parentElement.style.display='none'">Fechar</button>
      </div>
    `;
    return;
  }

  box.innerHTML = `
    <div class="u-d-flex u-justify-between u-items-center u-mb-10p">
      <div>
        <strong class="u-s-fa0ee0">${result.dry_run ? `${uiIcon('search', 'icon-xs')} Prévia de Provisionamento` : `${uiIcon('check-circle', 'icon-xs')} Provisionamento Aplicado`}</strong>
        <span class="u-text-muted u-d-block u-s-a2b44b">${escapeHtml(result.message)}</span>
      </div>
      <div class="u-d-flex u-gap-6p">
        ${result.dry_run ? `
          <button class="btn btn-xs btn-success" onclick="triggerProvisioningFlow(false)">${uiIcon('rocket', 'icon-xs')} Aplicar Modificações Agora</button>
        ` : ''}
        <button class="btn btn-xs btn-outline" onclick="document.getElementById('box-provisioning-diff').style.display='none'">Fechar</button>
      </div>
    </div>
    <div class="u-d-flex u-flex-col u-gap-2 u-scroll-y u-s-1af767">
      ${changes.map(c => `
        <div class="u-s-181be6">
          <div class="u-d-flex u-justify-between u-mb-1">
            <strong class="u-text-cyan">${escapeHtml(c.target_file)}</strong>
            <span class="badge u-s-3b4cd3">${c.change_type}</span>
          </div>
          <div class="u-text-muted u-s-cb8a8e">${escapeHtml(c.explanation)}</div>
        </div>
      `).join('')}
    </div>
  `;
}


// ============================================================================
// FASE 3: RELATÓRIOS VIVOS (DELTA), RAMIFICAÇÃO, DIFF & PROMOÇÃO OBSIDIAN
// ============================================================================

async function openRefreshDeltaModal(run_id) {
  const ok = await appConfirm(
    `Deseja lançar uma nova versão viva (Delta v2+) para a pesquisa ${run_id}?\nO Hermes reaproveitará o acervo de fontes e buscará atualizações e fatos recentes.`,
    { title: 'Lançar Versão Delta (v2+)', icon: 'refresh', confirmText: 'Criar Delta' }
  );
  if (!ok) return;

  try {
    const res = await fetch('/api/v1/research/mission/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ parent_run_id: run_id })
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast(`Versão Delta criada e enfileirada: ${data.mission.run_id}`, 'success');
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      toast('Erro: ' + (data.message || 'Falha ao criar delta'), 'error');
    }
  } catch (err) {
    console.error('Falha ao atualizar delta:', err);
  }
}

async function openBranchSectionModal(run_id) {
  const heading = await appPrompt(
    `[RAMIFICAR SEÇÃO — ${run_id}]\nDigite o título ou heading do tópico que deseja aprofundar de forma autônoma:\n(Ex: "Mapeamento empírico de latência no ReFS")`,
    '',
    { title: 'Ramificar Seção de Pesquisa', icon: 'git-branch', submitText: 'Avançar' }
  );
  if (!heading || !heading.trim()) return;

  const customQ = await appPrompt(
    `Pergunta específica para este subagente (ou deixe em branco para derivar automaticamente):`,
    '',
    { title: 'Pergunta da Ramificação', icon: 'info', submitText: 'Criar Missão' }
  );

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
      toast(`Missão ramificada criada e enfileirada com sucesso: ${data.mission.run_id}`, 'success');
      loadResearchHubMissions();
      switchResearchSubtab('active');
    } else {
      toast('Erro: ' + (data.message || 'Falha ao ramificar missão'), 'error');
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
        <div class="u-d-flex u-justify-between u-items-center u-mb-14p">
          <div>
            <h2 class="u-fw-700 u-text-white u-m-0 u-s-ff0d51">${uiIcon('search', 'icon-sm')} Comparador Myers Diff de Versões</h2>
            <span class="u-text-muted u-s-a2b44b">Missão: ${run_id} (Adições: +${d.additions}, Remoções: -${d.deletions})</span>
          </div>
          <button class="btn btn-xs btn-outline" onclick="inspectMissionDetail('${run_id}')">Voltar ao Relatório</button>
        </div>
        <div class="u-mono u-scroll-y u-s-b44794">
          ${d.diff_lines.length === 0 ? '<span class="u-text-dark">Nenhuma alteração entre as versões baseline e atual.</span>' : d.diff_lines.map(line => {
            if (line.startsWith('+')) {
              return `<div class="u-text-green u-s-83b80b">${escapeHtml(line)}</div>`;
            } else if (line.startsWith('-')) {
              return `<div class="u-text-red u-s-155aa5">${escapeHtml(line)}</div>`;
            } else {
              return `<div class="u-text-muted">${escapeHtml(line)}</div>`;
            }
          }).join('')}
        </div>
      `;
    } else {
      toast('Erro ao calcular diff: ' + (data.message || ''), 'error');
    }
  } catch (err) {
    console.error('Falha ao carregar diff:', err);
  }
}

async function promoteReportModal(run_id) {
  const ok = await appConfirm(
    `Deseja promover este relatório verificado para o Cofre Obsidian e Hipocampo L3?\nO Privacy Shield higienizará localmente quaisquer dados sensíveis (PII) antes de persistir.`,
    { title: 'Promover Relatório', icon: 'shield', confirmText: 'Promover com Privacy Shield' }
  );
  if (!ok) return;

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
      toast(`Relatório promovido com sucesso!\n- Obsidian: ${p.obsidian_file_path || 'raw/research/'}\n- PIIs Higienizados: ${p.pii_redactions_count}`, 'success');
      inspectMissionDetail(run_id);
    } else {
      toast('Erro na promoção: ' + (data.message || ''), 'error');
    }
  } catch (err) {
    console.error('Falha ao promover relatório:', err);
  }
}

// ============================================================================
// FASE 4: SCORECARD E³, DOMAIN TRUST LEDGER, FEEDBACK & SETTINGS
// ============================================================================

async function loadResearchScorecard() {
  try {
    const res = await fetch('/api/v1/research/scorecard');
    const data = await res.json();
    if (data.status === 'success') {
      const sc = data.scorecard;
      const elE3 = document.getElementById('metric-e3-score');
      const elAcc = document.getElementById('metric-accuracy-rate');
      const elClaims = document.getElementById('metric-claims-evaluated');
      const elCost = document.getElementById('metric-cost-per-verified');

      if (elE3) elE3.innerText = `${sc.e3_score.toFixed(1)} / 100`;
      if (elAcc) elAcc.innerText = `${(sc.epistemic_accuracy_rate * 100).toFixed(1)}%`;
      if (elClaims) elClaims.innerHTML = `${sc.total_claims_evaluated} (${sc.accurate_claims} ${uiIcon('check', 'icon-xs')} | ${sc.hallucinated_claims} ${uiIcon('x', 'icon-xs')})`;
      if (elCost) elCost.innerText = `$${sc.avg_cost_per_verified_claim.toFixed(3)}`;
    }
  } catch (err) {
    console.error('Falha ao carregar scorecard E3:', err);
  }
}

async function loadDomainTrustLedger() {
  const tbody = document.getElementById('tbody-domain-ledger');
  if (!tbody) return;

  try {
    const res = await fetch('/api/v1/research/domains');
    const data = await res.json();
    if (data.status === 'success') {
      const domains = Object.values(data.ledger.domains || {});
      domains.sort((a, b) => b.trust_score - a.trust_score);

      if (domains.length === 0) {
        tbody.innerHTML = `<tr><td class="u-text-center u-text-dark u-s-872b66" colspan="6">Nenhum domínio registrado.</td></tr>`;
        return;
      }

      tbody.innerHTML = domains.map(d => {
        const tierBadge = d.tier === 'boost'
          ? `<span class="u-d-inline-flex u-items-center u-gap-1 u-text-muted" style="font-size:0.75rem;"><span class="status-dot status-ok"></span> Boost</span>`
          : d.tier === 'block'
          ? `<span class="u-d-inline-flex u-items-center u-gap-1" style="font-size:0.75rem; color:#FCA5A5;"><span class="status-dot status-danger"></span> Block</span>`
          : `<span class="u-d-inline-flex u-items-center u-gap-1 u-text-muted" style="font-size:0.75rem;"><span class="status-dot" style="background:#71717A;"></span> Allow</span>`;

        const scoreColor = d.trust_score >= 0.8 ? '#10b981' : d.trust_score >= 0.5 ? '#facc15' : '#ef4444';

        return `
          <tr class="u-s-0a3b39">
            <td class="u-s-cb92ec">
              <strong class="u-s-221324">${escapeHtml(d.domain)}</strong>
              ${d.notes ? `<div class="u-text-dark u-s-cb8a8e">${escapeHtml(d.notes)}</div>` : ''}
            </td>
            <td class="u-s-cb92ec">${tierBadge}</td>
            <td class="u-s-cb92ec">
              <span class="u-fw-700" style="color:${scoreColor};">${(d.trust_score * 100).toFixed(0)}%</span>
            </td>
            <td class="u-text-muted u-s-cb92ec">${d.claims_count} (${d.verified_count} ${uiIcon('check', 'icon-xs')})</td>
            <td class="u-s-cb92ec" style="color:${d.hallucinated_count > 0 ? '#ef4444' : '#71717A'};">${d.hallucinated_count}</td>
            <td class="u-text-right u-s-cb92ec">
              <div class="u-d-inline-flex u-gap-1">
                ${d.tier !== 'boost' ? `<button class="btn btn-xs btn-outline u-text-green" onclick="changeDomainTier('${d.domain}', 'boost')">Boost</button>` : ''}
                ${d.tier !== 'allow' ? `<button class="btn btn-xs btn-outline u-text-slate" onclick="changeDomainTier('${d.domain}', 'allow')">Allow</button>` : ''}
                ${d.tier !== 'block' ? `<button class="btn btn-xs btn-outline u-text-red" onclick="changeDomainTier('${d.domain}', 'block')">Block</button>` : ''}
              </div>
            </td>
          </tr>
        `;
      }).join('');
    }
  } catch (err) {
    console.error('Falha ao carregar domain ledger:', err);
  }
}

async function addNewDomainTrust() {
  const input = document.getElementById('input-add-domain');
  const select = document.getElementById('select-add-tier');
  if (!input || !select) return;

  const domain = input.value.trim();
  const tier = select.value;
  if (!domain) {
    toast('Digite o domínio a ser cadastrado (ex: arxiv.org)', 'warning');
    return;
  }

  try {
    const res = await fetch('/api/v1/research/domains', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ domain, tier })
    });
    const data = await res.json();
    if (data.status === 'success') {
      input.value = '';
      loadDomainTrustLedger();
    } else {
      toast('Erro: ' + (data.message || 'Falha ao cadastrar'), 'error');
    }
  } catch (err) {
    console.error('Falha ao adicionar domínio:', err);
  }
}

async function changeDomainTier(domain, tier) {
  try {
    const res = await fetch('/api/v1/research/domains', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ domain, tier })
    });
    const data = await res.json();
    if (data.status === 'success') {
      loadDomainTrustLedger();
      loadResearchScorecard();
    } else {
      toast('Erro: ' + (data.message || 'Falha ao atualizar'), 'error');
    }
  } catch (err) {
    console.error('Falha ao alterar tier:', err);
  }
}

async function openClaimFeedbackModal(run_id) {
  const stmt = await appPrompt(`[AVALIAÇÃO DE AFIRMAÇÃO — ${run_id}]\nDigite o trecho ou afirmação que deseja avaliar:`, '', {
    title: 'Avaliar Afirmação',
    icon: 'scale',
    submitText: 'Próximo'
  });
  if (!stmt || !stmt.trim()) return;

  const dom = await appPrompt(`Domínio associado à fonte (opcional, ex: github.com ou spam.com):`, '', {
    title: 'Domínio da Fonte',
    icon: 'globe',
    submitText: 'Próximo'
  });
  const ratingStr = await appPrompt(
    `Classificação:\n1 = Correto / Conforme\n2 = Disputado / Inconclusivo\n3 = Alucinação / Falso`,
    '1',
    { title: 'Classificação Epistêmica', icon: 'bar-chart', submitText: 'Registrar' }
  );

  let rating = 'accurate';
  if (ratingStr === '2') rating = 'disputed';
  if (ratingStr === '3') rating = 'hallucinated';

  try {
    const res = await fetch('/api/v1/research/feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        run_id,
        claim_index: Date.now() % 1000,
        statement: stmt.trim(),
        domain: dom && dom.trim() ? dom.trim() : undefined,
        rating,
        feedback_by: 'human',
        recorded_at: new Date().toISOString()
      })
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast('Feedback registrado! O ledger de reputação e métricas E3 foram atualizados.', 'success');
      loadResearchScorecard();
      loadDomainTrustLedger();
    } else {
      toast('Erro: ' + (data.message || 'Falha ao registrar feedback'), 'error');
    }
  } catch (err) {
    console.error('Falha ao enviar feedback:', err);
  }
}

async function generateSkillPatchProposal() {
  const box = document.getElementById('box-skill-patch-proposal');
  const targetEl = document.getElementById('skill-patch-target');
  const contentEl = document.getElementById('skill-patch-content');
  if (!box || !targetEl || !contentEl) return;

  try {
    const res = await fetch('/api/v1/research/skill-patch', { method: 'POST' });
    const data = await res.json();
    if (data.status === 'success') {
      const p = data.proposal;
      targetEl.innerText = `Destino: ${p.target_file}\nJustificativa: ${p.rationale}`;
      contentEl.innerText = p.patch_preview;
      box.style.display = 'block';
    } else {
      toast('Erro ao gerar proposta: ' + (data.message || ''), 'error');
    }
  } catch (err) {
    console.error('Falha ao gerar auto-patch:', err);
  }
}

async function loadResearchSettings() {
  try {
    const res = await fetch('/api/v1/research/settings');
    const data = await res.json();
    if (data.status === 'success') {
      const s = data.settings;
      const bEl = document.getElementById('setting-max-budget');
      const eEl = document.getElementById('setting-default-effort');
      const dEl = document.getElementById('setting-default-depth');
      const lEl = document.getElementById('setting-lease-timeout');
      const oEl = document.getElementById('setting-auto-obsidian');

      if (bEl) bEl.value = s.max_budget_usd;
      if (eEl) eEl.value = s.default_effort;
      if (dEl) dEl.value = s.default_depth;
      if (lEl) lEl.value = s.lease_timeout_secs;
      if (oEl) oEl.checked = s.auto_promote_obsidian;
    }
  } catch (err) {
    console.error('Falha ao carregar configurações:', err);
  }
}

async function saveResearchSettings() {
  const bEl = document.getElementById('setting-max-budget');
  const eEl = document.getElementById('setting-default-effort');
  const dEl = document.getElementById('setting-default-depth');
  const lEl = document.getElementById('setting-lease-timeout');
  const oEl = document.getElementById('setting-auto-obsidian');

  const payload = {
    max_budget_usd: bEl ? parseFloat(bEl.value) || 5.0 : 5.0,
    default_effort: eEl ? eEl.value : 'L2Comparative',
    default_depth: dEl ? parseInt(dEl.value, 10) || 2 : 2,
    lease_timeout_secs: lEl ? parseInt(lEl.value, 10) || 180 : 180,
    auto_promote_obsidian: oEl ? oEl.checked : false
  };

  try {
    const res = await fetch('/api/v1/research/settings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.status === 'success') {
      toast('Configurações do Research Hub salvas com sucesso!', 'success');
    } else {
      toast('Erro ao salvar configurações: ' + (data.message || ''), 'error');
    }
  } catch (err) {
    console.error('Falha ao salvar configurações:', err);
  }
}
