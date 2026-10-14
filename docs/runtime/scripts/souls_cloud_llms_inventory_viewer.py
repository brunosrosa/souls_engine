#!/usr/bin/env python3
"""
SOULS Engine v7 — Cloud LLMs Inventory & Pareto Frontier Dossier
==============================================================
Extrator, analisador e visualizador SSOT da Cloud Arena a partir do banco
relacional `souls_heuristic_vault.db`.

Governança:
    - RFC-005 & ADR-007: Avaliação Multiobjetivo de Modelos de Nuvem
    - 5 Perfis Cognitivos Canônicos:
        * Coding / SWE
        * Agentic / Tools
        * Reasoning / Math
        * Fast / Triage
        * Design / Frontend
    - Algoritmo de Fronteira de Pareto:
        * 🏆 Quality Leader (Campeão de Qualidade Absoluta)
        * 💎 Best Value (Campeão de Custo-Benefício / ROI Index)
        * ⚡ Ultra Low Cost (Campeão de Custo Mínimo Viável)
        * 🚫 Dominated (Modelos fora da fronteira)
    - Persistência Relacional SSOT em:
        * `canonical_models`
        * `model_aliases`
        * `pricing_telemetry_snapshots`
        * `benchmark_snapshots`
        * `daily_evaluations`

Uso:
    python souls_cloud_llms_inventory_viewer.py
    python souls_cloud_llms_inventory_viewer.py --profile coding
    python souls_cloud_llms_inventory_viewer.py --output caminho/relatorio.md
    python souls_cloud_llms_inventory_viewer.py --json
"""

import os
import sys
import json
import sqlite3
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime

# Caminhos Canônicos Padrão
DEFAULT_DB_PATH = Path(r"Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db")
DEFAULT_OUTPUT_MD = Path(r"Z:\souls_engine\docs\observability\reports\souls_cloud_llms_inventory_summary.md")

PROFILE_METADATA = {
    "coding": {
        "title": "Coding / SWE (Engenharia de Software & Rust AST)",
        "icon": "💻",
        "description": "Refatorações complexas, geração de código Rust idiomático, resolução de erros do compilador e cirurgia de Myers Diff.",
        "primary_metric": "Coding ELO & IFEval",
        "weight_formula": "S_coding = 0.50·CodingELO + 0.30·IFEval + 0.20·BFCL",
    },
    "agentic": {
        "title": "Agentic / Tools (Despacho de Ferramentas & MCP)",
        "icon": "🤖",
        "description": "Orquestração de subagentes autônomos, aderência estrita a JSON Schemas e despacho de chamadas de ferramentas MCP.",
        "primary_metric": "BFCL (Berkeley Function Calling) & IFEval",
        "weight_formula": "S_agentic = 0.55·BFCL + 0.25·IFEval + 0.20·TTFT_inv",
    },
    "reasoning": {
        "title": "Reasoning / Math (Raciocínio Lógico & Provas Formais)",
        "icon": "🧠",
        "description": "Cadeias densas de pensamento (CoT), análise matemática, provas de invariantes de concorrência e dedução de alta entropia.",
        "primary_metric": "Hard Prompts ELO & GPQA",
        "weight_formula": "S_reasoning = 0.50·HardELO + 0.35·GPQA + 0.15·GeneralELO",
    },
    "fast": {
        "title": "Fast / Triage (Triagem de Alta Cadência & Streaming)",
        "icon": "⚡",
        "description": "Sanitização de contexto, triagem de intenção preliminar, ETL de logs e streaming instantâneo de turnos rápidos.",
        "primary_metric": "Throughput (TPS) & TTFT (ms)",
        "weight_formula": "S_fast = 0.50·TPS_norm + 0.30·TTFT_norm + 0.20·Price_inv",
    },
    "design": {
        "title": "Design / Frontend (Multimodal, UI & Coordenadas)",
        "icon": "🎨",
        "description": "Compreensão de capturas de tela, layouting CSS Vanilla, coordenadas visuais e inspeção multimodal de interfaces.",
        "primary_metric": "VQA Multimodal & General ELO",
        "weight_formula": "S_design = 0.60·VQAScore + 0.25·GeneralELO + 0.15·IFEval",
    },
}

def resolve_db_connection(custom_path: Optional[str] = None) -> sqlite3.Connection:
    """Abre conexão somente-leitura com o banco Heuristic Vault."""
    p = Path(custom_path) if custom_path else DEFAULT_DB_PATH
    if not p.exists():
        # Fallback para .souls_data relativo
        alt = Path(".souls_data/db/souls_heuristic_vault.db")
        if alt.exists():
            p = alt
        else:
            raise FileNotFoundError(f"Banco de dados Heuristic Vault não localizado em: {p}")
    
    conn = sqlite3.connect(f"file:{p.resolve()}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    return conn

def load_cloud_inventory(conn: sqlite3.Connection, target_date: Optional[str] = None) -> Dict[str, Any]:
    """Extrai todos os dados compilados da Cloud Arena do banco de dados."""
    cur = conn.cursor()

    # 1. Obter data mais recente se não especificada
    if not target_date:
        cur.execute("SELECT MAX(evaluation_date) FROM daily_evaluations")
        row = cur.fetchone()
        target_date = row[0] if row and row[0] else datetime.utcnow().strftime("%Y-%m-%d")

    # 2. Carregar Modelos Canônicos
    cur.execute("""
        SELECT id, display_name, family, developer, context_window,
               supports_tools, supports_structured_outputs, supports_caching,
               is_multimodal, created_at
        FROM canonical_models
        ORDER BY developer, display_name
    """)
    canonical_models = [dict(r) for r in cur.fetchall()]
    canonical_map = {m["id"]: m for m in canonical_models}

    # 3. Carregar Aliases e Mapeamentos por Provedor
    cur.execute("""
        SELECT canonical_id, alias, source
        FROM model_aliases
        ORDER BY canonical_id, source
    """)
    aliases_by_model: Dict[str, List[Dict[str, Any]]] = {}
    for r in cur.fetchall():
        cid = r["canonical_id"]
        aliases_by_model.setdefault(cid, []).append(dict(r))

    # 4. Carregar Snapshots de Preços e Telemetria mais recentes
    cur.execute("""
        SELECT canonical_id, source, prompt_price_per_m, completion_price_per_m,
               cache_read_price_per_m, cache_write_price_per_m, ttft_ms, throughput_tps, recorded_at
        FROM pricing_telemetry_snapshots
        WHERE recorded_at = (SELECT MAX(recorded_at) FROM pricing_telemetry_snapshots)
        ORDER BY canonical_id, prompt_price_per_m ASC
    """)
    pricing_by_model: Dict[str, List[Dict[str, Any]]] = {}
    for r in cur.fetchall():
        cid = r["canonical_id"]
        pricing_by_model.setdefault(cid, []).append(dict(r))

    # 5. Carregar Snapshots de Benchmarks mais recentes
    cur.execute("""
        SELECT canonical_id, general_elo, coding_elo, hard_prompts_elo,
               bfcl_score, ifeval_score, gpqa_score, vqa_multimodal_score, recorded_at
        FROM benchmark_snapshots
        WHERE recorded_at = (SELECT MAX(recorded_at) FROM benchmark_snapshots)
    """)
    benchmarks_by_model = {r["canonical_id"]: dict(r) for r in cur.fetchall()}

    # 6. Carregar Avaliações de Pareto do Dia
    cur.execute("""
        SELECT canonical_id, profile_task, composite_score, task_blended_cost,
               roi_index, is_pareto_optimal, pareto_tag, evaluation_date
        FROM daily_evaluations
        WHERE evaluation_date = ?
        ORDER BY profile_task, composite_score DESC
    """, (target_date,))
    evaluations_raw = [dict(r) for r in cur.fetchall()]

    evaluations_by_profile: Dict[str, List[Dict[str, Any]]] = {}
    for ev in evaluations_raw:
        prof = ev["profile_task"]
        cid = ev["canonical_id"]
        c_meta = canonical_map.get(cid, {})
        prices = pricing_by_model.get(cid, [{}])[0]
        bench = benchmarks_by_model.get(cid, {})
        
        merged = {
            **ev,
            "display_name": c_meta.get("display_name", cid),
            "developer": c_meta.get("developer", "Unknown"),
            "context_window": c_meta.get("context_window", 0),
            "supports_tools": bool(c_meta.get("supports_tools", False)),
            "is_multimodal": bool(c_meta.get("is_multimodal", False)),
            "supports_caching": bool(c_meta.get("supports_caching", False)),
            "provider": prices.get("source", "nous_portal"),
            "prompt_price": prices.get("prompt_price_per_m", 0.0),
            "completion_price": prices.get("completion_price_per_m", 0.0),
            "cache_read_price": prices.get("cache_read_price_per_m", 0.0),
            "throughput_tps": prices.get("throughput_tps", 0.0),
            "ttft_ms": prices.get("ttft_ms", 0.0),
            "coding_elo": bench.get("coding_elo", 0.0),
            "bfcl_score": bench.get("bfcl_score", 0.0),
        }
        evaluations_by_profile.setdefault(prof, []).append(merged)

    return {
        "evaluation_date": target_date,
        "canonical_models": canonical_models,
        "aliases_by_model": aliases_by_model,
        "pricing_by_model": pricing_by_model,
        "benchmarks_by_model": benchmarks_by_model,
        "evaluations_by_profile": evaluations_by_profile,
    }

def generate_markdown_report(data: Dict[str, Any], filter_profile: Optional[str] = None) -> str:
    """Gera o dossiê Markdown canônico completo da Cloud Arena."""
    lines = []
    eval_date = data["evaluation_date"]
    canon_list = data["canonical_models"]
    evals = data["evaluations_by_profile"]
    pricing_map = data["pricing_by_model"]
    bench_map = data["benchmarks_by_model"]

    # CABEÇALHO
    lines.append(f"# Dossiê Analítico SSOT: Cloud LLMs & Fronteira de Pareto")
    lines.append("")
    lines.append(f"> **Data de Avaliação:** {eval_date} (UTC)  ")
    lines.append(f"> **Fonte Relacional SSOT:** `Z:\\souls_engine\\.souls_data\\db\\souls_heuristic_vault.db`  ")
    lines.append(f"> **Modelos Canônicos Homologados:** {len(canon_list)} modelos em nuvem  ")
    lines.append(f"> **Provedores Monitorados:** Nous Research Portal & OpenRouter  ")
    lines.append(f"> **Governança:** RFC-005, ADR-007 e LLM Cloud Arena Specification")
    lines.append("")
    lines.append("---")
    lines.append("")

    # RESUMO EXECUTIVO DA FRONTEIRA DE PARETO
    lines.append("## 🏆 1. RESUMO EXECUTIVO: CAMPEÕES DA FRONTEIRA DE PARETO")
    lines.append("Matriz consolidada dos modelos dominantes em cada um dos 5 perfis cognitivos canônicos:")
    lines.append("")
    lines.append("| Perfil Cognitivo | 🏆 Quality Leader | 💎 Best Value (Maior ROI) | ⚡ Ultra Low Cost | Custo Ref. ($/Tarefa) |")
    lines.append("| :--- | :--- | :--- | :--- | :---: |")

    ordered_profiles = ["coding", "agentic", "reasoning", "fast", "design"]
    for p in ordered_profiles:
        p_list = evals.get(p, [])
        if not p_list:
            continue
        p_info = PROFILE_METADATA.get(p, {})
        leader = next((m for m in p_list if m.get("pareto_tag") == "Leader"), None)
        value = next((m for m in p_list if m.get("pareto_tag") == "BestValue"), None)
        lowcost = next((m for m in p_list if m.get("pareto_tag") == "UltraLowCost"), None)

        leader_str = f"**{leader['display_name']}** ({leader['composite_score']:.1f})" if leader else "—"
        value_str = f"**{value['display_name']}** (ROI: {value['roi_index']:.1f})" if value else "—"
        lowcost_str = f"`{lowcost['display_name']}` (${lowcost['task_blended_cost']:.4f})" if lowcost else "—"
        ref_cost = f"${value['task_blended_cost']:.4f}" if value else (f"${leader['task_blended_cost']:.4f}" if leader else "—")

        lines.append(f"| {p_info.get('icon', '')} **{p_info.get('title', p)}** | {leader_str} | {value_str} | {lowcost_str} | {ref_cost} |")

    lines.append("")
    lines.append("---")
    lines.append("")

    # SEÇÃO DETALHADA POR PERFIL
    lines.append("## 📊 2. ANÁLISE DETALHADA POR PERFIL COGNITIVO")
    lines.append("Modelos ranqueados pela pontuação composta multiobjetivo, métricas de custo e classificação de Pareto:")
    lines.append("")

    for p in ordered_profiles:
        if filter_profile and filter_profile.lower() != p:
            continue

        p_list = evals.get(p, [])
        if not p_list:
            continue

        p_info = PROFILE_METADATA.get(p, {})
        lines.append(f"### {p_info.get('icon', '🎯')} 2.{ordered_profiles.index(p)+1} Perfil: {p_info.get('title', p)}")
        lines.append(f"> **Descrição:** {p_info.get('description', '')}  ")
        lines.append(f"> **Métricas Primárias:** {p_info.get('primary_metric', '')} &bull; **Fórmula:** `{p_info.get('weight_formula', '')}`")
        lines.append("")
        lines.append("| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |")
        lines.append("| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |")

        for idx, m in enumerate(p_list, start=1):
            tag = m.get("pareto_tag", "Dominated")
            if tag == "Leader":
                tag_badge = "🏆 **Quality Leader**"
            elif tag == "BestValue":
                tag_badge = "💎 **Best Value**"
            elif tag == "UltraLowCost":
                tag_badge = "⚡ **Ultra Low Cost**"
            else:
                tag_badge = "🚫 Dominated"

            ctx_str = f"{m['context_window']//1000}k" if m['context_window'] else "N/A"
            tps_str = f"{m['throughput_tps']:.0f} tps" if m['throughput_tps'] else "—"

            lines.append(
                f"| {idx} | **{m['display_name']}**<br>`{m['canonical_id']}` | {m['provider']} | "
                f"${m['task_blended_cost']:.4f} | **{m['composite_score']:.1f}** | {m['roi_index']:.1f} | "
                f"{tps_str} | {ctx_str} | {tag_badge} |"
            )

        # Recomendações e Fallbacks
        leader = next((m for m in p_list if m.get("pareto_tag") == "Leader"), p_list[0])
        val_models = [m for m in p_list if m.get("pareto_tag") == "BestValue"]
        fb1 = val_models[0] if len(val_models) > 0 else p_list[1]
        fb2 = val_models[1] if len(val_models) > 1 else (p_list[2] if len(p_list) > 2 else p_list[0])

        lines.append("")
        lines.append(f"#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`{p}`)")
        lines.append("| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |")
        lines.append("| :--- | :--- | :--- | :---: | :--- |")
        lines.append(f"| **1. Primário (Custo-Benefício)** | `{fb1['display_name']}` | {fb1['provider']} | ${fb1['task_blended_cost']:.4f} | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |")
        lines.append(f"| **2. Escalação Máxima (Líder)** | `{leader['display_name']}` | {leader['provider']} | ${leader['task_blended_cost']:.4f} | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |")
        lines.append(f"| **3. Fallback de Baixo Custo** | `{fb2['display_name']}` | {fb2['provider']} | ${fb2['task_blended_cost']:.4f} | Standby caso o provedor principal sofra throttling ou indisponibilidade. |")
        lines.append("")
        lines.append("---")
        lines.append("")

    # SEÇÃO 3: CATÁLOGO DE MODELOS CANÔNICOS & RECURSOS
    lines.append("## 📦 3. CATÁLOGO CANÔNICO DE MODELOS DE NUVEM & RECURSOS")
    lines.append("Especificações arquiteturais e suporte a recursos avançados de cada modelo catalogado:")
    lines.append("")
    lines.append("| Identificador Canônico | Nome Comercial | Desenvolvedor | Janela de Contexto | Tool Calling | Prefix Caching | Multimodal | Structured Outputs |")
    lines.append("| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |")

    for m in canon_list:
        tools_str = "✅ Sim" if m["supports_tools"] else "❌ Não"
        cache_str = "✅ Sim" if m["supports_caching"] else "❌ Não"
        multi_str = "👁️ Sim" if m["is_multimodal"] else "❌ Não"
        struct_str = "✅ Sim" if m["supports_structured_outputs"] else "❌ Não"
        ctx_str = f"{m['context_window']:,} tokens"

        lines.append(
            f"| `{m['id']}` | **{m['display_name']}** | {m['developer']} | {ctx_str} | "
            f"{tools_str} | {cache_str} | {multi_str} | {struct_str} |"
        )

    lines.append("")
    lines.append("---")
    lines.append("")

    # SEÇÃO 4: MATRIZ COMERCIAL DE PREÇOS POR PROVEDOR
    lines.append("## 💸 4. MATRIZ COMERCIAL DE PREÇOS & TELEMETRIA POR PROVEDOR")
    lines.append("Preços de mercado reais levantados via APIs (Input, Output, Cache Read e latências TTFT):")
    lines.append("")
    lines.append("| Modelo Canônico | Provedor / Rota | Input $/1M | Output $/1M | Cache Read $/1M | TTFT Médio | Throughput Médio |")
    lines.append("| :--- | :--- | :---: | :---: | :---: | :---: | :---: |")

    for cid, plist in pricing_map.items():
        c_meta = next((c for c in canon_list if c["id"] == cid), None)
        c_name = c_meta["display_name"] if c_meta else cid
        for p in plist:
            cache_read = f"${p['cache_read_price_per_m']:.4f}" if p.get('cache_read_price_per_m') else "N/A"
            ttft = f"{p['ttft_ms']:.0f} ms" if p.get('ttft_ms') else "N/A"
            tps = f"{p['throughput_tps']:.1f} tok/s" if p.get('throughput_tps') else "N/A"

            lines.append(
                f"| **{c_name}** (`{cid}`) | `{p['source']}` | "
                f"${p['prompt_price_per_m']:.2f} | ${p['completion_price_per_m']:.2f} | "
                f"{cache_read} | {ttft} | {tps} |"
            )

    lines.append("")
    lines.append("---")
    lines.append("")

    # SEÇÃO 5: BENCHMARKS EMPÍRICOS
    lines.append("## 🎯 5. BENCHMARKS EMPÍRICOS E MÉTRICAS DE ENGENHARIA")
    lines.append("Resultados empíricos normalizados utilizados no cômputo da Fronteira de Pareto:")
    lines.append("")
    lines.append("| Modelo Canônico | General ELO | Coding ELO | Hard ELO | BFCL (Tools) | IFEval | GPQA | VQA Score |")
    lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")

    for cid, b in bench_map.items():
        c_meta = next((c for c in canon_list if c["id"] == cid), None)
        c_name = c_meta["display_name"] if c_meta else cid

        lines.append(
            f"| **{c_name}** | {b.get('general_elo', 0):.0f} | **{b.get('coding_elo', 0):.0f}** | "
            f"{b.get('hard_prompts_elo', 0):.0f} | {b.get('bfcl_score', 0):.1f}% | "
            f"{b.get('ifeval_score', 0):.1f}% | {b.get('gpqa_score', 0):.1f}% | "
            f"{b.get('vqa_multimodal_score', 0):.1f}% |"
        )

    lines.append("")
    lines.append("---")
    lines.append("")

    # SEÇÃO 6: DIRETRIZES DE FINOPS & SIMBIOSE LOCAL/NUVEM
    lines.append("## 🛡️ 6. DIRETRIZES DE FINOPS & MATRIZ DE DECISÃO LOCAL VS CLOUD")
    lines.append("""
1. **Regra de Ouro do Silício Local:** Tarefas de desidratação AST, triagem de entropia, inspeção de tipos e refatorações cirúrgicas de arquivo único pertencem **100% à dGPU RTX 2060m e CPU Host i9** (custo zero de nuvem).
2. **Critérios de Escalação para Nuvem:**
   - **Alta Entropia Cognitiva ($H > 0.85$):** Questões abertas de arquitetura de software demandam `Claude 3.7 Sonnet` ou `DeepSeek R1`.
   - **Estouro de Raio de Impacto (Blast Radius > 5 arquivos):** Refatorações cross-crate devem delegar a síntese para a Nuvem.
   - **Janela Gigante (> 32k tokens):** Contextos muito extensos devem utilizar o prefix caching do `DeepSeek V3` ou `Gemini 2.0 Flash`.
3. **Prefix Caching Obrigatório:** Sempre priorize provedores com suporte a cache read com desconto $\ge 75\%$ em sessões multi-turn longas.
""")

    return "\n".join(lines)

def print_console_summary(data: Dict[str, Any]):
    """Imprime resumo visual colorido e formatado no console."""
    print("=" * 80)
    print("SOULS ENGINE v7 — CLOUD LLMs & PARETO FRONTIER DOSSIER")
    print(f"Data de Avaliação: {data['evaluation_date']} UTC | Modelos Canônicos: {len(data['canonical_models'])}")
    print("=" * 80)

    for p in ["coding", "agentic", "reasoning", "fast", "design"]:
        p_list = data["evaluations_by_profile"].get(p, [])
        p_info = PROFILE_METADATA.get(p, {})
        print(f"\n{p_info.get('icon', '🎯')} PERFIL: {p_info.get('title', p).upper()}")
        print("-" * 80)
        print(f"{'Modelo':<26} | {'Provedor':<12} | {'Score':<6} | {'$/Tarefa':<10} | {'ROI':<6} | {'Status Pareto'}")
        print("-" * 80)

        for m in p_list:
            tag = m.get("pareto_tag", "Dominated")
            if tag == "Leader":
                tag_str = "🏆 Leader"
            elif tag == "BestValue":
                tag_str = "💎 BestValue"
            elif tag == "UltraLowCost":
                tag_str = "⚡ LowCost"
            else:
                tag_str = "  Dominated"

            print(f"{m['display_name'][:25]:<26} | {m['provider'][:11]:<12} | {m['composite_score']:<6.1f} | ${m['task_blended_cost']:<9.4f} | {m['roi_index']:<6.1f} | {tag_str}")

    print("\n" + "=" * 80)

def main():
    parser = argparse.ArgumentParser(description="Visualizador e Extrator SSOT de Modelos Cloud do Souls Engine")
    parser.add_argument("--db", type=str, help="Caminho customizado do souls_heuristic_vault.db")
    parser.add_argument("--output", "-o", type=str, default=str(DEFAULT_OUTPUT_MD), help="Caminho do arquivo Markdown de saída")
    parser.add_argument("--profile", "-p", type=str, help="Filtrar por perfil cognitivo (coding, agentic, reasoning, fast, design)")
    parser.add_argument("--date", "-d", type=str, help="Data de avaliação específica (YYYY-MM-DD)")
    parser.add_argument("--json", action="store_true", help="Exportar em formato JSON para stdout")
    parser.add_argument("--quiet", "-q", action="store_true", help="Não imprimir resumo no console")

    args = parser.parse_args()

    try:
        conn = resolve_db_connection(args.db)
    except Exception as e:
        print(f"ERRO: Não foi possível conectar ao banco Heuristic Vault: {e}", file=sys.stderr)
        sys.exit(1)

    data = load_cloud_inventory(conn, args.date)

    if args.json:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return

    if not args.quiet:
        print_console_summary(data)

    # Gera e grava o Markdown
    md_content = generate_markdown_report(data, args.profile)
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(md_content, encoding="utf-8")
    print(f"\n[OK] Dossiê Markdown salvo com sucesso em: {out_path.resolve()}")

if __name__ == "__main__":
    main()
