import re

with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    content = f.read()

# 1. Update Card 3 Title & Subtitle:
c3_target = """        <!-- CARD 3: TRÍADE CANÔNICA RECOMENDADA -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">⚡</div>
                Seleção Canônica do Silício (Tríade de Alta Performance)
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Os 3 modelos que extraem o teto máximo de velocidade e inteligência da NVIDIA RTX 2060m (5.2 GB VRAM útil).
              </p>
            </div>
          </div>"""

c3_replacement = """        <!-- CARD 3: O TRIO RECOMENDADO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">⚡</div>
                O Trio Recomendado
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Os 3 modelos essenciais para velocidade, raciocínio e código — calibrados para rodar com folga nos 6 GB da sua placa de vídeo.
              </p>
            </div>
          </div>"""

assert c3_target in content, "c3_target not found in content"
content = content.replace(c3_target, c3_replacement, 1)
print("Card 3 updated successfully!")

# 2. Update Card 4 Title, Subtitle, Sync Button, and View Mode Toggles:
c4_target = """        <!-- CARD 4: MODELOS HOMOLOGADOS POR ESPECIALIDADE -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-purple">🎯</div>
                Modelos Homologados por Especialidade
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Modelos calibrados por domínio cognitivo para máxima eficiência no silício local. Escolha a quantização ideal e baixe em 1 clique.
              </p>
            </div>
            <button class="btn btn-purple" onclick="fetchHardwareAndRecommendations()">
              <span>✨</span> Sincronizar Catálogo
            </button>
          </div>
          <div class="profile-selector" id="curated-profile-filters">
            <button class="profile-pill active" onclick="filterCuratedProfile('all')">Todos os Perfis</button>
            <button class="profile-pill" onclick="filterCuratedProfile('coding')">💻 Coding Specialist</button>
            <button class="profile-pill" onclick="filterCuratedProfile('reasoning')">🧠 Deep Reasoning (R1 CoT)</button>
            <button class="profile-pill" onclick="filterCuratedProfile('agentic')">🤖 Agentic & Tool Calling</button>
            <button class="profile-pill" onclick="filterCuratedProfile('design')">🎨 UI/UX & Frontend Design</button>
            <button class="profile-pill" onclick="filterCuratedProfile('vision')">👁️ Vision Multimodal (VLM)</button>
            <button class="profile-pill" onclick="filterCuratedProfile('fast')">⚡ Fast Reflex & Triage</button>
          </div>"""

c4_replacement = """        <!-- CARD 4: MELHORES ESCOLHAS POR FUNÇÃO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header" style="flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between;">
            <div>
              <div class="card-title">
                <div class="card-icon icon-purple">🎯</div>
                Melhores Escolhas por Função
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Modelos testados para o seu computador, organizados pelo que você precisa: Programação, Raciocínio Lógico, Visão ou Automação.
              </p>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
              <span id="catalog-last-sync-badge" class="badge" style="background: rgba(16, 185, 129, 0.12); color: #34d399; border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.68rem; padding: 3px 8px; font-weight: 600; display: none;">
                ✓ Sincronizado
              </span>
              <button id="btn-sync-catalog" class="btn btn-purple" onclick="syncCuratedCatalogLive()">
                <span id="btn-sync-catalog-icon">✨</span> Sincronizar Catálogo
              </button>
            </div>
          </div>

          <!-- CONTROLE DE VISUALIZAÇÃO & FILTROS HARMONIZADOS -->
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; flex-wrap: wrap; gap: 10px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 12px;">
            <div class="segmented-control" id="curated-view-mode" style="display: inline-flex; background: rgba(15, 23, 42, 0.6); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 3px; gap: 4px;">
              <button class="segmented-btn active" id="btn-curated-mode-trusted" onclick="setCuratedViewMode('trusted')" style="padding: 5px 12px; font-size: 0.75rem; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s; font-weight: 600; display: flex; align-items: center; gap: 6px; background: var(--purple); color: #fff;">
                <span>⭐</span> Mais Confiáveis
              </button>
              <button class="segmented-btn" id="btn-curated-mode-trending" onclick="setCuratedViewMode('trending')" style="padding: 5px 12px; font-size: 0.75rem; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s; font-weight: 500; display: flex; align-items: center; gap: 6px; background: transparent; color: var(--text-muted);">
                <span>🔥</span> Novidades em Alta
              </button>
            </div>

            <div class="profile-selector" id="curated-profile-filters" style="margin-bottom: 0;">
              <button class="profile-pill active" onclick="filterCuratedProfile('all')">Todos os Perfis</button>
              <button class="profile-pill" onclick="filterCuratedProfile('coding')">💻 Programação</button>
              <button class="profile-pill" onclick="filterCuratedProfile('reasoning')">🧠 Raciocínio (R1)</button>
              <button class="profile-pill" onclick="filterCuratedProfile('agentic')">🤖 Agêntico / MCP</button>
              <button class="profile-pill" onclick="filterCuratedProfile('design')">🎨 UI / Frontend</button>
              <button class="profile-pill" onclick="filterCuratedProfile('vision')">👁️ Visão (VLM)</button>
              <button class="profile-pill" onclick="filterCuratedProfile('fast')">⚡ Triagem Rápida</button>
            </div>
          </div>"""

assert c4_target in content, "c4_target not found in content"
content = content.replace(c4_target, c4_replacement, 1)
print("Card 4 updated successfully!")

# 3. Update Card 5 Title & Subtitle:
c5_target = """        <!-- CARD 5: EXPLORADOR HUGGING FACE HUB -->
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">🤗</div>
                Explorador Hugging Face Hub (GGUF)
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Pesquise qualquer repositório GGUF com estimativa preditiva de compatibilidade para a RTX 2060m.
              </p>
            </div>
          </div>"""

c5_replacement = """        <!-- CARD 5: EXPLORADOR HUGGING FACE HUB -->
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">🌐</div>
                Explorar Qualquer Modelo (Hugging Face)
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Pesquise qualquer novidade no catálogo global com estimativa automática se cabe ou não na sua máquina.
              </p>
            </div>
          </div>"""

assert c5_target in content, "c5_target not found in content"
content = content.replace(c5_target, c5_replacement, 1)
print("Card 5 updated successfully!")

with open("crates/souls_server/src/dashboard_ui.html", "w", encoding="utf-8") as f:
    f.write(content)

print("All HTML cards successfully patched!")

