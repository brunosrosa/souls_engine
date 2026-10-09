# 🏟️ SOULS ARENA — Relatório Executivo de Benchmarks de Silício Local

> **Data de Geração:** Unix Epoch 1791514930  
> **Hardware Host:** Intel Core i9-9880H (AVX2) | 32GB DDR4 | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)  
> **Governança:** ADR-004 (VRAM Ceiling 5.2GB) & ADR-007 (Desacoplamento Ontológico dos Benchmark Types)

## ⚡ Type 1: Hardware Profile (Desempenho Físico e Latência)

| Model ID | Tier | Engine | TTFT (ms) | TPOT (ms) | Sustained TPS | Peak VRAM (MB) | Process RAM (MB) | E³ Score | Status |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| `HuggingFaceTB_SmolLM3-3B-Q5_K_M` | Tier 1 | `mock` | 45.0 | 23.5 | **42.5** | 3120.0 | 280.0 | 9999.00 | `NOMINAL` |
| `smollm2-360m-instruct-q8_0` | Tier 0.5 | `mock` | 45.0 | 14.7 | **68.0** | 1429.0 | 280.0 | 9999.00 | `NOMINAL` |
| `smollm-135m-instruct.Q4_K_M` | Tier 0 | `mock` | 45.0 | 9.1 | **110.0** | 1429.0 | 280.0 | 9999.00 | `NOMINAL` |
| `SmolLM2-135M-Reasoning-5K-Q4_K_M` | Tier 0 | `mock` | 45.0 | 9.1 | **110.0** | 1429.0 | 280.0 | 9999.00 | `NOMINAL` |

---
*Gerado automaticamente pela CLI `souls_arena` (Souls Engine v7).*
