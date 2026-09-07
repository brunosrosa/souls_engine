# ADR-001 — Abandono de Interfaces Gráficas Desktop (Tauri v2 / Systray) em Favor de Daemon Headless Bare-Metal com Telemetria via Hermes Web Dashboard

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_server`, `souls_core`, `souls_inference_runtime`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

Nas iterações anteriores do projeto (eras SODA v5 e Souls MC v6), o sistema foi concebido como um aplicativo desktop unificado com interface gráfica nativa para Windows 11. Essa arquitetura utilizava:

1. O framework **Tauri v2** acoplado ao motor de renderização **Wry/Winit** (baseado no Microsoft Edge WebView2 nativo do Windows).
2. Um ícone permanente na área de notificação do Windows (**Systray** via crates `tray-icon` ou `tao`) governado por um loop de eventos de janela Win32 na thread principal.
3. Um console administrativo local compilado em **Svelte 5 (Runes)** com Tailwind CSS v4, cujos arquivos estáticos (`dist/`) eram incorporados diretamente no executável final via macro `rust-embed` e servidos por uma instância interna do servidor web Axum na porta `:3002`.

Embora a proposta inicial buscasse fornecer uma experiência "tudo-em-um" com controle visual imediato, a operação sob regime de estresse em hardware físico (CPU Intel Core i9-9880H e GPU dedicada NVIDIA GeForce RTX 2060m com 6.144 MB de VRAM) revelou conflitos de engenharia insolúveis entre a interface gráfica e os objetivos bare-metal do motor:

### 1.1 O Conflito de Silício: Otimização de VRAM vs. Consumo do WebView2

A margem útil de VRAM na RTX 2060m é severamente restrita. Sob o Windows Display Driver Model (WDDM 3.x), o Desktop Window Manager (`dwm.exe`) consome obrigatoriamente entre 500 MB e 850 MB de memória de vídeo para aceleração da interface do sistema operacional.

Quando o aplicativo instanciava a WebView2 do Edge para renderizar a interface Svelte:

- O subsistema gráfico do Chromium alocava entre **300 MB e 600 MB adicionais de VRAM** para texturas de renderização acelerada por hardware, pipelines de composição DirectX e swapchains DXGI.
- Somados os consumos do WDDM ($\approx 850\text{ MB}$) e da WebView2 ($\approx 500\text{ MB}$), a VRAM reservada antes de qualquer inferência ultrapassava **1.350 MB**.
- Essa perda reduzia o teto utilizável para o Tier 1 (`Qwen2.5-Coder` + FlashAttention-2 + KV Cache de 8K tokens) de 5.294 MB para menos de 4.794 MB.
- O menor pico de inferência forçava o Windows a acionar o **Shared GPU Memory Fallback** via barramento PCIe 3.0 x16, colapsando a taxa de geração de 45 tokens/s para menos de 1,5 tokens/s.

### 1.2 Vazamentos Térmicos e Estresse na GPU Móvel

A RTX 2060m é um componente térmico integrado em chassis de notebook. A execução contínua de um loop de renderização Chromium a 60/120 Hz, mesmo com janelas minimizadas ou em segundo plano, mantinha os clocks da GPU elevados em estados de energia de alta performance (P-States P0/P2).

Isso gerava uma temperatura basal em repouso de $62^\circ\text{C}$ **a** $68^\circ\text{C}$, empurrando a GPU prematuramente para a faixa preventiva ($75^\circ\text{C}$) ou crítica ($\ge 82^\circ\text{C}$) do Watchdog NVML no primeiro lote de inferência local, provocando _thermal throttling_ e acionamento ruidoso das ventoinhas de refrigeração.

### 1.3 Bloqueio de Threads NT e Concorrência de Mensagens Win32

Para gerenciar o ícone da Systray e as WebViews nativas, a thread principal do executável (`main.rs`) precisava rodar o clássico loop de despacho de mensagens Win32:

```
// Código legado com problemas crônicos de concorrência
while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
    TranslateMessage(&msg);
    DispatchMessageW(&msg);
}
```

Essa estrutura colidia diretamente com o modelo de concorrência assíncrona do **Tokio Runtime**. Operações de redimensionamento de janela, menus de contexto na bandeja ou suspensão transitória de foco pelo usuário pausavam a thread principal do sistema operacional, introduzindo jitter e latências esporádicas nos canais assíncronos MPSC e nas respostas do servidor MCP (`souls_mcp`).

### 1.4 Duplicação Funcional com o Ecossistema Hermes

Com a consolidação da simbiose com o framework **Hermes Agent** (Nous Research), o ecossistema passou a contar nativamente com o **Hermes Desktop** (interface executiva de diálogo e voz) e o **Hermes Web Dashboard** (`hermes dashboard`, interface unificada de governança). Manter uma terceira interface proprietária no Souls Engine configurava redundância desnecessária de código e desperdício de tokens de contexto durante a manutenção.

## 2. DECISÃO ARQUITETURAL

Fica decretada formalmente a **eliminação integral e irreversível de qualquer interface gráfica local, menus de notificação na bandeja (Systray) e servidores embutidos de frontend dentro do binário Rust do Souls Engine**.

O binário canônico `souls_server.exe` passa a ser um **Daemon de Infraestrutura 100% Headless Bare-Metal**, operando silenciosamente como um serviço de retaguarda no Windows 11.

### 2.1 Especificações da Operação Headless

1. **Ponto de Entrada Descontaminado:** O `main.rs` da crate `souls_server` inicializa diretamente o runtime multithread do Tokio otimizado para o processador Intel Core i9, alocando threads de trabalho puras sem acoplar loops de mensagens gráficos Win32 (`User32.dll` / `Gdi32.dll`).
2. **Consumo de Memória em Repouso:** Em estado ocioso (_idle_), sem modelos carregados na VRAM, o executável `souls_server.exe` consome entre **15 MB e 25 MB de RAM física**, com **estritamente 0 MB de VRAM alocados na dGPU RTX 2060m**.
3. **Comunicação de Baixo Nível:** A governança do daemon, o fornecimento de ferramentas e a telemetria operam exclusivamente através de dois vetores estáveis de comunicação:
    - **Localhost TCP Loopback com Streamable HTTP/SSE na porta `127.0.0.1:9123`** (rotas MCP, REST e OpenAI-compatible).
    - **Standard I/O (`stdio`) Descontaminado** (canal JSON-RPC 2.0 com isolamento estrito contra poluição de ANSI/stderr para integração direta com IDEs).

### 2.2 Terceirização da Observabilidade para o Hermes Web Dashboard

Toda a observabilidade visual, gráficos de hardware, calibração do ParetoBandit e telemetria de memória profunda são delegados ao ecossistema oficial do Hermes Agent através de um plugin modular em `$HERMES_HOME/plugins/souls_dashboard/`:
1. **Proxy Reverso FastAPI (`dashboard/plugin_api.py`):** O servidor web do Hermes detecta e monta automaticamente o roteador sob `/api/plugins/souls_dashboard/`, consumindo o endpoint REST `http://127.0.0.1:9123/api/v1/telemetry` exposto pelo daemon Rust.
2. **Componente Visual React (`dashboard/dist/index.js`):** Pacote compilado em JavaScript puro no formato IIFE, consumindo os componentes oficiais e o SDK global disponibilizados pelo Hermes em `window.__HERMES_PLUGIN_SDK__`.

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Recuperação de 100% da VRAM Operacional:** A GPU RTX 2060m não precisa renderizar uma única camada gráfica. Os 5.294 MB úteis ficam integralmente disponíveis para tensores CUDA do Tier 1 e KV Cache de 8.192 tokens.
- **Estabilidade Térmica:** A temperatura de repouso da GPU cai para a faixa de $42^\circ\text{C}$ **a** $48^\circ\text{C}$ (redução de mais de $20^\circ\text{C}$ em relação ao regime com WebView2 ativo), ampliando a margem para rajadas de inferência pesada sem atingir o limiar de estrangulamento térmico ($82^\circ\text{C}$).
- **Compilação e Link-Time Ultravelozes:** A remoção de `tauri`, `wry`, `tao`, `winit` e bibliotecas C++ associadas reduz o tempo de compilação limpa (`cargo build --release`) em mais de 65%, além de erradicar falhas crônicas de linkage com a toolchain MSVC no Windows 11.
- **Redução Drástica do Tamanho do Binário:** O executável compilado tem seu tamanho reduzido de $\approx 85\text{ MB}$ para menos de $\approx 18\text{ MB}$.
- **Zero Deadlocks de Mensagens Win32:** A concorrência assíncrona do Tokio opera sem bloqueios de renderização ou interferência de foco de digitação do usuário.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**                                               | **Impacto Técnico**                                                       | **Mitigação Canônica Implementada**                                                                                                                    |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Ausência de ícone na bandeja para verificar se o serviço está vivo.        | Usuário não visualiza um indicador visual permanente na barra de tarefas. | Implementação de subcomandos de terminal nativo rápido via `souls_server.exe status` e integração de monitoramento visual no Web Dashboard do Hermes.  |
| Impossibilidade de fechar o sistema com clique com botão direito do mouse. | Encerramento passa a depender de gerenciamento de processos.              | Implementação de ganchos de sinais Win32 para tratamento ordenado de `CTRL_C_EVENT` e `CTRL_SHUTDOWN_EVENT`, com Graceful Shutdown em 5 fases.         |
| Inviabilidade de edição de configurações por formulários nativos.          | Usuário não tem telas de configurações proprietárias locais.              | Edição centralizada através do arquivo estruturado `souls_config.toml` em `Z:\souls_engine\.souls_data\` ou via aba de governança no Hermes Dashboard. |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE ADIÇÃO DE CRATES DE UI / SYSTRAY:** É categoricamente proibido adicionar ao `Cargo.toml` raiz ou de qualquer crate as dependências:
    - `tauri`, `tauri-build`, `wry`, `tao`, `winit`, `tray-icon`, `muda`, `native-windows-gui`.
2. **PROIBIÇÃO DE EMPACOTAMENTO DE ARQUIVOS WEB VIA RUST-EMBED:** É proibido reintroduzir o macro `rust-embed` ou embutir código HTML/CSS/JS compilado no binário Rust.
3. **PROIBIÇÃO DE INICIALIZAÇÃO DE LOOPS GRÁFICOS WIN32:** Nenhuma crate pode invocar APIs de janelas do subsistema `Win32_UI_WindowsAndMessaging` ou bloquear threads com `GetMessageW` / `PeekMessageW`.
4. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** A presença de qualquer menção a renderização local, ícones de bandeja ou tentativa de restaurar o console `:3002` em revisões de código ou propostas arquiteturais implicará **rejeição sumária de PR** com violação de conformidade do ADR-001.

## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Canônica v7:** Seção 8 (_Interface, Telemetria e Hermes Web Dashboard_).
- **`AGENTS.md`:** Linha Vermelha 1 (_NENHUMA Interface Gráfica Local - Zero Desktop UI / Systray_).
- **`CRATES_SPECIFICATION.md`:** Seção 2.9 (_Anti-Patterns Estritos da crate souls_server_).
- **`DONOR_EXTRACTION_MAP.md`:** Seção 3.3 (_Necrose e Tecidos Biohazard em _donor/ - Tauri/Svelte_).