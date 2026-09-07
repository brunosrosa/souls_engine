# ADR-006 — Metabolismo Epistêmico Noturno, Segregação Ontológica (`STABLE` vs `EVOLVING`) e Langevin Decay no Chyros Daemon

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_memory`, `souls_protocol`, `souls_core`, `souls_server`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

Sistemas convencionais de RAG (_Retrieval-Augmented Generation_) e motores de persistência agêntica sofrem cronicamente do fenômeno de **entropia informacional e envenenamento contextual (**_**Context Rot / Memory Dilution**_**)**:
1. **A Amnésia Inversa por Acúmulo Indiscriminado:** À medida que um agente autônomo opera ao longo de semanas ou meses, o banco vetorial e relacional acumula centenas de fragmentos textuais transitórios (ex: suposições provisórias, rascunhos de depuração, respostas a erros de sintaxe efêmeros e logs parciais).
2. **Degradação de Precisão no kNN e BM25:** Em uma base de dados saturada de ruído, consultas semânticas densas sofrem distorção de proximidade. O espaço vetorial fica poluído por nós irrelevantes que competem diretamente com decisões arquiteturais críticas, reduzindo a relevância da Fusão Recíproca de Ranqueamento (RRF) e induzindo o agente a alucinações repetitivas.
3. **Rigidez vs. Fluidez Ontológica:** Em projetos de engenharia de software bare-metal, coexistem dois tipos fundamentais de conhecimento com dinâmicas temporais opostas:
    - **Conhecimento Axiomático/Estrutural:** Decisões arquiteturais (ADRs), regras de diretórios, convenções de código e esquemas de dados, cuja validade independe do tempo e não pode sofrer expurgo automático.
    - **Conhecimento Circunstancial/Evolutivo:** Observações de depuração, notas de trabalho de um turno específico, hipóteses socráticas temporárias e métricas transitórias, que perdem relevância rapidamente caso não sejam reforçadas.
4. **O Desastre da Desfragmentação Bloqueante em Disco:** Em bancos SQLite operando sob o kernel Windows NT em regime contínuo, a execução síncrona do comando `VACUUM` inline durante sessões ativas bloqueia completamente a escrita (`SQLITE_LOCKED`), descarta caches de páginas na RAM e causa picos de I/O no NVMe, fragmentando alocações no Dev Drive ReFS e degradando a latência do daemon.
5. **A Inviabilidade de Vector Stores Descartáveis:** Soluções prontas baseadas em Python (Chroma, FAISS, Pinecone local) não oferecem garantias transacionais ACID, geram vazamentos de memória crônicos em processos de longa duração e não se integram atomicamente com tabelas relacionais indexadas por FTS5 e grafos em memória.

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretada a **adoção do Metabolismo Epistêmico Noturno governado pelo Chyros Daemon na crate `souls_memory`, implementando Segregação Ontológica Bipartida (`STABLE` vs `EVOLVING`), decaimento estocástico térmico via Equação de Langevin e desfragmentação atômica assíncrona por replicação (`VACUUM INTO`) no Dev Drive ReFS**.

O subsistema de memória do Souls Engine assume a gestão ativa do ciclo de vida biológico das memórias do sistema, subordinando a retenção e o expurgo aos princípios matemáticos e invariantes de integridade descritos a seguir.

### 2.1 Segregação Ontológica Bipartida: Partição `STABLE` vs Partição `EVOLVING`

Todo e qualquer registro inserido no Hipocampo L3 (`epistemic_memories` no SQLite e `epistemic_vectors` no LanceDB) deve ser categorizado compulsoriamente em uma das duas partições ontológicas imutáveis:

| **Dimensão de Governança**     | **Partição STABLE (Axiomática)**                                                                         | **Partição EVOLVING (Circunstancial)**                                                                           |
| ------------------------------ | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| **Natureza Epistêmica**        | Decisões arquiteturais (ADRs), contratos de API, schemas DDL, regras de governança e perfis permanentes. | Rascunhos de sessões, notas de depuração, hipóteses socráticas provisórias e saídas transitórias de ferramentas. |
| **Saliência Base (**$S_m$**)** | Fixada perenemente em $S_m \equiv 1.0$.                                                                  | Dinâmica e normalizada no intervalo $S_m \in [0.0, 1.0]$.                                                        |
| **Decaimento Temporal**        | **IMUNIDADE ABSOLUTA.** Taxa de decaimento estritamente $\gamma = 0$.                                    | Sujeita ao decaimento contínuo estocástico via Equação de Langevin.                                              |
| **Expurgo Automatizado**       | **TERMINANTEMENTE PROIBIDO.** Nunca é deletada pelo daemon noturno.                                      | Expurgo físico atômico quando $S_m(t) < 0.15$.                                                                   |
| **Mecanismo de Promoção**      | Promovida explicitamente por ação HITL (humano) ou ferramenta MCP `knowledge` formal.                    | Estado inicial padrão para memórias capturadas passivamente pelo gancho `sync_turn`.                             |

### 2.2 Formulação Estocástica da Equação de Langevin

Para as memórias registradas na partição `EVOLVING`, o Chyros Daemon modela a relevância informacional não como um decaimento linear simplista, mas como uma partícula browniana sujeita a atrito determinístico e flutuações térmicas estocásticas:
$$\frac{d S_m(t)}{dt} = -\gamma \cdot S_m(t) + \sigma \cdot \xi(t)$$
Onde:
- $S_m(t) \in [0.0, 1.0]$: Saliência epistêmica da memória $m$ no instante $t$.
- $\gamma$: Coeficiente de amortecimento determinístico, modulado inversamente pela frequência de reforço de acesso:
$$\gamma = \frac{1}{\tau \cdot (\text{access\_count} + 1)}$$
- $\tau$: Constante de tempo de meia-vida básica calibrada para $\tau = 7.0\text{ dias}$. Cada invocação ou recuperação da memória em buscas híbridas incrementa `access_count`, estendendo exponencialmente sua longevidade no sistema.
- $\sigma \cdot \xi(t)$: Termo de difusão estocástica modelando ressonâncias semânticas indiretas e associações conceituais latentes, onde $\xi(t) \sim \mathcal{N}(0, 1)$ é ruído branco gaussiano com desvio padrão calibrado para $\sigma = 0.02$.

#### Discretização Temporal para o Ciclo Noturno ($\Delta t = 1\text{ dia}$)

A cada ciclo noturno de manutenção disparado pelo daemon, a atualização do índice de saliência para cada memória evolutiva é calculada pela solução em diferenças finitas:
$$S_m(t + \Delta t) = S_m(t) \cdot \exp\left(-\frac{\Delta t}{\tau \cdot (\text{access\_count} + 1)}\right) + \mathcal{N}(0, \sigma^2)$$

Com truncamento defensivo nos limites físicos: $S_m \leftarrow \max(0.0, \min(1.0, S_m))$.

### 2.3 O Limiar Crítico de Expurgo ($S_m < 0.15$) e a Purga em Duas Etapas

Se, após o decaimento termodinâmico, a saliência de uma memória da partição `EVOLVING` cruzar o piso crítico:
$$S_m(t + \Delta t) < 0.15$$
O registro é classificado como "ruído degenerado" e o Chyros Daemon executa compulsoriamente a **Purga Atômica Coordenada em Duas Etapas**:

1. **Etapa Relacional e Léxica (FrankenSQLite + FTS5):**
    ```
    -- A exclusão atômica dispara automaticamente os triggers de sincronização FTS5
    DELETE FROM epistemic_memories WHERE id = :id AND partition = 'EVOLVING';
    ```
    Os triggers `trg_epistemic_memories_ad` definidos no DDL eliminam imediatamente as entradas do índice invertido FTS5 (`epistemic_memories_fts`), expurgando termos obsoletos da busca BM25.
2. **Etapa Vetorial Colunar (LanceDB Arrow):**
    A thread assíncrona invoca a mutação na tabela LanceDB:
    ```
    lance_table.delete(&format!("id = '{}'", id_expurgado)).await?;
    ```
    Garantindo a remoção do tensor de embedding de dimensão 384 e restaurando a pureza dos índices de produto escalar e cosseno.

### 2.4 Protocolo de Desfragmentação sem Fragmentar o Dev Drive ReFS (`VACUUM INTO`)

No Windows 11 sobre Dev Drive ReFS, a invocação direta do comando `VACUUM` tradicional é proibida por reescrever o arquivo em si com locks exclusivos de longa duração.

Para desfragmentar páginas relacionais sem degradar a pilha de I/O do ReFS:
1. **Janela de Ociosidade Térmica e de CPU:** O Chyros Daemon só é acionado quando o escalonador Tokio detectar inatividade do usuário superior a 30 minutos entre 02:00 e 05:00 da manhã, com o monitor NVML indicando $T_{\text{GPU}} < 50^\circ\text{C}$ e CPU Intel i9 com utilização basal inferior a $5\%$.
2. **Replicação Desfragmentada sem Bloqueio de Leitura:** O motor invoca o snapshot compacto atômico:
    ```
    VACUUM INTO 'Z:\souls_engine\.souls_data\db\souls_state_compact.db';
    ```
    Essa operação constrói uma cópia integral, contígua e perfeitamente desfragmentada em segundo plano, sem interromper as leituras ativas concorrentes do Hermes Agent no arquivo principal `souls_state.db`.
3. **Substituição Atômica no Kernel Win32:** Após a conclusão bem-sucedida da compactação, os descritores são sincronizados e o arquivo original é substituído atomicamente utilizando primitivos Win32 da `windows-sys`:
    ```
    // Rotação atômica sob o kernel Windows NT
    MoveFileExW(
        compact_path_wide.as_ptr(),
        master_path_wide.as_ptr(),
        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
    );
    ```

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CHYROS DAEMON (Ocioso Noturno: 03:00)                    │
│          Gatilho: [Inatividade > 30min] E [CPU < 5%] E [GPU < 50°C]         │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                    Itera memórias epistêmicas L3
                                       ▼
                     ┌───────────────────────────────────┐
                     │   Qual é a Partição Ontológica?   │
                     └─┬───────────────────────────────┬─┘
                       │                               │
             `partition == 'STABLE'`         `partition == 'EVOLVING'`
                       │                               │
                       ▼                               ▼
        ┌─────────────────────────────┐  ┌─────────────────────────────┐
        │     IMUNIDADE ABSOLUTA      │  │     LANGEVIN DECAY E3       │
        │      Salience = 1.0         │  │   Sm(t+Δt) = Sm·e^(-Δt/τ)   │
        │   NENHUMA ação ou purga     │  │       + N(0, σ²)            │
        └─────────────────────────────┘  └──────────────┬──────────────┘
                                                        │
                                        Sm < 0.15 ? ────┼──── Sm >= 0.15
                                       (Sim)            │        (Não)
                                        │               │          │
                                        ▼               │          ▼
                        ┌───────────────────────────────┴┐  ┌──────────────┐
                        │      PURGA COORDENADA EM 2     │  │ Atualiza Sm  │
                        │   1. DELETE SQLite (FTS5 trg)  │  │ no SQLite e  │
                        │   2. DELETE LanceDB (Arrow)    │  │ segue ativo  │
                        └───────────────┬────────────────┘  └──────────────┘
                                        │
                         Varredura concluída com sucesso
                                        ▼
                        ┌────────────────────────────────┐
                        │     VACUUM INTO COMPACT.DB     │
                        │   + Rotação atômica Win32 NT   │
                        │  (MoveFileExW REPLACE_EXISTING)│
                        └────────────────────────────────┘
```

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Erradicação do** _**Context Rot**_ **e Amnésia Inversa:** O expurgo biológico contínuo impede que o banco de dados se transforme em um depósito de rascunhos obsoletos, garantindo que a recuperação semântica RRF mantenha taxas de precisão elevadas ao longo de anos de uso.
- **Preservação Soberana das Decisões de Arquitetura:** Ao atribuir imunidade absoluta à partição `STABLE`, o sistema garante que regras fundamentais, contratos e ADRs nunca sejam esquecidos ou diluídos por dados transitórios.
- **Índices Vetoriais Compactos e Ultravelozes:** A eliminação de tensores degenerados no LanceDB preserva o tamanho contíguo do arquivo colunar Arrow, permitindo que as buscas de cosseno operem consistentemente em menos de $8\text{ ms}$ na memória RAM física (com rigorosos $0\text{ MB}$ de VRAM alocados).
- **Zero Contenção de I/O em Horário de Produção:** Ao delegar manutenções estruturais pesadas (`VACUUM INTO`) para períodos noturnos ociosos, as sessões de desenvolvimento diurnas com o operador humano operam com a latência de I/O pura do Dev Drive ReFS.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**                             | **Impacto Técnico**                                                                                  | **Mitigação Canônica Implementada**                                                                                                                                                                                                           |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Risco Teórico de Expurgo de Memória Útil**             | Uma nota evolutiva útil que passe 30 dias sem ser acessada pode cair abaixo de $0.15$ e ser apagada. | Todo acesso ou reforço em buscas (`semantic_search`, `mem_open_nodes`) incrementa `access_count`, resetando a meia-vida $\tau$; notas de valor permanente devem ser promovidas compulsoriamente para a partição `STABLE` via MCP `knowledge`. |
| **Computação Noturna com Máquina Desligada**             | Se o operador suspender ou desligar o notebook durante a noite, o ciclo noturno não dispara.         | Validação defensiva no primeiro boot diurno: se `last_chyros_run` tiver mais de 48 horas, o daemon agenda uma execução em segundo plano com baixa prioridade de thread (`THREAD_PRIORITY_BELOW_NORMAL` via `windows-sys`).                    |
| **Sobrecarga Temporária de Disco Durante `VACUUM INTO`** | Duplicação momentânea do espaço do banco `souls_state.db` durante a compactação.                     | O Dev Drive ReFS opera em partição dedicada com teto mínimo provisionado de $30\text{ GB}$, e o arquivo compactado raramente excede centenas de megabytes.                                                                                    |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE EXPURGO OU DECAIMENTO EM REGISTROS `STABLE`:** É categoricamente proibido emitir comandos `DELETE` ou calcular decaimento de Langevin sobre registros onde `partition = 'STABLE'`. A partição estável é inviolável por processos automatizados.
2. **PROIBIÇÃO DE INSERÇÃO SEM CLASSIFICAÇÃO EXPLÍCITA:** Nenhuma struct ou função em `souls_memory` ou `souls_protocol` pode omitir o campo `partition`. A tentativa de salvar memórias com partição nula ou dinâmica inválida dispara erro fatal em tempo de compilação ou rejeição imediata pela constraint `CHECK(partition IN ('STABLE', 'EVOLVING'))` do SQLite STRICT.
3. **PROIBIÇÃO DE `VACUUM` INLINE EM REQUISIÇÕES DIURNAS:** É estritamente proibido invocar `VACUUM` síncrono nas rotinas de rota de API, nos manipuladores MCP ou no encerramento ordinário do servidor. A compactação relacional é monopólio exclusivo do Chyros Daemon via `VACUUM INTO`.
4. **PROIBIÇÃO DE VECTOR STORES DESCARTÁVEIS EM PYTHON:** É categoricamente proibido reintroduzir dependências ou sugerir o uso de ChromaDB, FAISS ou Qdrant embutidos no processo Python do Hermes. A persistência vetorial reside estritamente em `souls_memory` (LanceDB sobre Arrow no ReFS).
5. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** Modificações no código que desativem a checagem de piso crítico ($S_m < 0.15$) ou que removam a sincronização em duas etapas (SQLite + LanceDB) sofrerão **rejeição sumária de PR** com violação de conformidade do ADR-006.

## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Técnica Canônica v7:** Seção 7.4 (_Chyros Daemon & Langevin Decay: Metabolismo Epistêmico Noturno_).
- **`AGENTS.md`:** Seção 4 (_Matriz Simbiótica com o Hermes Agent — Soberania de Memória_) e Seção 6 (_Sistema de Documentação como Guardrails_).
- **`CRATES_SPECIFICATION.md`:** Seção 2.4 (Superfície Pública e Anti-Patterns da crate `souls_memory`).
- **`SCHEMAS_DDL_MANIFEST.md`:** Seção 2.1 (Tabela `epistemic_memories`), Seção 3.2 (Tabela `epistemic_vectors`) e Seção 4 (_Políticas de Retenção e Metabolismo do Chyros Daemon_).
- **`MCP_TOOL_CONTRACTS.md`:** Seção 3.3 (Ferramentas `knowledge`, `semantic_search` e operações `mem_*`).
- **ADR-005:** _Centralização de I/O em Dev Drive ReFS e Pipeline MPSC Desidratado_.