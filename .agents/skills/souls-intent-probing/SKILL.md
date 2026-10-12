---
name: souls-intent-probing
description: Playbook para medição de entropia e classificação de intenção socrática via Tier 0/0.5 SLM no Souls Engine. Governa intent para detecção de ambiguidade antes de tarefas pesadas.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "think"
---

# Souls Intent Probing — Playbook de Probing de Intenção (v7)

Este playbook rege a análise rápida de ambiguidade informacional e intenção em prompts de entrada utilizando modelos locais ultraleves (Tier 0 / 0.5 executados na CPU Host Intel i9 a 0 MB de VRAM).

## 🎯 QUANDO UTILIZAR
Quando o prompt do operador for curto, ambíguo ou de alta entropia (ex.: *"refatore isso"*, *"conserte"* ou instruções com múltiplas interpretações possíveis):

1. **Invocação de `intent`:**
   ```json
   intent({
     "prompt": "Instrução recebida do operador",
     "context_hint": "Último arquivo editado"
   })
   ```
2. **Interpretação da Entropia de Shannon:**
   * Se o modelo Tier 0.5 retornar $\mathcal{H} \ge 0.75$ (alta incerteza/entropia):
     * O agente não deve chutar ou iniciar alterações destrutivas; deve interpelar o Arquiteto Humano via protocolo `/grill-me`.
   * Se $\mathcal{H} < 0.40$ (baixa entropia / intenção clara):
     * O agente prossegue autonomamente com a execução sem interrupções desnecessárias.
