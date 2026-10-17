---
name: obsidian-privacy-shield
description: Playbook de segurança, desidentificação e soberania de dados para domínios de saúde e finanças no cofre Obsidian. Impõe higienização local de PII antes de qualquer análise analítica de fronteira.
metadata:
  version: "1.1.0"
  author: "Souls Engine Architecture"
  vault_root: "C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB"
---

# Obsidian Privacy Shield — Playbook de Soberania & Desidentificação PII (v7)

Este playbook governa a blindagem de privacidade de dados pessoais, médicos e financeiros armazenados no cofre Obsidian, equilibrando **soberania de identidade** com **excelência cognitiva analítica**.

---

## 🔒 REGRA INVIOLÁVEL: NUNCA ENVIAR PII DIRETO PARA A NUVEM

É **expressamente proibido** enviar dados de identificação pessoal (*Personally Identifiable Information* - PII) brutos para qualquer provedor de nuvem pública sem sanitização prévia:
* `raw/financas/`: Extratos bancários, faturas de cartão Ourocard, investimentos, números de conta/agência, senhas e chaves PIX.
* `raw/saude/`: Exames laboratoriais, relatórios médicos, biometria (smartwatch), prescrições contendo nome de médico/paciente ou CRM.
* Documentos contendo CPFs, RG, endereços residenciais ou dados de identificação civil.

---

## 🛠️ O PIPELINE HÍBRIDO DE DESIDENTIFICAÇÃO (LOCAL SANITIZER + FRONTIER REASONING)

> **Princípio Fundamental:** A análise de saúde e finanças **não pode ser rebaixada para modelos locais fracos (SLMs)** que alucinem em diagnósticos ou façam planejamento contábil raso.  
> O modelo local atua como **Agente de Redação/Sanitização de PII**, e o modelo de fronteira atua como o **Analista Clínico/Financeiro de Alta Inteligência**.

```
    Documento em raw/financas/ ou raw/saude/
                       │
                       ▼
    [1. Higienização Local de PII no Silício]
    - souls_core::pii_redactor (Aho-Corasick linear) ou Tier 0 local
    - Substituição de CPF, Nome, Contas por tokens:
      * Nome -> [USUARIO_ALVO]
      * Conta 12345-6 -> [CONTA_CORRENTE_A]
      * Laboratório X -> [LAB_REFERENCIA]
                       │
                       ▼
    [2. Payload Clínico / Financeiro Anonimizado]
    (Apenas biomarcadores, taxas, valores orçamentários limpos de identidade)
                       │
                       ▼
    [3. Análise Analítica de Ponta na Fronteira]
    - Modelo de alta capacidade (Zero Data Retention)
    - Correlações complexas de saúde, tendências de exames, planejamento fino
                       │
                       ▼
    [4. Consolidação Segura no Cofre]
    - Verbete atômico gerado em wiki/concepts/ ou wiki/entities/
    - Log de auditoria em wiki/log.md
```

---

## 🛡️ MODOS OPERACIONAIS DE PRIVACIDADE

1. **Modo `HYBRID_ANONYMIZED` (Padrão Recomendado):**
   - Redige PII localmente no silício (custo zero).
   - Envia o corpo analítico puro para o modelo de fronteira de alto contexto.
   - Entrega a máxima sofisticação médica e financeira com total blindagem de identidade.
2. **Modo `STRICT_LOCAL` (Para Notas Ultra-Confidenciais):**
   - Processamento 100% contido na dGPU RTX 2060m e CPU host i9 com AVX2 (`/v1/chat/completions` local em `127.0.0.1:9123`).
   - Zero tráfego de rede para documentos marcados com `confidentiality: airgap`.
