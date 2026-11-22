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

---

## 7. POST-MORTEM & APRENDIZADOS: SMOKE TEST AO VIVO DO MINIONS, SUBPROCESSOS ZUMBIS NO WINDOWS E PULSO MULTI-CADÊNCIA (v0.9.2.1)

### 7.1 Contexto & Sintomas Identificados
Durante o teste ao vivo (*live smoke test*) de `POST /api/v1/google/analyze/minions` no daemon real `souls_engine.exe` (`127.0.0.1:9123`) sobre um documento real de ~33 KB:
1. **Travamento >60s e Processo Zumbi (`PID 24532`):** O modo `"auto"` estourou timeout de cliente HTTP sem encerrar o filho `llama-completion.exe`, deixando um processo órfão com `Threads: {}` ativo no Windows.
2. **Economia de Tokens `0%` no Modo Determinístico:** O modo `"fast_deterministic"` retornou `savings_pct = 0.0` porque todo o corpus de 33 KB foi tratado como um único chunk.
3. **Condição de Corrida em Testes Paralelos:** `cargo test -p souls_server` apresentou flakiness pontual em testes que mutam `SOULS_HERMES_CONFIG_PATH` concorrentemente.

### 7.2 Causa-Raiz Técnica
1. **Subprocesso Bloqueante Sem Drenagem nem Kill:** `Tier1GenerativeEngine::execute_model_pass` (`souls_inference_runtime`) usava `cmd.output()` sem timeout nem limpeza do processo filho.
2. **Seleção Alfabética de GGUF + Contenção WDDM:** `resolve_local_minions_model()` selecionava o primeiro `.gguf` alfabeticamente (`Arch-Function-Chat-3B-Q5_K_M.gguf`) e o despachava com `-ngl 33` na RTX 2060m sob WDDM, travando a inicialização CUDA; além disso, modelos pequenos podem ecoar placeholders de gramática GBNF (`"string"`).
3. **Quebra Ingênua de Parágrafos:** `chunk_document` (`souls_core::research::chunker`) dividia exclusivamente por `"\n\n"`, falhando em textos densos separados apenas por `\n` simples.

### 7.3 Solução Implementada & Evidências Verificadas no Host Real
1. **Subprocessos Acotados e Imunes a Deadlock (commit `77d5fc5`):** `llama-completion.exe` passou a ser iniciado com `Stdio::null()` em `stdin`, thread OS dedicada drenando `stdout`, polling `try_wait()` sob `SOULS_LLAMA_TIMEOUT_SECS` e `child.kill()` + `child.wait()` obrigatórios no vencimento.
2. **Worker SLMs Compactos em CPU AVX2 (`0 MB VRAM`) (commit `7e65cb1`):** Filtro explícito de `mmproj`/`vl`/`vision`/`hy-mt`/`embed`/`rerank`, ranqueamento menor-primeiro na faixa `250 MB–1,8 GB` e execução em `CpuAvx2` (`-ngl 0`). Benchmark real no Intel i9-9880H com `Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf`: carga em `170 ms`, geração a `50,5 tok/s`, análise MinionS completa em `8.596 ms` com `0 MB VRAM` e proteção contra eco GBNF (`"string"`).
3. **Janela Deslizante para Corpus Denso (commit `090ba13`):** Subdivisão automática de blocos `>350` palavras em janelas de `300` palavras (`overlap` de `50`). No mesmo documento de 33 KB, o resultado saltou de `1` chunk / `0%` para `17` chunks / `85,9%` de economia em `5 ms` (e cache SHA-256 em `<1 ms`).
4. **Arquitetura de Pulso Multi-Cadência & SemVer 2.0 (commits `ce122d6`, `b41ef7b`, `193fcb0`):** Padronização de heartbeat base de 5 min com multiplicadores (`1x`/`2x`/`3x`), *backoff* isolado por monitor, **Quiet Hours configurável e desligável**, auditoria pré-spec de APIs externas (evitando Jules descontinuado e bloqueando Keep/Chat em `@gmail.com`) e versão SemVer 2.0 `0.9.2+1` no Cargo.

---

## 8. POST-MORTEM & APRENDIZADOS: VALIDAÇÃO ESTÁTICA EM CLIENTES MCP ESTRITOS (`tool_dispatch`), EXTIRPAÇÃO DE MOCKS EM INFERÊNCIA E RESOLUÇÃO DINÂMICA DO LLAMA.CPP (v0.9.2+3)

### 8.1 Contexto & Sintomas Identificados
1. **Bloqueio de Ferramentas Nível 1 no Cliente Hermes Agent:** O runtime Python do Hermes Agent valida o nome da tool a ser chamada contra a lista recebida estaticamente no handshake inicial do protocolo MCP. Como apenas 7 tools estavam no Boot Level 0 (para poupar contexto), qualquer tentativa do Hermes de chamar diretamente ferramentas descobertas via `tool_search` (ex: `google_gmail`, `semantic_search`) era barrada prematuramente no validador do cliente antes de atingir o servidor Axum.
2. **Resíduo de Mock Sintético em `/v1/chat/completions`:** Durante auditoria de chamadas locais, identificou-se que o gateway de completions ainda continha uma string de teste sintética (`"[LLAMA UPSTREAM GGML] Response generated with KV Cache..."`) retornada quando o binário local não respondia a tempo ou em caminhos de fallback.
3. **Múltiplas Versões de `llama.cpp` e Resolução Rígida:** O Hermes Agent baixa ferramentas em `%LOCALAPPDATA%\hermes\tools\` gerando múltiplas pastas (`llamacpp-cuda-10362-win32-x64`, `llamacpp-cuda-10679-win32-x64`, `llamacpp-cuda-11370-win32-x64`). O Souls Engine apontava para caminhos rígidos ou incompletos, correndo o risco de usar binários desatualizados ou não encontrar `llama-server.exe`.

### 8.2 Causa-Raiz Técnica
1. **Falta de Primitiva de Despacho Genérico no Nível 0:** A revelação progressiva funcionava perfeitamente para servidores e clientes com invocação dinâmica pura (ex: via HTTP REST ou MCP com validação relaxada), mas quebrava clientes estritos de IA que usam enums fechados baseados no primeiro `tools/list`.
2. **Timeout Curto (20s) vs Carga Fria CUDA em Mobile dGPU:** Em notebooks com GPU móvel (RTX 2060m), cold boot de modelos GGUF de 3-4 GB via CLI leva de 25 a 30 segundos. O timeout antigo de 20s forçava o kill do processo e ativava o fallback de mock que havia sido deixado no protótipo.
3. **Ausência de Scanner de Builds Semânticos:** Não havia função para escanear a pasta de tools do Hermes, ordenar as builds numericamente por sufixo e selecionar sempre a versão mais recente e estável com suporte CUDA.

### 8.3 Solução Implementada & Regras Canônicas
1. **Meta-Ferramenta `tool_dispatch` / `dispatch` no Boot Level 0 (commit `v0.9.2+3`):** Exposição permanente no handshake MCP inicial (elevando o Boot Level 0 para 8 ferramentas canônicas). Clientes estritos como o Hermes Agent agora executam chamadas como `tool_dispatch(tool="google_gmail", arguments={...})` sem gastar ~20k tokens de esquemas estáticos, com trava de recursão `-32602` e contenção ReFS.
2. **Extirpação Absoluta de Mocks em Produção (`AGENTS.md` Linha Zero):** Remoção total de respostas simuladas no caminho de geração de release. Fixtures de teste foram isoladas via detecção estrita de ambiente de testes (`SOULS_TEST_MODE` / caminhos `target\debug\deps\`). Em produção, a inferência é 100% física no silício (RTX 2060m / CPU AVX2) ou encaminhada ao `llama-server` upstream na porta 8080.
3. **Resolução Dinâmica da Build Mais Recente de `llama.cpp`:** Implementação de `resolve_latest_llamacpp_binary(bin_name)` em `souls_inference_runtime`, parseando os números de build como inteiros (`11370` > `10679` > `10362`) e selecionando dinamicamente os binários mais modernos (`llama-server.exe`, `llama-completion.exe`, `llama-cli.exe`, `llama-bench.exe`).
4. **Verificação Real em Hardware RTX 2060m:** Teste ao vivo de `/v1/chat/completions` executado com sucesso sobre `tier1_qwen_coder_gpu` (`Qwen2.5-Coder-3B-Instruct`), gerando inferência real via CUDA com saída confirmada no daemon de produção.
