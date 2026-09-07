# ADR-005 — Centralização Soberana de I/O em Dev Drive ReFS (`Z:\souls_engine\.souls_data\`) com Pipeline MPSC Desidratado contra Write Amplification no NVMe

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_core`, `souls_memory`, `souls_server`, `souls_protocol`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

Nas iterações experimentais do ecossistema (era Souls MC v6 / SODA), a gestão de armazenamento do sistema sofria de uma dicotomia arquitetural problemática:
1. **Dispersão em Pastas Tradicionais do Usuário:** O sistema fragmentava seus dados entre `%APPDATA%\souls_engine\` (chamado de "Cofre da Alma" para o banco relacional SQLite) e `%LOCALAPPDATA%\souls_engine\` (destinado a vetores LanceDB e modelos GGUF).
2. **I/O Síncrono Bloqueante de Telemetria:** A cada interação do usuário, geração de token ou chamada de ferramenta, o motor disparava operações diretas de escrita relacional no SQLite (`INSERT INTO telemetry ...`), exigindo a confirmação física de sincronização em disco através da API Win32 `FlushFileBuffers`.

A validação operacional e o monitoramento físico em regime contínuo de desenvolvimento no Windows 11 revelaram sérios gargalos no subsistema de armazenamento e integridade física do hardware:

### 1.1 O Desgaste Físico do NVMe por SSD Write Amplification

A escrita contínua de pequenos blocos de metadados em bancos de dados relacionais convencionais gera o fenômeno de **SSD Write Amplification Factor (WAF)**:
- No nível do sistema operacional, uma linha de log ou evento de telemetria possui entre $200\text{ bytes}$ e $1.500\text{ bytes}$.
- Contudo, a controladora física do drive NVMe gerencia células flash NAND organizadas em páginas de $4\text{ KB}$ a $16\text{ KB}$ agrupadas em blocos de eliminação de $2\text{ MB}$ a $8\text{ MB}$.
- Quando centenas de eventos de telemetria e auditoria são persistidos individual e sincronamente por segundo, a controladora é forçada a executar ciclos exaustivos de _Read-Modify-Write_ e rotações prematuras de _Garbage Collection_.
- Esse comportamento eleva o WAF a patamares superiores a $15\times \text{ a } 30\times$, degradando a vida útil (_TBW — Terabytes Written_) das células NAND do drive NVMe e provocando picos de latência no I/O do sistema de arquivos durante a contenção de filas no driver NVMe.

### 1.2 Sobrecarga de Inspeção do Microsoft Defender em NTFS

No sistema de arquivos padrão NTFS em partições de sistema (`C:\`), o subsistema de antivírus e segurança do Windows (**Microsoft Defender Antivirus**) acopla filtros minifilter no nível do kernel (`fltmgr.sys`):
- Cada abertura, fechamento e mutação de descritor em arquivos `.db`, `.db-wal` ou arquivos colunares Arrow do LanceDB intercepta a pilha de I/O para inspeção heurística síncrona.
- Em sessões ativas com agentes disparando dezenas de consultas de AST e varreduras vetoriais por segundo, a sobrecarga da thread `MsMpEng.exe` consumia entre $15\%$ e $28\%$ dos ciclos de CPU da máquina, introduzindo jitter imprevisível e latência artificial de até $80\text{ ms}$ por operação de disco.

### 1.3 A Fragmentação em Pastas Ocultas (`%APPDATA%` e `%LOCALAPPDATA%`)

O particionamento clássico do Windows NT em pastas ocultas do usuário provou-se contraproducente para um motor de engenharia bare-metal:
- Impedia auditoria forense rápida pelo desenvolvedor, forçando navegação por pastas ocultas do sistema operacional.
- Dificultava a portabilidade, a clonagem rápida de estado e rotinas de backup determinístico do ambiente de desenvolvimento.
- Induzia agentes de IA a cometer erros recorrentes de concatenação de caminhos, misturando barras invertidas (`\`) com barras normais (`/`) e tentando gravar em locais protegidos ou voláteis sem permissões adequadas.

### 1.4 Bloqueio Obrigatório e Falhas de Compartilhamento no Windows NT

Conforme formalizado na semântica do kernel Windows NT, o subsistema de arquivos impõe bloqueio obrigatório (_mandatory file locking_):
- Se o processo Rust abrir o banco SQLite ou mapear os arquivos do LanceDB sem as flags explícitas de compartilhamento Win32, qualquer leitura concorrente externa (ex: scripts de backup ou inspeções do hermes) dispara instantaneamente:
$$\text{Erro 32: } \texttt{ERROR\_SHARING\_VIOLATION} \quad \text{ou} \quad \text{Erro 1224: } \texttt{ERROR\_USER\_MAPPED\_FILE}$$
- A sincronização entre processos heterogêneos sob o Windows 11 exige uma governança de descritores padronizada e unificada sob um único processo mestre.

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretada a **adoção mandatória do Dev Drive formatado em ReFS (Resilient File System), montado soberanamente na partição canônica `Z:\souls_engine\.souls_data\`, combinada com um Pipeline de I/O Desidratado via MPSC com Batch-Flushing Transacional de 5 segundos**.

O Souls Engine assume controle exclusivo e unificado sobre todo o armazenamento persistente e temporário do sistema, extinguindo qualquer dependência de diretórios `%APPDATA%` ou `%LOCALAPPDATA%`.

### 2.1 Ancoragem Soberana no Dev Drive ReFS (`Z:\souls_engine\.souls_data\`)

Todo o estado relacional, vetorial, binário e transitório do Souls Engine é ancorado estritamente no volume montado sob a unidade `Z:`:

$$\text{Root Path} = \texttt{Z:\textbackslash souls\_engine\textbackslash .souls\_data\textbackslash}$$

A adoção do Dev Drive ReFS no Windows 11 introduz vantagens arquiteturais inalcançáveis em partições NTFS padrão:
1. **Desempenho Otimizado via Block Cloning (Copy-on-Write):**
    - O ReFS suporta primitivos de baixo nível `FSCTL_DUPLICATE_EXTENTS_TO_FILE`.
    - Permite criar snapshots instantâneos e cópias integrais de bancos de dados relacionais e vetoriais em tempo de execução sem duplicar blocos físicos no NVMe e com custo de I/O próximo a zero.
2. **Mitigação Nativa de Antivírus (Defender Performance Mode):**
    - No Windows 11, o Microsoft Defender opera em **Modo de Desempenho Assíncrono** sobre Dev Drives ReFS.
    - O escaneamento síncrono inline de minifilters é suspenso, desacoplando a verificação antivírus das chamadas de I/O da aplicação, reduzindo a sobrecarga de I/O do Defender em até $70\%$.
3. **Alocação de Clusters e Imunidade a Corrupção Silenciosa:**
    - O ReFS utiliza árvores B+ resilientes com alocação em blocos de $4\text{ KB}$ e integridade validada por somas de verificação (_checksums_ de metadados de $64\text{ bits}$), garantindo autocorreção contra corrupções silenciosas de bits (_bit rot_).

### 2.2 Topologia Canônica dos Diretórios em `Z:\`

O diretório mestre `.souls_data` é segregado em quatro subsistemas físicos com responsabilidades rígidas:

| **Diretório Canônico**                 | **Finalidade Estrutural**                                                    | **Tecnologia / Formato de Persistência**                               |
| -------------------------------------- | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `Z:\souls_engine\.souls_data\db\`      | Hipocampo relacional L3, grafo ontológico e tabelas de governança.           | FrankenSQLite STRICT WAL (`souls_state.db`, `-wal`, `-shm`).           |
| `Z:\souls_engine\.souls_data\vectors\` | Repositório colunar de busca vetorial semântica profunda.                    | Apache Arrow colunar via LanceDB (`souls_vectors.lance`).              |
| `Z:\souls_engine\.souls_data\models\`  | Pesos locais quantizados de SLMs e modelos de embedding/classificação.       | Formatos GGUF (llama.cpp) e ONNX com aceleradores AVX2.                |
| `Z:\souls_engine\.souls_data\spool\`   | Buffers transitórios de telemetria, traces estruturados e despejos de crash. | Arquivos de spool temporal delimitados e arquivos de lock de processo. |

É expressamente proibido criar arquivos ou pastas de dados fora dessa topologia estrita.

### 2.3 Pipeline de I/O Desidratado via MPSC (Batch-Flushing de 5 Segundos)

Para erradicar o problema de Write Amplification no NVMe e eliminar contenções de travamento em threads ativas:

1. **Buffer Não-Bloqueante em RAM:**
    - Toda emissão de métricas de telemetria, eventos de ciclo de vida, registros de auditoria FinOps e nós transitórios do grafo LadybugDB é enfileirada em um canal assíncrono Tokio:
$$\text{Canal MPSC} = \texttt{tokio::sync::mpsc::channel(10000)}$$
- A operação de envio (`tx.try_send(event)`) consome menos de $1\ \mu\text{s}$ na thread de trabalho, sem disparar qualquer chamada de sistema de disco.
1. **Flusher Daemon Transacional em Background:**
    - A crate `souls_core` opera uma thread assíncrona dedicada que agrega os eventos recebidos no vetor de memória RAM.
    - A gravação física no banco SQLite é executada exclusivamente sob duas condições:
        1. **Critério Volumétrico:** O buffer atinge $500\text{ eventos}$ em memória.
        2. **Critério Temporal:** O temporizador de $5\text{ segundos}$ dispara via `tokio::time::interval`.
2. **Descarga Transacional Agregada:**
    - Os $500$ eventos são descarregados em **uma única transação atômica** (`BEGIN IMMEDIATE ... COMMIT`).
    - O número de operações de escrita física no SSD cai de centenas por segundo para **apenas uma operação atômica a cada 5 segundos**, reduzindo o volume de gravações físicas no NVMe em mais de $94\%$.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    APLICAÇÃO / THREADS WORKER DO TOKIO                      │
│   (souls_ast, souls_model_router, souls_server, souls_inference_runtime)     │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ tx.send(event)  (< 1 µs na RAM)
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│               CANAL MPSC BUFFERIZADO (Capacidade: 10.000)                   │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Consumo contínuo
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                  DAEMON BATCH FLUSHER (souls_core / memory)                 │
│         Condição de Flush: [Buffer >= 500 itens] OU [Tick a cada 5s]        │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ UMA única transação agrupada (COMMIT)
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│               FRANKENSQLITE STRICT WAL EM DEV DRIVE REFS (Z:)                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.4 Primitivos Win32 e Flags de Compartilhamento Obrigatórias

Toda inicialização de descritores de arquivos e pools de conexões em Rust deve utilizar primitivos modernos da biblioteca `windows-sys` (`version = "0.59"`):

1. **Flags de Abertura:** Os descritores de arquivo devem explicitar as diretivas Win32:

```
// Abertura segura sem bloqueio obrigatório no kernel Windows NT
FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
```

2. **Parâmetros PRAGMA de Conexão no SQLite:**
    - `PRAGMA journal_mode = WAL;`: Desacopla leituras concorrentes da escrita física no log.
    - `PRAGMA synchronous = NORMAL;`: Elimina chamadas desnecessárias a `FlushFileBuffers`, sincronizando exclusivamente nos pontos críticos de rotação do WAL.
    - `PRAGMA busy_timeout = 5000;`: Janela de espera com recuo exponencial contra contenções de I/O.
3. **Governança Unificada de `mmap` do LanceDB:**
    - O acesso a `Z:\souls_engine\.souls_data\vectors\` é **monopólio absoluto do processo nativo `souls_server.exe`**.
    - Nenhum outro processo (inclusive o interpretador Python do Hermes) pode abrir descritores diretos nesses arquivos, prevenindo o erro `ERROR_USER_MAPPED_FILE` gerado pelo bloqueio de memória virtual.

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Preservação Física do Drive NVMe:** A amortização de escritas em lotes de 5 segundos reduz drasticamente o Write Amplification Factor (WAF) do SSD, estendendo a longevidade do hardware sob uso contínuo de agentes autônomos.
- **Latência Amortizada Quase Nula em Telemetria:** As rotinas de registro de logs e telemetria tornam-se não bloqueantes ($< 1\ \mu\text{s}$), permitindo que o Hermes Agent e as ferramentas MCP operem na velocidade de memória RAM.
- **Imunidade a Contenções do Microsoft Defender:** A concentração no Dev Drive ReFS desacopla a inspeção de minifilters antivírus da esteira crítica, garantindo tempos de resposta sub-milissegundo constantes em consultas ao FrankenSQLite e LanceDB.
- **Auditabilidade e Portabilidade Imediata:** Todo o estado do Souls Engine está unificado em `Z:\souls_engine\.souls_data\`. O backup, expurgo ou replicação do ecossistema é executado através de um único comando de cópia atômica.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**                            | **Impacto Técnico**                                                                                                             | **Mitigação Canônica Implementada**                                                                                                                                                                                  |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Dependência de Partição Dedicada `Z:`**               | Se o operador não configurar o Dev Drive ou a unidade `Z:` estiver ausente, o motor falha no boot.                              | Script de arranque e montagem determinística `boot.ps1` que valida ou provisiona um Dev Drive VHDX no Windows 11 com um único comando; validação defensiva em `souls_core::ensure_refs_directory`.                   |
| **Risco Teórico de Perda de Telemetria em Crash do SO** | Se a máquina sofrer queda abrupta de energia (_hard power-off_), os últimos 5 segundos de telemetria em RAM podem ser perdidos. | O pipeline MPSC é restrito a dados não críticos (telemetria e métricas FinOps). Memórias epistêmicas duradouras (`on_pre_compress` e transações explícitas) executam _commit_ síncrono imediato com confirmação WAL. |
| **Incompatibilidade com Versões Antigas do Windows**    | O Dev Drive com ReFS é um recurso nativo restrito ao Windows 11 (build 22621+).                                                 | O Souls Engine foi formalmente especificado para operar em ambiente bare-metal Windows 11 nativo, tornando a restrição irrelevante para o escopo do projeto.                                                         |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE PERSISTÊNCIA FORA DO DEV DRIVE:** É expressamente proibido sugerir ou criar arquivos em diretórios do sistema como `%APPDATA%`, `%LOCALAPPDATA%`, `C:\Users\` ou diretórios temporários do Windows. Todo e qualquer arquivo gerado pelo sistema deve residir em `Z:\souls_engine\.souls_data\`.
2. **PROIBIÇÃO DE ESCRITAS SÍNCRONAS DE TELEMETRIA:** Nenhuma crate pode invocar comandos diretos `INSERT INTO telemetry_events` de forma síncrona na thread principal de uma requisição. Toda telemetria deve ser canalizada compulsoriamente via `MpscBatchBuffer`.
3. **PROIBIÇÃO DE MODIFICAÇÃO DE FLAGS DE SINCRONIZAÇÃO:** É proibido alterar o pragma `synchronous = NORMAL` para `synchronous = FULL` ou desativar o modo `WAL` do SQLite. As diretivas de concorrência e integridade do ReFS são canônicas e invioláveis.
4. **PROIBIÇÃO DE CRIAÇÃO DE DESCRITORES DIRETOS NO LANCEDB POR PYTHON:** Nenhum script, plugin ou adapter em Python pode utilizar bibliotecas LanceDB para abrir os arquivos em `Z:\souls_engine\.souls_data\vectors\`. Todo I/O vetorial deve transitar via JSON-RPC/HTTP gerenciado pela crate `souls_memory`.
5. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** Propostas de código que introduzam caminhos estáticos para `C:\` ou que removam o buffer de desidratação MPSC de 5 segundos sofrerão **rejeição sumária de PR** com violação de conformidade do ADR-005.
## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Técnica Canônica v7:** Seção 2.2 (_Isolamento de Dados em Dev Drive ReFS_) e Seção 7.1 (_Pipeline de I/O Desidratado via MPSC_).
- **`AGENTS.md`:** Linha Vermelha 4 (_NENHUMA Poluição Fora do Dev Drive ReFS Z:\_) e Seção 3 (_Ambiente Operacional Windows 11_).
- **`CRATES_SPECIFICATION.md`:** Seção 2.2 (`souls_core`) e Seção 2.4 (`souls_memory`).
- **`SCHEMAS_DDL_MANIFEST.md`:** Seção 1 (_Preâmbulo e Topologia de Persistência Win32 ReFS_).
- **ADR-001:** _Abandono de UI Desktop em Favor de Daemon Headless_.
- **ADR-002:** _Adoção de Localhost TCP Loopback e Streamable HTTP/SSE_.
- **ADR-004:** _Governança de VRAM na RTX 2060m e Watchdog NVML_.