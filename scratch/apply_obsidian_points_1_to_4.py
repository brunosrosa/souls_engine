# -*- coding: utf-8 -*-
import sys

def main():
    file_path = "crates/souls_server/src/dashboard_ui.html"
    with open(file_path, "r", encoding="utf-8") as f:
        text = f.read()

    # =========================================================================
    # 1. PONTO #1: TOP 4 CARDS STYLED WITH THEIR SUBSYSTEM COLOR THEME
    # =========================================================================
    p_cards_start = text.find("<!-- 4 CARDS DE KPI CENTRADOS")
    p_cards_end = text.find("<!-- BANNER DE GOVERNANÇA DO COFRE", p_cards_start)
    assert p_cards_start != -1 and p_cards_end != -1, "Cards section not found"

    new_cards = """<!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico — 100% Reativos & Colorizados por Subsistema) -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 16px;">
          <!-- CARD 1: INTEGRAÇÃO DO COFRE (CIANO) -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-integration" onclick="focusVaultSetting('cfg-obsidian-storage-type')" title="Clique para focar na configuração de integração do cofre" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(56, 189, 248, 0.4); background: linear-gradient(180deg, rgba(56, 189, 248, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--cyan); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>📦</span> <span>INTEGRAÇÃO DO COFRE</span>
            </div>
            <div class="mono" id="kpi-vault-integration" style="font-size: 1.05rem; font-weight: 700; color: var(--cyan); white-space: nowrap;">
              ONEDRIVE SYNC
            </div>
            <div id="kpi-vault-integration-sub" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Sincronizado na Nuvem
            </div>
            <div style="font-size: 0.60rem; color: #64748b; margin-top: 4px; display: flex; align-items: center; gap: 3px;">
              <span>Ajustar abaixo</span> <span>↓</span>
            </div>
          </div>

          <!-- CARD 2: ZONA BRUTA RAW (VERDE) -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-raw" onclick="focusVaultSetting('cfg-obsidian-raw-ro')" title="Clique para alternar a proteção da zona raw/" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(16, 185, 129, 0.4); background: linear-gradient(180deg, rgba(16, 185, 129, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--green); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>🔒</span> <span>ZONA BRUTA (RAW/)</span>
            </div>
            <div class="mono" id="kpi-vault-raw" style="font-size: 1.05rem; font-weight: 700; color: var(--green); white-space: nowrap;">
              READ-ONLY TOTAL
            </div>
            <div id="kpi-vault-raw-sub" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              🔒 Imutável & Sagrado (Lei 1)
            </div>
            <div style="font-size: 0.60rem; color: #64748b; margin-top: 4px; display: flex; align-items: center; gap: 3px;">
              <span>Ajustar abaixo</span> <span>↓</span>
            </div>
          </div>

          <!-- CARD 3: PRIVACY SHIELD PII (ROXO) -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-privacy" onclick="focusVaultSetting('cfg-obsidian-privacy-shield')" title="Clique para focar nas regras do Privacy Shield" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(168, 85, 247, 0.4); background: linear-gradient(180deg, rgba(168, 85, 247, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--purple-light); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>🛡️</span> <span>PRIVACY SHIELD PII</span>
            </div>
            <div class="mono" id="kpi-vault-privacy" style="font-size: 1.05rem; font-weight: 700; color: var(--green); white-space: nowrap;">
              HÍBRIDO (SILÍCIO)
            </div>
            <div id="kpi-vault-privacy-sub" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Desidentificação PII Ativa
            </div>
            <div style="font-size: 0.60rem; color: #64748b; margin-top: 4px; display: flex; align-items: center; gap: 3px;">
              <span>Ajustar abaixo</span> <span>↓</span>
            </div>
          </div>

          <!-- CARD 4: VAZÃO ESTIMADA THE DRIP (ÂMBAR) -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-cadence" onclick="focusVaultSetting('cfg-obsidian-batch-size')" title="Clique para ajustar o tamanho do lote e cadência" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid rgba(245, 158, 11, 0.4); background: linear-gradient(180deg, rgba(245, 158, 11, 0.08) 0%, rgba(13, 19, 36, 0.85) 100%);">
            <div style="font-size: 0.66rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--amber); margin-bottom: 4px; display: flex; align-items: center; gap: 4px;">
              <span>⏱️</span> <span>VAZÃO ESTIMADA (THE DRIP)</span>
            </div>
            <div class="mono" id="metric-obsidian-cadence" style="font-size: 1.05rem; font-weight: 700; color: var(--amber); white-space: nowrap;">
              3 NOTAS / 30 MIN
            </div>
            <div id="metric-obsidian-cadence-sub" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              ~144 notas/dia • ~60/turno 10h
            </div>
            <div style="font-size: 0.60rem; color: #64748b; margin-top: 4px; display: flex; align-items: center; gap: 3px;">
              <span>Ajustar abaixo</span> <span>↓</span>
            </div>
          </div>
        </div>\n\n        """

    text = text[:p_cards_start] + new_cards + text[p_cards_end:]

    # =========================================================================
    # 2. PONTO #1 & #2: REPLACE CARD X TEXT WITH JUST EMOTICONS AND HARMONIC SPACING
    # =========================================================================
    old_tag1 = """<span>Modo de Integração</span>
                    <span class="badge" title="Configuração diretamente refletida no Card 1 acima" style="background: rgba(56, 189, 248, 0.12); color: var(--cyan); border: 1px solid rgba(56, 189, 248, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                      Card 1 ↑
                    </span>"""

    new_tag1 = """<span>Modo de Integração</span>
                    <span class="badge" title="Vinculado ao Card 📦 Integração do Cofre (Topo)" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan); border: 1px solid rgba(56, 189, 248, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">📦</span>"""

    assert old_tag1 in text, "old_tag1 not found"
    text = text.replace(old_tag1, new_tag1, 1)

    old_tag_batch = """<span>Micro-Lotes por Ciclo</span>
                      <span class="badge" title="Configuração diretamente refletida no Card 4 acima" style="background: rgba(245, 158, 11, 0.12); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Card 4 ↑
                      </span>"""

    new_tag_batch = """<span>Micro-Lotes por Ciclo</span>
                      <span class="badge" title="Vinculado ao Card ⏱️ Vazão Estimada The Drip (Topo)" style="background: rgba(245, 158, 11, 0.15); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">⏱️</span>"""

    assert old_tag_batch in text, "old_tag_batch not found"
    text = text.replace(old_tag_batch, new_tag_batch, 1)

    old_tag_int = """<span>Intervalo entre Ciclos</span>
                      <span class="badge" title="Configuração diretamente refletida no Card 4 acima" style="background: rgba(245, 158, 11, 0.12); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Card 4 ↑
                      </span>"""

    new_tag_int = """<span>Intervalo entre Ciclos</span>
                      <span class="badge" title="Vinculado ao Card ⏱️ Vazão Estimada The Drip (Topo)" style="background: rgba(245, 158, 11, 0.15); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">⏱️</span>"""

    assert old_tag_int in text, "old_tag_int not found"
    text = text.replace(old_tag_int, new_tag_int, 1)

    old_tag_priv = """<span>Obsidian Privacy Shield</span>
                      <span class="badge" title="Configuração diretamente refletida no Card 3 acima" style="background: rgba(168, 85, 247, 0.12); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Card 3 ↑
                      </span>"""

    new_tag_priv = """<span>Obsidian Privacy Shield</span>
                      <span class="badge" title="Vinculado ao Card 🛡️ Privacy Shield PII (Topo)" style="background: rgba(168, 85, 247, 0.15); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">🛡️</span>"""

    assert old_tag_priv in text, "old_tag_priv not found"
    text = text.replace(old_tag_priv, new_tag_priv, 1)

    old_tag_raw = """<span>Proteção Read-Only (raw/)</span>
                      <span class="badge" title="Configuração diretamente refletida no Card 2 acima" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Card 2 ↑
                      </span>"""

    new_tag_raw = """<span>Proteção Read-Only (raw/)</span>
                      <span class="badge" title="Vinculado ao Card 🔒 Zona Bruta (raw/) (Topo)" style="background: rgba(16, 185, 129, 0.15); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">🔒</span>"""

    assert old_tag_raw in text, "old_tag_raw not found"
    text = text.replace(old_tag_raw, new_tag_raw, 1)

    old_tag_pmode = """<span>Modo do Privacy Shield</span>
                      <span class="badge" title="Configuração diretamente refletida no Card 3 acima" style="background: rgba(168, 85, 247, 0.12); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Card 3 ↑
                      </span>"""

    new_tag_pmode = """<span>Modo do Privacy Shield</span>
                      <span class="badge" title="Vinculado ao Card 🛡️ Privacy Shield PII (Topo)" style="background: rgba(168, 85, 247, 0.15); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.4); font-size: 0.74rem; padding: 2px 7px; border-radius: 4px; margin-left: 8px; margin-right: 6px; display: inline-flex; align-items: center; justify-content: center;">🛡️</span>"""

    assert old_tag_pmode in text, "old_tag_pmode not found"
    text = text.replace(old_tag_pmode, new_tag_pmode, 1)

    # =========================================================================
    # 3. PONTO #3: LIVE BARE-METAL VAULT ENGINE IN PLACE OF TOY SIMULATOR
    # =========================================================================
    p_sim_start = text.find("<!-- CARD 4: SIMULADOR DE HIGIENIZAÇÃO")
    p_sim_end = text.find("</div>\n          </div>\n        </div>", p_sim_start)
    assert p_sim_start != -1 and p_sim_end != -1, "Simulator card section not found"

    new_sim_card = """<!-- CARD 4: MOTOR DE SANITIZAÇÃO PII & AUDITORIA NO SILÍCIO (LIVE ENGINE) -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px; justify-content: space-between;">
                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem;">🛡️</div>
                  Motor de Sanitização PII & Auditoria no Silício (Live Engine)
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>Inspeção Bare-Metal em Tempo Real</strong>
                      Executa varredura de PII e verificação de Lei 1 diretamente no daemon Rust (127.0.0.1:9123). Redige dados pessoais e protege notas brutas antes de qualquer tráfego para modelos de nuvem.
                    </span>
                  </span>
                </div>
                <span class="badge" id="badge-vault-engine-status" style="background: rgba(16, 185, 129, 0.15); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem;">Rust Bare-Metal Ativo</span>
              </div>

              <div style="font-size: 0.72rem; color: var(--text-muted); margin-bottom: 10px;">
                Carregue notas reais do seu cofre ou digite qualquer texto para inspecionar a desidentificação e a blindagem da Lei 1:
              </div>

              <!-- BOTÕES DE CARGA DE AMOSTRAS / NOTAS REAIS DO COFRE -->
              <div style="display: flex; gap: 6px; margin-bottom: 10px; flex-wrap: wrap;">
                <button class="sim-preset-btn" onclick="loadRealVaultSample('saude')" style="font-size: 0.70rem; padding: 4px 9px;">
                  🩺 Amostra: raw/saude
                </button>
                <button class="sim-preset-btn" onclick="loadRealVaultSample('financas')" style="font-size: 0.70rem; padding: 4px 9px;">
                  💳 Amostra: raw/financas
                </button>
                <button class="sim-preset-btn" onclick="loadRealVaultSample('deep_research')" style="font-size: 0.70rem; padding: 4px 9px; border-color: rgba(56, 189, 248, 0.4); color: var(--cyan);">
                  📑 Nota Real: raw/deep-researchs
                </button>
                <button class="sim-preset-btn" onclick="loadRealVaultSample('law1_violation')" style="font-size: 0.70rem; padding: 4px 9px; border-color: rgba(239, 68, 68, 0.4); color: #f87171;">
                  🚫 Teste Lei 1: Violação raw/
                </button>
              </div>

              <!-- CAMPO DE ENTRADA DO MOTOR -->
              <div style="margin-bottom: 10px;">
                <textarea id="sim-vault-input" class="setting-input" style="width: 100%; height: 64px; font-size: 0.74rem; font-family: var(--font-mono); resize: vertical; padding: 8px 10px;" placeholder="Insira texto clínico, dados financeiros ou notas para inspecionar no motor bare-metal...">Paciente: Bruno Rosa | CPF: 123.456.789-00 | CRM: 48921-SP | Lab Fleury | Hemoglobina Glicada: 5.4% | Glicemia Jejum: 88 mg/dL</textarea>
              </div>

              <button class="btn btn-green" id="btn-run-vault-sanitize" onclick="executeRealVaultSanitization()" style="width: 100%; justify-content: center; padding: 8px 14px; font-size: 0.76rem; font-weight: 700; margin-bottom: 12px;">
                <span>⚡</span> Executar Sanitização Bare-Metal no Silício
              </button>

              <!-- PAINEL DE RESULTADOS DA SIMULAÇÃO AO VIVO -->
              <div style="background: rgba(10, 16, 28, 0.85); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 10px 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; flex-wrap: wrap; gap: 6px;">
                  <span id="sim-vault-verdict" class="badge" style="background: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.68rem; font-weight: 700;">
                    ✓ PII HIGIENIZADO NO SILÍCIO
                  </span>
                  <span id="sim-vault-latency" style="font-size: 0.68rem; color: var(--text-muted); font-family: var(--font-mono);">
                    Latência Bare-Metal: <strong style="color: var(--cyan);">&lt; 0.02 ms (14 µs)</strong> &bull; Rust Engine
                  </span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <div style="font-size: 0.68rem; color: #94a3b8; font-weight: 600;">Saída Sanitizada para o Modelo de Nuvem:</div>
                  <div id="sim-vault-tags" style="font-size: 0.64rem; font-family: var(--font-mono); color: var(--green);">4 Entidades Redigidas</div>
                </div>
                <div id="sim-vault-preview" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 4px; padding: 7px 9px; font-size: 0.72rem; font-family: var(--font-mono); color: #e2e8f0; line-height: 1.45; max-height: 80px; overflow-y: auto; margin-bottom: 6px;">
                  Paciente: [USUARIO_ALVO] | CPF: [CPF_PROTEGIDO] | CRM: [MEDICO_CRM_OCULTO] | Lab: [LAB_REFERENCIA] | Hemoglobina Glicada: 5.4% | Glicemia Jejum: 88 mg/dL
                </div>
                <div id="sim-vault-rule" style="font-size: 0.68rem; color: var(--green); font-family: var(--font-mono);">
                  ✓ Obsidian Privacy Shield (§1 & §2 Desidentificação no Silício)
                </div>
              </div>
            </div>"""

    text = text[:p_sim_start] + new_sim_card + text[p_sim_end:]

    # =========================================================================
    # 4. PONTO #3 (JS): API FUNCTIONS FOR LIVE VAULT ENGINE
    # =========================================================================
    p1 = text.find("function setVaultScenario(type) {")
    p2 = text.find("function resetObsidianDefaults() {", p1)
    assert p1 != -1 and p2 != -1, "JS setVaultScenario section not found"

    new_js_sim = """async function loadRealVaultSample(category) {
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

    """

    text = text[:p1] + new_js_sim + text[p2:]

    # =========================================================================
    # 5. PONTO #4: UNIVERSAL GLOBAL PORTAL TOOLTIP HANDLER FOR ALL 62 INFO-ICONS
    # =========================================================================
    target_dom = "window.addEventListener('DOMContentLoaded', () => {"
    assert target_dom in text, "DOMContentLoaded target not found"

    tooltip_functions = """    // =========================================================
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
      initGlobalPortalTooltips();"""

    target_full = "// INICIALIZAÇÃO NO CARREGAMENTO DA PÁGINA\n    window.addEventListener('DOMContentLoaded', () => {"
    assert target_full in text, "target_full not found"
    text = text.replace(target_full, tooltip_functions, 1)

    with open(file_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("SUCCESS! Applied Points 1, 2, 3, and 4 cleanly to dashboard_ui.html!")

if __name__ == "__main__":
    main()
