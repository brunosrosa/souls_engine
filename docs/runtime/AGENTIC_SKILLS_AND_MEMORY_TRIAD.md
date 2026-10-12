# AGENTIC_SKILLS_AND_MEMORY_TRIAD.md — Matriz de Skills, Memória Tri-Tier e Curadoria (v7 Canônica)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E VINCULANTE (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** 2026-10-12  
> **APLICAÇÃO:** Hermes Agent Desktop/CLI, Antigravity CLI/IDE, Souls Engine Daemon.  
> **ADRS VINCULADOS:** `ADR-003`, `ADR-005` (Emenda 3.3), `ADR-008`, `ADR-009`.  

---

## 1. PREÂMBULO E A ONTOLOGIA DA SIMBIOSE

A autonomia agêntica de alto nível requer mais do que apenas expor 52 ferramentas atômicas MCP (`tools`): exige **procedimentos operacionais padronizados (Skills)**, uma **hierarquia de memória em 3 níveis** e **reflexos comportamentais condicionados** para que o sistema funcione silenciosamente, sem demandar microgerenciamento humano.

```
                   O ARQUITETO HUMANO (BRUNO)
                              │
                    Meta e Diálogo Executivo
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 HERMES AGENT (Python 3.11)                  │
│       - Chat Mestre (Master Brain na Nuvem / Nous)          │
│       - Orquestrador Executivo & Delegador de Tarefas       │
└──────────────────────────────┬──────────────────────────────┘
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
┌───────────────────────────┐         ┌───────────────────────────┐
│       SOULS ENGINE        │         │        OBSIDIAN DB        │
│   (127.0.0.1:9123 / L2)   │         │ (OneDrive C:\ / L3 Frio)  │
│  - Silício Local (dGPU)   │         │  - raw/ (Imutável)        │
│  - AST & Myers Diff       │         │  - wiki/ (Conceitos/MOCs) │
│  - LadybugDB & LanceDB    │         │  - log.md & Manifest      │
│  - ParetoBandit Router    │         │  - Blindagem PII Local    │
└───────────────────────────┘         └───────────────────────────┘
```

---

## 2. A ARQUITETURA DE MEMÓRIA EM 3 NÍVEIS (3-TIER EPISTEMIC MEMORY)

| Nível | Camada | Mídia Física | Latência | Escopo de Conteúdo | Mutabilidade |
|:---:|---|---|:---:|---|:---:|
| **L1** | **Memória Ativa** | RAM / Context Window do LLM | $0\text{ ms}$ | Histórico do turno corrente, variáveis de sessão, raciocínio imediato. | Volátil |
| **L2** | **Hipocampo Bare-Metal** | Souls Engine (`souls_state.db`, LanceDB, LadybugDB) em ReFS `Z:\` | $< 20\text{ ms}$ | Fatos de arquitetura de código, telemetria FinOps, dependências de AST, grafo ontológico de crates e sessões de subagentes. | Dinâmico / Langevin Decay |
| **L3** | **Enciclopédia Fria** | Cofre Obsidian (`C:\Users\rosas\OneDrive\Documentos\Obisidian DB`) | $< 100\text{ ms}$ | Conhecimento de longo prazo: saúde, finanças, projetos de vida, deep researchs, diário de decisões e síntese enciclopédica. | Permanente / Auditado |

### 2.1 Protocolo de Roteamento de Memória (Quando Ler e Onde Gravar)
1. **Regra de Consulta Inicial:** Diante de perguntas conceituais ou dúvidas sobre código, o agente consulta **L2** via `semantic_search` ou `knowledge`. Se a pergunta envolver diretrizes do usuário, contexto de projetos ou finanças/saúde, consulta **L3** na `wiki/` do Obsidian.
2. **Regra de Registro de Código:** Toda convenção ou contrato técnico consolidado é persistido em **L2** (`knowledge` ou `mem_create_entities`).
3. **Regra de Registro Holístico:** Toda decisão estratégica, reflexão de sessão ou aprendizado conceitual gera um relatório em `raw/sessions/` no Obsidian e é destilado em verbetes atômicos na `wiki/`.

---

## 3. A MATRIZ DAS 22 SKILLS CANÔNICAS (7 DOMÍNIOS OPERACIONAIS)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     CATÁLOGO DAS 22 SKILLS CANÔNICAS                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ 🦀 DOMÍNIO A: Engenharia de Código & Cirurgia de AST                        │
│   1. souls-code-surgeon       │ Desidratação: outline -> symbol -> edit     │
│   2. souls-structural-diff    │ Substituição atômica e Myers diff           │
│   3. souls-blast-radius       │ Medição de impacto e Frecency (repo_impact) │
│   4. souls-safe-sandbox       │ Execução isolada em sandbox de processo     │
├─────────────────────────────────────────────────────────────────────────────┤
│ 🧠 DOMÍNIO B: Hipocampo L2 Bare-Metal & Grafo Ontológico                    │
│   5. souls-memory-triad       │ Busca Híbrida RRF (LanceDB 0MB + BM25 FTS5) │
│   6. souls-knowledge-vault    │ Versionamento de diretrizes epistêmicas     │
│   7. souls-ontology-graph     │ Nós e arestas tipadas no LadybugDB          │
│   8. souls-langevin-decay     │ Metabolismo e expurgo de memórias fracas    │
├─────────────────────────────────────────────────────────────────────────────┤
│ ⚖️ DOMÍNIO C: Deliberação Cognitiva & Freio Socrático                        │
│   9. souls-socratic-thinking  │ Grafo DAG de hipóteses em tarefas complexas │
│  10. souls-intent-probing     │ Probing de ambiguidade via entropia e SLM   │
│  11. souls-speculative-branch │ Ramificação concorrente e merge consensual  │
├─────────────────────────────────────────────────────────────────────────────┤
│ ⚡ DOMÍNIO D: Silício, FinOps & Orquestração de Agentes                     │
│  12. souls-subagent-dispatch  │ Consulta ParetoBandit antes de delegar      │
│  13. souls-hardware-watchdog  │ Monitoramento térmico e VRAM na RTX 2060m   │
│  14. souls-cloud-arena-advisor│ Consulta de campeões e tarifas de nuvem     │
├─────────────────────────────────────────────────────────────────────────────┤
│ 🗜️ DOMÍNIO E: Contexto Lean, Compressão & Web                               │
│  15. souls-context-lean       │ Compressão LEAN e desidratação CCR          │
│  16. souls-web-intelligence   │ Busca DuckDuckGo e conversão HTML->Markdown │
├─────────────────────────────────────────────────────────────────────────────┤
│ 📚 DOMÍNIO F: Memória Fria de Longo Prazo (Obsidian DB)                     │
│  17. obsidian-vault-curator   │ Extração raw/ -> wiki/ com frontmatter YAML │
│  18. obsidian-manifest-tracker│ Rastreamento incremental de hashes SHA-256  │
│  19. obsidian-privacy-shield  │ Blindagem local (zero cloud) para saúde/PII │
│  20. obsidian-session-logger  │ Arquivamento de decisões em raw/sessions/   │
├─────────────────────────────────────────────────────────────────────────────┤
│ 🔄 DOMÍNIO G: Auto-Refinamento & Aprendizado Contínuo                       │
│  21. agent-self-learning-loop │ Captura e roteamento de correções (/learn)  │
│  22. agent-skill-evolution    │ Auto-auditoria periódica de playbooks       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. TOPOLOGIA FÍSICA E RESILIÊNCIA CONTRA DESMONTAGEM DE `Z:` (ADR-005)

Para garantir que o ecossistema permaneça **100% operacional** mesmo se a partição virtual `Z:` (Dev Drive ReFS) estiver desmontada durante o arranque do Windows:

1. **Lar Primário de Execução das Skills (`C:\`):**  
   `C:\Users\rosas\AppData\Local\hermes\skills\` (subpastas estruturadas por namespace).
2. **Espelhamento Global do Antigravity (`C:\`):**  
   `C:\Users\rosas\.gemini\config\skills/` (permite que a IDE execute as skills em qualquer projeto).
3. **Repositório Soberano de Versão (`Z:\`):**  
   `Z:\souls_engine\.agents\skills/` (versionado no Git do repositório Souls Engine).
4. **Script de Curadoria do Obsidian (`C:\`):**  
   `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\scripts\curate_obsidian.ps1` (reside diretamente no cofre e é sincronizado pelo OneDrive, independente de `Z:`).

---

## 5. A ESTEIRA DE CURADORIA INCREMENTAL DO OBSIDIAN DB

### 5.1 O Contrato de Governança do Cofre
* **`raw/` é Sagrado e Imutável:** Leitura estrita; **zero** edições, movimentações ou deleções.
* **`wiki/` é Dinâmica e Hiperconectada:** Verbetes com frontmatter YAML estrito:
  ```yaml
  ---
  type: entity | concept | summary
  tags: [dominio, subtopico]
  last_updated: YYYY-MM-DD
  reviewed: false
  ---
  ```
* **Auditoria Contínua:** Todas as ações são registradas na tabela de `wiki/log.md`.
* **Privacidade Local Absoluta:** Pastas sensíveis (`raw/financas/`, `raw/saude/`) têm tráfego de nuvem bloqueado; a curadoria é executada exclusivamente por modelos locais (Tier 1/Tier 0.5 via Souls Engine).

### 5.2 O Manifesto de Progresso (`raw_manifest.json`)
Armazenado em `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\wiki\raw_manifest.json`:
* Rastreia o hash SHA-256 de cada documento bruto.
* Estados de processamento: `PENDING`, `IN_PROGRESS`, `CURATED`, `SKIPPED`.
* Mapeamento de quais notas em `wiki/` foram derivadas de cada arquivo bruto.

### 5.3 Modelo de Disparo Híbrido
* **Sob Demanda:** Disparado pelo humano ou agente via PowerShell:
  ```powershell
  pwsh "C:\Users\rosas\OneDrive\Documentos\Obisidian DB\scripts\curate_obsidian.ps1" -BatchSize 5
  ```
* **Automático:** Cron job periódico no Hermes Agent (`%LOCALAPPDATA%\hermes\cron`) verificando novos arquivos.
* **Lote Seguro:** Limite fixo de até 5 arquivos por rodada, cumprindo a regra de autonomia $\le 10$ do cofre.

---

## 6. O CICLO DE AUTO-APRENDIZADO EM MALHA FECHADA (`/learn`)

Quando o Arquiteto Humano ensina um padrão ou corrige um comportamento:
1. **Ativação:** Comando `/learn` ou detecção de correção negativa ("não faça assim").
2. **Classificação e Aplicação:**
   * **Convenção de Código:** Gravada em `Z:\souls_engine\.agents\rules\` ou `AGENTS.md`.
   * **Procedimento de Ferramenta:** Atualiza o arquivo `SKILL.md` correspondente no Hermes e espelho.
   * **Conhecimento Duradouro:** Gravado em `knowledge` no Souls Engine (L2) e verbete correspondente no Obsidian (L3) com entrada no `wiki/log.md`.
3. **Garantia de Não-Regressão:** Toda nova sessão de agentes herda imediatamente as regras atualizadas.
