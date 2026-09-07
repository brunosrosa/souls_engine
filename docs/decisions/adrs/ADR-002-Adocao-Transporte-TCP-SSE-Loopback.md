# ADR-002 — Adoção de Localhost TCP Loopback e Streamable HTTP/SSE (Porta :9123) sobre Win32 Pipes Anônimos (STDIO) no Windows 11

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)
> **DATA DE RATIFICAÇÃO:** Setembro de 2026 (Linha de Base v7 Canônica)
> **CRATES DIRETAMENTE REGIDAS:** `souls_server`, `souls_protocol`, `souls_core`.
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code) e Engenheiros de Sistemas.

## 1. CONTEXTO E FORÇAS EM CONFLITO

Na especificação original do Model Context Protocol (MCP) promovida pela Anthropic, o vetor de transporte primário documentado e amplamente disseminado entre clientes CLI e IDEs de desenvolvimento é o **Standard Input / Standard Output (`stdio`)**, operando através de pipes anônimos do sistema operacional.

Nas iterações experimentais do Souls Engine operando no Windows 11 acoplado ao interpretador Python 3.11 do **Hermes Agent** (Nous Research), o uso de STDIO revelou-se estruturalmente instável, suscetível a impasses irreversíveis (_deadlocks_) e incompatível com os requisitos de concorrência e telemetria contínua do motor bare-metal.

As seguintes forças de engenharia de baixo nível forçaram a revisão do vetor de transporte:

### 1.1 A Falha Estrutural do Buffering em Pipes Anônimos Win32 (MSVCRT)

No kernel Windows NT, a semântica de descritores de console difere radicalmente da implementação POSIX:
- Quando um processo pai em Python instancia um processo filho em Rust utilizando `subprocess.Popen` com redirecionamento de `stdin` e `stdout` para `subprocess.PIPE`, o subsistema Win32 aloca pipes anônimos no driver de sistema `NPFS` (Named Pipe File System).
- O runtime C da Microsoft (**MSVCRT / UCRT**), linkado estaticamente ou dinamicamente aos binários nativos no Windows, inspeciona o descritor de arquivo através da API Win32 `GetFileType()`. Quando o descritor não aponta para um console de caractere interativo (`FILE_TYPE_CHAR`), o runtime **desativa compulsoriamente o line-buffering** e ativa o **block-buffering automático de** $4\text{ KB}$ **a** $8\text{ KB}$.
- Se o binário Rust ou o interpretador Python emitir mensagens serializadas em JSON-RPC 2.0 que não completem exatamente o bloco de $4.096\text{ bytes}$, os dados ficam retidos indefinidamente no buffer intermediário do runtime NT.
- Caso ambos os processos aguardem mutuamente por dados adicionais para fechar um frame antes de emitir um `flush` atômico, o canal entra em **impasse fatal silencioso (**_**deadlock**_**)**, paralisando o agente sem disparar exceções de timeout rastreáveis.

### 1.2 A Corrupção de Frames por Conversão CRLF (`\r\n` vs `\n`)

O subsistema Win32 opera historicamente sob a convenção de quebras de linha Carriage Return + Line Feed (`\r\n`).
- Em ambientes Windows, descritores de arquivo abertos em modo texto convertem silenciosamente o caractere LF (`\n`) em CRLF (`\r\n`).
- No protocolo MCP e nos envelopes JSON-RPC 2.0, mensagens estruturadas delimitadas por comprimento de bytes (`Content-Length: <n>\r\n\r\n<payload>`) ou delimitadas por nova linha sofrem desvios métricos de contagem.
- A injeção espúria de bytes `\r` altera os deslocamentos físicos computados pelos parsers Tree-sitter em memória (`souls_ast`) e gera erros catastróficos de desserialização no despachante RPC, resultando em falhas crônicas de protocolo (`ParseError: -32700`).

### 1.3 Limitações de Acesso Concorrente e Multiplexação em STDIO

Um pipe anônimo STDIO é um canal linear estritamente sequencial e unidirecional em cada extremidade.
- Ele impede que múltiplos componentes do sistema (ex: a suíte de benchmarks `souls_llm_local_arena`, o Thin Adapter de memória do Hermes, e o Hermes Web Dashboard) acessem simultaneamente o motor.
- O acoplamento via STDIO amarra o ciclo de vida do daemon Rust ao ciclo de vida de uma sessão interativa individual do terminal, inviabilizando que o Souls Engine permaneça rodando como serviço de infraestrutura perene durante a reinicialização do cliente Hermes ou comutação de perfis.

### 1.4 A Inviabilidade dos Named Pipes Win32 (`\\.\pipe\*`) para o Ecossistema Web

Embora os Named Pipes Win32 locais ofereçam altíssimo rendimento tirando proveito de I/O Completion Ports (IOCP), sua integração é deficiente:
- O framework do Hermes Agent incorpora nativamente suporte a transportes MCP baseados em **HTTP com Server-Sent Events (SSE)** (especificação MCP 2024-11-05).
- O `ProactorEventLoop` do Python 3.11 no Windows possui peculiaridades complexas na sincronização de Named Pipes assíncronos com threads de trabalho, além de exigir tratamento manual de Security Descriptors e Access Control Lists (ACLs) no Windows NT.
- Named Pipes não são inspecionáveis por navegadores web convencionais, impedindo que o Web Dashboard do Hermes (`hermes dashboard`) ou ferramentas de diagnóstico consumam telemetria de forma direta.

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretada a **adoção de Localhost TCP Loopback com transporte Streamable HTTP/SSE na porta fixa `127.0.0.1:9123` como o canal de transporte canônico primário do Souls Engine**.

O binário executável `souls_server.exe` passa a servir o catálogo MCP e as APIs do sistema via pilha assíncrona HTTP/1.1 baseada na crate **Axum** e **Tokio**, mantendo o canal **Standard I/O (`stdio`) exclusivamente como vetor secundário de compatibilidade local para IDEs**.

### 2.1 Especificação da Topologia de Rede em Loopback

1. **Endereço e Porta Soberana:** O servidor Axum vincula-se estritamente na interface de loopback local:
    $$\text{Socket Address} = 127.0.0.1:9123$$
    É terminantemente proibido vincular em interfaces abertas (`0.0.0.0` ou IPs de rede local).
2. **Mecanismo de Streaming SSE (`/mcp`):**
    - O cliente Hermes conecta-se ao endpoint `http://127.0.0.1:9123/mcp` via requisição HTTP `GET` persistente com cabeçalho `Accept: text/event-stream`.
    - O servidor estabelece um canal de saída de dados unidirecional contínuo através de Server-Sent Events (SSE).
    - O envio de comandos e invocações de ferramentas (`tools/call`) pelo Hermes ocorre via requisições HTTP `POST` independentes no mesmo endpoint ou na rota designada pelo handshake SSE.
3. **Arquitetura Winsock2 sobre I/O Completion Ports (IOCP):**
    - A camada de rede alavanca o driver de sockets `msafd.dll` do Windows NT acoplado ao escalonador de IOCP multithread do runtime Tokio.
    - As conexões persistentes operam com a diretiva `TCP_NODELAY = true` (desativação do algoritmo de Nagle), garantindo que pacotes JSON-RPC pequenos sejam despachados imediatamente para a pilha do kernel sem latência artificial.
4. **Trânsito com Finais de Linha LF Imutáveis:**
    - Todas as respostas HTTP e fluxos de eventos SSE trafegam com delimitadores canônicos estritamente higienizados com quebras de linha `\n` (LF) nos corpos das ferramentas, anulando desvios métricos de AST.

### 2.2 Papel Secundário Restrito do STDIO (Dev IDE Direct Pipe)

O canal Standard I/O (`stdio`) é mantido no binário `souls_server.exe` exclusivamente para conexão direta com editores locais (Cursor, Windsurf, Trae) operando via subprocessos de extensão:
- **Descontaminação Absoluta:** O descritor `stdout` é isolado e reservado com exclusividade para frames JSON-RPC 2.0 puros.
- **Proibição de ANSI e Logging:** Qualquer macro `println!`, `eprintln!` ou saída de depuração de bibliotecas externas (ex: logs internos do ONNX Runtime ou llama.cpp) é categoricamente redirecionada para arquivos de log estruturados em `Z:\souls_engine\.souls_data\spool\` ou canalizada pelo descritor `stderr` formatado.
- **Esvaziamento Compulsório:** Toda escrita no pipe de saída em modo stdio deve invocar `stdout.flush().await` explicitamente após cada mensagem.

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas

- **Erradicação Completa de Deadlocks:** O fluxo TCP sobre Winsock2 gerencia o controle de fluxo via janelas deslizantes na pilha IP do kernel, eliminando travamentos causados por buffering de bloco do MSVCRT.
- **Desacoplamento Total de Ciclos de Vida:** O daemon `souls_server.exe` pode ser iniciado antes do Hermes Agent e permanecer vivo continuamente. O Hermes pode reiniciar, recarregar plugins ou sofrer reinicializações transitórias sem derrubar a infraestrutura bare-metal.
- **Suporte Nativo ao Comando `/reload-mcp`:** A reconexão com o servidor MCP pode ser executada instantaneamente no Hermes através do fechamento e reabertura da conexão SSE em loopback, reconstruindo as tabelas de símbolos em menos de 50 ms sem reiniciar processos de sistema operacional.
- **Acesso Concorrente Multiponto:** Um único daemon `souls_server.exe` atende simultaneamente:
    1. O loop deliberativo do Hermes Agent via transporte MCP HTTP/SSE.
    2. O Thin Adapter Python de persistência de memória (`/api/v1/memory/*`).
    3. O proxy reverso FastAPI do Hermes Web Dashboard (`/api/v1/telemetry`).
    4. Sessões interativas pontuais de IDEs via loopback.
- **Throughput Elevado e Baixa Latência:** Em regime de tráfego local no Windows 11 (Loopback TCP persistente com Keep-Alive), as medições de latência situam-se na faixa de $50\ \mu\text{s}$ **a** $150\ \mu\text{s}$ por turno, com throughput sustentado de $600\text{ MB/s}$ **a** $900\text{ MB/s}$.

### 3.2 Desvantagens Identificadas e Mitigações

| **Desvantagem Identificada**             | **Impacto Técnico**                                                                         | **Mitigação Canônica Implementada**                                                                                                                                                                                                                         |
| ---------------------------------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Colisão de Portas de Rede**            | Se outro serviço alocar a porta `:9123`, o arranque do daemon falha.                        | Verificação preventiva no boot com verificação de socket via `SO_EXCLUSIVEADDRUSE` do Windows NT; se a porta estiver ocupada por instância zumbi anterior, emissão de erro claro com diagnóstico PID.                                                       |
| **Overhead Mínimo do Protocolo TCP**     | Adição de cabeçalhos IP/TCP em comparação com descritores de pipes nativos em memória pura. | Ativação mandatória de `TCP_NODELAY` e conexões persistentes HTTP/1.1 Keep-Alive, amortizando o custo de handshake de socket para zero em regime de operação normal.                                                                                        |
| **Vulnerabilidade a Varreduras de Rede** | Risco teórico de processos locais não autorizados consultarem o daemon.                     | Vinculação estrita à interface de loopback (`127.0.0.1`), sem exposição em bridges ou adaptadores Wi-Fi/Ethernet, complementada por cabeçalho opcional de validação de token de sessão local gerado no boot em `Z:\souls_engine\.souls_data\session.token`. |

## 4. LINHAS VERMELHAS E GUARDRAILS PARA AGENTES DE IA

Qualquer agente de IA operando no repositório (Cursor, Windsurf, Claude Code, etc.) deve aderir estritamente às seguintes restrições:
1. **PROIBIÇÃO DE EXPOSIÇÃO EM INTERFACES PÚBLICAS:** É expressamente proibido alterar o endereço de escuta do servidor Axum para `0.0.0.0` ou aceitar conexões externas de rede. A vinculação deve ser imutavelmente `127.0.0.1:9123`.
2. **PROIBIÇÃO DE ACOPLAMENTO MONOLÍTICO VIA SUBPROCESS STDIO PARA PRODUÇÃO:** Em integrações com o Hermes Agent, é categoricamente proibido orientar a configuração para pipes de processo anônimos locais via `command: souls_server.exe` em arquivos de produção do Hermes. A integração canônica deve apontar compulsoriamente para `url: http://127.0.0.1:9123/mcp`.
3. **PROIBIÇÃO DE PAYLOADS CRLF:** Todo módulo que devolve dados para o cliente HTTP/SSE ou canal stdio deve higienizar o fluxo, assegurando que finais de linha sejam rigorosamente `\n` (LF puro).
4. **PROIBIÇÃO DE LOGS NO STDOUT:** Nenhuma crate pode imprimir mensagens via `println!`, `dbg!` ou macros semelhantes para o descritor padrão de saída. Toda observabilidade deve utilizar a crate `tracing` estruturada com redirecionamento não-bloqueante para arquivos de spool ou `stderr`.
5. **CRITÉRIO DE REJEIÇÃO IMEDIATA:** Qualquer proposta arquitetural que tente reverter o transporte do Hermes para subprocessos anônimos sem autorização prévia resultará em **rejeição sumária de PR** com violação de conformidade do ADR-002.

## 5. REFERÊNCIAS CRUZADAS E DOCUMENTAÇÃO VINCULADA

- **Constituição Técnica Canônica v7:** Seção 2 (_Transporte Heterogêneo e Concorrência IPC no Windows 11 Nativo_).
- **`AGENTS.md`:** Seção 4 (_Matriz Simbiótica com o Hermes Agent - Vetor de Transporte Canônico_).
- **`MCP_TOOL_CONTRACTS.md`:** Seção 1.1 (_Vetores de Transporte e Nomenclatura Soberana_).
- **`CRATES_SPECIFICATION.md`:** Seção 2.9 (_Superfície Pública e Anti-Patterns da crate souls_server_).
- **ADR-001:** _Abandono de UI Desktop em Favor de Daemon Headless_.