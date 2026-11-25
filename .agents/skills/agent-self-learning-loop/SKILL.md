---
name: agent-self-learning-loop
description: Playbook para o ciclo de auto-aprendizado e evolução contínua em malha fechada. Rege a captura de correções do Arquiteto (/learn), análise de causa-raiz e auto-patch permanente em regras, skills e memórias.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
---

# Agent Self-Learning Loop — Playbook de Aprendizado Contínuo

Este playbook rege o ciclo de aprendizado em malha fechada (*closed-loop learning*), garantindo que toda correção, feedback negativo ou novo padrão ensinado pelo Arquiteto Humano seja absorvido de forma definitiva, impedindo regressões em sessões futuras.

## 🔄 O PRINCÍPIO DO PATCH PERPÉTUO
O agente nunca se limita a pedir desculpas e refazer o código. Diante de qualquer correção ou comando `/learn`, ele executa o protocolo de **Triagem Epistêmica**:

```
           Feedback ou Correção do Operador (/learn)
                               │
                               ▼
                   [Classificador de Causa-Raiz]
                               │
         ┌─────────────────────┼─────────────────────┐
         ▼                     ▼                     ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│  REGRA DE REPO  │   │ GUIA DE SKILL   │   │ FATO / MEMÓRIA  │
│  (AGENTS.md /   │   │ (skills/*.md)   │   │ (Souls L2 +     │
│   .agents/rules)│   │                 │   │  Obsidian L3)   │
└─────────────────┘   └─────────────────┘   └─────────────────┘
```

---

## 🛠️ FLUXO OPERACIONAL EM 3 ETAPAS

### Etapa 1: Diagnóstico da Correção
Identifique a natureza da instrução do operador:
* **Caso A (Convenção ou Guardrail de Código):**
  * Exemplo: *"Nunca use CRLF no Windows"* ou *"Sempre passe `-p <crate>` no cargo test"*.
  * **Ação:** Atualize compulsoriamente o [AGENTS.md](file:///Z:/souls_engine/AGENTS.md) ou crie/edite um arquivo em `.agents/rules/`.
* **Caso B (Procedimento de Uso de Ferramenta):**
  * Exemplo: *"Antes de refatorar funções públicas, sempre meça o blast radius"*.
  * **Ação:** Atualize a seção de instruções do playbook correspondente em `.agents/skills/<skill>/SKILL.md`.
* **Caso C (Fato Duradouro, Conceito ou Preferência Pessoal):**
  * Exemplo: *"O modelo do chat interativo mestre é fixo e nunca muda"* ou *"Bruno prefere explicações concisas com equações"*.
  * **Ação:**
    1. Grave em **L2** no Souls Engine via `knowledge(action="store", category="architecture", partition="STABLE")`.
    2. Registre em **L3** criando ou enriquecendo o verbete correspondente no Obsidian (`wiki/entities/bruno.md` ou `wiki/concepts/...`) com linha de log em `wiki/log.md`.

### Etapa 2: Aplicação Imediata do Patch
* Aplique a alteração no arquivo alvo com finais de linha `LF`.
* Garanta que a regra seja expressa em linguagem prescritiva e inequívoca ("SEMPRE", "NUNCA", "OBRIGATÓRIO").
* **Confira a durabilidade do alvo antes de escolhê-lo.** Uma skill regenerada por sync/build externo sem proteção de hash sofre overwrite silencioso. Para preferências permanentes do operador, prefira artefato durável (bloco protegido por marcadores `<!-- LOCAL_DIRECTIVES_START -->` / `<!-- LOCAL_DIRECTIVES_END -->` ou nota atômica no cofre + índice/log) e promova a `c1` apenas com validação explícita (`c1` é *fail-closed* por SHA-256 e exige re-assinatura).
* **Após qualquer sync/build, releia o arquivo em disco** antes de assumir que o patch sobreviveu; `success` na escrita não é evidência de persistência.

### Etapa 3: Confirmação de Aprendizado
Ao responder ao operador, confirme em uma única frase cirúrgica onde a lição foi perpetuada:
> *"Entendido. Regra perpetuada em `AGENTS.md` e verbete `[[wiki/concepts/...]]` atualizado. Não cometerei esse erro novamente."*
