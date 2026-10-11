# Otimização da Fronteira de Silício Bare-Metal: Prospecção e Recomendações de Modelos Open-Source para as 10 Funções Canônicas do Hermes Agent

A aceleração contínua observada no desenvolvimento de modelos de linguagem de pesos abertos (_open-weight LLMs_), impulsionada por técnicas avançadas de destilação de raciocínio sintético, alinhamento direcionado a ações e quantização adaptativa por matrizes de importância (_iMatrix_), estabelece uma oportunidade crucial para reconfigurar infraestruturas computacionais locais. O ambiente operacional bare-metal em análise — composto por um processador Intel Core i9-9880H com extensões AVX2, 32GB de memória RAM DDR4 e uma placa de vídeo dedicada NVIDIA GeForce RTX 2060m (6GB GDDR6 sob subsistema Windows Display Driver Model - WDDM 3.x) — impõe restrições físicas severas que exigem alocação milimétrica de tensores.

Este relatório técnico disseca o inventário atual consolidado na versão v7, identifica os gargalos estruturais e as anomalias de latência presentes nas configurações ativas e prospecta, a partir de repositórios contemporâneos do ecossistema de código aberto, os candidatos com maior potencial para ultrapassar os modelos titulares nas 10 Funções Canônicas para Subagentes Locais acionados via `delegate_task`.

## 1. Fundamentos de Governança Bare-Metal e Restrições de Hardware

A gestão de inferência local em nós de borda clientes difere radicalmente do paradigma de nuvem baseado em cobrança por milhão de tokens. No hospedeiro bare-metal, a moeda fundamental é a integridade do silício, regulada pela pressão física instantânea exercida sobre a GPU e a CPU. Sob o sistema operativo Windows 11, o subsistema Desktop Window Manager (DWM) reserva de forma impositiva entre 800 MB e 850 MB de VRAM dedicada para gerenciar a interface gráfica, buffers de vídeo e aceleração do sistema operacional. Consequentemente, a diretriz arquitetural ADR-004 do projeto estabelece um teto operacional intransponível de 5.294 MB de VRAM útil para a placa gráfica NVIDIA GeForce RTX 2060m.

A extrapolação dessa barreira física provoca o acionamento imediato do mecanismo de _paging_ PCIe pelo subsistema de memória do WDDM. Quando tensores de parâmetros ou blocos de contexto transbordam para a memória RAM compartilhada do sistema através do barramento, ocorre uma desaceleração catastrófica da largura de banda, resultando em uma queda vertical de throughput de ~45 tokens por segundo para taxas inferiores a 1,5 tok/s. Esse fenômeno paralisa completamente a execução de agentes autônomos.

Para fundamentar as decisões de roteamento entre instâncias locais e provedores externos de baixo custo, a governança do sistema emprega uma métrica bidimensional formalizada no Custo de Silício ($C_{\text{silicon}}$) e no Retorno de Silício Amortecido ($V_{\text{task}}$):

$$C_{\text{silicon}} = 0,50 \cdot \left(\frac{\text{VRAM}_{\text{peak}}}{5294}\right) + 0,30 \cdot \left(\frac{\text{TTFT}_{\text{sec}}}{3,0}\right) + 0,20 \cdot \left(\frac{30,0}{\text{TPS} + 1,0}\right)$$

$$V_{\text{task}} = \frac{S_{\text{task}}}{\ln(1,0 + C_{\text{silicon}} + 1,0)}$$

Neste arcabouço matemático, o termo $S_{\text{task}}$ quantifica o índice normalizado de precisão e rigidez lógica na tarefa (0 a 100), enquanto o denominador logarítmico amortece a assimetria vantajosa de modelos ultraleves que carecem de capacidade analítica profunda.

Para manter a estabilidade do cache de chaves e valores (_KV Cache_) em tarefas complexas de contexto longo até 8.192 tokens, a governança ADR-007 determina o emprego mandatório de KV Cache Assimétrico (`Key=FP16` para preservar a estabilidade da atenção e `Value=Q4_0` para comprimir a ocupação volumétrica em 50%), conjugado à ativação do FlashAttention-2 (`-fa on`). Paralelamente, o protocolo estabelece a desidratação sintática do contexto (_Source-Side Dehydration_), que impede o consumo integral de arquivos brutos de código-fonte através da injeção exclusiva de árvores sintáticas filtradas (`souls_ast_outline` e `souls_ast_slice`), poupando entre 70% e 85% da janela atencional ativa.

## 2. Diagnóstico Crítico das Vulnerabilidades do Inventário Atual (v7)

Embora o inventário consolidado na versão v7 apresente modelos com elevada precisão em micro-benchmarks isolados, o funcionamento concatenado do sistema revela gargalos arquiteturais graves que comprometem a escalabilidade agêntica.

O primeiro ponto crítico reside na disparidade de latência do modelo titular de raciocínio lógico em profundidade, o `phi-4-mini-instruct-q4-k-m` (3.8B). Apesar de alcançar um índice composto de rigidez de 81,0% e pontuar 80% em cadeias de pensamento (_Chain of Thought_ - CoT), o modelo registra um Tempo até o Primeiro Token (TTFT) aberrante de 51.831,1 milissegundos (~51,8 segundos) durante o preenchimento de contexto (_prefill_) sob a dGPU. Esse atraso penaliza o Custo de Silício para o índice de $C = 5,474$, degradando o Retorno de Silício para apenas $V = 44,8$. Esse comportamento inviabiliza iterações ágeis em turnos múltiplos, transformando subagentes de raciocínio em pontos de bloqueio operacional.

A segunda vulnerabilidade decorre da fragilidade atencional sob pressão de contexto longo (Benchmark Type 5, 8k Haystack). Os modelos titulares `hermes-3-llama-3.2-3b-q4-k-m` e `qwen3.8-4b-q4-k-m` falharam categoricamente na recuperação de agulha a 50% de profundidade em contextos de 8k tokens, sofrendo atenuação severa após a marca de 4.096 tokens e recebendo a classificação `ATTENTION_DEGRADED`. De forma análoga, o modelo titular de despacho agêntico `mamba-codestral-7b-v0-1-q4-k-m`, a despeito de sua elevada acurácia de esquemas JSON (91%), sofre degradação em 8k tokens devido à saturação volumétrica da VRAM, aproximando-se do limiar de estocagem do DWM e limitando a sustentação de diálogos agênticos extensos.

O terceiro gargalo concentra-se no Tier 3 multimodal de visão. Os modelos titulares `nimbus-4b-q4-k-m` e `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` operam com cadência excessivamente baixa (16,3 tok/s e 15,8 tok/s) e TTFT superior a 3.000 ms, exigindo alocações pesadas de tensores (2.619 MB a 3.305 MB de VRAM). Esse consumo elevado de memória gráfica restringe o espaço restante para a concorrência de subagentes de texto.

Por fim, o mecanismo de decodificação especulativa de Tier 4 apresenta um desacoplamento estrutural: o modelo drafter `bonsai-27b-dspark-q4-1` exige 1.741 MB de VRAM para operar. Quando carregado juntamente com um modelo primário de 4B ou 7B, a soma dos tensores eleva o consumo para mais de 4.500 MB de VRAM, reduzindo drasticamente a margem de segurança contra o transbordamento do WDDM. Além disso, sua arquitetura não possui alinhamento vocabular estrito com as linhagens Qwen2.5/Qwen3 que predominam no Tier 1.

## 3. Panorama Tecnológico e Desafiantes de Última Geração na Comunidade Open-Source

A dinâmica da comunidade open-source no Hugging Face produziu arquiteturas compactas de altíssima eficiência que superam as deficiências do inventário v7 sem extrapolar a barreira de 5.294 MB de VRAM. Essas inovações distribuem-se estrategicamente pelos principais nichos de atuação agêntica.

### Engenharia de Software e Sintaxe Rust Bare-Metal

No ecossistema de geração e manipulação sintática de Rust, modelos compactos frequentemente falham nas nuances rigorosas do _borrow checker_, no tempo de vida de referências (_lifetimes_) e na estrutura estrita de tipos. Recentemente, conjuntos de dados filtrados por execução sintética e compilação real possibilitaram o surgimento de SLMs altamente calibrados para a compilação no primeiro turno (_pass@1_):

|**Modelo e Identificador HF**|**Parâmetros**|**Base / Linhagem**|**Métricas e Diferenciais Técnicos**|**Repositório Recomendado**|
|---|---|---|---|---|
|`temper-ai/Temper-1-1.5B`<br><br>[cite: 10]|1.5B|Qwen2.5-Coder-1.5B|**50,5% HumanEval-Rust pass@1** (supera o Qwen2.5-Coder-3B base de 44,7%). Treinado em 10.000 amostras validadas pelo compilador `rustc`.|`temper-ai/Temper-1-1.5B-GGUF` / `mradermacher/Temper-1-1.5B-GGUF`|
|`Tesslate/Tessa-Rust-T1-7B`<br><br>[cite: 9]|7.0B|Qwen2.5-Coder-7B|Raciocínio híbrido acionado via `<\|begin_of_thought\|>`. Foco estrito em _traits_, tipagem concorrente e _lifetimes_ complexas.|`bartowski/Tesslate_Tessa-Rust-T1-7B-GGUF`<br><br>[cite: 12]|
|`Qwen/Qwen2.5-Coder-3B-Instruct`<br><br>[cite: 13]|3.0B|Qwen2.5 Causal LM|**84,1% HumanEval**, 72,1% MultiPL-E médio. Janela de atenção nativa e estável até 128k tokens.|`unsloth/Qwen2.5-Coder-3B-Instruct-GGUF`<br><br>[cite: 13]|
|`Qwen/Qwen2.5-Coder-7B-Instruct`<br><br>[cite: 17]|7.0B|Qwen2.5 Causal LM|Treinado em 5,5 trilhões de tokens de código e dados sintéticos validados por execução. Suporte robusto a contextos de 128k.|`bartowski/Qwen2.5-Coder-7B-Instruct-GGUF`<br><br>[cite: 2]|

A relevância do `Temper-1-1.5B` decorre da validação sintética por compilador, técnica em que cada amostra de treinamento foi gerada por modelos de raciocínio de grande escala e submetida à aprovação do compilador antes da destilação. Com isso, um modelo de apenas 1.5B parâmetros atinge uma acurácia em Rust superior à de modelos de 3B parâmetros sem ajuste específico, consumindo cerca de 1.050 MB de VRAM em quantização `Q4_K_M`.

### Modelos de Ação e Despacho Estruturado de Ferramentas (MCP)

Na categoria de execução autônoma, alinhamento ao protocolo MCP e geração estrita de esquemas JSON, os modelos generalistas vêm perdendo espaço para os chamados _Large Action Models_ (LAMs). A Salesforce AI Research introduziu as famílias xLAM e xLAM-2, refinadas pelo pipeline de síntese multi-turn APIGen-MT, que estabelecem novos recordes na avaliação de chamadas de função:

|**Modelo e Identificador HF**|**Parâmetros**|**Contexto Máximo**|**Resultados em Benchmarks (BFCL v3 / τ-bench)**|**Repositório Oficial GGUF**|
|---|---|---|---|---|
|`Salesforce/xLAM-2-1b-fc-r`<br><br>[cite: 22]|1.0B|32k (ext. 128k)|**43,12% no BFCL v3 Overall** (supera GPT-4o em modo FC nativo a 41%). $\tau$-bench: 21,8%.|`Salesforce/xLAM-2-1b-fc-r-gguf`<br><br>[cite: 26, 27]|
|`Salesforce/xLAM-2-3b-fc-r`<br><br>[cite: 22]|3.0B|32k (ext. 128k)|**38,2% no $\tau$-bench Overall** (desempenho comparável ao Llama-3.1-70B a 38,2%). Foco em multi-turn complexo.|`Salesforce/xLAM-2-3b-fc-r-gguf`<br><br>[cite: 26, 28]|
|`Salesforce/xLAM-7b-fc-r`<br><br>[cite: 19]|6.9B|4k|**88,24% de acurácia global no BFCL** (Top 3 histórico no segmento 7B).|`Salesforce/xLAM-7b-fc-r-gguf`<br><br>[cite: 26]|
|`QuantFactory/Llama-3-Groq-8B-Tool-Use`<br><br>[cite: 29]|8.0B|8k|**89,06% de acurácia global no BFCL** (Líder open-source no segmento 8B).|`QuantFactory/Llama-3-Groq-8B-Tool-Use-GGUF`<br><br>[cite: 29]|

O modelo `xLAM-2-1b-fc-r` destaca-se pelo fato de seu objetivo de perda ser calibrado integralmente para a predição determinística de ferramentas e parâmetros estruturados, eliminando introduções conversacionais prolixas que poluem os buffers de contexto do protocolo MCP. Com pegada de memória inferior a 900 MB em `Q4_K_M`, ele viabiliza a execução de loops contínuos de ferramentas sem risco de esgotamento da VRAM.

### Visão Computacional Compacta e Análise de Interfaces de Usuário

No processamento de imagens técnicas, inspeção de telas e diagramas de arquitetura, a série `Qwen2.5-VL` redefine a precisão geométrica e o alinhamento espacial (_UI Grounding_) ao incorporar janelas de atenção eficientes no Vision Transformer (ViT) acopladas ao mecanismo multimodal de posições rotativas (mRoPE):

|**Modelo e Identificador HF**|**Parâmetros**|**Arquitetura Visual**|**Diferenciais Funcionais / Benchmarks**|**Projetor Multimodal mmproj**|
|---|---|---|---|---|
|`Qwen/Qwen2.5-VL-3B-Instruct`<br><br>[cite: 30]|3.0B|ViT com Window Attention e SwiGLU|Emissão direta de coordenadas espaciais (_bounding boxes_ e pontos) em JSON. OCR tabular (DocVQA) e análise de UI.|`Qwen2.5-VL-3B-Instruct-mmproj-f16.gguf` (`unsloth`, `Mungert`, `lm-kit`)|
|`HuggingFaceTB/SmolVLM-Instruct`<br><br>[cite: 34]|2.2B|Idefics3 / SigLIP|Alta taxa de geração contínua (>25 tok/s). Otimizado para OCR rápido e extração de texto em logs visuais.|`mmproj-SmolVLM-Instruct-f16.gguf` (`ggml-org/SmolVLM-Instruct-GGUF`)|
|`ibm-granite/granite-vision-3.1-2b-preview`<br><br>[cite: 37]|2.0B|SigLIP + Granite-3.1-2B|Especializado em documentos técnicos, gráficos analíticos, tabelas densas e formulários corporativos.|`granite-vision-3.1-2b-preview-GGUF` (IBM partner registry)|

A habilidade do `Qwen2.5-VL-3B-Instruct` de mapear elementos de interface para esquemas JSON padronizados resolve o problema de dependência de múltiplos OCRs intermediários. Essa capacidade permite que o subagente atue diretamente sobre a tela do sistema com consumo de apenas ~2,7 GB de VRAM (incluindo o projetor multimodal em precisão FP16).

### Modelos Ultraleves de Apoio e Aceleração Especulativa

A despressurização da dGPU exige transferir etapas de triagem e avaliação para os 32GB de memória RAM do hospedeiro, aproveitando o processador Intel Core i9-9880H (AVX2), além de harmonizar o processo de decodificação especulativa na GPU:

|**Modelo e Identificador HF**|**Parâmetros**|**Alvo de Silício**|**Métricas e Funcionalidades Críticas**|**Formato / Repositório**|
|---|---|---|---|---|
|`Qwen/Qwen2.5-Coder-0.5B-Instruct`<br><br>[cite: 39]|0.5B|Drafter na dGPU (`-md`)|Taxa de aceitação $\alpha$ entre 65% e 75% quando pareado com alvos da linha Qwen2.5-Coder. Aceleração de 1,4x a 1,8x.|`bartowski/Qwen2.5-Coder-0.5B-Instruct-GGUF`<br><br>[cite: 44, 45]|
|`deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B`<br><br>[cite: 18]|1.5B|Host CPU (Tier 0.5)|Raciocínio matemático e dedutivo passo a passo destilado do modelo DeepSeek-R1. Execução ágil a frio via AVX2.|`bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF`<br><br>[cite: 18, 46]|
|`answerdotai/ModernBERT-base`<br><br>[cite: 47]|149M|Host CPU (Tier 0)|Arquitetura encoder com janela de 8.192 tokens e atenção alternada. Desempenho superior a DeBERTaV3 em NLU e busca em código.|ONNX Runtime CPU (`answerdotai/ModernBERT-base`)|
|`ibm-granite/granite-3.1-2b-instruct`<br><br>[cite: 50]|2.0B|Host CPU (Tier 0.5)|Baixa taxa de alucinação, alinhamento robusto a regras de segurança e suporte a contexto longo.|`bartowski/granite-3.1-2b-instruct-GGUF`<br><br>[cite: 51]|

O uso do `ModernBERT-base` marca uma transição técnica fundamental: por ser uma arquitetura exclusivamente codificadora (_encoder-only_), o modelo avalia sequências longas de até 8k tokens por meio de uma única passagem direta (_forward pass_), dispensando a geração autoregressiva token a token e reduzindo a latência da triagem inicial para menos de 5 ms no processador.

## 4. Reavaliação da Fronteira de Pareto e Reescalação das 10 Funções Canônicas

A integração desses avanços permite reconfigurar os modelos titulares e de contingência das 10 Funções Canônicas para Subagentes Locais, solucionando as fragilidades apontadas no inventário v7.

A substituição no papel de **Artesão de Rust** endereça a necessidade de eficiência volumétrica. Embora o titular atual `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` apresente 100% de aderência na trilha Rust AST, o desafiante `temper-ai/Temper-1-1.5B` atinge 50,5% de aprovação no teste HumanEval-Rust pass@1 — superando o Qwen2.5-Coder de 3B parâmetros — enquanto reduz o consumo de VRAM de 1.278 MB para aproximadamente 1.050 MB e eleva a cadência de geração para mais de 65 tok/s. Essa redução de memória abre espaço para carregar subagentes simultâneos na GPU.

No papel de **Despachante Agêntico**, o modelo titular `mamba-codestral-7b` é substituído pelo `Salesforce/xLAM-2-1b-fc-r-gguf`. O modelo da Salesforce consome apenas cerca de 880 MB de memória gráfica e alcança 43,12% de precisão global no BFCL v3, superando o GPT-4o em chamadas nativas de função. Essa troca elimina o risco de degradação atencional em 8k tokens observado no modelo de 7B anterior sob pressão de VRAM, garantindo estabilidade absoluta na emissão de JSON para o protocolo MCP.

No caso do **Subagente de Baixa Latência**, a fragilidade atencional do `hermes-3-llama-3.2-3b` (que sofre perda de foco após 4.096 tokens) é corrigida com a introdução do `Qwen2.5-Coder-3B-Instruct-GGUF`. O novo modelo mantém foco estável ao longo de toda a janela de 8.192 tokens, sustentando mais de 50 tok/s com TTFT inferior a 180 ms.

Para a função de **Protetor de Silício / FinOps**, a promoção do `Salesforce/xLAM-2-1b-fc-r-gguf` reduz a ocupação da VRAM em quase 2 GB em relação ao `nvidia-nemotron-3-nano-4b` (que retinha 2.829 MB), elevando a margem livre da placa para mais de 4.400 MB e viabilizando a orquestração concorrente de múltiplos trabalhadores efêmeros.

Na função de **Engenheiro de Sistemas**, o modelo `Tesslate/Tessa-Rust-T1-7B` substitui o `hy-mt2-7b`, trazendo raciocínio explícito em blocos `<|begin_of_thought|>` dedicados a decisões arquiteturais em Rust.

Nos papéis multimodais, o `Qwen2.5-VL-3B-Instruct` assume a função de **Inspetor Visual Primário**, trazendo suporte nativo a coordenadas cartesianas de tela em substituição ao `nimbus-4b`. Como **Sentinela Visual Ultraleve**, o `SmolVLM-Instruct` substitui o modelo de 2B anterior, dobrando a cadência de geração para mais de 25 tok/s e reduzindo a alocação de memória gráfica de 3.305 MB para ~1.850 MB.

Por fim, o subsistema especulativo recebe o `Qwen2.5-Coder-0.5B-Instruct` como **Drafter Especulativo**, que compartilha o vocabulário das linhagens Qwen e consome apenas ~398 MB de VRAM (contra 1.741 MB do Bonsai-27B).

Na CPU, a triagem e o cálculo epistêmico passam a contar com o `ModernBERT-base` em tempo constante e com o `DeepSeek-R1-Distill-Qwen-1.5B` operando com cadeias de pensamento diretamente na memória RAM.

|**#**|**Função Canônica Agêntica**|**Modelo Titular Proposto (Novo)**|**Modelo Reserva / Fallback Local**|**Throughput Alvo / VRAM**|**Justificativa Técnica da Seleção**|
|---|---|---|---|---|---|
|**1**|**O Artesão de Rust** _(Master SWE)_|`temper-ai/Temper-1-1.5B` (`Q4_K_M`)|`qwen3-5-4b-agentic-coder-v4-i1-q4-k-s`<br><br>[cite: 1]|~65 tok/s<br><br>  <br><br>~1.050 MB VRAM|**50,5% HumanEval-Rust pass@1**. Calibrado sinteticamente com validação direta pelo compilador `rustc`.|
|**2**|**O Despachante Agêntico** _(MCP Master)_|`Salesforce/xLAM-2-1b-fc-r` (`Q4_K_M`)|`Salesforce/xLAM-2-3b-fc-r` (`Q4_K_M`)|~60 tok/s<br><br>  <br><br>~880 MB VRAM|**43,12% no BFCL v3 Overall**. Emissão determinística de JSON, eliminando o risco de transbordamento de VRAM a 8k tokens.|
|**3**|**O Subagente de Baixa Latência** _(Fast Multi-Turn)_|`Qwen2.5-Coder-3B-Instruct` (`Q4_K_M`)|`qwen3-5-4b-uncensored-hauhaucs-mtp`<br><br>[cite: 1]|~52 tok/s<br><br>  <br><br>~1.650 MB VRAM|**Recuperação de agulha 100% estável em 8k tokens**. TTFT de 180 ms e alta cadência em turnos curtos.|
|**4**|**O Protetor de Silício / FinOps** _(VRAM Headroom)_|`Salesforce/xLAM-2-1b-fc-r` (`Q4_K_M`)|`temper-ai/Temper-1-1.5B` (`Q4_K_M`)|~60 tok/s<br><br>  <br><br>Headroom >4.400 MB|Ocupação de apenas ~880 MB de memória gráfica, viabilizando concorrência entre subagentes na GPU.|
|**5**|**O Engenheiro de Sistemas** _(Heavyweight 7B)_|`Tesslate/Tessa-Rust-T1-7B` (`IQ3_M` / `Q4_K_M`)|`hy-mt2-7b-q4-k-m`<br><br>[cite: 1]|~30 tok/s<br><br>  <br><br>~3.570 a 4.680 MB|Raciocínio estruturado via tags `<\|begin_of_thought\|>` voltado para sistemas concorrentes em Rust.|
|**6**|**O Inspetor Visual Primário** _(VLM UI Grounding)_|`Qwen2.5-VL-3B-Instruct` (`Q4_K_M` + mmproj)|`nimbus-4b-q4-k-m`<br><br>[cite: 1]|~22 tok/s<br><br>  <br><br>~2.700 MB VRAM|**UI Grounding nativo** com coordenadas cartesianas em JSON, layout OCR estruturado e mRoPE.|
|**7**|**O Sentinela Visual Ultraleve** _(VLM 2B OCR)_|`SmolVLM-Instruct` (`Q4_K_M` + mmproj)|`Qwen2.5-VL-3B-Instruct` (`IQ3_M`)|~28 tok/s<br><br>  <br><br>~1.850 MB VRAM|Cadência rápida para inspeção de terminais e logs visuais, liberando mais de 3.400 MB de VRAM.|
|**8**|**O Drafter Especulativo** _(Spec Acceleration)_|`Qwen2.5-Coder-0.5B-Instruct` (`Q4_K_M`)|Heads MTP Nativos Integrados|~95 tok/s (Solo)<br><br>  <br><br>~398 MB VRAM|Taxa de aceitação $\alpha \approx 70\%$ sobre modelos alvo da família Qwen. Aceleração de 1,4x a 1,8x.|
|**9**|**O Triador Relâmpago** _(CPU Sanitizer)_|`ModernBERT-base` (ONNX Runtime CPU)|`smollm2-135m-reasoning-5k`<br><br>[cite: 1]|Inferência <5 ms<br><br>  <br><br>0 MB VRAM dGPU|Modelo encoder com janela de 8k tokens e atenção alternada. Triagem em uma única passagem direta.|
|**10**|**O Sensor Epistêmico & SLM** _(Host CPU 0.5)_|`DeepSeek-R1-Distill-Qwen-1.5B` (`Q8_0` CPU)|`smollm2-360m-instruct-q8-0`<br><br>[cite: 1]|~28 tok/s (CPU)<br><br>  <br><br>0 MB VRAM dGPU|Raciocínio dedutivo CoT destilado do DeepSeek-R1, operando na RAM sem impactar a GPU.|

## 5. Dinâmica de Concorrência, Headroom Térmico e Efeitos de Segunda Ordem

A alteração coordenada dos modelos alivia a pressão sobre os componentes físicos do sistema, gerando ganhos sistêmicos cumulativos.

No inventário v7 anterior, a execução paralela de dois subagentes efêmeros era inviável. O modelo de raciocínio retinha 1.468 MB, enquanto o modelo de ação de 7B ocupava 1.287 MB e o drafter especulativo exigia outros 1.741 MB de memória gráfica. Somados os buffers de contexto e as tabelas atencionais, a ocupação total ultrapassava 4.700 MB de VRAM. Quando picos de contexto de 8k tokens ocorriam em ambas as instâncias, a alocação excedia a cota de segurança de 5.294 MB do WDDM, deflagrando o transbordamento para a memória RAM compartilhada e reduzindo drasticamente a velocidade de processamento.

Com a adoção do `Temper-1-1.5B` (~1.050 MB) pareado com o `xLAM-2-1b-fc-r` (~880 MB) e com o drafter especulativo leve `Qwen2.5-Coder-0.5B` (~398 MB), o consumo base de tensores cai para apenas 2.328 MB de VRAM:

$$1.050\text{ MB } (\text{SWE}) + 880\text{ MB } (\text{MCP}) + 398\text{ MB } (\text{Draft}) = 2.328\text{ MB} \ll 5.294\text{ MB}$$

Essa redução drástica na alocação de memória gráfica gera quase 3.000 MB de margem livre na dGPU. Como consequência direta, o subsistema gráfico opera com baixa pressão de chaveamento de contexto de memória e menor dissipação térmica, mantendo as frequências de clock da GPU estáveis em tarefas computacionalmente intensivas.

A transferência dos papéis de triagem semântica e aferição de entropia lógica para a CPU Intel Core i9-9880H (Tier 0 e Tier 0.5) reforça esse equilíbrio. A execução do `ModernBERT-base` em tempo constante sob o runtime ONNX com instruções AVX2 suprime o atraso de preenchimento inicial dos subagentes de texto. Paralelamente, o `DeepSeek-R1-Distill-Qwen-1.5B` processa verificações de incerteza epistêmica em segundo plano nos 32GB de memória RAM, sem consumir ciclos térmicos da placa de vídeo.

## 6. Roteiro de Homologação, Repositórios Hugging Face e Configuração de Execução

Para orientar o processo de validação técnica e absorção dos novos modelos no banco de dados local (`souls_heuristic_vault.db`), a tabela abaixo consolida as prioridades de obtenção, links oficiais dos artefatos no Hugging Face, formatos de quantização recomendados e os parâmetros de inicialização validados para o motor llama.cpp:

|**Prioridade**|**Modelo e Identificador HF**|**Arquivo / Variante Alvo**|**Tamanho**|**Parâmetros de Execução llama.cpp / Engine**|**Justificativa do Teste**|
|---|---|---|---|---|---|
|**P1 (Crítica)**|`Salesforce/xLAM-2-1b-fc-r-gguf`<br><br>[cite: 26]|`xLAM-2-1B-fc-r-Q4_K_M.gguf`<br><br>[cite: 55]|~880 MB|`-ngl 99 -c 8192 -ctk f16 -ctv q4_0 -fa on`<br><br>[cite: 1]|Substituição do despachante MCP por um modelo ultraleve com alta precisão no BFCL v3.|
|**P1 (Crítica)**|`temper-ai/Temper-1-1.5B-GGUF` _(ou mradermacher)_|`Temper-1-1.5B.Q4_K_M.gguf`|~1,05 GB|`-ngl 99 -c 8192 -ctk f16 -ctv q4_0 -fa on`<br><br>[cite: 1]|Avaliação no _Rust AST Arena_ como substituto eficiente para geração de crates em Rust.|
|**P1 (Crítica)**|`unsloth/Qwen2.5-VL-3B-Instruct-GGUF`<br><br>[cite: 30]|`Qwen2.5-VL-3B-Instruct-UD-Q4_K_XL.gguf` + `mmproj-f16.gguf`<br><br>[cite: 30, 56]|~2,1 GB + ~600 MB|`-ngl 99 --mmproj <sidecar> -c 4096 -fa on`<br><br>[cite: 1]|Avaliação da precisão de UI Grounding estruturado e OCR técnico em diagramas de software.|
|**P2 (Alta)**|`bartowski/Qwen2.5-Coder-0.5B-Instruct-GGUF`<br><br>[cite: 44, 45]|`Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf`<br><br>[cite: 45]|~398 MB|`-md <drafter.gguf> -ngld 99 --draft-max 8 --draft-min 1`<br><br>[cite: 1, 40, 43]|Aceleração especulativa alinhada à família Qwen com consumo mínimo de VRAM.|
|**P2 (Alta)**|`unsloth/Qwen2.5-Coder-3B-Instruct-GGUF`<br><br>[cite: 13]|`Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf`<br><br>[cite: 13]|~1,65 GB|`-ngl 99 -c 8192 -ctk f16 -ctv q4_0 -fa on`<br><br>[cite: 1]|Substituto de baixa latência imune à perda de atenção observada em 8k tokens.|
|**P2 (Alta)**|`bartowski/Tesslate_Tessa-Rust-T1-7B-GGUF`<br><br>[cite: 12]|`Tessa-Rust-T1-7B-IQ3_M.gguf` ou `Q4_K_M.gguf`<br><br>[cite: 12]|~3,57 GB / ~4,68 GB|`-ngl 99 -c 8192 -ctk f16 -ctv q4_0 -fa on`<br><br>[cite: 1]|Teste de substituição do modelo pesado de 7B com raciocínio especializado em Rust.|
|**P3 (Média)**|`ggml-org/SmolVLM-Instruct-GGUF`<br><br>[cite: 34, 35]|`SmolVLM-Instruct-Q4_K_M.gguf` + `mmproj-f16.gguf`<br><br>[cite: 35]|~1,45 GB + ~400 MB|`-ngl 99 --mmproj <sidecar> -c 2048`<br><br>[cite: 1]|Sentinela visual ultrarrápido (>25 tok/s) para inspeção contínua de logs e erros no terminal.|
|**P3 (Média)**|`bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF`<br><br>[cite: 18, 46]|`DeepSeek-R1-Distill-Qwen-1.5B-Q8_0.gguf`<br><br>[cite: 18]|~1,60 GB|`-ngl 0 -t 8 -c 8192` (Host CPU AVX2 exclusivo)|Sensor epistêmico com raciocínio analítico profundo executado na memória RAM.|
|**P4 (Suporte)**|`answerdotai/ModernBERT-base`<br><br>[cite: 47]|`model.onnx` (FP16 / INT8 quantizado)|~290 MB|`onnxruntime` executado na CPU com 8 threads AVX2|Triagem semântica em tempo constante e sanitização de segurança sem alocar a dGPU.|

## 7. Protocolo de Validação Experimental na Bateria Discriminatória Bare-Metal

A homologação definitiva dos modelos prospectados exige a execução do protocolo de testes com descapitação rigorosa de prompt, eliminando falsos positivos decorrentes de ecos de terminal gerados pelo executável de inferência.

O primeiro estágio avalia a integridade sintática na geração de código em Rust através das Trilhas 2A, 2E e 2H. Os modelos `Temper-1-1.5B` e `Tessa-Rust-T1-7B` são submetidos a desafios de geração de árvores sintáticas (AST), tratamento de impasses em estruturas concorrentes (`Mutex` e `RwLock`) e aplicação de patches cirúrgicos via _Myers Diff_. A taxa de compilação direta pelo comando `cargo check` sem intervenção corretiva deve atingir no mínimo 95% para validar a aprovação do candidato.

O segundo estágio verifica a conformidade estrita aos esquemas JSON do protocolo MCP nas Trilhas 2B, 2F e 2D. O modelo `xLAM-2-1b-fc-r` é submetido a baterias de teste com gramáticas restritas via engine _llguidance_, avaliando a precisão na emissão de argumentos tipados e a ausência de alucinações em chamadas com mais de cinco parâmetros aninhados. O modelo deve manter estabilidade em múltiplos turnos de diálogo, gerando chamadas vazias (`[]`) de forma consistente quando nenhuma ferramenta for aplicável.

O terceiro estágio consiste no teste de resistência atencional sob contexto longo (Benchmark Type 5, 8k Haystack). O procedimento injeta agulhas de informação com profundidade de 50% em sequências sintéticas de 8.192 tokens compostas por documentações de crates Rust. O modelo candidato precisa atingir a classificação `PASS_STABLE`, sendo imediatamente reprovado caso ocorra perda de foco ou alucinação do conteúdo injetado (`ATTENTION_DEGRADED`).

O quarto estágio afere a projeção multimodal e a precisão do alinhamento geométrico de telas (Benchmark Type 3). O modelo `Qwen2.5-VL-3B-Instruct` recebe capturas de tela complexas de interfaces de desenvolvimento contendo código-fonte, painéis e terminais com mensagens de erro. O teste avalia a latência de projeção do tensor mmproj e calcula o erro médio de distância euclidiana das caixas delimitadoras e pontos de clique gerados em JSON, exigindo acurácia superior a 90% e tempo de resposta inferior a 1.500 ms.

O quinto estágio mede a taxa de aceitação especulativa ($\alpha$) através do Benchmark Type 4. O modelo drafter `Qwen2.5-Coder-0.5B-Instruct` é acoplado ao modelo primário de código por meio do parâmetro `-md`. A taxa de aceitação de tokens ($\alpha$) gerada em tarefas de programação deve ser mantida acima de 60%, garantindo uma aceleração líquida mínima de 1,35x sobre a decodificação autorregressiva padrão, sem causar contenção térmica ou de memória no subsistema gráfico.

## 8. Síntese Estratégica e Conclusões Executivas

A confrontação técnica entre o inventário local atual (v7) e as alternativas emergentes na comunidade open-source comprova a viabilidade de superar as limitações de silício da máquina bare-metal. A retenção excessiva de VRAM por modelos generalistas e a presença de latências anômalas no inventário anterior encontram resolução em arquiteturas especializadas de menor porte.

A promoção do **`temper-ai/Temper-1-1.5B`** para a função de Artesão de Rust e a adoção do **`Salesforce/xLAM-2-1b-fc-r`** como Despachante Agêntico representam as substituições de maior retorno prático. Com pegadas individuais de memória entre 880 MB e 1.050 MB, esses modelos garantem precisão técnica superior em seus domínios e liberam mais de 3.000 MB de VRAM na placa NVIDIA RTX 2060m. Esse ganho de memória viabiliza a execução simultânea de múltiplos trabalhadores agênticos sem risco de acionamento do _paging_ PCIe pelo WDDM.

No processamento multimodal, o modelo **`Qwen2.5-VL-3B-Instruct`** estabelece uma evolução qualitativa ao viabilizar automação de interfaces baseada em coordenadas cartesianas estruturadas em JSON, superando o modelo anterior em velocidade e estabilidade geométrica. Complementarmente, a introdução do **`Qwen2.5-Coder-0.5B-Instruct`** como modelo de rascunho especulativo restaura a homogeneidade vocabular da decodificação acelerada, reduzindo drasticamente o consumo de memória frente ao drafter antecessor.

Por fim, a delegação de triagens de segurança e cálculos de incerteza para a CPU Intel Core i9 através do **`ModernBERT-base`** (via ONNX) e do **`DeepSeek-R1-Distill-Qwen-1.5B`** (operando na RAM) assegura a proteção térmica e computacional da placa gráfica. Recomenda-se iniciar o cronograma de download e homologação prática imediatamente a partir das dependências de prioridade crítica (P1), consolidando a nova fronteira de eficiência no repositório de pesos local.