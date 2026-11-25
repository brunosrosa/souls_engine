---
name: souls-context-lean
description: Playbook para compressão semântica de contexto e desidratação CCR na RAM. Mede tokens com tiktoken antes da leitura e reidrata texto sob demanda poupando janela de contexto.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "context"
---

# Souls Context Lean — Playbook de Compressão e CCR

Este playbook rege a leitura inteligente de documentações e arquivos densos, aplicando compressão semântica na RAM física e preservando a janela de atenção do modelo.

## 🗜️ FLUXO OPERACIONAL

### 1. Medição Prévia com `smart_read`
Antes de ler um arquivo longo não estruturado (documentações, logs, JSONs de configuração):
* Chame `smart_read(path="...", max_tokens=2000)`.
* O motor calcula a contagem real de tokens via `tiktoken` na CPU sem alocar VRAM.
* Se o arquivo exceder o teto, ele aplica auto-shrink preservando cabeçalhos e trechos mais densos semanticamente.

### 2. Leitura Concorrente em Lote com `multi_read`
Quando for necessário ler múltiplos arquivos de documentação simultaneamente:
* Use `multi_read(paths=["docs/a.md", "docs/b.md"])`.
* O motor agrupa os arquivos na memória RAM e formata com delimitadores compactos TOON.

### 3. Compressão Semântica com `compress` e Reidratação CCR com `fill`
* `compress(text="...")`: remove ruído sintático, comentários redundantes e espaços extras, substituindo trechos repetitivos por hashes de referência CCR na RAM.
* `fill(hash="...")`: reidrata um bloco comprimido de volta para o texto lossless quando o modelo precisar do conteúdo integral exato.

### 4. Chunking Resiliente de Corpus Denso (MinionS / BM25)
Textos reais vindos do Google Docs, Gmail, PDFs ou transcrições frequentemente usam quebras simples (`\n`) em vez de linhas em branco (`\n\n`).
* **Regra de Subdivisão:** Após o particionamento inicial por parágrafos (`"\n\n"`), todo bloco com mais de `350` palavras DEVE ser subdividido por janela deslizante de `300` palavras com sobreposição (`overlap`) de `50` palavras antes do ranqueamento BM25.
* **Verificação de Sanidade:** Nunca considere um pipeline de compressão validado se `savings_pct == 0` em documentos multi-KB sem inspecionar `chunks_total`; um corpus de ~33 KB deve gerar ~15–20 chunks e economia típica de `80%+` em `<10 ms`.
