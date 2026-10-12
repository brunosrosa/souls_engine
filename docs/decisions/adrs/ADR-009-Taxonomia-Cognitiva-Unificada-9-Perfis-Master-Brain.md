# ADR-009 — Taxonomia Cognitiva Unificada de 9 Facetas, Avaliação do Master Brain e Simbiose Cloud-Silício

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** Outubro de 2026 (Linha de Base v7 Canônica)  
> **CRATES DIRETAMENTE REGIDAS:** `souls_llm_cloud_arena`, `souls_llm_local_arena`, `souls_model_router`, `souls_protocol`, `souls_server`.  
> **ESPECIFICAÇÕES CORRELATAS:** `ADR-008`, `SOULS_CANON_MANIFEST.md`, `llm_benchmark_cost_matrix_crate_specification.md`, `MCP_TOOL_CONTRACTS.md`.  
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code, Trae, Hermes Agent) e Engenheiros de Sistemas.

---

## 1. CONTEXTO E FORÇAS EM CONFLITO

No desenho inicial da arquitetura agêntica e de benchmarking do Souls Engine (ADR-008), o sistema considerou primariamente modelos como **executores pontuais de tarefas operacionais** (coding, MCP JSON tools, raciocínio lógico formal e triagem de alta taxa de transferência).

Entretanto, uma análise mais profunda da simbiose com o **Hermes Agent** e dos princípios fundamentais do [SOULS_CANON_MANIFEST.md](file:///Z:/souls_engine/docs/planning/SOULS_CANON_MANIFEST.md) revelou três lacunas conceituais e operacionais críticas:

1. **A Assimetria do "Master Brain" (A Voz e Mente Principal):**  
   O modelo que atua no chat interativo principal com o Arquiteto Humano não é um operário de código. Ele atua como **Prótese de Função Executiva** e **Parceiro de Sparring Cognitivo** para mitigar o *Flow-Debt* do operador humano neurodivergente (TDAH/2e). Se esse modelo for selecionado puramente por métricas de menor custo por token ($/1M) ou velocidade bruta ($TPS$), ele alucinará diretrizes, falhará em seguir guardrails complexos de governança ([AGENTS.md](file:///Z:/souls_engine/AGENTS.md)) e causará sobrecarga de decisão ao humano.
2. **Facetas Cognitivas Não Mapeadas ("O Etc."):**  
   O desenvolvimento avançado exige mais do que apenas código e chamadas de ferramenta simples. São necessárias capacidades especializadas como:
   * **Research & Deep Synthesis:** Ingestão de repositórios gigantescos, bibliotecas externas e papers densos sem degradação de atenção em janelas de 128k a 1M tokens.
   * **Security & Code Audit:** Detecção de vulnerabilidades sutis, race conditions e memory leaks antes de commits.
   * **Technical Writing & Obsidian Curator:** Síntese didática, redação de manuais técnicos bilingues e curadoria estruturada da base de conhecimento Obsidian DB (`C:\Users\rosas\OneDrive\Documentos\Obisidian DB`).
3. **Desalinhamento Semântico entre Cloud e Silício Local:**  
   Enquanto a `souls_llm_cloud_arena` avalia APIs de nuvem e a `souls_llm_local_arena` avalia pesos GGUF na RTX 2060m/Intel i9, o despachante de subagentes ([`souls_model_router`](file:///Z:/souls_engine/crates/souls_model_router)) necessita de uma **mesma taxonomia semântica de perfis** para saber se uma subtarefa delegada deve rodar no silício local ou na nuvem.

---

## 2. DECISÃO ARQUITETURAL

Fica formalmente estabelecida a **Taxonomia Cognitiva Unificada de 9 Facetas**, harmonizada entre nuvem e silício bare-metal:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│             TAXONOMIA COGNITIVA UNIFICADA (SOULS ENGINE v7)                 │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
     ┌─────────────────────────────────┴─────────────────────────────────┐
     ▼                                                                   ▼
┌───────────────────────────┐                       ┌───────────────────────────┐
│     O MASTER BRAIN        │                       │    WORKERS ESPECIALIZADOS │
│   (Voz Mestre / Hermes)   │                       │ (Subagentes Delegados)    │
│  - Fidelidade a Regras    │                       │  - Executores de Escopo   │
│  - Decomposição Agêntica  │                       │  - Critério: Alto ROI     │
│  - Sparring & Empatia     │                       │  - Local (dGPU) ou Nuvem  │
│  - Long-Context Retenção  │                       └───────────────────────────┘
└───────────────────────────┘
```

---

### 2.1 As 9 Facetas Cognitivas Canônicas

| # | Identificador (`TaskProfileKind`) | Nome Canônico | Alvo Operacional Primário | Benchmarks Ponderados Chave |
|---|---|---|---|---|
| **1** | `executive` | **Executive / Master Brain** | Modelo padrão do chat mestre do Hermes. Orquestração, planejamento, diálogo de parceria e sparring. | Hard Elo (30%), IFEval (25%), BFCL (20%), GPQA (15%), Context Coherence (10%). |
| **2** | `coding` | **Coding & SWE** | Geração de código Rust/Python, refatoração de AST e patches de alta precisão. | Coding Elo (40%), BFCL (25%), Hard Elo (20%), IFEval (15%). |
| **3** | `agentic` | **Autonomous Agentic** | Execução de loops de ferramentas MCP, emissão JSON estrita e encadeamento autônomo. | BFCL (45%), IFEval (30%), Hard Elo (15%), TPS (10%). |
| **4** | `reasoning` | **Deep Reasoning** | Raciocínio matemático, provas lógicas, deduções formais e arquitetura de concorrência. | GPQA Diamond (45%), Hard Elo (35%), IFEval (20%). |
| **5** | `fast` | **Fast / Triage** | Triagem ultrarrápida de intenção, logit probing, gating heurístico e streaming instantâneo. | TPS Normalizado (40%), TTFT (30%), IFEval (15%), Coding Elo (15%). |
| **6** | `design` | **Design & Multimodal UI** | Geração e inspeção de UIs web modernas, visual engineering, design tokens e SVG. | Frontend/SWE (40%), VQA Multimodal (30%), IFEval (15%), Throughput (15%). |
| **7** | `research` | **Research & Deep Synthesis** | Leitura densa de documentações massivas, extração de conhecimento e RAG longo. | Context NIAH/RULER (40%), Summarization Elo (30%), Hard Elo (20%), IFEval (10%). |
| **8** | `security` | **Security & Code Audit** | Análise estática, detecção de vulnerabilidades de memória, race conditions e conformidade. | CyberMetric/SecEval (45%), Diff Reasoning (30%), Hard Elo (15%), IFEval (10%). |
| **9** | `curation` | **Technical Writing & Obsidian Curator** | Redação técnica impecável, curadoria de wikis/Obsidian DB, manuais e escrita bilingue. | Writing Elo (40%), MT-Bench Format (30%), IFEval (20%), TTFT (10%). |

---

### 2.2 Critério de Avaliação Diferenciado para o Master Brain

Para o perfil **`Executive / Master Brain`**, a métrica de escolha **rejeita o minimalismo financeiro cego**:
* Um modelo que custa \$0.15/1M mas comete um deslize arquitetural ou falha em seguir uma regra do [AGENTS.md](file:///Z:/souls_engine/AGENTS.md) causa horas de retrabalho e fadiga ao operador humano.
* A Fronteira de Pareto para o `Executive` adota amortecimento logarítmico ultra-suave sobre o custo, garantindo que apenas **modelos de fronteira cognitiva (Frontier Tier)** com pontuação de qualidade composta $S_{\text{executive}} \ge 90.0$ sejam elegíveis como Campeões de Liderança.

---

### 2.3 Harmonização entre Nuvem e Silício Local

1. **O Master Brain é de Nuvem (Nous Research Portal / Provedor Oficial):**  
   Respeitando a Lei Negativa #2 do [AGENTS.md](file:///Z:/souls_engine/AGENTS.md), o chat interativo mestre conecta-se diretamente à API de nuvem configurada no Hermes Agent. A `souls_llm_cloud_arena` fornece recomendações e ranking para que o Hermes Agent saiba qual modelo definir como default.
2. **Subagentes Delegados Consomem a Mesma Taxonomia:**  
   Quando o Master Brain invoca `souls_route_subtask(task_profile="coding" | "curation" | "fast")`, o `souls_model_router` consulta ambas as arenas:
   * Se o silício local (RTX 2060m) possuir um modelo não-dominado na Fronteira de Pareto com VRAM suficiente, ele é ativado com custo \$0.00.
   * Se a tarefa demandar raciocínio profundo ou contexto que exceda a dGPU, a rota é delegada para o campeão de nuvem daquela faceta.

---

## 3. CONSEQUÊNCIAS E GOVERNANÇA

1. **Preservação da Imutabilidade do Diálogo:**  
   Fica terminantemente proibido qualquer chaveamento dinâmico intradiálogo no chat mestre. A recomendação do `Executive Champion` atua na inicialização, configuração de perfis e no futuro Web Dashboard do Hermes.
2. **Exposição via MCP e REST:**  
   A ferramenta MCP `cloud_champions` e o endpoint REST `/api/v1/cloud/champions` aceitam o parâmetro `profile` mapeado para qualquer uma das 9 facetas, permitindo ao Hermes e à IDE consultarem especialistas dedicados para cada papel.
3. **Zero Poluição de Código:**  
   As 9 facetas representam refinamento puramente analítico e semântico, sem demandar novas crates ou alterações na topologia física de 10 crates canônicas.
