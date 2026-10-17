---
name: obsidian-vault-curator
description: Playbook mandatório para curadoria do cofre Obsidian (raw/ -> wiki/). Rege a extração de conceitos e entidades, formatação de frontmatter YAML, links bidirecionais [[...]] e registro compulsório em wiki/log.md.
metadata:
  version: "1.1.0"
  author: "Souls Engine Architecture"
  vault_root: "C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB"
---

# Obsidian Vault Curator — Playbook Canônico de Curadoria (v7)

Este playbook governa a transformação contínua de anotações e pesquisas brutas (`raw/`) em conhecimento atômico, hiperconectado e enciclopédico (`wiki/`) no cofre Obsidian em `C:\Users\rosas\OneDrive\Documentos\Obisidian DB`.

## 📜 AS 6 LEIS SUPREMAS DO COFRE (AGENTS.MD)

1. **`raw/` é Imutável e Sagrado (Read-Only Total):**  
   É **terminantemente proibido** modificar, renomear, mover ou apagar qualquer arquivo ou diretório dentro de `raw/`. Todo arquivo em `raw/` é dado primário puro.
2. **`wiki/` é Dinâmica e Atômica:**  
   Todo conhecimento compilado reside exclusivamente em `wiki/` segregado em:
   * `wiki/entities/`: Pessoas, organizações, projetos ou ferramentas (`bruno.md`, `souls_engine.md`, `hermes_agent.md`).
   * `wiki/concepts/`: Conceitos abstratos, regras, metodologias e padrões (`regras_categorizacao.md`, `simbiose_agente.md`).
   * `wiki/summaries/`: Sínteses densas de pesquisas de múltiplos arquivos (`pesquisa_deep_learning_2026.md`).
3. **Frontmatter YAML Padronizado Obrigatório na Wiki:**  
   Toda nota criada ou editada na `wiki/` deve conter compulsoriamente no topo:
   ```yaml
   ---
   type: entity | concept | summary
   tags: [dominio, subcategoria]
   sources: ["[[raw/subpasta/arquivo.md]]"]
   last_updated: YYYY-MM-DD
   reviewed: false
   ---
   ```
   *Novas chaves de metadados devem seguir estritamente `snake_case` (ex: `source_reliability`, `confidence_level`).*
4. **Preservação Aditiva e Não-Destrutiva no `raw/`:**  
   Caso um documento em `raw/` já possua cabeçalho YAML pré-existente (com tags, autor, data, etc.), é **terminantemente proibido** remover, truncar ou colapsar esses campos. O agente opera estritamente em modo de **merge aditivo**, preservando os campos originais do autor.
5. **Links Bidirecionais `[[...]]` Obrigatórios:**  
   Todo novo verbete deve conter referências cruzadas `[[caminho/nota|Título]]` conectando-se a entidades e conceitos pais (ex: `[[wiki/entities/bruno|Bruno]]`, `[[wiki/index|Índice Geral]]`).
6. **Auditoria Compulsória em `wiki/log.md`:**  
   Toda criação ou alteração na `wiki/` deve ser registrada na tabela do arquivo `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\wiki\log.md`:
   ```markdown
   | YYYY-MM-DD | [[wiki/concepts/novo_conceito.md|novo_conceito]] | Criação | Síntese de pesquisa bruta em raw/projetos/x.md |
   ```

---

## 🛡️ LIMITES DE AUTONOMIA & CADÊNCIA OPERACIONAL

* **Cadência Canônica de Execução (The 30-Minute Drip):**  
  A esteira opera em micro-lotes de **2 a 3 arquivos a cada 30 minutos** (via agendamento no Hermes Agent ou despacho Antigravity CLI). Essa cadência garante que ~40 a 50 notas sejam digeridas por dia útil sem estourar rate limits, sem superaquecer silício e liquidando o acervo de 498 arquivos em 10 a 12 dias.
* **Perfil Cognitivo Recomendado (FinOps & Long Context):**  
  Para síntese documental da Wiki, priorize modelos com **janela de contexto longa ($\ge 1\text{M}$ tokens)** e alta eficiência de custo (ex: `Gemini Flash`, `Claude Haiku/Sonnet`, `DeepSeek V4 Flash`). Não desperdice modelos de raciocínio formal pesado de alto custo ($5+/1M) para leitura e fatiamento documental básico.
* **ALWAYS (Executar Sem Confirmação):**
  * Ler arquivos em `raw/`.
  * Criar/editar notas em `wiki/concepts/`, `wiki/entities/` e `wiki/summaries/`.
  * Adicionar frontmatter YAML e links bidirecionais `[[...]]`.
  * Atualizar o índice mestre `wiki/index.md` e registrar a ação em `wiki/log.md`.
* **ASK FIRST (Pedir Confirmação Antes):**
  * Lotes que processem mais de 5 a 10 arquivos simultaneamente.
  * Contradições factuais diretas entre dois documentos antigos sem cronologia clara.
* **NEVER (Proibição Absoluta):**
  * Modificar ou truncar o corpo de dados dentro de `raw/`.
  * Excluir notas de `wiki/` sem mover cópia de segurança para `wiki/archive/`.
  * Enviar exames médicos, extratos bancários ou PII brutos para APIs públicas sem o processo prévio de anonimização (ver `obsidian-privacy-shield`).

---

## 🛠️ FLUXO DE EXECUÇÃO & DESPACHO VIA ANTIGRAVITY CLI

O Hermes Agent (Orquestrador Master) não deve inchar sua janela de contexto lendo dezenas de páginas de documentos do `raw/`. Ele despacha o **Antigravity CLI (`agy`)** como subagente executor isolado via terminal:

```bash
agy "Execute a curadoria dos próximos 2 arquivos PENDING de 'C:\Users\rosas\OneDrive\Documentos\Obisidian DB\raw' com base no raw_manifest.json, gere os verbetes conceituais em wiki/concepts/ ou wiki/entities/, anote em wiki/log.md e marque como CURATED."
```

### Passos Internos do Subagente Executor:
1. **Inspeção do Manifesto:** Leia `wiki/raw_manifest.json` para identificar os próximos 2 ou 3 arquivos com status `PENDING`.
2. **Leitura e Extração Fiel:** Leia o arquivo bruto em `raw/` aproveitando a janela longa de contexto para capturar o quadro geral.
3. **Destilação Atômica:**
   * Crie notas atômicas em `wiki/concepts/` ou `wiki/entities/` (arquivo nomeado em `snake_case.md`).
   * Adicione o frontmatter YAML completo com `sources: ["[[raw/...]]"]`.
   * Crie os wikilinks para notas correlatas existentes.
4. **Registro de Auditoria:**
   * Adicione a linha de log em `wiki/log.md`.
   * Atualize o `wiki/raw_manifest.json` marcando o arquivo como `CURATED` e associando as notas geradas.
