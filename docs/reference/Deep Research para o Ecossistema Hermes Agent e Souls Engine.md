# Arquitetura Antropofágica de Deep Research para o Ecossistema Hermes Agent e Souls Engine

## 1. Diagnóstico Arquitetural do Hospedeiro: Hermes Agent e Souls Engine

A construção de um motor proprietário de investigação profunda (_Deep Research_) capaz de rivalizar com os sistemas comerciais da Google (Gemini Deep Research), OpenAI e Anthropic exige, como premissa fundamental, o alinhamento estrito com a mecânica interna do ambiente hospedeiro. O ecossistema **Hermes Agent** (`NousResearch/hermes-agent`), distribuído sob licença MIT, opera sob o princípio arquitetural de separação estrita entre o _harness_ — o corpo persistente que detém a identidade, a memória, as ferramentas e o agendamento — e o modelo fundacional, tratado como um cérebro substituível. Qualquer tentativa de acoplar um _framework_ externo monolítico que ignore o ciclo nativo de conversação e delegação do Hermes resulta inevitavelmente na invalidação do _prompt caching_, em corrupção de estado na base de dados e no esgotamento prematuro do orçamento de execução.

O núcleo operacional do Hermes estrutura-se em três camadas interdependentes que separam as interfaces de entrada, o processamento cognitivo e o isolamento de execução. Na primeira camada situam-se as superfícies de entrada (_Surfaces_), que funcionam como adaptadores leves sem reimplementar a lógica central: incluem a interface de linha de comandos (CLI), a interface textual (`hermes --tui`, construída em Node.js e React Ink comunicando com o motor Python via JSON-RPC 2.0 sobre `stdio`), a interface Web (`hermes web`, em FastAPI e React SPA), o protocolo ACP para editores de código (`acp_adapter/` para VS Code, Zed e JetBrains) e um _Messaging Gateway_ de execução contínua (`gateway/run.py`) compatível com mais de 25 plataformas, como Telegram, Discord, Slack e WhatsApp. A segunda camada corresponde ao Agente Central (`AIAgent`), instanciado através da fachada `run_agent.py` e executado em `agent/conversation_loop.py`, que coordena quatro subsistemas modulares: o registo autodescoberto de mais de 70 ferramentas (`tools/registry.py` e `model_tools.py`), o carregador progressivo de competências (`hermes_cli/skills_hub.py`), o gestor de memória (`agent/memory_manager.py`) e a resolução dinâmica de fornecedores de modelos (`hermes_cli/runtime_provider.py`) através de quatro modos de API auto-detetados (`chat_completions`, `codex_responses`, `anthropic_messages` e `bedrock_converse`). A terceira camada compreende os ambientes de execução (_Execution Backends_) em `tools/environments/`, permitindo que comandos de terminal e código corram de forma transparente em sete infraestruturas distintas: terminal local (`local`), contentores isolados (`docker`), servidores remotos (`ssh`) ou ambientes _serverless_ e em nuvem (`modal`, `daytona`, `singularity` e `vercel_sandbox`).

A camada de identidade e acompanhamento proativo — conhecida no ecossistema como **Souls Engine** e materializada pela combinação da arquitetura nativa `SOUL.md` com o _plugin_ _zero-patch_ **Hermes_Soul_patch** (`hermes-companion`) e motores de persona como `SoulTavern` — introduz uma dimensão comportamental e temporal que altera profundamente o desenho de sistemas de investigação de longa duração. No Hermes, o ficheiro `$HERMES_HOME/SOUL.md` ocupa de forma determinística a primeira posição (`slot #1`) na montagem do _system prompt_ realizada por `agent/prompt_builder.py`, antecedendo orientações de ferramentas, instantâneos congelados de memória (`MEMORY.md` e `USER.md`) e instruções de projeto (`AGENTS.md`). Para garantir a estabilidade do _prompt caching_ nos fornecedores _cloud_ (como a cache de prefixos da Anthropic e da OpenAI), o Hermes mantém o _system prompt_ estritamente imutável durante toda a sessão.

O _plugin_ `Hermes_Soul_patch` (`gejifeng/Hermes_Soul_patch`) respeita esta imutabilidade ao separar a camada de integração (_Plugin Layer_) da camada de simulação comportamental (_Companion Layer_), utilizando pontos de extensão oficiais sem modificar o código-fonte do núcleo. Através do gancho de pré-inferência (`pre_llm_call`), o sistema anexa de forma efémera o contexto temporal, o estado emocional multidimensional (`valence`, `arousal`, `energy`, `social_need`, `confidence` e `momentum`, armazenados em `$HERMES_HOME/EMOTION_STATE.md`) e as mensagens proativas pendentes (`companion_pending.txt`) diretamente no contexto da mensagem do utilizador a cada turno, injetando estado fresco sem corromper a cache do _system prompt_. Após a resposta do modelo, o gancho `post_llm_call` dispara tarefas assíncronas em segundo plano que invocam um modelo auxiliar para deduzir variações emocionais e acionam o _World-Interaction Observer_, o qual analisa conservadoramente as interações para atualizar a agenda diária (`$HERMES_HOME/companion/events.json`) utilizando bloqueios de ficheiro atómicos (`fcntl.flock`) e renomeações seguras entre processos. Paralelamente, o intercetor `post_tool_call` monitoriza os resultados das ferramentas e reduz automaticamente a valência emocional (`valence`) sempre que deteta falhas ou erros de execução, enquanto uma _thread_ de batimento cardíaco proativo (_Proactive Heartbeat_) verifica a cada 300 segundos se o nível de ativação emocional (`arousal >= 0.70`) ou a proximidade de um evento agendado justificam o envio de uma mensagem espontânea via injeção direta (`ctx.inject_message()`), fila de espera ou notificação nativa na plataforma (`hermes cron --deliver telegram`).

Para desenhar a solução de _Deep Research_ sobre esta fundação, é imperativo respeitar as garantias de isolamento e as restrições rígidas da ferramenta nativa `delegate_task` (`tools/delegate_tool.py`), a qual interceta chamadas diretamente no ciclo do agente sem passar pelo despachante padrão de ferramentas. Quando o agente principal invoca `delegate_task`, o Hermes instancia filhos `AIAgent` dentro de um `ThreadPoolExecutor` (com limite padrão de até 10 tarefas concorrentes na arquitetura base ou lotes de 3 em configurações conservadoras, ajustável via `delegation.max_concurrent_children`). Os subagentes operam sob restrições arquiteturais estritas que ditam o desenho do fluxo de investigação, conforme sistematizado na tabela seguinte:

|**Mecanismo / Propriedade do Hermes**|**Comportamento Nativo e Limite Rígido**|**Impacto Direto na Arquitetura de Deep Research**|
|---|---|---|
|**Isolamento de Contexto**|O subagente inicia com histórico zero; recebe apenas os parâmetros `goal`, `context`, `images` (até 8) e `output_schema`.|O Orquestrador tem de sintetizar _briefings_ autocontidos; chamadas intermédias de ferramentas dos filhos nunca poluem o contexto do pai.|
|**Herança de Ficheiros de Projeto**|Subagentes incorporam automaticamente `.hermes.md` > `AGENTS.md` > `CLAUDE.md` > `.cursorrules`, mas **excluem** `SOUL.md` por padrão.|Permite usar `SOUL.md` para a voz e emoção do _companion_ principal sem afetar a objetividade analítica fria dos subagentes de pesquisa.|
|**Roteamento de Perfis (`profile_routes`)**|A configuração `delegation.backends.profile_routes` permite delegar para perfis específicos com credenciais, modelos e `SOUL.md` próprios.|Viabiliza a distribuição de subtarefas para modelos mais baratos ou SLMs locais (`llama.cpp` / Ollama) configurados em perfis dedicados.|
|**Bloqueio de Ferramentas em Folhas (`role="leaf"`)**|Os conjuntos `delegation`, `clarify`, `memory`, `code_execution` (`execute_code`), `send_message` e `cronjob` são bloqueados em subagentes folha.|Subagentes não podem gravar diretamente em `MEMORY.md` nem usar `execute_code`; a persistência de dados entre agentes tem de ocorrer via ferramentas de ficheiro (`write_file`).|
|**Orquestração Aninhada (`role="orchestrator"`)**|Retém `delegate_task` se `delegation.max_spawn_depth >= 2` (o padrão é `1`), aguardando sincronamente pelo seu lote antes de sintetizar.|Permite criar árvores recursivas de pesquisa (_breadth/depth_), mas exige controlo rigoroso para evitar explosão combinatória de agentes.|
|**Orçamento Partilhado (`IterationBudget`)**|Limite rígido de 90 turnos partilhado de forma _thread-safe_ entre o agente pai e todos os subagentes gerados na tarefa.|Uma árvore de pesquisa mal dimensionada esgota os 90 turnos rapidamente, acionando `_budget_exhausted_injected` e forçando uma sumarização precoce na chamada de graça (`_budget_grace_call`).|
|**Validação de Esquema (`output_schema`)**|Se o JSON retornado pelo subagente falhar a validação do esquema, o pai envia exatamente 1 turno de correção; se falhar novamente, retorna `schema_valid: false` e o texto bruto em `schema_note`.|O trabalho de pesquisa nunca é descartado por erros de formatação de SLMs ou modelos menores, garantindo resiliência na agregação.|

Adicionalmente, a interação com o `Souls Engine` (`Hermes_Soul_patch`) requer um isolamento cuidadoso de ganchos (_hooks_). Como o `post_tool_call` do `Hermes_Soul_patch` reduz a valência emocional (`valence`) sempre que deteta erros de ferramentas, e uma sessão de _Deep Research_ envolve dezenas de tentativas de _scraping_ onde bloqueios HTTP 403 ou _timeouts_ são ocorrências normais de exploração, executar raspagem bruta diretamente no agente principal deixaria o _companion_ artificialmente "frustrado" (`dominant: frustrated`). Ao confinar a pesquisa exploratória e a extração web a subagentes isolados ou a ferramentas encapsuladas que tratam falhas internamente, o estado emocional do `Souls Engine` permanece íntegro, podendo ser positivamente estimulado (`arousal` e `confidence`) quando o relatório final verificado é concluído e entregue via _Proactive Heartbeat_.

## 2. Validação da Premissa Económica: Cérebro Cloud vs. Offloading para SLMs e Modelos Baratos

A premissa de combinar o raciocínio estratégico de um modelo de fronteira na nuvem ("cérebro _cloud_") com a distribuição de tarefas operacionais menores para _Small Language Models_ (SLMs) locais ou modelos específicos de baixo custo é empiricamente verdadeira e economicamente determinante, mas apenas se implementada através de um protocolo assimétrico de decomposição atómica. Sem esse protocolo, a delegação ingénua para SLMs colapsa a qualidade factual do relatório final.

A urgência económica desta distribuição é demonstrada pelos dados de produção do sistema de investigação multiagente da Anthropic: enquanto agentes autónomos simples consomem cerca de 4 vezes mais _tokens_ do que uma interação de _chat_ padrão, um sistema de _Deep Research_ multiagente consome em média **15 vezes mais _tokens_** do que uma conversa normal. Na avaliação do _benchmark_ BrowseComp realizada pela Anthropic, o volume total de _tokens_ consumidos explicou sozinho **80% da variância de desempenho**, enquanto o número de chamadas de ferramentas e a escolha do modelo explicaram os restantes 15%. Esta distribuição estatística demonstra que a qualidade da investigação profunda é diretamente proporcional à capacidade do sistema de processar massas de texto em janelas de contexto isoladas e comprimi-las em conhecimento denso para o agente coordenador. Processar milhões de _tokens_ de páginas web brutas e PDFs exclusivamente num modelo de fronteira (como Claude Opus, GPT-5.4 ou o1/o3) torna o custo unitário proibitivo, ultrapassando frequentemente os $15 por execução extensa.

Para validar cientificamente como transferir este volume de leitura para SLMs locais (de 1B a 8B de parâmetros executados via `llama.cpp` nativo do Hermes, Ollama ou vLLM) sem degradar a inteligência do relatório, analisa-se a investigação do laboratório HazyResearch da Universidade de Stanford sobre os protocolos **Minion** e **MinionS** (`HazyResearch/minions`, arXiv:2502.15964). No protocolo ingénuo (**Minion**, no singular), o SLM local detém o acesso aos documentos longos e conversa livremente em múltiplos turnos com o modelo de fronteira na nuvem. Embora esta abordagem reduza o custo de API na nuvem em **30,4 vezes** (custando apenas 3,3% de uma execução puramente _cloud_), a precisão da resposta cai abruptamente para **87%** do desempenho do modelo de fronteira, representando uma perda média de 9,4 a 13 pontos percentuais de exatidão.

A investigação empírica identificou três causas fundamentais para a falha dos SLMs quando operam como subagentes autónomos em conversação livre. Em primeiro lugar, verifica-se um colapso severo em instruções multi-etapa: os modelos pequenos perdem a coerência lógica quando recebem pedidos compostos ou sequenciais, sendo que a simples divisão de uma instrução complexa em pedidos separados de passo único (_single-step_) aumenta o desempenho do SLM em **56%**. Em segundo lugar, ocorre uma degradação acentuada em contextos longos: quando o comprimento do documento fornecido ao SLM passa de menos de 1.000 _tokens_ para mais de 65.000 _tokens_, a precisão de extração do modelo pequeno cai **13%**. Em terceiro lugar, a conversação sequencial impede o processamento em lote (_batching_) no _hardware_ local (como GPUs de consumo RTX 4090 ou Apple Silicon via Metal), subutilizando a capacidade de cálculo paralelo, ao mesmo tempo que modelos abaixo do limiar crítico de **3 mil milhões de parâmetros (3B)** se revelam estruturalmente incapazes de sustentar o protocolo de colaboração.

Para superar estas limitações, o protocolo **MinionS** (no plural) substitui a conversação livre por um ciclo estruturado de _Decomposição–Execução–Agregação_. O modelo de fronteira na nuvem — sem nunca ler o documento bruto — gera código Python ou especificações formais $f(c, T)$ que dividem a tarefa em subtarefas de passo único sobre fragmentos curtos de texto, formalizadas como $t^{(i)} = (\tilde{q}^{(i)}, \tilde{c}^{(i)})$, onde $\tilde{q}^{(i)}$ é uma pergunta atómica e $\tilde{c}^{(i)}$ é um bloco reduzido do documento. O SLM local executa dezenas destas subtarefas em paralelo nos fragmentos locais (maximizando o _batching_ da GPU) e devolve saídas estritamente tipadas via esquemas Pydantic (`StructuredLocalOutput` contendo `explanation`, `citation` e `answer`), filtrando o ruído localmente antes de enviar apenas os fragmentos relevantes para a agregação do modelo _cloud_.

Os resultados desta arquitetura híbrida comprovam que um SLM de **8B parâmetros** sob o protocolo `MinionS` recupera **97,9% da precisão** do sistema puramente _cloud_, reduzindo os custos de nuvem em **5,7 vezes** (consumindo apenas 17,5% a 18,0% do orçamento _cloud_) em _benchmarks_ intensivos como FinanceBench, LongHealth e QASPER. Mesmo com um modelo local ultraleve de **3B parâmetros** (como Llama-3.2 3B ou Qwen2.5 3B executados no motor `llama.cpp` gerido pelo Hermes desde a versão v0.22.0), o protocolo atinge **93,4% da precisão** de fronteira a **16,6% do custo**. Adicionalmente, o protocolo `MinionS` supera o RAG clássico em tarefas de sumarização longa (como o _benchmark_ BooookScore) porque não se limita a recuperar fragmentos por similaridade vetorial, mas executa raciocínio extrativo estruturado sobre cada bloco no dispositivo local antes de transmitir a síntese.

Para além da colaboração com SLMs locais, a análise dos projetos líderes em código aberto — nomeadamente **GPT-Researcher** (`assafelovic/gpt-researcher`), **LangChain Open Deep Research** (`langchain-ai/open_deep_research`) e **Stanford STORM** (`stanford-oval/storm`) — demonstra uma segunda camada essencial de otimização económica: a **filtragem determinística pré-LLM** combinada com o **roteamento multi-nível de modelos na nuvem**. No GPT-Researcher, passar páginas web raspadas diretamente para o modelo sem filtragem prévia aumenta o custo total da pesquisa em **65%**. Ao aplicar um filtro prévio como o modelo utilitário `Jev` (que pontua a utilidade real de cada passagem para responder à pergunta, alcançando 73% de passagens relevantes — 59% superior a _embeddings_ vetoriais pelo mesmo custo) ou o _fallback_ local de custo zero **BM25** (ranking lexical por palavras-chave sem necessidade de API ou _embeddings_), elimina-se a maior parte do ruído HTML e SEO antes de gastar um único _token_ de LLM.

Consequentemente, a matriz ideal de alocação cognitiva para o motor de _Deep Research_ no Hermes divide o trabalho em quatro níveis de custo e capacidade, tirando partido nativo das secções `auxiliary.<task>` e `delegation.backends.profile_routes` do ficheiro `~/.hermes/config.yaml`:

|**Nível Cognitivo (Tier)**|**Equivalência nos Frameworks Open-Source**|**Funções Exatas no Pipeline de Deep Research**|**Modelos Recomendados (Exemplos)**|**Mecanismo Nativo de Configuração no Hermes Agent**|
|---|---|---|---|---|
|**Tier 0: Pré-Filtro Determinístico (Zero-LLM)**|`CONTEXT_FILTER=bm25` / `jev` (GPT-Researcher)|Limpeza de DOM/HTML, desduplicação de URLs normalizados, _chunking_ de texto e pontuação lexical BM25 ou utilitária Jev sobre páginas raspadas.|`rank_bm25` (Python local) ou API `Jev` (`TYPESAFE_API_KEY`).|Script Python em ferramenta customizada (`tools/research_filter.py`) ou pré-processamento no extrator.|
|**Tier 1: Operário Local / SLM (`LOCAL_SLM`)**|Worker Local `MinionS` (HazyResearch) / `auxiliary` (Hermes)|Execução em paralelo de subtarefas de passo único sobre fragmentos curtos (<2K _tokens_), extração de citações literais (`StructuredLocalOutput`), triagem inicial e compressão de _logs_.|Modelos 3B a 8B locais: Qwen2.5-7B, Llama-3.1-8B, Granite-3.3-8B via `llama.cpp` nativo, Ollama ou vLLM.|Motor `llama.cpp` gerido do Hermes (`v0.22.0+`), `auxiliary.<task>` em `config.yaml` e perfil `local-minion` em `profile_routes`.|
|**Tier 2: Explorador Rápido (`FAST_LLM`)**|`FAST_LLM` (GPT-Researcher) / `summarization_model` (LangChain) / `question_asker_lm` (STORM)|Formulação de _queries_ SERP diversificadas, simulação rápida de perguntas de perspetiva, navegação de subagentes (`Sub-researcher`), sumarização intermédia de fontes e compressão de ramo (`compress_research`).|`gpt-5.4-mini`, `gpt-4.1-mini`, `gemini-2.0-flash`, `deepseek-v3.2` ($0.26/$0.38 por 1M _tokens_), `glm-4.5-air`, `qwen3-32b`.|Perfil dedicado `fast-scout` invocado via `delegate_task` (`delegation.backends.profile_routes`) com `output_schema` ativo.|
|**Tier 3: Cérebro Estratégico e Redator (`STRATEGIC_LLM` / `SMART_LLM`)**|`STRATEGIC_LLM` & `SMART_LLM` (GPT-Researcher) / `LeadResearcher` (Anthropic) / `final_report_model` (LangChain)|Clarificação inicial com o utilizador, planeamento da árvore de investigação, geração de especificações de decomposição `MinionS`, reconstrução do espaço de trabalho (`IterResearch`), síntese narrativa longa e verificação final de citações.|`claude-opus-4` / `claude-sonnet-4`, `openai:gpt-5.4` / `o3-mini` (esforço `high`), `deepseek-r1`, `gemini-2.5-pro`, `kimi-k2.5`.|Modelo principal da sessão (`model.default` em `~/.hermes/config.yaml`) ou cadeia `ultrabrain`/`deep`.|

## 3. Engenharia Reversa dos Laboratórios de Fronteira e Melhorias de Estado da Arte

O desenho de um sistema de _Deep Research_ de altíssima qualidade exige incorporar as lições arquiteturais extraídas dos sistemas de produção dos grandes laboratórios de IA — especificamente o sistema multiagente da Anthropic (Claude Research), o OpenAI Deep Research, o Google Gemini Deep Research e o **Tongyi DeepResearch** do Alibaba NLP (`Alibaba-NLP/DeepResearch`) — corrigindo simultaneamente os modos de falha documentados por estas equipas.

Na arquitetura da **Anthropic**, o sistema rejeita pipelines estáticos de RAG em favor de um padrão dinâmico _Orchestrator-Worker_. Um agente líder (`LeadResearcher`, tipicamente alimentado por Claude Opus 4) analisa o pedido, utiliza _Extended Thinking_ como bloco de rascunho controlável para traçar a estratégia e **grava o plano de investigação numa memória externa persistente** antes de delegar. Esta gravação prévia é vital: como sessões de investigação complexas ultrapassam frequentemente os **200.000 _tokens_**, a janela de contexto do agente líder sofre truncagem; persistir o plano fora do histórico conversacional impede que o orquestrador perca a direção estratégica a meio da execução. De seguida, o `LeadResearcher` gera entre 3 a 5 subagentes (`Subagents`, tipicamente Claude Sonnet 4) em paralelo, e cada subagente executa 3 ou mais ferramentas concorrentemente, utilizando _Interleaved Thinking_ após cada busca para avaliar a qualidade dos resultados, identificar lacunas e refinar as _queries_ seguintes — um paralelismo de dois níveis que reduz o tempo total de pesquisa em até **90%**. Finalmente, quando o líder dá a recolha por concluída, o rascunho e os documentos passam para um **`CitationAgent` dedicado**, cuja única função é auditar cada afirmação e vincular as localizações exatas das citações.

Por sua vez, o laboratório **Alibaba Tongyi** (`Tongyi-DeepResearch-30B-A3B`, um modelo MoE de 30,5 mil milhões de parâmetros totais com apenas 3,3 mil milhões ativados por _token_) diagnosticou o principal motivo pelo qual agentes de longo horizonte degradam a qualidade da escrita ao fim de dezenas de passos: a **"Asfixia Cognitiva" (_Cognitive Suffocation_) e a "Poluição por Ruído" (_Noise Pollution_)**. Quando um agente acumula todo o histórico de chamadas de ferramentas e páginas raspadas numa única janela de contexto contínua (como no modo ReAct clássico de 128K _tokens_), a atenção do modelo dispersa-se no ruído histórico. Para resolver este problema, o Tongyi introduziu o paradigma **IterResearch (Heavy Mode)** baseado na **Reconstrução do Espaço de Trabalho (_Workspace Reconstruction_)**. Em vez de manter uma conversa contínua, a pesquisa é dividida em rondas discretas; no final de cada ronda, o histórico bruto de interações é descartado e o agente reconstrói um espaço de trabalho enxuto contendo apenas as descobertas essenciais extraídas e um **Relatório Central em Evolução (_Evolving Central Report_)**. Múltiplos agentes executam este ciclo `IterResearch` em caminhos paralelos antes que um `Synthesis Agent` unifique os relatórios parciais.

Ao cruzar estas arquiteturas de fronteira com as falhas operacionais detetadas em produção, emergem seis melhorias estruturais obrigatórias para a arquitetura proposta. A primeira melhoria consiste na imposição de **regras explícitas de escalonamento de esforço (_Effort Scaling Heuristics_)**: a equipa de engenharia da Anthropic descobriu que os agentes falham sistematicamente ao julgar o esforço adequado para diferentes pedidos, chegando a lançar 50 subagentes para perguntas triviais ou a pesquisar indefinidamente fontes inexistentes. Para evitar este desperdício, o _prompt_ do orquestrador tem de embutir orçamentos determinísticos segundo a complexidade da tarefa: a verificação simples de factos exige apenas **1 subagente com 3 a 10 chamadas de ferramentas**; comparações diretas de elementos alocam **2 a 4 subagentes com 10 a 15 chamadas cada**; e investigações abertas de alta complexidade mobilizam **mais de 10 subagentes** (organizados em ondas concorrentes) com fronteiras de responsabilidade estritamente divididas para impedir trabalho duplicado (como o erro documentado em que um subagente pesquisou a crise de semicondutores de 2021 enquanto outros dois repetiam buscas idênticas sobre cadeias de abastecimento de 2025).

A segunda melhoria reside na **estratégia de busca "Amplo Primeiro, Estreito Depois" (_Broad-to-Narrow Search_) aliada a heurísticas de autoridade de fonte**: os modelos de linguagem tendem nativamente a formular _queries_ iniciais demasiado longas e específicas que devolvem poucos ou nenhuns resultados, além de selecionarem frequentemente _content farms_ otimizadas para SEO em detrimento de documentos académicos ou fontes primárias. Os subagentes devem ser instruídos a iniciar obrigatoriamente com _queries_ curtas e panorâmicas para mapear a paisagem informacional antes de afunilar progressivamente, aplicando filtros de prioridade para PDFs científicos, documentação técnica oficial e relatórios primários. A terceira melhoria substitui a cópia de texto entre contextos pela **passagem de ponteiros de sistema de ficheiros (_Filesystem Pointer Handoff_)**: como a execução síncrona de subagentes cria gargalos de coordenação e copiar textos extensos através do histórico conversacional inflaciona o consumo de _tokens_, cada subagente deve gravar os seus artefactos densos diretamente no sistema de ficheiros partilhado (`$HERMES_HOME/research/{run_id}/scouts/scout_{n}.md`) e devolver ao Orquestrador apenas uma referência leve acompanhada de metadados de cobertura.

A quarta melhoria incorpora a **descoberta prévia de perspetivas (_Perspective-Guided Question Asking_) e o mapa conceptual dinâmico**, oriundos do projeto **Stanford STORM / Co-STORM** (`stanford-oval/storm`), garantindo que o agente levanta artigos análogos para identificar múltiplos ângulos de análise antes de gerar o esqueleto do relatório, eliminando o viés de perspetiva única. A quinta melhoria introduz **travas determinísticas anti-loop por desativação dinâmica de ações**, extraídas do **Jina DeepResearch** (`jina-ai/node-DeepResearch`): em vez de permitir que um subagente esgote os 90 turnos do `IterationBudget` do Hermes em buscas repetitivas, a máquina de estados monitoriza os retornos — se uma ação `search` não devolver URLs inéditos, a capacidade `search` é desativada no turno imediato; se uma ação `reflect` gerar sub-perguntas repetidas na fila de lacunas (`gaps queue`), a reflexão é bloqueada, forçando o agente a ler as páginas recolhidas (`visit`) ou a sintetizar o conhecimento. Finalmente, a sexta melhoria combina a **escada de extração progressiva (`Claw`) com a disciplina epistémica de quatro níveis**: para contornar páginas com JavaScript pesado ou bloqueios anti-bot sem desperdiçar recursos quando uma leitura simples basta, o extrator escala sequencialmente (`HTTP simples` $\rightarrow$ `Parsing HTML` $\rightarrow$ `Impersonação de Fingerprint TLS` $\rightarrow$ `Stealth Browser Camoufox/Playwright` $\rightarrow$ `PDF` $\rightarrow$ `OCR Tesseract`), enquanto cada afirmação recolhida recebe obrigatoriamente uma etiqueta de grau de evidência (`[VERIFIED]`, `[SOURCED]`, `[REASONED]` ou `[ESTIMATED]`) e um indicador de polaridade (`supports` vs. `refutes`).

## 4. Avaliação Comparativa de Projetos Open-Source para Canibalização

Para determinar qual a melhor opção de projeto _open-source_ a "antropofagizar" — absorvendo a sua lógica de orquestração, engenharia de _prompts_, filtros de contexto e validação factual dentro da estrutura nativa do Hermes Agent — foram auditados sete projetos de referência no ecossistema global de _Deep Research_.

### Análise Profunda dos Candidatos e Veredito Antropofágico

O projeto **`langchain-ai/open_deep_research`** constitui a referência primária para a lógica de estado, divisão de modelos e engenharia de _prompts_. Com um núcleo extremamente conciso de **719 linhas de código** em `src/open_deep_research/deep_researcher.py` (apoiado por `configuration.py` e `prompts.py`), esta implementação reproduz fielmente o padrão Supervisor-Pesquisador da Anthropic e atinge paridade competitiva no _leaderboard_ **Deep Research Bench** (composto por 100 tarefas de nível de doutoramento em 22 domínios científicos e empresariais). O seu fluxo divide-se em três fases altamente coesas. Na primeira fase (_Scoping_), o nó `clarify_with_user` avalia se o pedido do utilizador contém ambiguidades ou siglas desconhecidas e, caso `allow_clarification` esteja ativo, interrompe a execução para colocar perguntas de clarificação; em seguida, o nó `write_research_brief` transforma o histórico conversacional num _briefing_ de investigação estruturado e isento de suposições não verificadas. Na segunda fase (_Research_), o nó `research_supervisor` gere o ciclo de vida através de três ferramentas exclusivas (`think_tool`, `ConductResearch` e `ResearchComplete`), sendo regido por uma instrução crítica no `lead_researcher_prompt` (linhas 79–136 de `prompts.py`) que obriga o supervisor a invocar `think_tool` **antes** de `ConductResearch` para planear a decomposição e **imediatamente após** cada retorno para avaliar o progresso, proibindo chamadas paralelas a `think_tool`. Cada trabalhador (`researcher`) opera numa janela de contexto isolada (`ResearcherState`) até atingir uma condição determinística de paragem (3+ fontes relevantes, 2 buscas consecutivas com resultados duplicados ou o teto `max_react_tool_calls`), passando então o histórico bruto pelo nó **`compress_research`**, que atua como um filtro de engenharia de contexto capaz de remover o ruído dos _logs_ de ferramentas preservando integralmente e _verbatim_ todas as estatísticas, factos, citações e URLs. Na terceira fase (_Report Writing_), o modelo final (`final_report_model`) sintetiza os pacotes comprimidos num relatório exaustivo, beneficiando de uma configuração nativa que já separa quatro _slots_ distintos de modelos (`summarization_model`, `research_model`, `compression_model` e `final_report_model`).

O segundo pilar de canibalização é o **`assafelovic/gpt-researcher`** (com mais de 29.700 estrelas no GitHub), que oferece a melhor engenharia de exploração recursiva em árvore, agregação multi-fonte e filtragem pré-LLM. O seu modo **Deep Research** (`gpt_researcher/skills/researcher.py` / `ResearchConductor`) implementa uma árvore de busca configurável (`deep_research_breadth: 4`, `deep_research_depth: 2`) controlada por semáforos assíncronos (`asyncio.Semaphore`), onde a cada nível de profundidade a largura de exploração é reduzida para metade (`new_breadth = max(2, breadth // 2)`, `new_depth = depth - 1`) para aprofundar as pistas mais promissoras, completando investigações profundas em cerca de 5 minutos por um custo médio de **$0,40** (utilizando `o3-mini` com esforço de raciocínio `high`). O GPT-Researcher destaca-se ainda por agregar mais de 20 fontes web por tarefa para mitigar o viés de confirmação pela frequência cruzada de factos, suportar pesquisa híbrida combinando buscadores web e servidores MCP (`export RETRIEVER=tavily,mcp`), ingerir documentos locais (`DOC_PATH` compatível com PDF, texto, CSV, Excel, Markdown, PowerPoint e Word) e aplicar os filtros pré-LLM `Jev` e `BM25`. Além disso, a sua arquitetura multiagente (`/multi_agents`) introduz uma equipa de 7 agentes (`Chief Editor`, `Browser`, `Editor`, `Researcher`, `Reviewer`, `Revisor`, `Writer` e `Publisher`) onde o ciclo de validação entre `Reviewer` e `Revisor` audita cada rascunho de secção antes da compilação final.

O terceiro pilar reside nos projetos nativos do próprio ecossistema Hermes — especificamente **`witt3rd/oh-my-hermes`**, **`moonlight-lupin/agent-skills`** e **`rlaope/oh-my-hermes`** — que já resolveram as fricções de adaptação às primitivas do Hermes Agent. No pacote `witt3rd/oh-my-hermes`, a competência `omh-deep-research` estrutura um pipeline de 4 estágios (Decompor $\rightarrow$ Busca Paralela $\rightarrow$ Sintetizar $\rightarrow$ Verificar Citações) desenhado para um envelope de custo previsível de **5 a 8 chamadas `delegate_task`** no caminho normal (3 a 5 pesquisadores + 0 a 1 _followup_ + 1 sintetizador + 1 verificador) e limitado por um teto rígido de 3 tentativas (_3-strike retry cap_, máximo de 14 a 16 chamadas antes de sinalizar estado `BLOCKED`). Crucialmente, `omh-deep-research` isola o agente verificador num contrato estrito de apenas leitura (`toolsets=["file"]`, governado por `role-research-verifier.md`) e grava o resultado validado numa sentinela durável em `.omh/research/{slug}-report.md` (`status: confirmed`). Complementarmente, o repositório `moonlight-lupin/agent-skills` (`research/deep-research`, `source-tracker` e `fact-checker`) fornece os portões determinísticos de validação (_deterministic validation gates_), a classificação de fontes em três níveis (primária, secundária e terciária), a ancoragem de citações literais (_verbatim quote anchoring_), o rastreio de polaridade de refutação e a disciplina das quatro etiquetas epistémicas (`[VERIFIED]`, `[SOURCED]`, `[REASONED]`, `[ESTIMATED]`), enquanto `rlaope/oh-my-hermes` contribui com a pontuação prévia de pedidos e as cadeias editáveis de recomendação e _fallback_ de modelos (`ultrabrain`, `architect`, `deep`, `unspecified-high`, `unspecified-low`, `quick`).

Os restantes candidatos oferecem mecanismos pontuais valiosos, mas não devem ser adotados como esqueleto principal. O **ByteDance DeerFlow** (`bytedance/deer-flow`), cuja versão 2.0 constitui uma reescrita completa como _SuperAgent Harness_ com 9 _middlewares_ sequenciais (`ThreadDataMiddleware` até `ClarificationMiddleware`) e um _sandbox_ Docker All-in-One (`/mnt/user-data/{workspace,uploads,outputs}`), sobrepõe-se diretamente às funcionalidades que o Hermes Agent já possui nativamente; contudo, a sua lógica de deteção determinística de _loops_ de ferramentas (`loop_detection.enabled`) e a separação de 5 papéis da sua versão 1.x (`Coordinator`, `Planner`, `Researcher`, `Coder`, `Reporter`) justificam a incorporação de capacidades de execução analítica de código durante a investigação. O **Stanford STORM / Co-STORM** (`stanford-oval/storm`), implementado sobre DSPy, fornece o modelo ideal de separação de LLMs via `STORMWikiLMConfigs` (`conv_simulator_lm` e `question_asker_lm` em modelos rápidos vs. `outline_gen_lm`, `article_gen_lm` e `article_polish_lm` em modelos de fronteira) e o protocolo de geração de perspetivas. Por fim, entre os projetos em TypeScript, o **`dzhng/deep-research`** (<500 linhas de código) funciona apenas como referência didática — um estudo comparativo da Langfuse demonstrou que os seus relatórios atingem apenas ~12.000 caracteres (custo ~$0,71 em ~3m34s) contra ~87.000 caracteres do OpenAI Deep Research, além de o repositório estar praticamente dormente desde meados de 2025 — ao passo que o **`jina-ai/node-DeepResearch`** fornece a excelente máquina de estados de 4 ações (`search`, `visit`, `reflect`, `answer`) e o mecanismo de encerramento forçado por orçamento (_Beast Mode_), que devem ser portados diretamente para Python.

### A Matriz de Canibalização ("O Que Copiar vs. O Que Descartar")

A **MELHOR OPÇÃO** arquitetural consiste em executar uma **Fusão Antropofágica Tripartida em Python**: adotar o fluxo de estado e os _prompts_ do **`langchain-ai/open_deep_research`** como espinha dorsal de coordenação, acoplar-lhe a árvore recursiva e os filtros pré-LLM do **`assafelovic/gpt-researcher`** juntamente com o protocolo de SLMs **`HazyResearch/minions` (`MinionS`)**, e encapsular todo o conjunto dentro dos contratos nativos de `delegate_task` e verificação do **`witt3rd/oh-my-hermes` + `moonlight-lupin/agent-skills`**.

|**Repositório Fonte**|**Ficheiros e Módulos Exatos a Canibalizar ("Copiar")**|**O Que Descartar (Evitar Conflito com o Hermes)**|**Destino na Arquitetura Hermes + Souls Engine**|
|---|---|---|---|
|**`langchain-ai/open_deep_research`**|`src/open_deep_research/prompts.py` (`clarify_with_user_instructions`, `transform_messages_into_research_topic_prompt`, `lead_researcher_prompt`, `research_system_prompt`, `compress_research_system_prompt`) e lógica de nós de `deep_researcher.py`.|Servidor LangGraph Studio, Open Agent Platform (OAP) e camada de persistência LangGraph (substituída por `SessionDB` e ficheiros `.md` do Hermes).|_Prompts_ do Orquestrador, ferramenta `think_tool` e perfil de compressão `compress_research`.|
|**`assafelovic/gpt-researcher`**|Lógica de decaimento em árvore (`breadth // 2`, `depth - 1`) de `gpt_researcher/skills/researcher.py`, filtro pré-LLM `BM25` / `Jev`, ingestão `DOC_PATH` e ciclo `Reviewer` $\rightarrow$ `Revisor` de `/multi_agents`.|Frontend Next.js/Tailwind próprio, servidor FastAPI `main.py` e _wrappers_ LangChain redundantes.|Ferramenta nativa `research_fetch_and_filter` e subagente de revisão.|
|**`HazyResearch/minions`**|`minions/minions.py` (protocolo `MinionS` plural), esquema Pydantic `StructuredLocalOutput` (`explanation`, `citation`, `answer`) e lógica de decomposição em _chunks_.|Aplicação Streamlit `app.py`, interface WebGPU e clientes LLM redundantes (substituídos pelo motor `llama.cpp` / `auxiliary_client.py` do Hermes).|Módulo `minions_filter.py` acionado pelos subagentes para leitura pesada em SLM local (3B–8B).|
|**`witt3rd/oh-my-hermes` & `moonlight-lupin/agent-skills`**|`skills/omh-deep-research/`, `role-research-verifier.md` (`toolsets=["file"]`), etiquetas epistémicas (`[VERIFIED]`, `[SOURCED]`, `[REASONED]`, `[ESTIMATED]`) e `source-tracker`.|Nenhum — compatibilidade 100% nativa com as primitivas do Hermes Agent.|Diretório `~/.hermes/skills/deepsoul-research/` e portões determinísticos de verificação.|
|**`Alibaba-NLP/DeepResearch` & `jina-ai/node-DeepResearch`**|Conceito de _Workspace Reconstruction_ + _Evolving Central Report_ (`IterResearch`), `gaps queue` com bloqueio de ações repetidas e _Beast Mode_.|Pesos do modelo Qwen3-30B (opcional) e _runtime_ Node.js/TypeScript do Jina.|Gestor de estado em disco (`state_manager.py`) e intercetor de orçamento no `PluginContext`.|
|**`stanford-oval/storm`**|Passo inicial de _Perspective-Guided Question Asking_ (geração de personas/perspetivas divergentes antes do _outline_).|Dependência pesada da biblioteca DSPy e motor Wiki específico.|Fase 1 de planeamento do Orquestrador (`write_research_brief`).|

## 5. A Proposta de Construção: Arquitetura "Hermes-DeepSoul" (Blueprint para Agente de IA)

Esta secção está estruturada como uma **Especificação Técnica Executável (_Context Blueprint_)**, desenhada para ser fornecida diretamente como contexto a um agente de engenharia de software (como o próprio Hermes Agent, Claude Code ou Codex) para implementar a solução completa sem ambiguidades.

### 5.1. Topologia do Sistema e Fluxo de Execução em 6 Fases

O motor **Hermes-DeepSoul** opera como um pacote híbrido composto por um **Plugin Nativo Hermes** (`~/.hermes/plugins/deepsoul_research/`) e um **Conjunto de Skills Modulares** (`~/.hermes/skills/deepsoul-research/`), coexistindo sem conflitos com o **`Hermes_Soul_patch` (`Souls Engine`)**.

#### Fase 0: Admissão via Souls Engine e Orçamentação de Esforço

Quando o utilizador solicita uma investigação profunda (ou invoca o comando `/deep-research <tema>`), o gancho `pre_llm_call` do `Hermes_Soul_patch` preserva a identidade `SOUL.md` e regista automaticamente na agenda diária (`$HERMES_HOME/companion/events.json`) um compromisso de investigação ativa (`kind="self"`, `title="Deep Research: <slug>"`). Nesse instante, o _plugin_ `deepsoul_research` ativa um escudo de estado emocional (`emotion_shield.lock`) que impede que erros transitórios de raspagem nos subagentes reduzam a valência (`valence`) em `EMOTION_STATE.md`, e classifica a complexidade da tarefa segundo as regras de escalonamento da Anthropic: tarefas de Nível 1 (_Fact-finding_) alocam 1 subagente com `depth=1` e `breadth=2`; tarefas de Nível 2 (_Comparativo_) alocam 3 subagentes com `depth=2` e `breadth=3`; e tarefas de Nível 3 (_Investigação Exaustiva_) alocam até 5 subagentes concorrentes por ronda com `depth=3` e `breadth=4`.

#### Fase 1: Scoping Socrático e Descoberta de Perspetivas (Cérebro Cloud — Tier 3)

Combinando a lógica dos nós `clarify_with_user` e `write_research_brief` do `langchain-ai/open_deep_research` com a geração de perspetivas do `stanford-oval/storm`, o Cérebro Cloud analisa o pedido inicial. Se o parâmetro `allow_clarification` estiver ativo e o pedido apresentar ambiguidades materiais ou siglas indefinidas, o agente formula no máximo três perguntas diretas acompanhadas de padrões recomendados (seguindo o padrão de entrevista socrática de `omh-deep-interview` e `Salomondiei08/oh-my-hermes`). Uma vez confirmado o escopo, o Cérebro Cloud identifica entre 3 a 5 perspetivas disciplinares divergentes sobre o tema e persiste o contrato mestre no disco em `$HERMES_HOME/research/{run_id}/00_research_brief.json`, contendo os objetivos específicos, as fronteiras de exclusão, a hierarquia de fontes prioritárias e a árvore inicial de sub-tópicos.

#### Fase 2: Planeamento do Supervisor e Reconstrução de Espaço de Trabalho (IterResearch — Tier 3)

Seguindo a arquitetura de isolamento estrito documentada na comunidade Hermes (`deep-researcher`), o Orquestrador Principal **nunca toca diretamente na web** — não invoca `web_search` nem ferramentas de navegador para proteger a sua janela de contexto de 200.000 _tokens_ contra poluição por ruído. Em cada ronda de profundidade $d \in \{1 \dots \text{depth}\}$, o Orquestrador invoca obrigatoriamente a ferramenta `think_tool` antes de qualquer delegação para avaliar o `00_research_brief.json`, o relatório central em evolução (`01_evolving_report.md`) e a fila de lacunas abertas (`gaps_queue.json`). Após verificar o registo de desduplicação (`dedup_registry.json`) para garantir que nenhum subagente recebe sub-tópicos sobrepostos, o Orquestrador dispara um lote paralelo via `delegate_task(tasks=[...], profile="fast-scout")`, fornecendo a cada trabalhador um objetivo atómico, instruções explícitas sobre o que não investigar, o caminho de gravação no sistema de ficheiros e um `output_schema` JSON obrigatório.

#### Fase 3: Execução Paralela de Subagentes e Escada de Extração Claw (Tier 2 — FAST_LLM)

Cada subagente (`Sub-researcher`) executa o seu trabalho numa sessão isolada recorrendo a um modelo rápido e económico (`deepseek-v3.2`, `gpt-5.4-mini` ou `gemini-2.0-flash`), operando sob um orçamento rígido de no máximo **8 buscas**, **12 extrações de páginas** e recolha mínima de **3 a 5 fontes primárias ou secundárias**. O subagente consulta inicialmente a base de conhecimento local partilhada (como `OpenViking` ou o histórico FTS5 do `SessionDB`) para reutilização a custo zero e, ao ir à web, seleciona o motor de busca consoante a intenção da _query_ — Exa para amplitude conceptual, Tavily para velocidade, Linkup (`linkup_search` em modo `deep`) ou You.com MCP para dados verificáveis e financeiros. Aplicando a estratégia _Broad-to-Narrow_, o subagente inicia com buscas curtas e, após cada ação, regista a tríade `Found / Confirms / Gaps`, direcionando a _query_ seguinte exclusivamente para o `Gap` identificado enquanto aplica as travas do Jina (desativando novas buscas se não surgirem URLs inéditos face ao `dedup_registry.json`). Para extrair o texto das URLs escolhidas, aciona a escada de extração `Claw` (`claw_extractor.py`), que tenta primeiro `HTTP simples + HTML parsing` e escala automaticamente para `TLS Fingerprint Impersonation`, `Stealth Browser (Camoufox / Playwright CDP)` ou `PDF / OCR Tesseract` apenas quando a leitura barata falha.

#### Fase 4: Filtragem Determinística BM25/Jev e Compressão Local MinionS (Tier 0 + Tier 1 — LOCAL_SLM)

Para evitar que páginas extensas ou relatórios em PDF sobrecarreguem o contexto do subagente ou inflacionem a fatura de API, a ferramenta de extração aplica imediatamente o pipeline de dois estágios antes de devolver os resultados. No estágio Tier 0, o documento é segmentado em blocos (_chunks_) de cerca de 600 palavras (~800 a 1.000 _tokens_) e submetido ao filtro local `BM25` (ou ao modelo utilitário `Jev` se a chave `TYPESAFE_API_KEY` estiver presente), descartando instantaneamente os fragmentos irrelevantes sem custo de LLM. No estágio Tier 1, os fragmentos de alta utilidade são enviados em paralelo para o SLM local (3B a 8B de parâmetros, como Qwen2.5-7B ou Llama-3.1-8B no servidor `llama.cpp` gerido pelo Hermes ou Ollama) seguindo o protocolo **`MinionS`**. O SLM local executa uma instrução estrita de passo único sobre cada bloco para extrair citações literais, estatísticas exatas e entidades, devolvendo um objeto validado com as quatro etiquetas epistémicas (`[VERIFIED]`, `[SOURCED]`, `[REASONED]`, `[ESTIMATED]`) e a polaridade da evidência. O subagente grava então o artefacto completo e comprimido (segundo as regras do nó `compress_research` da LangChain) em `$HERMES_HOME/research/{run_id}/scouts/scout_{id}.md` e retorna ao Orquestrador apenas o JSON compacto validado pelo `output_schema`.

#### Fase 5: Reconstrução IterResearch, Síntese Narrativa e Portão de Verificação Read-Only (Tier 3)

Ao receber os resumos estruturados do lote de subagentes, o Orquestrador executa o ciclo de **Reconstrução do Espaço de Trabalho (`IterResearch`)**: atualiza o `01_evolving_report.md` e a `gaps_queue.json`, descartando o histórico bruto intermédio para evitar a asfixia cognitiva. Se `depth > 1` e persistirem lacunas críticas ou contradições entre fontes, o Orquestrador reduz a largura de busca para metade (`new_breadth = max(2, breadth // 2)`) e lança uma ronda cirúrgica de _follow-up_. Caso o `IterationBudget` partilhado de 90 turnos do Hermes atinja o limiar de segurança de 72 turnos (80% do orçamento), o _middleware_ do _plugin_ aciona o **"Beast Mode"**, bloqueando novas buscas e forçando a transição imediata para a síntese. Na fase de redação (`Writer`), o Cérebro Cloud lê todos os ficheiros `scout_{id}.md`, desduplica as fontes por URL normalizado, resolve contradições expondo as diferentes perspetivas com rigor analítico e redige o relatório completo em `$HERMES_HOME/research/{run_id}/02_draft_report.md`. De imediato, o Orquestrador invoca uma última chamada `delegate_task` para o perfil `verifier-readonly` (`toolsets=["file"]`), que audita cada afirmação e numeração de citação contra os artefactos brutos dos _scouts_ — se detetar qualquer alegação não suportada (e enquanto não for atingido o teto de 3 tentativas, _3-strike retry cap_), devolve o rascunho para correção ou move a informação em falta para uma secção explícita de lacunas; uma vez aprovado, grava o relatório confirmado em `.omh/research/{slug}-report.md` (`status: confirmed`) e o recibo `03_verification_receipt.json`, acionando o `Hermes_Soul_patch` para atualizar `events.json` para `done` e notificar proativamente o utilizador via _Heartbeat_.

### 5.2. Mapa de Ficheiros e Componentes para Implementação no Hermes

Para que um agente de IA construa esta arquitetura de forma determinística, a tabela seguinte especifica o mapa exato de ficheiros a criar dentro de `$HERMES_HOME` (`~/.hermes/`), detalhando o subsistema proprietário e a responsabilidade técnica de cada artefacto:

|**Caminho do Ficheiro em $HERMES_HOME (~/.hermes/)**|**Subsistema / Tipo**|**Responsabilidade Técnica e Origem Antropofágica**|
|---|---|---|
|`config.yaml`|Configuração Central Hermes|Define o Cérebro Cloud (`model.default`), descarrega tarefas `auxiliary` para SLMs locais e regista `delegation.backends.profile_routes`.|
|`profiles/fast-scout/config.yaml`|Perfil de Subagente (`Tier 2`)|Configura o modelo explorador rápido (`deepseek-v3.2`, `gpt-5.4-mini` ou `gemini-2.0-flash`) e chaves de busca web/MCP.|
|`profiles/fast-scout/SOUL.md`|Identidade de Subagente|Define uma persona estritamente analítica, cética e concisa para os exploradores, isolada da persona emocional principal.|
|`profiles/local-minion/config.yaml`|Perfil de Operário SLM (`Tier 1`)|Aponta para o servidor local `llama.cpp` gerido pelo Hermes ou Ollama (`qwen2.5:7b-instruct` / `llama3.1:8b`) para execução `MinionS`.|
|`profiles/verifier-readonly/config.yaml`|Perfil Auditor (`Tier 3`)|Restringe as ferramentas a `toolsets: ["file"]` (contrato `READ-ONLY` de `witt3rd/oh-my-hermes`) para auditoria de citações e factos.|
|`plugins/deepsoul_research/plugin.yaml`|Manifesto de Plugin Nativo|Declara `provides_tools` (`think_tool`, `research_fetch_and_extract`), `provides_hooks` e `config_schema` para a UI Desktop.|
|`plugins/deepsoul_research/__init__.py`|Motor do Plugin (`register(ctx)`)|Regista ferramentas, o _middleware_ `llm_execution` do _Beast Mode_ (aos 72 turnos) e o escudo de proteção do `EMOTION_STATE.md`.|
|`plugins/deepsoul_research/minions_filter.py`|Pipeline `Tier 0` + `Tier 1`|Implementa o _chunking_ deslizante, o ranking lexical local `BM25` (GPT-Researcher) e a extração paralela `MinionS` em SLM local.|
|`plugins/deepsoul_research/claw_extractor.py`|Escada de Extração Web|Executa a escalada progressiva: `HTTP` $\rightarrow$ `HTML Parser` $\rightarrow$ `TLS Impersonation` $\rightarrow$ `Camoufox/Browser` $\rightarrow$ `PDF/OCR`.|
|`plugins/deepsoul_research/state_manager.py`|Persistência Atómica (`IterResearch`)|Gere leituras e escritas com `fcntl.flock` para `00_research_brief.json`, `01_evolving_report.md`, `dedup_registry.json` e `gaps_queue.json`.|
|`skills/deepsoul-research/SKILL.md`|Playbook do Orquestrador|Combina o `lead_researcher_prompt` (LangChain), a recursão `breadth // 2` (GPT-Researcher) e a reconstrução `IterResearch` (Tongyi).|
|`skills/deepsoul-research/references/scout-instructions.md`|Instruções de Subagente (`Level 2`)|Carregado sob demanda pelos _scouts_; impõe a busca _Broad-to-Narrow_, a tríade `Found/Confirms/Gaps` e as 4 etiquetas epistémicas.|
|`skills/deepsoul-research/references/verifier-rubric.md`|Rubrica de Verificação (`Level 2`)|Define os portões determinísticos de validação (`moonlight-lupin`) e o protocolo `CitationAgent` da Anthropic.|

### 5.3. Especificação de Configuração (`~/.hermes/config.yaml`)

O bloco seguinte define a configuração exata que habilita o roteamento multi-modelo, a proteção de concorrência em `delegate_task` e o descarregamento de tarefas auxiliares para modelos locais e baratos no Hermes Agent:

```YAML
# ~/.hermes/config.yaml — Configuração Híbrida Cloud Brain + Fast Scouts + Local SLM MinionS
model:
  default: anthropic/claude-sonnet-4-5   # Cérebro Cloud Principal (ou openai/gpt-5.4, claude-opus-4)
  provider: openrouter

# Descarregamento de tarefas secundárias do Hermes para modelos baratos/locais
auxiliary:
  compression:
    provider: custom
    model: qwen2.5:7b-instruct
    base_url: http://localhost:11434/v1
    api_key: local
  triage:
    provider: custom
    model: qwen2.5:7b-instruct
    base_url: http://localhost:11434/v1
    api_key: local
  title:
    provider: custom
    model: qwen2.5:3b-instruct
    base_url: http://localhost:11434/v1
    api_key: local

# Configuração do Sistema de Delegação e Rotas de Perfis Especialistas
delegation:
  max_concurrent_children: 6             # Permite fan-out paralelo de até 6 subagentes por ronda
  max_spawn_depth: 2                     # Permite que subagentes orquestradores lancem filhos se necessário
  orchestrator_enabled: true
  fallback_providers:
    - openrouter
    - custom
  backends:
    profile_routes:
      fast-scout:
        profile: fast-scout              # Usa deepseek-v3.2 ou gpt-5.4-mini para exploração web rápida
        description: "Subagente explorador web rápido para recolha e triagem de fontes"
      local-minion:
        profile: local-minion            # Usa SLM local (llama.cpp / Ollama 7B-8B) para leitura de chunks
        description: "Operário SLM local para extração estruturada sobre fragmentos longos"
      verifier-readonly:
        profile: verifier-readonly       # Usa modelo forte com toolsets restritos a ficheiros
        description: "Auditor determinístico de citações e verificação factual Read-Only"

plugins:
  entries:
    deepsoul_research:
      enabled: true
      allow_tool_override: false
      settings:
        default_breadth: 4
        default_depth: 2
        context_filter_mode: "bm25"      # Opções: "bm25" (local zero-cost) | "jev" | "hybrid"
        local_slm_endpoint: "http://localhost:11434/v1"
        local_slm_model: "qwen2.5:7b-instruct"
        beast_mode_budget_threshold: 72  # Aciona Beast Mode aos 72 turnos (de 90 do IterationBudget)
```

### 5.4. Implementação do Plugin, Filtro `MinionS` e Contratos de Esquema

O código abaixo especifica o núcleo do _plugin_ `~/.hermes/plugins/deepsoul_research/__init__.py` e do processador `minions_filter.py`, demonstrando o uso da API oficial `PluginContext` (`ctx`), a interceção de _middlewares_ (`hermes.middleware.v1`), a filtragem `BM25` e a decomposição atómica para o SLM local:

```Python
# ~/.hermes/plugins/deepsoul_research/__init__.py
import json
import os
import fcntl
from pathlib import Path
from typing import Dict, Any, List, Optional
from pydantic import BaseModel, Field

# Esquema Pydantic canibalizado do HazyResearch/minions (Protocolo MinionS)
# Garante instruções de passo único sobre chunks curtos (<1500 tokens)
class StructuredLocalChunkOutput(BaseModel):
    is_relevant: bool = Field(description="True se o fragmento contiver factos úteis para a sub-pergunta.")
    epistemic_tag: str = Field(description="Etiqueta obrigatória: [VERIFIED], [SOURCED], [REASONED] ou [ESTIMATED].")
    polarity: str = Field(description="Relação com a hipótese: 'supports', 'refutes' ou 'neutral'.")
    verbatim_quotes: List[str] = Field(description="Citações exatas e estatísticas extraídas palavra por palavra.")
    condensed_finding: str = Field(description="Síntese factual direta em 1 a 3 frases com números preservados.")

# Esquema JSON (output_schema) exigido pelo delegate_task aos subagentes Fast-Scout
SCOUT_OUTPUT_SCHEMA: Dict[str, Any] = {
    "type": "object",
    "required": [
        "subtopic", "artifact_file_path", "executive_summary",
        "sources_collected", "discovered_gaps", "contradictions_found"
    ],
    "properties": {
        "subtopic": {"type": "string"},
        "artifact_file_path": {
            "type": "string",
            "description": "Caminho absoluto do ficheiro Markdown onde as extrações densas foram gravadas."
        },
        "executive_summary": {
            "type": "string",
            "description": "Resumo denso (máx. 400 palavras) das descobertas para reconstrução IterResearch."
        },
        "sources_collected": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["url", "title", "tier", "epistemic_label"],
                "properties": {
                    "url": {"type": "string"},
                    "title": {"type": "string"},
                    "tier": {"type": "string", "enum": ["primary", "secondary", "tertiary"]},
                    "epistemic_label": {"type": "string", "enum": ["[VERIFIED]", "[SOURCED]", "[REASONED]", "[ESTIMATED]"]}
                }
            }
        },
        "discovered_gaps": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Novas perguntas concretas que ficaram sem resposta nesta ronda."
        },
        "contradictions_found": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Divergências factuais detetadas entre diferentes fontes primárias/secundárias."
        }
    }
}

def _atomic_update_emotion_shield(hermes_home: Path, shield_active: bool) -> None:
    """Protege o EMOTION_STATE.md do Souls Engine contra quedas de valência durante scraping massivo."""
    flag_file = hermes_home / "plugin-data" / "deepsoul_research" / "emotion_shield.lock"
    flag_file.parent.mkdir(parents=True, exist_ok=True)
    with open(flag_file, "w", encoding="utf-8") as f:
        fcntl.flock(f.fileno(), fcntl.LOCK_EX)
        try:
            json.dump({"shield_active": shield_active}, f)
        finally:
            fcntl.flock(f.fileno(), fcntl.LOCK_UN)

def register(ctx) -> None:
    """Ponto de entrada oficial do Plugin Nativo Hermes Agent."""
    
    # 1. Registar a ferramenta de reflexão estratégica (canibalizada de langchain-ai/open_deep_research)
    ctx.register_tool(
        name="think_tool",
        toolset="deepsoul_research",
        schema={
            "name": "think_tool",
            "description": (
                "Ferramenta obrigatória de reflexão estratégica. Deve ser chamada ANTES de delegate_task "
                "para planear a árvore de pesquisa e IMEDIATAMENTE APÓS cada delegate_task para avaliar "
                "a cobertura, atualizar o relatório central e decidir se avança na profundidade ou conclui."
            ),
            "parameters": {
                "type": "object",
                "required": ["reflection", "coverage_score", "next_action"],
                "properties": {
                    "reflection": {"type": "string", "description": "Análise crítica do que foi descoberto e do que falta."},
                    "coverage_score": {"type": "integer", "description": "Pontuação de 0 a 100 sobre a completude atual."},
                    "next_action": {"type": "string", "enum": ["spawn_scouts", "follow_up_gaps", "enter_synthesis"]}
                }
            }
        },
        handler=lambda args, **kw: json.dumps({"status": "reflection_logged", "next_action": args.get("next_action")}),
        description="Regista reflexão estruturada antes e depois da delegação de pesquisa"
    )

    # 2. Middleware de Controlo de Orçamento ("Beast Mode" do Jina + Proteção do IterationBudget de 90 turnos)
    def budget_and_loop_middleware(payload: Dict[str, Any], next_call):
        turn_count = payload.get("context", {}).get("turn_count", 0)
        threshold = 72  # 80% do limite rígido de 90 turnos do Hermes Agent
        if turn_count >= threshold:
            messages = payload.get("request", {}).get("messages", [])
            messages.append({
                "role": "system",
                "content": (
                    "[BEAST MODE ATIVADO - ORÇAMENTO DE TURNOS CRÍTICO]: Suspenda imediatamente novas "
                    "chamadas de busca ou delegação exploratória. Consolide todos os artefactos já "
                    "gravados em disco e avance diretamente para a síntese final e verificação."
                )
            })
        return next_call(payload)

    ctx.register_middleware("llm_execution", budget_and_loop_middleware)
```

Para o processamento de documentos longos e páginas raspadas sem consumir _tokens_ do modelo principal, o módulo `minions_filter.py` implementa o pipeline **BM25 + Protocolo `MinionS`**, onde o SLM local executa tarefas de passo único em lote:

```Python
# ~/.hermes/plugins/deepsoul_research/minions_filter.py
import asyncio
import json
import re
from typing import List, Dict, Any
import httpx

def chunk_text_sliding(text: str, max_words: int = 600, overlap: int = 80) -> List[str]:
    """Divide páginas longas em blocos curtos (<1K tokens) para evitar a queda de 13% nos SLMs."""
    words = text.split()
    chunks = []
    step = max(1, max_words - overlap)
    for i in range(0, len(words), step):
        chunk = " ".join(words[i : i + max_words])
        if len(chunk.strip()) > 100:
            chunks.append(chunk)
    return chunks

def bm25_pre_filter(query: str, chunks: List[str], top_k: int = 8) -> List[str]:
    """Filtro Tier 0 (Custo Zero, inspirado no GPT-Researcher) para descartar ruído HTML/SEO."""
    query_terms = set(re.findall(r"\w+", query.lower()))
    if not query_terms or len(chunks) <= top_k:
        return chunks[:top_k]
    
    scored = []
    for idx, chunk in enumerate(chunks):
        chunk_words = re.findall(r"\w+", chunk.lower())
        if not chunk_words:
            continue
        # Pontuação lexical rápida normalizada pelo tamanho do bloco
        matches = sum(1 for w in chunk_words if w in query_terms)
        unique_matches = len(query_terms.intersection(set(chunk_words)))
        score = (unique_matches * 3.0) + (matches / (len(chunk_words) ** 0.5))
        scored.append((score, idx, chunk))
    
    scored.sort(key=lambda x: x[0], reverse=True)
    # Reordena pela posição original no documento para preservar a coesão lógica
    top_chunks = sorted(scored[:top_k], key=lambda x: x[1])
    return [c[2] for c in top_chunks]

async def execute_minions_single_step_slm(
    atomic_question: str,
    filtered_chunks: List[str],
    source_url: str,
    slm_endpoint: str = "http://localhost:11434/v1/chat/completions",
    slm_model: str = "qwen2.5:7b-instruct"
) -> List[Dict[str, Any]]:
    """
    Executa o protocolo MinionS (HazyResearch): o SLM local (3B-8B) recebe uma única
    instrução atómica sobre cada bloco curto em paralelo, devolvendo JSON estrito.
    """
    async def process_single_chunk(client: httpx.AsyncClient, chunk_idx: int, chunk_text: str) -> Dict[str, Any]:
        single_step_prompt = (
            f"Pergunta Atómica: {atomic_question}\n"
            f"Fonte: {source_url} (Fragmento #{chunk_idx})\n"
            f"Texto do Fragmento:\n\"\"\"\n{chunk_text}\n\"\"\"\n\n"
            "Instrução de Passo Único: Extraia APENAS factos, números exatos e citações literais "
            "presentes no texto acima que respondam diretamente à Pergunta Atómica. Não invente nem extrapole. "
            "Responda estritamente em JSON com as chaves: "
            "is_relevant (bool), epistemic_tag ('[VERIFIED]'|'[SOURCED]'|'[REASONED]'|'[ESTIMATED]'), "
            "polarity ('supports'|'refutes'|'neutral'), verbatim_quotes (lista de strings), condensed_finding (string)."
        )
        payload = {
            "model": slm_model,
            "messages": [{"role": "user", "content": single_step_prompt}],
            "temperature": 0.0,
            "response_format": {"type": "json_object"}
        }
        try:
            resp = await client.post(slm_endpoint, json=payload, timeout=30.0)
            resp.raise_for_status()
            data = resp.json()
            parsed = json.loads(data["choices"][0]["message"]["content"])
            if parsed.get("is_relevant"):
                parsed["chunk_index"] = chunk_idx
                parsed["source_url"] = source_url
                return parsed
        except Exception:
            # Fallback resiliente caso o SLM falhe a formatação
            return {}
        return {}

    async with httpx.AsyncClient() as client:
        tasks = [process_single_chunk(client, i, ch) for i, ch in enumerate(filtered_chunks)]
        results = await asyncio.gather(*tasks)
    
    return [r for r in results if r and r.get("is_relevant")]
```

## 6. Diretrizes de Execução e Roteiro de Implementação

Para garantir que esta arquitetura atinja a qualidade final dos relatórios do Google Gemini Deep Research e do OpenAI Deep Research dentro das fronteiras técnicas do Hermes Agent, a implementação deve observar três salvaguardas operacionais críticas identificadas na documentação de restrições do ecossistema (`hermes-constraints.md`). Em primeiro lugar, é necessário contornar o bloqueio nativo das ferramentas `execute_code` e `memory` nos subagentes folha (`role="leaf"`). Como os subagentes não podem executar scripts programáticos via `execute_code` nem escrever diretamente na memória partilhada, toda a computação em lote (como agregação estatística em Python/DuckDB ou geração de tabelas de dados) tem de ser executada pelo **Orquestrador Principal** — que beneficia ainda do reembolso automático de turnos no `IterationBudget` ao usar `execute_code` — ou encapsulada dentro da ferramenta `research_fetch_and_extract` registada pelo _plugin_, cabendo também exclusivamente ao Orquestrador persistir as conclusões duráveis em `MEMORY.md` no final da Fase 5.

Em segundo lugar, deve implementar-se um mecanismo determinístico de prevenção de paragem prematura (_Stop-Prevention Sentinel_). Uma vez que o Hermes Agent não dispõe de um _hook_ de veto de encerramento de sessão (`pre_session_end`), modelos de linguagem podem tentar entregar um relatório superficial logo após a primeira ronda exploratória. Para neutralizar esta tendência, a competência `deepsoul-research/SKILL.md` e o _plugin_ exigem a verificação mecânica de um ficheiro sentinela: o Orquestrador é contratualmente impedido de finalizar a resposta ao utilizador enquanto o ficheiro `$HERMES_HOME/research/{run_id}/03_verification_receipt.json` (gerado exclusivamente pelo subagente `verifier-readonly` com `"status": "CONFIRMED"` e espelhado em `.omh/research/{slug}-report.md`) não existir no sistema de ficheiros. Em terceiro lugar, a solução deve capitalizar o ciclo fechado de auto-evolução do Hermes (`skill_manage` aliado ao motor evolucionário _offline_ **GEPA** + DSPy em `NousResearch/hermes-agent-self-evolution`, cuidando para auditar a tendência documentada de auto-congratulação do Curador) para destilar heurísticas de pesquisa específicas de domínio em novas sub-competências reutilizáveis, enquanto o `Souls Engine` (`Hermes_Soul_patch`) transforma a conclusão de relatórios assíncronos numa experiência proativa através de `events.json` e `hermes cron`.

O processo de realização técnica para o agente construtor divide-se em três fases sequenciais de implementação e validação, sintetizadas na tabela abaixo:

|**Incremento de Construção**|**Componentes Alvo**|**Comandos de Validação e Critérios de Aceitação no Hermes Agent**|
|---|---|---|
|**Incremento 1: Fundação de Perfis e Motor SLM Local**|`~/.hermes/config.yaml` e diretórios `~/.hermes/profiles/{fast-scout,local-minion,verifier-readonly}`.|Executar `hermes model` $\rightarrow$ _Local models_ (ou iniciar Ollama local com `qwen2.5:7b-instruct`), configurar `credential pools` via `hermes auth` e validar que `delegate_task` roteia corretamente para cada perfil em `profile_routes`.|
|**Incremento 2: Plugin Nativo, Filtro `MinionS` e Escada `Claw`**|Pacote `~/.hermes/plugins/deepsoul_research/` (`plugin.yaml`, `__init__.py`, `minions_filter.py`, `claw_extractor.py`, `state_manager.py`).|Executar `hermes plugins validate ~/.hermes/plugins/deepsoul_research --install-deps` e `hermes plugins doctor deepsoul_research`, confirmando o registo de `think_tool`, a filtragem `BM25` + `MinionS` e o disparo do _Beast Mode_ aos 72 turnos.|
|**Incremento 3: Orquestração `IterResearch` e Acoplamento ao Souls Engine**|Diretório `~/.hermes/skills/deepsoul-research/` (`SKILL.md`, `scout-instructions.md`, `verifier-rubric.md`) e integração com `Hermes_Soul_patch`.|Executar uma tarefa completa de _Deep Research_ verificando a criação dos artefactos em `$HERMES_HOME/research/{run_id}/`, a validação `READ-ONLY` das 4 etiquetas epistémicas em `.omh/research/{slug}-report.md` e a entrega via _Proactive Heartbeat_ sem degradação da `valence` em `EMOTION_STATE.md`.|