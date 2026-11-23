// multimodal_hub.js — Voz Soberana em Português & Visão Híbrida (v0.9.3)

(function () {
  const MODE_META = {
    auto: {
      label: 'Automático',
      preview: 'Modo Automático: ajusta ritmo e acolhimento ao seu estado imediato, com pausas suaves e linguagem natural em português.'
    },
    hands_on: {
      label: 'Mão na massa',
      preview: 'Modo Mão na Massa: respostas curtas, diretas e orientadas ao próximo passo prático, sem rodeios.'
    },
    sparring: {
      label: 'Debate crítico',
      preview: 'Modo Debate Crítico: questiona premissas com respeito, testa argumentos e aponta pontos cegos.'
    },
    decompression: {
      label: 'Descompressão',
      preview: 'Modo Descompressão: escuta acolhedora, tom leve e zero cobrança para você espairecer com tranquilidade.'
    },
    crisis_paralysis: {
      label: 'Apoio na crise',
      preview: 'Modo Apoio na Crise: voz calma e estável propondo apenas um micro-passo simples e alcançável agora.'
    },
    philosophical: {
      label: 'Reflexão profunda',
      preview: 'Modo Reflexão Profunda: ritmo pausado e aberto para explorar sentidos, nuances e perguntas amplas.'
    }
  };

  const VOICE_PRESETS = {
    celebrar: 'Excelente! Fechamos essa entrega antes do prazo — e o sistema respondeu com folga e zero sobrecarga.',
    acalmar: 'Respira fundo um segundo... Não precisa resolver tudo agora. Vamos escolher apenas um passo bem simples para começar.',
    explicar: 'Essa função separa a leitura leve em texto da captura visual — assim você economiza bateria, memória e tempo de resposta.'
  };

  const EMOTION_LABELS_PT = {
    neutral: 'Tom neutro',
    happy: 'Animado',
    angry: 'Tenso ou frustrado',
    hesitant: 'Pensativo ou em dúvida',
    laughter: 'Descontraído',
    sad: 'Cansado ou abatido',
    focused: 'Concentrado'
  };

  const VERDICT_LABELS_PT = {
    continue_listening: 'Continuar ouvindo (pensamento em aberto)',
    emit_backchannel: 'Sinal breve de escuta e aguardar continuação',
    contemplative_wait: 'Aguardar pausa reflexiva antes de falar',
    take_turn_immediate: 'Pronto para responder imediatamente',
    barge_in_abort: 'Interrupção detectada — ceder a palavra',
    standby_timeout: 'Silêncio prolongado — modo de repouso'
  };

  const state = {
    activeCognitiveMode: 'auto',
    rapportProfile: {
      user_id: 'default',
      formality_score: 3.5,
      avg_sentence_length: 12,
      preferred_pacing: 'short_direct',
      slang_lexicon: [],
      mirroring_ratio: 0.25,
      active_cognitive_mode: 'auto',
      contemplative_base_ms: 250,
      backchannel_enabled: true,
      last_emotion: 'neutral',
      turn_count: 0
    },
    previousLiveAxTree: null,
    previousLiveWindowTitle: '',
    diagnostics: {
      audio_status: null,
      rapport: null,
      hermes_voice: null,
      vision_status: null,
      last_tts: null,
      last_stt: null,
      last_vision: null
    },
    mediaRecorder: null,
    mediaStream: null,
    recordedChunks: [],
    isRecording: false,
    slidersBound: false
  };

  function escapeHtml(raw) {
    if (typeof window.escapeHtml === 'function') {
      return window.escapeHtml(raw);
    }
    if (raw === null || raw === undefined) return '';
    return String(raw)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

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

  function refreshLabDiagnostics() {
    const el = document.getElementById('mm-lab-diagnostics');
    if (!el) return;
    try {
      el.textContent = JSON.stringify(state.diagnostics, null, 2);
    } catch (_) {
      el.textContent = 'Diagnóstico indisponível.';
    }
  }

  function bindRangeReadouts() {
    if (state.slidersBound) return;
    state.slidersBound = true;

    const formRange = document.getElementById('mm-formality-range');
    const formVal = document.getElementById('mm-formality-val');
    if (formRange && formVal) {
      formRange.addEventListener('input', () => {
        formVal.textContent = `${Number(formRange.value).toFixed(1)} / 10`;
      });
    }

    const waitRange = document.getElementById('mm-contemplative-ms-range');
    const waitVal = document.getElementById('mm-contemplative-ms-val');
    if (waitRange && waitVal) {
      waitRange.addEventListener('input', () => {
        waitVal.textContent = `${Math.round(Number(waitRange.value))} ms`;
      });
    }

    const mirRange = document.getElementById('mm-mirroring-range');
    const mirVal = document.getElementById('mm-mirroring-val');
    if (mirRange && mirVal) {
      mirRange.addEventListener('input', () => {
        mirVal.textContent = `${Math.round(Number(mirRange.value))}%`;
      });
    }
  }

  function applyCognitiveModeUi(mode, customPreviewText) {
    const normalized = MODE_META[mode] ? mode : 'auto';
    state.activeCognitiveMode = normalized;
    state.rapportProfile.active_cognitive_mode = normalized;

    const cards = document.querySelectorAll('#mm-cognitive-modes-grid .mm-mode-card');
    cards.forEach((card) => {
      const cardMode = card.getAttribute('data-cognitive-mode');
      card.classList.toggle('active', cardMode === normalized);
    });

    const meta = MODE_META[normalized];
    const badge = document.getElementById('mm-active-mode-badge');
    if (badge) {
      badge.innerHTML = `${icon('sparkles', 'icon-xs')} <span>Modo Ativo: ${escapeHtml(meta.label)}</span>`;
    }

    const preview = document.getElementById('mm-turn-guidance-preview');
    if (preview) {
      preview.textContent = customPreviewText || meta.preview;
    }
  }

  function applyRapportFormUi(profile, ephemeralNote) {
    if (!profile || typeof profile !== 'object') return;
    state.rapportProfile = Object.assign({}, state.rapportProfile, profile);

    const mode = profile.active_cognitive_mode || state.activeCognitiveMode || 'auto';
    applyCognitiveModeUi(mode, ephemeralNote || undefined);

    const formRange = document.getElementById('mm-formality-range');
    const formVal = document.getElementById('mm-formality-val');
    const formality = Number.isFinite(Number(profile.formality_score)) ? Number(profile.formality_score) : 3.5;
    if (formRange) formRange.value = String(formality);
    if (formVal) formVal.textContent = `${formality.toFixed(1)} / 10`;

    const pacingSelect = document.getElementById('mm-pacing-select');
    if (pacingSelect && profile.preferred_pacing) {
      pacingSelect.value = profile.preferred_pacing;
    }

    const waitRange = document.getElementById('mm-contemplative-ms-range');
    const waitVal = document.getElementById('mm-contemplative-ms-val');
    const waitMs = Number.isFinite(Number(profile.contemplative_base_ms)) ? Number(profile.contemplative_base_ms) : 250;
    if (waitRange) waitRange.value = String(waitMs);
    if (waitVal) waitVal.textContent = `${Math.round(waitMs)} ms`;

    const mirRange = document.getElementById('mm-mirroring-range');
    const mirVal = document.getElementById('mm-mirroring-val');
    const ratio = Number.isFinite(Number(profile.mirroring_ratio)) ? Number(profile.mirroring_ratio) : 0.25;
    const pct = Math.min(35, Math.max(0, Math.round(ratio * 100)));
    if (mirRange) mirRange.value = String(pct);
    if (mirVal) mirVal.textContent = `${pct}%`;

    const backToggle = document.getElementById('mm-backchannel-toggle');
    if (backToggle) {
      backToggle.checked = profile.backchannel_enabled !== false;
    }

    const slangInput = document.getElementById('mm-slang-input');
    if (slangInput) {
      const lex = Array.isArray(profile.slang_lexicon) ? profile.slang_lexicon : [];
      slangInput.value = lex.join(', ');
    }
  }

  function renderAudioStatus(audioData, visionData) {
    const vramChip = document.getElementById('mm-vram-chip');
    const ttsChip = document.getElementById('mm-tts-chip');
    const sttChip = document.getElementById('mm-stt-chip');
    const enginesList = document.getElementById('mm-voice-engines-list');

    const audioVram = Number(audioData?.vram_consumed_mb ?? 0);
    const visionVram = Number(visionData?.vram_consumed_mb ?? 0);
    const totalVram = audioVram + visionVram;
    if (vramChip) {
      vramChip.innerHTML = `${icon('cpu', 'icon-xs')} <span>${totalVram} MB de vídeo</span>`;
    }

    const inv = audioData?.inventory || audioData || {};
    const activeTts = inv.active_tts_label_pt || inv.active_tts_engine || 'Voz do Sistema';
    const activeStt = inv.active_stt_label_pt || inv.active_stt_engine || 'Escuta Local';

    if (ttsChip) {
      ttsChip.innerHTML = `${icon('message-square', 'icon-xs')} <span>Voz: ${escapeHtml(activeTts)}</span>`;
    }
    if (sttChip) {
      sttChip.innerHTML = `${icon('mic', 'icon-xs')} <span>Escuta: ${escapeHtml(activeStt)}</span>`;
    }

    if (enginesList) {
      const items = [
        {
          label: 'Voz neural rápida em português',
          ready: Boolean(inv.onnx_kokoro_ready || inv.piper_ready)
        },
        {
          label: 'Escuta local com percepção de tom',
          ready: Boolean(inv.onnx_sensevoice_ready || inv.whisper_ready)
        },
        {
          label: 'Detector inteligente de pausas',
          ready: Boolean(inv.silero_vad_ready)
        },
        {
          label: 'Voz nativa de reserva do sistema',
          ready: inv.windows_speech_ready !== false
        }
      ];

      enginesList.innerHTML = items.map((item) => {
        const badgeCls = item.ready ? 'badge badge-value' : 'badge badge-lowcost';
        const ic = item.ready ? icon('check-circle', 'icon-xs') : icon('clock', 'icon-xs');
        const stateTxt = item.ready ? 'Pronto' : 'Reserva';
        return `<span class="${badgeCls}">${ic} <span>${escapeHtml(item.label)}: ${stateTxt}</span></span>`;
      }).join('');
    }
  }

  function renderHermesVoiceBanner(report) {
    const banner = document.getElementById('mm-hermes-voice-banner');
    const summary = document.getElementById('mm-hermes-voice-summary');
    const issuesEl = document.getElementById('mm-hermes-voice-issues');
    if (!banner || !summary || !issuesEl) return;

    if (!report || typeof report !== 'object') {
      summary.textContent = 'Não foi possível ler o perfil de voz do Hermes no momento.';
      issuesEl.innerHTML = '';
      return;
    }

    const synced = Boolean(report.is_synced_with_souls_pt_br);
    banner.classList.toggle('customs-warn', !synced);
    summary.textContent = report.human_summary_pt || (synced
      ? 'Hermes e Telegram já estão alinhados para conversar em português local.'
      : 'Algumas opções de idioma ou voz local no Hermes ainda precisam de ajuste.');

    const issues = Array.isArray(report.issues_found) ? report.issues_found : [];
    if (issues.length === 0) {
      issuesEl.innerHTML = `<span class="badge badge-value">${icon('check-circle', 'icon-xs')} <span>Tudo alinhado em português</span></span>`;
    } else {
      issuesEl.innerHTML = issues.map((issue) => (
        `<div class="mm-issue-item">${icon('alert-triangle', 'icon-xs u-text-amber')} <span>${escapeHtml(issue)}</span></div>`
      )).join('');
    }
  }

  async function fetchJsonSafe(url, options) {
    try {
      const res = await fetch(url, options);
      const data = await res.json().catch(() => null);
      return { ok: res.ok, status: res.status, data };
    } catch (err) {
      return { ok: false, status: 0, data: null, error: err };
    }
  }

  async function loadMultimodalHubTab(force = false) {
    bindRangeReadouts();

    const [audioRes, rapportRes, hermesRes, visionRes] = await Promise.all([
      fetchJsonSafe('/api/v1/audio/status'),
      fetchJsonSafe('/api/v1/audio/rapport'),
      fetchJsonSafe('/api/v1/hermes/voice-customs'),
      fetchJsonSafe('/api/v1/vision/status')
    ]);

    if (audioRes.ok && audioRes.data) {
      state.diagnostics.audio_status = audioRes.data;
    }
    if (visionRes.ok && visionRes.data) {
      state.diagnostics.vision_status = visionRes.data;
    }
    renderAudioStatus(audioRes.data, visionRes.data);

    if (rapportRes.ok && rapportRes.data) {
      state.diagnostics.rapport = rapportRes.data;
      const profile = rapportRes.data.profile || rapportRes.data;
      const note = rapportRes.data.ephemeral_turn_note_pt || '';
      applyRapportFormUi(profile, note);
    } else {
      applyCognitiveModeUi(state.activeCognitiveMode);
    }

    if (hermesRes.ok && hermesRes.data) {
      state.diagnostics.hermes_voice = hermesRes.data;
      renderHermesVoiceBanner(hermesRes.data.report || hermesRes.data);
    } else {
      renderHermesVoiceBanner(null);
    }

    refreshLabDiagnostics();
    if (force) {
      toast('Status de Voz & Visão atualizado.', 'success');
    }
  }

  async function selectCognitiveVoiceMode(mode) {
    applyCognitiveModeUi(mode);

    const payload = Object.assign({}, state.rapportProfile, {
      active_cognitive_mode: state.activeCognitiveMode
    });

    const res = await fetchJsonSafe('/api/v1/audio/rapport', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (res.ok && res.data) {
      state.diagnostics.rapport = res.data;
      const profile = res.data.profile || res.data;
      const note = res.data.ephemeral_turn_note_pt || '';
      applyRapportFormUi(profile, note);
      refreshLabDiagnostics();
      toast(`Modo "${MODE_META[state.activeCognitiveMode].label}" ativado.`, 'success');
    } else {
      toast(`Prévia ajustada para "${MODE_META[state.activeCognitiveMode].label}".`, 'info');
    }
  }

  async function syncHermesVoiceCustoms() {
    const btn = document.getElementById('btn-sync-hermes-voice');
    if (btn) btn.disabled = true;

    const res = await fetchJsonSafe('/api/v1/hermes/voice-customs/sync', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ apply: true })
    });

    if (btn) btn.disabled = false;

    if (res.ok && res.data) {
      state.diagnostics.hermes_voice = res.data;
      renderHermesVoiceBanner(res.data.report || res.data);
      refreshLabDiagnostics();
      toast('Voz em português sincronizada no Hermes com sucesso!', 'success');
    } else {
      toast(res.data?.message || 'Não foi possível sincronizar o perfil de voz do Hermes.', 'warning');
    }
  }

  async function provisionAudioEngines() {
    const btn = document.getElementById('btn-provision-audio');
    if (btn) btn.disabled = true;
    toast('Baixando e preparando modelo local de reconhecimento de voz em português (0 MB de vídeo)...', 'info');

    const res = await fetchJsonSafe('/api/v1/audio/provision', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({})
    });

    if (btn) btn.disabled = false;

    if (res.ok && res.data) {
      state.diagnostics.audio_status = res.data;
      renderAudioStatus(res.data, state.diagnostics.vision_status);
      refreshLabDiagnostics();
      toast(res.data.message || 'Voz local em português preparada e ativada (0 MB de vídeo)!', 'success');
    } else {
      toast(res.data?.message || 'Falha ao baixar e preparar voz local em português.', 'warning');
    }
  }

  async function testVoiceSynthesize() {
    const inputEl = document.getElementById('mm-tts-input');
    const previewEl = document.getElementById('mm-tts-shaped-preview');
    const audioEl = document.getElementById('mm-tts-audio');
    const text = (inputEl?.value || '').trim();

    if (!text) {
      toast('Digite uma frase em português para ouvir a síntese.', 'warning');
      return;
    }

    if (previewEl) {
      previewEl.textContent = 'Preparando ritmo respiratório e sintetizando voz no processador...';
    }

    const res = await fetchJsonSafe('/api/v1/audio/synthesize', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        input: text,
        cognitive_mode: state.activeCognitiveMode,
        apply_prosody_shaping: true
      })
    });

    if (!res.ok || !res.data) {
      if (previewEl) {
        previewEl.textContent = res.data?.message || 'Serviço de síntese indisponível no momento.';
      }
      toast('Não foi possível sintetizar a fala agora.', 'warning');
      return;
    }

    state.diagnostics.last_tts = Object.assign({}, res.data, {
      audio_base64: res.data.audio_base64 ? `[${res.data.audio_base64.length} chars]` : ''
    });
    refreshLabDiagnostics();

    const shaped = res.data.shaped_text || text;
    if (previewEl) {
      const waitMs = Number(res.data.contemplative_delay_ms || 0);
      previewEl.innerHTML = `
        <div class="u-fw-600 u-text-white u-mb-1">Texto com pausas naturais (${escapeHtml(res.data.engine_name || 'voz local')}, espera ${waitMs} ms):</div>
        <div class="u-text-slate">${escapeHtml(shaped)}</div>
      `;
    }

    if (audioEl && res.data.audio_base64) {
      const fmt = (res.data.audio_format || 'wav').toLowerCase();
      const mime = fmt === 'mp3' ? 'audio/mpeg' : (fmt === 'ogg' ? 'audio/ogg' : 'audio/wav');
      audioEl.src = `data:${mime};base64,${res.data.audio_base64}`;
      audioEl.classList.remove('is-hidden');
      audioEl.play().catch(() => {});
    }
  }

  function testVoicePreset(presetKey) {
    const inputEl = document.getElementById('mm-tts-input');
    const sample = VOICE_PRESETS[presetKey];
    if (inputEl && sample) {
      inputEl.value = sample;
    }
    return testVoiceSynthesize();
  }

  function renderTranscriptionResult(data) {
    const box = document.getElementById('mm-stt-result-box');
    if (!box) return;

    if (!data) {
      box.textContent = 'Não foi possível avaliar o áudio agora.';
      return;
    }

    const clean = data.clean_text || data.text || '(sem fala detectada)';
    const emoKey = String(data.emotion_tag || 'neutral').toLowerCase();
    const emoLabel = EMOTION_LABELS_PT[emoKey] || emoKey;
    const verdictKey = String(data.turn_evaluation?.verdict || 'take_turn_immediate').toLowerCase();
    const verdictLabel = VERDICT_LABELS_PT[verdictKey] || verdictKey;
    const waitMs = Number(data.turn_evaluation?.recommended_wait_ms ?? 0);

    box.innerHTML = `
      <div class="u-d-flex u-gap-2 u-flex-wrap u-mb-2">
        <span class="badge badge-champion">${icon('sparkles', 'icon-xs')} <span>Humor percebido: ${escapeHtml(emoLabel)}</span></span>
        <span class="badge badge-value">${icon('clock', 'icon-xs')} <span>Ritmo do turno: ${escapeHtml(verdictLabel)} (${waitMs} ms)</span></span>
      </div>
      <div class="u-text-white u-fw-600 u-mb-1">Transcrição limpa:</div>
      <div class="u-text-slate">${escapeHtml(clean)}</div>
    `;
  }

  function blobToBase64(blob) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = String(reader.result || '');
        const commaIdx = result.indexOf(',');
        resolve(commaIdx >= 0 ? result.slice(commaIdx + 1) : result);
      };
      reader.onerror = () => reject(new Error('Falha ao ler arquivo de áudio.'));
      reader.readAsDataURL(blob);
    });
  }

  async function sendAudioForTranscription(payload) {
    const box = document.getElementById('mm-stt-result-box');
    if (box) {
      box.textContent = 'Avaliando fala, pausas e tom no processador...';
    }

    const res = await fetchJsonSafe('/api/v1/audio/transcribe', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (res.ok && res.data) {
      state.diagnostics.last_stt = res.data;
      refreshLabDiagnostics();
      renderTranscriptionResult(res.data);
      toast('Avaliação de fala concluída.', 'success');
    } else {
      if (box) {
        box.textContent = res.data?.message || 'Não foi possível transcrever o áudio enviado.';
      }
      toast('Falha ao avaliar áudio.', 'warning');
    }
  }

  function updateMicButtonUi(recording) {
    const btn = document.getElementById('btn-mm-mic-toggle');
    if (!btn) return;
    btn.classList.toggle('is-recording', recording);
    btn.innerHTML = recording
      ? `${icon('square', 'icon-sm')} <span>Parar Gravação</span>`
      : `${icon('mic', 'icon-sm')} <span>Gravar pelo Microfone</span>`;
  }

  async function toggleMicRecording() {
    if (state.isRecording && state.mediaRecorder) {
      state.isRecording = false;
      updateMicButtonUi(false);
      try {
        state.mediaRecorder.stop();
      } catch (_) {}
      return;
    }

    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia || typeof window.MediaRecorder === 'undefined') {
      toast('Seu navegador não liberou gravação direta aqui. Você pode enviar um arquivo de áudio ou testar a avaliação.', 'info');
      return;
    }

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      state.mediaStream = stream;
      state.recordedChunks = [];
      const recorder = new MediaRecorder(stream);
      state.mediaRecorder = recorder;

      recorder.ondataavailable = (ev) => {
        if (ev.data && ev.data.size > 0) {
          state.recordedChunks.push(ev.data);
        }
      };

      recorder.onstop = async () => {
        const mimeType = recorder.mimeType || 'audio/webm';
        const blob = new Blob(state.recordedChunks, { type: mimeType });
        if (state.mediaStream) {
          state.mediaStream.getTracks().forEach((t) => t.stop());
          state.mediaStream = null;
        }
        try {
          const audioBase64 = await blobToBase64(blob);
          await sendAudioForTranscription({
            audio_base64: audioBase64,
            filename: 'microfone.webm',
            mime_type: mimeType,
            language: 'pt'
          });
        } catch (err) {
          toast(err.message || 'Erro ao preparar áudio gravado.', 'warning');
        }
      };

      state.isRecording = true;
      updateMicButtonUi(true);
      recorder.start();
    } catch (_) {
      state.isRecording = false;
      updateMicButtonUi(false);
      toast('Permissão do microfone não concedida. Use "Enviar Áudio" ou "Testar Avaliação de Fala".', 'warning');
    }
  }

  async function testTranscriptUpload() {
    const fileInput = document.getElementById('mm-stt-file');
    const file = fileInput && fileInput.files && fileInput.files[0];

    if (file) {
      try {
        const audioBase64 = await blobToBase64(file);
        await sendAudioForTranscription({
          audio_base64: audioBase64,
          filename: file.name,
          mime_type: file.type || 'audio/wav',
          language: 'pt'
        });
      } catch (err) {
        toast(err.message || 'Falha ao ler o arquivo escolhido.', 'warning');
      }
      return;
    }

    await sendAudioForTranscription({
      sample_text: 'Então, eu queria revisar essa ideia agora porque ainda estou decidindo o melhor caminho...',
      silence_duration_ms: 480,
      language: 'pt'
    });
  }

  function updateBlindSpotBanner(blindSpot) {
    const banner = document.getElementById('mm-vision-blindspot-banner');
    const textEl = document.getElementById('mm-vision-blindspot-text');
    if (!banner || !textEl) return;

    if (blindSpot && blindSpot.requires_pixel_snapshot) {
      textEl.textContent = blindSpot.recommendation_pt || 'Área gráfica detectada — recomendamos capturar uma imagem com marcações.';
      banner.classList.remove('is-hidden');
    } else {
      banner.classList.add('is-hidden');
    }
  }

  function getVisionFilterParams() {
    const winEl = document.getElementById('mm-vision-window-input');
    const intentEl = document.getElementById('mm-vision-intent-select');
    const modeEl = document.getElementById('mm-vision-mode-select');
    return {
      window_filter: (winEl?.value || '').trim() || null,
      intent_filter: intentEl?.value || 'actionable',
      mode: modeEl?.value || 'auto'
    };
  }

  async function inspectLiveScreen() {
    const outEl = document.getElementById('mm-vision-lean-output');
    if (outEl) outEl.textContent = 'Lendo estrutura da tela em texto...';

    const params = getVisionFilterParams();
    const res = await fetchJsonSafe('/api/v1/vision/axtree/live', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        window_filter: params.window_filter,
        intent_filter: params.intent_filter
      })
    });

    if (!res.ok || !res.data) {
      if (outEl) {
        outEl.textContent = res.data?.message || 'Leitura ao vivo indisponível no momento.';
      }
      toast('Não foi possível ler a tela agora.', 'warning');
      return;
    }

    state.diagnostics.last_vision = res.data;
    refreshLabDiagnostics();

    const liveTree = res.data.tree || res.data.root || res.data.ax_tree || null;
    if (liveTree) {
      state.previousLiveAxTree = liveTree;
      state.previousLiveWindowTitle = res.data.window_title || '';
    }

    updateBlindSpotBanner(res.data.blind_spot);

    if (outEl) {
      outEl.textContent = res.data.lean_text || 'Nenhum elemento estrutural visível encontrado com esse filtro.';
    }
    toast('Leitura rápida da tela concluída com 0 MB de vídeo.', 'success');
  }

  async function diffScreenSnapshots() {
    const outEl = document.getElementById('mm-vision-lean-output');
    if (!state.previousLiveAxTree) {
      await inspectLiveScreen();
      toast('Referência inicial salva! Mude algo na tela e clique em "Comparar Mudanças" novamente.', 'info');
      return;
    }

    if (outEl) outEl.textContent = 'Comparando mudanças desde a última leitura...';
    const params = getVisionFilterParams();

    const res = await fetchJsonSafe('/api/v1/vision/axtree/diff', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        previous_tree: state.previousLiveAxTree,
        window_filter: params.window_filter,
        intent_filter: params.intent_filter
      })
    });

    if (!res.ok || !res.data) {
      if (outEl) {
        outEl.textContent = res.data?.message || 'Falha ao comparar mudanças da tela.';
      }
      toast('Não foi possível comparar mudanças.', 'warning');
      return;
    }

    state.diagnostics.last_vision = res.data;
    refreshLabDiagnostics();

    const nextTree = res.data.current_tree || res.data.tree || res.data.after_tree || null;
    if (nextTree) {
      state.previousLiveAxTree = nextTree;
    }

    const diff = res.data.diff || res.data;
    const savings = Number(diff.token_savings_pct ?? 0).toFixed(0);
    const compact = diff.compact_lean_diff || 'Nenhuma mudança estrutural detectada na janela.';

    if (outEl) {
      outEl.textContent = `[Economia de contexto: ${savings}%]\n\n${compact}`;
    }
    toast('Comparação concluída enviando apenas as diferenças.', 'success');
  }

  async function captureOnDemandSnapshot() {
    const previewEl = document.getElementById('mm-vision-snapshot-preview');
    const somListEl = document.getElementById('mm-vision-som-list');
    if (previewEl) previewEl.textContent = 'Capturando visualização sob demanda...';
    if (somListEl) somListEl.innerHTML = '';

    const params = getVisionFilterParams();
    const res = await fetchJsonSafe('/api/v1/vision/snapshot', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params)
    });

    if (!res.ok || !res.data) {
      if (previewEl) {
        previewEl.textContent = res.data?.message || 'Captura sob demanda indisponível.';
      }
      toast('Não foi possível capturar a tela agora.', 'warning');
      return;
    }

    const snap = res.data.snapshot || res.data;
    state.diagnostics.last_vision = Object.assign({}, snap, {
      image_base64_png: snap.image_base64_png ? `[${snap.image_base64_png.length} chars]` : null
    });
    refreshLabDiagnostics();

    updateBlindSpotBanner(snap.blind_spot);

    if (previewEl) {
      const parts = [];
      if (snap.screen_unchanged) {
        parts.push(`<div class="u-mb-2"><span class="badge badge-value">${icon('check-circle', 'icon-xs')} <span>Tela inalterada — nenhum recurso gasto</span></span></div>`);
      }
      if (snap.window_title || snap.app_name) {
        parts.push(`<div class="gh-text-xs u-text-muted u-mb-2">${escapeHtml(snap.app_name || '')} ${snap.window_title ? '— ' + escapeHtml(snap.window_title) : ''}</div>`);
      }
      if (snap.image_base64_png) {
        parts.push(`<img class="mm-snapshot-img" src="data:image/png;base64,${escapeHtml(snap.image_base64_png)}" alt="Captura da tela com marcações" />`);
      } else if (!snap.screen_unchanged) {
        parts.push('<div class="gh-text-xs u-text-muted">Captura estrutural concluída sem necessidade de imagem completa.</div>');
      }
      if (snap.visual_analysis) {
        parts.push(`<div class="mm-shaped-preview u-mt-2">${escapeHtml(snap.visual_analysis)}</div>`);
      }
      previewEl.innerHTML = parts.join('');
    }

    if (somListEl) {
      const marks = Array.isArray(snap.som_marks) ? snap.som_marks : [];
      if (marks.length === 0) {
        somListEl.innerHTML = '<div class="gh-text-xs u-text-muted">Nenhum elemento numerado retornado nesta janela.</div>';
      } else {
        somListEl.innerHTML = marks.slice(0, 30).map((m) => {
          const cx = Array.isArray(m.click_point) ? Math.round(Number(m.click_point[0] || 0)) : 0;
          const cy = Array.isArray(m.click_point) ? Math.round(Number(m.click_point[1] || 0)) : 0;
          return `
            <div class="mm-som-chip">
              <span class="badge badge-champion">#${escapeHtml(m.index)}</span>
              <span class="u-fw-600 u-text-white">${escapeHtml(m.label || m.node_id || 'Elemento')}</span>
              <span class="u-text-muted">(${escapeHtml(m.role || 'item')} · ${cx}, ${cy})</span>
            </div>
          `;
        }).join('');
      }
    }

    toast(snap.screen_unchanged ? 'Tela idêntica à anterior — zero gasto extra.' : 'Captura inteligente concluída.', 'success');
  }

  async function saveRapportPreferences() {
    const formRange = document.getElementById('mm-formality-range');
    const pacingSelect = document.getElementById('mm-pacing-select');
    const waitRange = document.getElementById('mm-contemplative-ms-range');
    const mirRange = document.getElementById('mm-mirroring-range');
    const backToggle = document.getElementById('mm-backchannel-toggle');
    const slangInput = document.getElementById('mm-slang-input');

    const formality = Number(formRange?.value ?? 3.5);
    const pacing = pacingSelect?.value || 'short_direct';
    const contemplativeMs = Math.round(Number(waitRange?.value ?? 250));
    const mirRatio = Math.min(0.35, Math.max(0, Number(mirRange?.value ?? 25) / 100));
    const backchannelEnabled = Boolean(backToggle?.checked ?? true);
    const slangLexicon = String(slangInput?.value || '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);

    const payload = Object.assign({}, state.rapportProfile, {
      formality_score: formality,
      preferred_pacing: pacing,
      contemplative_base_ms: contemplativeMs,
      mirroring_ratio: mirRatio,
      backchannel_enabled: backchannelEnabled,
      slang_lexicon: slangLexicon,
      active_cognitive_mode: state.activeCognitiveMode
    });

    const res = await fetchJsonSafe('/api/v1/audio/rapport', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (res.ok && res.data) {
      state.diagnostics.rapport = res.data;
      const profile = res.data.profile || res.data;
      const note = res.data.ephemeral_turn_note_pt || '';
      applyRapportFormUi(profile, note);
      refreshLabDiagnostics();
      toast('Preferências de estilo e ritmo salvas!', 'success');
    } else {
      toast(res.data?.message || 'Não foi possível salvar suas preferências agora.', 'warning');
    }
  }

  window.loadMultimodalHubTab = loadMultimodalHubTab;
  window.selectCognitiveVoiceMode = selectCognitiveVoiceMode;
  window.syncHermesVoiceCustoms = syncHermesVoiceCustoms;
  window.provisionAudioEngines = provisionAudioEngines;
  window.testVoiceSynthesize = testVoiceSynthesize;
  window.testVoicePreset = testVoicePreset;
  window.toggleMicRecording = toggleMicRecording;
  window.testTranscriptUpload = testTranscriptUpload;
  window.inspectLiveScreen = inspectLiveScreen;
  window.diffScreenSnapshots = diffScreenSnapshots;
  window.captureOnDemandSnapshot = captureOnDemandSnapshot;
  window.saveRapportPreferences = saveRapportPreferences;
})();
