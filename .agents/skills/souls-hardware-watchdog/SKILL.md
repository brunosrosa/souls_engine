---
name: souls-hardware-watchdog
description: Playbook para monitoramento termodinâmico da GPU RTX 2060m e FinOps de silício via NVML. Governa metrics e feedback para proteção contra throttling térmico e estouro de VRAM sob WDDM.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ops"
---

# Souls Hardware Watchdog — Playbook de Governança de Silício (v7)

Este playbook rege a observabilidade física em tempo real do hardware bare-metal hospedeiro (NVIDIA GeForce RTX 2060m 6GB e Intel Core i9-9880H).

## 🌡️ LIMITES FÍSICOS INVIOLÁVEIS (ADR-004)
1. **Teto Útil WDDM da VRAM:** O Windows DWM consome até 850 MB de VRAM. A margem útil real para cargas CUDA é de **5.294 MB**.
2. **Barreira Térmica:** A temperatura de junção da GPU não deve ultrapassar **$78^\circ\text{C}$**.
3. **Spillover PCIe Proibido:** Se a alocação de vídeo ultrapassar o teto útil, o Windows aciona a memória compartilhada via PCIe, derrubando a taxa de 45 tok/s para menos de 1,5 tok/s.

---

## 🛠️ FLUXO OPERACIONAL

### 1. Inspecione a Telemetria com `metrics`
Antes de inicializar subagentes pesados locais ou batches de inferência:
* Invoque `metrics()`.
* **Verificações Críticas:**
  * Se $\text{VRAM}_{\text{livre}} \le 800\text{ MB}$: desvie a tarefa para execução em CPU Host ou Nuvem.
  * Se $T \ge 78^\circ\text{C}$: pause execuções locais para resfriamento passivo.

### 2. Auditoria FinOps com `feedback`
Para inspecionar o histórico de custos acumulados e score $E^3$:
* `feedback(limit=20)`: extrai médias de tokens gerados, tempos até o primeiro token ($TTFT$) e custos agregados.
