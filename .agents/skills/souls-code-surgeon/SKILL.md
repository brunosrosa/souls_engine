---
name: souls-code-surgeon
description: Playbook mandatório para cirurgia sintática de código. Proíbe leitura de arquivos inteiros; impõe o fluxo outline -> symbol -> edit/delta_diff poupando 70-85% de tokens.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "code"
---

# Souls Code Surgeon — Playbook de Cirurgia Sintática (v7)

Este playbook rege a inspeção e modificação de código-fonte no workspace do Souls Engine (`Z:\souls_engine`).

## 🚨 REGRA CONSTITUCIONAL ANTI-INCHAÇO (FINOPS)
É **estritamente proibido** ler arquivos de código inteiros (`read_file` ou equivalente) para descobrir assinaturas, tipos, métodos ou arquitetura. Violações desta regra esgotam desnecessariamente o contexto do modelo e geram degradação de atenção.

---

## 🛠️ FLUXO OPERACIONAL EM 3 ETAPAS

```
1. outline(path) ────────► 2. symbol(path, symbol_query) ────────► 3. edit ou delta_diff
   (Visão Esquelética)        (Localização Cirúrgica de Linhas)       (Patch Atômico)
```

### Etapa 1: Inspeção Esquelética com `outline`
Ao investigar um módulo ou arquivo desconhecido:
* Chame `outline(path="caminho/do/arquivo.rs", include_private=false)`.
* **Resultado:** O parser Tree-sitter desidrata os corpos das funções e métodos, preservando 100% das assinaturas públicas, structs, enums, traits e docstrings com marcadores métricos (ex.: `/* ... omitted [32 lines] ... */`).
* **Economia:** 70% a 85% de redução de tokens em relação à leitura crua.

### Etapa 2: Localização de Coordenadas com `symbol`, `callers` e `callees`
Quando souber qual função, método ou struct precisa ser modificado:
* Use `symbol(path="...", symbol_query="nome_do_simbolo")` para extrair as coordenadas precisas (`file:line:col`) e o corpo exato daquele símbolo.
* Para analisar dependências de chamada antes de refatorar:
  * `callers(path="...", symbol_name="...")`: mapeia quem invoca esse símbolo no workspace.
  * `callees(path="...", symbol_name="...")`: mapeia quais métodos o símbolo consome internamente.

### Etapa 3: Patch Cirúrgico com `edit` ou `delta_diff`
* **Para edições pontuais (Search/Replace):**
  * Invoque `edit(path="...", old_content="...", new_content="...")`.
  * Garanta que `old_content` seja um trecho de código único e exato.
* **Para blocos estruturais ou verificação de diffs:**
  * Invoque `delta_diff(path="...", original_content="...", modified_content="...")` para calcular o Myers Diff estrutural antes de aplicar alterações destrutivas.

---

## 🔒 GUARDRAILS E HIGIENE DE COMPILAÇÃO
1. **Finais de Linha LF Obrigatórios:** Ao gerar ou editar qualquer arquivo, use exclusivamente quebras `\n` (LF puro). O parser de AST Tree-sitter e o cálculo de offsets dependem de contagens de bytes exatas.
2. **Verificação Compulsória Pós-Edição:** Ao terminar a edição de uma crate, execute obrigatoriamente:
   ```powershell
   cargo check -p <crate_name>
   cargo test -p <crate_name>
   ```
