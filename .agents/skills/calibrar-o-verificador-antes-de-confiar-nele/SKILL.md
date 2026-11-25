---
name: calibrar-o-verificador-antes-de-confiar-nele
description: "Gates deterministicos e juizes LLM produziram 21 casos de falso positivo/negativo nesta sessao: txt[:2000] truncava frontmatter; TODO sem word boundary casava T"
version: "0.1.0"
author: "Souls Ralph Loop"
license: "MIT"
tags: ["ops", "evolutive-harness", "ralph-loop"]
---

# Calibrar o verificador antes de confiar nele

## When to Use
Use this skill whenever the agent encounters the recurring problem pattern:
- Gates deterministicos e juizes LLM produziram 21 casos de falso positivo/negativo nesta sessao: txt[:2000] truncava frontmatter; TODO sem word boundary casava Todo; pipe escapado de tabela parecia wikilink quebrado; equivalencia numerica laxa rotulava SOURCED falso (13 modelos, ADR-004, 0 MB); rubrica cega sem sidecar acusava EXTERNAL legitimo como invencao; e um patch inseriu entradas fora do dict enquanto o log ja dizia corrigido.

## Procedure
1. Antes de valer qualquer gate, rode-o contra 1 caso bom + 1 caso com defeito real + o dataset de FPs conhecidos; meça FP e FN. 2. Gates medem SEMANTICA/conteudo, nunca presenca/ausencia de string exata. 3. Toda correcao de gate e uma NOVA MEDICAO declarada (T1-b, T2-b); nunca substitua o resultado oficial. 4. Juizes LLM recebem sempre claim_provenance.json: EXTERNAL com source_hint nao e invencao. 5. Nunca registre "corrigido" sem reler em disco (py_compile + hash). 6. Congele o limiar ANTES de medir e explicite o criterio de descarte por camada. 7. Equivalencia numerica exige contexto/unidade; caso incerto vai para qualifier needs_review, nunca para EXTERNAL.

## Guardrails & Pitfalls
- Never overwrite constitutionally pinned `c1` files (`AGENTS.md`, `SOUL.md`).
- Execute candidate validators strictly in a credential-scrubbed (`SCRUB_ENV`) workspace.

## Verification
- Confirm the issue does not recur and record positive feedback if resolved cleanly.
