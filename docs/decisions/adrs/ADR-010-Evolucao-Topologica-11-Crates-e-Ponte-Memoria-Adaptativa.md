# ADR-010 — Evolução Topológica para 11 Crates Canônicas e Ponte Bidirecional de Memória Adaptativa

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** 29 de Outubro de 2026 (Linha de Base v7 Canônica)  
> **CRATES DIRETAMENTE REGIDAS:** `souls_protocol`, `souls_memory`, `souls_inference_runtime`, `souls_llm_inventory`, `souls_server`.  
> **ESPECIFICAÇÕES CORRELATAS:** `CRATES_SPECIFICATION.md`, `ADR-007`, `ADR-008`, `ADR-009`, `SOULS_CANON_MANIFEST.md`, `item_a_memory_bridge_plan.md`.  
> **APLICAÇÃO:** Todos os Agentes de IA e Engenheiros de Sistemas do Souls Engine.

---

## 1. CONTEXTO E MOTIVAÇÃO

O workspace inicial do Souls Engine v7 foi concebido com 9 crates canônicas e posteriormente expandido para 10 crates canônicas através da inclusão da `souls_llm_cloud_arena` (ADR-008 e ADR-009).

Ao longo do avanço das baterias de testes e da integração física com o ecossistema do **Hermes Agent** e da máquina bare-metal (Intel Core i9-9980HK, 32GB RAM e NVIDIA GeForce RTX 2060m com teto útil de 5.294 MB no WDDM), duas necessidades de primeira grandeza surgiram:

1. **Gestão Segura de Inventário de Silício e Downloader Resumível (RFC-007):**
   A aquisição, verificação criptográfica (SHA-256) e inspeção de compatibilidade física de modelos locais GGUF e bundles VLM (`mmproj-F16.gguf`) exigiam um subsistema isolado com chamadas seguras à API NT (`GlobalMemoryStatusEx`), verificação de VRAM via NVML e download HTTP sequencial com suporte a `.part` e Range headers. Misturar essa complexidade de rede e arquivos na `souls_llm_local_arena` violaria o Princípio da Responsabilidade Única e aumentaria o raio de impacto de compilação.
2. **Ponte Bidirecional de Memória L3 e Recall Adaptativo ("Terceira Via"):**
   O Hermes Agent necessitava de soberania absoluta sobre seus arquivos primários (`MEMORY.md`, `USER.md` e `SOUL.md`), enquanto consumia o hipocampo bare-metal L2/L3 (`souls_memory.db` no SQLite STRICT WAL e Grafo Ontológico LadybugDB). A introdução de um *Snapshot Congelado por Sessão* (`FrozenMemorySnapshot`) protege o *prefix caching* do LLM de fronteira (>95% de retenção), enquanto o **Recall Adaptativo** atende com perfeição à cognição neurodivergente do operador humano (TDAH + AH/SD), alternando entre modo enxuto e expandido sob demanda.

---

## 2. DECISÕES ARQUITETURAIS

### 2.1 Formalização da Topologia Canônica de 11 Crates

Fica ratificado que o workspace do Souls Engine é composto por **exatamente 11 crates canônicas**:

```
                              ┌──────────────────┐
                              │  souls_protocol  │ (Nível 0: Contratos Puros)
                              └────────┬─────────┘
                                       │
                    ┌──────────────────┴──────────────────┐
                    ▼                                     ▼
           ┌─────────────────┐               ┌─────────────────────────┐
           │   souls_core    │               │ souls_inference_runtime │ (Nível 1 & 2)
           └────────┬────────┘               └────────────┬────────────┘
                    │                                     │
       ┌────────────┼────────────┐           ┌────────────┼────────────┐
       ▼            ▼            ▼           ▼            ▼            ▼
 ┌───────────┐┌───────────┐┌───────────┐┌──────────┐┌───────────┐┌────────────┐
 │ souls_ast ││souls_memor││cloud_arena││local_aren││ inventory ││anthropophag│
 └─────┬─────┘└─────┬─────┘└─────┬─────┘└────┬─────┘└─────┬─────┘└─────┬──────┘
       │            │            │           │            │            │
       └────────────┼────────────┴───────────┼────────────┴────────────┘
                    ▼                        │
          ┌────────────────────┐             │
          │ souls_model_router │◄────────────┘
          └─────────┬──────────┘
                    │
                    ▼
          ┌────────────────────┐
          │    souls_server    │ (Nível 4: Daemon Headless Axum :9123)
          └────────────────────┘
```

1. `souls_protocol`: Contratos agnósticos, envelopes JSON-RPC 2.0 e DTOs.
2. `souls_core`: Serviços bare-metal, redator PII reversível (Aho-Corasick), CoW ReFS e subprocessos.
3. `souls_ast`: Árvores sintáticas Tree-sitter, Myers Diff, Frecency e Gitoxide.
4. `souls_memory`: FrankenSQLite STRICT WAL, LanceDB/Arrow, LadybugDB e Ponte L3.
5. `souls_inference_runtime`: NVML Watchdog, ASUS ATKACPI Ioctl, llama.cpp CUDA e Tiers 0/0.5/1.
6. `souls_model_router`: ParetoBandit multiobjetivo, barreira termodinâmica e FinOps.
7. `souls_llm_local_arena`: 5 baterias de benchmark em silício local e banco empírico de heurísticas.
8. `souls_llm_cloud_arena`: Ingestão de mercados de nuvem (Nous, OpenRouter, AA) e 9 perfis cognitivos.
9. `souls_llm_inventory`: Profiler de hardware NT, Downloader FIFO com SHA-256 e HuggingFace Hub.
10. `souls_anthropophagy`: Dissecação poliglota, SAST e destilação de repositórios (em transplante).
11. `souls_server`: Daemon executável headless Axum (`127.0.0.1:9123`), Switchboard e Web Dashboard.

### 2.2 Especificação da 11ª Crate: `souls_llm_inventory`

A crate `souls_llm_inventory` tem como responsabilidades exclusivas:
- **Hardware Fit Profiler:** Medição exata da memória física disponível via `GlobalMemoryStatusEx` e VRAM útil via NVML C-FFI, classificando modelos como `FitGuaranteed`, `TightFit` ou `ExceedsResources`.
- **Download Manager Resumível:** Downloads sequenciais via HTTP Range requests gravando em extensões `.part` com buffer circular de 64KB e hash SHA-256 incremental.
- **Gestão de Bundles Multimodais (VLM):** Tratamento pareado de pesos do modelo principal e projetor visual (`mmproj-F16.gguf`).
- **Isolamento de Concorrência:** Downloads são limitados a 1 job ativo por vez para preservar a largura de banda do host e evitar contenção de I/O no disco NVMe.

### 2.3 Contrato da Ponte Bidirecional de Memória & Recall Adaptativo ("Terceira Via")

Fica estabelecido o protocolo de simbiose de memória com o Hermes Agent:
1. **Soberania e Snapshot Congelado:**
   - O Hermes Agent mantém a soberania de escrita em `MEMORY.md` e `USER.md`.
   - O Souls Engine lê passivamente esses arquivos no cold boot através do `HermesBootstrapFiles`.
   - No início de cada diálogo (`session_id`), o `SnapshotCoordinator` congela um snapshot estático na RAM. O modelo de chat recebe o mesmo texto exato de memória em todos os turnos, assegurando taxa de acerto de *prefix caching* >95%.
2. **Recall Adaptativo para TDAH + AH/SD:**
   - **Modo Enxuto (`lean`):** Injeta apenas âncoras essenciais e verdades epistêmicas estáveis (~800 caracteres), mantendo o foco cognitivo hiper-direcionado e livre de poluição de contexto.
   - **Modo Expandido (`expanded`):** Sob demanda do Arquiteto ou quando o `intent` registrar alta entropia exploratória, expande nós causais e arestas do Grafo LadybugDB (~2400 caracteres), permitindo conexões multidisciplinares amplas.
3. **Segurança de PII:** Toda busca de memória que atravesse para fora do ambiente isolado é compulsoriamente higienizada pelo `PiiRedactor` bare-metal antes do consumo.

---

## 3. CONSEQUÊNCIAS E CONFORMIDADE

- **Positivas:**
  - Código de download e profiling isolado em crate dedicada, reduzindo o tempo de compilação incremental de outras partes do sistema.
  - Zero risco de corrupção de downloads em conexões instáveis.
  - Mitigação de fadiga de decisão e desvio de foco no Arquiteto Humano, garantindo consistência no comportamento do modelo de diálogo.
- **Rigor de Governança:**
  - Nenhuma crate além de `souls_llm_inventory` e `souls_llm_cloud_arena` pode instanciar clientes HTTP externos.
  - O teto de 11 crates é agora a **nova linha de base imutável** do Souls Engine v7.
