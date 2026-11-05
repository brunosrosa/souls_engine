---
name: gemini-notebooklm-master
description: Guia mestre e playbook operacional completo para o Gemini Notebook (Google NotebookLM) via MCP e CLI. Cobre autenticação, RAG profundo, busca transversal cross-notebook, ingestão de fontes (Drive, Web, PDF), Studio criativo (Audio Overview, Slides, Mapas Mentais) e pipelines automatizados.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "tools"
---

# Gemini Notebook (NotebookLM) Master Playbook

Este playbook rege a orquestração e uso pleno do **Google NotebookLM** integrado via MCP (`notebooklm-mcp-cli` / `gemini-notebook-mcp`) e CLI local (`nlm`). Ele permite ao agente navegar por acervos de pesquisa, ingerir fontes, executar consultas RAG sem alucinação e gerar artefatos analíticos de alto nível.

---

## 🔑 1. ARQUITETURA DE ACESSO & AUTENTICAÇÃO

O MCP conecta-se diretamente aos endpoints RPC internos do Google NotebookLM utilizando a sessão autenticada local do usuário.

### 1.1 Configuração do Servidor MCP (`mcp_config.json`)
No arquivo de configuração MCP (ex: `c:\Users\rosas\.gemini\config\mcp_config.json`), o servidor deve apontar diretamente para o binário nativo instalado:

```json
{
  "mcpServers": {
    "gemini_notebook_mcp": {
      "command": "C:\\Users\\rosas\\.local\\bin\\notebooklm-mcp.exe"
    }
  }
}
```
> **Nota de Inicialização:** O binário opera em modo `stdio` nativo por padrão. Não utilize argumentos adicionais como `"mcp"`. Caso use via `uvx`, o pacote PyPI oficial é `notebooklm-mcp-cli`:
> `"args": ["--from", "notebooklm-mcp-cli", "notebooklm-mcp"]`.

### 1.2 Gestão de Sessão & Expiração de Cookies
Os cookies de sessão do Google (como `__Secure-1PSIDTS`) expiram periodicamente por razões de segurança.
* **Diagnóstico de Saúde:**
  ```powershell
  nlm doctor
  ```
* **Renovação de Login (Desktop Interativo):**
  Quando o MCP retornar `Authentication expired`, execute no terminal do Windows:
  ```powershell
  nlm login
  ```
  Isso abrirá uma janela do Chrome para autenticação segura. O servidor MCP detecta as novas credenciais instantaneamente sem necessidade de reiniciar.
* **Alternância de Contas:**
  Use a ferramenta `profile(action="list")` e `profile(action="switch", name="work")` para alternar entre contas pessoais e corporativas.

---

## 📚 2. GESTÃO DE CADERNOS & NAVEGAÇÃO

### 2.1 Listagem e Localização de Cadernos
* **Listar todos os cadernos:**
  ```json
  notebook_list()
  ```
* **Consultar detalhes de um caderno:**
  ```json
  notebook_get({"notebook_id": "<notebook_id_ou_alias>"})
  ```
* **Resumo executivo gerado por IA sobre o conteúdo do caderno:**
  ```json
  notebook_describe({"notebook_id": "<notebook_id>"})
  ```
* **Apelidos Rápidos (Aliases):**
  Para evitar manipular UUIDs extensos em sessões repetidas:
  ```json
  alias({"action": "set", "alias": "souls-arch", "target": "<uuid_do_caderno>"})
  ```
  A partir de então, qualquer chamada aceita `souls-arch` no parâmetro `notebook_id`.

---

## 🔍 3. CONSULTAS RAG & SÍNTESE COGNITIVA

O maior poder do NotebookLM reside na capacidade de síntese orientada por evidências extraídas estritamente das fontes enviadas, com zero alucinação externa.

### 3.1 Consulta Focada em um Caderno (`notebook_query`)
Pergunta direcionada ao acervo documental do caderno, retornando respostas com citações explícitas:
```json
notebook_query({
  "notebook_id": "souls-arch",
  "query": "Quais foram as razões técnicas para a rejeição do KùzuDB e escolha do LadybugDB?"
})
```

### 3.2 Consulta Transversal Entre Múltiplos Cadernos (`cross_notebook_query`)
Cruza simultaneamente o conhecimento de múltiplos cadernos independentes:
```json
cross_notebook_query({
  "notebook_ids": ["souls-arch", "voz-multimodal", "pesquisa-hardware"],
  "query": "Como as decisões de VRAM na RTX 2060m limitam a pipeline de voz e o grafo ontológico?"
})
```

### 3.3 Preservação de Descobertas como Notas
Salva um raciocínio gerado no chat diretamente como uma nota interna do caderno:
```json
note({
  "notebook_id": "souls-arch",
  "action": "create",
  "title": "Síntese: Ponto de Inflexão da Sessão 0",
  "content": "Conclusão da auditoria sobre o isolamento da Sessão 0 no Windows NT e impacto no NVML..."
})
```

---

## 📥 4. INGESTÃO & CURADORIA DE FONTES

### 4.1 Adicionar Fontes (`source_add`)
O NotebookLM suporta múltiplos tipos de fontes em uma única ferramenta:
* **Página Web:** `source_add({"notebook_id": "...", "source_type": "url", "url": "https://..."})`
* **Texto Puro / Markdown:** `source_add({"notebook_id": "...", "source_type": "text", "text": "...", "title": "Nota Técnica"})`
* **Arquivo Local (PDF, TXT, MD):** `source_add({"notebook_id": "...", "source_type": "file", "file_path": "Z:\\docs\\ref.pdf"})`
* **Google Drive:** `source_add({"notebook_id": "...", "source_type": "drive", "document_id": "<google_doc_id>"})`

### 4.2 Inspeção e Busca nas Fontes
* **Buscar trechos específicos:** `source_search({"notebook_id": "...", "query": "Langevin Decay"})`
* **Extrair o texto integral de uma fonte:** `source_get_content({"notebook_id": "...", "source_id": "..."})`
* **Sincronizar documentos do Drive:** `source_sync_drive({"notebook_id": "...", "source_id": "..."})`

---

## 🎨 5. STUDIO CRIATIVO & ARTEFATOS MULTIMODAIS

O NotebookLM Studio permite transformar fontes densas em artefatos multimídia de alta retenção:

### 5.1 Tipos de Artefatos Geráveis (`studio_create`)
```json
studio_create({
  "notebook_id": "souls-arch",
  "artifact_type": "audio",
  "audio_format": "deep_dive",
  "confirm": true
})
```
* **`audio` (Audio Overview / Podcast):** Diálogo entre dois anfitriões sintetizados debatendo as fontes em formato de podcast.
* **`slides` (Slide Deck):** Apresentação estruturada de slides com visual e roteiro.
* **`mindmap` (Mapa Mental):** Estrutura hierárquica de conceitos-chave e ramificações.
* **`report` (Relatório Interativo):** Documento analítico completo com seções dinâmicas.
* **`infographic` (Infográfico):** Síntese visual de métricas e relações.
* **`quiz` / `flashcards`:** Material pedagógico para fixação rápida de conceitos.

### 5.2 Monitoramento e Download
1. **Verificar progresso:** `studio_status({"notebook_id": "...", "artifact_id": "..."})`
2. **Baixar arquivo:** `download_artifact({"notebook_id": "...", "artifact_id": "...", "output_path": "Z:\\souls_engine\\docs\\artifacts\\podcast.mp3"})`
3. **Baixar todos:** `download_all_artifacts({"notebook_id": "...", "download_dir": "Z:\\souls_engine\\.souls_data\\artifacts\\"})`

---

## 🚀 6. PESQUISA AUTÔNOMA EM SEGUNDO PLANO

Permite delegar ao mecanismo interno do NotebookLM a exploração de novas fontes na web:
1. **Iniciar Pesquisa:**
   ```json
   research_start({
     "notebook_id": "souls-arch",
     "query": "Rust windows-service Session 0 NVML C-FFI bypass"
   })
   ```
2. **Monitorar e Importar Fontes Encontradas:**
   * Acompanhar status via `research_status`.
   * Importar automaticamente os melhores achados via `research_import`.

---

## 🛡️ 7. BOAS PRÁTICAS & GUARDRAILS DO SOULS ENGINE

1. **FinOps de Contexto:** Sempre prefira `notebook_query` sobre um caderno existente em vez de carregar arquivos de texto brutos de centenas de kilobytes na janela de contexto local do agente.
2. **Confirmação Explícita (`confirm=true`):** Criações pesadas de áudio e vídeo demandam cotas do Google; valide a necessidade antes de disparar gerações no Studio.
3. **Segurança de PII:** Nunca submeta documentos contendo dados pessoais de saúde ou finanças sem prévia sanitização local pelo `obsidian-privacy-shield`.

