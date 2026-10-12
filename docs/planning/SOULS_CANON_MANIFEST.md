# SOULS CANON MANIFEST (v7 Canônica — Souls Engine Era)

> **ESTATUS DO DOCUMENTO:** IMUTÁVEL / SSOT EPISTÊMICO (ÚNICA FONTE DE VERDADE)
> **APLICAÇÃO:** Agentes de IA Analistas, LLMs do Pipeline de Antropofagia, Engenheiros Principais de Sistemas.
> **AXIOMA CENTRAL:** _"O silício é o nosso limite; a soberania é o nosso dogma; o silêncio é a nossa estética."_

## 1. IDENTIDADE, MISSÃO E PARCERIA SIMBIÓTICA (O "PORQUÊ")

O **Souls Engine** não é um chatbot, não é um assistente de produtividade genérico e não é um invólucro superficial (_wrapper_) de APIs de modelos de linguagem.

Ele é concebido como um **Exoesqueleto Cognitivo** e uma **Prótese de Função Executiva de Baixo Nível**, desenhado primordialmente para apoiar operadores humanos **neurodivergentes (TDAH e Dupla Excepcionalidade — 2e)** na erradicação da **Dívida de Fluxo (**_**Flow-Debt**_**)**.

### 1.1 A Mitigação da Dívida de Fluxo (_Flow-Debt_)

Para o operador neurodivergente, o atrito cognitivo não decorre da falta de ideias ou incapacidade técnica, mas da sobrecarga crônica nas funções executivas do córtex pré-frontal:
- **Fadiga de Decisão por Microtarefas:** O custo de transição de contexto, manipulação de arquivos, navegação em diretórios profundos e setups manuais drena a dopamina antes do trabalho profundo começar.
- **Esgotamento da Memória de Trabalho:** Hipóteses brilhantes são perdidas no momento em que a mente é forçada a resolver detalhes de infraestrutura ou sintaxe repetitiva.
- **Inércia de Inicialização e Paralisia de Análise:** A dificuldade em decompor problemas massivos em passos atômicos executáveis.

O Souls Engine atua como um parceiro simbiótico residente no bare-metal do usuário: ele absorve a fricção mecânica, protege o estado de hiperfoco, gerencia o histórico de decisões e fornece uma âncora estrutural permanente para a cognição humana.

### 1.2 O Sparring Cognitivo (Criatividade Ancorada)

A interação entre o operador humano e o Souls Engine é uma **parceria de pensamento crítico (**_**cognitive sparring**_**)**:
- O sistema não assume passividade subserviente nem alucina concordâncias vazias.
- Ele questiona premissas frágeis, calcula o raio de impacto de refatorações, aponta contradições ontológicas em decisões passadas e fornece alternativas de implementação com custos FinOps explícitos.
- A inteligência gerada é sempre ancorada na realidade física do hardware e na integridade matemática do código.
### 1.3 O Paradoxo do Silêncio e a Simbiose com o Hermes Agent

Diferente de sistemas legados que tentavam construir interfaces gráficas pesadas, janelas flutuantes e consoles coloridos:
- O Souls Engine v7 é um **Daemon 100% Headless Bare-Metal (`souls_server.exe`)**. Ele é invisível, silencioso e opera na retaguarda como infraestrutura de alta velocidade.
- A interface executiva, conversacional e de voz pertence com exclusividade ao **Hermes Agent / Hermes Desktop** (Nous Research), com o qual o Souls Engine opera em simbiose através de Localhost TCP Loopback na porta `127.0.0.1:9123`.
- Toda a observabilidade visual reside exclusivamente no Web Dashboard oficial do Hermes (`hermes dashboard`) via componentes integrados ao SDK global.

## 2. O TREINO DE GRAVIDADE E AS LEIS FÍSICAS DO SILÍCIO

Qualquer projeto de software, biblioteca ou repositório open-source sob escrutínio da esteira antropofágica deve ser avaliado contra a sua capacidade de sobreviver ao nosso **Treino de Gravidade Físico**.

A nossa máquina de referência não é um cluster infinito de servidores em nuvem, mas um ambiente bare-metal Windows 11 com hardware estritamente delimitado:

| **Componente Físico**    | **Especificação Soberana**               | **Limite Operacional Inviolável**                              |
| ------------------------ | ---------------------------------------- | -------------------------------------------------------------- |
| **Processador (CPU)**    | Intel Core i9-9880H (16 threads lógicas) | Conjunto de instruções vetoriais AVX2 / OpenVINO.              |
| **Memória RAM**          | 32 GB DDR4 (Dual-Channel)                | Heap controlado; zero vazamentos em daemons contínuos.         |
| **Placa Gráfica (dGPU)** | NVIDIA GeForce RTX 2060m (GDDR6)         | **Teto útil real de 5.294 MB** sob WDDM 3.x do Windows 11.     |
| **Armazenamento**        | Drive NVMe sob Dev Drive ReFS            | Volume `Z:\souls_engine\.souls_data\` com _Block Cloning_ CoW. |

### 2.1 As Quatro Leis Inegociáveis do Silício

1. **A Barreira Termodinâmica e Exclusão Mútua de VRAM:**
    - O subsistema gráfico do Windows consome até $850\text{ MB}$ de VRAM para o compositor de janelas (`dwm.exe`).
    - O espaço utilizável para alocações CUDA é de no máximo $5.294\text{ MB}$.
    - Ultrapassar esse limite ativa o _Shared GPU Memory Fallback_ via PCIe 3.0, derrubando a geração de tokens de $45\text{ t/s}$ para menos de $1,5\text{ t/s}$.
    - O modelo primário de código (Tier 1) e modelos pesados em segundo plano (Tier 2) **nunca coexistem na dGPU**. Cargas concorrentes devem ser obrigatoriamente desviadas para a CPU Intel i9 (AVX2).
2. **Fobia de Runtimes Tóxicos Contínuos:**
    - É terminantemente proibida a incorporação de dependências permanentes de Node.js, Electron, Python ou Java Virtual Machine (JVM) rodando em background dentro da infraestrutura do Souls Engine.
    - O núcleo do daemon é estritamente compilado em **Rust assíncrono (Tokio)** nativo MSVC.
3. **Garantia de 0 MB de VRAM para AST e Vetores:**
    - Mecanismos de busca semântica (LanceDB sobre Apache Arrow), parsers sintáticos (Tree-sitter) e modelos de classificação de embeddings operam **estritamente em memória RAM física e CPU AVX2**, preservando 100% da VRAM disponível para o LLM local.
4. **Isolamento de Sidecars e Morte Atômica:**
    - Ferramentas ou scripts externos essenciais que não possam ser reescritos imediatamente em Rust devem ser executados em sandboxes isoladas (Wasmtime para lógica pura; AppContainer/JobObject Win32 para processos de sistema) e devem sofrer terminação forçada (`SIGKILL`) imediatamente após o término da tarefa.

## 3. A RÉGUA ANTROPOFÁGICA: OURO PURO VS. CARCAÇA TÓXICA

Na filosofia do modernismo antropofágico, não imitamos o estrangeiro: nós o devoramos, digerimos sua carne nutritiva e defecamos seus ossos e impurezas.

Ao analisar uma solução open-source, o agente de IA deve decompor o projeto em duas categorias ontológicas diametralmente opostas:
### 3.1 O Que é "Ouro Puro" (A Matéria Nutritiva)

Buscamos avidamente os órgãos vitais que amplificam a cognição e resolvem problemas de engenharia no silício:
- **Algoritmos Matemáticos Puros:** Equações de decaimento, cálculos bayesianos de utilidade, métricas de calor (_frecency_), grafos acíclicos dirigidos (DAGs) e máquinas de estado determinísticas.
- **Gramáticas e Parsers Estruturais:** Extratores de árvores sintáticas (CST/AST), fatiadores semânticos de escopo, detectores de assinaturas e algoritmos de cálculo de diferenças mínimas (Myers Diff).
- **Estratégias de Poda e Desidratação de Contexto:** Heurísticas de compressão de texto sem perda de significado (_lossless compression_), deduplicação de blocos de código e reidratação ultraveloz em memória RAM.
- **Heurísticas Cognitivas de Redução de Flow-Debt:** Fluxos inovadores de orquestração de pensamento (árvores socráticas, checagem de ambiguidade, refinamento reflexivo de hipóteses).
- **Contratos Estritos e Tipagens Blindadas:** Estruturas de dados canônicas, invariantes de domínio e esquemas relacionais que impedem alucinações.
### 3.2 O Que é "Carcaça Tóxica" (O Resíduo a Ser Incinerado)

Repudiamos com firmeza a complexidade acidental e os vícios da indústria tradicional de software:
- **Camadas de Apresentação e Frameworks de UI:** Componentes React inchados, CSS redundante, Electron, Svelte/Tauri embutidos no binário, interfaces gráficas proprietárias locais.
- **Armazenamento Desorganizado:** Bancos NoSQL não estruturados, bancos vetoriais em Python sem garantias ACID (ChromaDB, FAISS embutido em script), arquivos espalhados em `%APPDATA%` ou caminhos arbitrários.
- **Acoplamento a Serviços de Nuvem Obrigatórios:** Soluções que exigem login social, autenticação remota proprietária ou telemetria invasiva de uso.
- **Proxies de Interceptação Cegos:** Intermediadores de rede que quebram o _prefix caching_ de provedores comerciais ou corrompem streams SSE de tokens.
- **Dependências Circulares e Runtimes Inchados:** Projetos que exigem centenas de pacotes npm ou pip para executar cálculos aritméticos triviais.

## 4. O PRISMA DIALÉTICO DAS TRÊS LENTES DE ANÁLISE

Quando uma LLM for acionada para analisar as essências destiladas de um repositório candidato, ela deve vestir rigorosamente a sua **Lente Designada**, operando como um especialista de visão cirúrgica e dialética.
### 4.1 Lente A: Produto, UX Cognitiva & Flow (O Sentido & A Vontade)
- **Pergunta Fundamental:** _"Como essa solução mitiga a Dívida de Fluxo (_Flow-Debt_) e expande a capacidade executiva do operador neurodivergente?"_
- **Diretrizes de Julgamento:**
    1. **Análise de Valor Prático:** A solução resolve um atrito cognitivo real ou é apenas um malabarismo técnico desprovido de utilidade diária?
    2. **Tradução em Ferramentas e Skills:** Se a ideia for brilhante, como ela se materializa no ecossistema? Ela inspira uma nova **ferramenta MCP** atômica? Dá origem a uma **Skill procedural** canônica que ensina a IA a agir de forma autônoma? Pode inspirar uma visualização analítica no **Hermes Web Dashboard**?
    3. **Economia de Atenção:** A funcionalidade poupa cliques, elimina trocas de contexto manuais e ajuda o humano a permanecer imerso no fluxo criativo?
### 4.2 Lente B: Arquiteto Bare-Metal & Estrutura (A Física & A Transmutação)
- **Pergunta Fundamental:** _"Onde reside a alma matemática desta solução e qual é a fricção exata para transmutá-la em Rust nativo para as 9 crates canônicas?"_
- **Diretrizes de Julgamento:**
    1. **Dissecação Anatômica:** Onde está o algoritmo puro? É possível separar a lógica de negócio da carcaça do framework original (ex: extrair a matemática de um script Python e reescrever em Rust puro)?
    2. **Conformidade Físico-Computacional:** O código transmutado roda com consumo de $0\text{ MB}$ de VRAM? Ele tira proveito de paralelismo Tokio e instruções AVX2 da CPU? Opera com alocação em complexidade assintótica favorável ($\mathcal{O}(1)$ ou $\mathcal{O}(N)$)?
    3. **Ancoragem nas 9 Crates:** Em qual das crates do workspace o órgão canibalizado deve ser instalado (`souls_ast`, `souls_memory`, `souls_core`, etc.)?
### 4.3 Lente C: Operações, Auditoria & FinOps (A Realidade & O Pessimismo)
- **Pergunta Fundamental:** _"Quais são as armadilhas ocultas, a toxicidade de licença e o custo operacional contínuo desta solução?"_
- **Diretrizes de Julgamento:**
    1. **Firewall de Licenças SPDX:** A licença é comercialmente soberana e permissiva (**MIT, Apache-2.0, BSD-3-Clause, ISC**)? Licenças virais ou restritivas (**GPL, AGPL, SSPL, Commons Clause**) disparam rejeição imediata do código-fonte, autorizando apenas a apropriação da ideia conceitual limpa (_clean-room reverse engineering_).
    2. **Eficiência FinOps (Métrica** $E^3$**):** A ferramenta ajuda a poupar tokens de inferência na nuvem? Evita chamadas de APIs externas pagas? Reduz o custo operacional por tarefa de desenvolvimento?
    3. **Manutenibilidade e Aging:** O repositório depende de bibliotecas abandonadas? Introduz vulnerabilidades de segurança (SAST)? Exige manutenção contínua de infraestrutura ou roda de forma autônoma e determinística?

## 5. OS VETORES DE DESTINO DO VALOR CANIBALIZADO

O conhecimento, heurística ou algoritmo extraído de um repositório dissecado não fica solto no vácuo: ele deve ser canalizado compulsoriamente para um dos **cinco vetores canônicos de aterrissagem** do Souls Engine:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MATÉRIA-PRIMA CANIBALIZADA NO REPOSITÓRIO                │
│             (Algoritmos Puros, Heurísticas de Poda, Gramáticas, DDL)        │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                         Transmutação Antropofágica
                                       ▼
 ┌──────────────────┬──────────────────┬──────────────────┬──────────────────┐
 │                  │                  │                  │                  │
 ▼                  ▼                  ▼                  ▼                  ▼
[VETOR 1]          [VETOR 2]          [VETOR 3]          [VETOR 4]          [VETOR 5]
Módulo em Crate    Garra MCP          Playbook de Skill  Aba de Extensão    Estrutura DDL
Rust (9 Crates)    Soberana (:9123)   Procedural         Hermes Dashboard   Hipocampal L3
(ex: souls_ast)    (ex: outline)      (.agents/skills)   (React IIFE)       (STRICT / Lance)
```

1. **Vetor 1: Módulos das 9 Crates Canônicas:**
    - Absorção direta no código-fonte em Rust nativo sob `#![forbid(unsafe_code)]`.
    - Alocação na crate correta conforme as fronteiras do `CRATES_SPECIFICATION.md`.
2. **Vetor 2: Novas Ferramentas MCP (`souls_mcp`):**
    - Exposição como função atômica no catálogo MCP servido pelo daemon Axum em `127.0.0.1:9123/mcp`.
    - Envelopes JSON-RPC 2.0 estritos, parâmetros tipados e integração ao mecanismo de revelação progressiva (`tool_search`).
3. **Vetor 3: Skills Procedurais Canônicas:**
    - Se a inovação for uma metodologia de trabalho ou fluxo de raciocínio, ela é formalizada como um arquivo de Skill (em `.agents/skills/` ou `$HERMES_HOME/skills/`), ensinando a IA a encadear ferramentas existentes com sabedoria.
4. **Vetor 4: Extensões do Hermes Web Dashboard:**
    - Se a inovação envolver visualizações de telemetria, gráficos de hardware ou governança de estado, ela aterrissa como endpoint FastAPI (`dashboard/plugin_api.py`) e componente React compilado no formato IIFE consumindo `window.__HERMES_PLUGIN_SDK__`.
5. **Vetor 5: Estrutura Hipocampal e DDL de Memória:**
    - Se a solução envolver novas formas de indexação, relacionamentos conceituais ou persistência de contexto, ela é transmutada em esquemas relacionais no FrankenSQLite STRICT WAL ou tabelas colunares Apache Arrow no LanceDB.

## 6. O SCORECARD ANTROPOFÁGICO E SÍNTESE EXECUTIVA

Ao concluir a análise das essências destiladas de um projeto sob sua respectiva lente, o agente deve formular seu julgamento estruturado sobre as quatro dimensões canônicas, atribuindo notas de $1.0$ a $5.0$:

| **Dimensão de Avaliação**                  | **Nota (1.0 a 5.0)** | **Critério de Excelência (Nota 5.0)**                                                                                                        | **Critério de Rejeição (Nota 1.0)**                                                                                           |
| ------------------------------------------ | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Desbloqueio de Flow (**$V_f$**)**        | $1.0 \dots 5.0$      | Mitiga diretamente a fadiga executiva, alivia a sobrecarga de memória de trabalho e cria um atalho cognitivo evidente para neurodivergentes. | Utilitário irrelevante, adiciona mais atrito operacional do que resolve, ou é mero preciosismo cosmético.                     |
| **Pureza Algorítmica (**$P_a$**)**         | $1.0 \dots 5.0$      | A lógica central é matematicamente limpa, independente de bibliotecas pesadas e isolável em funções puras com estado explícito.              | Código espaguete profundamente entrelaçado com frameworks web, ORMs lentos ou chamadas de sistema proprietárias.              |
| **Viabilidade de Transplante (**$T_r$**)** | $1.0 \dots 5.0$      | Facilmente recompilável ou reescrevível em Rust assíncrono em menos de 300 linhas de código nativo; zero C-FFI arriscado.                    | Exige emular ecossistemas inteiros (Python/Node), consome memória de forma descontrolada ou requer drivers proprietários.     |
| **Soberania FinOps & Silício (**$S_s$**)** | $1.0 \dots 5.0$      | Opera com 0 MB de VRAM, poupa chamadas de modelos caros de nuvem ($E^3$ elevado) e possui licença estritamente permissiva (MIT/Apache).      | Requer GPUs corporativas de alta VRAM, impõe pagamentos contínuos de tokens de terceiros ou carrega licença viral (AGPL/GPL). |

### 6.1 As Quatro Decisões Executivas Possíveis

Com base na ponderação das quatro notas, a análise deve culminar em um **Veredito Canônico Inequívoco**:
1. **`ASSIMILAÇÃO_DIRETA`:**
    - _Critério:_ $V_f \ge 4.0$, $P_a \ge 4.0$, $T_r \ge 4.0$ e Licença Permissiva Aprovada.
    - _Ação:_ O órgão algorítmico deve ser extraído cirurgicamente, reescrito em Rust e integrado imediatamente a uma das 9 crates canônicas.
2. **`TRANSPLANTE_CIRÚRGICO_REFATORADO`:**
    - _Critério:_ Valor de Flow elevado ($V_f \ge 4.0$), mas a pureza ou facilidade de transplante exige isolamento (código acoplado).
    - _Ação:_ A lógica de negócios é isolada em um módulo independente ou compilada temporariamente em Wasm/Sidecar efêmero, planejando-se sua purificação gradual.
3. **`INSPIRAÇÃO_CONCEITUAL_CLEAN_ROOM`:**
    - _Critério:_ A ideia ou heurística de UX é genial, mas o código original é biohazard (licença GPL viral, dependência crônica de Electron ou lixo de dependências).
    - _Ação:_ O código-fonte original é 100% rejeitado e incinerado. A heurística conceitual é reescrita do zero (_clean-room_) sob a forma de uma Skill procedural ou ferramenta MCP pura.
4. **`REJEIÇÃO_E_INCINERAÇÃO`:**
    - _Critério:_ Falha em demonstrar mitigação real de Flow-Debt ($V_f < 2.5$) ou violação intolerável dos limites de silício e FinOps.
    - _Ação:_ O repositório é sumariamente descartado. Nenhum byte ou conceito entra no ecossistema do Souls Engine.