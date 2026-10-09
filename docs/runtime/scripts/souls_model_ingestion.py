#!/usr/bin/env python3
"""
SOULS Engine v7 — Model Ingestion & Unified Silicon Taxonomy
============================================================
Script de engenharia para transplantar, organizar e sincronizar modelos locais
baixados via LM Studio (C:\\Users\\rosas\\.lmstudio\\models) para o repositório canônico
estruturado por Tiers Operacionais (C:\\Users\\rosas\\llm_local_models).

Modos de Uso:
    python souls_model_ingestion.py --dry-run
    python souls_model_ingestion.py --apply
    python souls_model_ingestion.py --sync
"""

import os
import sys
import json
import shutil
import hashlib
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime

# Diretórios padrão
DEFAULT_SOURCE = Path(os.environ.get("USERPROFILE", "C:/Users/rosas")) / ".lmstudio" / "models"
DEFAULT_TARGET = Path(os.environ.get("USERPROFILE", "C:/Users/rosas")) / "llm_local_models"

TIER_FOLDERS = {
    "Tier 0": "tier_0_bootstrap",
    "Tier 0.5": "tier_05_epistemic",
    "Tier 1": "tier_1_master_dgpu",
    "Tier 2": "tier_2_background_ram",
    "Tier 3": "tier_3_multimodal_vision",
    "Tier 4": "tier_4_speculative_draft",
}

def extract_quantization(name: str) -> str:
    name_upper = name.upper()
    for q in [
        "I1-Q4_K_S", "IQ3_M", "IQ4_XS", "IQ2_XXS", "Q4_K_M", "Q4_K_S",
        "Q5_K_M", "Q5_K_S", "Q8_0", "Q4_0", "Q4_1", "Q3_K_M", "Q2_K",
        "Q1_0", "I2_S", "BF16", "F16", "F32"
    ]:
        if q in name_upper:
            return q
    if "ONNX" in name_upper:
        return "ONNX"
    return "GGUF"

def extract_parameters(name: str, folder: str) -> str:
    combined = f"{name} {folder}".lower()
    for p in ["135m", "360m", "790m", "500m", "1.2b", "1.5b", "1.7b", "2.7b", "3.8b", "2b", "3b", "4b", "7b", "8b", "14b", "27b", "33b"]:
        if p in combined:
            return p.upper()
    return "Unknown"

def classify_tier(
    filename: str,
    folder_name: str,
    size_mb: float,
    has_paired_mmproj: bool,
    is_drafter: bool,
) -> str:
    combined = f"{filename} {folder_name}".lower()

    # 1. Drafters sempre são Tier 4
    if is_drafter or "dspark" in combined or "dflash" in combined or "draft" in combined:
        return "Tier 4"

    # 2. Modelos com projetor visual pareado são Tier 3
    if has_paired_mmproj or "vision" in combined or "-vl" in combined or "vlm" in combined:
        return "Tier 3"

    # 3. Modelos ultraleves < 350 MB são Tier 0 (Bootstrap / ONNX / Sanity)
    if "135m" in combined or "gliclass" in combined or "k1" in combined or size_mb < 350.0:
        return "Tier 0"

    # 4. Modelos de probing 360M a 1.8GB são Tier 0.5 (Sensor Epistêmico)
    if (
        ("360m" in combined or "790m" in combined or "1.2b" in combined or "1.5b" in combined or "bitnet" in combined)
        and size_mb <= 2000.0
    ):
        return "Tier 0.5"

    # 5. Modelos pesados > 4500 MB ou arquiteturas gigantes MoE são Tier 2
    if size_mb > 4500.0 or "27b" in combined or "33b" in combined or "moe" in combined or "apex" in combined:
        # Exceção: se for 27B quantizado em 1-bit que pesa < 4000MB mas é 27B arquitetural
        return "Tier 2"

    # 6. Padrão: Tier 1 (Master / Coding na dGPU RTX 2060m)
    return "Tier 1"

def sanitize_model_identity(publisher: str, repo: str, filename: str) -> Tuple[str, str]:
    """
    Normaliza o ID e o novo nome do arquivo para erradicar nomes genéricos
    como 'unsloth.Q4_K_M.gguf' ou 'ggml-model-i2_s.gguf'.
    """
    fn_lower = filename.lower()
    clean_name = filename[:-5] if filename.endswith(".gguf") else filename

    # Caso especial: unsloth genérico em pasta mamba-codestral
    if "unsloth" in fn_lower and "mamba-codestral" in repo.lower():
        quant = extract_quantization(filename)
        return "mamba-codestral-7b-instruct-agnuxo", f"Mamba-Codestral-7B-Instruct-{quant}.gguf"

    # Caso especial: BitNet 1.58 genérico
    if "ggml-model" in fn_lower and "bitnet" in repo.lower():
        quant = extract_quantization(filename)
        return "bitnet-b1.58-2b-4t", f"BitNet-b1.58-2B-4T-{quant}.gguf"

    # Caso padrão: usa o nome do repositório/arquivo
    clean_id = clean_name.replace(".", "-").replace("_", "-").lower()
    return clean_id, filename

def analyze_models(source_dir: Path) -> List[Dict[str, Any]]:
    """Varre e monta o catálogo de intenção de migração."""
    discovered = []

    for dirpath, dirnames, filenames in os.walk(source_dir):
        gguf_candidates = [f for f in filenames if f.endswith(".gguf") or f.endswith(".onnx")]
        if not gguf_candidates:
            continue

        rel_dir = Path(dirpath).relative_to(source_dir)
        parts = rel_dir.parts
        publisher = parts[0] if len(parts) > 0 else "local"
        repo = parts[1] if len(parts) > 1 else (parts[0] if parts else "root")

        # Verifica se há sidecar mmproj nesta pasta
        mmproj_files = [
            f for f in filenames
            if ("mmproj" in f.lower() or "-vision.gguf" in f.lower()) and f.endswith(".gguf")
        ]

        # Modelos principais nesta pasta (não adiciona mmproj avulso como modelo principal)
        primary_models = [
            f for f in gguf_candidates
            if not ("mmproj" in f.lower() or "-vision.gguf" in f.lower())
        ]

        for pm in primary_models:
            pm_path = Path(dirpath) / pm
            sz_mb = pm_path.stat().st_size / (1024 * 1024)
            is_drafter = any(k in pm.lower() for k in ["dspark", "dflash", "draft"])

            # Pareamento estrito de mmproj
            paired_mmproj = None
            if mmproj_files:
                if len(mmproj_files) == 1 and len(primary_models) == 1:
                    paired_mmproj = mmproj_files[0]
                else:
                    # Tenta prefix match
                    for mf in mmproj_files:
                        stem = pm.split("-")[0].lower()
                        if stem in mf.lower():
                            paired_mmproj = mf
                            break

            tier = classify_tier(
                filename=pm,
                folder_name=repo,
                size_mb=sz_mb,
                has_paired_mmproj=paired_mmproj is not None,
                is_drafter=is_drafter,
            )

            model_id, target_filename = sanitize_model_identity(publisher, repo, pm)
            params = extract_parameters(target_filename, repo)
            quant = extract_quantization(target_filename)

            # Satélites associados
            satellites = [
                f for f in filenames
                if f in ["tokenizer.json", "tokenizer_config.json", "config.json", "adapter_config.json"]
            ]

            target_folder_name = model_id.lower()
            tier_dir_name = TIER_FOLDERS.get(tier, "tier_1_master_dgpu")

            discovered.append({
                "source_path": pm_path,
                "publisher": publisher,
                "repo": repo,
                "original_filename": pm,
                "target_filename": target_filename,
                "model_id": model_id,
                "parameters": params,
                "quantization": quant,
                "size_mb": sz_mb,
                "size_bytes": pm_path.stat().st_size,
                "tier": tier,
                "tier_folder": tier_dir_name,
                "target_folder_name": target_folder_name,
                "paired_mmproj": Path(dirpath) / paired_mmproj if paired_mmproj else None,
                "satellites": [Path(dirpath) / s for s in satellites],
            })

    # Desambiguação de pastas para evitar colisões
    seen_folders = {}
    for m in discovered:
        key = (m["tier_folder"], m["target_folder_name"])
        seen_folders.setdefault(key, []).append(m)

    for (tf, tfn), group in seen_folders.items():
        if len(group) > 1:
            for item in group:
                clean_pub = item["publisher"].replace(".", "-").replace("_", "-").lower()
                item["target_folder_name"] = f"{item['target_folder_name']}-{clean_pub}"

    return discovered

def detect_duplicates(models: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    """Agrupa modelos por tamanho em bytes para identificar duplicatas idênticas."""
    by_size = {}
    for m in models:
        sz = m["size_bytes"]
        by_size.setdefault(sz, []).append(m)
    
    return {sz: group for sz, group in by_size.items() if len(group) > 1}

def execute_dry_run(models: List[Dict[str, Any]], target_base: Path):
    print("\n" + "=" * 140)
    print("  🏟️ SOULS ENGINE v7 — PLANO DE INGESTÃO & ORGANIZAÇÃO DE SILÍCIO (DRY-RUN)")
    print("=" * 140)
    print(f"Origem : {DEFAULT_SOURCE}")
    print(f"Destino: {target_base}\n")

    # Tabela formatada
    header = f"{'Tier':<10} | {'Model ID':<38} | {'Params':<6} | {'Quant':<9} | {'Size(MB)':>9} | {'mmproj':<7} | {'Destino Planejado'}"
    print(header)
    print("-" * 140)

    total_bytes = 0
    for m in sorted(models, key=lambda x: (x['tier'], x['size_mb'])):
        total_bytes += m['size_bytes']
        mmproj_flag = "SIM" if m['paired_mmproj'] else "-"
        dest = f"{m['tier_folder']}/{m['target_folder_name']}/"
        print(f"{m['tier']:<10} | {m['model_id'][:38]:<38} | {m['parameters']:<6} | {m['quantization']:<9} | {m['size_mb']:>9.1f} | {mmproj_flag:<7} | {dest}")

    print("-" * 140)
    print(f"Total de Modelos: {len(models)} | Volume Total: {total_bytes / (1024**3):.2f} GB\n")

    # Relatório de duplicatas
    dups = detect_duplicates(models)
    if dups:
        print("[!] DUPLICATAS FÍSICAS DETECTADAS (Mesmo tamanho em bytes):")
        for sz, group in dups.items():
            print(f"   -> {sz / (1024*1024):.1f} MB:")
            for item in group:
                print(f"       * {item['publisher']}/{item['repo']} -> {item['original_filename']}")
        print("   Nota: No modo --apply, duplicatas idênticas serão desambiguadas com sufixo do publisher.")
    print("=" * 140 + "\n")

def execute_apply(models: List[Dict[str, Any]], target_base: Path):
    print(f"[*] Iniciando transplante e organização em: {target_base}")
    target_base.mkdir(parents=True, exist_ok=True)

    moved_count = 0
    manifests_created = 0

    for m in models:
        tier_dir = target_base / m["tier_folder"]
        model_dir = tier_dir / m["target_folder_name"]
        model_dir.mkdir(parents=True, exist_ok=True)

        # Destino do GGUF principal
        dest_gguf = model_dir / m["target_filename"]

        # Se já existe no destino, desambigua com publisher
        if dest_gguf.exists() and dest_gguf != m["source_path"]:
            dest_gguf = model_dir / f"{m['publisher']}_{m['target_filename']}"

        print(f" -> [MOVE] {m['original_filename']} ==> {dest_gguf.relative_to(target_base)}")
        try:
            shutil.move(str(m["source_path"]), str(dest_gguf))
            moved_count += 1
        except Exception as e:
            print(f"[!] Erro ao mover {m['source_path']}: {e}", file=sys.stderr)
            continue

        # Move sidecar mmproj se houver
        dest_mmproj_name = None
        if m["paired_mmproj"] and m["paired_mmproj"].exists():
            dest_mmproj = model_dir / m["paired_mmproj"].name
            print(f"    + [MMPROJ] {m['paired_mmproj'].name} ==> {dest_mmproj.relative_to(target_base)}")
            shutil.move(str(m["paired_mmproj"]), str(dest_mmproj))
            dest_mmproj_name = m["paired_mmproj"].name

        # Copia satélites se houver
        copied_satellites = []
        for sat in m["satellites"]:
            if sat.exists():
                dest_sat = model_dir / sat.name
                shutil.copy2(str(sat), str(dest_sat))
                copied_satellites.append(sat.name)

        # Escreve model_manifest.json
        manifest = {
            "model_id": m["model_id"],
            "target_filename": dest_gguf.name,
            "tier": m["tier"],
            "parameters": m["parameters"],
            "quantization": m["quantization"],
            "size_mb": round(m["size_mb"], 2),
            "original_publisher": m["publisher"],
            "original_repo": m["repo"],
            "original_filename": m["original_filename"],
            "has_mmproj": dest_mmproj_name is not None,
            "mmproj_filename": dest_mmproj_name,
            "satellites": copied_satellites,
            "migrated_at": datetime.utcnow().isoformat() + "Z",
        }

        manifest_path = model_dir / "model_manifest.json"
        with open(manifest_path, "w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2, ensure_ascii=False)
        manifests_created += 1

    print("\n" + "=" * 80)
    print(f"[✓] Migração concluída com sucesso!")
    print(f"    - Modelos movidos: {moved_count}")
    print(f"    - Manifests criados: {manifests_created}")
    print(f"    - Repositório unificado pronto em: {target_base}")
    print("=" * 80)

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Souls Engine Model Ingestion & Taxonomy Manager")
    parser.add_argument("--dry-run", action="store_true", help="Simula o plano de migração sem mover arquivos")
    parser.add_argument("--apply", action="store_true", help="Aplica a migração física e cria a taxonomia")
    parser.add_argument("--sync", action="store_true", help="Detecta novos downloads no LM Studio e sincroniza")
    parser.add_argument("--source", type=str, default=str(DEFAULT_SOURCE), help="Diretório de origem do LM Studio")
    parser.add_argument("--target", type=str, default=str(DEFAULT_TARGET), help="Diretório de destino unificado")
    args = parser.parse_args()

    source = Path(args.source)
    target = Path(args.target)

    if not source.exists():
        print(f"[!] Diretório de origem não existe: {source}", file=sys.stderr)
        sys.exit(1)

    models = analyze_models(source)
    if not models:
        print(f"[*] Nenhum modelo GGUF/ONNX encontrado em: {source}")
        sys.exit(0)

    if args.apply or args.sync:
        execute_apply(models, target)
    else:
        execute_dry_run(models, target)

if __name__ == "__main__":
    main()
