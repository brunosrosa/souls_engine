---
name: obsidian-vault-curator
description: Playbook mandatório para curadoria do cofre Obsidian (raw/ -> wiki/). Rege a extração de conceitos e entidades, formatação de frontmatter YAML, links bidirecionais [[...]] e registro compulsório em wiki/log.md.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  vault_root: "C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB"
---

# Obsidian Vault Curator — Playbook Canônico de Curadoria (v7)

Este playbook governa a transformação contínua de anotações e pesquisas brutas (`raw/`) em conhecimento atômico, hiperconectado e enciclopédico (`wiki/`) no cofre Obsidian em `C:\Users\rosas\OneDrive\Documentos\Obisidian DB`.

## 📜 AS 5 LEIS SUPREMAS DO COFRE (AGENTS.MD)

1. **`raw/` é Imutável e Sagrado (Read-Only Total):**  
   É **terminantemente proibido** modificar, renomear, mover ou apagar qualquer arquivo ou diretório dentro de `raw/`. Todo arquivo em `raw/` é dado primário puro.
2. **`wiki/` é Dinâmica e Atômica:**  
   Todo conhecimento compilado reside exclusivamente em `wiki/` segregado em:
   * `wiki/entities/`: Pessoas, organizações, projetos ou ferramentas (`bruno.md`, `souls_engine.md`, `hermes_agent.md`).
   * `wiki/concepts/`: Conceitos abstratos, regras, metodologias e padrões (`regras_categorizacao.md`, `simbiose_agente.md`).
   * `wiki/summaries/`: Sínteses densas de pesquisas de múltiplos arquivos (`pesquisa_deep_learning_2026.md`).
3. **Frontmatter YAML Padronizado Obrigatório:**  
   Toda nota criada ou editada na `wiki/` deve conter compulsoriamente no topo:
   ```yaml
   ---
   type: entity | concept | summary
   tags: [dominio, subcategoria]
   last_updated: YYYY-MM-DD
   reviewed: false
   ---
   ```
   *Novas chaves de metadados devem seguir estritamente `snake_case` (ex: `source_reliability`, `confidence_level`).*
4. **Links Bidirecionais `[[...]]` Obrigatórios:**  
   Todo novo verbete deve conter referências cruzadas `[[caminho/nota|Título]]` conectando-se a entidades e conceitos pais (ex: `[[wiki/entities/bruno|Bruno]]`, `[[wiki/index|Índice Geral]]`).
5. **Auditoria Compulsória em `wiki/log.md`:**  
   Toda criação ou alteração na `wiki/` deve ser registrada na tabela do arquivo `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\wiki\log.md`:
   ```markdown
   | YYYY-MM-DD | [[wiki/concepts/novo_conceito.md|novo_conceito]] | Criação | Síntese de pesquisa bruta em raw/projetos/x.md |
   ```

---

## 🛡️ LIMITES DE AUTONOMIA

* **ALWAYS (Executar Sem Confirmação):**
  * Ler arquivos em `raw/`.
  * Criar/editar notas em `wiki/concepts/`, `wiki/entities/` e `wiki/summaries/`.
  * Adicionar frontmatter YAML e links bidirecionais.
  * Atualizar o índice mestre `wiki/index.md` e registrar a ação em `wiki/log.md`.
* **ASK FIRST (Pedir Confirmação Antes):**
  * Lotes que processem mais de 5 a 10 arquivos simultaneamente.
  * Contradições factuais diretas entre dois documentos antigos sem cronologia clara.
* **NEVER (Proibição Absoluta):**
  * Tocar em qualquer arquivo dentro de `raw/`.
  * Excluir notas de `wiki/` sem mover cópia de segurança para `wiki/archive/`.
  * Enviar exames médicos, extratos bancários ou PII para APIs de nuvem com treino aberto.

---

## 🛠️ FLUXO DE EXECUÇÃO DE UM LOTE DE CURADORIA

1. **Inspeção do Manifesto:** Leia `wiki/raw_manifest.json` para identificar quais arquivos de `raw/` estão com status `PENDING`.
2. **Leitura do Arquivo Bruto:** Leia o arquivo em `raw/` sem suprimir detalhes técnicos ou contextuais.
3. **Extração e Destilação:**
   * Crie notas atômicas em `wiki/concepts/` ou `wiki/entities/` (arquivo nomeado em `snake_case.md`).
   * Adicione o frontmatter YAML completo.
   * Crie os wikilinks para notas correlatas.
4. **Registro de Auditoria:**
   * Adicione a linha de log em `wiki/log.md`.
   * Atualize o `wiki/raw_manifest.json` marcando o arquivo como `CURATED` e associando as notas geradas.
