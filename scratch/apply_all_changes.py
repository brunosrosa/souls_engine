import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# Ensure strictly LF
content = content.replace('\r\n', '\n')

# -------------------------------------------------------------
# 1. Modality Filter Pills HTML
# -------------------------------------------------------------
target_pills = """              <div class="filter-pills-group" id="cloud-modality-filter-pills">
                <button class="filter-provider-btn active" id="btn-cloud-modal-all" onclick="setCloudModalityFilter('all')">🌐 Todas</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-multi" onclick="setCloudModalityFilter('multimodal')" title="Apenas modelos multimodais que processam imagens e visão">👁️ Multimodal (Visão)</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-text" onclick="setCloudModalityFilter('text')" title="Modelos focados puramente em texto, código e raciocínio">📝 Texto Puro</button>
              </div>"""

replacement_pills = """              <div class="filter-pills-group" id="cloud-modality-filter-pills" style="display: flex; gap: 4px; flex-wrap: wrap;">
                <button class="filter-provider-btn active" id="btn-cloud-modal-all" onclick="setCloudModalityFilter('all')">🌐 Todas</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-vision" onclick="setCloudModalityFilter('vision')" title="Apenas modelos multimodais que processam imagens, diagramas e OCR">👁️ Visão & OCR</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-image_gen" onclick="setCloudModalityFilter('image_gen')" title="Modelos que geram imagens e artefatos visuais diretamente (Text-to-Image / Canvas)">🎨 Geração de Imagem</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-audio" onclick="setCloudModalityFilter('audio')" title="Modelos conversacionais de voz nativa e baixa latência (Realtime)">🎙️ Áudio & Voz</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-video" onclick="setCloudModalityFilter('video')" title="Modelos de compreensão temporal e dinâmica de múltiplos frames">🎬 Vídeo & Temporal</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-world" onclick="setCloudModalityFilter('world')" title="Modelos de simulação física, ambientes espaciais e mundos 3D interativos">🌍 World Models</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-decision" onclick="setCloudModalityFilter('decision')" title="Motores de decisão não-autorregressivos e tipagem estrita (estilo Jev / TypeSafe AI)">⚡ Decisão (Jev)</button>
                <button class="filter-provider-btn" id="btn-cloud-modal-text" onclick="setCloudModalityFilter('text')" title="Modelos focados puramente em texto, código e raciocínio simbólico">📝 Texto Puro</button>
              </div>"""

assert target_pills in content, "Error: target_pills not found in content"
content = content.replace(target_pills, replacement_pills)
print("1. Modality Pills replaced successfully.")

# -------------------------------------------------------------
# 2. setCloudModalityFilter function
# -------------------------------------------------------------
target_set_mod = """    function setCloudModalityFilter(modality) {
      currentCloudModalityFilter = modality;
      document.querySelectorAll('#cloud-modality-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const activeBtn = document.getElementById(`btn-cloud-modal-${modality === 'multimodal' ? 'multi' : (modality === 'text' ? 'text' : 'all')}`);
      if (activeBtn) activeBtn.classList.add('active');
      renderCloudTable();
      renderCloudRankedTable();
    }"""

replacement_set_mod = """    function setCloudModalityFilter(modality) {
      currentCloudModalityFilter = modality;
      document.querySelectorAll('#cloud-modality-filter-pills .filter-provider-btn').forEach(btn => btn.classList.remove('active'));
      const btnId = `btn-cloud-modal-${modality === 'multimodal' ? 'vision' : modality}`;
      const activeBtn = document.getElementById(btnId) || document.getElementById('btn-cloud-modal-all');
      if (activeBtn) activeBtn.classList.add('active');
      renderCloudTable();
      renderCloudRankedTable();
    }"""

assert target_set_mod in content, "Error: target_set_mod not found in content"
content = content.replace(target_set_mod, replacement_set_mod)
print("2. setCloudModalityFilter replaced successfully.")

# -------------------------------------------------------------
# 3. getFilteredCloudModelsList function (Modalities filtering)
# -------------------------------------------------------------
target_get_filt = """      // 2. Filtrar por Modalidade
      if (currentCloudModalityFilter && currentCloudModalityFilter !== 'all') {
        if (currentCloudModalityFilter === 'multimodal') {
          list = list.filter(m => m.is_multimodal === 1 || m.is_multimodal === true);
        } else if (currentCloudModalityFilter === 'text') {
          list = list.filter(m => !m.is_multimodal || m.is_multimodal === 0);
        }
      }"""

replacement_get_filt = """      // 2. Filtrar por Modalidade Expandida (Visão, Imagem, Áudio, Vídeo, World Models, Decisão, Texto)
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
      }"""

assert target_get_filt in content, "Error: target_get_filt not found in content"
content = content.replace(target_get_filt, replacement_get_filt)
print("3. getFilteredCloudModelsList replaced successfully.")

# -------------------------------------------------------------
# 4. capTags & Model Cell Rendering (Tag do lab antes do nome, linha 1 apenas capacidades)
# -------------------------------------------------------------
target_cap_tags = """        // Tags de capacidade ao lado do nome do modelo
        let capTags = [];
        if (m.is_multimodal) {
          capTags.push('<span class="cap-tag cap-vision" title="Multimodal: Processa imagens e visão computacional">👁️ Visão</span>');
        }
        if (m.supports_caching) {
          capTags.push('<span class="cap-tag cap-caching" title="Prompt Caching: Economia de até 80% em contexto repetido">⚡ Cache</span>');
        }
        if (m.supports_tools) {
          capTags.push('<span class="cap-tag cap-tools" title="Function Calling: Suporte nativo a chamadas de ferramentas e MCP">🤖 Tools</span>');
        }
        if (m.supports_structured_outputs) {
          capTags.push('<span class="cap-tag cap-schema" title="Structured Outputs: Garante aderência estrita a schemas JSON">📑 JSON</span>');
        }"""

replacement_cap_tags = """        // Tags de capacidade ao lado do nome do modelo
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
        }"""

assert target_cap_tags in content, "Error: target_cap_tags not found in content"
content = content.replace(target_cap_tags, replacement_cap_tags)
print("4. capTags replaced successfully.")

# Target cell layout
target_cell = """            <!-- 2. MODELO / IDENTIFICADOR (CONGELADO ANTES DO STATUS) -->
            <td class="sticky-col-2">
              <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; width: 100%; min-width: 0;">
                <div style="min-width: 0; flex: 1; overflow: hidden;">
                  <div style="display: flex; align-items: center; gap: 5px; flex-wrap: wrap; margin-bottom: 4px;">
                    ${brandBadge}
                    ${capTags.join('')}
                  </div>
                  <div class="model-name-title" title="${m.canonical_id}" style="font-weight: 600; color: #fff; font-size: 0.84rem; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 290px;">
                    ${friendlyName}
                  </div>"""

replacement_cell = """            <!-- 2. MODELO / IDENTIFICADOR (CONGELADO ANTES DO STATUS) -->
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
                  </div>"""

assert target_cell in content, "Error: target_cell not found in content"
content = content.replace(target_cell, replacement_cell)
print("5. Cell layout replaced successfully (brandBadge before model name, line 1 capabilities only).")

# -------------------------------------------------------------
# 5. modal-quick-apply HTML & JS
# -------------------------------------------------------------
target_modal_apply = """  <!-- MODAL RÁPIDO DE APLICAÇÃO DE MODELO -->
  <div class="modal-backdrop" id="modal-quick-apply">
    <div class="modal-box" style="max-width: 480px;">
      <div class="modal-header">
        <div style="font-weight: 700; font-size: 1.05rem; color: #fff; display: flex; align-items: center; gap: 8px;">
          <span style="color: var(--cyan);">⚡</span> Aplicar Modelo no Hermes Agent
        </div>
        <button class="modal-close" onclick="closeQuickApplyModal()">&times;</button>
      </div>
      <div class="modal-body">
        <div style="background: rgba(15, 23, 42, 0.7); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 16px;">
          <div style="font-size: 0.65rem; color: var(--text-muted); font-family: var(--font-mono); text-transform: uppercase;">Modelo Selecionado:</div>
          <div id="quick-apply-model-name" style="font-weight: 700; color: var(--cyan); font-family: var(--font-mono); font-size: 0.95rem; margin-top: 4px; word-break: break-all;">--</div>
          <div id="quick-apply-provider-tag" style="font-size: 0.72rem; color: var(--text-muted); margin-top: 4px;">--</div>
        </div>

        <div class="modal-field">
          <label class="modal-label">Onde você deseja aplicar este modelo?</label>
          <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 8px;">
            <label style="display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); background: rgba(255,255,255,0.02);">
              <input type="radio" name="apply-target" value="master" checked style="margin-top: 3px;" onchange="toggleApplyTarget()">
              <div>
                <strong style="color: #fff; font-size: 0.85rem;">👑 Master Brain Geral (Hermes Chat Mestre)</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">Atualiza o <code>config.yaml</code> do Hermes Agent para usar este modelo no chat diário interativo.</div>
              </div>
            </label>
            <label style="display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); background: rgba(255,255,255,0.02);">
              <input type="radio" name="apply-target" value="profile" style="margin-top: 3px;" onchange="toggleApplyTarget()">
              <div>
                <strong style="color: #fff; font-size: 0.85rem;">🤖 Perfil de Trabalho (Subagente)</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">Associa este modelo a um perfil de subagente delegado pelo ParetoBandit.</div>
              </div>
            </label>
          </div>
        </div>

        <div class="modal-field" id="field-select-profile" style="display: none;">
          <label class="modal-label" for="quick-apply-profile-select">Selecione o Perfil de Trabalho:</label>
          <select id="quick-apply-profile-select" class="setting-input" style="width: 100%;">
            <!-- Preenchido dinamicamente com os perfis -->
          </select>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn" onclick="closeQuickApplyModal()">Cancelar</button>
        <button class="btn btn-cyan" onclick="submitQuickApply()">
          <span>⚡</span> Confirmar e Aplicar
        </button>
      </div>
    </div>
  </div>"""

replacement_modal_apply = """  <!-- MODAL RÁPIDO DE APLICAÇÃO DE MODELO -->
  <div class="modal-backdrop" id="modal-quick-apply">
    <div class="modal-box" style="max-width: 520px;">
      <div class="modal-header">
        <div style="font-weight: 700; font-size: 1.05rem; color: #fff; display: flex; align-items: center; gap: 8px;">
          <span style="color: var(--cyan);">⚡</span> Aplicar Modelo no Hermes & Souls Engine
        </div>
        <button class="modal-close" onclick="closeQuickApplyModal()">&times;</button>
      </div>
      <div class="modal-body">
        <div style="background: rgba(15, 23, 42, 0.7); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 16px;">
          <div style="font-size: 0.65rem; color: var(--text-muted); font-family: var(--font-mono); text-transform: uppercase;">Modelo Selecionado:</div>
          <div id="quick-apply-model-name" style="font-weight: 700; color: var(--cyan); font-family: var(--font-mono); font-size: 0.95rem; margin-top: 4px; word-break: break-all;">--</div>
          <div id="quick-apply-provider-tag" style="font-size: 0.72rem; color: var(--text-muted); margin-top: 4px;">--</div>
        </div>

        <div class="modal-field">
          <label class="modal-label">Onde você deseja aplicar este modelo?</label>
          <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 8px;">
            <!-- Opção 1: Master Brain -->
            <label style="display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); background: rgba(255,255,255,0.02);">
              <input type="radio" name="apply-target" value="master" checked style="margin-top: 3px;" onchange="toggleApplyTarget()">
              <div>
                <strong style="color: #fff; font-size: 0.85rem;">👑 Master Brain Geral (Hermes Chat Mestre)</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">Atualiza o <code>config.yaml</code> do Hermes Agent para usar este modelo no chat diário interativo.</div>
              </div>
            </label>
            <!-- Opção 2: Domínio Canônico de Subagente -->
            <label style="display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); background: rgba(255,255,255,0.02);">
              <input type="radio" name="apply-target" value="domain" style="margin-top: 3px;" onchange="toggleApplyTarget()">
              <div>
                <strong style="color: #fff; font-size: 0.85rem;">🤖 Domínio Canônico de Subagente (Especialidade)</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">Atribui o modelo à especialidade de subagente delegada pelo ParetoBandit (Coding, Reasoning, Fast, etc.).</div>
              </div>
            </label>
            <!-- Opção 3: Perfil Customizado -->
            <label style="display: flex; align-items: flex-start; gap: 10px; cursor: pointer; padding: 10px; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); background: rgba(255,255,255,0.02);">
              <input type="radio" name="apply-target" value="custom_profile" style="margin-top: 3px;" onchange="toggleApplyTarget()">
              <div>
                <strong style="color: #fff; font-size: 0.85rem;">➕ Perfil de Trabalho Customizado</strong>
                <div style="font-size: 0.72rem; color: var(--text-muted); margin-top: 2px;">Associa ou cria um perfil personalizado de subagente no Souls Engine.</div>
              </div>
            </label>
          </div>
        </div>

        <!-- Seletor de Domínio Canônico -->
        <div class="modal-field" id="field-select-domain" style="display: none; margin-top: 12px;">
          <label class="modal-label" for="quick-apply-domain-select">Selecione a Especialidade / Domínio do Subagente:</label>
          <select id="quick-apply-domain-select" class="setting-input" style="width: 100%;">
            <option value="coding">💻 Coding & Software Engineering (Cirurgia Sintática & AST)</option>
            <option value="reasoning">🧠 Reasoning & Deep Audit (Raciocínio & Auditoria Socrática)</option>
            <option value="fast">⚡ Fast & Triage (Triagem & Gating de Entropia Tier 0.5)</option>
            <option value="agentic">🎯 Agentic & Delegation (Orquestração & Handoffs)</option>
            <option value="design">🎨 Design & Frontend (UI/UX & Wireframes)</option>
            <option value="web_research">🔎 Web & Intelligence (Pesquisa Web & Documentação)</option>
          </select>
          <div style="font-size: 0.70rem; color: var(--text-muted); margin-top: 4px;">O ParetoBandit desviará para este modelo quando tarefas deste domínio precisarem de computação em nuvem.</div>
        </div>

        <!-- Seletor de Perfil Customizado -->
        <div class="modal-field" id="field-select-custom-profile" style="display: none; margin-top: 12px;">
          <label class="modal-label" for="quick-apply-profile-select">Selecione o Perfil Customizado:</label>
          <select id="quick-apply-profile-select" class="setting-input" style="width: 100%;">
            <!-- Preenchido dinamicamente com os perfis -->
          </select>
          <div style="margin-top: 8px;">
            <button type="button" class="btn btn-purple" style="font-size: 0.72rem; padding: 4px 10px;" onclick="closeQuickApplyModal(); openNewProfileModal();">
              <span>➕</span> Criar Novo Perfil de Trabalho
            </button>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn" onclick="closeQuickApplyModal()">Cancelar</button>
        <button class="btn btn-cyan" onclick="submitQuickApply()">
          <span>⚡</span> Confirmar e Aplicar
        </button>
      </div>
    </div>
  </div>"""

assert target_modal_apply in content, "Error: target_modal_apply not found in content"
content = content.replace(target_modal_apply, replacement_modal_apply)
print("6. modal-quick-apply HTML replaced successfully.")

# JS functions: openQuickApplyModal, toggleApplyTarget, submitQuickApply
target_js_apply = """    function openQuickApplyModal(modelId, provider, isLocal = false) {
      quickApplyTargetModel = modelId;
      quickApplyTargetProvider = provider || (isLocal ? 'local_silicon' : 'nous');
      quickApplyIsLocal = Boolean(isLocal);

      const nameEl = document.getElementById('quick-apply-model-name');
      const tagEl = document.getElementById('quick-apply-provider-tag');
      if (nameEl) nameEl.innerText = modelId;
      if (tagEl) tagEl.innerText = `Provedor: ${quickApplyTargetProvider} • Tipo: ${isLocal ? 'Silício Bare-Metal' : 'Cloud API'}`;

      const sel = document.getElementById('quick-apply-profile-select');
      if (sel && profilesData && profilesData.length > 0) {
        sel.innerHTML = profilesData.map(p => `
          <option value="${p.id}">${p.name} (${p.role === 'orchestrator' ? '👑 Master' : '🤖 Subagente'})</option>
        `).join('');
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
      const field = document.getElementById('field-select-profile');
      if (field) {
        field.style.display = (val === 'profile') ? 'block' : 'none';
      }
    }

    async function submitQuickApply() {
      if (!quickApplyTargetModel) return;
      const val = document.querySelector('input[name="apply-target"]:checked')?.value;

      if (val === 'master') {
        closeQuickApplyModal();
        await setHermesMasterModel(quickApplyTargetModel, quickApplyTargetProvider, activePreset);
      } else {
        const profSelect = document.getElementById('quick-apply-profile-select');
        const profId = profSelect ? profSelect.value : activePreset;
        closeQuickApplyModal();
        await saveSubagentProfilePreset(profId, quickApplyTargetModel, quickApplyTargetProvider);
      }
    }"""

replacement_js_apply = """    function openQuickApplyModal(modelId, provider, isLocal = false) {
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
    }"""

assert target_js_apply in content, "Error: target_js_apply not found in content"
content = content.replace(target_js_apply, replacement_js_apply)
print("7. JS Quick Apply functions replaced successfully.")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("All modifications written to crates/souls_server/src/dashboard_ui.html successfully!")

