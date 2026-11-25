---
name: souls-safe-sandbox
description: Playbook para execução segura de comandos de terminal e processos em sandbox restrita. Governa execute e shell com isolamento contra travamentos de terminal e higienização ANSI.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "sys"
---

# Souls Safe Sandbox — Playbook de Execução Segura

Este playbook rege a execução de comandos de sistema operacional e ferramentas de linha de comando no Windows 11.

## 🛡️ GUARDRAILS DE EXECUÇÃO
1. **Zero Comandos Bloqueantes Interativos:** Comandos que aguardam entrada de usuário (`stdin`) interativa sem timeout são proibidos.
2. **Higienização de Escape ANSI:** O descritor de saída do comando é sanitizado contra sequências de escape ANSI que corrompem o parsing de texto.
3. **Isolamento de Diretório:** Todo comando deve especificar o diretório de trabalho (`cwd`) dentro do Dev Drive `Z:\souls_engine` ou diretórios permitidos, nunca alterando o ambiente global com `cd`.

---

## 🛠️ FLUXO OPERACIONAL

### 1. Execução Controlada com `execute`
Para rodar comandos atômicos de compilação ou ferramentas CLI:
```json
execute({
  "command": "cargo test -p souls_server -- --nocapture",
  "cwd": "Z:/souls_engine",
  "timeout_secs": 60
})
```

### 2. Uso de Shell com Poda de Logs
Se o comando produzir grandes volumes de log:
* Use `shell(command="...", max_output_lines=200)` para prevenir estouro de contexto do agente.

### 3. Higiene Bare-Metal de Subprocessos Rust no Windows
Ao implementar ou auditar chamadas de subprocessos em Rust (`std::process::Command`) sob Windows 11:
1. **Nunca use `.output()` sem teto temporal:** Se o processo filho travar (ex.: `llama-completion.exe` sob WDDM), o handle permanece preso indefinidamente e gera processos zumbis órfãos.
2. **Padrão Canônico Anti-Deadlock:**
   * Configure `.stdin(Stdio::null())`, `.stdout(Stdio::piped())`, `.stderr(Stdio::null())` (ou drenado).
   * Drene `stdout` em uma `std::thread::spawn` dedicada enquanto o loop principal faz polling em `child.try_wait()` (evita *deadlock* no buffer de 4 KB de pipe do Windows).
   * No vencimento do timeout (ex.: `SOULS_LLAMA_TIMEOUT_SECS`), execute compulsoriamente `let _ = child.kill(); let _ = child.wait();` antes de retornar erro.
3. **Testes Determinísticos com Env Vars:** Crates cujos testes alteram variáveis globais do processo (`SOULS_HERMES_CONFIG_PATH`, `SOULS_GOOGLE_DATA_DIR`) devem ser validadas com `cargo test -p <crate> -- --test-threads=1`.
