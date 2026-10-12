---
name: souls-web-intelligence
description: Playbook para pesquisa web e raspagem leve de documentações. Governa web_search e fetch_web com conversão direta de HTML para Markdown na RAM, economizando tokens de DOM.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "sys"
---

# Souls Web Intelligence — Playbook de Pesquisa Web Leve (v7)

Este playbook rege a pesquisa externa e ingestão de documentações da web sem sangria de tokens de HTML.

## 🌐 FLUXO OPERACIONAL

### 1. Pesquisa Web sem Rastreamento com `web_search`
Para consultar documentação de bibliotecas, APIs externas ou erros de compilador:
```json
web_search({
  "query": "rust windows-sys 0.59 FILE_SHARE_READ",
  "max_results": 5
})
```
* O motor extrai snippets diretos e URLs sem carregar páginas web inteiras.

### 2. Ingestão Limpa de Páginas com `fetch_web`
Quando for necessário ler a documentação completa de um link:
```json
fetch_web({
  "url": "https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SqlitePoolOptions.html"
})
```
* **Vantagem FinOps:** O conversor embutido transforma o DOM HTML em Markdown limpo na memória RAM, descartando tags `script`, `style` e menus de navegação, reduzindo até 90% do volume de tokens brutos.
