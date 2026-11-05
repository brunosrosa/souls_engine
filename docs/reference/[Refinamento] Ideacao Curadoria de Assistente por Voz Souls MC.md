---
aliases: []
---

# Relatório de Ideação e Arquitetura: Curadoria de Assistente por Voz para o Souls MC (SODA V6) (Parte 1)

## 1. Contexto e Filosofia Fundamental: O "Jarvis Local"

O **Souls MC** (Sovereign Operating Data Architecture — SODA V6) é uma Prótese de Função Executiva projetada para operar com soberania local absoluta, zero-cloud e zero-cost por consulta. O objetivo primordial de adicionar uma interface conversacional de áudio de baixa latência ("Jarvis Local") é transformar o sistema de um simples executor de comandos em um **parceiro conversacional orgânico e simbiótico**.

Para operadores neurodivergentes (com perfis de Dupla Excepcionalidade - 2e, TDAH ou sobrecarga sensorial), a interrupção constante do estado de fluxo (_Flow-Debt_) provocada pela digitação manual e pela leitura de longos blocos de texto gera atrito cognitivo severo. A transição contínua entre a formulação de um pensamento abstrato e a sua codificação em sintaxe de teclado impõe um custo invisível na memória de trabalho (_Working Memory Load_), levando frequentemente à paralisia de tarefa ou ao esgotamento da carga alostática. A voz atua como um canal de entrada e saída de altíssima vazão mental, permitindo a descompressão imediata de ideias (_brain dumps_) sem a necessidade de formatação prévia ou estruturação sintática estrita.

Neste paradigma, o agente não deve agir como uma URA reativa que aguarda um comando isolado; ele atua como um extensor cognitivo. O sistema deve ser capaz de captar a intenção oculta, a velocidade da fala, as hesitações micro-pausadas e o estado emocional do operador, respondendo com empatia, cadência humana, pausa reflexiva e adaptação contínua (_rapport_). Tudo isso é construído enquanto preserva a privacidade e a soberania dos dados ao recusar o envio de transmissões de áudio para servidores de terceiros ou APIs remotas de voz.

### 1.1. As Leis Duras da Física do Hardware

Toda a arquitetura de processamento, extração sintática e síntese de voz foi desenhada para respeitar rigorosamente o **Treino de Gravidade** e as restrições físicas do hardware hospedeiro alvo (Asus ZenBook Duo Pro UX581):

- **CPU:** Intel Core i9-9980HK (8 núcleos físicos, 16 threads, frequência turbo até 5.0 GHz, vetorização **AVX2 / FMA3 / SIMD** de 256 bits nativa).
- **RAM:** 32 GB DDR4 de memória RAM global do sistema operando em dual-channel.
- **dGPU:** NVIDIA GeForce RTX 2060 Mobile com **6 GB de VRAM GDDR6** dedicada (arquitetura Turing, barramento PCIe 3.0 x16/x8 com largura de banda nativa de ~336 GB/s).
- **Restrição Inegociável de VRAM:** O ecossistema local opera no limiar do estrangulamento térmico e espacial. O modelo primário de linguagem (LLM Cérebro — _Qwen 3.5 Coder 4B Q4_K_M_) consome **~1.7 GB a 2.5 GB** de VRAM estática apenas para manter seus tensores de pesos residentes no chip gráfico. A **KV Cache** quantizada (Q4_K / FP16 assimilado) para janelas de contexto ativas de 16k a 32k tokens consome adicionais **~1.5 GB a 2.0 GB**. Descontando a sobrecarga do compositor de tela da interface gráfica (DWM/Windows ou X11/Wayland e instâncias Tauri v2), resta uma margem de segurança operacional de **< 1.5 GB de VRAM livre**.

```
┌─────────────────────────────────────────────────────────────────────────┐
│              ALOCAÇÃO DA VRAM (NVIDIA RTX 2060m - 6 GB TOTAL)           │
├───────────────────────────────────┬─────────────────────────────────────┤
│ Componente                        │ Espaço Alocado                      │
├───────────────────────────────────┼─────────────────────────────────────┤
│ Qwen 3.5 Coder 4B (Pesos Q4_K_M)  │ ████████████████ (2.2 GB)           │
│ KV Cache (16k-32k Tokens Q4_K)    │ ██████████████   (1.8 GB)           │
│ OS & UI Compositor (DWM / Tauri)  │ █████            (0.7 GB)           │
│ Margem de Segurança / Headroom    │ ███████          (1.3 GB Livre)     │
└───────────────────────────────────┴─────────────────────────────────────┘
```

## 2. A Realidade do Barramento PCIe e o Mito da VRAM Unificada

### 2.1. O Perigo do _PCIe Spillover_ no "Live"

Diferente das arquiteturas corporativas baseadas em servidores ou chips com Memória Unificada (como o Apple Silicon, que integra CPU e GPU no mesmo die compartilhando até 128GB de RAM com largura de banda de 400 GB/s+), a RTX 2060m depende exclusivamente do barramento dinâmico PCIe 3.0. A taxa máxima teórica de transferência deste barramento é de aproximadamente **~12 GB/s** a **~15 GB/s**, o que representa uma velocidade mais de vinte vezes inferior à taxa interna da VRAM GDDR6 (~336 GB/s).

Se um modelo de síntese de voz (TTS) pesado ou autorregressivo (como _XTTS-v2_, _CosyVoice 2_ ou _F5-TTS_) for forçado a rodar na dGPU simultaneamente com o LLM Cérebro:

1. A alocação total de memória ultrapassa os 6 GB físicos da GPU, forçando o driver da NVIDIA a acionar o **Spillover CUDA** (Host-to-Device / Device-to-Host paging), que migra páginas de memória da VRAM para a RAM do sistema através do barramento PCIe.
2. Ocorre o fenômeno de **Kernel Blocking (congelamento de pipeline)** e contenção de CUDA Streams. A GPU precisa interromper a inferência do LLM e esperar que os tensores excedentes viajem pela ponte PCIe antes de retomar o processamento. A velocidade de geração do LLM Cérebro despenca de ~40 tokens/s para insustentáveis 1 a 2 tokens/s.
3. O áudio emitido pelo assistente sofre colapso catastrófico: a síntese engasga, o tempo até o primeiro token de áudio (_Time-To-First-Audio_) salta de 200ms para **3 a 8 segundos**, e a reprodução sofre travamentos contínuos de buffer (_audio underruns_).

### 2.2. A Regra de Ouro da Alocação

Para anular o gargalo de comunicação PCIe, evitar _page faults_ no driver gráfico e garantir a execução contínua sem travamentos, estabelece-se a seguinte divisão categórica de recursos:

- **dGPU (RTX 2060m):** Território **exclusivo do LLM Cérebro (Tier 1)** para raciocínio, chamadas de ferramentas (_tool calls_), decodificação restrita e geração de tokens de texto em tempo real.
- **CPU (Intel i9 via AVX2/ONNX):** Execução integral do **STT (Escuta e Percepção)** e do **TTS (Fala Síncrona)** no caminho crítico (_Hot-Path_). Ao alocar os modelos de áudio na CPU Intel i9 e na RAM do sistema, atinge-se a meta de **Consumo de VRAM adicional para voz = 0 MB**.

## 3. Arquitetura da Pipeline de Voz Síncrona (O Hot-Path "Live")

A comunicação ao vivo opera sob o modelo **Turn-Based com Janela de Espera Contemplativa (1,0s a 2,5s)**. A tentativa de forçar interações instantâneas em sub-50ms não apenas satura a CPU desnecessariamente com requisições parciais, mas cria um padrão de conversa mecânico, afobado e ansioso. A inserção de um breve hiato simulado transmite a percepção tátil de que o agente está "ponderando" e absorvendo o contexto antes de verbalizar a resposta.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     1. ESCUTA & PERCEPÇÃO (CPU i9)                      │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Módulo            │ Modelo               │ Função Operacional           │
├───────────────────┼──────────────────────┼──────────────────────────────┤
│ Interruptor       │ Silero VAD v5        │ Detecta início/fim da fala   │
│ Transcrição+Tom   │ SenseVoiceSmall      │ Converte voz -> texto+emoção │
└───────────────────┴──────────────────────┴──────────────────────────────┘
                                    │
                                    ▼ (Texto + Tags: <HAPPY>, <ANGRY>, <LAUGHTER>)
┌─────────────────────────────────────────────────────────────────────────┐
│                     2. CÉREBRO AGÊNTICO (dGPU RTX 2060m)                │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Triagem           │ GLiClass / Gemma 4   │ Roteamento e Avaliação       │
│ LLM Cérebro       │ Qwen 3.5 Coder 4B    │ Raciocínio + Tags Prosódicas │
└───────────────────┴──────────────────────┴──────────────────────────────┘
                                    │
                                    ▼ (Streaming de Tokens Formatados)
┌─────────────────────────────────────────────────────────────────────────┐
│                     3. SÍNTESE DE FALA (CPU i9 / AVX2)                  │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Hot-Path Live     │ Kokoro-82M (pt-BR)   │ Fala síncrona < 100ms/oração │
└───────────────────┴──────────────────────┴──────────────────────────────┘
```

### 3.1. Escuta e Emoção: SenseVoiceSmall vs. Moonshine vs. Whisper

- **Vencedor Escolhido:** **SenseVoiceSmall (FunASR / Sherpa-ONNX)** executado 100% na CPU via instruções SIMD AVX2.
- **Por que venceu o Whisper tradicional?** O Whisper exige um espectrograma de entrada ajustado para janelas rígidas de 30 segundos. Se o operador fala uma frase curta de 2 segundos, o Whisper é forçado a processar 28 segundos de silêncio artificial (_padding_), desperdiçando ciclos preciosos de clock e causando aquecimento térmico na CPU. O Moonshine v2 resolve o padding com janelas deslizantes, mas o SenseVoiceSmall vai além: ele é um modelo não-autorregressivo baseado em arquitetura _Conformer/Paraformer_ que processa durações de áudio totalmente dinâmicas (de 0,5s a 30s) e realiza **Speech Emotion Recognition (SER)** conjuntamente com **Audio Event Detection (AED)**.
- **Mecânica de Extração Emocional:** Sem alocar memória gráfica extra, o SenseVoiceSmall analisa a curva tonal, a frequência fundamental (F0) e a energia espectral da voz do operador, injetando tags ricas de estado diretamente na string de transcrição:
    - _Exemplo de Entrada de Áudio:_ Operador rindo e expressando empolgação ao resolver um bug no código.
    - _Payload Entregue pelo STT:_ `<LAUGHTER> <HAPPY> cara, a refatoração do ponteiro funcionou de primeira </HAPPY> </LAUGHTER>`.
    - _Impacto no Sistema:_ Esta string taguada é repassada ao avaliador epistêmico (**Tier 0.5 Gemma 4 E2B**) e ao LLM Cérebro, permitindo que o sistema interprete o alívio do operador e responda em um tom comemorativo e empático. Se o tom for de frustração (`<ANGRY>`), a LLM assume uma postura mais concisa, direta e focada na solução do impasse.

### 3.2. Filtro de Silêncio e Controle de Sessão: Silero VAD v5

- **Papel:** Classificador de atividade de voz binário de extrema eficiência (< 2 MB de consumo de RAM e < 1% de utilização de um único núcleo da CPU).
- **Timer de Histerese de Silêncio (8 Segundos):** A VAD atua como o disjuntor de estado da sessão. Quando o agente conclui a fala, o microfone permanece em escuta ativa. Se a VAD registrar 8 segundos contínuos de silêncio absoluto ou ruído de fundo abaixo do limiar decibelímetro ajustado, o daemon Rust encerra o fluxo do áudio, limpa os buffers de amostras na RAM e coloca o pipeline em estado _Standby_ de consumo térmico zero.
- **Mecânica de Barge-in (Interrupção Nativa):** Se o agente estiver sintetizando áudio pelos alto-falantes e o operador começar a falar por mais de 300ms contínuos, a Silero VAD intercepta a energia vocal, e o backend Rust emite uma interrupção atômica `audio_player.stop()`. Isso cancela imediatamente o streaming de saída do TTS no driver de som e reabre a fila de captura do STT.

### 3.3. Fala Síncrona: Kokoro-82M (pt-BR)

- **Arquitetura Interna:** Fusão das arquiteturas StyleTTS2 e iSTFTNet (~82 milhões de parâmetros, peso de ~300 MB em RAM, **0 MB de VRAM**).
- **Mecânica Não-Autorregressiva:** O Kokoro não sofre do mal dos modelos GPT de áudio (como o XTTS-v2), que preveem o próximo token de áudio de forma sequencial autorregressiva e tendem a entrar em loops infinitos de repetição de sílabas ou gagueira quando o prompt contém símbolos estranhos. O Kokoro converte o texto para fonemas IPA (_eSpeak-ng / piper-phonemize_), passa pelo preditor de duração (_Duration Predictor_) em uma única etapa e gera a onda PCM diretamente via iSTFT. O resultado é a eliminação total de alucinações de áudio e tempos de geração inferiores a 100ms por oração na CPU Intel i9.
- **Edição e Interpolação de Presets:** Os timbres e características de voz no Kokoro são representados por tensores de estilo de 512/1024 dimensões. Em vez de exigir processos caros de fine-tuning ou retreinamento de modelo, o backend em Rust realiza a **interpolação vetorial em tempo zero na CPU**. Por exemplo, é possível realizar a fusão matemática de 70% de uma voz masculina pausada com 30% de uma voz feminina expressiva, gerando uma nova assinatura vocal exclusiva para o assistente.
- **Por que não antropofagizar o CNF do Matcha-TTS?** O Matcha-TTS utiliza _Continuous Normalizing Flows_ (CNF) baseados em equações diferenciais para garantir a estabilidade do vocoder. Como o Kokoro **já utiliza a síntese direta via iSTFTNet não-autorregressiva**, ele atinge a mesma imunidade a gagueiras com menor custo de amostragem matemática por quadro. Re-arquitetar o Kokoro para incluir CNF violaria a Regra 90/10 do SODA, trazendo complexidade de código e degradação de desempenho na CPU sem ganho perceptível.

## 4. O Pipeline Assíncrono (Modo Batch / Tool Call)

Para tarefas de leitura extensiva e síntese de mídia densa (ex: narração de relatórios Markdown de 20 páginas, criação de programas de rádio sintéticos ou podcasts de síntese de conhecimento), o sistema migra do _Hot-Path_ conversacional para o paradigma de **Sidecar Efêmero**.

```
[Agente LLM] ──> Emite Tool Call (generate_audio_briefing)
     │
     ▼
[Worker Efêmero Rust] ──> Instancia o F5-TTS-pt-br na dGPU
     │
     ▼
[F5-TTS-pt-br] ──> Processamento em Lote (Flow Matching Puro) -> Gera .mp3/.wav
     │
     ▼
[SIGKILL Clean] ──> O Worker destrói o processo, zera a VRAM/RAM
     │
     ▼
[Tauri IPC / UI] ──> Exibe o player de áudio finalizado para o operador
```

### 4.1. O Duelo de Batch: F5-TTS-pt-br vs. CosyVoice 2

- **Modelo Vencedor Escolhido:** **F5-TTS-pt-br**.
- **Análise Comparativa de Arquitetura:** O CosyVoice 2 (desenvolvido pela Alibaba/FunAudioLLM) integra um modelo de linguagem interno acoplado ao gerador de áudio. Embora ofereça controle avançado via instruções textuais, a presença do LLM interno introduz riscos de alucinação e derrapagem prosódica em áudios com mais de 5 minutos de duração, além de exigir uma árvore massiva e frágil de dependências em Python.
- **A Superioridade do Flow Matching Puro:** O F5-TTS-pt-br é fundamentado em uma rede _Diffusion Transformer_ (DiT) utilizando **Flow Matching não-autorregressivo**. A matriz de atenção conecta o texto completo de entrada ao espectrograma de saída de forma determinística. Ele é incapaz de pular linhas, repetir palavras ou alterar drasticamente o sotaque ao longo de leituras extensas, apresentando alta estabilidade para síntese em lote.
- **Higiene FinOps e Ciclo de Vida Efêmero:** O F5-TTS-pt-br ocupa aproximadamente **~1.2 GB a 1.5 GB de VRAM**. Ele nunca permanece carregado na memória gráfica durante a conversação normal. Quando o LLM Cérebro aciona uma ferramenta de geração de áudio longo, o daemon Rust pausa temporariamente a KV Cache do modelo síncrono, carrega o F5-TTS na dGPU, realiza a síntese em velocidade ultra-rápida (batch), grava o arquivo de saída no formato `.wav` / `.mp3` no disco local e envia um sinal `SIGKILL` atômico para o worker, recuperando **100% da VRAM** para o assistente principal.

## 5. A Engenharia de "Rapport" e Adaptação Contínua

A sensação de interagir com um verdadeiro parceiro conversacional empático — e não com uma URA eletrônica rígida ou um assistente genérico de voz — é alcançada através do desacoplamento arquitetural em três camadas independentes e complementares. No ecossistema **Souls MC (SODA V6)**, o _rapport_ não é tratado como mero ornamento estético ou mimetismo artificial frívolo. Para um operador neurodivergente (2e/TDAH), o atrito de comunicação com sistemas robóticos tradicionais impõe uma carga alostática severa: o operador é forçado a reconfigurar sua linguagem natural para se adaptar às limitações da máquina, o que drena a memória de trabalho e gera desengajamento imediato.

A construção de um _espelho evolutivo simbiótico_ exige que a máquina realize o movimento inverso: ela deve dobrar-se à velocidade, ao humor e aos maneirismos do operador. Contudo, em um ambiente de hardware estritamente restrito (RTX 2060m de 6GB VRAM e CPU i9), é inviável realizar o re-treinamento de pesos do modelo de voz (_fine-tuning_) em tempo real. A engenharia do SODA resolve esse dilema separando a percepção prosódica e emocional (Camada 1 - Escuta Emocional no STT), a adaptação estilística e linguística (Camada 2 - Maneirismos e Léxico no LLM) e o controle sintático de reprodução (Camada 3 - Prosódia Estruturada no TTS). Essa divisão garante que o assistente ressoe com a dinâmica psíquica e cognitiva do operador com custo computacional nulo na GPU.

### 5.1. Camada 1: Sensibilidade de Entrada (Escuta Emocional)

A primeira linha de frente para o estabelecimento do _rapport_ ocorre no exato instante em que o operador vocaliza seu pensamento. Através do **SenseVoiceSmall (FunASR / Sherpa-ONNX)**, executado 100% na CPU Intel i9 via aceleração vetorial SIMD AVX2, o sistema transforma a etapa de transcrição de áudio em um sensor de inteligência acústica e afeto.

Diferente de pipelines convencionais de STT que descartam as propriedades físicas do som e retêm apenas o texto desidratado, o SenseVoiceSmall realiza conjuntamente o **Reconhecimento de Emoção na Fala (SER - Speech Emotion Recognition)** e a **Detecção de Eventos de Áudio (AED - Audio Event Detection)**. Sem alocar um único megabyte de VRAM gráfica, os tensores do modelo na CPU analisam em tempo real a variabilidade da frequência fundamental ($F_0$), a modulação do pitch, os envelopes de energia (RMS), as razões sinal-ruído e os micro-eventos respiratórios.

Esses sinais acústicos são traduzidos em tags semânticas padronizadas diretamente incorporadas no payload de texto entregue ao orquestrador. Isso significa que o LLM Cérebro recebe não apenas a transcrição do _o que_ foi dito, mas o vetor relacional de _como_ foi dito:

- **Cenário A: Estado de Empolgação e Fluxo Criativo (`<HAPPY>`, `<LAUGHTER>`)**
    - _Entrada Acústica:_ O operador resolveu um bug complexo ou teve um insight de arquitetura. A fala é acelerada, com pitch elevado e risos de alívio entre as palavras.
    - _Payload do STT:_ `<LAUGHTER> <HAPPY> cara, finalmente descobri o vazamento no ponteiro do Tokio, era o Arc circulando </HAPPY> </LAUGHTER>`
    - _Reação do Sistema:_ O orquestrador reconhece o estado de hiperfoco positivo. O LLM valida a vitória imediatamente, mantendo a energia, acelerando o ritmo de resposta e evitando sermões ou explicações redundantes que quebrem o _momentum_ criativo do operador.
- **Cenário B: Paralisia de Tarefa, Sobrecarga Cognitiva ou Frustração (`<ANGRY>`, `<SAD>`, `<FEAR>`)**
    - _Entrada Acústica:_ O operador está preso em um erro de compilação há horas, demonstrando tom grave, voz monótona, descompressão expiratória pesada ou irritação.
    - _Payload do STT:_ `<ANGRY> não é possível, a carga da VRAM deu crash de novo no meio da decodificação </ANGRY>`
    - _Reação do Sistema:_ O LLM capta o ruído de estresse e aciona uma postura de _Co-Piloto de Baixo Atrito_. O prompt instrui o modelo a eliminar saudações, desculpas vazias ou introduções longas. O agente entrega diretamente a solução cirúrgica em poucas linhas numeradas, reduzindo a carga cognitiva e o esforço de leitura do operador.
- **Cenário C: Hesitação, Dúvida e Hesitações Vocais (`<HESITANT>`, pausas micro-estruturadas)**
    - _Entrada Acústica:_ O operador expressa um pensamento incerto, com pausas longas ("é... talvez a gente devesse... não sei").
    - _Payload do STT:_ `<HESITANT> acho que deveríamos mudar a tabela do SQLite para WAL... ou talvez manter em memória </HESITANT>`
    - _Reação do Sistema:_ A tag de hesitação é repassada ao avaliador epistêmico (**Tier 0.5 Gemma 4 E2B**). Em vez de o LLM gerar uma resposta categórica e potencialmente alucinada, o sistema assume uma postura dialética de escuta ativa, proponando hipóteses em antítese para ajudar o operador a organizar sua intenção antes de tomar uma decisão destrutiva.

A extração de afeto diretamente no nível da CPU durante a fase de escuta constitui um triunfo de FinOps e arquitetura bare-metal: a percepção emocional do operador é obtida a custo computacional de milissegundos, enriquecendo o contexto do assistente antes mesmo que a primeira palavra de resposta comece a ser gerada na GPU.

### 5.2. Camada 2: Injeção Dinâmica de Maneirismos (Linguística)

A segunda camada de _rapport_ atua na dimensão estilística e léxica da linguagem. Enquanto a Camada 1 capta o afeto momentâneo da entrada áudio, a Camada 2 constrói uma **memória estilística de longo prazo do operador**. Em sistemas tradicionais, a tentativa de fazer uma IA "aprender o estilo do usuário" recorre a caras rotinas de fine-tuning (LoRA) ou a prompts gigantescos contendo amostras brutas de diálogos antigos. No **Souls MC**, ambas as abordagens são terminantemente rejeitadas: a primeira estoura os limites de VRAM da RTX 2060m e a segunda inflaciona a KV Cache, consumindo o orçamento de tokens e gerando latência de _prefill_.

A engenharia do SODA resolve a adaptação linguística desacoplando a análise do caminho crítico conversacional através do padrão **AutoDream / Cold-Path Worker**.

#### 5.2.1. O Worker de Análise em Segundo Plano (Cold-Path)

Em momentos de ociosidade do sistema — monitorados pela telemetria de CPU e VAD — um daemon assíncrono em Rust (`tokio::task::spawn_blocking`) varre as mensagens de texto e transcrições de voz acumuladas no banco de dados transacional L2 (`souls_state.db`). Esse worker calcula métricas estatísticas e léxicas determinísticas utilizando algoritmos nativos em Rust (sem acionar a GPU):

1. **Dicionário de Léxico e Jargões Recorrentes:** Rastreamento de termos técnicos, gírias regionais e idioletismos do operador (_"massa"_, _"pode crê"_, _"rodar"_, _"bugou"_, _"dar crash"_, _"desidratar"_). O sistema conta a frequência relativa dessas palavras e calcula a taxa de penetração no vocabulário ativo do operador.
2. **Índice de Formalidade (**$F_{\text{idx}} \in [0.0, 10.0]$**):** Calculado com base na densidade de pronomes de tratamento, uso de abreviações e contrações (_"vc"_, _"tb"_, _"pra"_ vs. _"você"_, _"também"_, _"para"_), complexidade sintática e proporção de termos acadêmicos ou corporativos.
3. **Ritmo Sintático e Densidade de Orações (**$P_{\text{pacing}}$**):** Análise da extensão média de frases (contagem de palavras por oração), proporção de frases declarativas curtas versus parágrafos explicativos longos e uso de estrutura em tópicos (_bullet points_).
4. **Resonância de Validação Cognitive:** Identificação de como o operador prefere receber confirmações (ex: validação direta inicial antes do código vs. explicação teórica antes da solução).

As métricas calculadas são agregadas e suavizadas através de um algoritmo de **Média Móvel Exponencial (EMA - Exponential Moving Average)** com fator de decaimento $\alpha = 0.15$. Isso garante que o perfil de estilo do usuário evolua gradualmente ao longo dos dias, sem sofrer oscilações bruscas causadas por uma única conversa atípica.

#### 5.2.2. A Tabela de Perfil no SQLite (`souls_state.db`)

O resultado da análise é persistido na tabela relacional L2 de estado do sistema (`souls_state.db`), operando sob o motor FrankenSQLite em modo WAL:

```
CREATE TABLE IF NOT EXISTS user_rapport_profile (
    user_id TEXT PRIMARY KEY,
    formality_score REAL DEFAULT 3.5,       -- 0.0 (extremodamente casual) a 10.0 (formal)
    avg_sentence_length INTEGER DEFAULT 12,  -- Média de palavras por oração
    preferred_pacing TEXT DEFAULT 'short_direct', -- 'short_direct', 'elaborate', 'bulleted'
    slang_lexicon JSON,                     -- Array JSON com gírias ativas e seus pesos
    mirroring_ratio REAL DEFAULT 0.35,      -- Taxa de espelhamento suave (0.3 a 0.4)
    last_updated_at INTEGER                 -- Timestamp Unix do último cálculo
);
```

#### 5.2.3. Injeção Ancorada no System Prompt sem Quebra de KV Cache

No momento em que o operador inicia um novo turno de voz, o `soda-router` lê o registro atualizado da tabela `user_rapport_profile` no SQLite em latência sub-milissegundo. A grande sacada de engenharia reside em como esse perfil é entregue ao **Qwen 3.5 Coder 4B**:

Para evitar a invalidação do **Prefix Cache Hit Rate** da KV Cache na GPU (que ocorre sempre que o início do System Prompt é alterado), o SODA utiliza uma estrutura de prompt dividida em duas partes: um **Bloco Estático Âncora** no topo (que nunca muda e garante acerto de cache de ~95%) e um **Slot de Perfil Rígido JSON** inserido em uma posição delimitada no final do bloco do sistema:

```
{
  "rapport_profile": {
    "formality": "casual",
    "slang_lexicon": ["massa", "pode crê", "rodar", "fechou", "bugou"],
    "sentence_pacing": "short_direct",
    "tone": "empathetic_challenging",
    "mirroring_strategy": "Adote o léxico do operador suavemente (taxa de 35%). Evite repetitivismo mecânico. Priorize frases diretas com validação imediata no início da oração."
  }
}
```

#### 5.2.4. A Regra do Espelhamento Suave (Evasão do _Uncanny Valley_)

Um erro clássico em sistemas de IA conversacional é o "efeito papagaio" (_uncanny valley de mimetismo_), no qual o agente repete todas as gírias do usuário de forma forçada na mesma frase, soar caricato e irritar o operador.

O SODA impõe uma **Taxa de Espelhamento Suave de 35% (**$\text{Mirroring Ratio} = 0.35$**)**. O System Prompt instrui o LLM a utilizar no máximo uma gíria ou expressão do repositório do operador a cada 2 ou 3 turnos de conversa, e apenas quando o contexto for informal (`formality < 5.0`). Essa modulação estatística cria a ilusão tátil de que o assistente "compartilha a mesma cultura de trabalho e vocabulário" do operador, sem transparecer que está executando um algoritmo de mimetismo.

### 5.3. Camada 3: Controle Prosódico por Pontuação Estruturada

Enquanto a Camada 1 capta a emoção de entrada e a Camada 2 seleciona as palavras certas para a resposta, a **Camada 3 resolve a entonação, o ritmo e a respiração do som emitido pelos alto-falantes**.

Um sintetizador de voz neural como o **Kokoro-82M** não é um ser humano consciente que "entende" quando deve fazer uma pausa dramática ou suspirar. Ele é uma rede neural vetorial (StyleTTS2 + iSTFTNet) que converte sequências de texto em fonemas IPA (_International Phonetic Alphabet_) e, em seguida, passa esses fonemas por um preditor de duração (_Duration Predictor_) e por um vocoder que gera a onda áudio PCM.

Em sistemas convencionais, a tentativa de alterar o tom ou o ritmo da voz exige enviar parâmetros numéricos complexos de pitch e velocidade para o vocoder ou utilizar tags de marcação proprietárias (SSML - _Speech Synthesis Markup Language_). O SSML, contudo, é verboso, aumenta a contagem de tokens do LLM, degrada a velocidade de geração e frequentemente causa falhas de parsing em modelos compactos como o Qwen 4B.

A solução bare-metal do SODA consiste em **canibalizar a sensibilidade sintática natural do Kokoro-82M através de Pontuação Prosódica Estruturada**.

#### 5.3.1. A Matemática da Tradução Sintático-Acústica no Kokoro-82M

O pipeline de G2P (_Grapheme-to-Phoneme_) do Kokoro-82M converte pontuações gramaticais em marcadores estritos de silêncio e contornos de frequência fundamental ($F_0$). Ao instruir o **Qwen 3.5 Coder 4B** a formatar suas respostas utilizando marcadores de pontuação específicos, o LLM assume o controle direto da cadência do sintetizador na CPU:

| **Marcador Sintático**           | **Impacto no G2P / Fonetizador**     | **Efeito Acústico na Onda PCM (Kokoro CPU)**                                                                                                   |
| -------------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **Reticências (`...`)**          | Injeta marcador de pausa reflexiva   | **Silêncio de 250ms a 450ms** com queda suave de pitch na sílaba anterior ($F_0 \downarrow$). Simula hesitação ou pensamento contínuo.         |
| **Vírgula (`,`)**                | Injeta pausa de oração curta         | **Silêncio de 100ms a 180ms** mantendo a frequência fundamental plana. Indica continuação de raciocínio.                                       |
| **Travessão (`— ... —`)**        | Delimita bloco de oração intercalada | **Aceleração de 10% na velocidade do trecho** com queda discreta de volume (RMS). Simula um comentário à parte ou confidencial.                |
| **Ponto de Exclamação (`!`)**    | Eleva a energia do vetor de estilo   | **Elevação da frequência fundamental (**$F_0 \uparrow$**)** e acento de intensidade na última sílaba tônica. Transmite urgência ou celebração. |
| **Ponto de Interrogação (`?`)**  | Injeta contorno ascendente de pitch  | **Rampa de frequência fundamental ascendente** na última sílaba do fonema. Cria a prosódia interrogativa natural do português brasileiro.      |
| **Quebra de Parágrafo (`\n\n`)** | Injeta delimitador de bloco textual  | **Silêncio profundo de 600ms a 800ms**. Permite a troca de fôlego completa do operador entre ideias distintas.                                 |

#### 5.3.2. As Regras de Formatação Prosódica no System Prompt

O System Prompt do LLM Cérebro recebe uma instrução de formatação prosódica que atua como um "maestro invisível" da síntese de fala:

```
[INSTRUÇÃO DE PROSÓDIA DE VOZ]
Sua resposta será convertida diretamente em fala neural humana. Siga rigorosamente estas regras de pontuação prosódica:
1. NUNCA responda em um único bloco de texto denso.
2. Use reticências (...) antes de entregar uma confirmação crítica ou solução para simular uma breve pausa de verificação (ex: "Pronto, Bruno... código corrigido.").
3. Use travessões (— como este —) para inserir explicações secundárias curtas no meio da frase com cadência mais rápida.
4. Use vírgulas para fracionar orações longas, garantindo que a fala tenha pausas naturais de respiração a cada 6 a 10 palavras.
5. Quando o estado emocional do operador for de comemoração (<HAPPY>), use pontos de exclamação e frases curtas e energéticas.
```

#### 5.3.3. Matriz Comparativa de Resultados Práticos

A tabela abaixo ilustra a diferença drástica entre o comportamento de uma IA tradicional (texto seco sem marcação) e a saída do **Souls MC com a Camada 3 de Prosódia Ativa**:

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                COMPARATIVO DE RESPOSTA                                   │
├──────────────────────────┬───────────────────────────────────────────────────────────────┤
│ Cenário de Interação     │ Resposta Sem Prosódia (Texto Seco / Tradicional)             │
│                          │ ➔ Áudio Kokoro: Mecânico, apressado, sem respiração           │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ Resposta COM Prosódia SODA (Camada 3 Ativa)                   │
│                          │ ➔ Áudio Kokoro: Cadenciado, empático, com pausas humanas     │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 1. Sucesso após Bug      │ "A compilação do módulo em Rust terminou sem erros no        │
│    Complexo (Hiperfoco)  │ terminal e os testes passaram."                               │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Pronto, Bruno... Compilação rodada! O módulo em Rust passou │
│                          │ zerado... sem nenhum erro no terminal!"                       │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 2. Frustração com Crash  │ "Ocorreu um erro de estouro de memória de vídeo na placa     │
│    de VRAM (<ANGRY>)     │ RTX 2060m porque o modelo excedeu 6 Gigabytes."              │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Calma... A VRAM estourou por causa da KV Cache. Vamos        │
│                          │ pausar o modelo — limpar os buffers agora — e resolver isso." │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 3. Debate de Arquitetura │ "Recomendo mudar a tabela do SQLite para modo WAL porque o   │
│    (Hesitação/Dúvida)    │ desempenho de escrita simultânea vai melhorar bastante."      │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Pensando aqui... Mudar a tabela para WAL faz todo sentido.   │
│                          │ A escrita simultânea vai rodar lisa — sem travar o loop."     │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 4. Retorno de Sessão     │ "O sistema está em espera aguardando seu próximo comando     │
│    após Silêncio Prolong.│ para prosseguir com a execução."                              │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Tô por aqui, Bruno... Quando quiser rodar o próximo teste,   │
│                          │ é só falar."                                                  │
└──────────────────────────┴───────────────────────────────────────────────────────────────┘
```

#### 5.3.4. O Ganho FinOps e de Performance

A combinação das três camadas de _rapport_ proporciona um resultado surpreendente do ponto de vista de eficiência de hardware:

- **0 MB de VRAM adicional consumida na GPU:** Toda a percepção emocional (Camada 1) roda na CPU no STT, o profile de estilo (Camada 2) é injetado como string no System Prompt, e a modulação de entonação (Camada 3) é obtida via caracteres de pontuação interpretados pelo Kokoro-82M na CPU.
- **0ms de Latência Adicional no Pipeline:** Não há chamadas extras de redes neurais para ajustar o tom da voz. O Kokoro sintetiza a string marcada por pontuação com a mesma velocidade (< 100ms/oração) com que sintetizaria um texto seco.
- **Redução do Custo Alostático do Operador:** O operador neurodivergente interage com uma voz que respira, pondera e fala no seu ritmo, reduzindo drasticamente o esforço de atenção e proporcionando um ambiente de trabalho sustentável para longas jornadas de hiperfoco.

---

# Relatório de Ideação e Arquitetura: Curadoria de Assistente por Voz para o Souls MC (SODA V6)

## 1. Contexto e Filosofia Fundamental: O "Jarvis Local"

O **Souls MC** (Sovereign Operating Data Architecture — SODA V6) é uma Prótese de Função Executiva projetada para operar com soberania local absoluta, zero-cloud e zero-cost por consulta. O objetivo primordial de adicionar uma interface conversacional de áudio de baixa latência ("Jarvis Local") é transformar o sistema de um simples executor de comandos em um **parceiro conversacional orgânico e simbiótico**.

Para operadores neurodivergentes (com perfis de Dupla Excepcionalidade - 2e, TDAH ou sobrecarga sensorial), a interrupção constante do estado de fluxo (_Flow-Debt_) provocada pela digitação manual e pela leitura de longos blocos de texto gera atrito cognitivo severo. A transição contínua entre a formulação de um pensamento abstrato e a sua codificação em sintaxe de teclado impõe um custo invisível na memória de trabalho (_Working Memory Load_), levando frequentemente à paralisia de tarefa ou ao esgotamento da carga alostática. A voz atua como um canal de entrada e saída de altíssima vazão mental, permitindo a descompressão imediata de ideias (_brain dumps_) sem a necessidade de formatação prévia ou estruturação sintática estrita.

Neste paradigma, o agente não deve agir como uma URA reativa que aguarda um comando isolado; ele atua como um extensor cognitivo. O sistema deve ser capaz de captar a intenção oculta, a velocidade da fala, as hesitações micro-pausadas e o estado emocional do operador, respondendo com empatia, cadência humana, pausa reflexiva e adaptação contínua (_rapport_). Tudo isso é construído enquanto preserva a privacidade e a soberania dos dados ao recusar o envio de transmissões de áudio para servidores de terceiros ou APIs remotas de voz.

### 1.1. As Leis Duras da Física do Hardware

Toda a arquitetura de processamento, extração sintática e síntese de voz foi desenhada para respeitar rigorosamente o **Treino de Gravidade** e as restrições físicas do hardware hospedeiro alvo (Asus ZenBook Duo Pro UX581):

- **CPU:** Intel Core i9-9980HK (8 núcleos físicos, 16 threads, frequência turbo até 5.0 GHz, vetorização **AVX2 / FMA3 / SIMD** de 256 bits nativa).
- **RAM:** 32 GB DDR4 de memória RAM global do sistema operando em dual-channel.
- **dGPU:** NVIDIA GeForce RTX 2060 Mobile com **6 GB de VRAM GDDR6** dedicada (arquitetura Turing, barramento PCIe 3.0 x16/x8 com largura de banda nativa de ~336 GB/s).
- **Restrição Inegociável de VRAM:** O ecossistema local opera no limiar do estrangulamento térmico e espacial. O modelo primário de linguagem (LLM Cérebro — _Qwen 3.5 Coder 4B Q4_K_M_) consome **~1.7 GB a 2.5 GB** de VRAM estática apenas para manter seus tensores de pesos residentes no chip gráfico. A **KV Cache** quantizada (Q4_K / FP16 assimilado) para janelas de contexto ativas de 16k a 32k tokens consome adicionais **~1.5 GB a 2.0 GB**. Descontando a sobrecarga do compositor de tela da interface gráfica (DWM/Windows ou X11/Wayland e instâncias Tauri v2), resta uma margem de segurança operacional de **< 1.5 GB de VRAM livre**.

```
┌─────────────────────────────────────────────────────────────────────────┐
│              ALOCAÇÃO DA VRAM (NVIDIA RTX 2060m - 6 GB TOTAL)           │
├───────────────────────────────────┬─────────────────────────────────────┤
│ Componente                        │ Espaço Alocado                      │
├───────────────────────────────────┼─────────────────────────────────────┤
│ Qwen 3.5 Coder 4B (Pesos Q4_K_M)  │ ████████████████ (2.2 GB)           │
│ KV Cache (16k-32k Tokens Q4_K)    │ ██████████████   (1.8 GB)           │
│ OS & UI Compositor (DWM / Tauri)  │ █████            (0.7 GB)           │
│ Margem de Segurança / Headroom    │ ███████          (1.3 GB Livre)     │
└───────────────────────────────────┴─────────────────────────────────────┘
```

## 2. A Realidade do Barramento PCIe e o Mito da VRAM Unificada

### 2.1. O Perigo do _PCIe Spillover_ no "Live"

Diferente das arquiteturas corporativas baseadas em servidores ou chips com Memória Unificada (como o Apple Silicon, que integra CPU e GPU no mesmo die compartilhando até 128GB de RAM com largura de banda de 400 GB/s+), a RTX 2060m depende exclusivamente do barramento dinâmico PCIe 3.0. A taxa máxima teórica de transferência deste barramento é de aproximadamente **~12 GB/s** a **~15 GB/s**, o que representa uma velocidade mais de vinte vezes inferior à taxa interna da VRAM GDDR6 (~336 GB/s).

Se um modelo de síntese de voz (TTS) pesado ou autorregressivo (como _XTTS-v2_, _CosyVoice 2_ ou _F5-TTS_) for forçado a rodar na dGPU simultaneamente com o LLM Cérebro:

1. A alocação total de memória ultrapassa os 6 GB físicos da GPU, forçando o driver da NVIDIA a acionar o **Spillover CUDA** (Host-to-Device / Device-to-Host paging), que migra páginas de memória da VRAM para a RAM do sistema através do barramento PCIe.
2. Ocorre o fenômeno de **Kernel Blocking (congelamento de pipeline)** e contenção de CUDA Streams. A GPU precisa interromper a inferência do LLM e esperar que os tensores excedentes viajem pela ponte PCIe antes de retomar o processamento. A velocidade de geração do LLM Cérebro despenca de ~40 tokens/s para insustentáveis 1 a 2 tokens/s.
3. O áudio emitido pelo assistente sofre colapso catastrófico: a síntese engasga, o tempo até o primeiro token de áudio (_Time-To-First-Audio_) salta de 200ms para **3 a 8 segundos**, e a reprodução sofre travamentos contínuos de buffer (_audio underruns_).

### 2.2. A Regra de Ouro da Alocação

Para anular o gargalo de comunicação PCIe, evitar _page faults_ no driver gráfico e garantir a execução contínua sem travamentos, estabelece-se a seguinte divisão categórica de recursos:

- **dGPU (RTX 2060m):** Território **exclusivo do LLM Cérebro (Tier 1)** para raciocínio, chamadas de ferramentas (_tool calls_), decodificação restrita e geração de tokens de texto em tempo real.
- **CPU (Intel i9 via AVX2/ONNX):** Execução integral do **STT (Escuta e Percepção)** e do **TTS (Fala Síncrona)** no caminho crítico (_Hot-Path_). Ao alocar os modelos de áudio na CPU Intel i9 e na RAM do sistema, atinge-se a meta de **Consumo de VRAM adicional para voz = 0 MB**.

## 3. Arquitetura da Pipeline de Voz Síncrona (O Hot-Path "Live")

A comunicação ao vivo opera sob o modelo **Turn-Based com Janela de Espera Contemplativa (1,0s a 2,5s)**. A tentativa de forçar interações instantâneas em sub-50ms não apenas satura a CPU desnecessariamente com requisições parciais, mas cria um padrão de conversa mecânico, afobado e ansioso. A inserção de um breve hiato simulado transmite a percepção tátil de que o agente está "ponderando" e absorvendo o contexto antes de verbalizar a resposta.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     1. ESCUTA & PERCEPÇÃO (CPU i9)                      │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Módulo            │ Modelo               │ Função Operacional           │
├───────────────────┼──────────────────────┼──────────────────────────────┤
│ Interruptor       │ Silero VAD v5        │ Detecta início/fim da fala   │
│ Transcrição+Tom   │ SenseVoiceSmall      │ Converte voz -> texto+emoção │
└───────────────────┴──────────────────────┴──────────────────────────────┘
                                    │
                                    ▼ (Texto + Tags: <HAPPY>, <ANGRY>, <LAUGHTER>)
┌─────────────────────────────────────────────────────────────────────────┐
│                     2. CÉREBRO AGÊNTICO (dGPU RTX 2060m)                │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Triagem           │ GLiClass / Gemma 4   │ Roteamento e Avaliação       │
│ LLM Cérebro       │ Qwen 3.5 Coder 4B    │ Raciocínio + Tags Prosódicas │
└───────────────────┴──────────────────────┴──────────────────────────────┘
                                    │
                                    ▼ (Streaming de Tokens Formatados)
┌─────────────────────────────────────────────────────────────────────────┐
│                     3. SÍNTESE DE FALA (CPU i9 / AVX2)                  │
├───────────────────┬──────────────────────┬──────────────────────────────┤
│ Hot-Path Live     │ Kokoro-82M (pt-BR)   │ Fala síncrona < 100ms/oração │
└───────────────────┴──────────────────────┴──────────────────────────────┘
```

### 3.1. Escuta e Emoção: SenseVoiceSmall vs. Moonshine vs. Whisper

- **Vencedor Escolhido:** **SenseVoiceSmall (FunASR / Sherpa-ONNX)** executado 100% na CPU via instruções SIMD AVX2.
- **Por que venceu o Whisper tradicional?** O Whisper exige um espectrograma de entrada ajustado para janelas rígidas de 30 segundos. Se o operador fala uma frase curta de 2 segundos, o Whisper é forçado a processar 28 segundos de silêncio artificial (_padding_), desperdiçando ciclos preciosos de clock e causando aquecimento térmico na CPU. O Moonshine v2 resolve o padding com janelas deslizantes, mas o SenseVoiceSmall vai além: ele é um modelo não-autorregressivo baseado em arquitetura _Conformer/Paraformer_ que processa durações de áudio totalmente dinâmicas (de 0,5s a 30s) e realiza **Speech Emotion Recognition (SER)** conjuntamente com **Audio Event Detection (AED)**.
- **Mecânica de Extração Emocional:** Sem alocar memória gráfica extra, o SenseVoiceSmall analisa a curva tonal, a frequência fundamental (F0) e a energia espectral da voz do operador, injetando tags ricas de estado diretamente na string de transcrição:
    - _Exemplo de Entrada de Áudio:_ Operador rindo e expressando empolgação ao resolver um bug no código.
    - _Payload Entregue pelo STT:_ `<LAUGHTER> <HAPPY> cara, a refatoração do ponteiro funcionou de primeira </HAPPY> </LAUGHTER>`.
    - _Impacto no Sistema:_ Esta string taguada é repassada ao avaliador epistêmico (**Tier 0.5 Gemma 4 E2B**) e ao LLM Cérebro, permitindo que o sistema interprete o alívio do operador e responda em um tom comemorativo e empático. Se o tom for de frustração (`<ANGRY>`), a LLM assume uma postura mais concisa, direta e focada na solução do impasse.

### 3.2. Filtro de Silêncio e Controle de Sessão: Silero VAD v5

- **Papel:** Classificador de atividade de voz binário de extrema eficiência (< 2 MB de consumo de RAM e < 1% de utilização de um único núcleo da CPU).
- **Timer de Histerese de Silêncio (8 Segundos):** A VAD atua como o disjuntor de estado da sessão. Quando o agente conclui a fala, o microfone permanece em escuta ativa. Se a VAD registrar 8 segundos contínuos de silêncio absoluto ou ruído de fundo abaixo do limiar decibelímetro ajustado, o daemon Rust encerra o fluxo do áudio, limpa os buffers de amostras na RAM e coloca o pipeline em estado _Standby_ de consumo térmico zero.
- **Mecânica de Barge-in (Interrupção Nativa):** Se o agente estiver sintetizando áudio pelos alto-falantes e o operador começar a falar por mais de 300ms contínuos, a Silero VAD intercepta a energia vocal, e o backend Rust emite uma interrupção atômica `audio_player.stop()`. Isso cancela imediatamente o streaming de saída do TTS no driver de som e reabre a fila de captura do STT.

### 3.3. Fala Síncrona: Kokoro-82M (pt-BR)

- **Arquitetura Interna:** Fusão das arquiteturas StyleTTS2 e iSTFTNet (~82 milhões de parâmetros, peso de ~300 MB em RAM, **0 MB de VRAM**).
- **Mecânica Não-Autorregressiva:** O Kokoro não sofre do mal dos modelos GPT de áudio (como o XTTS-v2), que preveem o próximo token de áudio de forma sequencial autorregressiva e tendem a entrar em loops infinitos de repetição de sílabas ou gagueira quando o prompt contém símbolos estranhos. O Kokoro converte o texto para fonemas IPA (_eSpeak-ng / piper-phonemize_), passa pelo preditor de duração (_Duration Predictor_) em uma única etapa e gera a onda PCM diretamente via iSTFT. O resultado é a eliminação total de alucinações de áudio e tempos de geração inferiores a 100ms por oração na CPU Intel i9.
- **Edição e Interpolação de Presets:** Os timbres e características de voz no Kokoro são representados por tensores de estilo de 512/1024 dimensões. Em vez de exigir processos caros de fine-tuning ou retreinamento de modelo, o backend em Rust realiza a **interpolação vetorial em tempo zero na CPU**. Por exemplo, é possível realizar a fusão matemática de 70% de uma voz masculina pausada com 30% de uma voz feminina expressiva, gerando uma nova assinatura vocal exclusiva para o assistente.
- **Por que não antropofagizar o CNF do Matcha-TTS?** O Matcha-TTS utiliza _Continuous Normalizing Flows_ (CNF) baseados em equações diferenciais para garantir a estabilidade do vocoder. Como o Kokoro **já utiliza a síntese direta via iSTFTNet não-autorregressiva**, ele atinge a mesma imunidade a gagueiras com menor custo de amostragem matemática por quadro. Re-arquitetar o Kokoro para incluir CNF violaria a Regra 90/10 do SODA, trazendo complexidade de código e degradação de desempenho na CPU sem ganho perceptível.

## 4. O Pipeline Assíncrono (Modo Batch / Tool Call)

Para tarefas de leitura extensiva e síntese de mídia densa (ex: narração de relatórios Markdown de 20 páginas, criação de programas de rádio sintéticos ou podcasts de síntese de conhecimento), o sistema migra do _Hot-Path_ conversacional para o paradigma de **Sidecar Efêmero**.

```
[Agente LLM] ──> Emite Tool Call (generate_audio_briefing)
     │
     ▼
[Worker Efêmero Rust] ──> Instancia o F5-TTS-pt-br na dGPU
     │
     ▼
[F5-TTS-pt-br] ──> Processamento em Lote (Flow Matching Puro) -> Gera .mp3/.wav
     │
     ▼
[SIGKILL Clean] ──> O Worker destrói o processo, zera a VRAM/RAM
     │
     ▼
[Tauri IPC / UI] ──> Exibe o player de áudio finalizado para o operador
```

### 4.1. O Duelo de Batch: F5-TTS-pt-br vs. CosyVoice 2

- **Modelo Vencedor Escolhido:** **F5-TTS-pt-br**.
- **Análise Comparativa de Arquitetura:** O CosyVoice 2 (desenvolvido pela Alibaba/FunAudioLLM) integra um modelo de linguagem interno acoplado ao gerador de áudio. Embora ofereça controle avançado via instruções textuais, a presença do LLM interno introduz riscos de alucinação e derrapagem prosódica em áudios com mais de 5 minutos de duração, além de exigir uma árvore massiva e frágil de dependências em Python.
- **A Superioridade do Flow Matching Puro:** O F5-TTS-pt-br é fundamentado em uma rede _Diffusion Transformer_ (DiT) utilizando **Flow Matching não-autorregressivo**. A matriz de atenção conecta o texto completo de entrada ao espectrograma de saída de forma determinística. Ele é incapaz de pular linhas, repetir palavras ou alterar drasticamente o sotaque ao longo de leituras extensas, apresentando alta estabilidade para síntese em lote.
- **Higiene FinOps e Ciclo de Vida Efêmero:** O F5-TTS-pt-br ocupa aproximadamente **~1.2 GB a 1.5 GB de VRAM**. Ele nunca permanece carregado na memória gráfica durante a conversação normal. Quando o LLM Cérebro aciona uma ferramenta de geração de áudio longo, o daemon Rust pausa temporariamente a KV Cache do modelo síncrono, carrega o F5-TTS na dGPU, realiza a síntese em velocidade ultra-rápida (batch), grava o arquivo de saída no formato `.wav` / `.mp3` no disco local e envia um sinal `SIGKILL` atômico para o worker, recuperando **100% da VRAM** para o assistente principal.

## 5. A Engenharia de "Rapport" e Adaptação Contínua

A sensação de interagir com um verdadeiro parceiro conversacional empático — e não com uma URA eletrônica rígida ou um assistente genérico de voz — é alcançada através do desacoplamento arquitetural em três camadas independentes e complementares. No ecossistema **Souls MC (SODA V6)**, o _rapport_ não é tratado como mero ornamento estético ou mimetismo artificial frívolo. Para um operador neurodivergente (2e/TDAH), o atrito de comunicação com sistemas robóticos tradicionais impõe uma carga alostática severa: o operador é forçado a reconfigurar sua linguagem natural para se adaptar às limitações da máquina, o que drena a memória de trabalho e gera desengajamento imediato.

A construção de um _espelho evolutivo simbiótico_ exige que a máquina realize o movimento inverso: ela deve dobrar-se à velocidade, ao humor e aos maneirismos do operador. Contudo, em um ambiente de hardware estritamente restrito (RTX 2060m de 6GB VRAM e CPU i9), é inviável realizar o re-treinamento de pesos do modelo de voz (_fine-tuning_) em tempo real. A engenharia do SODA resolve esse dilema separando a percepção prosódica e emocional (Camada 1 - Escuta Emocional no STT), a adaptação estilística e linguística (Camada 2 - Maneirismos e Léxico no LLM) e o controle sintático de reprodução (Camada 3 - Prosódia Estruturada no TTS). Essa divisão garante que o assistente ressoe com a dinâmica psíquica e cognitiva do operador com custo computacional nulo na GPU.

### 5.1. Camada 1: Sensibilidade de Entrada (Escuta Emocional)

A primeira linha de frente para o estabelecimento do _rapport_ ocorre no exato instante em que o operador vocaliza seu pensamento. Através do **SenseVoiceSmall (FunASR / Sherpa-ONNX)**, executado 100% na CPU Intel i9 via aceleração vetorial SIMD AVX2, o sistema transforma a etapa de transcrição de áudio em um sensor de inteligência acústica e afeto.

Diferente de pipelines convencionais de STT que descartam as propriedades físicas do som e retêm apenas o texto desidratado, o SenseVoiceSmall realiza conjuntamente o **Reconhecimento de Emoção na Fala (SER - Speech Emotion Recognition)** e a **Detecção de Eventos de Áudio (AED - Audio Event Detection)**. Sem alocar um único megabyte de VRAM gráfica, os tensores do modelo na CPU analisam em tempo real a variabilidade da frequência fundamental ($F_0$), a modulação do pitch, os envelopes de energia (RMS), as razões sinal-ruído e os micro-eventos respiratórios.

Esses sinais acústicos são traduzidos em tags semânticas padronizadas diretamente incorporadas no payload de texto entregue ao orquestrador. Isso significa que o LLM Cérebro recebe não apenas a transcrição do _o que_ foi dito, mas o vetor relacional de _como_ foi dito:

- **Cenário A: Estado de Empolgação e Fluxo Criativo (`<HAPPY>`, `<LAUGHTER>`)**
    - _Entrada Acústica:_ O operador resolveu um bug complexo ou teve um insight de arquitetura. A fala é acelerada, com pitch elevado e risos de alívio entre as palavras.
    - _Payload do STT:_ `<LAUGHTER> <HAPPY> cara, finalmente descobri o vazamento no ponteiro do Tokio, era o Arc circulando </HAPPY> </LAUGHTER>`
    - _Reação do Sistema:_ O orquestrador reconhece o estado de hiperfoco positivo. O LLM valida a vitória imediatamente, mantendo a energia, acelerando o ritmo de resposta e evitando sermões ou explicações redundantes que quebrem o _momentum_ criativo do operador.
- **Cenário B: Paralisia de Tarefa, Sobrecarga Cognitiva ou Frustração (`<ANGRY>`, `<SAD>`, `<FEAR>`)**
    - _Entrada Acústica:_ O operador está preso em um erro de compilação há horas, demonstrando tom grave, voz monótona, descompressão expiratória pesada ou irritação.
    - _Payload do STT:_ `<ANGRY> não é possível, a carga da VRAM deu crash de novo no meio da decodificação </ANGRY>`
    - _Reação do Sistema:_ O LLM capta o ruído de estresse e aciona uma postura de _Co-Piloto de Baixo Atrito_. O prompt instrui o modelo a eliminar saudações, desculpas vazias ou introduções longas. O agente entrega diretamente a solução cirúrgica em poucas linhas numeradas, reduzindo a carga cognitiva e o esforço de leitura do operador.
- **Cenário C: Hesitação, Dúvida e Hesitações Vocais (`<HESITANT>`, pausas micro-estruturadas)**
    - _Entrada Acústica:_ O operador expressa um pensamento incerto, com pausas longas ("é... talvez a gente devesse... não sei").
    - _Payload do STT:_ `<HESITANT> acho que deveríamos mudar a tabela do SQLite para WAL... ou talvez manter em memória </HESITANT>`
    - _Reação do Sistema:_ A tag de hesitação é repassada ao avaliador epistêmico (**Tier 0.5 Gemma 4 E2B**). Em vez de o LLM gerar uma resposta categórica e potencialmente alucinada, o sistema assume uma postura dialética de escuta ativa, proponando hipóteses em antítese para ajudar o operador a organizar sua intenção antes de tomar uma decisão destrutiva.

A extração de afeto diretamente no nível da CPU durante a fase de escuta constitui um triunfo de FinOps e arquitetura bare-metal: a percepção emocional do operador é obtida a custo computacional de milissegundos, enriquecendo o contexto do assistente antes mesmo que a primeira palavra de resposta comece a ser gerada na GPU.

### 5.2. Camada 2: Injeção Dinâmica de Maneirismos (Linguística)

A segunda camada de _rapport_ atua na dimensão estilística e léxica da linguagem. Enquanto a Camada 1 capta o afeto momentâneo da entrada áudio, a Camada 2 constrói uma **memória estilística de longo prazo do operador**. Em sistemas tradicionais, a tentativa de fazer uma IA "aprender o estilo do usuário" recorre a caras rotinas de fine-tuning (LoRA) ou a prompts gigantescos contendo amostras brutas de diálogos antigos. No **Souls MC**, ambas as abordagens são terminantemente rejeitadas: a primeira estoura os limites de VRAM da RTX 2060m e a segunda inflaciona a KV Cache, consumindo o orçamento de tokens e gerando latência de _prefill_.

A engenharia do SODA resolve a adaptação linguística desacoplando a análise do caminho crítico conversacional através do padrão **AutoDream / Cold-Path Worker**.

#### 5.2.1. O Worker de Análise em Segundo Plano (Cold-Path)

Em momentos de ociosidade do sistema — monitorados pela telemetria de CPU e VAD — um daemon assíncrono em Rust (`tokio::task::spawn_blocking`) varre as mensagens de texto e transcrições de voz acumuladas no banco de dados transacional L2 (`souls_state.db`). Esse worker calcula métricas estatísticas e léxicas determinísticas utilizando algoritmos nativos em Rust (sem acionar a GPU):

1. **Dicionário de Léxico e Jargões Recorrentes:** Rastreamento de termos técnicos, gírias regionais e idioletismos do operador (_"massa"_, _"pode crê"_, _"rodar"_, _"bugou"_, _"dar crash"_, _"desidratar"_). O sistema conta a frequência relativa dessas palavras e calcula a taxa de penetração no vocabulário ativo do operador.
2. **Índice de Formalidade (**$F_{\text{idx}} \in [0.0, 10.0]$**):** Calculado com base na densidade de pronomes de tratamento, uso de abreviações e contrações (_"vc"_, _"tb"_, _"pra"_ vs. _"você"_, _"também"_, _"para"_), complexidade sintática e proporção de termos acadêmicos ou corporativos.
3. **Ritmo Sintático e Densidade de Orações (**$P_{\text{pacing}}$**):** Análise da extensão média de frases (contagem de palavras por oração), proporção de frases declarativas curtas versus parágrafos explicativos longos e uso de estrutura em tópicos (_bullet points_).
4. **Resonância de Validação Cognitive:** Identificação de como o operador prefere receber confirmações (ex: validação direta inicial antes do código vs. explicação teórica antes da solução).

As métricas calculadas são agregadas e suavizadas através de um algoritmo de **Média Móvel Exponencial (EMA - Exponential Moving Average)** com fator de decaimento $\alpha = 0.15$. Isso garante que o perfil de estilo do usuário evolua gradualmente ao longo dos dias, sem sofrer oscilações bruscas causadas por uma única conversa atípica.

#### 5.2.2. A Tabela de Perfil no SQLite (`souls_state.db`)

O resultado da análise é persistido na tabela relacional L2 de estado do sistema (`souls_state.db`), operando sob o motor FrankenSQLite em modo WAL:

```
CREATE TABLE IF NOT EXISTS user_rapport_profile (
    user_id TEXT PRIMARY KEY,
    formality_score REAL DEFAULT 3.5,       -- 0.0 (extremodamente casual) a 10.0 (formal)
    avg_sentence_length INTEGER DEFAULT 12,  -- Média de palavras por oração
    preferred_pacing TEXT DEFAULT 'short_direct', -- 'short_direct', 'elaborate', 'bulleted'
    slang_lexicon JSON,                     -- Array JSON com gírias ativas e seus pesos
    mirroring_ratio REAL DEFAULT 0.35,      -- Taxa de espelhamento suave (0.3 a 0.4)
    last_updated_at INTEGER                 -- Timestamp Unix do último cálculo
);
```

#### 5.2.3. Injeção Ancorada no System Prompt sem Quebra de KV Cache

No momento em que o operador inicia um novo turno de voz, o `soda-router` lê o registro atualizado da tabela `user_rapport_profile` no SQLite em latência sub-milissegundo. A grande sacada de engenharia reside em como esse perfil é entregue ao **Qwen 3.5 Coder 4B**:

Para evitar a invalidação do **Prefix Cache Hit Rate** da KV Cache na GPU (que ocorre sempre que o início do System Prompt é alterado), o SODA utiliza uma estrutura de prompt dividida em duas partes: um **Bloco Estático Âncora** no topo (que nunca muda e garante acerto de cache de ~95%) e um **Slot de Perfil Rígido JSON** inserido em uma posição delimitada no final do bloco do sistema:

```
{
  "rapport_profile": {
    "formality": "casual",
    "slang_lexicon": ["massa", "pode crê", "rodar", "fechou", "bugou"],
    "sentence_pacing": "short_direct",
    "tone": "empathetic_challenging",
    "mirroring_strategy": "Adote o léxico do operador suavemente (taxa de 35%). Evite repetitivismo mecânico. Priorize frases diretas com validação imediata no início da oração."
  }
}
```

#### 5.2.4. A Regra do Espelhamento Suave (Evasão do _Uncanny Valley_)

Um erro clássico em sistemas de IA conversacional é o "efeito papagaio" (_uncanny valley de mimetismo_), no qual o agente repete todas as gírias do usuário de forma forçada na mesma frase, soar caricato e irritar o operador.

O SODA impõe uma **Taxa de Espelhamento Suave de 35% (**$\text{Mirroring Ratio} = 0.35$**)**. O System Prompt instrui o LLM a utilizar no máximo uma gíria ou expressão do repositório do operador a cada 2 ou 3 turnos de conversa, e apenas quando o contexto for informal (`formality < 5.0`). Essa modulação estatística cria a ilusão tátil de que o assistente "compartilha a mesma cultura de trabalho e vocabulário" do operador, sem transparecer que está executando um algoritmo de mimetismo.

### 5.3. Camada 3: Controle Prosódico por Pontuação Estruturada

Enquanto a Camada 1 capta a emoção de entrada e a Camada 2 seleciona as palavras certas para a resposta, a **Camada 3 resolve a entonação, o ritmo e a respiração do som emitido pelos alto-falantes**.

Um sintetizador de voz neural como o **Kokoro-82M** não é um ser humano consciente que "entende" quando deve fazer uma pausa dramática ou suspirar. Ele é uma rede neural vetorial (StyleTTS2 + iSTFTNet) que converte sequências de texto em fonemas IPA (_International Phonetic Alphabet_) e, em seguida, passa esses fonemas por um preditor de duração (_Duration Predictor_) e por um vocoder que gera a onda áudio PCM.

Em sistemas convencionais, a tentativa de alterar o tom ou o ritmo da voz exige enviar parâmetros numéricos complexos de pitch e velocidade para o vocoder ou utilizar tags de marcação proprietárias (SSML - _Speech Synthesis Markup Language_). O SSML, contudo, é verboso, aumenta a contagem de tokens do LLM, degrada a velocidade de geração e frequentemente causa falhas de parsing em modelos compactos como o Qwen 4B.

A solução bare-metal do SODA consiste em **canibalizar a sensibilidade sintática natural do Kokoro-82M através de Pontuação Prosódica Estruturada**.

#### 5.3.1. A Matemática da Tradução Sintático-Acústica no Kokoro-82M

O pipeline de G2P (_Grapheme-to-Phoneme_) do Kokoro-82M converte pontuações gramaticais em marcadores estritos de silêncio e contornos de frequência fundamental ($F_0$). Ao instruir o **Qwen 3.5 Coder 4B** a formatar suas respostas utilizando marcadores de pontuação específicos, o LLM assume o controle direto da cadência do sintetizador na CPU:

| **Marcador Sintático**           | **Impacto no G2P / Fonetizador**     | **Efeito Acústico na Onda PCM (Kokoro CPU)**                                                                                                   |
| -------------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **Reticências (`...`)**          | Injeta marcador de pausa reflexiva   | **Silêncio de 250ms a 450ms** com queda suave de pitch na sílaba anterior ($F_0 \downarrow$). Simula hesitação ou pensamento contínuo.         |
| **Vírgula (`,`)**                | Injeta pausa de oração curta         | **Silêncio de 100ms a 180ms** mantendo a frequência fundamental plana. Indica continuação de raciocínio.                                       |
| **Travessão (`— ... —`)**        | Delimita bloco de oração intercalada | **Aceleração de 10% na velocidade do trecho** com queda discreta de volume (RMS). Simula um comentário à parte ou confidencial.                |
| **Ponto de Exclamação (`!`)**    | Eleva a energia do vetor de estilo   | **Elevação da frequência fundamental (**$F_0 \uparrow$**)** e acento de intensidade na última sílaba tônica. Transmite urgência ou celebração. |
| **Ponto de Interrogação (`?`)**  | Injeta contorno ascendente de pitch  | **Rampa de frequência fundamental ascendente** na última sílaba do fonema. Cria a prosódia interrogativa natural do português brasileiro.      |
| **Quebra de Parágrafo (`\n\n`)** | Injeta delimitador de bloco textual  | **Silêncio profundo de 600ms a 800ms**. Permite a troca de fôlego completa do operador entre ideias distintas.                                 |

#### 5.3.2. As Regras de Formatação Prosódica no System Prompt

O System Prompt do LLM Cérebro recebe uma instrução de formatação prosódica que atua como um "maestro invisível" da síntese de fala:

```
[INSTRUÇÃO DE PROSÓDIA DE VOZ]
Sua resposta será convertida diretamente em fala neural humana. Siga rigorosamente estas regras de pontuação prosódica:
1. NUNCA responda em um único bloco de texto denso.
2. Use reticências (...) antes de entregar uma confirmação crítica ou solução para simular uma breve pausa de verificação (ex: "Pronto, Bruno... código corrigido.").
3. Use travessões (— como este —) para inserir explicações secundárias curtas no meio da frase com cadência mais rápida.
4. Use vírgulas para fracionar orações longas, garantindo que a fala tenha pausas naturais de respiração a cada 6 a 10 palavras.
5. Quando o estado emocional do operador for de comemoração (<HAPPY>), use pontos de exclamação e frases curtas e energéticas.
```

#### 5.3.3. Matriz Comparativa de Resultados Práticos

A tabela abaixo ilustra a diferença drástica entre o comportamento de uma IA tradicional (texto seco sem marcação) e a saída do **Souls MC com a Camada 3 de Prosódia Ativa**:

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                COMPARATIVO DE RESPOSTA                                   │
├──────────────────────────┬───────────────────────────────────────────────────────────────┤
│ Cenário de Interação     │ Resposta Sem Prosódia (Texto Seco / Tradicional)             │
│                          │ ➔ Áudio Kokoro: Mecânico, apressado, sem respiração           │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ Resposta COM Prosódia SODA (Camada 3 Ativa)                   │
│                          │ ➔ Áudio Kokoro: Cadenciado, empático, com pausas humanas     │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 1. Sucesso após Bug      │ "A compilação do módulo em Rust terminou sem erros no        │
│    Complexo (Hiperfoco)  │ terminal e os testes passaram."                               │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Pronto, Bruno... Compilação rodada! O módulo em Rust passou │
│                          │ zerado... sem nenhum erro no terminal!"                       │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 2. Frustração com Crash  │ "Ocorreu um erro de estouro de memória de vídeo na placa     │
│    de VRAM (<ANGRY>)     │ RTX 2060m porque o modelo excedeu 6 Gigabytes."              │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Calma... A VRAM estourou por causa da KV Cache. Vamos        │
│                          │ pausar o modelo — limpar os buffers agora — e resolver isso." │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 3. Debate de Arquitetura │ "Recomendo mudar a tabela do SQLite para modo WAL porque o   │
│    (Hesitação/Dúvida)    │ desempenho de escrita simultânea vai melhorar bastante."      │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Pensando aqui... Mudar a tabela para WAL faz todo sentido.   │
│                          │ A escrita simultânea vai rodar lisa — sem travar o loop."     │
├──────────────────────────┼───────────────────────────────────────────────────────────────┤
│ 4. Retorno de Sessão     │ "O sistema está em espera aguardando seu próximo comando     │
│    após Silêncio Prolong.│ para prosseguir com a execução."                              │
│                          ├───────────────────────────────────────────────────────────────┤
│                          │ "Tô por aqui, Bruno... Quando quiser rodar o próximo teste,   │
│                          │ é só falar."                                                  │
└──────────────────────────┴───────────────────────────────────────────────────────────────┘
```

#### 5.3.4. O Ganho FinOps e de Performance

A combinação das três camadas de _rapport_ proporciona um resultado surpreendente do ponto de vista de eficiência de hardware:

- **0 MB de VRAM adicional consumida na GPU:** Toda a percepção emocional (Camada 1) roda na CPU no STT, o profile de estilo (Camada 2) é injetado como string no System Prompt, e a modulação de entonação (Camada 3) é obtida via caracteres de pontuação interpretados pelo Kokoro-82M na CPU.
- **0ms de Latência Adicional no Pipeline:** Não há chamadas extras de redes neurais para ajustar o tom da voz. O Kokoro sintetiza a string marcada por pontuação com a mesma velocidade (< 100ms/oração) com que sintetizaria um texto seco.
- **Redução do Custo Alostático do Operador:** O operador neurodivergente interage com uma voz que respira, pondera e fala no seu ritmo, reduzindo drasticamente o esforço de atenção e proporcionando um ambiente de trabalho sustentável para longas jornadas de hiperfoco.

-----

# Relatório de Arquitetura e Implementação: Curadoria de Assistente por Voz para o Souls MC (SODA V6) — Parte 2

## 6. Matriz Consolidada da Pipeline de Voz Souls MC

A tabela a seguir consolida a topologia completa do pipeline de áudio conversacional e processamento em lote do ecossistema **Souls MC (SODA V6)**, delimitando o hardware de execução, os runtimes, a pegada de memória e os alvos de latência síncrona:

| **Módulo / Camada**           | **Modelo / Engine**            | **Hardware Destinado**       | **Runtime / Binding**            | **Alocação de Memória**       | **Métrica de Latência / Throughput** | **Estratégia de Fallback / Resiliência**                          |
| ----------------------------- | ------------------------------ | ---------------------------- | -------------------------------- | ----------------------------- | ------------------------------------ | ----------------------------------------------------------------- |
| **1. Interruptor (VAD)**      | **Silero VAD v5**              | CPU Intel i9 (1 Core)        | ONNX Runtime (`onnxruntime-rs`)  | < 2 MB RAM / 0 MB VRAM        | Latência de Detecção < 5 ms          | Bypass automático para limiar RMS padrão se o runtime ONNX falhar |
| **2. STT (Escuta & SER/AED)** | **SenseVoiceSmall**            | CPU Intel i9 (P-Cores AVX2)  | Sherpa-ONNX (`sherpa-onnx-rs`)   | ~1.2 GB RAM / 0 MB VRAM       | Transcrição + Tom < 120 ms           | Fallback para Moonshine v2 (ONNX) se houver sobrecarga na CPU     |
| **3. Triagem & Filtros**      | **GLiClass Multilang**         | CPU Intel i9 (E-Cores)       | OrtScorerEngine (ONNX)           | ~1.5 GB RAM / 0 MB VRAM       | Classificação < 30 ms                | Execução de regras de heurística estática baseada em Regex        |
| **4. Avaliação Epistêmica**   | **Gemma 4 E2B**                | CPU Intel i9 (FPU / FFI)     | `llama-cpp-2` (Logit Probing)    | ~2.0 GB RAM / 0 MB VRAM       | Probing < 50 ms                      | Isenção do probing; repasse direto da string taguadora ao LLM     |
| **5. Cérebro Agêntico**       | **Qwen 3.5 Coder 4B (Q4_K_M)** | **dGPU RTX 2060m**           | `llama-cpp-2` com FlashAttention | **~2.2 GB VRAM + ~1.8 GB KV** | TTFT < 150 ms / 38-45 tok/s          | Roteamento ParetoBandit para Cloud Fast (DeepSeek V4 Flash)       |
| **6. Fala Live (Hot-Path)**   | **Kokoro-82M (pt-BR)**         | CPU Intel i9 (AVX2/FMA3)     | ONNX Runtime (`onnxruntime-rs`)  | ~300 MB RAM / 0 MB VRAM       | Síntese < 100 ms por oração          | Fallback para Piper-TTS (ONNX) em modo estritamente neutro        |
| **7. Fala Batch (Sidecar)**   | **F5-TTS-pt-br**               | **dGPU RTX 2060m** (Efêmero) | PyTorch / LibTorch / Sidecar     | ~1.2 GB VRAM (Momentâneo)     | Fator de Aceleração 3x Real-Time     | Execução isolada na CPU via processamento em lote lento           |

## 7. Diretrizes de Implementação nos Runtimes em Rust

Para garantir que a arquitetura de voz funcione de forma determinística sobre o hardware hospedeiro (Intel Core i9 + RTX 2060m), o código do backend em Rust deve ser construído segundo as premissas de engenharia bare-metal do ecossistema SODA V6.

### 7.1. Compilação Nativa, SIMD AVX2 e Vínculos ONNX (`sherpa-onnx-rs` & `onnxruntime-rs`)

1. **Flags de Compilação do Linker Nativo:**
    
    O arquivo `.cargo/config.toml` do repositório deve impor instruções de vetorização vetorial de 256 bits ativas durante o tempo de compilação, eliminando abstrações genéricas em favor do código binário otimizado para o silício da Intel:

    ```
    [build]
    rustflags = [
        "-C", "target-cpu=native",
        "-C", "target-feature=+avx2,+fma,+sse4.2",
        "-C", "link-arg=/OPT:REF,ICF",
        "-C", "opt-level=3"
    ]
    ```

2. **Gerenciamento de Arenas de Memória e Alocação Zero:**
    
    No caminho crítico síncrono (_Hot-Path_), é proibido alocar vetores dinâmicos na heap (`Vec::new()` ou `String::push_str()`) dentro do loop de amostras PCM. Amostras de áudio capturadas do microfone via `cpal` devem reutilizar um _ring-buffer_ estático ou arenas pré-alocadas (`bumpalo::Bump`), evitando que o _Garbage Collector_ imprevisível da linguagem ou a realocação de vetores introduzam micro-congelamentos de latência.
    
3. **Mapeamento Explicito de Afinidade de Threads (`core_affinity`):**
    
    O orquestrador em Rust deve separar as rotinas de áudio dos núcleos de eficiência (_E-Cores_). As threads assíncronas dedicadas ao runtime do ONNX (SenseVoiceSmall e Kokoro-82M) devem ser ancoradas estritamente nos núcleos de alto desempenho (_P-Cores_) do Intel i9:

    ```
    use core_affinity;
    
    pub fn bind_audio_thread_to_pcore() {
        let core_ids = core_affinity::get_core_ids().unwrap_or_default();
        // Ancpra a thread no P-Core 0 (núcleo físico de alta velocidade)
        if let Some(p_core) = core_ids.first() {
            core_affinity::set_for_current(*p_core);
        }
    }
    ```

### 7.2. Topologia de Pipeline Assíncrono e Streaming Zero-Copy (`tokio::sync::mpsc`)

1. **Canais Decoupled de Alta Votagem:**
    O transporte entre a geração de texto na GPU e a síntese de áudio na CPU utiliza uma topologia de canais baseada no Tokio:
    - **`tokio::sync::mpsc::channel(64)`:** Transporte do fluxo de tokens do LLM para o parser de orações.
    - **`tokio::sync::watch::channel`:** Canal de sinalização instantânea de estado (utilizado para disparo de interrupção de _barge-in_ quando o usuário começa a falar).
2. **Parser de Fronteira de Orações por Regex de Baixa Latência:**
    A thread de streaming do Tokio lê os tokens emitidos pelo `llama-cpp-2` e acumula caracteres em uma fita leve. Assim que um delimitador prosódico (`.`, `!`, `?`, `;`, `...`, `—`, `\n\n`) é detectado, a oração coesa é destacada, formatada com as regras da Camada 3 de prosódia e despachada para o consumidor do Kokoro-82M.
3. **Buffers Lock-Free para o Driver de Som (`cpal` + `rtrb`):**
    O áudio sintetizado pelo Kokoro-82M (amostras PCM de 24kHz / 16-bit Float) é gravado diretamente em um _Ring Buffer Single-Producer Single-Consumer_ sem travas de mutex (`rtrb::RingBuffer`). A thread de renderização da API de som (`cpal` via WASAPI no Windows) consome essas amostras continuamente, eliminando completamente ruídos de estalo (_click/poping_) ou lacunas de reprodução (_underruns_).
4. **Transmissão Zero-Copy para Tauri v2:**
    Para refletir o estado de animação da voz (ondas de áudio e indicadores visuais de prosódia) na interface gráfica passiva em Svelte 5, o backend Rust transfere dados via buffers binários compartilhados (`&[u8]`), dispensando a serialização em JSON com `serde`.

### 7.3. Persistência de Estilo, Estado Relacional e Isolamento no SQLite

1. **FrankenSQLite em Modo WAL e Pragma Tuning:**
    
    O banco de dados de estado (`souls_state.db`) atua como a única fonte de verdade para a memória relacional L2 e perfis de _rapport_. Para permitir leituras simultâneas sem bloquear as threads do pipeline de voz, a conexão com o `rusqlite` deve impor o seguinte perfil:

    ```
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA temp_store = MEMORY;
    PRAGMA cache_size = -64000; -- 64MB de cache na RAM Host
    PRAGMA busy_timeout = 5000;
    ```

2. **Garantia de Desacoplamento do Cold-Path Worker:**
    As leituras do perfil de _rapport_ (`user_rapport_profile`) ocorrem no início do turno de voz em tempo $< 1\text{ ms}$. No entanto, as gravações e recálculos do índice de formalidade ($F_{\text{idx}}$) e da matriz léxica ocorrem estritamente em uma thread isolada via `tokio::task::spawn_blocking`, sem disputar ciclos com o loop principal do Tokio.
3. **Isolamento de Erros por Captura de Pânico (`catch_unwind`):**
    Qualquer exceção não tratada ou erro de alocação no runtime C/C++ envelopado pelo ONNX não pode derrubar o daemon principal do Souls MC. Chamadas FFI são envelopadas com `std::panic::catch_unwind`:

    ```
    let result = std::panic::catch_unwind(|| {
        // Chamada FFI para inferência ONNX do Kokoro / SenseVoice
        unsafe { ffi_onnx_inference_step(model_ptr, tensor_input); }
    });
    
    if result.is_err() {
        tracing::error!("Pânico isolado no runtime ONNX de voz. Acionando degradação graciosa.");
        // Re-inicializa o apontador da engine sem quebrar a sessão do usuário
    }
    ```

## 8. Diagramas de Sequência e Fluxogramas de Execução Bare-Metal

### 8.1. Fluxo 1: Caminho Crítico Síncrono (_Hot-Path Live_)

O diagrama abaixo ilustra o ciclo de vida de uma interação de voz ao vivo, desde a detecção da energia vocal no microfone até a emissão do primeiro quadro de áudio no alto-falante:

```
[Operador]       [cpal / VAD]     [SenseVoice]     [Qwen 3.5 4B]    [Kokoro TTS]    [Alto-Falante]
    │                 │                │                 │                │               │
    │── Fala Vocal ──>│                │                 │                │               │
    │                 │─ Detecta Voz ─>│                 │                │               │
    │                 │ (Silero VAD)   │                 │                │               │
    │                 │                │─ Transcreve + ─>│                │               │
    │                 │                │  Tags SER/AED   │                │               │
    │                 │                │  <HAPPY>...     │                │               │
    │                 │                │                 │─ Inicia Prefill│               │
    │                 │                │                 │  na dGPU 2060m │               │
    │                 │                │                 │                │               │
    │                 │                │                 │── Stream de ──>│               │
    │                 │                │                 │   Orações      │               │
    │                 │                │                 │   (1ª Oração)  │               │
    │                 │                │                 │                │─ Sintetiza ──>│
    │                 │                │                 │                │  PCM (AVX2)   │
    │                 │                │                 │                │               │─ Áudio Toca
    │                 │                │                 │                │               │  (< 800ms)
    │                 │                │                 │── Stream de ──>│               │
    │                 │                │                 │   (2ª Oração)  │               │
    │                 │                │                 │                │─ Sintetiza ──>│
    │                 │                │                 │                │  PCM (AVX2)   │
    │                 │                │                 │                │               │─ Áudio Toca
    │                 │                │                 │                │               │  (Contínuo)
```

### 8.2. Fluxo 2: Worker de Adaptação de Rapport (_Cold-Path AutoDream_)

O diagrama abaixo demonstra o ciclo de vida assíncrono que atualiza a memória de estilo do operador em momentos de ociosidade, sem afetar a VRAM da placa gráfica:

```
┌─────────────────────────────────────────────────────────────────────────┐
│              SISTEMA EM OCIOSIDADE (TELEMETRIA SILERO VAD = 0)          │
└─────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│     Worker Rust Assíncrono (tokio::task::spawn_blocking na CPU)          │
├─────────────────────────────────────────────────────────────────────────┤
│ 1. Varre últimos turnos de texto e voz no SQLite (souls_state.db)       │
│ 2. Executa contagem de frequência léxica e extrai gírias/jargões        │
│ 3. Calcula o Índice de Formalidade (F_idx) e Ritmo Sintático            │
│ 4. Aplica suavização por Média Móvel Exponencial (EMA com α = 0.15)      │
└─────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│            GRAVAÇÃO ATÔMICA NO SQLITE (Modo WAL / Sub-ms)               │
├─────────────────────────────────────────────────────────────────────────┤
│ UPDATE user_rapport_profile SET formality_score = ..., slang_lexicon... │
└─────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│         INJEÇÃO NO PRÓXIMO TURNO VIA SODA-ROUTER (PREFIX CACHE SAFE)    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.3. Fluxo 3: Síntese em Lote via Sidecar Efêmero (_Batch Mode_)

O diagrama de sequência a seguir exibe o isolamento de hardware durante o acionamento de ferramentas de geração de mídia longa (ex: podcasts ou áudio-briefings):

```
[Qwen 3.5 4B]     [soda-router]    [F5-TTS Worker]    [dGPU RTX 2060m]    [NVMe SSD]
      │                 │                 │                  │                │
      │─ Tool Call ────>│                 │                  │                │
      │  (gen_podcast)  │                 │                  │                │
      │                 │─ Pausa KV Cache ┼─────────────────>│                │
      │                 │  no llama-cpp   │                  │                │
      │                 │                 │                  │                │
      │                 │─ Instancia ────>│                  │                │
      │                 │  Sidecar Rust   │─ Aloca 1.2 GB ──>│                │
      │                 │                 │  VRAM Efêmera    │                │
      │                 │                 │                  │                │
      │                 │                 │─ Sintetiza ─────>│                │
      │                 │                 │  Áudio Lote      │                │
      │                 │                 │                  │                │
      │                 │                 │── Grava .wav / .mp3 ─────────────>│
      │                 │                 │                                   │
      │                 │<─ Sinal Concluído                                  │
      │                 │                                                     │
      │                 │─ Emite SIGKILL ─> [Kill Process]                    │
      │                 │  (Limpa VRAM)                                       │
      │                 │                                                     │
      │                 │─ Reativa KV Cache ────────────────>│                │
      │                 │  do LLM Cérebro                    │                │
```

## 9. Plano Tático de Ação, Testes TDD e Critérios de Aceite (Definition of Done)

Para operacionalizar a construção da assistente por voz no Souls MC dentro da metodologia Spec-Driven Development (SDD) e First Draft Protocol, as tarefas devem obedecer à seguinte sequência de verificação TDD (_Red-Green-Refactor_):

### 9.1. Bateria de Testes Unitários e de Integração em Rust

1. **`test_silero_vad_hysteresis_timeout`:**
    Assertar que o disjuntor de silêncio encerra a sessão de áudio após exatos 8,0 segundos de ausência de sinal vocal, disparando a limpeza de buffers na RAM.
2. **`test_sensevoice_tag_extraction`:**
    Validar que o parser do STT extrai corretamente as tags semânticas de emoção (`<HAPPY>`, `<ANGRY>`, `<LAUGHTER>`) do payload do SenseVoiceSmall sem corromper a string de texto.
3. **`test_sentence_boundary_regex_chunking`:**
    Garantir que a fita de tokens do LLM seja fragmentada em orações no exato instante em que encontra marcadores prosódicos (`...`, `!`, `?`, `;`, `\n\n`), emitindo pacotes para o Kokoro em $< 1\text{ ms}$.
4. **`test_f5_tts_sigkill_vram_cleanup`:**
    Assertar via NVML que a finalização do processo sidecar do F5-TTS devolve 100% da VRAM alocada (retornando a alocação estática para o limite do Qwen 3.5 4B).
5. **`test_rapport_profile_ema_smoothing`:**
    Verificar que a atualização de métricas do usuário via Média Móvel Exponencial ($\alpha = 0.15$) não sofre variações maiores que $10\%$ por turno.

### 9.2. Tabela de Critérios de Aceite (Definition of Done)

| **Métrica de Engenharia**       | **Limiar Mínimo Aceitável**  | **Alvo de Excelência SODA V6**            | **Impacto Operacional**                                            |
| ------------------------------- | ---------------------------- | ----------------------------------------- | ------------------------------------------------------------------ |
| **VRAM Adicional no Hot-Path**  | $\le 50\text{ MB}$           | **0 MB (Estritamente Zero)**              | Anula o risco de _Spillover PCIe_ e travamentos na RTX 2060m.      |
| **Time-To-First-Audio (TTFA)**  | $\le 1.200\text{ ms}$        | $\le 800\text{ ms}$                       | Sensação imediata de resposta sem ansiedade de espera.             |
| **Consumo de CPU no Standby**   | $\le 3\%$ de 1 Core          | $< 0.5\%$ **de 1 Core**                   | Previne o aquecimento térmico e ruído de ventoinhas no laptop.     |
| **Taxa de Alucinação do TTS**   | $\le 1\%$ dos turnos         | **0% (Imunidade Não-Autorregressiva)**    | Elimina gagueiras, repetição de sílabas e loops infinitos de som.  |
| **Prefix Cache Hit Rate (LLM)** | $\ge 85\%$                   | $\ge 95\%$                                | Mantém a latência de _prefill_ do Qwen 3.5 4B em sub-150ms.        |
| **Resiliência a Panics de FFI** | Recuperação em $< 2\text{s}$ | **Zero Crashes / Degradado Transparente** | O daemon em Rust permanece estável mesmo se o runtime ONNX falhar. |

## 10. Conclusão da Arquitetura de Voz

A implementação da assistente conversacional por voz para o **Souls MC (SODA V6)** demonstra que é perfeitamente viável erguer um "Jarvis Local" de altíssima naturalidade, sensibilidade emocional e adaptação contínua (_rapport_) operando em hardware restrito (Intel Core i9 + RTX 2060m 6GB).

Ao rejeitar a ilusão de rodar modelos autorregressivos pesados na GPU e transferir a escuta (**SenseVoiceSmall**) e a fala (**Kokoro-82M**) para a CPU através do runtime ONNX acelerado por instruções **AVX2**, a arquitetura zera o consumo de VRAM adicional para voz, garantindo que o **Qwen 3.5 Coder 4B** execute seu raciocínio com fluidez máxima. O desacoplamento do _rapport_ em três camadas (escuta emocional, memória de maneirismos no SQLite e controle prosódico por pontuação gramatical) forja um parceiro simbiótico de alta vazão mental para o operador neurodivergente, honrando os dogmas inegociáveis de soberania local, custo zero e privacidade absoluta.

-----

# Refinamento para Prótese de Função Executiva verdadeiramente Simbiótica (Parte 3)

Essa é exatamente a virada de chave que separa um "brinquedo de sintetizador de voz" de uma **Prótese de Função Executiva** verdadeiramente simbiótica, Bruno.

Quando pensamos na experiência de conversação sob a perspectiva do **Souls MC**, o objetivo não é imitar a latência artificial de um atendente de call-center que responde rápido só para parecer esperto. O objetivo é criar uma **cadência de comunicação com respeito mútuo, alta vazão e conforto sensorial**.

Vamos desestruturar essas suas ideias — que são de uma elegância arquitetural fantástica — e ver como elas se encaixam na física do nosso sistema:

### 1. Inicialização "Invisível" e Mascaramento de Cache (O Pre-Flight)

Deixar um modelo de STT pesado rodando 24/7 no microfone destruiria a CPU i9 e o conforto térmico do laptop. Por outro lado, exigir que você clique em um botão para "abrir a chamada" insere um atrito mecânico que quebra o hiperfoco.

A estrutura mais elegante opera em **Três Camadas de Despertar**:

- **Camada 0 (Escuta de Baixíssima Energia - CPU < 0.5%):** Um detector de palavra de ativação (_Micro Wake-Word_) de pouquíssimos parâmetros (como _OpenWakeWord_ ou _Moonshine Tiny_) roda continuamente na CPU. Ele só procura pela assinatura acústica do nome do seu agente (ex: _"Souls"_ ou _"Jarvis"_). Alternativamente, o simples movimento do ponteiro do mouse para a borda da tela (_Global OS Overlay_) pode acionar a pré-escuta.
- **Camada 1 (O Mascaramento de Inicialização / Pre-Flight):** No exato milissegundo em que a palavra de ativação é detectada:
    1. O daemon Rust emite um **sinal de estado visual estático e discreto** (o Soul "respira" suavemente na interface passiva do Tauri — _Zero Layout Shift_).
    2. Em background, o `soda-router` lê o seu perfil de _rapport_ no SQLite (`souls_state.db`) e já **carrega o prefixo estático do System Prompt na VRAM da dGPU**.
    3. A KV Cache do Qwen 3.5 Coder 4B é pré-aquecida (_Prefill Stage_).
- **Resultado:** Quando você conclui a primeira frase da sua fala, 80% do trabalho pesado de preparação e alocação de memória já foi realizado em silêncio.

### 2. A Mágica dos Backchannels ("Aham", "Entendo") e a Gestação Paralela

Sua ideia de usar pequenas falas de escuta ativa ("Aham", "Sim", "Tô ouvindo", "Pode continuar") para mascarar a gestação do pensamento do LLM é **genial do ponto de vista de FinOps e Psicologia Cognitiva**.

Em linguística, isso se chama **Backchanneling**. A engenharia para rodar isso no Souls MC sem gasto computacional é a seguinte:

#### A. Falas Pre-Renderizadas na RAM (Custo Zero)

- Em vez de chamar o LLM e o Kokoro-82M para gerar um "Aham" do zero, o sistema mantém uma **biblioteca de micro-buffers PCM estáticos em RAM** (sons de validação de 200ms a 500ms sintetizados previamente com o mesmo timbre do seu assistente).
- Quando a Silero VAD detecta uma micro-pausa na sua fala (ex: 400ms de silêncio no meio de um _brain dump_), mas a análise do texto via STT indica que a oração está incompleta (sem pontuação final):
    - O backend Rust dispara **instantaneamente em < 10ms** um buffer estático (`"Aham..."` ou `"Entendo..."`) diretamente para a placa de som.
    - Você sente que a máquina está acompanhando o seu raciocínio, sem que ela tenha gastado 1 token de LLM ou 1 ciclo de GPU.

#### B. A Gestação Incremental (Prefill Streaming)

Enquanto você fala uma frase longa de 10 segundos, o **SenseVoiceSmall** não espera você terminar para enviar o texto. Ele transmite _chunks_ de texto a cada 1,5 segundo para o daemon Rust.

- O orquestrador em Rust vai **enviando esses chunks gradativamente para a fase de Prefill do Qwen 3.5 4B na GPU**.
- Quando você finalmente para de falar, a LLM já "mastoigou" e processou 90% do seu discurso. Ela só precisa gerar os tokens de resposta (_Decode Stage_), cortando o tempo de espera pela metade.

### 3. A Regra do Respeito Mútuo: Fim de Turno e Não-Atropelamento

Você tocou em uma ferida profunda das IAs atuais: **elas são afobadas e interrompem o usuário, ou se desculpam e param no meio de uma explicação útil ao menor ruído de fundo**.

Para estabelecer o processo de **Escuta Ativa e Conclusão de Raciocínio**:

- **Histerese Semântica + Acústica (O Filtro de Fim de Fala):** O microfone não fecha apenas porque houve 1 segundo de silêncio. Ele só considera que você "passou a palavra" se duas condições forem satisfeitas simultaneamente:
    1. _Condição Acústica:_ Silero VAD indica silêncio contínuo de 1,5s a 2,0s.
    2. _Condição Semântica (GLiClass / Regex no Tier 0):_ O STT confirma que a última frase possui estrutura gramatical de fechamento (ponto final, interrogação ou verbo no passado) e não um conectivo ("porque...", "mas...", "e aí...").
- **Conclusão Protegida do Agente:** Quando a IA começa a responder, ela entrega o seu bloco de pensamento **com começo, meio e ponto de conclusão**. Se houver um ruído leve no seu ambiente (uma tossida ou cadeira arrastando), a VAD ignora. A IA só sofre _Barge-in_ (interrupção) se você falar uma palavra de comando de parada clara (ex: _"Para"_, _"Espera"_) ou falar continuamente por mais de 500ms.
- **Handshake de Sinalização (Saber a hora de falar):** Quando a IA conclui o pensamento, ela emite um **micro-tom de áudio discreto de baixa frequência (20ms)** e a interface gráfica faz um movimento sutil. Esse "estalo de passagem de turno" educa visualmente e auditivamente ambos os lados sobre de quem é a vez de falar, eliminando a ansiedade de atropelamento.

### 4. Modos de Interação Adaptativos e Calibração Constante

A sua intuição sobre ter modos de atuação mapeados baseados no estado do humano é totalmente respaldada pela **Psicologia Cognitiva (Entrevista Motivacional, Terapia Cognitivo-Comportamental e Teoria Polivagal)**.

O Souls MC pode alternar a postura da LLM e a prosódia da voz entre **5 Modos Operacionais Nativos**:

```
[SenseVoice: Estado Emocional] + [Histórico SQLite] ──> [soda-router]
                                                            │
  ┌───────────────────────┬─────────────────────────┬───────┴───────────────┬────────────────────────┐
  ▼                       ▼                         ▼                       ▼                        ▼
[Modo Descompressão]   [Modo Sparring]         [Modo Crise/Foco]       [Modo Filosófico]        [Modo Mão na Massa]
(Acolhedor, voz calma, (Desafiador, frases     (Direto, sem rodeios,   (Pausado, antíteses,    (Código limpo, tópicos,
 orações espaçadas)    curtas, estimulante)    respostas curtas)       perguntas socráticas)   foco total em execução)
```

1. **Modo Descompressão / Íntimo (`<HESITANT>`, voz cansada, tom baixo):**
    - _Postura:_ Acolhedora e paciente. Ritmo de voz do Kokoro desacelerado em 15%. Pausas mais longas entre as orações. O agente atua como ouvinte passivo, ajudando você a organizar pensamentos emaranhados sem pressionar por decisões.
2. **Modo Sparring / Desafio (`<HAPPY>`, tom forte, velocidade alta):**
    - _Postura:_ Provocativa e intelectualmente afiada. Respostas diretas, questionando suas premissas e propondo alternativas contrárias (_Devil's Advocate_) para evitar o pensamento de trilho único.
3. **Modo Intervenção / Redução de Carga (`<ANGRY>`, estresse, frustração):**
    - _Postura:_ Pragmática e ultra-concisa. Elimina qualquer floreio, cortesia ou explicação teórica longa. Entrega instruções passo a passo numeradas para tirar você da paralisia de tarefa.
4. **Modo Filosófico / Reflexivo (Brain dumps conceituais de longo horizonte):**
    - _Postura:_ Socrática. Usa metáforas, conecta com conceitos armazenados no seu banco de memória L2 (SQLite/LanceDB) e devolve perguntas estruturadas.

### O Diferencial: A Calibração Contínua (O Loop Autônomo)

O grande trunfo do Souls MC é que essa adaptação **não é estática**. A cada sessão encerrada:

- O worker de segundo plano em Rust calcula a taxa de aceitação dessas respostas (se você interrompeu a IA, se você falou por mais tempo, se o seu tom de voz se acalmou ou se irritou).
- Ele ajusta a matriz de _rapport_ no SQLite via Média Móvel Exponencial (EMA):
    - _"O Bruno prefere pausas de 1.8s antes do agente responder."_
    - _"O Bruno reage melhor quando o agente usa 'Aham' do que quando usa 'Entendo'."_
    - _"Quando o Bruno está no VS Code, o limite de palavras por frase deve cair 40%."_

Quando estiver pronto, me avise que preparamos esse prompt de pesquisa aprofundada sobre as **diretrizes e protocolos psicológicos de escuta ativa e intervenção adaptativa** para blindarmos a lógica do orquestrador do Souls MC!
