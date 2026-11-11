# ADR-028: Google Universe Bridge, Engenharia Reversa do Protocolo Wiz (NotebookLM) e Migração da Frota Jules para Antigravity CLI (agy)

## Status
**Aprovado e Implementado** (Milestone v0.8.2)

## Data
2026-11-10

## Contexto e Forças em Conflito
O ecossistema de inteligência do operador utiliza amplamente o Google NotebookLM e o workspace do Google Drive para síntese de documentos e pesquisa aprofundada (Deep Research). Contudo, três desafios arquiteturais críticos bloqueavam a autonomia dos subagentes no Souls Engine:

1. **Inexistência de API Oficial Pública para o NotebookLM:** O Google NotebookLM não oferece endpoints REST públicos padronizados para agentes externos. A comunicação depende da emulação precisa da pilha interna do Google Wiz Engine via RPCs `batchexecute`.
2. **Descontinuação do Google Jules:** O Google Jules CLI foi oficialmente descomissionado em 29/10/2026, deixando fluxos legados de automação de código órfãos.
3. **Soberania de Credenciais e Segurança Local:** Sessões Google utilizam cookies complexos (`__Secure-1PSID`, `__Secure-1PSIDTS`, `SAPISID`) e tokens CSRF (`SNlM0e`). O vazamento ou armazenamento em texto plano dessas credenciais no disco violaria as diretrizes de segurança e a Lei 1.4 do `AGENTS.md` (Zero Poluição fora do Dev Drive ReFS `Z:\`).

---

## Decisão de Arquitetura

Optou-se por uma arquitetura em 3 camadas implementada bare-metal em `souls_core::google_bridge` e exposta em `souls_server`:

### 1. Cofre Encriptado Local (`GoogleSessionVault`)
- Persistência exclusiva em `Z:\souls_engine\.souls_data\google_sessions.vault`.
- Criptografia reversível baseada em máquina com cabeçalho mágico `GV1:`, permitindo persistência segura e ofuscação de credenciais em memória e telemetria.
- Suporte a mascaramento automático (`masked()`) exibindo apenas 4 caracteres iniciais e finais para observabilidade sem vazamento de segredos.

### 2. Emulação do Protocolo Wiz `batchexecute` (`WizRpcClient`)
- Reconstrução de envelopes `f.req` com codificação RFC 3986 percent-encoding determinística e zero dependências externas:
  $$f.req = [[[\text{rpc\_id}, \text{serialized\_args}, \text{null}, \text{"generic"}]]]$$
- Injeção obrigatória dos cabeçalhos anti-CSRF `X-Same-Domain: 1` e do token `at={SNlM0e}`.
- Sanitização anti-XSS (`)]}'\n`) e parser recursivo de payloads JSON em profundidade arbitrária (`find_wrb_fr`).
- Suporte aos 22 RPCs canônicos do NotebookLM: `wXbhsf` (notas), `rLM1Ne` (leitura), `izAoDd` (ingestão Drive/PDF), `QA9ei` (Deep Research autônomo), `R7cb6c` (Audio Overview e Mindmap), `orqRsd` (consulta RAG com citações) e `s07Lfd` (listagem de cadernos).

### 3. Roteador de Migração Transparente Jules ➔ `agy` (`JulesMigrationRouter`)
- Detecção em voo de invocações da CLI descontinuada `jules`.
- Tradução determinística para a **Antigravity CLI (`agy`)** com flags de headless safety:
  `-p "{prompt}" --mode=accept-edits --output-format=stream-json --dangerously-skip-permissions`
- Dual-Layer Workspace vinculando raízes locais no Dev Drive ReFS (`Z:\souls_engine`) a diretórios do Google Drive e IDs de documentos sincronizados.

---

## Consequências

### Positivas
- **Independência de Navegador:** Subagentes disparam pesquisas profundas (Deep Research) e geram notas e podcasts no NotebookLM diretamente via TCP loopback sem abrir janelas de browser nem gastar RAM com instâncias de Electron.
- **Continuidade Operacional:** Comandos legados do Google Jules funcionam transparentemente sem quebra de pipelines legados.
- **Conformidade de Segurança:** Segredos de sessão permanecem confinados e encriptados no Dev Drive ReFS.
