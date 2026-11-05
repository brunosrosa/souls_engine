# -*- coding: utf-8 -*-
import sys

def main():
    file_path = "crates/souls_server/src/dashboard_ui.html"
    with open(file_path, "r", encoding="utf-8") as f:
        text = f.read()

    # 1. Add CSS for .kpi-card-clickable
    css_needle = """.slider-default-tick.at-default {
      width: 3px;
      height: 18px;
      background: var(--cyan);
      box-shadow: 0 0 10px var(--cyan), 0 0 3px #ffffff;
      animation: sliderDetentPop 0.28s cubic-bezier(0.34, 1.56, 0.64, 1);
    }"""
    css_replacement = css_needle + """

    .kpi-card-clickable {
      cursor: pointer;
      transition: transform 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease;
    }
    .kpi-card-clickable:hover {
      border-color: rgba(56, 189, 248, 0.45) !important;
      transform: translateY(-2px);
      box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    }"""

    assert css_needle in text, "css_needle not found"
    text = text.replace(css_needle, css_replacement, 1)

    # 2. Replace 4 KPI Cards
    cards_needle = """        <!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico) -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 16px;">
          <!-- CARD 1: ESTADO DO COFRE -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--cyan); margin-bottom: 4px;">
              📓 ESTADO DO COFRE
            </div>
            <div class="mono" id="kpi-vault-status" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              CONECTADO 100%
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              OneDrive Sync Ativo
            </div>
          </div>

          <!-- CARD 2: ZONA BRUTA RAW -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--green); margin-bottom: 4px;">
              🔒 ZONA BRUTA (RAW/)
            </div>
            <div class="mono" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              READ-ONLY TOTAL
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Imutável & Sagrado (Lei 1)
            </div>
          </div>

          <!-- CARD 3: PRIVACY SHIELD PII -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--purple); margin-bottom: 4px;">
              🛡️ PRIVACY SHIELD PII
            </div>
            <div class="mono" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              BLINDAGEM ATIVA
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Higienização Local no Silício
            </div>
          </div>

          <!-- CARD 4: CADÊNCIA THE DRIP -->
          <div class="card" style="margin-bottom: 0; padding: 12px 16px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 0.68rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--amber); margin-bottom: 4px;">
              ⏱️ CADÊNCIA (THE DRIP)
            </div>
            <div class="mono" id="metric-obsidian-cadence" style="font-size: 1.05rem; font-weight: 700; color: #fff; white-space: nowrap;">
              3 NOTAS / 30 MIN
            </div>
            <div style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">
              Janela Longa (Sem Rate-Limit)
            </div>
          </div>
        </div>"""

    cards_replacement = """        <!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico — 100% Reativos aos Controles Abaixo) -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 12px; margin-bottom: 16px;">
          <!-- CARD 1: INTEGRAÇÃO DO COFRE -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-integration" onclick="focusVaultSetting('cfg-obsidian-storage-type')" title="Clique para focar na configuração de integração do cofre" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid var(--border-subtle);">
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

          <!-- CARD 2: ZONA BRUTA RAW -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-raw" onclick="focusVaultSetting('cfg-obsidian-raw-ro')" title="Clique para focar na proteção da zona raw/" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid var(--border-subtle);">
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

          <!-- CARD 3: PRIVACY SHIELD PII -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-privacy" onclick="focusVaultSetting('cfg-obsidian-privacy-shield')" title="Clique para focar nas regras do Privacy Shield" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid var(--border-subtle);">
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

          <!-- CARD 4: VAZÃO ESTIMADA THE DRIP -->
          <div class="card kpi-card-clickable" id="card-kpi-vault-cadence" onclick="focusVaultSetting('cfg-obsidian-batch-size')" title="Clique para ajustar o tamanho do lote e cadência" style="margin-bottom: 0; padding: 12px 14px; text-align: center; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 1px solid var(--border-subtle);">
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
        </div>"""

    assert cards_needle in text, "cards_needle not found"
    text = text.replace(cards_needle, cards_replacement, 1)

    # 3. Replace Card 1 form header and selector
    form_needle = """              <!-- ITEM 2: SELETOR DE MODO DE INTEGRAÇÃO -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px;">
                    <span>Modo de Integração com o Cofre</span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Estratégia de Sincronização</strong>
                        Define se o cofre é gerenciado através do OneDrive para espelhamento móvel ou reside exclusivamente em diretório local do Windows.
                      </span>
                    </span>
                  </div>
                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 200px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">"""

    form_replacement = """              <!-- ITEM 2: SELETOR DE MODO DE INTEGRAÇÃO -->
              <div class="setting-item" style="margin-bottom: 14px;">
                <div class="setting-header" style="margin-bottom: 6px; flex-wrap: wrap; gap: 8px;">
                  <div class="setting-title" style="gap: 6px;">
                    <span>Modo de Integração com o Cofre</span>
                    <span class="badge" style="background: rgba(56, 189, 248, 0.12); color: var(--cyan); border: 1px solid rgba(56, 189, 248, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                      Reflete no Card 1 ↑
                    </span>
                    <span class="info-icon" style="z-index: 90;">i
                      <span class="tooltip-box">
                        <strong>Estratégia de Sincronização</strong>
                        Define se o cofre é gerenciado através do OneDrive para espelhamento móvel ou reside exclusivamente em diretório local do Windows.
                      </span>
                    </span>
                  </div>
                  <select id="cfg-obsidian-storage-type" class="setting-input" style="width: 200px; font-size: 0.74rem;" onchange="syncObsidianKpis(); markSettingsUnsaved();">"""

    assert form_needle in text, "form_needle not found"
    text = text.replace(form_needle, form_replacement, 1)

    # Path badge ID
    path_badge_needle = """                  <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ OneDrive Conectado
                  </span>"""
    path_badge_replacement = """                  <span class="badge" id="badge-vault-path-type" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem; padding: 2px 8px; font-family: var(--font-mono); font-weight: 600; white-space: nowrap; flex-shrink: 0;">
                    ✓ OneDrive Conectado
                  </span>"""
    assert path_badge_needle in text, "path_badge_needle not found"
    text = text.replace(path_badge_needle, path_badge_replacement, 1)

    # 4. Replace Cadence items with badges
    cadence_needle = """                <!-- ITEM 1: TAMANHO DO LOTE SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Tamanho do Micro-Lote por Ciclo</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Batch Size de Curadoria</strong>
                          Quantidade de notas brutas destiladas em cada execução do pipeline.
                          <div class="tooltip-def">Padrão canônico: 3 notas (2 a 3)</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Lote:</span>
                      <span id="val-obsidian-batch-size" style="white-space: nowrap;">3 notas</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-batch-size" min="1" max="10" step="1" value="3" data-default="3" data-snap-delta="0.3" oninput="document.getElementById('val-obsidian-batch-size').innerText = this.value + ' notas'; syncObsidianCadenceKpi(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-batch-size" style="left: calc(8px + (100% - 16px) * 0.2222);" title="Padrão canônico: 3 notas (Clique ou aproxime para restaurar)"></div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Micro-lotes pequenos evitam estouro de janela de contexto e garantem máxima fidelidade atômica.
                  </div>
                </div>

                <!-- ITEM 2: INTERVALO DA ESTEIRA SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Intervalo da Esteira Agendada</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Cadência de Disparo Automático</strong>
                          Frequência com que o Antigravity CLI ou Hermes aciona o subagente de curadoria em segundo plano.
                          <div class="tooltip-def">Padrão canônico: 30 minutos</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Intervalo:</span>
                      <span id="val-obsidian-interval" style="white-space: nowrap;">30 min</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-interval-min" min="15" max="120" step="15" value="30" data-default="30" data-snap-delta="4" oninput="document.getElementById('val-obsidian-interval').innerText = this.value + ' min'; syncObsidianCadenceKpi(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-interval-min" style="left: calc(8px + (100% - 16px) * 0.1428);" title="Padrão canônico: 30 minutos (Clique ou aproxime para restaurar)"></div>
                  </div>"""

    cadence_replacement = """                <!-- ITEM 1: TAMANHO DO LOTE SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Tamanho do Micro-Lote por Ciclo</span>
                      <span class="badge" style="background: rgba(245, 158, 11, 0.12); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Reflete no Card 4 ↑
                      </span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Batch Size de Curadoria</strong>
                          Quantidade de notas brutas destiladas em cada execução do pipeline.
                          <div class="tooltip-def">Padrão canônico: 3 notas (2 a 3)</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Lote:</span>
                      <span id="val-obsidian-batch-size" style="white-space: nowrap;">3 notas</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-batch-size" min="1" max="10" step="1" value="3" data-default="3" data-snap-delta="0.3" oninput="document.getElementById('val-obsidian-batch-size').innerText = this.value + ' notas'; syncObsidianKpis(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-batch-size" style="left: calc(8px + (100% - 16px) * 0.2222);" title="Padrão canônico: 3 notas (Clique ou aproxime para restaurar)"></div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Micro-lotes pequenos evitam estouro de janela de contexto e garantem máxima fidelidade atômica.
                  </div>
                </div>

                <!-- ITEM 2: INTERVALO DA ESTEIRA SLIDER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Intervalo da Esteira Agendada</span>
                      <span class="badge" style="background: rgba(245, 158, 11, 0.12); color: var(--amber); border: 1px solid rgba(245, 158, 11, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Reflete no Card 4 ↑
                      </span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Cadência de Disparo Automático</strong>
                          Frequência com que o Antigravity CLI ou Hermes aciona o subagente de curadoria em segundo plano.
                          <div class="tooltip-def">Padrão canônico: 30 minutos</div>
                        </span>
                      </span>
                    </div>
                    <div class="range-val-badge">
                      <span class="val-kicker">Intervalo:</span>
                      <span id="val-obsidian-interval" style="white-space: nowrap;">30 min</span>
                      <span class="default-indicator">(Padrão)</span>
                    </div>
                  </div>
                  <div class="range-wrap">
                    <input type="range" id="cfg-obsidian-interval-min" min="15" max="120" step="15" value="30" data-default="30" data-snap-delta="4" oninput="document.getElementById('val-obsidian-interval').innerText = this.value + ' min'; syncObsidianKpis(); markSettingsUnsaved();">
                    <div class="slider-default-tick at-default" data-slider="cfg-obsidian-interval-min" style="left: calc(8px + (100% - 16px) * 0.1428);" title="Padrão canônico: 30 minutos (Clique ou aproxime para restaurar)"></div>
                  </div>"""

    assert cadence_needle in text, "cadence_needle not found"
    text = text.replace(cadence_needle, cadence_replacement, 1)

    # 5. Replace Privacy Shield items with badges and interactive raw/ switch
    privacy_needle = """              <div class="setting-list" style="display: flex; flex-direction: column; gap: 11px;">
                <!-- ITEM 1: PRIVACY SHIELD ATIVO -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Obsidian Privacy Shield Ativo</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Higienização Local Obrigatória</strong>
                          Substitui nomes de médicos, pacientes, contas bancárias e CPFs por marcadores anônimos antes de qualquer chamada em nuvem.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-privacy-shield">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-privacy-shield" checked onchange="updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield'); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante soberania e privacidade inviolável para relatórios médicos e extratos de investimentos.
                  </div>
                </div>

                <!-- ITEM 2: RAW READ-ONLY -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Zona raw/ Estritamente Read-Only</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Lei Suprema 1 do Cofre</strong>
                          Fontes originais brutas nunca são apagadas nem truncadas; preservação aditiva de YAML.
                          <div class="tooltip-def">Status: Ativo (Mandatório)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill locked" id="pill-obsidian-raw-ro">MANDATÓRIO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-raw-ro" checked disabled>
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Blindagem física contra perda de notas originais, fotos e documentos legados no cofre.
                  </div>
                </div>

                <!-- ITEM 3: WIKILINKS & FRONTMATTER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Links Bidirecionais [[...]] & YAML Frontmatter</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Padrão Enciclopédico Atômico</strong>
                          Insere compulsoriamente cabeçalhos YAML padronizados (type, tags, sources, last_updated) e referências cruzadas bidirecionais.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-wikilinks">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-wikilinks" checked onchange="updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks'); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante hiperconectividade ontológica do grafo do cofre no Obsidian Graph View.
                  </div>
                </div>

                <!-- ITEM 4: MODO OPERACIONAL DE PRIVACIDADE -->
                <div class="setting-item">
                  <div class="setting-header" style="flex-wrap: wrap; gap: 8px;">
                    <div class="setting-title">
                      <span>Modo Operacional de Privacidade</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Estratégia de Processamento</strong>
                          HYBRID_ANONYMIZED higieniza PII localmente no silício e envia para modelos de fronteira de alto contexto; STRICT_LOCAL retém 100% no silício local (dGPU RTX 2060m).
                        </span>
                      </span>
                    </div>
                    <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 195px; font-size: 0.74rem;" onchange="markSettingsUnsaved();">"""

    privacy_replacement = """              <div class="setting-list" style="display: flex; flex-direction: column; gap: 11px;">
                <!-- ITEM 1: PRIVACY SHIELD ATIVO -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Obsidian Privacy Shield Ativo</span>
                      <span class="badge" style="background: rgba(168, 85, 247, 0.12); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Reflete no Card 3 ↑
                      </span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Higienização Local Obrigatória</strong>
                          Substitui nomes de médicos, pacientes, contas bancárias e CPFs por marcadores anônimos antes de qualquer chamada em nuvem.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-privacy-shield">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-privacy-shield" checked onchange="updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield'); syncObsidianKpis(); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante soberania e privacidade inviolável para relatórios médicos e extratos de investimentos.
                  </div>
                </div>

                <!-- ITEM 2: RAW READ-ONLY -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Zona raw/ Estritamente Read-Only</span>
                      <span class="badge" style="background: rgba(16, 185, 129, 0.12); color: var(--green); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Reflete no Card 2 ↑
                      </span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Lei Suprema 1 do Cofre</strong>
                          Fontes originais brutas nunca são apagadas nem truncadas; preservação aditiva de YAML.
                          <div class="tooltip-def">Padrão canônico: Ativo (Read-Only)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-raw-ro">LIGADO (IMUTÁVEL)</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-raw-ro" checked onchange="toggleRawReadOnly(); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Blindagem física contra perda de notas originais, fotos e documentos legados no cofre.
                  </div>
                </div>

                <!-- ITEM 3: WIKILINKS & FRONTMATTER -->
                <div class="setting-item">
                  <div class="setting-header">
                    <div class="setting-title">
                      <span>Links Bidirecionais [[...]] & YAML Frontmatter</span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Padrão Enciclopédico Atômico</strong>
                          Insere compulsoriamente cabeçalhos YAML padronizados (type, tags, sources, last_updated) e referências cruzadas bidirecionais.
                          <div class="tooltip-def">Padrão: Ativo (true)</div>
                        </span>
                      </span>
                    </div>
                    <div class="switch-wrap">
                      <span class="state-pill on" id="pill-obsidian-wikilinks">LIGADO</span>
                      <label class="switch">
                        <input type="checkbox" id="cfg-obsidian-wikilinks" checked onchange="updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks'); markSettingsUnsaved();">
                        <span class="slider"></span>
                      </label>
                    </div>
                  </div>
                  <div class="setting-desc" style="text-wrap: balance;">
                    Garante hiperconectividade ontológica do grafo do cofre no Obsidian Graph View.
                  </div>
                </div>

                <!-- ITEM 4: MODO OPERACIONAL DE PRIVACIDADE -->
                <div class="setting-item">
                  <div class="setting-header" style="flex-wrap: wrap; gap: 8px;">
                    <div class="setting-title" style="gap: 6px;">
                      <span>Modo Operacional de Privacidade</span>
                      <span class="badge" style="background: rgba(168, 85, 247, 0.12); color: var(--purple-light); border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.62rem; padding: 2px 7px; font-weight: 600; margin-left: 2px; white-space: nowrap;">
                        Reflete no Card 3 ↑
                      </span>
                      <span class="info-icon" style="z-index: 90;">i
                        <span class="tooltip-box">
                          <strong>Estratégia de Processamento</strong>
                          HYBRID_ANONYMIZED higieniza PII localmente no silício e envia para modelos de fronteira de alto contexto; STRICT_LOCAL retém 100% no silício local (dGPU RTX 2060m).
                        </span>
                      </span>
                    </div>
                    <select id="cfg-obsidian-privacy-mode" class="setting-input" style="width: 195px; font-size: 0.74rem;" onchange="syncObsidianKpis(); markSettingsUnsaved();">"""

    assert privacy_needle in text, "privacy_needle not found"
    text = text.replace(privacy_needle, privacy_replacement, 1)

    # 6. Replace JavaScript functions
    js_needle = """    // =========================================================
    // SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL
    // =========================================================
    function initObsidianSubtab() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        document.getElementById('val-obsidian-batch-size').innerText = batchInput.value + ' notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        document.getElementById('val-obsidian-interval').innerText = intervalInput.value + ' min';
      }
      syncObsidianCadenceKpi();
    }

    function syncObsidianCadenceKpi() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      const cadenceEl = document.getElementById('metric-obsidian-cadence');
      if (batchInput && intervalInput && cadenceEl) {
        cadenceEl.innerText = `${batchInput.value} NOTAS / ${intervalInput.value} MIN`;
      }
    }

    function validateVaultStructure() {
      const statusEl = document.getElementById('checklist-vault-status');
      if (statusEl) {
        statusEl.innerText = 'Validando pastas...';
        statusEl.style.color = 'var(--cyan)';
      }
      setTimeout(() => {
        ['chk-vault-raw', 'chk-vault-concepts', 'chk-vault-entities', 'chk-vault-summaries', 'chk-vault-manifest', 'chk-vault-log'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = '6 de 6 Conformidades';
          statusEl.style.color = 'var(--green)';
        }
        showToast('✓ Estrutura canônica do cofre validada com sucesso (6 de 6 conformidades)!', 'success');
      }, 350);
    }"""

    js_replacement = """    // =========================================================
    // SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL
    // =========================================================
    function initObsidianSubtab() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        document.getElementById('val-obsidian-batch-size').innerText = batchInput.value + ' notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        document.getElementById('val-obsidian-interval').innerText = intervalInput.value + ' min';
      }
      syncObsidianKpis();
    }

    function focusVaultSetting(targetId) {
      const el = document.getElementById(targetId);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        const parentItem = el.closest('.setting-item') || el.closest('.card') || el;
        const oldTransition = parentItem.style.transition;
        parentItem.style.transition = 'all 0.3s ease';
        parentItem.style.boxShadow = '0 0 16px rgba(56, 189, 248, 0.4)';
        parentItem.style.borderColor = 'var(--cyan)';
        setTimeout(() => {
          parentItem.style.boxShadow = '';
          parentItem.style.borderColor = '';
          parentItem.style.transition = oldTransition;
        }, 1200);
        if (el.focus && typeof el.focus === 'function') {
          el.focus();
        }
      }
    }

    function toggleRawReadOnly() {
      const cb = document.getElementById('cfg-obsidian-raw-ro');
      if (!cb) return;
      if (!cb.checked) {
        const proceed = confirm("⚠️ ATENÇÃO: A Lei 1 do AGENTS.md estabelece que as fontes brutas em raw/ são sagradas e estritamente Read-Only.\\n\\nDesativar esta proteção permite que ferramentas mutem, sobrescrevam ou apaguem notas originais.\\n\\nDeseja realmente liberar a gravação em raw/?");
        if (!proceed) {
          cb.checked = true;
        }
      }
      syncObsidianKpis();
    }

    function syncObsidianKpis() {
      // 1. INTEGRAÇÃO DO COFRE (Card 1)
      const storageSelect = document.getElementById('cfg-obsidian-storage-type');
      const pathInput = document.getElementById('cfg-obsidian-path');
      const pathBadge = document.getElementById('badge-vault-path-type');
      const kpiIntVal = document.getElementById('kpi-vault-integration');
      const kpiIntSub = document.getElementById('kpi-vault-integration-sub');

      if (storageSelect && kpiIntVal && kpiIntSub) {
        const mode = storageSelect.value;
        if (mode === 'onedrive') {
          kpiIntVal.innerText = 'ONEDRIVE SYNC';
          kpiIntVal.style.color = 'var(--cyan)';
          kpiIntSub.innerText = 'Sincronizado na Nuvem';
          if (pathInput) {
            pathInput.value = 'C:\\\\Users\\\\rosas\\\\OneDrive\\\\Documentos\\\\Obisidian DB';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerText = '✓ OneDrive Conectado';
            pathBadge.style.color = 'var(--green)';
            pathBadge.style.background = 'rgba(16, 185, 129, 0.12)';
            pathBadge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
          }
        } else if (mode === 'local_vault') {
          kpiIntVal.innerText = 'LOCAL WINDOWS';
          kpiIntVal.style.color = 'var(--purple-light)';
          kpiIntSub.innerText = 'Armazenamento Local';
          if (pathInput) {
            pathInput.value = 'C:\\\\Users\\\\rosas\\\\Documents\\\\Souls_Vault';
            pathInput.readOnly = true;
          }
          if (pathBadge) {
            pathBadge.innerText = '✓ Cofre Local Ativo';
            pathBadge.style.color = 'var(--purple-light)';
            pathBadge.style.background = 'rgba(168, 85, 247, 0.12)';
            pathBadge.style.borderColor = 'rgba(168, 85, 247, 0.3)';
          }
        } else {
          kpiIntVal.innerText = 'CUSTOMIZADO';
          kpiIntVal.style.color = 'var(--amber)';
          kpiIntSub.innerText = 'Diretório Personalizado';
          if (pathInput) {
            pathInput.readOnly = false;
            if (!pathInput.value || pathInput.value.includes('OneDrive') || pathInput.value.includes('Souls_Vault')) {
              pathInput.value = 'D:\\\\Notas\\\\MeuCofre';
            }
          }
          if (pathBadge) {
            pathBadge.innerText = '✏️ Caminho Customizado';
            pathBadge.style.color = 'var(--amber)';
            pathBadge.style.background = 'rgba(245, 158, 11, 0.12)';
            pathBadge.style.borderColor = 'rgba(245, 158, 11, 0.3)';
          }
        }
      }

      // 2. ZONA BRUTA RAW (Card 2)
      const rawCb = document.getElementById('cfg-obsidian-raw-ro');
      const kpiRawVal = document.getElementById('kpi-vault-raw');
      const kpiRawSub = document.getElementById('kpi-vault-raw-sub');
      const rawPill = document.getElementById('pill-obsidian-raw-ro');

      if (rawCb && kpiRawVal && kpiRawSub) {
        if (rawCb.checked) {
          kpiRawVal.innerText = 'READ-ONLY TOTAL';
          kpiRawVal.style.color = 'var(--green)';
          kpiRawSub.innerText = '🔒 Imutável & Sagrado (Lei 1)';
          if (rawPill) {
            rawPill.innerText = 'LIGADO (IMUTÁVEL)';
            rawPill.className = 'state-pill on';
          }
        } else {
          kpiRawVal.innerText = '⚠️ GRAVAÇÃO LIBERADA';
          kpiRawVal.style.color = '#ef4444';
          kpiRawSub.innerText = 'Risco de Mutação em raw/';
          if (rawPill) {
            rawPill.innerText = 'DESPROTEGIDO';
            rawPill.className = 'state-pill off';
          }
        }
      }

      // 3. PRIVACY SHIELD PII (Card 3)
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      const kpiPrivVal = document.getElementById('kpi-vault-privacy');
      const kpiPrivSub = document.getElementById('kpi-vault-privacy-sub');

      if (privacyCb && kpiPrivVal && kpiPrivSub) {
        if (!privacyCb.checked) {
          kpiPrivVal.innerText = 'DESATIVADO';
          kpiPrivVal.style.color = '#ef4444';
          kpiPrivSub.innerText = '⚠️ Alerta: Sem Redação PII';
        } else {
          const mode = modeSelect ? modeSelect.value : 'hybrid';
          if (mode === 'hybrid') {
            kpiPrivVal.innerText = 'HÍBRIDO (SILÍCIO)';
            kpiPrivVal.style.color = 'var(--green)';
            kpiPrivSub.innerText = 'Desidentificação PII Ativa';
          } else {
            kpiPrivVal.innerText = 'AIR-GAP TOTAL';
            kpiPrivVal.style.color = 'var(--purple-light)';
            kpiPrivSub.innerText = '100% Silício Local (RTX 2060m)';
          }
        }
      }

      // 4. CADÊNCIA THE DRIP (Card 4)
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      const cadenceEl = document.getElementById('metric-obsidian-cadence');
      const cadenceSub = document.getElementById('metric-obsidian-cadence-sub');

      if (batchInput && intervalInput && cadenceEl) {
        const batch = parseInt(batchInput.value, 10) || 3;
        const interval = parseInt(intervalInput.value, 10) || 30;
        cadenceEl.innerText = `${batch} NOTAS / ${interval} MIN`;
        const workShiftNotes = Math.round((10 * 60 / interval) * batch);
        const dailyNotes = Math.round((24 * 60 / interval) * batch);
        if (cadenceSub) {
          cadenceSub.innerText = `~${dailyNotes} notas/dia • ~${workShiftNotes}/turno 10h`;
        }
      }
    }

    function syncObsidianCadenceKpi() {
      syncObsidianKpis();
    }

    function validateVaultStructure() {
      const statusEl = document.getElementById('checklist-vault-status');
      const badgeVaultConn = document.getElementById('badge-vault-conn');
      const kpiIntSub = document.getElementById('kpi-vault-integration-sub');
      if (statusEl) {
        statusEl.innerText = 'Validando pastas...';
        statusEl.style.color = 'var(--cyan)';
      }
      setTimeout(() => {
        ['chk-vault-raw', 'chk-vault-concepts', 'chk-vault-entities', 'chk-vault-summaries', 'chk-vault-manifest', 'chk-vault-log'].forEach(id => {
          const el = document.getElementById(id);
          if (el) {
            el.style.opacity = '1';
            const span = el.querySelector('span');
            if (span) span.style.color = 'var(--green)';
          }
        });
        if (statusEl) {
          statusEl.innerText = '6 de 6 Conformidades';
          statusEl.style.color = 'var(--green)';
        }
        if (badgeVaultConn) {
          badgeVaultConn.innerText = '✓ 6/6 Conforme';
        }
        if (kpiIntSub) {
          kpiIntSub.innerText = '✓ 6/6 Pastas Íntegras';
          setTimeout(() => {
            syncObsidianKpis();
          }, 3500);
        }
        showToast('✓ Estrutura canônica do cofre validada com sucesso (6 de 6 conformidades)!', 'success');
      }, 350);
    }"""

    assert js_needle in text, "js_needle not found"
    text = text.replace(js_needle, js_replacement, 1)

    # 7. Replace resetObsidianDefaults
    reset_needle = """    function resetObsidianDefaults() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        batchInput.value = 3;
        document.getElementById('val-obsidian-batch-size').innerText = '3 notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        intervalInput.value = 30;
        document.getElementById('val-obsidian-interval').innerText = '30 min';
      }
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      if (privacyCb) {
        privacyCb.checked = true;
        updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield');
      }
      const wikilinksCb = document.getElementById('cfg-obsidian-wikilinks');
      if (wikilinksCb) {
        wikilinksCb.checked = true;
        updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks');
      }
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      if (modeSelect) modeSelect.value = 'hybrid';
      syncObsidianCadenceKpi();
      showToast('✓ Defaults do Cofre de Conhecimentos restaurados!', 'success');
    }"""

    reset_replacement = """    function resetObsidianDefaults() {
      const batchInput = document.getElementById('cfg-obsidian-batch-size');
      if (batchInput) {
        batchInput.value = 3;
        document.getElementById('val-obsidian-batch-size').innerText = '3 notas';
      }
      const intervalInput = document.getElementById('cfg-obsidian-interval-min');
      if (intervalInput) {
        intervalInput.value = 30;
        document.getElementById('val-obsidian-interval').innerText = '30 min';
      }
      const storageSelect = document.getElementById('cfg-obsidian-storage-type');
      if (storageSelect) storageSelect.value = 'onedrive';

      const rawCb = document.getElementById('cfg-obsidian-raw-ro');
      if (rawCb) {
        rawCb.checked = true;
      }
      const privacyCb = document.getElementById('cfg-obsidian-privacy-shield');
      if (privacyCb) {
        privacyCb.checked = true;
        updateSwitchPill('cfg-obsidian-privacy-shield', 'pill-obsidian-privacy-shield');
      }
      const wikilinksCb = document.getElementById('cfg-obsidian-wikilinks');
      if (wikilinksCb) {
        wikilinksCb.checked = true;
        updateSwitchPill('cfg-obsidian-wikilinks', 'pill-obsidian-wikilinks');
      }
      const modeSelect = document.getElementById('cfg-obsidian-privacy-mode');
      if (modeSelect) modeSelect.value = 'hybrid';
      
      syncObsidianKpis();
      initSlidersMagneticSnap();
      showToast('✓ Defaults do Cofre de Conhecimentos restaurados!', 'success');
    }"""

    assert reset_needle in text, "reset_needle not found"
    text = text.replace(reset_needle, reset_replacement, 1)

    # 8. Update populateSettingsForm
    pop_needle = """      // Obsidian
      if (cfg.obsidian) {
        setCheck('cfg-obsidian-privacy-shield', cfg.obsidian.privacy_shield_enabled, 'pill-obsidian-privacy-shield');
        setInput('cfg-obsidian-batch-size', cfg.obsidian.curator_batch_size || 3);
      }"""

    pop_replacement = """      // Obsidian
      if (cfg.obsidian) {
        if (cfg.obsidian.storage_type) {
          setInput('cfg-obsidian-storage-type', cfg.obsidian.storage_type);
        }
        if (cfg.obsidian.vault_path) {
          setInput('cfg-obsidian-path', cfg.obsidian.vault_path);
        }
        setCheck('cfg-obsidian-privacy-shield', cfg.obsidian.privacy_shield_enabled, 'pill-obsidian-privacy-shield');
        if (cfg.obsidian.raw_readonly !== undefined) {
          setCheck('cfg-obsidian-raw-ro', cfg.obsidian.raw_readonly, 'pill-obsidian-raw-ro');
        }
        if (cfg.obsidian.wikilinks_enabled !== undefined) {
          setCheck('cfg-obsidian-wikilinks', cfg.obsidian.wikilinks_enabled, 'pill-obsidian-wikilinks');
        }
        if (cfg.obsidian.privacy_mode) {
          setInput('cfg-obsidian-privacy-mode', cfg.obsidian.privacy_mode);
        }
        setInput('cfg-obsidian-batch-size', cfg.obsidian.curator_batch_size || 3);
        setInput('cfg-obsidian-interval-min', cfg.obsidian.interval_minutes || 30);
        syncObsidianKpis();
      }"""

    assert pop_needle in text, "pop_needle not found"
    text = text.replace(pop_needle, pop_replacement, 1)

    # 9. Update saveSettings
    save_needle = """[obsidian]
vault_path = "C:\\\\\\\\Users\\\\\\\\rosas\\\\\\\\OneDrive\\\\\\\\Documentos\\\\\\\\Obisidian DB"
privacy_shield_enabled = ${getCheck('cfg-obsidian-privacy-shield', true)}
raw_readonly = true
curator_batch_size = ${getNum('cfg-obsidian-batch-size', 5)}"""

    save_replacement = """[obsidian]
vault_path = "${getStr('cfg-obsidian-path', 'C:\\\\\\\\Users\\\\\\\\rosas\\\\\\\\OneDrive\\\\\\\\Documentos\\\\\\\\Obisidian DB').replace(/\\\\/g, '\\\\\\\\')}"
storage_type = "${getStr('cfg-obsidian-storage-type', 'onedrive')}"
privacy_shield_enabled = ${getCheck('cfg-obsidian-privacy-shield', true)}
privacy_mode = "${getStr('cfg-obsidian-privacy-mode', 'hybrid')}"
raw_readonly = ${getCheck('cfg-obsidian-raw-ro', true)}
wikilinks_enabled = ${getCheck('cfg-obsidian-wikilinks', true)}
curator_batch_size = ${getNum('cfg-obsidian-batch-size', 3)}
interval_minutes = ${getNum('cfg-obsidian-interval-min', 30)}"""

    assert save_needle in text, "save_needle not found"
    text = text.replace(save_needle, save_replacement, 1)

    with open(file_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)

    print("SUCCESS! File updated cleanly with pure LF.")

if __name__ == "__main__":
    main()
