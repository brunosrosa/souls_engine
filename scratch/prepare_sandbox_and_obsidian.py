# -*- coding: utf-8 -*-
import sys

def main():
    # 1. Prepare subtab-sandbox HTML
    with open('scratch/subtab_sandbox_saved.html', 'r', encoding='utf-8') as f:
        sandbox_html = f.read()

    # Apply Card 1 header refinement (Point #1 from Request 6)
    old_c1 = '''            <!-- CARD 1: SETUP & PROVISIONAMENTO DO ESPAÇO DE DADOS -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px; justify-content: space-between;">
                <div class="card-title" style="font-size: 0.86rem;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem;">📦</div>
                  Setup & Provisionamento do Espaço Local
                  <span class="info-icon" style="z-index: 90;">i
                    <span class="tooltip-box">
                      <strong>Provisionamento e Nascimento do Espaço</strong>
                      Gerencia a criação e integridade das 6 subpastas essenciais de persistência do Souls Engine. Em instalações novas ou manutenção, cria e valida todos os diretórios com 1 clique.
                    </span>
                  </span>
                </div>
                <span class="badge badge-champion" style="font-size: 0.68rem;">ReFS CoW Ativo</span>
              </div>'''

    new_c1 = '''            <!-- CARD 1: PROVISIONAMENTO DO ESPAÇO DE DADOS -->
            <div class="card" style="margin-bottom: 0;">
              <div class="card-header" style="padding-bottom: 8px; display: flex; justify-content: space-between; align-items: center; gap: 16px;">
                <div class="card-title" style="font-size: 0.86rem; display: flex; align-items: center; gap: 8px; margin: 0;">
                  <div class="card-icon icon-cyan" style="width: 24px; height: 24px; font-size: 0.88rem; flex-shrink: 0;">📦</div>
                  <span>Provisionamento do Espaço Local</span>
                  <span class="info-icon" style="z-index: 90; margin-left: 2px; flex-shrink: 0;">i
                    <span class="tooltip-box">
                      <strong>Provisionamento e Nascimento do Espaço</strong>
                      Gerencia a criação e integridade das 6 subpastas essenciais de persistência do Souls Engine. Em instalações novas ou manutenção, cria e valida todos os diretórios com 1 clique.
                    </span>
                  </span>
                </div>
                <span class="badge badge-champion" id="badge-storage-fs" style="font-size: 0.68rem; flex-shrink: 0; white-space: nowrap; margin-left: 12px;">ReFS CoW Ativo</span>
              </div>'''

    if old_c1 in sandbox_html:
        sandbox_html = sandbox_html.replace(old_c1, new_c1)
        print("Updated Card 1 header in sandbox!")

    # Dynamic Banner (Point #2 from Request 6)
    old_banner = '''        <!-- BANNER DE GOVERNANÇA & ISOLAMENTO DE DADOS -->
        <div style="background: rgba(16, 185, 129, 0.05); border: 1px solid rgba(16, 185, 129, 0.25); border-radius: var(--radius-md); padding: 10px 14px; margin-bottom: 16px; display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
          <div style="font-size: 0.74rem; line-height: 1.5; color: #e2e8f0; flex: 1; min-width: 280px; text-wrap: balance;">
            <strong style="color: var(--green);">💡 Governança de Silício & Isolamento de Dados:</strong> O Souls Engine aplica a Linha Vermelha 4 do AGENTS.md, garantindo que agentes e ferramentas operem com contenção estrita no diretório autorizado. No ambiente atual de workstation, opera sobre Dev Drive ReFS com Copy-on-Write (CoW); em instalações padrão de usuário, utiliza o diretório local isolado (%LOCALAPPDATA%\\SoulsEngine\\data). Modificações de código usam Myers Diff com validação atômica e o shell conta com timeout anti-congelamento.
          </div>
          <span style="font-size: 0.68rem; font-family: var(--font-mono); color: var(--green); background: rgba(16, 185, 129, 0.12); border: 1px solid rgba(16, 185, 129, 0.3); padding: 3px 8px; border-radius: 4px; white-space: nowrap; align-self: center;">
            • Confinamento Ativo (Linha Vermelha 4)
          </span>
        </div>'''

    new_banner = '''        <!-- BANNER DINÂMICO DE GOVERNANÇA & ISOLAMENTO DE DADOS (Ponto #2) -->
        <div style="background: rgba(16, 185, 129, 0.05); border: 1px solid rgba(16, 185, 129, 0.25); border-radius: var(--radius-md); padding: 10px 14px; margin-bottom: 16px; display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
          <div id="banner-storage-desc" style="font-size: 0.74rem; line-height: 1.5; color: #e2e8f0; flex: 1; min-width: 280px; text-wrap: balance;">
            <strong style="color: var(--green);">💡 Modo Workstation (Dev Drive ReFS):</strong> Operando sobre volume dedicado ReFS com aceleração Copy-on-Write (CoW) em <code>Z:\\souls_engine\\.souls_data\\</code>. Taxas de I/O de até 4.5 GB/s para operações em lote nos índices SQLite WAL e LanceDB, com contenção absoluta (Linha Vermelha 4) e zero poluição no sistema operacional.
          </div>
          <span id="banner-storage-badge" style="font-size: 0.68rem; font-family: var(--font-mono); color: var(--green); background: rgba(16, 185, 129, 0.12); border: 1px solid rgba(16, 185, 129, 0.3); padding: 3px 8px; border-radius: 4px; white-space: nowrap; align-self: center;">
            • Dev Drive ReFS Ativo (Linha Vermelha 4)
          </span>
        </div>'''

    if old_banner in sandbox_html:
        sandbox_html = sandbox_html.replace(old_banner, new_banner)
        print("Updated Banner in sandbox!")

    # 2. Read dashboard_ui.html
    dash_path = 'crates/souls_server/src/dashboard_ui.html'
    with open(dash_path, 'r', encoding='utf-8') as f:
        dash_content = f.read()

    # Find where the messed up subtab-sandbox starts
    sub6_marker = '<!-- SUB-ABA 6: SEGURANÇA, ISOLAMENTO & ESPAÇO DE TRABALHO -->'
    start_pos = dash_content.find(sub6_marker)
    assert start_pos != -1, "sub6_marker not found"

    # Find where subtab-raw starts
    raw_marker = 'id="subtab-raw"'
    raw_pos = dash_content.find(raw_marker)
    assert raw_pos != -1, "raw_marker not found"

    # The comment before subtab-raw
    end_pos = dash_content.rfind('<!-- ==================================================== -->', 0, raw_pos)
    assert end_pos != -1, "end comment before subtab-raw not found"

    current_block = dash_content[start_pos:end_pos]

    # Find where the 4 obsidian cards start in current_block
    obsidian_cards_marker = '<!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico — 100% Reativos & Colorizados por Subsistema) -->'
    obs_idx = current_block.find(obsidian_cards_marker)
    assert obs_idx != -1, "obsidian_cards_marker not found"

    obsidian_body = current_block[obs_idx:].rstrip()
    # If it ends with </div>, ensure it ends properly for subtab-obsidian
    if not obsidian_body.endswith('</div>'):
        obsidian_body += '\n      </div>'

    obsidian_subtab_html = f'''<!-- ==================================================== -->
      <!-- SUB-ABA 7: COFRE DE CONHECIMENTOS & CURADORIA PESSOAL -->
      <!-- ==================================================== -->
      <div class="settings-subpane" id="subtab-obsidian">
        <!-- HEADER DA SUB-ABA COM AÇÕES -->
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
          <div>
            <h2 style="font-size: 1.15rem; font-weight: 700; color: #fff; display: flex; align-items: center; gap: 8px; margin: 0 0 4px 0;">
              <span>📓</span>
              <span>Cofre de Conhecimentos & Curadoria Pessoal</span>
              <span class="info-icon" style="z-index: 95;">i
                <span class="tooltip-box">
                  <strong>Curadoria Enciclopédica & Soberania Pessoal</strong>
                  Processa fontes brutas imutáveis em <code>raw/</code> e destila conceitos e entidades atômicas em <code>wiki/</code> com links bidirecionais <code>[[...]]</code>, aplicando desidentificação local de PII no silício.
                </span>
              </span>
            </h2>
            <p style="font-size: 0.78rem; color: var(--text-muted); margin: 0;">
              Destilação contínua de anotações brutas (raw/) em conhecimento enciclopédico hiperconectado (wiki/), com blindagem local de dados sensíveis (Privacy Shield).
            </p>
          </div>
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-secondary" onclick="resetObsidianDefaults()" style="padding: 6px 12px; font-size: 0.75rem;">
              <span>⚡</span> Defaults do Cofre
            </button>
            <button class="btn btn-green" onclick="saveSettings()" style="padding: 6px 14px; font-size: 0.75rem;">
              <span>💾</span> Salvar Configurações
            </button>
          </div>
        </div>

        {obsidian_body}
'''

    # Assemble combined block: subtab-sandbox + subtab-obsidian
    combined_block = sandbox_html.strip() + '\n\n      ' + obsidian_subtab_html.strip() + '\n\n      '

    dash_new = dash_content[:start_pos] + combined_block + dash_content[end_pos:]

    # Ensure 100% LF
    dash_new = dash_new.replace('\r\n', '\n')

    with open(dash_path, 'w', encoding='utf-8') as f:
        f.write(dash_new)

    print("SUCCESS! Both subtab-sandbox and subtab-obsidian assembled cleanly!")

if __name__ == '__main__':
    main()

