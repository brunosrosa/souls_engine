#!/usr/bin/env python3
"""
SOULS Engine v7 — Silicon Observability & LLM Inventory Dossier
===============================================================
Extrator e visualizador SSOT do banco relacional `souls_heuristic_vault.db` e
do repositório unificado de silício `C:\\Users\\rosas\\llm_local_models`.

Governança:
    - ADR-004: Teto Útil de VRAM 5.2GB no Windows 11 WDDM
    - ADR-007: Desacoplamento Ontológico dos Benchmark Types (Types 1 a 5)
    - Matriz de Tiers Físicos: Tiers 0, 0.5, 1, 2, 3, 4
    - Foco Especial: Leaderboard de Prontidão e Qualificação para o Hermes Agent (Tier 1)

Uso:
    python souls_llms_inventory_viewer.py
    python souls_llms_inventory_viewer.py --tier1-only
    python souls_llms_inventory_viewer.py --output caminho/relatorio.md
    python souls_llms_inventory_viewer.py --json
"""

import os
import sys
import re
import json
import sqlite3
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime

# Constantes de Governança Bare-Metal (RTX 2060m WDDM)
SAFE_WDDM_VRAM_CEILING_MB = 5294.0
RTX_2060M_TOTAL_VRAM_MB = 6144.0

TIER_DESCRIPTIONS = {
    "Tier 0": {
        "title": "Tier 0 — Bootstrap, Sanity & Fast Heuristic Triage",
        "role": "Classificação preliminar de intenção, triagem de prompts, bootstrap ultra-rápido (<15ms) e verificações de sanidade a frio sem alocação de VRAM.",
        "specs": "CPU Host Intel Core i9 (extensões AVX2) | Modelos < 350 MB (135M parâmetros ou ONNX) | 0 MB VRAM dGPU | Meta: Throughput > 100 TPS.",
        "engines": "`onnx-cpu` (ONNX Runtime) e `llama-cpu` (`-ngl 0`).",
    },
    "Tier 0.5": {
        "title": "Tier 0.5 — Epistemic Probing & SLM Host Sensors",
        "role": "Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward-only prefill sem geração recursiva) e triagens heurísticas leves.",
        "specs": "CPU Host Intel i9 AVX2 | 0 MB VRAM dGPU (estritamente RAM do sistema) | Modelos SLM de 360M a 2B parâmetros | Latência de probe < 150 ms.",
        "engines": "`llama-cpu` (`-ngl 0 -t 8`), BitNet 1.58b, SmolLM2-360M.",
    },
    "Tier 1": {
        "title": "Tier 1 — Master Interactive Chat, Coding & Autonomous Tool-Calling",
        "role": "Cérebro executivo local e motor principal para Live Chat, geração de código Rust AST e despacho estruturado de ferramentas MCP para o Hermes Agent.",
        "specs": "dGPU NVIDIA GeForce RTX 2060m (teto seguro WDDM de 5.294 MB de VRAM) | Modelos de 2.7B a 7B (quantizações Q4_0, Q4_K_M, IQ3_M) | KV Cache Assimétrico (Key=FP16, Value=Q4_0) com FlashAttention-2 | Meta: TPS ≥ 30 tok/s e TTFT < 2.500 ms.",
        "engines": "`llama-cuda` (`-ngl 99 -ctk f16 -ctv q4_0 -fa on`).",
    },
    "Tier 2": {
        "title": "Tier 2 — Background Heavyweight & Deep Cognitive Reasoning",
        "role": "Raciocínio profundo assíncrono (deep CoT), arquiteturas Mixture-of-Experts (MoE) ou modelos de alta capacidade (>14B / 27B) que excedem a VRAM da dGPU.",
        "specs": "32GB RAM DDR4 Host via AVX2 / CPU Pinned (0 MB VRAM na GPU para evitar spillover PCIe forçado pelo Windows) | Operação assíncrona em background via delegate_task.",
        "engines": "`llama-cpu` (`-ngl 0 -t 8`), rnj-1 (8B Gemma 3), laguna-xs (16B MoE).",
    },
    "Tier 3": {
        "title": "Tier 3 — Multimodal Vision-Language Models & UI Grounding",
        "role": "Inspeção visual de interfaces, OCR técnico de diagramas e captura multimodal via projetores mmproj pareados.",
        "specs": "dGPU NVIDIA RTX 2060m com projetores acoplados via --mmproj | Modelos VLM de 2B a 7B (Qwen2-VL, Gemma-4-E2B-IT, UI-TARS).",
        "engines": "`llama-cuda` (`-ngl 99 --mmproj <sidecar_path>`).",
    },
    "Tier 4": {
        "title": "Tier 4 — Speculative Decoding Drafters & Acceleration Sparks",
        "role": "Modelos compactos (drafters) executados em conjunto com o modelo mestre para especular múltiplos tokens por passo e acelerar a geração via taxa de aceitação α.",
        "specs": "Modelos ultracompactos (300M a 1.6GB ou heads MTP) co-alocados na VRAM da dGPU ou RAM | Meta: Taxa de aceitação α ≥ 55% para speedup de 1.4x a 2.1x.",
        "engines": "`llama-cuda` (`-md <drafter.gguf>`).",
    },
}

def extract_parameters(name: str, folder: str = "") -> str:
    """Extrai a contagem canônica de parâmetros a partir do nome e pasta."""
    combined = f"{name} {folder}".lower()

    # 1. Regras explícitas de famílias canônicas
    if "phi-4-mini" in combined or "phi-4" in combined:
        return "3.8B"
    if "mamba-codestral" in combined or "codestral" in combined:
        return "7B"
    if "nemotron-3-nano" in combined:
        return "4B"
    if "zamba2-2.7b" in combined or "zamba2" in combined:
        return "2.7B"
    if "neuralai-mamba-k1" in combined or "mamba-k1" in combined:
        return "135M"
    if "laguna-xs-2.1-dspark" in combined or ("dspark" in combined and "laguna" in combined):
        return "300M"
    if "laguna-xs" in combined or "laguna" in combined:
        return "16B MoE"
    if "rnj-1" in combined:
        return "8B"
    if "bonsai-27b" in combined:
        return "27B"
    if "fara1.5-4b" in combined or "fara1.5" in combined:
        return "4B"
    if "fara-7b" in combined:
        return "7B"
    if "nimbus-4b" in combined:
        return "4B"
    if "gemma-4-e2b" in combined or "e2b" in combined:
        return "2B"
    if "smollm3-3b" in combined or "smollm3" in combined:
        return "3B"
    if "arch-function-chat-3b" in combined:
        return "3B"
    if "hy-mt2-7b" in combined:
        return "7B"
    if "ui-tars-1.5-7b" in combined or "ui-tars" in combined:
        return "7B"
    if "lfm2.5-1.2b" in combined or "lfm2-5-1-2b" in combined or "lfm2.5" in combined:
        return "1.2B"

    # Remove falsos positivos de quantização como 4bit, 8bit
    clean = re.sub(r"\b\d+bit\b", "", combined)

    # 2. Busca ordenada decrescente para evitar colisões
    for p in [
        "70b", "33b", "27b", "16b", "14b", "13b", "12b", "9b", "8b", "7b",
        "4b", "3.8b", "3.5b", "3b", "2.7b", "2b", "1.7b", "1.5b", "1.2b", "1b", "0.5b",
        "790m", "500m", "360m", "135m", "300m"
    ]:
        pattern = r"(?:^|[^0-9a-z])" + re.escape(p) + r"(?:$|[^0-9a-z])"
        if re.search(pattern, clean):
            return p.upper()

    return "Unknown"

def resolve_db_path(custom_path: Optional[str] = None) -> Path:
    """Localiza o banco de dados SSOT souls_heuristic_vault.db."""
    if custom_path:
        p = Path(custom_path)
        if p.exists():
            return p.resolve()
        print(f"[!] Caminho customizado não encontrado: {custom_path}", file=sys.stderr)

    candidate_paths = [
        Path("Z:/souls_engine/.souls_data/db/souls_heuristic_vault.db"),
        Path(__file__).resolve().parent.parent.parent / ".souls_data" / "db" / "souls_heuristic_vault.db",
        Path.cwd() / ".souls_data" / "db" / "souls_heuristic_vault.db",
    ]

    for p in candidate_paths:
        if p.exists():
            return p.resolve()

    print(f"[!] ERRO BARE-METAL: Banco SQLite não encontrado nas rotas canônicas: {candidate_paths[0]}", file=sys.stderr)
    sys.exit(1)

def resolve_canonical_models_dir() -> Path:
    """Localiza o repositório unificado de modelos locais."""
    user_profile = os.environ.get("USERPROFILE", "C:/Users/rosas")
    p = Path(user_profile) / "llm_local_models"
    return p

def format_bytes(bytes_val: Optional[int]) -> str:
    if not bytes_val or bytes_val <= 0:
        return "0 MB"
    gb = bytes_val / (1024 ** 3)
    if gb >= 1.0:
        return f"{gb:.2f} GB"
    mb = bytes_val / (1024 ** 2)
    return f"{mb:.1f} MB"

def format_tokens(num: int) -> str:
    if num >= 1_000_000:
        return f"{num / 1_000_000:.2f}M"
    if num >= 1_000:
        return f"{num / 1_000:.1f}k"
    return str(num)

def check_table_exists(conn: sqlite3.Connection, table_name: str) -> bool:
    cursor = conn.cursor()
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name=?", (table_name,))
    return cursor.fetchone() is not None

def calculate_e3_score(accuracy_pct: float, ttft_ms: float, tpot_ms: float, fallback_latency_ms: float = 0.0) -> float:
    """
    Score de Eficiência FinOps (E3 Score):
    E³ = (Acurácia²) / (Latência Média em segundos + 0.001)
    onde Acurácia ∈ [0.0, 1.0].
    """
    acc_ratio = max(0.0, min(1.0, accuracy_pct / 100.0))
    if ttft_ms > 0 or tpot_ms > 0:
        latency_sec = (ttft_ms + tpot_ms) / 1000.0
    elif fallback_latency_ms > 0:
        latency_sec = fallback_latency_ms / 1000.0
    else:
        latency_sec = 0.0

    e3 = (acc_ratio ** 2) / (latency_sec + 0.001)
    return round(e3, 4)

def load_local_manifests(models_dir: Path) -> Dict[str, Dict[str, Any]]:
    """Carrega os arquivos model_manifest.json gerados no repositório de modelos."""
    manifests = {}
    if not models_dir.exists():
        return manifests

    for manifest_path in models_dir.rglob("model_manifest.json"):
        try:
            with open(manifest_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                target_fn = data.get("target_filename")
                if target_fn:
                    manifests[target_fn.lower()] = data
                m_id = data.get("model_id")
                if m_id:
                    manifests[m_id.lower()] = data
                folder = manifest_path.parent.name.lower()
                manifests[folder] = data
                manifests[manifest_path.parent.resolve().as_posix().lower()] = data
                target_path = data.get("target_path")
                if target_path:
                    manifests[Path(target_path).resolve().as_posix().lower()] = data
        except Exception:
            pass

    return manifests

def fetch_inventory_data(conn: sqlite3.Connection) -> List[Dict[str, Any]]:
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    table_name = "model_registry" if check_table_exists(conn, "model_registry") else "local_models"
    has_telemetry = check_table_exists(conn, "arena_telemetry")

    cursor.execute(f"PRAGMA table_info({table_name})")
    cols = {row["name"] for row in cursor.fetchall()}

    def col_or_default(name: str, def_val: str) -> str:
        return f"mr.{name}" if name in cols else f"{def_val} as {name}"

    c_model_id = col_or_default("model_id", "mr.model_name")
    c_tier = col_or_default("tier_classified", "'Tier 1'")
    c_tps = col_or_default("tps", "0.0")
    c_engine = col_or_default("engine_selected", "COALESCE(mr.engine_type, 'llama.cpp')")
    c_last_epoch = col_or_default("last_benchmarked_epoch", "0")

    c_ttft = col_or_default("ttft_ms", "0.0")
    c_tpot = col_or_default("tpot_ms", "0.0")
    c_vram = col_or_default("vram_peak_mb", "0.0")
    c_e3 = col_or_default("e3_score", "0.0")
    c_json = col_or_default("score_json_tools", "0.0")
    c_code = col_or_default("score_code_ast", "0.0")
    c_reason = col_or_default("score_reasoning", "0.0")
    c_vision = col_or_default("score_vision_vqa", "0.0")
    c_has_mmproj = col_or_default("has_mmproj_sidecar", "0")
    c_mmproj_path = col_or_default("mmproj_file_path", "NULL")
    c_mtp_rate = col_or_default("mtp_acceptance_rate", "0.0")
    c_deact_at = col_or_default("deactivated_at", "0")
    c_deact_reason = col_or_default("deactivation_reason", "NULL")

    if has_telemetry:
        query = f"""
        SELECT 
            mr.file_path,
            mr.model_name,
            {c_model_id},
            {c_tier},
            mr.parameters,
            mr.context_length,
            mr.quantization,
            mr.file_size_bytes,
            mr.is_active,
            {c_engine},
            {c_ttft},
            {c_tpot},
            {c_tps},
            {c_vram},
            {c_e3},
            {c_json},
            {c_code},
            {c_reason},
            {c_vision},
            {c_has_mmproj},
            {c_mmproj_path},
            {c_mtp_rate},
            {c_deact_at},
            {c_deact_reason},
            {c_last_epoch},
            COUNT(at.id) as telemetry_count,
            MAX(CASE WHEN at.benchmark_type = 'HardwareProfile' THEN at.ttft_ms END) as dyn_ttft_ms,
            MAX(CASE WHEN at.benchmark_type = 'HardwareProfile' THEN at.tpot_ms END) as dyn_tpot_ms,
            MAX(CASE WHEN at.benchmark_type = 'HardwareProfile' THEN at.tps END) as dyn_tps,
            MAX(CASE WHEN at.benchmark_type = 'HardwareProfile' THEN at.vram_peak_mb END) as dyn_vram_peak_mb,
            MAX(CASE WHEN at.benchmark_type = 'HardwareProfile' THEN at.e3_score END) as dyn_e3_score
        FROM {table_name} mr
        LEFT JOIN arena_telemetry at ON (mr.file_path = at.file_path OR mr.model_id = at.model_id)
        GROUP BY mr.file_path
        """
    else:
        query = f"""
        SELECT 
            mr.file_path,
            mr.model_name,
            {c_model_id},
            {c_tier},
            mr.parameters,
            mr.context_length,
            mr.quantization,
            mr.file_size_bytes,
            mr.is_active,
            {c_engine},
            {c_ttft},
            {c_tpot},
            {c_tps},
            {c_vram},
            {c_e3},
            {c_json},
            {c_code},
            {c_reason},
            {c_vision},
            {c_has_mmproj},
            {c_mmproj_path},
            {c_mtp_rate},
            {c_deact_at},
            {c_deact_reason},
            {c_last_epoch},
            0 as telemetry_count,
            0.0 as dyn_ttft_ms,
            0.0 as dyn_tpot_ms,
            0.0 as dyn_tps,
            0.0 as dyn_vram_peak_mb,
            0.0 as dyn_e3_score
        FROM {table_name} mr
        """

    cursor.execute(query)
    rows = [dict(r) for r in cursor.fetchall()]
    return rows

def evaluate_hermes_readiness(model: Dict[str, Any]) -> Tuple[str, str]:
    """
    Avalia a prontidão do modelo para atuar como cérebro ou executor do Hermes Agent.
    Critérios:
        - Teto VRAM: VRAM pico < 5.294 MB (garantindo headroom de contexto)
        - Throughput TPS: >= 30 tok/s para chat interativo fluido
        - Código & Tools: Acurácia em Rust AST e JSON MCP
    """
    vram = model.get("vram_peak_mb", 0.0) or 0.0
    tps = model.get("tps", 0.0) or 0.0
    code_score = model.get("score_code_ast", 0.0) or 0.0
    json_score = model.get("score_json_tools", 0.0) or 0.0
    has_benchmark = (model.get("telemetry_count", 0) > 0) or (tps > 0.0)

    if not has_benchmark:
        return "🟡 AGUARDANDO BENCHMARK", "Aguardando execução da bateria Type 1 & Type 2 na Arena"

    if vram > SAFE_WDDM_VRAM_CEILING_MB:
        return "🔴 RISCO DE SPILLOVER WDDM", f"VRAM de {vram:.0f}MB excede o teto seguro de 5.2GB"

    if tps >= 35.0 and (code_score >= 0.85 or json_score >= 0.85 or not (code_score > 0 or json_score > 0)):
        return "🟢 CANDIDATO PRINCIPAL HERMES", f"Alta cadência ({tps:.1f} tok/s) com VRAM estável ({vram:.0f}MB)"
    elif tps >= 20.0:
        return "🔵 SUBAGENTE DE SUPORTE", f"Adequado para tarefas assíncronas via delegate_task ({tps:.1f} tok/s)"
    else:
        return "🟠 CADÊNCIA LENTA", f"Throughput de {tps:.1f} tok/s abaixo do ideal interativo"

def generate_inventory_report(
    custom_db: Optional[str] = None,
    output_file: Optional[str] = None,
    tier1_only: bool = False,
    include_ghosts: bool = False,
) -> str:
    db_path = resolve_db_path(custom_db)
    models_dir = resolve_canonical_models_dir()
    repo_root = db_path.parent.parent.parent

    print(f"[+] Conectando ao Banco SSOT em: {db_path}")
    print(f"[+] Cruzando com Repositório Unificado em: {models_dir}")

    conn = sqlite3.connect(db_path)
    all_rows = fetch_inventory_data(conn)
    
    # Busca telemetrias da arena (Type 3, Type 4 e Type 5)
    cursor = conn.cursor()
    cursor.execute("SELECT model_id, e3_score, status, created_at FROM arena_telemetry WHERE benchmark_type = 'ContextPressure' ORDER BY id DESC")
    pressure_raw = cursor.fetchall()

    cursor.execute("SELECT model_id, file_path, ttft_ms, status FROM arena_telemetry WHERE benchmark_type = 'MultimodalVision' ORDER BY id DESC")
    vision_raw = cursor.fetchall()

    cursor.execute("SELECT model_id, file_path, tps, e3_score, status FROM arena_telemetry WHERE benchmark_type = 'SpeculativeDraft' ORDER BY id DESC")
    spec_raw = cursor.fetchall()
    conn.close()

    pressure_map = {}
    for pm_id, pe3, pstatus, pcreated in pressure_raw:
        if pm_id not in pressure_map:
            pressure_map[pm_id] = {
                "max_effective_ctx": int(pe3) if pe3 > 0 else 0,
                "status": pstatus,
                "tested_ctx": 8192,
            }

    vision_map = {}
    for vm_id, vfile, vttft, vstatus in vision_raw:
        if vm_id not in vision_map:
            vision_map[vm_id] = {
                "sidecar_path": vfile,
                "ttft_ms": vttft,
                "status": vstatus,
            }

    spec_map = {}
    for sm_id, sfile, stps, se3, sstatus in spec_raw:
        if sm_id not in spec_map:
            spec_map[sm_id] = {
                "drafter": sfile,
                "speculative_tps": stps,
                "speedup_ratio": se3,
                "verdict": sstatus,
            }

    # Filtra por padrão apenas arquivos que existem fisicamente no disco
    if not include_ghosts:
        existing_rows = [r for r in all_rows if os.path.exists(r.get("file_path", ""))]
        ghost_count = len(all_rows) - len(existing_rows)
        if ghost_count > 0:
            print(f"[*] Filtrados {ghost_count} modelos fantasmas/legados inexistentes no disco. (Use --all para incluí-los).")
        all_rows = existing_rows

    manifests = load_local_manifests(models_dir)

    # Processamento e enriquecimento dos registros (com deduplicação canônica)
    processed_models = []
    seen_canonical_paths = set()

    for r in all_rows:
        fp = r.get("file_path", "")
        fp_lower = fp.lower()

        # Ignora arquivos safetensors parciais ou shards
        if fp_lower.endswith(".safetensors"):
            continue

        try:
            canonical_path = str(Path(fp).resolve()).lower()
        except Exception:
            canonical_path = fp_lower

        if canonical_path in seen_canonical_paths:
            continue
        seen_canonical_paths.add(canonical_path)

        fn = os.path.basename(fp).lower()
        folder = Path(fp).parent.name.lower()
        mid = str(r.get("model_id", "")).lower()
        fp_posix = Path(fp).resolve().as_posix().lower()

        manifest = manifests.get(fp_posix) or manifests.get(folder) or manifests.get(fn) or manifests.get(mid) or {}

        # Sincroniza metadados do manifest se disponíveis
        if "gliclass" in fn or "gliclass" in mid:
            tier = "Tier 0"
        else:
            tier = manifest.get("tier") or r.get("tier_classified") or "Tier 1"

        r["clean_tier"] = tier.split("(")[0].strip()

        # Resolução e inferência robusta de parâmetros
        raw_params = manifest.get("parameters") or r.get("parameters")
        if not raw_params or raw_params == "Unknown" or (raw_params == "2B" and "1.2b" in fn):
            raw_params = extract_parameters(fn, folder)
        r["parameters"] = raw_params or "Unknown"

        # Resolução e disambiguação de model_id
        r["model_id"] = manifest.get("model_id") or r.get("model_id") or r.get("model_name")
        r["quantization"] = manifest.get("quantization") or r.get("quantization") or "GGUF"
        r["original_publisher"] = manifest.get("original_publisher") or "LMStudio"
        r["has_mmproj"] = manifest.get("has_mmproj", False) or (r.get("has_mmproj_sidecar", 0) == 1)

        # Resolução de TTFT, TPOT e TPS dinâmicos
        ttft = r.get("dyn_ttft_ms") if (r.get("dyn_ttft_ms") or 0.0) > 0 else r.get("ttft_ms", 0.0)
        tpot = r.get("dyn_tpot_ms") if (r.get("dyn_tpot_ms") or 0.0) > 0 else r.get("tpot_ms", 0.0)
        tps = r.get("dyn_tps") if (r.get("dyn_tps") or 0.0) > 0 else r.get("tps", 0.0)
        if tps == 0.0 and tpot > 0.0:
            tps = 1000.0 / tpot

        vram = r.get("dyn_vram_peak_mb") if (r.get("dyn_vram_peak_mb") or 0.0) > 0 else r.get("vram_peak_mb", 0.0)
        e3 = r.get("dyn_e3_score") if (r.get("dyn_e3_score") or 0.0) > 0 else r.get("e3_score", 0.0)

        r["ttft_ms"] = round(ttft or 0.0, 1)
        r["tpot_ms"] = round(tpot or 0.0, 1)
        r["tps"] = round(tps or 0.0, 1)
        r["vram_peak_mb"] = round(vram or 0.0, 0)
        r["vram_headroom_mb"] = round(SAFE_WDDM_VRAM_CEILING_MB - r["vram_peak_mb"], 0)
        r["e3_score"] = round(e3 or 0.0, 2)

        verdict_badge, verdict_reason = evaluate_hermes_readiness(r)
        r["hermes_badge"] = verdict_badge
        r["hermes_reason"] = verdict_reason

        processed_models.append(r)

    # Ordenação dos modelos: estritamente crescente por Tier físico (0 a 4), depois por TPS decrescente
    tier_order = {
        "Tier 0": 0,
        "Tier 0.5": 1,
        "Tier 1": 2,
        "Tier 2": 3,
        "Tier 3": 4,
        "Tier 4": 5,
    }

    processed_models.sort(
        key=lambda x: (
            tier_order.get(x.get("clean_tier", "Tier 1"), 99),
            -(x.get("tps") or 0.0),
            -(x.get("e3_score") or 0.0),
        )
    )

    tier1_models = [m for m in processed_models if m.get("clean_tier") == "Tier 1"]

    # Destino do relatório
    reports_dir = repo_root / "docs" / "observability" / "reports"
    reports_dir.mkdir(parents=True, exist_ok=True)
    summary_md_file = Path(output_file) if output_file else reports_dir / "souls_llms_inventory_summary.md"

    lines = []
    lines.append("# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)")
    lines.append(f"> **Gerado em:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} | **Banco SSOT:** `{db_path}`")
    lines.append(f"> **Host Físico:** Intel Core i9-9880H (AVX2) | 32GB RAM | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)")
    lines.append(f"> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB) & ADR-007 (Desacoplamento dos Benchmark Types 1 a 5)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # =========================================================================
    # SEÇÃO 1: LEADERBOARD TIER 1 — PRONTIDÃO PARA O HERMES AGENT
    # =========================================================================
    lines.append("## 🏆 1. LEADERBOARD EXECUTIVO TIER 1: PRONTIDÃO PARA O HERMES AGENT")
    lines.append("Esta matriz classifica exclusivamente os modelos de **Tier 1** (Live Chat, Coding e Tool-Calling) capazes de acelerar localmente o Hermes Agent sem colapso de VRAM.")
    lines.append("")
    lines.append("| Rank | Modelo / Identificador | Params | Quant | Throughput (TPS) | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Qualificação Hermes Agent |")
    lines.append("| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |")

    for idx, m in enumerate(tier1_models, start=1):
        m_id = m.get("model_id") or m.get("model_name")
        tps_str = f"**{m['tps']:.1f}**" if m['tps'] > 0 else "N/A"
        ttft_str = f"{m['ttft_ms']:.1f}" if m['ttft_ms'] > 0 else "N/A"
        vram_str = f"{m['vram_peak_mb']:.0f} MB" if m['vram_peak_mb'] > 0 else "N/A"
        headroom_str = f"{m['vram_headroom_mb']:.0f} MB" if m['vram_peak_mb'] > 0 else "5.294 MB"
        e3_str = f"{m['e3_score']:.2f}" if m['e3_score'] > 0 else "N/A"

        lines.append(
            f"| {idx} | `{m_id}` | {m.get('parameters', 'N/A')} | `{m.get('quantization', 'N/A')}` | "
            f"{tps_str} | {ttft_str} | {vram_str} | {headroom_str} | {e3_str} | {m['hermes_badge']} |"
        )

    lines.append("")
    lines.append("> **Critério de Ouro FinOps:** Modelos com mais de **35 tokens/s** e VRAM inferior a **4.200 MB** deixam margem folgada para até **8k tokens de contexto**, sendo os candidatos ideais para a experiência mestre do Hermes Agent.")
    lines.append("")
    lines.append("---")
    lines.append("")

    if not tier1_only:
        # =========================================================================
        # SEÇÃO 2: MATRIZ DE SILÍCIO POR TIER OPERACIONAL (ORDEM CRESCENTE TIER 0 A 4)
        # =========================================================================
        lines.append(f"## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER ({len(processed_models)} Modelos)")
        lines.append("Apresentação canônica e sequencial de todos os modelos locais categorizados por Tier Físico e ordenados por throughput decrescente dentro de cada camada:")
        lines.append("")

        ordered_tiers = ["Tier 0", "Tier 0.5", "Tier 1", "Tier 2", "Tier 3", "Tier 4"]
        for tier_key in ordered_tiers:
            models_in_tier = [m for m in processed_models if m.get("clean_tier") == tier_key]
            if not models_in_tier:
                continue

            info = TIER_DESCRIPTIONS.get(tier_key, {})
            lines.append(f"### 🎯 {info.get('title', tier_key)}")
            lines.append(f"> **Papel Arquitetural:** {info.get('role', '')}")
            lines.append(f"> **Hardware Alvo & Limites:** {info.get('specs', '')}")
            lines.append(f"> **Motores Homologados:** {info.get('engines', '')}")
            lines.append("")
            lines.append("| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |")
            lines.append("| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |")

            for sub_idx, m in enumerate(models_in_tier, start=1):
                m_id = m.get("model_id") or m.get("model_name")
                params_str = m.get("parameters", "N/A")
                size_str = format_bytes(m.get("file_size_bytes"))
                tps_str = f"{m['tps']:.1f}" if m['tps'] > 0 else "N/A"
                ttft_str = f"{m['ttft_ms']:.1f}" if m['ttft_ms'] > 0 else "N/A"
                vram_str = f"{m['vram_peak_mb']:.0f} MB" if m['vram_peak_mb'] > 0 else "0 MB"
                e3_str = f"{m['e3_score']:.2f}" if m['e3_score'] > 0 else "N/A"
                engine_str = m.get("engine_selected") or "llama.cpp"

                lines.append(
                    f"| {sub_idx} | `{m_id}` | {params_str} | `{m.get('quantization')}` | {size_str} | `{engine_str}` | "
                    f"{tps_str} | {ttft_str} | {vram_str} | {e3_str} | {m['hermes_badge']} |"
                )
            lines.append("")

        lines.append("---")
        lines.append("")

        # =========================================================================
        # SEÇÃO 3: COLISEU COGNITIVO (TYPE 2: RUST AST, TOOLS & CoT)
        # =========================================================================
        lines.append("## 🧬 3. COLISEU COGNITIVO (BENCHMARK TYPE 2: QUALIDADE & RIGIDEZ)")
        lines.append("Aferição de integridade lógica necessária para subagentes executivos e MCP Tools:")
        lines.append("")
        lines.append("| Modelo | Tier | 🦀 Track 2A (Rust AST Code) | 🛠️ Track 2B (MCP JSON Tools) | 🧠 Track 2C (CoT Reasoning) | Veredito Cognitivo |")
        lines.append("| :--- | :---: | :---: | :---: | :---: | :--- |")

        eval_models = [m for m in processed_models if (m.get("score_code_ast") or 0.0) > 0 or (m.get("score_json_tools") or 0.0) > 0 or (m.get("score_reasoning") or 0.0) > 0]
        if not eval_models:
            eval_models = tier1_models

        for m in eval_models:
            m_id = m.get("model_id") or m.get("model_name")
            s_code = f"{m['score_code_ast'] * 100:.0f}%" if (m.get("score_code_ast") or 0.0) > 0 else "N/A"
            s_json = f"{m['score_json_tools'] * 100:.0f}%" if (m.get("score_json_tools") or 0.0) > 0 else "N/A"
            s_reason = f"{m['score_reasoning'] * 100:.0f}%" if (m.get("score_reasoning") or 0.0) > 0 else "N/A"

            has_eval = any(v != "N/A" for v in [s_code, s_json, s_reason])
            verdict = "🟢 Aprovado em Sintaxe Estrita" if (m.get("score_code_ast", 0) >= 0.85 or m.get("score_json_tools", 0) >= 0.85) else ("🟡 Aguardando Bateria Type 2" if not has_eval else "🔴 Falha em Validação Estrita")
            lines.append(f"| `{m_id}` | {m['clean_tier']} | {s_code} | {s_json} | {s_reason} | {verdict} |")

        lines.append("")
        lines.append("---")
        lines.append("")

        # =========================================================================
        # SEÇÃO 4: PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (TYPE 5)
        # =========================================================================
        if pressure_map:
            lines.append(f"## 🎯 4. PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (BENCHMARK TYPE 5) ({len(pressure_map)} Avaliados)")
            lines.append("Aferição física de recuperação de agulha (*Needle-in-a-Haystack*) injetada a 50% de profundidade sob dGPU CUDA:")
            lines.append("")
            lines.append("| Modelo | Contexto Testado | Contexto Efetivo Recuperado | Status Agulha | Veredito de Atenção |")
            lines.append("| :--- | :---: | :---: | :---: | :--- |")

            for mid, pinfo in pressure_map.items():
                status = pinfo["status"]
                verdict = (
                    "🟢 PASS_STABLE (Sem perda de foco)"
                    if status == "PASS_STABLE"
                    else ("🟠 ATTENTION_DEGRADED (Perda após 4k)" if status == "ATTENTION_DEGRADED" else "🔴 RETRIEVAL_FAILED")
                )
                t_ctx = f"{pinfo['tested_ctx'] // 1024}k tokens"
                e_ctx = f"{pinfo['max_effective_ctx'] // 1024}k tokens" if pinfo['max_effective_ctx'] > 0 else "0k (Falha)"
                lines.append(f"| `{mid}` | {t_ctx} | {e_ctx} | `{status}` | {verdict} |")

            lines.append("")
            lines.append("---")
            lines.append("")

        # =========================================================================
        # SEÇÃO 5: BENCHMARK TYPE 3: MULTIMODAL VISION & SIDECARS MMPROJ
        # =========================================================================
        lines.append("## 👁️ 5. AUDITORIA MULTIMODAL DE VISÃO (BENCHMARK TYPE 3: MMPROJ PROJECTION)")
        lines.append("Verificação física de acoplamento de tensores visuais mmproj e latência de projeção:")
        lines.append("")
        lines.append("| Modelo | Tier | Sidecar mmproj | Latência Projeção | Status Type 3 | Veredito Multimodal |")
        lines.append("| :--- | :---: | :---: | :---: | :---: | :--- |")

        for m in processed_models:
            m_id = m.get("model_id") or m.get("model_name")
            vinfo = vision_map.get(m_id) or vision_map.get(m.get("model_name"))
            if vinfo:
                sidecar = vinfo["sidecar_path"]
                lat = f"{vinfo['ttft_ms']:.1f}ms" if vinfo["ttft_ms"] > 0 else "0.0ms"
                status = vinfo["status"]
                verdict = "✅ Projetor Pareado (Pronto para VLM)" if status == "PAIRED_QUALIFIED" else "🛡️ Texto/Código Puro (Sem Sidecar)"
                lines.append(f"| `{m_id}` | {m['clean_tier']} | `{sidecar}` | {lat} | `{status}` | {verdict} |")
            elif m.get("clean_tier") == "Tier 3":
                lines.append(f"| `{m_id}` | {m['clean_tier']} | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |")

        lines.append("")
        lines.append("---")
        lines.append("")

        # =========================================================================
        # SEÇÃO 6: BENCHMARK TYPE 4: ACELERAÇÃO ESPECULATIVA & MTP DRAFTING
        # =========================================================================
        lines.append("## ⚡ 6. ACELERAÇÃO ESPECULATIVA & MTP DRAFTING (BENCHMARK TYPE 4)")
        lines.append("Aferição física de speedup por predição multi-token (MTP) e acoplamento de modelos Drafter:")
        lines.append("")
        lines.append("| Modelo Target | Drafter Acoplado | Taxa Aceitação Alpha (α) | Throughput Projetado | Speedup Real | Veredito FinOps |")
        lines.append("| :--- | :---: | :---: | :---: | :---: | :--- |")

        for m in processed_models:
            m_id = m.get("model_id") or m.get("model_name")
            sinfo = spec_map.get(m_id) or spec_map.get(m.get("model_name"))
            if sinfo:
                drafter = sinfo["drafter"]
                speedup = f"{sinfo['speedup_ratio']:.2f}x"
                spec_tps = f"{sinfo['speculative_tps']:.1f} tok/s"
                verdict_str = sinfo["verdict"]
                alpha_val = m.get("mtp_acceptance_rate") or 0.0
                alpha_str = f"{alpha_val * 100:.1f}%" if alpha_val > 0 else "0.0%"
                if verdict_str == "MTP_QUALIFIED":
                    badge = "🟢 MTP Acelerado (Heads Integradas)"
                elif verdict_str == "SPEEDUP_FEASIBLE":
                    badge = "🔵 Drafter Pareado Fechado"
                else:
                    badge = "⚪ Autoregressivo Standalone"
                lines.append(f"| `{m_id}` | `{drafter}` | {alpha_str} | {spec_tps} | {speedup} | {badge} |")
            elif m.get("clean_tier") == "Tier 4":
                lines.append(f"| `{m_id}` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |")

        lines.append("")

    lines.append("---")
    lines.append("*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*")

    report_content = "\n".join(lines)
    with open(summary_md_file, "w", encoding="utf-8") as f:
        f.write(report_content)

    print(f"\n[✓] Relatório Executivo gerado com sucesso em:")
    print(f"    {summary_md_file}\n")
    return str(summary_md_file)

def main():
    parser = argparse.ArgumentParser(description="SOULS Engine v7 Silicon Observability & Hermes LLM Dossier")
    parser.add_argument("--db", type=str, default=None, help="Caminho customizado para o souls_heuristic_vault.db")
    parser.add_argument("--output", "-o", type=str, default=None, help="Caminho do arquivo Markdown de saída")
    parser.add_argument("--tier1-only", action="store_true", help="Gera relatório focado exclusivamente no Tier 1 (Hermes Agent)")
    parser.add_argument("--all", action="store_true", help="Inclui modelos legados/fantasmas cujo arquivo não existe no disco")
    parser.add_argument("--json", action="store_true", help="Exporta inventário em JSON")
    args = parser.parse_args()

    if args.json:
        db_path = resolve_db_path(args.db)
        conn = sqlite3.connect(db_path)
        data = fetch_inventory_data(conn)
        conn.close()
        print(json.dumps(data, indent=2, ensure_ascii=False))
    else:
        generate_inventory_report(args.db, args.output, args.tier1_only, include_ghosts=args.all)

if __name__ == "__main__":
    main()
