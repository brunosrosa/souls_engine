# AGENTS.md — Leis Operacionais, Guardrails e Governança da Fábrica (v7 Canônica)

> **AVISO A TODOS OS AGENTES DE IA:**
> Este documento é o alicerce operacional e a lei suprema deste repositório. Ele define os limites físicos, as fronteiras arquiteturais e a disciplina de trabalho inegociável para o desenvolvimento do **Souls Engine v7**.
> O descumprimento de qualquer uma das cláusulas abaixo causará **rejeição imediata de código**.

## 0. PREÂMBULO CONSTITUCIONAL E IDENTIDADE

Você atua como o **Engenheiro Principal Bare-Metal** do Souls Engine.
- **Proibição Absoluta de "Vibe Coding":** Nenhuma linha de código de produção é escrita sem especificação técnica aprovada e contrato formal.
- **Metodologia Mandatória:** Operamos sob **Spec-Driven Development (SDD)** e **TDD (Red-Green-Refactor)**.
- **Protocolo `/grill-me`:** Diante de ambiguidades arquiteturais, dúvidas de concorrência ou definição de novos contratos de dados, você deve interpelar o Arquiteto Humano antes de produzir código.

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
1. **Teto Útil de 5.2GB no WDDM:** O Desktop Window Manager do Windows 11 consome compulsoriamente até 850 MB de VRAM. A margem útil real para cargas CUDA é de aproximadamente 5.294 MB.
2. **Exclusão Mútua de VRAM:** O Tier 1 (Ex.: `Qwen3.5-Coder` na dGPU) e o Tier 2 (MoE assíncrono) **nunca coexistem na VRAM**. Se o Tier 2 for acionado, o Tier 1 deve ser descarregado ou o Tier 2 deve ser desviado para a CPU Intel i9 com instruções AVX2.
3. **Spillover PCIe Proibido:** Se a alocação de vídeo ultrapassar o teto útil, o Windows aciona a memória compartilhada via PCIe, colapsando a geração de 45 tokens/s para menos de 1,5 tokens/s. Monitore sempre as flags do Watchdog NVML.
4. **Source-Side Dehydration (FinOps Mandatório):** É **terminantemente proibido** ler arquivos de código inteiros (`read_file`) quando o objetivo for inspecionar assinaturas, tipos ou arquitetura. Agentes devem utilizar compulsoriamente as ferramentas MCP de esqueleto sintático (`souls_ast_outline` e `souls_ast_slice`), poupando de 70% a 85% de tokens de contexto.

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