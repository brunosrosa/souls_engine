# LESSONS_LEARNED_AND_POSTMORTEM.md — Caderno de Lições Aprendidas, Anti-Patterns & Post-Mortems (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (DOCS-AS-GUARDRAILS)  
> **DATA DE CONSOLIDAÇÃO:** 2026-10-12  
> **DISCIPLINA OPERACIONAL:** Agent Self-Learning Loop (`/learn`)

---

## 1. POST-MORTEM: A CORRUPÇÃO DO `config.yaml` DO HERMES AGENT

### 1.1 Contexto & Sintoma
Ao executar o comando `hermes update`, o utilitário acusava a seguinte quebra sintática:
```text
Your settings file (C:\Users\rosas\AppData\Local\hermes\config.yaml) has a formatting error at line 8.
Details: expected <block end>, but found '<block mapping start>'
```

### 1.2 Causa-Raiz Técnica (Root Cause Analysis)
O manipulador de presets do Web Dashboard (`crates/souls_server/src/routes/dashboard.rs`) utilizava substituição ingênua de linhas baseada em casamento de prefixos:
```rust
// ANTI-PATTERN: Substituição de texto cega em arquivo YAML hierárquico
if line.trim_start().starts_with("model:") {
    new_lines.push(format!("model: \"{}\"", target_model));
}
```
No arquivo `config.yaml` do Hermes Agent, a chave `model:` de nível superior não é um escalar puro, mas o início de um bloco mapeado com campos aninhados:
```yaml
model:
  default: meituan/longcat-2.5-preview:free
  provider: nous
  base_url: 'https://inference-api.nousresearch.com/v1'
```
Ao substituir a linha `model:` por `model: "deepseek/deepseek-chat"`, a linha seguinte (`  default: ...`) tornou-se um filho mapeado sob um valor escalar. Em conformidade com a especificação formal do YAML, isso gera erro imediato de gramática (`expected <block end>, but found '<block mapping start>'`).

### 1.3 Solução Implementada & Regra Canônica
1. **Parser Consciente de Contexto:** A rota foi refatorada no commit `e3269f8` para rastrear o estado hierárquico (`inside_top_model`) e atualizar **exclusivamente** o campo `  default:` sob o bloco `model:` raiz, preservando todas as demais chaves e indentação.
2. **Lei Negativa:** É **estritamente proibido** aplicar regex ou `starts_with` em arquivos de configuração YAML/TOML sem parsing estruturado ou rastreamento explícito de blocos.

---

## 2. POST-MORTEM: CONFLITO DE PORTA TCP (WSAEADDRINUSE) E PROCESSOS ZUMBIS

### 2.1 Contexto & Sintoma
Ao tentar abrir uma segunda instância do `souls_server.exe` ou `souls_engine.exe` enquanto outro processo já ocupava a porta `127.0.0.1:9123`, o `TcpListener::bind` retornava erro fatal de sistema operacional (`os error 10048`), causando saída abrupta com `ExitCode(1)`.

### 2.2 Solução Bare-Metal: Single-Instance Probe & PID Tracking
No commit `1163afb`:
1. **Sonda de Saúde HTTP Imediata:** Diante do erro de bind, o processo emite um ping assíncrono para `http://127.0.0.1:9123/health` (timeout de 800ms).
   - Se responder **200 OK Saudável**: O processo reconhece que o Souls Engine já está operando normalmente, exibe mensagem clara ao usuário e encerra a execução redundante com **código 0 (sucesso limpo)**, evitando janelas de erro no Windows.
   - Se não responder (processo travado/órfão): O processo emite alerta preciso informando que a porta está ocupada por um processo zumbi e instrui o uso de `stop_souls.bat`.
2. **Rastreamento de Ciclo de Vida via PID:** Gravação determinística de `souls_server.pid` no diretório `.souls_data` durante a inicialização e exclusão atômica no graceful shutdown.
3. **Purga em Lote:** O script `stop_souls.bat` finaliza concorrentemente instâncias de `souls_server` e `souls_engine`.

---

## 3. APRENDIZADO: A DUPLA IDENTIDADE DE BINÁRIOS NATIVOS NO RUST CARGO

### 3.1 Contexto
O usuário solicitou um executável com a identidade direta `souls_engine.exe`, enquanto a crate executável canônica do workspace foi nomeada historicamente como `souls_server` (em aderência à Linha Vermelha 1 do `AGENTS.md` como daemon headless).

### 3.2 Solução de Engenharia & Build Release
1. Duplicação de targets `[[bin]]` com o mesmo `src/main.rs` em um único `Cargo.toml` gera warning de manifesto no compilador (`file found to be present in multiple build targets`).
2. Para manter **0 warnings no Cargo**, preservamos o binário canônico `souls_server` e construímos a esteira de publicação em `tools/build_souls_release.ps1`, que compila em modo `--release` otimizado (MSVC x86_64, AVX2, LTO) e gera simultaneamente:
   - `target\release\souls_server.exe` (34.9 MB)
   - `target\release\souls_engine.exe` (34.9 MB)
3. O launcher de bandeja (`tools/souls_launcher.ps1`) e o script `start_souls.bat` foram programados para priorizar o executável de release `souls_engine.exe` caso exista, com fallback transparente para o modo debug.

---

## 4. APRENDIZADO: RESTRIÇÕES DE PRIVILÉGIOS NO `hermes update` SOB WINDOWS 11

### 4.1 Contexto
Ao rodar `hermes update`, pacotes binários dentro de `%LOCALAPPDATA%\hermes\tools\` (como `ffmpeg-9.0.1-win32-x64`) podem falhar ao serem renomeados para `.previous-*` com `[WinError 5] Acesso negado`.

### 4.2 Causa & Procedimento
1. O Windows 11 bloqueia renomeações atômicas de pastas se houver processos em segundo plano com handles de DLL ou arquivo abertos (ex: áudio do Hermes ou terminal em execução).
2. O comando `hermes update` deve ser executado obrigatoriamente a partir de um terminal **PowerShell com privilégios de Administrador**, com os gateways do Hermes previamente pausados (`hermes gateway stop`).
