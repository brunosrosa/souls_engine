// google_hub.js — Google Personal & Life Hub, Gemini Notebooks (+ Audio Player), Agent Inbox (BMAD) & GV2 Vault (v0.9.2)

(function () {
  const state = {
    accounts: [],
    selectedAccountId: '',
    inboxStatusFilter: 'pending',
    proposalsById: {},
    notebooks: [],
    selectedNotebookId: null,
    selectedNotebookSources: [],
    lastYouTubeSummary: null,
    audioBound: false
  };

  function escapeHtml(raw) {
    if (raw === null || raw === undefined) return '';
    return String(raw)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }
  window.escapeHtml = escapeHtml;

  function icon(name, cls = 'icon-sm') {
    if (typeof window.uiIcon === 'function') {
      return window.uiIcon(name, cls);
    }
    return `<svg class="ui-icon ${escapeHtml(cls)}" aria-hidden="true"><use href="#icon-${escapeHtml(name)}"></use></svg>`;
  }

  function toast(msg, type = 'info') {
    if (typeof window.showToast === 'function') {
      window.showToast(msg, type);
    }
  }

  function policyLabel(policy) {
    switch (policy) {
      case 'read_only': return 'Somente Leitura';
      case 'drafts_and_tasks_only': return 'Seguro (Rascunhos e Tarefas)';
      case 'balanced_hotl_hitl': return 'Equilibrado (Pede Confirmação)';
      case 'whitelist_write': return 'Automação Ampla';
      default: return policy || 'Equilibrado (Pede Confirmação)';
    }
  }

  function kindLabel(kind) {
    switch (kind) {
      case 'personal_gmail': return 'Pessoal (@gmail.com)';
      case 'workspace_enterprise': return 'Workspace Enterprise';
      case 'secondary_sandbox': return 'Secundaria Sandbox';
      default: return kind || 'Google';
    }
  }

  function formatUnixTime(ts) {
    if (!ts) return '--';
    try {
      return new Date(ts * 1000).toLocaleString('pt-BR');
    } catch (_) {
      return String(ts);
    }
  }

  function formatDuration(secs) {
    if (!Number.isFinite(secs) || secs < 0) return '00:00';
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  function updateInboxBadges(pendingCount) {
    const count = Number.isFinite(Number(pendingCount)) ? Number(pendingCount) : 0;
    const topEl = document.getElementById('top-inbox-count');
    const navBadge = document.getElementById('badge-inbox-count');
    const heroCount = document.getElementById('inbox-hero-pending-count');
    const pill = document.getElementById('pill-agent-inbox');

    if (topEl) topEl.textContent = String(count);
    if (navBadge) navBadge.textContent = String(count);
    if (heroCount) heroCount.textContent = String(count);
    if (pill) pill.classList.toggle('has-pending', count > 0);
  }

  async function refreshInboxBadgeCount() {
    try {
      const res = await fetch('/api/v1/inbox/proposals?status=pending');
      if (!res.ok) return;
      const data = await res.json();
      updateInboxBadges(data.pending_count ?? (data.proposals || []).length);
    } catch (_) {
      // Silencioso no boot/polling
    }
  }
  window.refreshInboxBadgeCount = refreshInboxBadgeCount;

  // =========================================================================
  // 1. GOOGLE LIFE HUB (BRIEFING, GMAIL, TASKS, CALENDAR, ROUTES, DOCS, SHEETS, SLIDES, CONTACTS, YOUTUBE)
  // =========================================================================

  async function loadGoogleAccountsSelect() {
    try {
      const res = await fetch('/api/v1/google/accounts');
      if (!res.ok) return [];
      const data = await res.json();
      state.accounts = Array.isArray(data.accounts) ? data.accounts : [];
      const select = document.getElementById('gh-account-select');
      if (select) {
        const prev = state.selectedAccountId || select.value;
        const opts = ['<option value="">Conta Primaria (Auto)</option>'].concat(
          state.accounts.map((a) => {
            const label = escapeHtml(a.label || a.email || a.account_id);
            const email = a.email ? ` (${escapeHtml(a.email)})` : '';
            const primaryTag = a.is_primary ? ' [Primaria]' : '';
            return `<option value="${escapeHtml(a.account_id)}">${label}${email}${primaryTag}</option>`;
          })
        );
        select.innerHTML = opts.join('');
        if (prev && state.accounts.some((a) => a.account_id === prev)) {
          select.value = prev;
        }
        state.selectedAccountId = select.value || '';
      }
      updateSelectedAccountPolicyBadge();
      return state.accounts;
    } catch (_) {
      return [];
    }
  }

  function updateSelectedAccountPolicyBadge() {
    const badge = document.getElementById('gh-account-policy-badge');
    if (!badge) return;
    const active = state.accounts.find((a) => a.account_id === state.selectedAccountId)
      || state.accounts.find((a) => a.is_primary)
      || state.accounts[0];
    const label = active ? policyLabel(active.policy) : 'Equilibrado (Pede Confirmação)';
    badge.innerHTML = `${icon('shield', 'icon-xs')} <span>${escapeHtml(label)}</span>`;
  }

  function getActiveAccountId() {
    const sel = document.getElementById('gh-account-select');
    return sel ? (sel.value || '') : (state.selectedAccountId || '');
  }

  function onGoogleHubAccountChange() {
    const sel = document.getElementById('gh-account-select');
    state.selectedAccountId = sel ? sel.value : '';
    updateSelectedAccountPolicyBadge();
    loadGoogleBriefing(false);
    loadGoogleTasks();
  }
  window.onGoogleHubAccountChange = onGoogleHubAccountChange;

  async function loadGoogleHubTab() {
    await loadGoogleAccountsSelect();
    await Promise.all([
      loadGoogleBriefing(false),
      loadGoogleTasks(),
      refreshInboxBadgeCount()
    ]);
  }
  window.loadGoogleHubTab = loadGoogleHubTab;

  async function loadGoogleBriefing(isManual = false) {
    const accountId = getActiveAccountId();
    const qs = accountId ? `?account_id=${encodeURIComponent(accountId)}` : '';
    try {
      const res = await fetch(`/api/v1/google/briefing${qs}`);
      if (!res.ok) {
        if (isManual) toast('Falha ao carregar briefing diario Google.', 'error');
        return;
      }
      const b = await res.json();
      updateInboxBadges(b.pending_inbox_count || 0);

      const banner = document.getElementById('gh-oauth-banner');
      if (banner) banner.classList.toggle('is-hidden', Boolean(b.oauth_connected));

      const statusBadge = document.getElementById('gh-briefing-status-badge');
      if (statusBadge) {
        if (b.oauth_connected) {
          statusBadge.className = 'badge badge-value';
          statusBadge.innerHTML = `${icon('check-circle', 'icon-xs')} <span>OAuth Conectado (${escapeHtml(b.account_email || 'Ativo')})</span>`;
        } else {
          statusBadge.className = 'badge badge-lowcost';
          statusBadge.innerHTML = `${icon('alert-triangle', 'icon-xs')} <span>Modo Offline / Sem OAuth</span>`;
        }
      }

      const notesEl = document.getElementById('gh-briefing-notes');
      if (notesEl) {
        const notes = Array.isArray(b.notes) ? b.notes : [];
        if (notes.length > 0) {
          notesEl.classList.remove('is-hidden');
          notesEl.innerHTML = notes.map((n) => `<div class="gh-note-line">${icon('info', 'icon-xs')} <span>${escapeHtml(n)}</span></div>`).join('');
        } else {
          notesEl.classList.add('is-hidden');
          notesEl.innerHTML = '';
        }
      }

      renderBriefingEvents(b.upcoming_events || []);
      renderCommuteAlerts(b.commute_alerts || []);
      renderFocusSlots(b.focus_slots || []);

      if ((b.priority_emails || []).length > 0) {
        renderGmailList(b.priority_emails, b.account_email || '');
      }
      if ((b.pending_tasks || []).length > 0) {
        renderTasksList(b.pending_tasks);
      }
      if (isManual) toast('Briefing diario Google atualizado.', 'success');
    } catch (err) {
      if (isManual) toast('Erro ao atualizar briefing: ' + err.message, 'error');
    }
  }
  window.loadGoogleBriefing = loadGoogleBriefing;

  function renderBriefingEvents(events) {
    const el = document.getElementById('gh-briefing-events');
    if (!el) return;
    if (!events.length) {
      el.innerHTML = '<div class="gh-empty-state">Nenhum evento proximo encontrado na agenda.</div>';
      return;
    }
    el.innerHTML = events.map((ev) => `
      <div class="gh-item-row">
        <div class="gh-item-title">${escapeHtml(ev.summary || 'Evento sem titulo')}</div>
        <div class="gh-item-meta u-mono">${escapeHtml(ev.start_rfc3339 || '')} &rarr; ${escapeHtml(ev.end_rfc3339 || '')}</div>
        ${ev.location ? `<div class="gh-item-sub">${icon('compass', 'icon-xs')} <span>${escapeHtml(ev.location)}</span></div>` : ''}
      </div>
    `).join('');
  }

  function renderCommuteAlerts(alerts) {
    const el = document.getElementById('gh-commute-alerts');
    if (!el) return;
    if (!alerts.length) {
      el.innerHTML = '<div class="gh-empty-state">Nenhum alerta de trânsito no momento.</div>';
      return;
    }
    el.innerHTML = alerts.map((a) => {
      const mins = Math.round((a.duration_seconds || 0) / 60);
      const km = ((a.distance_meters || 0) / 1000).toFixed(1);
      const delayMins = Math.round((a.traffic_delay_seconds || 0) / 60);
      return `
        <div class="gh-item-row gh-item-alert">
          <div class="gh-item-title">${escapeHtml(a.event_title)}</div>
          <div class="gh-item-meta">Sair ate: <strong class="u-text-amber u-mono">${escapeHtml(a.suggested_departure_rfc3339)}</strong></div>
          <div class="gh-item-sub">${escapeHtml(a.destination)} &bull; ${mins} min (${km} km, atraso ${delayMins} min)</div>
          <div class="gh-item-sub">${escapeHtml(a.advisory)}</div>
        </div>
      `;
    }).join('');
  }

  function renderFocusSlots(slots) {
    const el = document.getElementById('gh-focus-slots');
    if (!el) return;
    if (!slots.length) {
      el.innerHTML = '<div class="gh-empty-state">Nenhum bloco livre calculado ainda.</div>';
      return;
    }
    el.innerHTML = slots.map((s) => `
      <div class="gh-item-row">
        <div class="gh-item-title">${escapeHtml(s.label || 'Bloco de Foco')} (${escapeHtml(s.duration_minutes)} min)</div>
        <div class="gh-item-meta u-mono">${escapeHtml(s.start_rfc3339)} &rarr; ${escapeHtml(s.end_rfc3339)}</div>
      </div>
    `).join('');
  }

  async function triageGmail() {
    const accountId = getActiveAccountId();
    const query = document.getElementById('gh-gmail-query')?.value?.trim() || 'in:inbox';
    const listEl = document.getElementById('gh-gmail-list');
    if (listEl) listEl.innerHTML = '<div class="gh-empty-state">Escaneando mensagens no Gmail...</div>';

    try {
      const params = new URLSearchParams({ query, max_results: '15' });
      if (accountId) params.set('account_id', accountId);
      const res = await fetch(`/api/v1/google/gmail/triage?${params.toString()}`);
      const data = await res.json();
      if (!res.ok) {
        if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(data.message || 'OAuth Gmail nao disponivel')}</div>`;
        toast(data.message || 'Falha na triagem Gmail.', 'warning');
        return;
      }
      renderGmailList(data.priority_messages || [], data.account_email || '');
      toast(`Triagem concluida: ${data.total_scanned || 0} mensagens analisadas.`, 'success');
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.triageGmail = triageGmail;

  function renderGmailList(messages, accountEmail) {
    const listEl = document.getElementById('gh-gmail-list');
    if (!listEl) return;
    if (!messages.length) {
      listEl.innerHTML = '<div class="gh-empty-state">Nenhuma mensagem prioritaria encontrada.</div>';
      return;
    }
    listEl.innerHTML = messages.map((m, idx) => `
      <div class="gh-item-row">
        <div class="u-d-flex u-justify-between u-items-start u-gap-2">
          <div class="gh-item-title">${escapeHtml(m.subject || '(Sem assunto)')}</div>
          <span class="badge badge-champion">${escapeHtml(m.category || 'prioridade')}</span>
        </div>
        <div class="gh-item-meta">${escapeHtml(m.from || accountEmail)} &bull; Score ${(m.priority_score || 0).toFixed(2)}</div>
        <div class="gh-item-sub">${escapeHtml(m.snippet || '')}</div>
        <div class="gh-btn-row u-mt-2">
          <button type="button" class="btn btn-xs btn-secondary" data-idx="${idx}" onclick="prefillGmailReply('${escapeHtml(m.from || '')}', '${escapeHtml(m.subject || '')}')">
            ${icon('edit', 'icon-xs')} <span>Preparar Rascunho</span>
          </button>
        </div>
      </div>
    `).join('');
  }

  function prefillGmailReply(fromRaw, subjectRaw) {
    const toInput = document.getElementById('gh-gmail-to');
    const subInput = document.getElementById('gh-gmail-subject');
    const match = String(fromRaw || '').match(/<([^>]+)>/);
    if (toInput) toInput.value = match ? match[1] : (fromRaw || '');
    if (subInput) {
      const s = String(subjectRaw || '');
      subInput.value = s.toLowerCase().startsWith('re:') ? s : `Re: ${s}`;
    }
    toast('Formulario de rascunho preenchido.', 'info');
  }
  window.prefillGmailReply = prefillGmailReply;

  async function createGmailDraft() {
    const to = document.getElementById('gh-gmail-to')?.value?.trim() || '';
    const subject = document.getElementById('gh-gmail-subject')?.value?.trim() || '';
    const body = document.getElementById('gh-gmail-body')?.value || '';
    if (!to || !subject) {
      toast('Informe destinatario e assunto para criar rascunho.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/gmail/draft', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          account_id: getActiveAccountId() || null,
          to,
          subject,
          body
        })
      });
      const data = await res.json();
      if (data.status === 'queued_for_approval') {
        toast('Rascunho enviado para a Central de Aprovações por segurança.', 'warning');
        refreshInboxBadgeCount();
      } else if (res.ok) {
        toast('Rascunho salvo com segurança no Gmail.', 'success');
      } else {
        toast(data.message || 'Nao foi possivel criar o rascunho.', 'error');
      }
    } catch (err) {
      toast('Erro ao criar rascunho: ' + err.message, 'error');
    }
  }
  window.createGmailDraft = createGmailDraft;

  async function proposeGmailSend() {
    const to = document.getElementById('gh-gmail-to')?.value?.trim() || '';
    const subject = document.getElementById('gh-gmail-subject')?.value?.trim() || '';
    const body = document.getElementById('gh-gmail-body')?.value || '';
    if (!to || !subject) {
      toast('Informe destinatario e assunto antes de propor envio.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/gmail/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          account_id: getActiveAccountId() || null,
          to,
          subject,
          body
        })
      });
      const data = await res.json();
      if (res.status === 202 || data.status === 'queued_for_approval') {
        toast('Envio preparado na Central de Aprovações! Deslize para confirmar.', 'success');
        refreshInboxBadgeCount();
      } else {
        toast(data.message || 'Falha ao enviar para a Central de Aprovações.', 'error');
      }
    } catch (err) {
      toast('Erro ao propor envio: ' + err.message, 'error');
    }
  }
  window.proposeGmailSend = proposeGmailSend;

  async function loadGoogleTasks() {
    const listEl = document.getElementById('gh-tasks-list');
    const accountId = getActiveAccountId();
    const qs = accountId ? `?account_id=${encodeURIComponent(accountId)}` : '';
    try {
      const res = await fetch(`/api/v1/google/tasks${qs}`);
      const data = await res.json();
      if (!res.ok) {
        if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(data.message || 'Google Tasks aguardando OAuth2.')}</div>`;
        return;
      }
      renderTasksList(data.tasks || []);
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.loadGoogleTasks = loadGoogleTasks;

  function renderTasksList(tasks) {
    const listEl = document.getElementById('gh-tasks-list');
    if (!listEl) return;
    if (!tasks.length) {
      listEl.innerHTML = '<div class="gh-empty-state">Nenhuma tarefa pendente no Google Tasks.</div>';
      return;
    }
    listEl.innerHTML = tasks.map((t) => `
      <div class="gh-item-row u-d-flex u-justify-between u-items-center u-gap-2">
        <div>
          <div class="gh-item-title">${escapeHtml(t.title)}</div>
          ${t.notes ? `<div class="gh-item-sub">${escapeHtml(t.notes)}</div>` : ''}
          ${t.due_rfc3339 ? `<div class="gh-item-meta u-mono">Prazo: ${escapeHtml(t.due_rfc3339)}</div>` : ''}
        </div>
        <button type="button" class="btn btn-xs btn-secondary" onclick="completeGoogleTask('${escapeHtml(t.id)}', '${escapeHtml(t.title)}')">
          ${icon('check-circle', 'icon-xs')} <span>Concluir</span>
        </button>
      </div>
    `).join('');
  }

  async function createGoogleTask() {
    const title = document.getElementById('gh-task-title')?.value?.trim() || '';
    const notes = document.getElementById('gh-task-notes')?.value?.trim() || '';
    if (!title) {
      toast('Digite um titulo para criar a Google Task.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'create',
          account_id: getActiveAccountId() || null,
          title,
          notes: notes || null
        })
      });
      const data = await res.json();
      if (data.status === 'queued_for_approval') {
        toast('Criação de tarefa enviada para a Central de Aprovações.', 'warning');
        refreshInboxBadgeCount();
      } else if (res.ok) {
        toast('Google Task criada com sucesso.', 'success');
        loadGoogleTasks();
      } else {
        toast(data.message || 'Erro ao criar Google Task.', 'error');
      }
    } catch (err) {
      toast('Erro ao criar Google Task: ' + err.message, 'error');
    }
  }
  window.createGoogleTask = createGoogleTask;

  async function completeGoogleTask(taskId, title) {
    try {
      const res = await fetch('/api/v1/google/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'complete',
          account_id: getActiveAccountId() || null,
          task_id: taskId,
          title: title || 'Task'
        })
      });
      const data = await res.json();
      if (data.status === 'queued_for_approval') {
        toast('Conclusão de tarefa enviada para a Central de Aprovações.', 'warning');
        refreshInboxBadgeCount();
      } else if (res.ok) {
        toast('Tarefa concluida.', 'success');
        loadGoogleTasks();
      } else {
        toast(data.message || 'Erro ao concluir tarefa.', 'error');
      }
    } catch (err) {
      toast('Erro: ' + err.message, 'error');
    }
  }
  window.completeGoogleTask = completeGoogleTask;

  async function quickCaptureToObsidian() {
    const title = document.getElementById('gh-task-title')?.value?.trim() || '';
    const body = document.getElementById('gh-task-notes')?.value?.trim() || title;
    const tagsRaw = document.getElementById('gh-task-tags')?.value?.trim() || '';
    if (!title) {
      toast('Informe um titulo para capturar no Obsidian raw/inbox/.', 'warning');
      return;
    }
    const tags = tagsRaw
      ? tagsRaw.split(',').map((s) => s.trim()).filter(Boolean)
      : ['quick-capture'];
    try {
      const res = await fetch('/api/v1/google/quick-capture', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, body, tags })
      });
      const data = await res.json();
      if (res.ok) {
        toast(`Nota higienizada salva em ${data.note_path || 'raw/inbox/'}`, 'success');
      } else {
        toast(data.message || 'Erro no Quick Capture.', 'error');
      }
    } catch (err) {
      toast('Erro no Quick Capture: ' + err.message, 'error');
    }
  }
  window.quickCaptureToObsidian = quickCaptureToObsidian;

  async function createCalendarEvent() {
    const summary = document.getElementById('gh-cal-summary')?.value?.trim() || '';
    const start = document.getElementById('gh-cal-start')?.value?.trim() || '';
    const end = document.getElementById('gh-cal-end')?.value?.trim() || '';
    const location = document.getElementById('gh-cal-location')?.value?.trim() || '';
    if (!summary || !start || !end) {
      toast('Informe titulo, inicio e fim em formato RFC3339.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/calendar/events', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          account_id: getActiveAccountId() || null,
          summary,
          start_rfc3339: start,
          end_rfc3339: end,
          location: location || null,
          attendees: [],
          send_updates: 'none'
        })
      });
      const data = await res.json();
      if (data.status === 'queued_for_approval') {
        toast('Evento enviado para a Central de Aprovações por segurança.', 'warning');
        refreshInboxBadgeCount();
      } else if (res.ok) {
        toast('Evento agendado silenciosamente (sem enviar convites).', 'success');
        loadGoogleBriefing(false);
      } else {
        toast(data.message || 'Erro ao criar evento.', 'error');
      }
    } catch (err) {
      toast('Erro ao criar evento: ' + err.message, 'error');
    }
  }
  window.createCalendarEvent = createCalendarEvent;

  async function calculateCommuteRoute() {
    const origin = document.getElementById('gh-route-origin')?.value?.trim() || '';
    const destination = document.getElementById('gh-route-destination')?.value?.trim() || '';
    const eventStart = document.getElementById('gh-route-event-start')?.value?.trim() || '';
    const buffer = parseInt(document.getElementById('gh-route-buffer')?.value || '15', 10);
    const apiKey = document.getElementById('gh-route-apikey')?.value?.trim() || '';
    const box = document.getElementById('gh-route-result');

    if (!origin || !destination) {
      toast('Informe origem e destino para calcular o tempo de viagem.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/maps/commute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          origin,
          destination,
          event_start_rfc3339: eventStart || null,
          buffer_mins: Number.isFinite(buffer) ? buffer : 15,
          api_key: apiKey || null
        })
      });
      const data = await res.json();
      if (box) {
        box.classList.remove('is-hidden');
        if (res.ok) {
          const mins = Math.round((data.duration_seconds || 0) / 60);
          const km = ((data.distance_meters || 0) / 1000).toFixed(1);
          box.innerHTML = `
            <div class="gh-item-title">Sair ate: <span class="u-text-amber u-mono">${escapeHtml(data.suggested_departure_rfc3339)}</span></div>
            <div class="gh-item-sub">Duracao: ${mins} min &bull; Distancia: ${km} km &bull; ${escapeHtml(data.advisory)}</div>
          `;
        } else {
          box.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Erro ao calcular rota.')}</div>`;
        }
      }
    } catch (err) {
      if (box) {
        box.classList.remove('is-hidden');
        box.innerHTML = `<div class="u-text-red">${escapeHtml(err.message)}</div>`;
      }
    }
  }
  window.calculateCommuteRoute = calculateCommuteRoute;

  async function loadDocsScratchpad() {
    const docId = document.getElementById('gh-doc-id')?.value?.trim() || '';
    const title = document.getElementById('gh-doc-title')?.value?.trim() || '';
    if (!docId) {
      toast('Informe um doc_id para carregar o scratchpad.', 'warning');
      return;
    }
    try {
      const params = new URLSearchParams({ doc_id: docId });
      if (title) params.set('title', title);
      const res = await fetch(`/api/v1/google/docs/scratchpad?${params.toString()}`);
      const data = await res.json();
      if (!res.ok) {
        toast(data.message || 'Falha ao carregar scratchpad.', 'error');
        return;
      }
      const editor = document.getElementById('gh-doc-editor');
      const titleInput = document.getElementById('gh-doc-title');
      const statusBox = document.getElementById('gh-doc-status');
      if (editor) editor.value = data.rendered_text || '';
      if (titleInput && data.scratchpad?.title) titleInput.value = data.scratchpad.title;
      if (statusBox) {
        statusBox.classList.remove('is-hidden');
        statusBox.innerHTML = `<div>Scratchpad <code>${escapeHtml(docId)}</code> carregado (revisao ${escapeHtml(data.scratchpad?.revision ?? 1)}).</div>`;
      }
      toast('Scratchpad carregado.', 'success');
    } catch (err) {
      toast('Erro ao carregar scratchpad: ' + err.message, 'error');
    }
  }
  window.loadDocsScratchpad = loadDocsScratchpad;

  async function saveDocsScratchpad() {
    const docId = document.getElementById('gh-doc-id')?.value?.trim() || '';
    const title = document.getElementById('gh-doc-title')?.value?.trim() || docId;
    const op = document.getElementById('gh-doc-op')?.value || 'replace';
    const startLine = parseInt(document.getElementById('gh-doc-line-start')?.value || '1', 10);
    const endLine = parseInt(document.getElementById('gh-doc-line-end')?.value || String(startLine), 10);
    const rawText = document.getElementById('gh-doc-editor')?.value || '';
    if (!docId) {
      toast('Informe o doc_id do scratchpad.', 'warning');
      return;
    }
    const lines = rawText.replace(/\r\n/g, '\n').split('\n');
    try {
      const res = await fetch('/api/v1/google/docs/scratchpad', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          doc_id: docId,
          title,
          op,
          at_line: startLine,
          start_line: startLine,
          end_line: endLine,
          lines
        })
      });
      const data = await res.json();
      const statusBox = document.getElementById('gh-doc-status');
      if (res.ok) {
        if (document.getElementById('gh-doc-editor')) {
          document.getElementById('gh-doc-editor').value = data.rendered_text || '';
        }
        if (statusBox) {
          statusBox.classList.remove('is-hidden');
          statusBox.innerHTML = `<div>Rascunho local salvo (revisao ${escapeHtml(data.scratchpad?.revision ?? 1)}).</div>`;
        }
        toast('Rascunho local atualizado.', 'success');
      } else {
        toast(data.message || 'Erro ao salvar rascunho.', 'error');
      }
    } catch (err) {
      toast('Erro ao salvar scratchpad: ' + err.message, 'error');
    }
  }
  window.saveDocsScratchpad = saveDocsScratchpad;

  async function pushDocsScratchpad() {
    const docId = document.getElementById('gh-doc-id')?.value?.trim() || '';
    const baseline = document.getElementById('gh-doc-baseline')?.value || '';
    if (!docId) {
      toast('Informe o ID do documento para enviar a alteração.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/docs/scratchpad/push', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          doc_id: docId,
          account_id: getActiveAccountId() || null,
          remote_baseline: baseline || null
        })
      });
      const data = await res.json();
      const statusBox = document.getElementById('gh-doc-status');
      if (statusBox) statusBox.classList.remove('is-hidden');

      if (data.status === 'queued_for_approval') {
        if (statusBox) {
          statusBox.innerHTML = `
            <div class="u-text-amber u-fw-600">Alteração enviada para a Central de Aprovações (${escapeHtml(data.proposal?.id || '')})</div>
            ${renderMyersDiffHtml(data.myers_diff || '')}
          `;
        }
        toast('Alteração enviada para revisão na Central de Aprovações!', 'success');
        refreshInboxBadgeCount();
      } else if (res.ok) {
        if (statusBox) {
          statusBox.innerHTML = `
            <div class="u-text-green u-fw-600">Documento atualizado diretamente</div>
            ${renderMyersDiffHtml(data.myers_diff || '')}
          `;
        }
        toast('Push direto executado no Google Docs.', 'success');
      } else {
        if (statusBox) statusBox.innerHTML = `<div class="u-text-red">${escapeHtml(data.message || 'Falha ao propor push.')}</div>`;
        toast(data.message || 'Erro no push Docs.', 'error');
      }
    } catch (err) {
      toast('Erro ao propor push: ' + err.message, 'error');
    }
  }
  window.pushDocsScratchpad = pushDocsScratchpad;

  async function analyzeSheetsRange() {
    const spreadsheetId = document.getElementById('gh-sheet-id')?.value?.trim() || 'local_sheet';
    const range = document.getElementById('gh-sheet-range')?.value?.trim() || 'A1:D20';
    const rowsRaw = document.getElementById('gh-sheet-rows')?.value?.trim() || '';
    const box = document.getElementById('gh-sheet-result');

    let parsedRows = null;
    if (rowsRaw) {
      try {
        parsedRows = JSON.parse(rowsRaw);
      } catch (e) {
        toast('JSON inline de linhas invalido.', 'error');
        return;
      }
    }

    try {
      const res = await fetch('/api/v1/google/sheets/analyze', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          spreadsheet_id: spreadsheetId,
          range,
          account_id: getActiveAccountId() || null,
          rows: parsedRows
        })
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');
      if (!res.ok) {
        if (box) box.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Erro ao analisar planilha.')}</div>`;
        return;
      }
      const totals = (data.totals_by_column || []).map(([k, v]) => `<li>${escapeHtml(k)}: <strong>${Number(v).toFixed(2)}</strong></li>`).join('');
      const anomalies = (data.anomalies || []).map((a) => `<li class="u-text-amber">${escapeHtml(a)}</li>`).join('');
      if (box) {
        box.innerHTML = `
          <div class="gh-item-title">${escapeHtml(data.spreadsheet_id)} (${escapeHtml(data.range)}) &bull; ${escapeHtml(data.row_count)} linhas</div>
          ${totals ? `<ul class="gh-compact-list">${totals}</ul>` : '<div class="gh-item-sub">Sem colunas numericas detectadas.</div>'}
          ${anomalies ? `<div class="gh-field-label u-mt-1">Anomalias:</div><ul class="gh-compact-list">${anomalies}</ul>` : ''}
        `;
      }
    } catch (err) {
      toast('Erro ao analisar Sheets: ' + err.message, 'error');
    }
  }
  window.analyzeSheetsRange = analyzeSheetsRange;

  async function generateSlidesPlan() {
    const topic = document.getElementById('gh-slides-topic')?.value?.trim() || '';
    const briefing = document.getElementById('gh-slides-briefing')?.value?.trim() || '';
    const targetSlides = parseInt(document.getElementById('gh-slides-count')?.value || '8', 10);
    const createRemote = Boolean(document.getElementById('gh-slides-create-remote')?.checked);
    const box = document.getElementById('gh-slides-result');

    if (!topic) {
      toast('Informe o tema da apresentacao para planejar slides.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/slides/plan', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          topic,
          briefing,
          target_slides: Number.isFinite(targetSlides) ? targetSlides : 8,
          create_remote: createRemote,
          account_id: getActiveAccountId() || null
        })
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');

      if (data.status === 'queued_for_approval') {
        if (box) box.innerHTML = `<div class="u-text-amber">Criação da apresentação enviada para a Central de Aprovações.</div>`;
        refreshInboxBadgeCount();
        return;
      }
      if (!res.ok) {
        if (box) box.innerHTML = `<div class="u-text-red">${escapeHtml(data.message || 'Falha ao gerar plano.')}</div>`;
        return;
      }
      const plan = data.plan || {};
      const slidesHtml = (plan.slides || []).map((s) => `
        <div class="gh-slide-item">
          <span class="u-mono u-text-cyan">#${escapeHtml(s.index)}</span>
          <strong>${escapeHtml(s.title)}</strong>
          <div class="gh-item-sub">${escapeHtml((s.bullets || []).join(' | '))}</div>
        </div>
      `).join('');
      if (box) {
        box.innerHTML = `<div class="gh-item-title u-mb-2">${escapeHtml(plan.title || topic)} (${(plan.slides || []).length} slides)</div>${slidesHtml}`;
      }
    } catch (err) {
      toast('Erro ao gerar plano de slides: ' + err.message, 'error');
    }
  }
  window.generateSlidesPlan = generateSlidesPlan;

  async function searchGoogleContacts() {
    const query = document.getElementById('gh-contacts-query')?.value?.trim() || '';
    const birthdays = Boolean(document.getElementById('gh-contacts-birthdays')?.checked);
    const listEl = document.getElementById('gh-contacts-list');
    try {
      const params = new URLSearchParams({ query, page_size: '20' });
      const acct = getActiveAccountId();
      if (acct) params.set('account_id', acct);
      if (birthdays) params.set('birthdays_within_days', '30');
      const res = await fetch(`/api/v1/google/contacts?${params.toString()}`);
      const data = await res.json();
      if (!res.ok) {
        if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(data.message || 'Contatos indisponiveis sem OAuth.')}</div>`;
        return;
      }
      const contacts = data.contacts || [];
      if (!contacts.length) {
        if (listEl) listEl.innerHTML = '<div class="gh-empty-state">Nenhum contato encontrado para o filtro.</div>';
        return;
      }
      if (listEl) {
        listEl.innerHTML = contacts.map((c) => `
          <div class="gh-item-row">
            <div class="gh-item-title">${escapeHtml(c.display_name || c.resource_name)}</div>
            <div class="gh-item-sub">${escapeHtml((c.emails || []).join(', ') || 'Sem email')} ${c.birthday ? ` &bull; Aniversario: ${escapeHtml(c.birthday)}` : ''}</div>
          </div>
        `).join('');
      }
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.searchGoogleContacts = searchGoogleContacts;

  async function fetchYouTubeTranscript() {
    const urlOrId = document.getElementById('gh-yt-url')?.value?.trim() || '';
    const lang = document.getElementById('gh-yt-lang')?.value || 'en';
    const box = document.getElementById('gh-yt-preview');
    const captureBtn = document.getElementById('gh-yt-capture-btn');
    if (!urlOrId) {
      toast('Informe a URL ou ID de um video YouTube.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/youtube/transcript', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url_or_id: urlOrId, preferred_lang: lang })
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');
      if (!res.ok) {
        if (box) box.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Falha ao extrair transcricao.')}</div>`;
        if (captureBtn) captureBtn.classList.add('is-hidden');
        return;
      }
      state.lastYouTubeSummary = data;
      const previewText = String(data.transcript_text || '').slice(0, 1400);
      if (box) {
        box.innerHTML = `
          <div class="gh-item-title">${escapeHtml(data.title || data.video_id)} (${escapeHtml(data.language)})</div>
          <div class="gh-item-meta">${escapeHtml(data.segment_count || 0)} segmentos &bull; Conteúdo isolado com segurança</div>
          <pre class="gh-pre-preview">${escapeHtml(previewText)}</pre>
        `;
      }
      if (captureBtn) captureBtn.classList.remove('is-hidden');
      toast('Transcricao extraida com enquadramento seguro.', 'success');
    } catch (err) {
      toast('Erro ao extrair transcricao: ' + err.message, 'error');
    }
  }
  window.fetchYouTubeTranscript = fetchYouTubeTranscript;

  async function sendYouTubeToQuickCapture() {
    const yt = state.lastYouTubeSummary;
    if (!yt) return;
    const title = yt.title ? `YouTube: ${yt.title}` : `YouTube: ${yt.video_id}`;
    const body = `URL: ${yt.url || yt.video_id}\n\n${yt.transcript_text || ''}`;
    try {
      const res = await fetch('/api/v1/google/quick-capture', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, body, tags: ['youtube', 'transcript'] })
      });
      const data = await res.json();
      if (res.ok) {
        toast(`Transcricao capturada em ${data.note_path || 'Obsidian raw/inbox/'}`, 'success');
      } else {
        toast(data.message || 'Falha ao enviar transcricao.', 'error');
      }
    } catch (err) {
      toast('Erro ao capturar transcricao: ' + err.message, 'error');
    }
  }
  window.sendYouTubeToQuickCapture = sendYouTubeToQuickCapture;

  // =========================================================================
  // 2. GEMINI NOTEBOOKS (NOTEBOOKLM) + PLAYER DE AUDIO PRECISION DARK
  // =========================================================================

  function bindNotebookAudioPlayer() {
    if (state.audioBound) return;
    const audio = document.getElementById('nb-audio-element');
    const seek = document.getElementById('nb-audio-seek');
    const timeEl = document.getElementById('nb-audio-time');
    const toggleIcon = document.getElementById('nb-audio-toggle-icon');
    if (!audio) return;

    state.audioBound = true;
    const updateIcon = (playing) => {
      if (toggleIcon) {
        toggleIcon.innerHTML = `<use href="#icon-${playing ? 'pause' : 'play'}"></use>`;
      }
    };

    audio.addEventListener('play', () => updateIcon(true));
    audio.addEventListener('pause', () => updateIcon(false));
    audio.addEventListener('ended', () => updateIcon(false));
    audio.addEventListener('loadedmetadata', () => {
      if (timeEl) timeEl.textContent = `${formatDuration(audio.currentTime)} / ${formatDuration(audio.duration)}`;
    });
    audio.addEventListener('timeupdate', () => {
      if (seek && Number.isFinite(audio.duration) && audio.duration > 0) {
        seek.value = ((audio.currentTime / audio.duration) * 100).toFixed(1);
      }
      if (timeEl) timeEl.textContent = `${formatDuration(audio.currentTime)} / ${formatDuration(audio.duration)}`;
    });
    if (seek) {
      seek.addEventListener('input', () => {
        if (Number.isFinite(audio.duration) && audio.duration > 0) {
          audio.currentTime = (parseFloat(seek.value) / 100) * audio.duration;
        }
      });
    }
  }

  function loadNotebookAudioStream(notebookId, artifactId, titleText) {
    bindNotebookAudioPlayer();
    const audio = document.getElementById('nb-audio-element');
    const titleEl = document.getElementById('nb-audio-title');
    const subEl = document.getElementById('nb-audio-subtitle');
    const dlLink = document.getElementById('nb-audio-download-link');
    const manualNb = document.getElementById('nb-audio-manual-nb');
    const manualArt = document.getElementById('nb-audio-manual-art');

    if (!notebookId || !artifactId) {
      toast('Informe notebook_id e artifact_id para carregar audio.', 'warning');
      return;
    }

    const url = `/api/v1/google/notebooks/${encodeURIComponent(notebookId)}/audio/${encodeURIComponent(artifactId)}`;
    if (audio) {
      audio.src = url;
      audio.load();
    }
    if (titleEl) titleEl.textContent = titleText || `${notebookId} — ${artifactId}`;
    if (subEl) subEl.textContent = `Reproduzindo áudio de ${titleText || notebookId}`;
    if (dlLink) {
      dlLink.href = `${url}?download=true`;
      dlLink.classList.remove('is-hidden');
    }
    if (manualNb) manualNb.value = notebookId;
    if (manualArt) manualArt.value = artifactId;
  }
  window.loadNotebookAudioStream = loadNotebookAudioStream;

  function loadManualNotebookAudio() {
    const nb = document.getElementById('nb-audio-manual-nb')?.value?.trim() || '';
    const art = document.getElementById('nb-audio-manual-art')?.value?.trim() || '';
    loadNotebookAudioStream(nb, art, `Stream: ${nb} / ${art}`);
  }
  window.loadManualNotebookAudio = loadManualNotebookAudio;

  function toggleNotebookAudio() {
    bindNotebookAudioPlayer();
    const audio = document.getElementById('nb-audio-element');
    if (!audio || !audio.src) {
      toast('Selecione um Audio Overview ou carregue um stream antes de reproduzir.', 'info');
      return;
    }
    if (audio.paused) {
      audio.play().catch((err) => toast('Nao foi possivel iniciar reproducao: ' + err.message, 'warning'));
    } else {
      audio.pause();
    }
  }
  window.toggleNotebookAudio = toggleNotebookAudio;

  function setNotebookAudioSpeed(rate) {
    const audio = document.getElementById('nb-audio-element');
    const speed = parseFloat(rate || '1');
    if (audio && Number.isFinite(speed)) {
      audio.playbackRate = speed;
    }
  }
  window.setNotebookAudioSpeed = setNotebookAudioSpeed;

  async function loadGeminiNotebooksTab() {
    bindNotebookAudioPlayer();
    await loadNotebooksGallery(false);
  }
  window.loadGeminiNotebooksTab = loadGeminiNotebooksTab;

  async function loadNotebooksGallery(isManual = false) {
    const listEl = document.getElementById('nb-gallery-list');
    const countEl = document.getElementById('nb-gallery-count');
    const backendBadge = document.getElementById('nb-backend-badge');
    const topBadge = document.getElementById('badge-notebooks-count');

    try {
      const res = await fetch('/api/v1/google/notebooks');
      const data = await res.json();
      const backend = data.backend || 'none';
      if (backendBadge) {
        const bkLabel = backend === 'nlm' ? 'NotebookLM Conectado' : (backend === 'wiz_rpc' ? 'Modo Web' : backend);
        backendBadge.innerHTML = `${icon('radio', 'icon-xs')} <span>${escapeHtml(bkLabel)}</span>`;
      }

      const raw = data.notebooks;
      let notebooks = [];
      if (Array.isArray(raw)) {
        notebooks = raw.map((item, idx) => {
          if (item && typeof item === 'object' && !Array.isArray(item)) {
            return {
              notebook_id: item.notebook_id || item.id || `nb_${idx + 1}`,
              title: item.title || item.name || `Caderno ${idx + 1}`,
              source_count: item.source_count ?? (Array.isArray(item.sources) ? item.sources.length : 0),
              has_stale_sources: Boolean(item.has_stale_sources),
              source_backend: item.source_backend || backend
            };
          }
          return {
            notebook_id: Array.isArray(item) && item[2] ? String(item[2]) : `nb_${idx + 1}`,
            title: Array.isArray(item) && item[0] ? String(item[0]) : `Caderno ${idx + 1}`,
            source_count: 0,
            has_stale_sources: false,
            source_backend: backend
          };
        });
      }

      state.notebooks = notebooks;
      if (countEl) countEl.textContent = `${notebooks.length} caderno(s) encontrado(s)`;
      if (topBadge) topBadge.textContent = String(notebooks.length);

      if (!listEl) return;
      if (!notebooks.length) {
        listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(data.message || 'Nenhum caderno encontrado. Conecte sua conta do NotebookLM na aba Conexões (leva 1 clique).')}</div>`;
        return;
      }

      listEl.innerHTML = notebooks.map((nb) => {
        const isSelected = nb.notebook_id === state.selectedNotebookId;
        return `
          <div class="nb-gallery-card ${isSelected ? 'is-selected' : ''}" onclick="selectNotebook('${escapeHtml(nb.notebook_id)}', '${escapeHtml(nb.title)}')">
            <div class="u-d-flex u-justify-between u-items-start u-gap-2">
              <div class="gh-item-title">${escapeHtml(nb.title)}</div>
              ${nb.has_stale_sources ? '<span class="badge badge-lowcost">Drive Desatualizada</span>' : ''}
            </div>
            <div class="gh-item-meta u-mono">${escapeHtml(nb.notebook_id)} &bull; ${escapeHtml(nb.source_count)} fonte(s)</div>
            <div class="gh-btn-row u-mt-2">
              <span class="badge badge-dominated">${escapeHtml(nb.source_backend === 'nlm' ? 'NotebookLM' : nb.source_backend)}</span>
              <button type="button" class="btn btn-xs btn-secondary" onclick="event.stopPropagation(); syncNotebookDrive('${escapeHtml(nb.notebook_id)}')">
                ${icon('refresh', 'icon-xs')} <span>Sincronizar Drive</span>
              </button>
            </div>
          </div>
        `;
      }).join('');

      if (!state.selectedNotebookId && notebooks.length > 0) {
        selectNotebook(notebooks[0].notebook_id, notebooks[0].title);
      }
      if (isManual) toast('Galeria de cadernos atualizada.', 'success');
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.loadNotebooksGallery = loadNotebooksGallery;

  async function selectNotebook(notebookId, titleHint) {
    state.selectedNotebookId = notebookId;
    const titleEl = document.getElementById('nb-selected-title');
    const idEl = document.getElementById('nb-selected-id');
    const sourcesEl = document.getElementById('nb-sources-list');
    const audioListEl = document.getElementById('nb-cached-audio-list');

    if (titleEl) titleEl.textContent = titleHint || notebookId;
    if (idEl) idEl.textContent = notebookId;

    document.querySelectorAll('.nb-gallery-card').forEach((card) => {
      card.classList.toggle('is-selected', card.textContent.includes(notebookId));
    });

    try {
      const res = await fetch(`/api/v1/google/notebooks/${encodeURIComponent(notebookId)}`);
      const data = await res.json();
      const nb = data.notebook;

      let sources = [];
      if (nb && typeof nb === 'object' && Array.isArray(nb.sources)) {
        sources = nb.sources;
      }
      state.selectedNotebookSources = sources;

      if (sourcesEl) {
        if (sources.length > 0) {
          sourcesEl.innerHTML = sources.map((s) => `
            <div class="gh-item-row">
              <div class="u-d-flex u-justify-between u-items-center u-gap-2">
                <div class="gh-item-title">${escapeHtml(s.title || s.source_id)}</div>
                ${s.is_stale_drive ? '<span class="badge badge-lowcost">Drive Desatualizada</span>' : `<span class="badge badge-value">${escapeHtml(s.status || 'ok')}</span>`}
              </div>
              <div class="gh-item-meta u-mono">${escapeHtml(s.source_type || 'source')} &bull; ${escapeHtml(s.source_id || '')}</div>
            </div>
          `).join('');
        } else {
          sourcesEl.innerHTML = '<div class="gh-empty-state">Nenhuma fonte detalhada retornada para este caderno ainda.</div>';
        }
      }

      const cached = Array.isArray(data.cached_audio) ? data.cached_audio : [];
      if (audioListEl) {
        if (cached.length > 0) {
          audioListEl.innerHTML = cached.map((a) => `
            <div class="gh-item-row u-d-flex u-justify-between u-items-center u-gap-2">
              <div>
                <div class="gh-item-title">${escapeHtml(a.filename || a.artifact_id)}</div>
                <div class="gh-item-meta u-mono">${escapeHtml(a.artifact_id)} &bull; ${Math.round((a.size_bytes || 0) / 1024)} KB</div>
              </div>
              <button type="button" class="btn btn-xs btn-primary" onclick="loadNotebookAudioStream('${escapeHtml(notebookId)}', '${escapeHtml(a.artifact_id)}', '${escapeHtml(titleHint || notebookId)}')">
                ${icon('play', 'icon-xs')} <span>Carregar no Player</span>
              </button>
            </div>
          `).join('');
        } else {
          audioListEl.innerHTML = '<div class="gh-empty-state">Nenhum audio em cache local para este caderno.</div>';
        }
      }
    } catch (err) {
      if (sourcesEl) sourcesEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.selectNotebook = selectNotebook;

  async function syncSelectedNotebookDrive() {
    if (!state.selectedNotebookId) {
      toast('Selecione um caderno primeiro.', 'warning');
      return;
    }
    await syncNotebookDrive(state.selectedNotebookId);
  }
  window.syncSelectedNotebookDrive = syncSelectedNotebookDrive;

  async function syncNotebookDrive(notebookId) {
    const sourceIds = state.selectedNotebookSources.map((s) => s.source_id).filter(Boolean);
    try {
      const res = await fetch(`/api/v1/google/notebooks/${encodeURIComponent(notebookId)}/sync-drive`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source_ids: sourceIds })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Fontes do Google Drive sincronizadas com sucesso.', 'success');
        selectNotebook(notebookId, document.getElementById('nb-selected-title')?.textContent);
      } else {
        toast(data.message || 'Falha ao sincronizar Drive.', 'warning');
      }
    } catch (err) {
      toast('Erro na sincronizacao Drive: ' + err.message, 'error');
    }
  }
  window.syncNotebookDrive = syncNotebookDrive;

  async function addNotebookSource() {
    const nbId = state.selectedNotebookId;
    if (!nbId) {
      toast('Selecione um caderno antes de ingerir fontes.', 'warning');
      return;
    }
    const sourceType = document.getElementById('nb-source-type')?.value || 'url';
    const title = document.getElementById('nb-source-title')?.value?.trim() || 'Nova Fonte';
    const url = document.getElementById('nb-source-url')?.value?.trim() || null;
    const content = document.getElementById('nb-source-content')?.value?.trim() || null;
    try {
      const res = await fetch(`/api/v1/google/notebooks/${encodeURIComponent(nbId)}/sources`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source_type: sourceType, title, url, content })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Fonte adicionada ao caderno com sucesso.', 'success');
        selectNotebook(nbId, document.getElementById('nb-selected-title')?.textContent);
      } else {
        toast(data.message || 'Falha ao ingerir fonte.', 'warning');
      }
    } catch (err) {
      toast('Erro ao ingerir fonte: ' + err.message, 'error');
    }
  }
  window.addNotebookSource = addNotebookSource;

  async function querySelectedNotebook() {
    const nbId = state.selectedNotebookId;
    const query = document.getElementById('nb-rag-query')?.value?.trim() || '';
    const answerEl = document.getElementById('nb-rag-answer');
    if (!nbId) {
      toast('Selecione um caderno primeiro.', 'warning');
      return;
    }
    if (!query) {
      toast('Digite sua pergunta para consulta RAG.', 'warning');
      return;
    }
    if (answerEl) answerEl.innerHTML = '<div class="gh-empty-state">Consultando NotebookLM RAG...</div>';
    try {
      const res = await fetch(`/api/v1/google/notebooks/${encodeURIComponent(nbId)}/query`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query })
      });
      const data = await res.json();
      if (!res.ok) {
        if (answerEl) answerEl.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Falha na consulta RAG.')}</div>`;
        return;
      }
      const text = typeof data.answer === 'string'
        ? data.answer
        : JSON.stringify(data.answer ?? data.data ?? data, null, 2);
      if (answerEl) answerEl.innerHTML = `<pre class="gh-pre-preview">${escapeHtml(text)}</pre>`;
    } catch (err) {
      if (answerEl) answerEl.innerHTML = `<div class="u-text-red">${escapeHtml(err.message)}</div>`;
    }
  }
  window.querySelectedNotebook = querySelectedNotebook;

  async function triggerNotebookStudio(kind) {
    const nbId = state.selectedNotebookId;
    const outEl = document.getElementById('nb-studio-output');
    if (!nbId) {
      toast('Selecione um caderno para acionar o Studio.', 'warning');
      return;
    }
    if (outEl) {
      outEl.classList.remove('is-hidden');
      outEl.innerHTML = `<div class="gh-empty-state">Gerando artefato ${escapeHtml(kind)} no NotebookLM Studio...</div>`;
    }
    try {
      const res = await fetch(`/api/v1/google/notebooks/${encodeURIComponent(nbId)}/studio`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ kind })
      });
      const data = await res.json();
      if (outEl) {
        outEl.innerHTML = res.ok
          ? `<pre class="gh-pre-preview">${escapeHtml(JSON.stringify(data, null, 2))}</pre>`
          : `<div class="u-text-amber">${escapeHtml(data.message || 'Falha no Studio.')}</div>`;
      }
      if (res.ok) {
        toast(`Studio '${kind}' acionado com sucesso.`, 'success');
      }
    } catch (err) {
      if (outEl) outEl.innerHTML = `<div class="u-text-red">${escapeHtml(err.message)}</div>`;
    }
  }
  window.triggerNotebookStudio = triggerNotebookStudio;

  // =========================================================================
  // 3. AGENT INBOX (BMAD + MYERS DIFF + GRAVITY LOCK)
  // =========================================================================

  function renderMyersDiffHtml(diffText) {
    if (!diffText) return '';
    const lines = String(diffText).replace(/\r\n/g, '\n').split('\n');
    const htmlLines = lines.map((line) => {
      if (line.startsWith('+') && !line.startsWith('+++')) {
        return `<div class="myers-diff-line myers-diff-add">${escapeHtml(line)}</div>`;
      }
      if (line.startsWith('-') && !line.startsWith('---')) {
        return `<div class="myers-diff-line myers-diff-del">${escapeHtml(line)}</div>`;
      }
      if (line.startsWith('@@')) {
        return `<div class="myers-diff-line myers-diff-hunk">${escapeHtml(line)}</div>`;
      }
      return `<div class="myers-diff-line myers-diff-ctx">${escapeHtml(line)}</div>`;
    }).join('');
    return `<div class="myers-diff-box">${htmlLines}</div>`;
  }

  function setInboxStatusFilter(status, btn) {
    state.inboxStatusFilter = status || 'pending';
    document.querySelectorAll('#inbox-status-filters .subtab-btn').forEach((el) => el.classList.remove('active'));
    if (btn) btn.classList.add('active');
    const summaryEl = document.getElementById('inbox-filter-summary');
    if (summaryEl) {
      const labels = {
        pending: 'Exibindo propostas pendentes de aprovacao humana',
        executed: 'Exibindo propostas ja executadas',
        rejected: 'Exibindo propostas rejeitadas pelo operador',
        all: 'Exibindo todo o histórico de aprovações'
      };
      summaryEl.textContent = labels[state.inboxStatusFilter] || '';
    }
    loadAgentInboxTab(false);
  }
  window.setInboxStatusFilter = setInboxStatusFilter;

  async function loadAgentInboxTab(isManual = false) {
    const listEl = document.getElementById('inbox-proposals-list');
    try {
      const statusParam = encodeURIComponent(state.inboxStatusFilter || 'pending');
      const res = await fetch(`/api/v1/inbox/proposals?status=${statusParam}`);
      const data = await res.json();
      updateInboxBadges(data.pending_count ?? 0);

      const proposals = Array.isArray(data.proposals) ? data.proposals : [];
      state.proposalsById = {};
      proposals.forEach((p) => { state.proposalsById[p.id] = p; });

      if (!listEl) return;
      if (!proposals.length) {
        listEl.innerHTML = '<div class="gh-empty-state">Nenhuma proposta encontrada para este filtro no Agent Inbox.</div>';
        if (isManual) toast('Agent Inbox atualizado (sem pendencias).', 'info');
        return;
      }

      listEl.innerHTML = proposals.map((p) => renderProposalCard(p)).join('');
      bindGravityLockSliders();
      if (isManual) toast(`Agent Inbox atualizado (${proposals.length} proposta(s)).`, 'success');
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.loadAgentInboxTab = loadAgentInboxTab;

  function renderProposalCard(p) {
    const isTier2 = p.tier === 'tier2_external_destructive';
    const tierText = isTier2 ? 'Tier 2 (Externo / Destrutivo)' : 'Tier 1 (Seguro / Reversivel)';
    const tierBadgeCls = isTier2 ? 'badge-danger' : 'badge-champion';
    const statusCls = p.status === 'executed' ? 'badge-value' : (p.status === 'rejected' || p.status === 'failed' ? 'badge-danger' : 'badge-lowcost');
    const taintHtml = p.tainted_origin
      ? `<span class="taint-badge">${icon('alert-triangle', 'icon-xs')} <span>Origem Externa (Revisar)</span></span>`
      : '';

    let diffHtml = '';
    if (p.myers_diff) {
      diffHtml = renderMyersDiffHtml(p.myers_diff);
    } else if (p.before_snapshot || p.after_snapshot) {
      const combined = [
        p.before_snapshot ? `--- Antes\n- ${String(p.before_snapshot).replace(/\n/g, '\n- ')}` : '',
        p.after_snapshot ? `+++ Depois\n+ ${String(p.after_snapshot).replace(/\n/g, '\n+ ')}` : ''
      ].filter(Boolean).join('\n');
      diffHtml = renderMyersDiffHtml(combined);
    }

    const controlsHtml = p.status === 'pending'
      ? `
        <div class="gravity-lock-panel">
          <div class="gravity-lock-track-wrap">
            <div class="gravity-lock-header">
              <span class="gravity-lock-label">${icon('lock', 'icon-xs')} <span>Deslize até o fim para confirmar e executar</span></span>
              <span class="u-mono gravity-lock-pct" id="gl-pct-${escapeHtml(p.id)}">0%</span>
            </div>
            <input type="range" min="0" max="100" value="0" class="gravity-lock-slider" data-proposal-id="${escapeHtml(p.id)}" aria-label="Gravity Lock para aprovar proposta" />
          </div>
          <button type="button" class="btn btn-sm btn-danger" onclick="rejectInboxProposal('${escapeHtml(p.id)}')">
            ${icon('x-circle', 'icon-sm')} <span>Rejeitar</span>
          </button>
        </div>
      `
      : `
        <div class="gh-item-meta u-mt-2">
          ${p.rejection_reason ? `Motivo da rejeicao: <strong>${escapeHtml(p.rejection_reason)}</strong>` : `Status final: <strong>${escapeHtml(p.status)}</strong>`}
        </div>
      `;

    return `
      <div class="card inbox-proposal-card" data-proposal-id="${escapeHtml(p.id)}">
        <div class="u-d-flex u-justify-between u-items-start u-flex-wrap u-gap-3 u-mb-2">
          <div>
            <div class="u-d-flex u-items-center u-gap-2 u-flex-wrap u-mb-1">
              <span class="badge ${tierBadgeCls}">${escapeHtml(tierText)}</span>
              <span class="badge ${statusCls}">${escapeHtml(p.status)}</span>
              ${taintHtml}
            </div>
            <h4 class="u-text-white u-m-0">${escapeHtml(p.title)}</h4>
          </div>
          <div class="u-mono gh-text-xs u-text-muted">
            ${escapeHtml(p.service)}.${escapeHtml(p.action)} &bull; conta: ${escapeHtml(p.account_id)}
          </div>
        </div>
        <div class="gh-item-sub u-mb-2">${escapeHtml(p.summary)} &bull; Alvo: <code>${escapeHtml(p.target_resource)}</code></div>
        ${diffHtml}
        ${controlsHtml}
      </div>
    `;
  }

  function bindGravityLockSliders() {
    document.querySelectorAll('.gravity-lock-slider').forEach((slider) => {
      if (slider.dataset.bound === '1') return;
      slider.dataset.bound = '1';
      const propId = slider.getAttribute('data-proposal-id');
      const pctEl = document.getElementById(`gl-pct-${propId}`);

      const updateProgress = () => {
        const val = parseInt(slider.value || '0', 10);
        if (pctEl) pctEl.textContent = `${val}%`;
        slider.classList.remove('is-rebounding');
      };

      const checkRelease = () => {
        if (slider.disabled) return;
        const val = parseInt(slider.value || '0', 10);
        if (val >= 96) {
          slider.value = '100';
          if (pctEl) pctEl.textContent = '100%';
          slider.disabled = true;
          approveInboxProposal(propId, slider);
        } else {
          slider.classList.add('is-rebounding');
          slider.value = '0';
          if (pctEl) pctEl.textContent = '0%';
        }
      };

      slider.addEventListener('input', updateProgress);
      slider.addEventListener('change', checkRelease);
      slider.addEventListener('pointerup', checkRelease);
      slider.addEventListener('touchend', checkRelease);
    });
  }

  async function approveInboxProposal(proposalId, sliderEl) {
    const proposal = state.proposalsById[proposalId];
    const token = proposal ? (proposal.gravity_lock_token || null) : null;
    try {
      const res = await fetch(`/api/v1/inbox/proposals/${encodeURIComponent(proposalId)}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          gravity_lock_confirmed: true,
          gravity_lock_token: token
        })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Ação confirmada e executada com sucesso!', 'success');
        loadAgentInboxTab(false);
      } else {
        toast(data.message || 'Falha ao aprovar proposta.', 'error');
        if (sliderEl) {
          sliderEl.disabled = false;
          sliderEl.value = '0';
        }
      }
    } catch (err) {
      toast('Erro ao aprovar proposta: ' + err.message, 'error');
      if (sliderEl) {
        sliderEl.disabled = false;
        sliderEl.value = '0';
      }
    }
  }

  async function rejectInboxProposal(proposalId) {
    let reason = 'Recusado pelo usuário na Central de Aprovações';
    if (typeof window.appPrompt === 'function') {
      const entered = await window.appPrompt('Motivo da recusa (opcional):', 'Operação não autorizada', {
        title: 'Recusar Ação',
        submitText: 'Recusar Ação',
        icon: 'x-circle'
      });
      if (entered === null) return;
      if (entered.trim()) reason = entered.trim();
    }
    try {
      const res = await fetch(`/api/v1/inbox/proposals/${encodeURIComponent(proposalId)}/reject`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Proposta rejeitada e registrada na auditoria.', 'info');
        loadAgentInboxTab(false);
      } else {
        toast(data.message || 'Erro ao rejeitar proposta.', 'error');
      }
    } catch (err) {
      toast('Erro ao rejeitar proposta: ' + err.message, 'error');
    }
  }
  window.rejectInboxProposal = rejectInboxProposal;

  async function loadInboxAuditTrail() {
    const listEl = document.getElementById('inbox-audit-list');
    try {
      const res = await fetch('/api/v1/inbox/audit?limit=100');
      const data = await res.json();
      const events = Array.isArray(data.events) ? data.events : [];
      if (!listEl) return;
      if (!events.length) {
        listEl.innerHTML = '<div class="gh-empty-state">Nenhum evento registrado na trilha de auditoria.</div>';
        return;
      }
      listEl.innerHTML = events.map((ev) => `
        <div class="gh-item-row">
          <div class="u-d-flex u-justify-between u-items-center u-gap-2">
            <span class="u-mono u-text-cyan">${escapeHtml(ev.tool_name)}</span>
            <span class="badge ${ev.tainted ? 'badge-lowcost' : 'badge-dominated'}">${escapeHtml(ev.outcome)}</span>
          </div>
          <div class="gh-item-sub">${escapeHtml(ev.redacted_summary)}</div>
          <div class="gh-item-meta u-mono">${escapeHtml(formatUnixTime(ev.timestamp))} &bull; conta: ${escapeHtml(ev.account_id)}</div>
        </div>
      `).join('');
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }
  window.loadInboxAuditTrail = loadInboxAuditTrail;

  // =========================================================================
  // 4. COFRE MULTI-CONTA GV2, OAUTH2 PKCE, IMPORT GWS, GOOGLE DOCTOR & GEMS
  // =========================================================================

  async function loadGoogleConnectionsVault(isManual = false) {
    await Promise.all([
      loadVaultAccountsList(),
      loadGoogleDoctorReport(isManual),
      refreshInboxBadgeCount()
    ]);
    if (isManual) toast('Conexões Google e diagnóstico atualizados.', 'success');
  }
  window.loadGoogleConnectionsVault = loadGoogleConnectionsVault;

  async function loadVaultAccountsList() {
    const listEl = document.getElementById('gv2-accounts-list');
    const countBadge = document.getElementById('gv2-accounts-count-badge');
    const sessionBadge = document.getElementById('google-session-badge');
    try {
      const res = await fetch('/api/v1/google/accounts');
      if (!res.ok) return;
      const data = await res.json();
      const accounts = Array.isArray(data.accounts) ? data.accounts : [];
      state.accounts = accounts;
      if (countBadge) countBadge.textContent = `${accounts.length} conta(s)`;
      if (sessionBadge && accounts.some((a) => a.has_oauth || a.has_nlm_bridge)) {
        sessionBadge.className = 'badge badge-value';
        sessionBadge.innerHTML = `${icon('check-circle', 'icon-xs')} <span>Google Conectado (${accounts.length})</span>`;
      }

      if (!listEl) return;
      if (!accounts.length) {
        listEl.innerHTML = '<div class="gh-empty-state">Nenhuma conta conectada ainda. Envie o JSON ao lado para conectar em 1 clique.</div>';
        return;
      }

      listEl.innerHTML = accounts.map((a) => `
        <div class="gh-item-row">
          <div class="u-d-flex u-justify-between u-items-start u-gap-2">
            <div>
              <div class="gh-item-title">
                ${escapeHtml(a.label || a.account_id)}
                ${a.is_primary ? '<span class="badge badge-champion">Principal</span>' : ''}
              </div>
              <div class="gh-item-meta">${escapeHtml(a.email || a.account_id)} &bull; ${escapeHtml(kindLabel(a.kind))}</div>
            </div>
            <button type="button" class="btn btn-xs btn-danger" onclick="deleteGoogleVaultAccount('${escapeHtml(a.account_id)}')" title="Remover conta Google">
              ${icon('trash', 'icon-xs')}
            </button>
          </div>
          <div class="u-d-flex u-items-center u-gap-2 u-flex-wrap u-mt-2">
            <span class="badge ${a.has_oauth ? 'badge-value' : 'badge-dominated'}">${a.has_oauth ? 'Login Google Ativo' : 'Aguardando Login'}</span>
            ${a.has_nlm_bridge ? '<span class="badge badge-champion">NotebookLM Web</span>' : ''}
            <select class="form-control gh-select-xs" onchange="updateGoogleAccountPolicy('${escapeHtml(a.account_id)}', this.value)" aria-label="Nível de autonomia da conta">
              <option value="read_only" ${a.policy === 'read_only' ? 'selected' : ''}>Somente Leitura</option>
              <option value="drafts_and_tasks_only" ${a.policy === 'drafts_and_tasks_only' ? 'selected' : ''}>Seguro (Rascunhos e Tarefas)</option>
              <option value="balanced_hotl_hitl" ${a.policy === 'balanced_hotl_hitl' ? 'selected' : ''}>Equilibrado (Pede Confirmação)</option>
              <option value="whitelist_write" ${a.policy === 'whitelist_write' ? 'selected' : ''}>Automação Ampla</option>
            </select>
          </div>
        </div>
      `).join('');
    } catch (err) {
      if (listEl) listEl.innerHTML = `<div class="gh-empty-state">${escapeHtml(err.message)}</div>`;
    }
  }

  async function updateGoogleAccountPolicy(accountId, policy) {
    try {
      const res = await fetch(`/api/v1/google/accounts/${encodeURIComponent(accountId)}/policy`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ policy })
      });
      const data = await res.json();
      if (res.ok) {
        toast(`Politica de '${accountId}' atualizada para ${policyLabel(policy)}.`, 'success');
        loadVaultAccountsList();
      } else {
        toast(data.message || 'Falha ao atualizar politica.', 'error');
      }
    } catch (err) {
      toast('Erro ao atualizar politica: ' + err.message, 'error');
    }
  }
  window.updateGoogleAccountPolicy = updateGoogleAccountPolicy;

  async function deleteGoogleVaultAccount(accountId) {
    if (typeof window.appConfirm === 'function') {
      const ok = await window.appConfirm(`Desconectar e remover a conta '${accountId}'?`, {
        title: 'Remover Conta Google',
        danger: true,
        confirmText: 'Remover'
      });
      if (!ok) return;
    }
    try {
      const res = await fetch(`/api/v1/google/accounts/${encodeURIComponent(accountId)}`, { method: 'DELETE' });
      if (res.ok) {
        toast(`Conta '${accountId}' removida.`, 'info');
        loadVaultAccountsList();
        loadGoogleDoctorReport(true);
      } else {
        toast('Nao foi possivel remover a conta.', 'error');
      }
    } catch (err) {
      toast('Erro ao remover conta: ' + err.message, 'error');
    }
  }
  window.deleteGoogleVaultAccount = deleteGoogleVaultAccount;

  async function startGoogleOAuthPkce() {
    const label = document.getElementById('gv2-oauth-label')?.value?.trim() || 'Google Account';
    const email = document.getElementById('gv2-oauth-email')?.value?.trim() || '';
    const kind = document.getElementById('gv2-oauth-kind')?.value || 'personal_gmail';
    const policy = document.getElementById('gv2-oauth-policy')?.value || 'balanced_hotl_hitl';
    const clientId = document.getElementById('gv2-oauth-client-id')?.value?.trim() || '';
    const clientSecret = document.getElementById('gv2-oauth-client-secret')?.value?.trim() || '';
    const box = document.getElementById('gv2-oauth-result');

    try {
      const res = await fetch('/api/v1/google/oauth/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          label,
          login_hint: email || null,
          kind,
          policy,
          client_id: clientId || null,
          client_secret: clientSecret || null
        })
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');
      if (res.ok && data.auth_url) {
        if (box) {
          box.innerHTML = `
            <div class="gh-item-title u-text-cyan">Autorização Google pronta</div>
            <div class="gh-item-sub">Credencial detectada: <strong>${data.client_id_configured ? 'Sim' : 'Ainda não — envie o JSON ou cole Client ID acima'}</strong></div>
            <a class="btn btn-xs btn-primary u-mt-2" href="${escapeHtml(data.auth_url)}" target="_blank" rel="noopener noreferrer">
              ${icon('link', 'icon-xs')} <span>Abrir Login do Google</span>
            </a>
          `;
        }
        toast('Link de login do Google pronto.', 'success');
      } else {
        if (box) box.innerHTML = `<div class="u-text-red">${escapeHtml(data.message || 'Falha ao iniciar login Google.')}</div>`;
      }
    } catch (err) {
      toast('Erro ao iniciar login Google: ' + err.message, 'error');
    }
  }
  window.startGoogleOAuthPkce = startGoogleOAuthPkce;

  async function upsertManualGoogleAccount() {
    const label = document.getElementById('gv2-oauth-label')?.value?.trim() || '';
    const email = document.getElementById('gv2-oauth-email')?.value?.trim() || '';
    const kind = document.getElementById('gv2-oauth-kind')?.value || 'personal_gmail';
    const policy = document.getElementById('gv2-oauth-policy')?.value || 'balanced_hotl_hitl';
    const clientId = document.getElementById('gv2-oauth-client-id')?.value?.trim() || null;
    const clientSecret = document.getElementById('gv2-oauth-client-secret')?.value?.trim() || null;

    if (!email && !label) {
      toast('Informe ao menos email ou rotulo da conta.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/accounts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          label: label || email,
          email,
          kind,
          policy,
          is_primary: state.accounts.length === 0,
          secondary_web_opt_in: kind === 'secondary_sandbox',
          client_id: clientId,
          client_secret: clientSecret
        })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Conta Google salva com sucesso.', 'success');
        loadVaultAccountsList();
        loadGoogleDoctorReport(true);
      } else {
        toast(data.message || 'Erro ao salvar conta.', 'error');
      }
    } catch (err) {
      toast('Erro ao registrar conta: ' + err.message, 'error');
    }
  }
  window.upsertManualGoogleAccount = upsertManualGoogleAccount;

  async function importGwsCredentials() {
    try {
      const res = await fetch('/api/v1/google/accounts/import-gws', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      });
      const data = await res.json();
      if (data.status === 'imported') {
        toast('Credenciais do gws importadas automaticamente!', 'success');
        loadVaultAccountsList();
        loadGoogleDoctorReport(true);
      } else if (data.status === 'not_found') {
        toast('Nenhum login anterior do gws encontrado. Use os 3 passos acima para enviar o JSON.', 'warning');
      } else {
        toast(data.message || 'Falha ao detectar credenciais.', 'error');
      }
    } catch (err) {
      toast('Erro ao importar credenciais: ' + err.message, 'error');
    }
  }
  window.importGwsCredentials = importGwsCredentials;

  async function triggerNlmOneClickLogin(force = false) {
    const statusBox = document.getElementById('nlm-login-status');
    const btn = document.getElementById('nlm-login-btn');
    if (btn) btn.disabled = true;
    try {
      const res = await fetch('/api/v1/google/nlm/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force: Boolean(force) })
      });
      const data = await res.json();
      if (statusBox) statusBox.classList.remove('is-hidden');
      if (data.status === 'already_authenticated') {
        if (statusBox) {
          statusBox.innerHTML = `<div class="u-text-green u-fw-600">${escapeHtml(data.message || 'NotebookLM já está conectado e pronto!')}</div>`;
        }
        toast('NotebookLM já conectado e pronto!', 'success');
        loadGoogleDoctorReport(true);
      } else if (data.status === 'browser_opened') {
        if (statusBox) {
          statusBox.innerHTML = `
            <div class="u-text-cyan u-fw-600">Janela do navegador aberta!</div>
            <div class="gh-item-sub u-mt-1">${escapeHtml(data.message || 'Faça login na sua conta Google na janela que abriu e depois clique em Verificar Conexão.')}</div>
            <button type="button" class="btn btn-xs btn-secondary u-mt-2" onclick="loadGoogleDoctorReport(true)">
              ${icon('refresh', 'icon-xs')} <span>Já fiz login, verificar agora</span>
            </button>
          `;
        }
        toast('Navegador aberto para login no NotebookLM.', 'info');
      } else {
        if (statusBox) statusBox.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Não foi possível iniciar o login automático.')}</div>`;
        toast(data.message || 'Falha ao abrir login do NotebookLM.', 'warning');
      }
    } catch (err) {
      toast('Erro ao conectar NotebookLM: ' + err.message, 'error');
    } finally {
      if (btn) btn.disabled = false;
    }
  }
  window.triggerNlmOneClickLogin = triggerNlmOneClickLogin;

  async function triggerGwsOneClickLogin() {
    const box = document.getElementById('gv2-oauth-result');
    try {
      const res = await fetch('/api/v1/google/gws/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: '{}'
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');
      if (data.status === 'browser_opened') {
        if (box) {
          box.innerHTML = `
            <div class="u-text-cyan u-fw-600">Navegador aberto para conectar Workspace</div>
            <div class="gh-item-sub">${escapeHtml(data.message || '')}</div>
          `;
        }
        toast('Navegador aberto para login Google Workspace.', 'info');
      } else {
        if (box) {
          box.innerHTML = `<div class="u-text-amber">${escapeHtml(data.message || 'Use o botão Selecionar JSON acima para conectar sem depender de linha de comando.')}</div>`;
        }
        toast(data.message || 'Siga os 3 passos com o JSON acima.', 'warning');
      }
    } catch (err) {
      toast('Erro ao iniciar login Workspace: ' + err.message, 'error');
    }
  }
  window.triggerGwsOneClickLogin = triggerGwsOneClickLogin;

  function toggleGoogleJsonPaste() {
    const el = document.getElementById('gv2-json-paste-box');
    if (el) el.classList.toggle('is-hidden');
  }
  window.toggleGoogleJsonPaste = toggleGoogleJsonPaste;

  function handleGoogleSecretFileUpload(inputEl) {
    const file = inputEl && inputEl.files && inputEl.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = function () {
      const raw = String(reader.result || '');
      const textArea = document.getElementById('gv2-json-raw');
      if (textArea) textArea.value = raw;
      uploadGoogleClientJsonAndLogin(raw);
    };
    reader.readAsText(file);
  }
  window.handleGoogleSecretFileUpload = handleGoogleSecretFileUpload;

  async function uploadGoogleClientJsonAndLogin(rawJsonOverride) {
    const raw = typeof rawJsonOverride === 'string'
      ? rawJsonOverride
      : (document.getElementById('gv2-json-raw')?.value?.trim() || '');
    const email = document.getElementById('gv2-oauth-email')?.value?.trim() || null;
    const label = document.getElementById('gv2-oauth-label')?.value?.trim() || null;
    const kind = document.getElementById('gv2-oauth-kind')?.value || null;
    const policy = document.getElementById('gv2-oauth-policy')?.value || 'balanced_hotl_hitl';
    const box = document.getElementById('gv2-oauth-result');

    if (!raw) {
      toast('Selecione o arquivo client_secret.json ou cole o conteúdo primeiro.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/oauth/upload-credentials', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          client_secret_json: raw,
          email,
          label,
          kind,
          policy,
          auto_start_oauth: true
        })
      });
      const data = await res.json();
      if (box) box.classList.remove('is-hidden');
      if (res.ok && data.auth_url) {
        if (box) {
          box.innerHTML = `
            <div class="gh-item-title u-text-green">Credencial salva! Falta só confirmar no Google:</div>
            <div class="gh-item-sub u-mt-1">Se a aba do Google não abriu automaticamente, clique no botão abaixo:</div>
            <a class="btn btn-xs btn-primary u-mt-2" href="${escapeHtml(data.auth_url)}" target="_blank" rel="noopener noreferrer">
              ${icon('link', 'icon-xs')} <span>Confirmar Acesso na Conta Google</span>
            </a>
          `;
        }
        toast('Credencial salva! Abrindo página de login do Google...', 'success');
        window.open(data.auth_url, '_blank', 'noopener,noreferrer');
        loadVaultAccountsList();
        loadGoogleDoctorReport(true);
      } else {
        if (box) box.innerHTML = `<div class="u-text-red">${escapeHtml(data.message || 'Arquivo JSON inválido ou incompleto.')}</div>`;
        toast(data.message || 'Arquivo JSON inválido.', 'error');
      }
    } catch (err) {
      toast('Erro ao processar JSON do Google: ' + err.message, 'error');
    }
  }
  window.uploadGoogleClientJsonAndLogin = uploadGoogleClientJsonAndLogin;

  async function saveInlineApiKey(envKey, inputId) {
    const input = document.getElementById(inputId);
    const val = input?.value?.trim() || '';
    if (!val || val.length < 6) {
      toast(`Cole uma chave válida para ${envKey}.`, 'warning');
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
        toast(`${envKey} salva e ativada (${data.masked || '***'})!`, 'success');
        loadGoogleDoctorReport(true);
        if (typeof window.fetchConnectionsData === 'function') {
          window.fetchConnectionsData();
        }
      } else {
        toast(data.message || `Falha ao salvar ${envKey}.`, 'error');
      }
    } catch (err) {
      toast(`Erro ao salvar ${envKey}: ` + err.message, 'error');
    }
  }
  window.saveInlineApiKey = saveInlineApiKey;

  async function loadGoogleDoctorReport(forceRefresh = false) {
    const statusBadge = document.getElementById('gv2-doctor-status-badge');
    const clisEl = document.getElementById('gv2-doctor-clis');
    const notesEl = document.getElementById('gv2-doctor-notes');
    const nlmBadge = document.getElementById('nlm-connection-badge');
    const nlmBtnLabel = document.getElementById('nlm-login-btn-label');
    const geminiKeyBadge = document.getElementById('conn-gemini-key-badge');
    const mapsKeyBadge = document.getElementById('conn-maps-key-badge');

    try {
      const qs = forceRefresh ? '?refresh=true' : '';
      const res = await fetch(`/api/v1/google/doctor${qs}`);
      if (!res.ok) return;
      const doc = await res.json();

      if (statusBadge) {
        const ok = doc.overall_status === 'ready' || doc.overall_status === 'healthy';
        statusBadge.className = `badge ${ok ? 'badge-value' : 'badge-lowcost'}`;
        statusBadge.textContent = ok ? 'Tudo Pronto' : 'Ação Pendente';
      }

      const cliFriendlyNames = {
        nlm: 'NotebookLM (nlm)',
        gws: 'Google Workspace (gws)',
        gemini: 'Gemini CLI',
        clasp: 'Apps Script (clasp)'
      };

      const nlmCli = (doc.clis || []).find((c) => c.name === 'nlm');
      const hasOAuthAcct = Array.isArray(state.accounts) && state.accounts.some((a) => a.has_oauth || a.has_nlm_bridge);
      const sessionBadge = document.getElementById('google-session-badge');
      if (sessionBadge) {
        if (hasOAuthAcct && nlmCli?.authenticated) {
          sessionBadge.className = 'badge badge-value';
          sessionBadge.innerHTML = `${icon('check-circle', 'icon-xs')} <span>Google / NotebookLM Ativo</span>`;
        } else if (hasOAuthAcct) {
          sessionBadge.className = 'badge badge-value';
          sessionBadge.innerHTML = `${icon('check-circle', 'icon-xs')} <span>Google Workspace Ativo</span>`;
        } else if (nlmCli?.authenticated) {
          sessionBadge.className = 'badge badge-value';
          sessionBadge.innerHTML = `${icon('check-circle', 'icon-xs')} <span>NotebookLM Conectado</span>`;
        } else {
          sessionBadge.className = 'badge badge-lowcost';
          sessionBadge.innerHTML = `${icon('lock', 'icon-xs')} <span>Configuração Pendente</span>`;
        }
      }
      if (nlmBadge && nlmCli) {
        if (nlmCli.authenticated) {
          nlmBadge.className = 'badge badge-value';
          nlmBadge.textContent = 'Conectado';
        } else if (nlmCli.installed) {
          nlmBadge.className = 'badge badge-lowcost';
          nlmBadge.textContent = 'Falta Entrar';
        } else {
          nlmBadge.className = 'badge badge-dominated';
          nlmBadge.textContent = 'Não Instalado';
        }
      }
      if (nlmBtnLabel && nlmCli) {
        nlmBtnLabel.textContent = nlmCli.authenticated
          ? 'Verificar / Trocar Conta NotebookLM'
          : 'Conectar NotebookLM com 1 Clique';
      }

      if (geminiKeyBadge) {
        geminiKeyBadge.className = `badge ${doc.gemini_api_key_ready ? 'badge-value' : 'badge-dominated'}`;
        geminiKeyBadge.textContent = doc.gemini_api_key_ready ? 'Configurada' : 'Opcional';
      }
      if (mapsKeyBadge) {
        mapsKeyBadge.className = `badge ${doc.routes_v2_ready || doc.maps_api_key_ready ? 'badge-value' : 'badge-dominated'}`;
        mapsKeyBadge.textContent = (doc.routes_v2_ready || doc.maps_api_key_ready) ? 'Configurada' : 'Opcional';
      }

      if (clisEl) {
        const cliRows = (doc.clis || []).map((c) => `
          <div class="gh-doctor-row">
            <div>
              <strong>${escapeHtml(cliFriendlyNames[c.name] || c.name)}</strong>
              <span class="gh-item-meta">${escapeHtml(c.version || (c.installed ? 'instalado' : 'não encontrado'))}</span>
            </div>
            <span class="badge ${c.installed ? (c.authenticated ? 'badge-value' : 'badge-lowcost') : 'badge-dominated'}">
              ${c.installed ? (c.authenticated ? 'Conectado' : 'Precisa Login') : 'Ausente'}
            </span>
          </div>
        `).join('');
        const routesRow = `
          <div class="gh-doctor-row">
            <div><strong>Previsão de Trânsito</strong> <span class="gh-item-meta">Alertas de horário de saída</span></div>
            <span class="badge ${doc.routes_v2_ready ? 'badge-value' : 'badge-dominated'}">${doc.routes_v2_ready ? 'Ativa' : 'Sem Chave'}</span>
          </div>
          <div class="gh-doctor-row">
            <div><strong>Google Gemini API</strong> <span class="gh-item-meta">Modelos Gemini na nuvem</span></div>
            <span class="badge ${doc.gemini_api_key_ready ? 'badge-value' : 'badge-dominated'}">${doc.gemini_api_key_ready ? 'Ativa' : 'Sem Chave'}</span>
          </div>
        `;
        clisEl.innerHTML = cliRows + routesRow;
      }

      if (notesEl) {
        const warnings = (doc.warnings || []).map((w) => `<div>${escapeHtml(w)}</div>`).join('');
        notesEl.innerHTML = warnings
          ? `<div class="u-text-amber">${warnings}</div>`
          : '<div class="u-text-green">Tudo pronto para sincronizar seus serviços Google.</div>';
      }
    } catch (_) {
      // Silencioso
    }
  }
  window.loadGoogleDoctorReport = loadGoogleDoctorReport;

  async function loadGoogleGems() {
    const listEl = document.getElementById('gv2-gems-list');
    try {
      const res = await fetch('/api/v1/google/gems');
      const data = await res.json();
      const gems = Array.isArray(data.gems) ? data.gems : [];
      if (!listEl) return;
      if (!gems.length) {
        listEl.innerHTML = '<div class="gh-empty-state">Nenhum Gem salvo.</div>';
        return;
      }
      listEl.innerHTML = gems.map((g) => `
        <div class="gh-item-row u-d-flex u-justify-between u-items-center u-gap-2">
          <div>
            <div class="gh-item-title">${escapeHtml(g.name)} <span class="u-mono gh-text-xs">(${escapeHtml(g.id)})</span></div>
            <div class="gh-item-meta">${escapeHtml(g.backend)} &bull; ${escapeHtml(g.model)}</div>
          </div>
          <button type="button" class="btn btn-xs btn-secondary" onclick="selectGoogleGem('${escapeHtml(g.id)}', '${escapeHtml(g.name)}', '${escapeHtml(g.backend)}', '${escapeHtml(g.system_instruction)}')">
            Selecionar
          </button>
        </div>
      `).join('');
    } catch (_) {}
  }
  window.loadGoogleGems = loadGoogleGems;

  function selectGoogleGem(id, name, backend, instruction) {
    const idEl = document.getElementById('gv2-gem-id');
    const nameEl = document.getElementById('gv2-gem-name');
    const bkEl = document.getElementById('gv2-gem-backend');
    const instEl = document.getElementById('gv2-gem-instruction');
    if (idEl) idEl.value = id;
    if (nameEl) nameEl.value = name;
    if (bkEl) bkEl.value = backend || 'gemini_api';
    if (instEl) instEl.value = instruction || '';
  }
  window.selectGoogleGem = selectGoogleGem;

  async function saveGoogleGem() {
    const id = document.getElementById('gv2-gem-id')?.value?.trim() || '';
    const name = document.getElementById('gv2-gem-name')?.value?.trim() || id;
    const backend = document.getElementById('gv2-gem-backend')?.value || 'gemini_api';
    const systemInstruction = document.getElementById('gv2-gem-instruction')?.value || '';
    if (!id) {
      toast('Informe um gem_id para salvar.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/gems', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'upsert', id, name, backend, system_instruction: systemInstruction })
      });
      const data = await res.json();
      if (res.ok) {
        toast('Gem salvo com sucesso.', 'success');
        loadGoogleGems();
      } else {
        toast(data.message || 'Falha ao salvar Gem.', 'error');
      }
    } catch (err) {
      toast('Erro ao salvar Gem: ' + err.message, 'error');
    }
  }
  window.saveGoogleGem = saveGoogleGem;

  async function dispatchGoogleGem() {
    const id = document.getElementById('gv2-gem-id')?.value?.trim() || null;
    const name = document.getElementById('gv2-gem-name')?.value?.trim() || null;
    const backend = document.getElementById('gv2-gem-backend')?.value || 'gemini_api';
    const accountId = document.getElementById('gv2-gem-account-id')?.value?.trim() || null;
    const systemInstruction = document.getElementById('gv2-gem-instruction')?.value || '';
    const prompt = document.getElementById('gv2-gem-prompt')?.value?.trim() || '';
    const outBox = document.getElementById('gv2-gem-output');

    if (!prompt) {
      toast('Informe um prompt de teste para o Gem.', 'warning');
      return;
    }
    try {
      const res = await fetch('/api/v1/google/gems', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: backend === 'secondary_web' ? 'secondary_stream_generate' : 'dispatch',
          id,
          name,
          backend,
          account_id: accountId,
          system_instruction: systemInstruction,
          prompt
        })
      });
      const data = await res.json();
      if (outBox) {
        outBox.classList.remove('is-hidden');
        outBox.innerHTML = `<pre class="gh-pre-preview">${escapeHtml(JSON.stringify(data, null, 2))}</pre>`;
      }
      if (!res.ok) {
        toast(data.message || 'Despacho bloqueado ou falhou.', 'warning');
      }
    } catch (err) {
      toast('Erro ao despachar Gem: ' + err.message, 'error');
    }
  }
  window.dispatchGoogleGem = dispatchGoogleGem;
})();
