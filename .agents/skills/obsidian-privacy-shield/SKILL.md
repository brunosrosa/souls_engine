---
name: obsidian-privacy-shield
description: Playbook de segurança e soberania de dados para domínios sensíveis do cofre Obsidian. Proíbe envio de dados de saúde, finanças e PII para APIs de nuvem com treino aberto; impõe processamento 100% local no silício.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  vault_root: "C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB"
---

# Obsidian Privacy Shield — Playbook de Soberania de Dados PII (v7)

Este playbook governa a blindagem de privacidade de dados pessoais, médicos e financeiros armazenados no cofre Obsidian.

## 🔒 REGRA INVIOLÁVEL DE SOBERANIA LOCAL
É **expressamente proibido** enviar para provedores de nuvem públicos ou APIs de treinamento livre (ex.: Gemini Free Tier, endpoints sem zero-data-retention) qualquer conteúdo oriundo de:
* `raw/financas/`: Extratos bancários, faturas de cartão Ourocard, investimentos, comprovantes e dados fiscais.
* `raw/saude/`: Exames laboratoriais, biometria (Samsung Health), relatórios médicos, prescrições e histórico clínico.
* Documentos contendo PII (*Personally Identifiable Information*): CPFs, senhas, documentos de identidade e endereços.

---

## 🛠️ FLUXO DE PROCESSAMENTO LOCAL

```
Documento em raw/financas/ ou raw/saude/
                   │
                   ▼
       [Verificação de Rota PII]
                   │
                   ▼
  Execução 100% Local no Silício Bare-Metal:
  - dGPU RTX 2060m (Tier 1: Qwen2.5-Coder / Llama-3.2)
  - CPU Host Intel i9 AVX2 (Tier 0 / Tier 0.5)
                   │
                   ▼
  Geração do Verbete em wiki/ com Custo Zero e Risco Zero
```

1. **Classificação Automática:** Ao processar um arquivo que resida em `raw/financas/` ou `raw/saude/`, o agente ativa a tag `LOCAL_ONLY`.
2. **Despacho Bare-Metal:** O processamento deve ocorrer estritamente via motor local do Souls Engine (`souls_inference_runtime` / llama.cpp) ou endpoints locais do Hermes (`http://127.0.0.1:9123/v1/chat/completions`).
3. **Higienização de Entidades:** Ao sintetizar notas na `wiki/`, valores específicos de transações podem ser agregados em resumos categóricos para manter a clareza sem redundância desnecessária.
