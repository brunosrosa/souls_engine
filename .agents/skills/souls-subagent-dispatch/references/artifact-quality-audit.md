# Verificação de Qualidade de Artefatos Compilados

Use após qualquer lote de curadoria/síntese, antes de declarar "pronto". Aplica-se a notas `wiki/`, dossiês, relatórios e qualquer artefato gerado por modelo a partir de fontes.

## Princípio
Auditoria **auto-referente não é evidência**: um juiz da mesma família e dos mesmos prompts que geraram o artefato compartilha os mesmos vieses de preferência. Separe **FORMA** (determinística, FACTO) de **CONTEÚDO** (juízo, HIPÓTESE). Nunca apresentar nota de juiz como facto.

## Camadas (baratas -> caras; custo zero em créditos pagos quando locais/assinatura)

| # | Camada | O que prova | Como |
| :--- | :--- | :--- | :--- |
| 0 | Rotulação epistêmica | Não confundir forma com conteúdo | FACTO = fontes, frontmatter, links, contagens. HIPÓTESE = densidade/fidelidade. |
| 1 | Juiz independente | Quebra o viés de preferência | Auditar com modelo de **família diferente** do gerador, rubrica **cega**. |
| 2 | **Ancoragem na fonte** | O que realmente falta | Extrair claims verificáveis e buscá-los no bundle -> `% de claims ancorados`. |
| 3 | Juiz adversarial | Remove o viés de cortesia | Inverter o pedido: "tente provar que este artefato é inútil — o que falta, o que foi inventado". |
| 4 | Testar o verificador | Evita falsos positivos do gate | Rodar cada gate contra **1 artefato bom + 1 com defeito real** antes de valer (RED -> GREEN). |
| 5 | Cristalização governada | Fecha o loop com aprovação humana | `harness_ralph_crystallize` -> **Quarentena** -> aprovação no Painel; `harness_tombstone_resolve` para contradições; exigir `pins_verified == true`. |
| 6 | Métrica de resultado | Substitui nota subjetiva | Amostrar N perguntas reais e medir quantas são respondidas só com o artefato compilado. |

## Ancoragem: normalizar antes de acusar

Antes de declarar um claim "não ancorado", normalize `,` <-> `.` decimal, separador de milhar, espaço entre número e unidade (`6GB` <-> `6 GB`), prefixo `v` (`v0.3` <-> `0.3`) e extensões duplicadas. Gere variantes conservadoras (comprimento >= 3) e **confira literalmente** o resultado. Critério de descarte de um detector mede **precisão**, não ruído isolado.

## Calibração RED -> GREEN do verificador (executar antes de qualquer lote)

1. **Monte um dataset de rótulo conhecido** com trechos **REAIS** dos artefatos: falsos positivos históricos (`PASSA`), 1 artefato bom (`PASSA`) e defeitos por **mutação em memória** (`FALHA`: frontmatter ausente, placeholder genuíno, wikilink quebrado, heading duplicado).
2. **Prove o RED**: rode as versões antigas/quebradas dos gates e confirme que acusam nos casos de FP histórico.
3. **Pontue por gate**: `FP` = acusou em `PASSA`; `FN` = não acusou em `FALHA`.
4. **Descarte** todo gate com `FP > 0` ou `FN > 0`.
5. **Zere os warnings** (`python -W error::SyntaxWarning`).
6. **Feche com integridade**: snapshot de hash dos arquivos de governança e confirmação de zero mutações no cofre.

### Pitfalls da calibração

- **Regex de placeholder case-sensitive**: `\bTODO\b` sem `re.I` para não casar `Todo`/`Todos`.
- **Gate de seção condicional ao marcador**: retorne cedo quando o marcador da seção não existir no artefato.
- **Defeitos por mutação em memória**, nunca gravados no cofre.
- **Preservar artefato reprovado em scratch** antes de descartar/corrigir para permitir diagnóstico.
- **Ao editar `c1`, meça o gate no meio**: `fail_closed_triggered` deve disparar com a edição e voltar a `false` após a re-assinatura (`expected == actual == sha256_lf(disk)` e `pins_verified == true`).
