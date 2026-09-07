# ADR-003 — Rejeição de Proxy Interceptor L7 Intradiálogo (Porta :3001) em Favor de Despacho Estrito de Subagentes (`delegate_task`) e Slots Auxiliares

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_server`, `souls_model_router`, `souls_protocol`, `souls_inference_runtime`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

Nas iterações experimentais do ecossistema (era Souls MC v6 e arquitetura SODA), o sistema implementava um componente denominado `souls_gateway` operando como um proxy reverso HTTP de Camada 7 (L7) na porta local `:3001`.

A premissa original postulava que todas as requisições de inferência geradas pelo cliente principal do Hermes Agent deveriam ser interceptadas localmente antes de atingirem provedores externos (OpenRouter, Anthropic, OpenAI, DeepSeek). Esse proxy executava duas operações invasivas em trânsito:
1. **Chaveamento Dinâmico de Modelos Turno-a-Turno (**_**Turn-Level Model Switching**_**):** O motor tentava avaliar cada prompt do usuário em tempo real e comutar o modelo de linguagem no meio da conversação (ex: comutando do Claude 3.5 Sonnet para o DeepSeek V3 Flash em perguntas triviais, e retornando ao Sonnet em etapas complexas).
2. **Injeção Passiva de Memória em Trânsito:** O proxy realizava buscas na base de vetores e injetava blocos de contexto diretamente no corpo da requisição HTTP antes de reencaminhá-la para a nuvem.

Embora teoricamente atraente para redução de custos, a validação empírica em ambiente de desenvolvimento revelou que a interceptação L7 intradiálogo constitui uma **falha arquitetural catastrófica**, colidindo diretamente com as mecânicas fundamentais dos provedores comerciais modernos e com a estabilidade de fluxos agênticos:

### 1.1 A Destruição Sistemática de Prefix Caching (Prompt Caching)

Os provedores de fronteira (Anthropic, OpenAI, DeepSeek) implementam mecanismos avançados de _Prompt Caching_ baseados na retenção do estado do KV Cache nos clusters de inferência. Esses caches são governados por invariantes rígidas:
- A chave de cache depende da **identidade estrita do modelo**, da chave de API e da correspondência exata, bit-a-bit, do prefixo de tokens da conversa.
- Se um proxy comuta o modelo principal no meio de um diálogo (Turno $N$ com Claude 3.5 Sonnet $\rightarrow$ Turno $N+1$ com DeepSeek Flash), o novo modelo **não possui o histórico prévio em cache**.
- O provedor de destino é forçado a reprocessar todos os $15.000$ a $80.000\text{ tokens}$ acumulados da conversa pela tarifa de **entrada cheia sem cache (**_**uncached input tokens**_**)**.
- Se no Turno $N+2$ o diálogo retorna ao modelo anterior, o cache original frequentemente já expirou pelo algoritmo LRU do provedor ou teve seu alinhamento quebrado pela inserção de mensagens heterogêneas, forçando uma nova leitura completa.
- **Resultado Financeiro:** A tentativa de economizar alguns centavos em um turno pontual gerou aumentos de **300% a 700% nos custos globais de API**, além de introduzir picos inaceitáveis de _Time-To-First-Token_ (TTFT).

### 1.2 Corrupção de Streams SSE e Quebra de Chamadas de Ferramentas Nativas

O framework Hermes Agent baseia sua deliberação e chamada de ferramentas em esquemas estruturados (`tools`, chamadas `<tool_call>` em XML/JSON e streaming de eventos SSE `text/event-stream`).
- Colocar um proxy L7 man-in-the-middle no fluxo de tokens introduz riscos crônicos de desserialização e reconstrução de pacotes.
- Se o proxy tentar inspecionar ou desidratar o corpo da resposta em voo para economizar contexto, marcadores de escape em strings multilinhas de código-fonte, tabelas Markdown e parâmetros JSON encadeados sofrem corrupção métrica, disparando erros de parsing irrecuperáveis no interpretador Python do Hermes.

### 1.3 Latência de Hop Intermediário e Redundância com o Hermes Runtime

O orquestrador do Hermes Agent (`AIAgent`) já incorpora nativamente:
- Gerenciamento e rotação de credenciais de provedores em pool.
- Roteamento dinâmico via comando de terminal `/model`.
- Esteira resiliente de contingência em múltiplos provedores (`fallback_providers`).

Inserir um segundo despachante HTTP L7 no meio desse pipeline adicionou latência de rede em loopback, duplicou rotinas de reconexão e introduziu um ponto único de falha (_Single Point of Failure_) que paralisava o agente caso o processo intermediário sofresse _backpressure_.

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretada a **rejeição absoluta e irreversível de qualquer proxy interceptador L7 de LLMs e a extinção definitiva da porta `:3001` no Souls Engine**.

O Souls Engine **NUNCA** intercepta, inspeciona ou altera o tráfego HTTP/SSE do modelo de conversação principal do Hermes Agent. O Hermes Agent conecta-se de forma direta, soberana e desimpedida aos endpoints comerciais de nuvem via conexões seguras TLS/HTTPS.

O algoritmo contextual bayesiano multiobjetivo **ParetoBandit** passa a atuar **com exclusividade nas seguintes fronteiras de desacoplamento estrutural**:
1. **Despacho de Subagentes Delegados (`delegate_task`):** Alocação dinâmica de Tiers de modelo para agentes secundários efêmeros.
2. **Resolução de Slots de Modelos Auxiliares (`auxiliary.*`):** Direcionamento de tarefas rotineiras e não conversacionais do framework Hermes.
3. **Hospedagem de Inferência Local Bare-Metal:** Disponibilização dos Tiers 0, 0.5 e 1 diretamente no daemon Rust para consumo sob demanda.

### 2.1 Desacoplamento do Modelo Principal vs. Subagentes

A distinção operacional entre o loop conversacional mestre e os nós de trabalho secundários é expressa pela seguinte matriz:

| **Dimensão Arquitetural**       | **Chat Principal Interativo (Hermes Master)**             | **Subagentes Delegados (delegate_task)**                                 |
| ------------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------ |
| **Ciclo de Vida**               | Sessão perene de longa duração.                           | Processo efêmero de tarefa única.                                        |
| **Histórico Conversacional**    | Acumulativo ($5\text{k} \rightarrow 100\text{k tokens}$). | Inicia rigorosamente em **zero tokens**.                                 |
| **Dependência de Prompt Cache** | **Crítica e mandatória** (alinhamento rígido).            | **Nula** (sem histórico prévio para amortizar).                          |
| **Ambiente de Arquivos**        | Workspace raiz do projeto (`Z:\souls_engine`).            | Worktree Git isolada (`.worktrees/subagent-*`).                          |
| **Atuação do ParetoBandit**     | **TERMINANTEMENTE PROIBIDA.**                             | **MANDATÓRIA E ATIVA.**                                                  |
| **Conexão de Rede**             | Hermes $\rightarrow$ Nuvem direta (TLS HTTPS).            | Hermes $\rightarrow$ `souls_route_subtask` $\rightarrow$ Endpoint ótimo. |

### 2.2 Despacho de Subagentes via `delegate_task`

Quando o modelo mestre do Hermes decide paralelizar uma demanda de engenharia através da ferramenta nativa `delegate_task`, o fluxo canônico opera da seguinte forma:
1. O agente invoca previamente a ferramenta MCP `souls_route_subtask` fornecida pelo daemon Rust na porta `:9123`.
2. O módulo `souls_model_router` processa o vetor de contexto da subtarefa $x_i \in \mathbb{R}^d$ (extensão estimada, necessidade de código, complexidade ciclomática).
3. O motor ParetoBandit cruza a utilidade esperada com a barreira termodinâmica da RTX 2060m $\Phi(T_{\text{GPU}}, V_{\text{livre}}, k)$ e a tabela FinOps, recomendando o Tier ótimo (ex: Tier 1 local na GPU se houver folga de VRAM; Tier 3 na nuvem se a tarefa for puramente semântica).
4. O Hermes Agent instancia a sub-instância de `AIAgent` configurando explicitamente os parâmetros de modelo retornados (`delegation.model` e `delegation.base_url`), apontando para o daemon local ou para a nuvem conforme a recomendação.
5. Ao concluir a execução na worktree isolada, apenas o resultado sintético consolidado retorna ao contexto mestre, preservando a integridade da conversa principal.

### 2.3 Resolução dos 11 Slots Auxiliares (`auxiliary.*`)

O Hermes Agent isola nativamente tarefas de suporte que não compartilham a janela conversacional principal. O Souls Engine atua como backend preferencial para alimentar esses slots configurados no `config.yaml` do Hermes:
- `compression`: Alocado preferencialmente para o Tier 3 Nuvem Fast ou Tier 0.5 CPU.
- `title_generation`: Alocado para o Tier 1 local (`Qwen2.5-Coder`) ou Tier 0 ONNX.
- `approval`: Alocado para o Tier 0 CPU AVX2 (`GLiClass Multilang Ultra`, $< 15\text{ ms}$).
- `web_extract`: Alocado para o Tier 3 Nuvem Fast.
- `triage_specifier`: Alocado para o Tier 1 local ou Tier 3 Nuvem.
- `curator`: Alocado para o Tier 1 local ou Tier 4 Nuvem.

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Preservação de 100% da Eficiência de Cache:** O modelo conversacional mestre mantém estabilidade absoluta de prefixo nos servidores da Anthropic, OpenAI e DeepSeek, garantindo taxas de acerto de prompt caching superiores a 90% em sessões longas e reduzindo a fatura de tokens de entrada em até 80%.
- **Zero Degradação de Streaming:** Eliminação completa de latência espúria de pacotes em trânsito. O usuário visualiza o fluxo de geração SSE nativo com latência pura de borda.
- **Erradicação da Porta `:3001`:** Menos um listener de rede, menos complexidade de firewall local e zero superfície de ataque para injeção acidental de pacotes em loopback.
- **Desacoplamento Limpo de Código:** A crate `souls_server` torna-se estritamente um provedor de APIs e MCP, sem a complexidade de manter pipelines de proxy reverso assíncrono com retentativas e parsing de streaming com `tower-http`.
- **Eficácia FinOps Direcionada:** O algoritmo ParetoBandit atua onde ele é matematicamente eficaz: alocando tarefas pesadas em lote para o hardware local (custo direto zero) e direcionando subagentes complexos para a nuvem, maximizando a métrica $E^3$.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**                                                | **Impacto Técnico**                                                                         | **Mitigação Canônica Implementada**                                                                                                                       |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Impossibilidade de forçar modelo local no chat mestre via proxy silencioso. | O usuário precisa configurar explicitamente o modelo principal no Hermes (`model.default`). | Criação de perfis predefinidos no Hermes (`hermes profile switch local` vs `hermes profile switch cloud`), dando controle soberano ao operador humano.    |
| Memória profunda não é injetada passivamente em cada requisição.            | O modelo principal não recebe automaticamente memórias antigas sem solicitá-las.            | Adoção do padrão ativo via MCP (`souls_memory_recall` / `mem_search`) e injeção assíncrona passiva na inicialização via Thin Adapter de `MemoryProvider`. |
| Subagentes exigem consulta prévia ao roteador.                              | O modelo mestre precisa saber quando consultar o `souls_route_subtask`.                     | Adoção da Skill canônica `souls-ops-finops` no prompt de sistema, condicionando o uso de `delegate_task` à consulta prévia de roteamento.                 |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE REINTRODUÇÃO DE PROXIES L7:** É categoricamente proibido criar servidores de proxy reverso, intermediadores de chamadas OpenAI ou escutar conexões de interceptação na porta `:3001` ou qualquer porta análoga.
2. **PROIBIÇÃO DE COMUTAÇÃO INTRADIÁLOGO:** É expressamente proibido propor lógicas que alternem o modelo do diálogo mestre a cada mensagem do usuário com base em custo ou entropia. O modelo mestre é imutável durante o diálogo.
3. **PROIBIÇÃO DE INJEÇÃO PASSIVA EM STREAMS:** Nenhum middleware em Rust pode interceptar requisições para anexar dados contextuais de forma oculta nos prompts em trânsito. Toda recuperação de memória deve ocorrer por ganchos oficiais (`MemoryProvider`) ou ferramentas MCP explícitas.
4. **RESTRIÇÃO DO PARETOBANDIT A SUBTAREFAS:** Qualquer implementação ou invocação da crate `souls_model_router` deve ser restrita ao cálculo de utilidade para a ferramenta MCP `souls_route_subtask` e governança de slots `auxiliary.*`.
5. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** A presença de propostas que tentem ressuscitar o gateway `:3001` ou intermediar chamadas externas do Hermes resultará em **rejeição sumária de PR** com violação de conformidade do ADR-003.

## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Técnica Canônica v7:** Seção 6 (_O Mecanismo ParetoBandit, FinOps e Desidratação Sintática LEAN_).
- **`AGENTS.md`:** Linha Vermelha 2 (_NENHUM Proxy Interceptor L7 para LLMs - Sem porta :3001_) e Linha Vermelha 3 (_NENHUM Chaveamento Intradiálogo pelo ParetoBandit_).
- **`CRATES_SPECIFICATION.md`:** Seções 2.6 (`souls_model_router`) e 2.9 (`souls_server`).
- **`MCP_TOOL_CONTRACTS.md`:** Seção 3.5.1 (Contrato da ferramenta `route_subtask`).
- **ADR-001:** _Abandono de UI Desktop em Favor de Daemon Headless_.
- **ADR-002:** _Adoção de Localhost TCP Loopback e Streamable HTTP/SSE_.