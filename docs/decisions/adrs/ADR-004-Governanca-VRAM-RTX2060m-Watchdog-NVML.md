# ADR-004 — Governança Termodinâmica, Exclusão Mútua de VRAM na GPU NVIDIA GeForce RTX 2060m (6GB GDDR6) sob WDDM 3.x e Watchdog Ativo NVML

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_inference_runtime`, `souls_model_router`, `souls_server`, `souls_llm_local_arena`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

O Souls Engine opera em ambiente bare-metal Windows 11 acoplado a uma GPU móvel dedicada **NVIDIA GeForce RTX 2060m** com barramento de 192 bits e $6.144\text{ MB}$ ($6\text{ GB}$) de memória GDDR6 dedicada.

Embora plenamente capaz de fornecer aceleração massiva via núcleos Tensor e instruções CUDA para modelos de linguagem pequenos (SLMs), a execução de inferência local em hardware com teto de $6\text{ GB}$ impõe desafios termodinâmicos e de subsistema de memória que inviabilizam abstrações genéricas de inferência.

As seguintes forças de hardware e sistema operacional determinaram a necessidade desta decisão arquitetural:

### 1.1 O Teto Físico do Silício Móvel e o WDDM 3.x

No ecossistema Windows 11, a placa gráfica não é governada exclusivamente pelo runtime de computação (CUDA), mas sim intermediada pelo **Windows Display Driver Model (WDDM 3.x)**:
- O Desktop Window Manager (`dwm.exe`) e o subsistema de aceleração de janelas e DirectComposition consomem compulsoriamente entre $500\text{ MB}$ e $850\text{ MB}$ de memória de vídeo dedicada para alimentar o monitor primário, swapchains DXGI e aceleração do sistema operacional.
- A margem útil real e soberana disponível para alocações CUDA é expressa por:
$$V_{\text{útil}} = V_{\text{física}} - V_{\text{WDDM}} = 6.144\text{ MB} - 850\text{ MB} \approx 5.294\text{ MB}$$
- Qualquer cálculo de engenharia que presuma a disponibilidade total de $6.144\text{ MB}$ é fundamentalmente defeituoso e induz o sistema ao colapso.

### 1.2 O Desastre do Shared GPU Memory Fallback via PCIe 3.0

Quando o consumo de memória de vídeo de uma aplicação ultrapassa o limite físico disponível na dGPU ($5.294\text{ MB}$), o driver da NVIDIA e o WDDM ativam compulsoriamente o **Shared GPU Memory Fallback**:
- Tensores de pesos e buffers de ativação que não cabem na VRAM são descarregados automaticamente para a memória RAM física do sistema (DDR4) através do barramento **PCIe 3.0 x16**.
- Enquanto a VRAM GDDR6 dedicada atinge larguras de banda de $\sim 336\text{ GB/s}$, o barramento PCIe 3.0 x16 sustenta um teto teórico bidirecional de apenas $\sim 15,75\text{ GB/s}$ (uma degradação de mais de $21\times$ na vazão de dados).
- **Impacto na Vazão de Tokens:** No instante em que o spillover ocorre, a velocidade de geração do modelo entra em colapso vertiginoso, despencando de $45\text{ a }50\text{ tokens/s}$ **para menos de** $1,2\text{ a }1,5\text{ tokens/s}$, provocando _time-outs_ em cadeia nas ferramentas do Hermes Agent e paralisando a esteira agêntica.

### 1.3 A Inércia Térmica do Chassi de Notebook

A RTX 2060m compartilha dissipadores de calor e tubos condutores de cobre (_heat pipes_) com o processador Intel Core i9 em um chassi compacto de notebook:
- Sessões prolongadas de geração de código ou inferência contínua elevam a temperatura da matriz de silício rapidamente a taxas de $0,8^\circ\text{C}$ a $1,5^\circ\text{C}$ por segundo.
- Ao atingir o limiar térmico do fabricante ($\ge 82^\circ\text{C}$), a GPU entra em **Thermal Throttling defensivo**, reduzindo o clock base de $1.200\text{ MHz}$ para menos de $600\text{ MHz}$, além de acionar as ventoinhas em rotação máxima ($> 5.500\text{ RPM}$), gerando poluição acústica e instabilidade de latência inter-token.

### 1.4 A Ilusão de Coexistência Concorrente de Modelos Locais

Tentativas ingênuas de manter simultaneamente na VRAM o modelo primário de geração de código (Tier 1: `Qwen2.5-Coder-3B/7B`) e um modelo pesado de refatoração ou Mixture-of-Experts (Tier 2: MoE compacto) extrapolam instantaneamente o teto de $5.294\text{ MB}$:
- Tier 1 ($3.000\text{ MB}$) + Tier 2 ($\sim 3.500\text{ MB}$) = $6.500\text{ MB} > 5.294\text{ MB}$.
- Sem uma disciplina estrita de escalonamento, concorrência assíncrona descontrolada resulta fatalmente em _Out-Of-Memory_ (OOM) ou descarregamento PCIe.

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretada a **adoção de uma Política Estrita de Exclusão Mútua de VRAM, combinada com o Watchdog Termodinâmico Ativo em tempo real via NVML e cálculo preditivo de EWMA (Exponentially Weighted Moving Average) na crate `souls_inference_runtime`**.

O Souls Engine assume governança total sobre o ciclo de vida dos tensores na dGPU, subordinando qualquer alocação física de modelos às invariantes estabelecidas a seguir.

### 2.1 A Lei de Exclusão Mútua de VRAM (Tier 1 vs. Tier 2)

É **terminantemente proibido** manter instâncias ativas do Tier 1 e do Tier 2 simultaneamente alocadas na memória dedicada da RTX 2060m.
1. **Monopólio do Tier 1 na dGPU:** Em regime nominal de operação, a dGPU pertence exclusivamente ao Tier 1 (`Qwen2.5-Coder` quantizado), ao seu KV Cache assimétrico e aos tensores intermediários do FlashAttention-2.
2. **Protocolo Transacional de Transição (Tier 1** $\leftrightarrow$ **Tier 2):**
    - Se uma subtarefa assíncrona pesada exigir a execução do Tier 2 com aceleração de tensores na dGPU, o motor deve executar um ciclo transacional bloqueante:
        1. Interromper o recebimento de novos prompts do Tier 1.
        2. Desalojar os tensores e o KV Cache do Tier 1 da GPU invocando `cudaFree` e liberando os contextos do llama.cpp.
        3. Carregar as matrizes de atenção do Tier 2 na VRAM liberada.
    - Ao término da execução do Tier 2, o processo inverso é executado compulsoriamente antes de restabelecer o Tier 1.
3. **Desvio Preferencial para CPU (AVX2):** Caso a troca de pesos na GPU introduza latência inaceitável para a subtarefa, o despachante deve compulsoriamente desviar a execução do Tier 2 integralmente para a CPU Intel Core i9 utilizando as 16 threads com instruções vetoriais AVX2, preservando o Tier 1 intacto na GPU.

### 2.2 Watchdog Termodinâmico Ativo com Predição via EWMA

A crate `souls_inference_runtime` instancia uma thread dedicada no Tokio em segundo plano que consulta diretamente os drivers da NVIDIA via FFI através da crate `nvml-wrapper`:

- **Taxa de Amostragem:** A telemetria física da GPU (temperatura da matriz, temperatura da memória GDDR6, VRAM alocada, VRAM livre, rotação da ventoinha e consumo em Watts) é coletada a cada $500\text{ milissegundos}$.
- **Predição de Pico Térmico via EWMA:** Para antecipar o estrangulamento térmico antes que a temperatura crítica seja atingida, o sistema calcula a Média Móvel Exponencialmente Ponderada da taxa de variação de calor $\Delta T$:

$$\text{EWMA}_t = \alpha \cdot \left(\frac{T_t - T_{t-1}}{\Delta t}\right) + (1 - \alpha) \cdot \text{EWMA}_{t-1}$$

Com o fator de suavização calibrado para $\alpha = 0,35$. Se a derivada térmica projetar que a GPU atingirá $82^\circ\text{C}$ nos próximos $3\text{ segundos}$, o sistema entra imediatamente em estado preventivo.

### 2.3 As Três Faixas de Operação e a Função de Barreira $\Phi$  

O despachante de inferência e o algoritmo ParetoBandit utilizam o estado do Watchdog para modular suas decisões através de três faixas operacionais rigorosas:

| **Faixa Operacional** | **Condição Físico-Térmica**                                                                                            | **Ação Imediata do Sistema**                                                                                                   | **Barreira ParetoBandit Φ**                                 |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------- |
| **FAIXA NOMINAL**     | $V_{\text{livre}} > 1.400\text{ MB}$ **e** $T_{\text{GPU}} < 75^\circ\text{C}$                                         | Autorização plena para geração rápida do Tier 1 na dGPU.                                                                       | $\Phi = 0$                                                  |
| **FAIXA PREVENTIVA**  | $800\text{ MB} < V_{\text{livre}} \le 1.400\text{ MB}$ **ou** $75^\circ\text{C} \le T_{\text{GPU}} < 82^\circ\text{C}$ | Congelamento de novos contextos. Redução do teto de geração de tokens. Embeddings desviados para CPU ONNX.                     | $\Phi = \lambda \cdot \frac{1.400 - V_{\text{livre}}}{600}$ |
| **FAIXA CRÍTICA**     | $V_{\text{livre}} \le 800\text{ MB}$ **ou** $T_{\text{GPU}} \ge 82^\circ\text{C}$                                      | **Interrupção imediata da GPU.** Aborto de inferência local, `cudaDeviceReset` e fallback gracioso para Nuvem (Tier 3) ou CPU. | $\Phi = +\infty$                                            |

A formulação matemática da função de barreira termodinâmica $\Phi(T_{\text{GPU}}, V_{\text{livre}}, k)$ é integrada diretamente na função de utilidade escalarizada do ParetoBandit:

$$\Phi(T_{\text{GPU}}, V_{\text{livre}}, k) = \begin{cases} 0 & \text{se } k \notin \{\text{Tier 1}, \text{Tier 2}\} \text{ ou } (V_{\text{livre}} > 1.400\text{ MB} \land T_{\text{GPU}} < 75^\circ\text{C}) \\ \lambda \cdot \left(\frac{1.400 - V_{\text{livre}}}{600}\right) + \mu \cdot \left(\frac{T_{\text{GPU}} - 75}{7}\right) & \text{se } 800\text{ MB} < V_{\text{livre}} \le 1.400\text{ MB} \lor 75^\circ\text{C} \le T_{\text{GPU}} < 82^\circ\text{C} \\ +\infty & \text{se } V_{\text{livre}} \le 800\text{ MB} \lor T_{\text{GPU}} \ge 82^\circ\text{C} \end{cases}$$

Onde $\lambda = 2,5$ e $\mu = 1,8$ são fatores de penalização calibrados empiricamente.

### 2.4 KV Cache Assimétrico e FlashAttention-2

Para que o Tier 1 mantenha uma janela de contexto utilizável de $8.192\text{ tokens}$ sem exceder o teto de $3.000\text{ MB}$ alocado para o modelo na VRAM:

1. **Quantização Assimétrica do Cache:**
    - Tensores de Chave (**Key Cache**): Mantidos em **FP16** para preservar a fidelidade posicional de atenção em código-fonte.
    - Tensores de Valor (**Value Cache**): Quantizados em **Q4_0**, reduzindo o footprint de memória em mais de $60\%$ sem degradação mensurável de perplexidade.
2. **FlashAttention-2 Nativo:** Ativação compulsória dos kernels CUDA otimizados do FlashAttention-2 no backend do llama.cpp, reduzindo a pegada de tensores intermediários e acelerando a decodificação em $28\%$.
3. **Orçamento Rigoroso de Silício:**
    - Pesos do Modelo (`Qwen2.5-Coder-3B` Q4_K_M): $\sim 2.100\text{ MB}$.
    - KV Cache Assimétrico (8K context): $\sim 900\text{ MB}$.
    - Subtotal CUDA: $3.000\text{ MB}$.
    - Reserva do WDDM 3.x: $850\text{ MB}$.
    - **Consumo Total:** $3.850\text{ MB}$.
    - **Folga Térmica e de Fragmentação Garantida:** $1.444\text{ MB}$ livres na dGPU.

### 2.5 Isolamento Estrito: Vetores e AST a 0 MB de VRAM

Nenhum outro subsistema do Souls Engine tem permissão para alocar memória de vídeo na RTX 2060m:
- O banco vetorial LanceDB opera **estritamente acoplado ao Dev Drive ReFS via `mmap` e RAM física do sistema (**$0\text{ MB}$ **de VRAM)**.
- Os modelos de embedding (`bge-small-en-v1.5`) e classificação sintática (`ModernBERT` / `GLiClass`) rodam **exclusivamente via ONNX Runtime em CPU AVX2 (**$0\text{ MB}$ **de VRAM)**.
- Parsers Tree-sitter e algoritmos Myers Diff rodam **exclusivamente na CPU (**$0\text{ MB}$ **de VRAM)**.

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Erradicação do Colapso por Shared Memory:** O sistema nunca atinge o teto do WDDM, impedindo que o Windows descarregue tensores via PCIe 3.0. A taxa de geração permanece estável na faixa de $45\text{ a }50\text{ tokens/s}$.
- **Proteção Físico-Mecânica do Notebook:** O acionamento preventivo do Watchdog impede que a GPU opere de forma sustentada acima de $75^\circ\text{C}$, eliminando quedas abruptas de clock por throttling térmico e reduzindo o desgaste das ventoinhas.
- **Resiliência Transacional FinOps:** Se o hardware estiver sob estresse térmico em dias quentes ou sob cargas simultâneas, o sistema não quebra: ele recua de modo matemático e gracioso para Nuvem (Tier 3 Fast) ou CPU, garantindo que o agente conclua sua tarefa.
- **Previsibilidade Estrita de Heap:** A segregação total de VRAM para o Tier 1 torna o consumo de memória de vídeo determinístico e reprodutível.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**           | **Impacto Técnico**                                                                                         | **Mitigação Canônica Implementada**                                                                                                                                                                    |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Latência na Troca de Pesos (Swap)**  | Descarregar o Tier 1 e carregar o Tier 2 exige leitura de disco no NVMe ($\sim 1,5\text{ a }2,5\text{ s}$). | Pré-carregamento dos pesos do Tier 2 na RAM física do sistema (Page Cache do ReFS) via `memmap2`, acelerando a transferência para a VRAM; ou desvio compulsório do Tier 2 para execução nativa em CPU. |
| **Sobrecarga da Thread de Telemetria** | Consultar a NVML a cada 500 ms consome ciclos de CPU.                                                       | Chamadas FFI puras via driver kernel-mode com consumo inferior a $0,05\%$ de uma única thread do processador Intel i9.                                                                                 |
| **Limitação a um Único Usuário Local** | A dGPU não suporta múltiplos streams paralelos pesados de LLMs.                                             | A arquitetura foi concebida como motor de assistência individual bare-metal; concorrência adicional é absorvida por subagentes na nuvem.                                                               |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE COEXISTÊNCIA CUDA:** É expressamente proibido propor rotinas que instanciem dois contextos de modelos de linguagem simultaneamente na GPU RTX 2060m. Tier 1 e Tier 2 são mutuamente exclusivos na VRAM.
2. **PROIBIÇÃO DE ELEVAÇÃO DE THRESHOLDS:** É proibido alterar o limiar crítico do Watchdog para valores superiores a $82^\circ\text{C}$ ou reduzir a margem livre de VRAM para menos de $800\text{ MB}$. Os parâmetros de segurança física são invioláveis.
3. **PROIBIÇÃO DE VRAM FORA DE `souls_inference_runtime`:** Nenhuma linha de código em `souls_memory` (LanceDB), `souls_ast` (Tree-sitter) ou `souls_core` pode requisitar memória de vídeo. Qualquer tentativa de mover embeddings ou vetores para CUDA resultará em rejeição sumária de PR.
4. **MANDATORIEDADE DE FLASHATTENTION-2 E KV CACHE ASSIMÉTRICO:** Toda configuração de arranque de modelos locais em CUDA deve explicitar `flash_attn = true` e quantização assimétrica de cache (`cache_type_k = "f16"`, `cache_type_v = "q4_0"`).
5. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** Propostas arquiteturais que desativem o Watchdog NVML ou ignorem a penalidade $\Phi = +\infty$ no ParetoBandit sofrerão **rejeição sumária de PR** com violação de conformidade do ADR-004.

## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Técnica Canônica v7:** Seção 5 (_Taxonomia dos 8 Tiers de Modelos e Governança Físico-Computacional_) e Seção 6.2 (_Formalização Matemática do ParetoBandit_).
- **`AGENTS.md`:** Seção 5 (_Governança Físico-Computacional e FinOps - RTX 2060m_).
- **`CRATES_SPECIFICATION.md`:** Seção 2.5 (`souls_inference_runtime`) e Seção 2.6 (`souls_model_router`).
- **`MCP_TOOL_CONTRACTS.md`:** Código de Erro `-32002` (`VramThermalThrottled`).
- **ADR-001:** _Abandono de UI Desktop em Favor de Daemon Headless_.
- **ADR-003:** _Rejeição de Proxy L7 em Favor de Roteamento de Subagentes_.