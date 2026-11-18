# LESSONS_LEARNED_AND_POSTMORTEM.md — Caderno de Lições Aprendidas, Anti-Patterns & Post-Mortems (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (DOCS-AS-GUARDRAILS)  
> **DATA DE CONSOLIDAÇÃO:** 2026-10-12  
> **DISCIPLINA OPERACIONAL:** Agent Self-Learning Loop (`/learn`)

---

## 1. POST-MORTEM: A CORRUPÇÃO DO `config.yaml` DO HERMES AGENT

### 1.1 Contexto & Sintoma
Ao executar o comando `hermes update`, o utilitário acusava a seguinte quebra sintática:
```text
Your settings file (C:\Users\rosas\AppData\Local\hermes\config.yaml) has a formatting error at line 8.
Details: expected <block end>, but found '<block mapping start>'
```

### 1.2 Causa-Raiz Técnica (Root Cause Analysis)
O manipulador de presets do Web Dashboard (`crates/souls_server/src/routes/dashboard.rs`) utilizava substituição ingênua de linhas baseada em casamento de prefixos:
```rust
// ANTI-PATTERN: Substituição de texto cega em arquivo YAML hierárquico
if line.trim_start().starts_with("model:") {
    new_lines.push(format!("model: \"{}\"", target_model));
}
```
No arquivo `config.yaml` do Hermes Agent, a chave `model:` de nível superior não é um escalar puro, mas o início de um bloco mapeado com campos aninhados:
```yaml
model:
  default: meituan/longcat-2.5-preview:free
  provider: nous
  base_url: 'https://inference-api.nousresearch.com/v1'
```
Ao substituir a linha `model:` por `model: "deepseek/deepseek-chat"`, a linha seguinte (`  default: ...`) tornou-se um filho mapeado sob um valor escalar. Em conformidade com a especificação formal do YAML, isso gera erro imediato de gramática (`expected <block end>, but found '<block mapping start>'`).

### 1.3 Solução Implementada & Regra Canônica
1. **Parser Consciente de Contexto:** A rota foi refatorada no commit `e3269f8` para rastrear o estado hierárquico (`inside_top_model`) e atualizar **exclusivamente** o campo `  default:` sob o bloco `model:` raiz, preservando todas as demais chaves e indentação.
2. **Lei Negativa:** É **estritamente proibido** aplicar regex ou `starts_with` em arquivos de configuração YAML/TOML sem parsing estruturado ou rastreamento explícito de blocos.

---

## 2. POST-MORTEM: CONFLITO DE PORTA TCP (WSAEADDRINUSE) E PROCESSOS ZUMBIS

### 2.1 Contexto & Sintoma
Ao tentar abrir uma segunda instância do `souls_server.exe` ou `souls_engine.exe` enquanto outro processo já ocupava a porta `127.0.0.1:9123`, o `TcpListener::bind` retornava erro fatal de sistema operacional (`os error 10048`), causando saída abrupta com `ExitCode(1)`.

### 2.2 Solução Bare-Metal: Single-Instance Probe & PID Tracking
No commit `1163afb`:
1. **Sonda de Saúde HTTP Imediata:** Diante do erro de bind, o processo emite um ping assíncrono para `http://127.0.0.1:9123/health` (timeout de 800ms).
   - Se responder **200 OK Saudável**: O processo reconhece que o Souls Engine já está operando normalmente, exibe mensagem clara ao usuário e encerra a execução redundante com **código 0 (sucesso limpo)**, evitando janelas de erro no Windows.
   - Se não responder (processo travado/órfão): O processo emite alerta preciso informando que a porta está ocupada por um processo zumbi e instrui o uso de `stop_souls.bat`.
2. **Rastreamento de Ciclo de Vida via PID:** Gravação determinística de `souls_server.pid` no diretório `.souls_data` durante a inicialização e exclusão atômica no graceful shutdown.
3. **Purga em Lote:** O script `stop_souls.bat` finaliza concorrentemente instâncias de `souls_server` e `souls_engine`.

---

## 3. APRENDIZADO: A DUPLA IDENTIDADE DE BINÁRIOS NATIVOS NO RUST CARGO

### 3.1 Contexto
O usuário solicitou um executável com a identidade direta `souls_engine.exe`, enquanto a crate executável canônica do workspace foi nomeada historicamente como `souls_server` (em aderência à Linha Vermelha 1 do `AGENTS.md` como daemon headless).

### 3.2 Solução de Engenharia & Build Release
1. Duplicação de targets `[[bin]]` com o mesmo `src/main.rs` em um único `Cargo.toml` gera warning de manifesto no compilador (`file found to be present in multiple build targets`).
2. Para manter **0 warnings no Cargo**, preservamos o binário canônico `souls_server` e construímos a esteira de publicação em `tools/build_souls_release.ps1`, que compila em modo `--release` otimizado (MSVC x86_64, AVX2, LTO) e gera simultaneamente:
   - `target\release\souls_server.exe` (34.9 MB)
   - `target\release\souls_engine.exe` (34.9 MB)
3. O launcher de bandeja (`tools/souls_launcher.ps1`) e o script `start_souls.bat` foram programados para priorizar o executável de release `souls_engine.exe` caso exista, com fallback transparente para o modo debug.

---

## 4. APRENDIZADO: RESTRIÇÕES DE PRIVILÉGIOS NO `hermes update` SOB WINDOWS 11

### 4.1 Contexto
Ao rodar `hermes update`, pacotes binários dentro de `%LOCALAPPDATA%\hermes\tools\` (como `ffmpeg-9.0.1-win32-x64`) podem falhar ao serem renomeados para `.previous-*` com `[WinError 5] Acesso negado`.

### 4.2 Causa & Procedimento
1. O Windows 11 bloqueia renomeações atômicas de pastas se houver processos em segundo plano com handles de DLL ou arquivo abertos (ex: áudio do Hermes ou terminal em execução).
2. O comando `hermes update` deve ser executado obrigatoriamente a partir de um terminal **PowerShell com privilégios de Administrador**, com os gateways do Hermes previamente pausados (`hermes gateway stop`).

---

## 5. POST-MORTEM: CLASSIFICAÇÃO DE PARÂMETROS, ARQUITETURAS MoE E ANCORAGEM FLEXBOX

### 5.1 Contexto & Sintoma (Sessão Model Hub)
1. **Falso Positivo em Modelos MoE:** Um modelo com identificador `35B-A3B` foi incorretamente rotulado como `⚡ Sweet Spot dGPU (100% VRAM)` no explorador do Hugging Face.
2. **Desalinhamento Vertical de Cards Flexbox:** A introdução do badge `✓ INSTALADO` no canto superior direito empurrou os elementos do lado esquerdo para baixo em 12px.
3. **Equívoco de Teto de KV Cache:** Exibição inicial de 32k tokens de contexto sem considerar que modelos 7B em Q4_K_M deixam apenas ~920 MB de VRAM livre, estourando a VRAM em KV cache FP16.

### 5.2 Causa-Raiz Técnica
1. **Parsing por Substring Ingênua:** O teste utilizava `.includes('3b')`, que casava com o `"3b"` do sufixo de parâmetros ativos `"a3b"` dentro de `"35b-a3b"`, ignorando que o modelo tem 35 bilhões de parâmetros totais em disco (~20 GB).
2. **Quebra de Fluxo em Contêiner Flex:** O elemento wrapper `<span id="curated-badge-installed-...">` permaneceu como filho inline no fluxo de um container flex (`flex-direction: column; gap: 12px;`), ativando o espaçamento vertical mesmo com o badge interno posicionado como absoluto.
3. **Cálculo Teórico Desacoplado da dGPU:** Em modelos de 7B/8B, o KV cache FP16 em 32k demanda ~1.88 GB. Sem quantização de KV cache (`Q4_0`), o spillover PCIe colapsa a inferência.

### 5.3 Solução Implementada & Regras Canônicas
1. **Parser Regex com Fronteiras e Detecção MoE:**
   - Detecção explícita de notações MoE `(\d+)b-a(\d+)b` e `(\d+)x(\d+)b`. Modelos MoE como `35B-A3B` são atribuídos ao **Tier 2 (CPU Host AVX2 / 32 GB RAM)** com badge de aviso.
   - Modelos densos usam regex estrito `\b(\d+)b\b`. Apenas modelos $\le 4\text{B}$ recebem o selo `Sweet Spot dGPU`.
2. **Âncoras Absolutas Fora de Fluxo:** Todo elemento de canto superior em card flexbox deve residir em um container com `position: absolute; pointer-events: none;` (`.installed-badge-anchor`), garantindo impacto zero no alinhamento vertical dos filhos do flex.
3. **Partição Explícita de VRAM:** A telemetria física expõe explicitamente `Teto Pesos: ~3.8 GB (Q4_K_M)` e `Margem KV Cache: ~1.3 GB`, com teto seguro para 7B calibrado em `16.384 tokens (até 32k c/ KV Q4)`.

---

## 6. POST-MORTEM & APRENDIZADOS: AUDITORIA CROSS-TAB, PARIDADE ADR-009 E CURADORIA DINÂMICA (v0.9.1)

### 6.1 Contexto & Sintomas Identificados
1. **Desalinhamento de Perfis Cognitivos na Cloud Arena e Settings:** A Cloud Arena expunha apenas 6 perfis estáticos enquanto a governança ADR-009 exige 9 perfis (`executive`, `coding`, `agentic`, `reasoning`, `fast`, `design`, `research`, `security`, `curation`), além de perfis built-in em `souls_server::profiles` usando IDs legados (`web_researcher`, `memory_curator`, `security_auditor`) que falhavam quando o catálogo era reidratado parcialmente.
2. **Estagnação de Curadoria e Falsos "Instalado" no Model Hub:** "O Trio Recomendado" apenas apontava modelos já baixados sem sugerir o próximo upgrade compatível; "Melhores Escolhas por Função" dependia de metadados estáticos sem fusão com novidades ao vivo do Hugging Face; e a marcação `Instalado` verificava apenas a família/repositório, exibindo `⚡ Usar` mesmo para quantizações (`Q6_K`, `Q8_0`) que não estavam em disco.
3. **KPIs Fictícios e Stubs em Abas Secundárias:** O Cockpit Pulse e algumas sub-abas de Settings exibiam métricas estáticas (`$14.20 USD`, `142 fatos`, `88.4%`) ou botões simuladores desconectados da telemetria bare-metal real, além de controles sem feedback tátil de sincronização (`Atualizar`).

### 6.2 Causa-Raiz Técnica
1. **Identificadores Divergentes entre Crates & Frontend:** Ausência de contrato único de IDs dos 9 perfis entre `souls_llm_cloud_arena`, `souls_server::profiles` e o router matrix da UI.
2. **Casamento Frouxo de Artefatos GGUF:** O estado `is_installed` comparava nomes de modelo/família em vez do arquivo GGUF e da quantização exata selecionada no dropdown; as estimativas de throughput usavam constantes fixas em vez de cálculo físico por largura de banda de memória.
3. **Resquícios de Protótipo na Camada de Telemetria:** Handlers de resumo retornavam literais de fallback mesmo quando SQLite STRICT WAL, LadybugDB, Obsidian e NVML já forneciam contagens reais.

### 6.3 Solução Implementada & Regras Canônicas
1. **Paridade Estrita dos 9 Perfis ADR-009:** Unificação canônica de `executive`, `coding`, `agentic`, `reasoning`, `fast`, `design`, `research`, `security` e `curation` em todo o stack (`souls_llm_cloud_arena`, `souls_server::profiles`, Router Matrix e filtros da Cloud Arena), com ranking multiobjetivo dinâmico na Fronteira de Pareto e reidratação individual de perfis faltantes.
2. **Model Hub Evolutivo, Híbrido & Físico:** Implementação do Trio Recomendado em **Modo Evolutivo** (campeão local + próximo upgrade compatível com o teto VRAM da GPU), catálogo híbrido (curados + HF live/trending), detecção estrita `is_installed` por variante de quantização (alternando dinamicamente `⚡ Usar` vs `⬇ Baixar`), estimativa física de TPS por largura de banda e Explorer HF unificado com autocomplete (`250 ms` debounce).
3. **Telemetria 100% Real & Feedback Visual Zero-Atrito:** Substituição integral dos stubs em `/api/v1/system/pulse` e Settings por métricas auditáveis em 4 pilares (SQLite STRICT, LadybugDB, Obsidian e NVML), com formatação concisa de hardware (`Estável (XX°C • X.X/X.X GB)`) e trava visual anti-duplo clique com pulso de confirmação no botão `Atualizar`.


