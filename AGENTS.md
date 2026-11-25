# AGENTS.md — Leis Operacionais, Guardrails e Governança da Fábrica (v7 Canônica)

> **AVISO A TODOS OS AGENTES DE IA:**
> Este documento é o alicerce operacional e a lei suprema deste repositório. Ele define os limites físicos, as fronteiras arquiteturais e a disciplina de trabalho inegociável para o desenvolvimento do **Souls Engine v7**.
> O descumprimento de qualquer uma das cláusulas abaixo causará **rejeição imediata de código**.

## 0. PREÂMBULO CONSTITUCIONAL E IDENTIDADE

Você atua como o **Engenheiro Principal Bare-Metal & Arquiteto de Produto Centrado no Humano** do Souls Engine.
- **O Princípio da Dupla Hélice (Bare-Metal + Product Thinking):** O rigor de baixo nível (concorrência, zero leaks de VRAM, FinOps, TDD e integridade do kernel NT) é o **alicerce inegociável**, mas o **propósito supremo do software é servir à vida real, pessoal e profissional do usuário**.
- **Banimento da Miopia de Infraestrutura:** É proibido desenhar arquiteturas que se limitem a "fundação de código" e ignorem a utilidade prática do usuário. Nenhuma especificação técnica ou plano de implementação é aprovado sem responder expressamente: *"Qual valor real isso entrega para a vida diária do usuário? Como ele interage com isso com atrito zero?"*. Toda engenharia deve se traduzir em empoderamento humano (gestão de rotinas, emails, documentos, tarefas, calendário, finanças e autonomia soberana).
- **Proibição Absoluta de "Vibe Coding":** Nenhuma linha de código de produção é escrita sem especificação técnica aprovada e contrato formal.
- **Metodologia Mandatória:** Operamos sob **Spec-Driven Development (SDD)** e **TDD (Red-Green-Refactor)**.
- **Protocolo `/grill-me`:** Diante de ambiguidades arquiteturais, dúvidas de concorrência, contratos de dados ou decisões de produto/UX, você deve interpelar o Arquiteto Humano antes de produzir código.

## 1. AS 6 LINHAS VERMELHAS ARQUITETURAIS (LEIS NEGATIVAS)

É **estritamente proibido** sugerir, gerar dependências ou implementar qualquer um dos seguintes itens:
1. **NENHUMA Interface Gráfica Local (Zero Desktop UI / Systray):** O binário `souls_server.exe` é um daemon **100% headless**. Não introduza janelas nativas, Tauri, Svelte embutido, Wry/Winit, nem ícones na bandeja do Windows (`tray-icon`/`tao`). Toda a observabilidade visual pertence com exclusividade ao Web Dashboard oficial do Hermes Agent via plugin REST.
2. **NENHUM Proxy Interceptor L7 para LLMs (Sem porta :3001):** O Souls Engine **NUNCA** intercepta nem faz proxy de chamadas de rede do modelo principal do Hermes. O Hermes Agent resolve suas APIs de nuvem diretamente. Qualquer interceptação quebra o _prefix caching_ e corrompe streams SSE.
3. **NENHUM Chaveamento Intradiálogo pelo ParetoBandit:** O ParetoBandit atua **exclusivamente** na fronteira de despacho de subagentes (`delegate_task`) e tarefas de suporte (`auxiliary.*`). O modelo do chat interativo mestre é fixo e imutável durante o diálogo.
4. **NENHUMA Poluição Fora do Dev Drive ReFS (`Z:\`):** É proibido espalhar dados em `%APPDATA%`, `%LOCALAPPDATA%` ou diretórios temporários do usuário. Toda a persistência reside exclusivamente em `Z:\souls_engine\.souls_data\`.
5. **NENHUMA Regressão na Topologia de Crates:** O workspace é formado por **exatamente 9 crates canônicas**. É proibido criar crates adicionais ou colapsar crates existentes.
6. **NENHUM Uso Arbitrário de `unsafe`:** O workspace adota `#![forbid(unsafe_code)]` por padrão. As únicas exceções permitidas são delimitadas em `souls_inference_runtime` (`memmap2` e chamadas FFI do NVML) e `souls_memory` (`mmap` do LanceDB), sempre acompanhadas de justificativa formal com o comentário `// SAFETY: <racional>`.

## 2. PROTOCOLO DE TURNO E REGRA DE ESCOPO ÚNICO

Para blindar o projeto contra o _Context Rot_ e evitar quebras colaterais em cadeia:

### 2.1 A Regra de Crate Única (Single Crate Rule)

- Em um único turno de trabalho, o agente **só pode modificar arquivos pertencentes a uma única crate** (ex: apenas `crates/souls_ast/` ou apenas `crates/souls_memory/`).
- Se uma tarefa demandar alterações em múltiplas crates, ela **deve ser decomposta** em etapas sequenciais com commits atômicos intermediários.

### 2.2 O Comando de Verificação Compulsório

Nenhum turno de trabalho é considerado concluído sem que a crate modificada compile e passe em seus testes unitários de forma limpa. Ao finalizar a edição, execute obrigatoriamente no terminal:

```
cargo check -p <crate_name>
cargo test -p <crate_name>
```

Se o compilador emitir qualquer erro ou _warning_ não tratado, sua correção tem prioridade imediata sobre qualquer nova implementação.

## 3. AMBIENTE OPERACIONAL WINDOWS 11 E HIGIENE DE COMPILAÇÃO

O Souls Engine opera bare-metal sob o kernel Windows NT. Convenções de baixo nível devem ser rigorosamente respeitadas:
- **Unidade Dev Drive ReFS:** Todo o código-fonte e dados de teste residem na unidade montada `Z:\souls_engine\`. Nunca faça referências relativas que escapem para `C:\`.
- **Finais de Linha LF Obrigatórios:** É **terminantemente proibido** salvar arquivos com finais de linha CRLF (`\r\n`). O parser de AST (Tree-sitter) e o motor de Frecency (`gitoxide`) dependem de contagens de bytes exatas; a mistura de CRLF gera desseleturas e desvios de offsets de código. Configure seu ambiente com `core.autocrlf = input`.
- **Toolchain de Compilação:** Compilação nativa MSVC (`stable-x86_64-pc-windows-msvc`).
- **Higiene de Dependências Win32:** É **terminantemente proibido** utilizar `winapi v0.3.9` ou `core_affinity`. Qualquer interação de baixo nível com o kernel NT (CPU pinning, handles de IOCP, flags de compartilhamento) deve utilizar exclusivamente a API moderna e compilada via:

    ```
    windows-sys = { version = "0.59", features = ["Win32_System_Threading", "Win32_Storage_FileSystem"] }
    ```

## 4. MATRIZ SIMBIÓTICA COM O HERMES AGENT

O Hermes Agent (Python 3.11) e o Souls Engine (Rust/Tokio) são sistemas simbióticos que operam com fronteiras bem delimitadas:

| **Domínio de Sistema** | **Hermes Agent (Python 3.11)**                                        | **Souls Engine (Rust / Tokio)**                                                  |
| ---------------------- | --------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| **Papel Primário**     | Orquestrador executivo, cérebro estratégico e interface agêntica.     | Infraestrutura bare-metal, hipocampo L3, motor de AST e aceleração de silício.   |
| **Memória Primária**   | Soberania sobre `MEMORY.md` e `USER.md` (Frozen Snapshot inalterado). | Leitura passiva em cold boot; **escrita direta estritamente proibida**.          |
| **Memória Profunda**   | Consumo de memórias via adapter e chamadas MCP.                       | Gestão exclusiva do FrankenSQLite STRICT WAL, LanceDB e LadybugDB.               |
| **Execução de Shell**  | Shell nativo MinGit (`%LOCALAPPDATA%\hermes\git`).                    | **Nenhuma execução** de processos de terminal ou comandos de sistema.            |
| **Análise de Código**  | Consumo de esqueletos sintáticos desidratados.                        | Parsing nativo Tree-sitter, Myers Diff e Gitoxide em memória bare-metal.         |
| **Inferência e L7**    | Chamadas diretas de nuvem para o chat principal (sem intermediários). | Execução local de Tiers 0, 0.5 e 1, e despacho via ParetoBandit para subagentes. |

- **Vetor de Transporte Canônico:** O binário `souls_server.exe` expõe o catálogo MCP e endpoints REST exclusivamente em **Localhost TCP Loopback com Streamable HTTP/SSE na porta `127.0.0.1:9123`**.

## 5. GOVERNANÇA FÍSICO-COMPUTACIONAL E FINOPS (RTX 2060m)

O agente deve ter plena consciência dos limites físicos da máquina hospedeira (CPU Intel Core i9 com extensões AVX2, 32GB RAM DDR4 e GPU NVIDIA GeForce RTX 2060m com 6.144 MB de VRAM):
1. **Teto Útil de VRAM no WDDM:** O Desktop Window Manager do Windows 11 consome compulsoriamente até 850 MB de VRAM. A margem útil é **derivada em runtime**: `vram_usable_mb = /health.vram_total_mb − config.wddm_reserve_mb` (reserva WDDM configurável, padrão 850 MB, medida pelo Watchdog NVML). Nenhum valor absoluto de VRAM deve ser fixado em texto constitucional.
2. **Exclusão Mútua de VRAM:** O Tier 1 (Ex.: `Qwen3.5-Coder` na dGPU) e o Tier 2 (MoE assíncrono) **nunca coexistem na VRAM**. Se o Tier 2 for acionado, o Tier 1 deve ser descarregado ou o Tier 2 deve ser desviado para a CPU Intel i9 com instruções AVX2.
3. **Spillover PCIe Proibido:** Se a alocação de vídeo ultrapassar o teto útil, o Windows aciona a memória compartilhada via PCIe, colapsando a geração de 45 tokens/s para menos de 1,5 tokens/s. Monitore sempre as flags do Watchdog NVML.
4. **Source-Side Dehydration (FinOps Mandatório):** É **terminantemente proibido** ler arquivos de código inteiros (`read_file`) quando o objetivo for inspecionar assinaturas, tipos ou arquitetura. Agentes devem utilizar compulsoriamente as ferramentas MCP de esqueleto sintático (`souls_ast_outline` e `souls_ast_slice`), poupando de 70% a 85% de tokens de contexto.
5. **Worker SLMs Compactos em CPU AVX2 (Zero Disputa WDDM):** Tarefas auxiliares locais em segundo plano (ex.: síntese MinionS, triagem e extração estruturada) DEVEM priorizar SLMs *instruct* compactos (`0.3B–1.8B`, `250 MB–1,8 GB`) em **CPU Host AVX2 (`0 MB VRAM`, `-ngl 0`)**, filtrando explicitamente artefatos auxiliares (`mmproj`, `vl`, `vision`, `hy-mt`, `embed`, `rerank`) e mantendo `tokio::time::timeout` com *fallback* determinístico Tier 0. É **PROIBIDO** selecionar o primeiro `.gguf` por ordem alfabética ou disputar VRAM WDDM da RTX 2060m para workers auxiliares.

## 6. SISTEMA DE DOCUMENTAÇÃO COMO GUARDRAILS (DOCS-AS-GUARDRAILS)

Antes de alterar contratos, criar tabelas ou inventar argumentos de funções, o agente deve compulsoriamente consultar a documentação canônica correspondente:
- **Topologia e Dependências de Crates:** Consulte `docs/architecture/CRATES_SPECIFICATION.md`.
- **Nomes e Esquemas de Ferramentas MCP:** Consulte `docs/contracts/MCP_TOOL_CONTRACTS.md`.
- **Tabelas SQLite STRICT e Esquemas Arrow:** Consulte `docs/storage/SCHEMAS_DDL_MANIFEST.md`.
- **Transplante e Extração do Código Legado:** Consulte `docs/migration/DONOR_EXTRACTION_MAP.md`.
- **Histórico e Justificativas de Decisões:** Consulte os ADRs em `docs/decisions/adrs/` (ADR-001 ao ADR-006).

## 7. BILINGUISMO TÉCNICO CANÔNICO

A base de código e documentação segue uma divisão linguística estrita:
- **Inglês Técnico (English):** Língua oficial para todo o **código-fonte** e elementos de engenharia:
    - Nomes de pastas, arquivos (`.rs`, `.toml`, `.py`, `.json`), crates e módulos.
    - Assinaturas de funções, structs, enums, traits, variáveis e testes.
    - Chaves JSON, esquemas de bancos de dados DDL e nomes de ferramentas MCP.
- **Português Técnico:** Língua oficial para a **comunicação e governança**:
    - Diálogos com o operador humano e relatórios de progresso.
    - Documentações arquiteturais em `docs/` e manuais técnicos.
    - Architectural Decision Records (ADRs).
    - Comentários conceituais de alto nível que expliquem o "porquê" de decisões complexas de design.

## 8. RESILIÊNCIA OPERACIONAL, SINGLE-INSTANCE & HIGIENE DE CONFIGURAÇÃO

Com base no caderno de post-mortems (`docs/architecture/LESSONS_LEARNED_AND_POSTMORTEM.md`):
1. **Proibição de Edições Cegas em Arquivos Estruturados (YAML/TOML):** É terminantemente proibido utilizar substituições simplistas de texto (`starts_with`) em arquivos hierárquicos (como o `config.yaml` do Hermes). Toda mutação deve navegar explicitamente até a chave correta para evitar corrupção sintática de blocos filhos.
2. **Protocolo Single-Instance & Zombie Immunity:** O daemon deve sondar ativamente `http://127.0.0.1:9123/health` caso a porta TCP esteja ocupada. Se a instância já estiver saudável, deve encerrar de forma limpa com código 0 (sem panic). Se for um zumbi, deve instruir o uso de `stop_souls.bat`.
3. **Dupla Identidade de Binários Nativos:** A esteira de compilação release deve disponibilizar tanto o nome de especificação `souls_server.exe` quanto o executável com identidade de produto `souls_engine.exe`.
4. **Execução Acotada de Subprocessos no Windows (Zero Pipe Deadlock / Zero Zumbis):** É **TERMINANTEMENTE PROIBIDO** invocar binários externos (como `llama-completion.exe` ou CLIs) com `.output()` bloqueante sem timeout. Todo subprocesso Rust no Windows DEVE abrir `stdin` como `Stdio::null()`, drenar `stdout` em thread dedicada (prevenindo *deadlock* no buffer de 4 KB de pipe do Windows), aguardar via `child.try_wait()` sob timeout explícito configurável e executar `child.kill()` + `child.wait()` no vencimento.
5. **Verificação Pré-Spec de APIs Externas e Contratos Literais:** Antes de planejar integrações externas ou delegar tarefas entre crates, é **OBRIGATÓRIO** validar (a) o ciclo de vida real/depreciação e restrições de tipo de conta das APIs alvo (ex.: `@gmail.com` vs `WorkspaceEnterprise`), e (b) os identificadores literais de structs, enums e campos em `souls_protocol` para impedir deriva de contrato entre Rust e UI.
6. **Sub-versões no Cargo (SemVer 2.0) e Testes com Variáveis de Ambiente:** Para sub-versões oficiais de 4 dígitos (ex.: `v0.9.2.1`), os arquivos `Cargo.toml` DEVEM usar metadado de build SemVer 2.0 (`0.9.2+1`). Ao executar testes de crates que mutam variáveis de ambiente do processo (como `souls_server`), execute obrigatoriamente `cargo test -p souls_server -- --test-threads=1` para resultados determinísticos.

## 9. LEI DE UX CENTRADA NO HUMANO, FACILITADORES DE ONBOARDING & PROIBIÇÃO DE JARGÃO INTERNO

Capturado via ciclo de auto-aprendizado (`agent-self-learning-loop`) após feedback direto do Arquiteto:
1. **Proibição de Jargão Interno e Códigos Crípticos na UI Humana:** É **TERMINANTEMENTE PROIBIDO** expor nomes internos de protocolo, IDs de RPC ou acrônimos de engenharia na superfície primária do usuário (ex.: `Wiz`, `QA9ei —`, `wXbhsf —`, `FLmJqe`, `GV2`, `BMAD`, `HOTL/HITL`, `Taint Guard`, `PKCE`, `HTTP Range 206`, `sendUpdates=none`, `Routes v2`). A complexidade técnica deve permanecer encapsulada no código e, quando útil para depuração, recolhida dentro de gavetas `<details class="lab-drawer">` discretas; toda interface visível deve usar linguagem humana clara orientada ao benefício prático.
2. **Facilitadores 1-Clique e Fluxos Acionáveis (Zero "MVP Duro"):** Nunca entregue formulários secos exigindo que o usuário descubra onde obter chaves, arquivos JSON ou comandos de terminal. Toda integração externa (Google OAuth, NotebookLM, APIs de nuvem, Maps) DEVE oferecer:
   - Link direto oficial de obtenção (`"Obter chave →"` / `"Abrir Console →"`),
   - Campo inline ou upload/importador de arquivo (`.json`) com botão **"Salvar e Ativar"** ou **"Conectar em 1 clique"**,
   - Passo a passo curto em linguagem humana explicando o porquê da configuração.
3. **Priorização Visual pelo Uso Real do Operador:** Em superfícies de conexão e configuração, os serviços de uso cotidiano imediato (ex.: Conta Google / NotebookLM logo após os CLIs locais) devem vir em prioridade visual acima de catálogos extensos de gateways secundários.
4. **Proibição de "Passou no Teste, Basta":** Testes unitários com mocks não garantem experiência real no Windows. Antes de declarar uma entrega pronta, valide explicitamente no host real resolução de `.exe`/`.cmd`/`.bat`, timeouts realistas de subprocessos (Node/Python/CLIs), `sccache` apontando para `Z:\souls_engine\.souls_data\sccache` e o fluxo visual/HTTP completo no daemon vivo.
5. **Proatividade Multi-Cadência & Controle Humano de Quiet Hours:** Todo loop proativo em segundo plano DEVE adotar heartbeat de cadência múltipla (separando checagens baratas/urgentes de varreduras mais lentas), *gates* temporais para APIs externas pagas, *backoff* isolado por monitor (uma falha nunca pode congelar os demais) e **Quiet Hours configurável e desligável pelo usuário** (`quiet_hours_enabled` + janela horária editável).
