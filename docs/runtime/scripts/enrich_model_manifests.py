#!/usr/bin/env python3
"""
SOULS Engine v7 — Model Manifest & Chat Template Enricher
==========================================================
Scans all local models in C:\\Users\\rosas\\llm_local_models, parses GGUF binary
headers, extracts canonical chat templates, context lengths, architecture
specs, and role markers, and generates enriched, high-fidelity model_manifest.json
files for 100% operational readiness across Souls Engine and Hermes Agent.
"""

import os
import sys
import json
import struct
import re
from pathlib import Path
from datetime import datetime
from typing import Dict, Any, Optional, Tuple, List

BASE_MODELS_DIR = Path(os.environ.get("USERPROFILE", "C:/Users/rosas")) / "llm_local_models"

TYPE_SIZES = {0: 1, 1: 1, 2: 2, 3: 2, 4: 4, 5: 4, 6: 4, 7: 1, 10: 8, 11: 8, 12: 8}
FMT_MAP = {0: '<B', 1: '<b', 2: '<H', 3: '<h', 4: '<I', 5: '<i', 6: '<f', 7: '<B', 10: '<Q', 11: '<q', 12: '<d'}

def parse_gguf_metadata(filepath: Path) -> Optional[Dict[str, Any]]:
    """Zero-dependency GGUF v2/v3 header parser in pure Python."""
    if not filepath.exists() or filepath.suffix.lower() != ".gguf":
        return None

    meta = {}
    try:
        with open(filepath, "rb") as f:
            magic = f.read(4)
            if magic != b"GGUF":
                return None
            version, n_tensors, n_kv = struct.unpack("<IQQ", f.read(20))
            meta["__version"] = version
            meta["__n_tensors"] = n_tensors
            meta["__n_kv"] = n_kv

            def read_str():
                slen = struct.unpack("<Q", f.read(8))[0]
                return f.read(slen).decode("utf-8", errors="replace")

            def read_val(vtype):
                if vtype in TYPE_SIZES:
                    raw = f.read(TYPE_SIZES[vtype])
                    val = struct.unpack(FMT_MAP[vtype], raw)[0]
                    return bool(val) if vtype == 7 else val
                elif vtype == 8:
                    return read_str()
                elif vtype == 9:
                    item_type, arr_len = struct.unpack("<IQ", f.read(12))
                    if arr_len > 256:
                        # Skip large arrays like token lists and logits
                        if item_type in TYPE_SIZES:
                            f.seek(arr_len * TYPE_SIZES[item_type], 1)
                        elif item_type == 8:
                            for _ in range(arr_len):
                                sl = struct.unpack("<Q", f.read(8))[0]
                                f.seek(sl, 1)
                        else:
                            raise ValueError(f"Cannot skip array of type {item_type}")
                        return f"<skipped array of {arr_len} items>"
                    else:
                        return [read_val(item_type) for _ in range(arr_len)]
                else:
                    raise ValueError(f"Unknown type: {vtype}")

            for _ in range(n_kv):
                key = read_str()
                vtype = struct.unpack("<I", f.read(4))[0]
                val = read_val(vtype)
                if not isinstance(val, str) or not val.startswith("<skipped"):
                    meta[key] = val

        return meta
    except Exception as e:
        print(f"[!] Error reading GGUF header for {filepath.name}: {e}", file=sys.stderr)
        return None

def detect_chat_format(chat_template: Optional[str], arch: str, model_id: str) -> Dict[str, Any]:
    """Detects chat format and generates canonical role markers."""
    mid = model_id.lower()
    
    # 1. Fallback rules for models without embedded Jinja template
    if not chat_template:
        if "zamba2" in mid or arch == "zamba2":
            return {
                "format_name": "zamba2",
                "has_embedded_template": False,
                "system_prefix": "<extra_id_0>System\n",
                "system_suffix": "\n",
                "user_prefix": "<extra_id_1>User\n",
                "user_suffix": "\n",
                "assistant_prefix": "<extra_id_1>Assistant\n",
                "assistant_suffix": "\n",
                "stop_tokens": ["<extra_id_1>", "<eos>", "<|endoftext|>"]
            }
        elif "codestral" in mid or "mistral" in mid:
            return {
                "format_name": "mistral",
                "has_embedded_template": False,
                "system_prefix": "[INST] ",
                "system_suffix": "\n\n",
                "user_prefix": "",
                "user_suffix": " [/INST]",
                "assistant_prefix": "",
                "assistant_suffix": "</s>",
                "stop_tokens": ["</s>", "[INST]"]
            }
        elif "mamba" in mid:
            return {
                "format_name": "raw_completion",
                "has_embedded_template": False,
                "system_prefix": "",
                "system_suffix": "\n",
                "user_prefix": "",
                "user_suffix": "\n",
                "assistant_prefix": "",
                "assistant_suffix": "",
                "stop_tokens": ["\n\n", "<|endoftext|>"]
            }
        else:
            return {
                "format_name": "chatml_fallback",
                "has_embedded_template": False,
                "system_prefix": "<|im_start|>system\n",
                "system_suffix": "<|im_end|>\n",
                "user_prefix": "<|im_start|>user\n",
                "user_suffix": "<|im_end|>\n",
                "assistant_prefix": "<|im_start|>assistant\n",
                "assistant_suffix": "<|im_end|>",
                "stop_tokens": ["<|im_end|>", "<|endoftext|>"]
            }

    # 2. Template inspection for models with embedded Jinja
    tpl = chat_template

    if "<|start_header_id|>" in tpl:
        return {
            "format_name": "llama3",
            "has_embedded_template": True,
            "system_prefix": "<|start_header_id|>system<|end_header_id|>\n\n",
            "system_suffix": "<|eot_id|>",
            "user_prefix": "<|start_header_id|>user<|end_header_id|>\n\n",
            "user_suffix": "<|eot_id|>",
            "assistant_prefix": "<|start_header_id|>assistant<|end_header_id|>\n\n",
            "assistant_suffix": "<|eot_id|>",
            "stop_tokens": ["<|eot_id|>", "<|end_of_text|>"]
        }
    elif "<|system|>" in tpl and ("phi" in arch or "phi" in mid):
        return {
            "format_name": "phi3_phi4",
            "has_embedded_template": True,
            "system_prefix": "<|system|>\n",
            "system_suffix": "<|end|>\n",
            "user_prefix": "<|user|>\n",
            "user_suffix": "<|end|>\n",
            "assistant_prefix": "<|assistant|>\n",
            "assistant_suffix": "<|end|>",
            "stop_tokens": ["<|end|>", "<|user|>"]
        }
    elif "<start_of_turn>" in tpl or "gemma" in arch:
        return {
            "format_name": "gemma",
            "has_embedded_template": True,
            "system_prefix": "",
            "system_suffix": "",
            "user_prefix": "<start_of_turn>user\n",
            "user_suffix": "<end_of_turn>\n",
            "assistant_prefix": "<start_of_turn>model\n",
            "assistant_suffix": "<end_of_turn>",
            "stop_tokens": ["<end_of_turn>"]
        }
    elif "<|im_start|>" in tpl:
        # Check if Nemotron or thinking tag is preferred
        is_nemotron = "nemotron" in mid or "enable_thinking" in tpl
        asst_prefix = "<|im_start|>assistant\n<think></think>" if is_nemotron else "<|im_start|>assistant\n"
        return {
            "format_name": "chatml_nemotron" if is_nemotron else "chatml",
            "has_embedded_template": True,
            "system_prefix": "<|im_start|>system\n",
            "system_suffix": "<|im_end|>\n",
            "user_prefix": "<|im_start|>user\n",
            "user_suffix": "<|im_end|>\n",
            "assistant_prefix": asst_prefix,
            "assistant_suffix": "<|im_end|>",
            "stop_tokens": ["<|im_end|>", "<|endoftext|>"]
        }
    elif "[INST]" in tpl:
        return {
            "format_name": "mistral_inst",
            "has_embedded_template": True,
            "system_prefix": "[INST] ",
            "system_suffix": "\n\n",
            "user_prefix": "",
            "user_suffix": " [/INST]",
            "assistant_prefix": "",
            "assistant_suffix": "</s>",
            "stop_tokens": ["</s>", "[INST]"]
        }
    else:
        # Generic fallback
        return {
            "format_name": "custom_jinja",
            "has_embedded_template": True,
            "system_prefix": "<|im_start|>system\n",
            "system_suffix": "<|im_end|>\n",
            "user_prefix": "<|im_start|>user\n",
            "user_suffix": "<|im_end|>\n",
            "assistant_prefix": "<|im_start|>assistant\n",
            "assistant_suffix": "<|im_end|>",
            "stop_tokens": ["<|im_end|>", "<|endoftext|>"]
        }

def build_recommended_runtime(
    tier: str,
    arch: str,
    context_length: int,
    mmproj_filename: Optional[str],
    is_drafter: bool
) -> Dict[str, Any]:
    """Computes recommended execution engine and physical CLI flags."""
    if tier == "Tier 1":
        ctx = min(context_length, 16384) if context_length > 0 else 8192
        return {
            "engine": "llama-cuda",
            "context_window": ctx,
            "flags": ["-ngl", "99", "-ctk", "f16", "-ctv", "q4_0", "-fa", "on", "-c", str(ctx)],
            "rationale": "dGPU NVIDIA RTX 2060m bare-metal acceleration with Asymmetric Quantized KV Cache (FP16 Key + Q4_0 Value) to strictly observe 5.2GB WDDM ceiling."
        }
    elif tier == "Tier 0.5":
        ctx = min(context_length, 8192) if context_length > 0 else 4096
        return {
            "engine": "llama-cpu",
            "context_window": ctx,
            "flags": ["-ngl", "0", "-t", "8", "-c", str(ctx)],
            "rationale": "Intel Core i9 AVX2 CPU execution with 0 MB VRAM footprint for zero dGPU interference during primary tasks."
        }
    elif tier == "Tier 0":
        ctx = min(context_length, 4096) if context_length > 0 else 2048
        return {
            "engine": "llama-cpu",
            "context_window": ctx,
            "flags": ["-ngl", "0", "-t", "4", "-c", str(ctx)],
            "rationale": "Ultralight bootstrap and heuristic triage execution on CPU."
        }
    elif tier == "Tier 2":
        ctx = min(context_length, 16384) if context_length > 0 else 8192
        return {
            "engine": "llama-cpu",
            "context_window": ctx,
            "flags": ["-ngl", "0", "-t", "8", "-c", str(ctx)],
            "rationale": "Large parameter model or MoE pinned to 32GB DDR4 System RAM via AVX2."
        }
    elif tier == "Tier 3":
        ctx = min(context_length, 8192) if context_length > 0 else 4096
        flags = ["-ngl", "99", "-ctk", "f16", "-ctv", "q4_0", "-fa", "on", "-c", str(ctx)]
        if mmproj_filename:
            flags.extend(["--mmproj", mmproj_filename])
        return {
            "engine": "llama-cuda",
            "context_window": ctx,
            "flags": flags,
            "rationale": "Multimodal Vision-Language Model on dGPU with paired mmproj projector."
        }
    elif tier == "Tier 4":
        return {
            "engine": "llama-cuda",
            "context_window": 4096,
            "flags": ["-ngl", "99", "-c", "4096"],
            "rationale": "Speculative Drafter model paired with master via -md flag."
        }
    else:
        return {
            "engine": "llama-cuda",
            "context_window": 8192,
            "flags": ["-ngl", "99", "-ctk", "f16", "-ctv", "q4_0", "-fa", "on", "-c", "8192"],
            "rationale": "Default dGPU profile."
        }

def enrich_all_manifests(base_dir: Path) -> Tuple[int, int]:
    print("=" * 110)
    print("  🚀 SOULS ENGINE v7 — ENRIQUECIMENTO CANÔNICO DE MODEL_MANIFEST.JSON E CHAT TEMPLATES")
    print("=" * 110)
    print(f"Diretório base: {base_dir}\n")

    manifest_paths = list(base_dir.rglob("model_manifest.json"))
    print(f"[*] Manifestos existentes encontrados: {len(manifest_paths)}")

    updated = 0
    errors = 0

    for m_path in sorted(manifest_paths):
        model_dir = m_path.parent
        try:
            with open(m_path, "r", encoding="utf-8") as f:
                manifest_data = json.load(f)
        except Exception as e:
            print(f"[!] Erro ao ler {m_path}: {e}")
            errors += 1
            continue

        model_id = manifest_data.get("model_id", model_dir.name)
        target_filename = manifest_data.get("target_filename")
        gguf_path = model_dir / target_filename if target_filename else None

        if not gguf_path or not gguf_path.exists():
            ggufs = [g for g in model_dir.glob("*.gguf") if "mmproj" not in g.name.lower()]
            if ggufs:
                gguf_path = ggufs[0]
                manifest_data["target_filename"] = gguf_path.name

        if not gguf_path or not gguf_path.exists():
            print(f"[!] Nenhum GGUF encontrado em {model_dir.name}")
            errors += 1
            continue

        meta = parse_gguf_metadata(gguf_path)
        if not meta:
            print(f"[!] Falha no parsing GGUF de {gguf_path.name}")
            errors += 1
            continue

        arch = meta.get("general.architecture", "unknown")
        ctx = meta.get(f"{arch}.context_length", meta.get("llama.context_length", 0))
        block_count = meta.get(f"{arch}.block_count", meta.get("llama.block_count", 0))
        embedding_length = meta.get(f"{arch}.embedding_length", meta.get("llama.embedding_length", 0))
        head_count = meta.get(f"{arch}.attention.head_count", meta.get("llama.attention.head_count", 0))
        head_count_kv = meta.get(f"{arch}.attention.head_count_kv", head_count)
        feed_forward_length = meta.get(f"{arch}.feed_forward_length", 0)
        chat_template = meta.get("tokenizer.chat_template")
        bos_id = meta.get("tokenizer.ggml.bos_token_id")
        eos_id = meta.get("tokenizer.ggml.eos_token_id")

        chat_format_info = detect_chat_format(chat_template, arch, model_id)
        
        tier = manifest_data.get("tier", "Tier 1")
        mmproj_file = manifest_data.get("mmproj_filename")
        is_drafter = "dspark" in model_id.lower() or "dflash" in model_id.lower() or "draft" in model_id.lower()

        runtime_info = build_recommended_runtime(
            tier=tier,
            arch=arch,
            context_length=ctx,
            mmproj_filename=mmproj_file,
            is_drafter=is_drafter
        )

        # Atualiza o manifesto de forma completa e canônica
        manifest_data["architecture"] = arch
        manifest_data["context_length"] = ctx
        manifest_data["block_count"] = block_count
        manifest_data["embedding_length"] = embedding_length
        manifest_data["head_count"] = head_count
        manifest_data["head_count_kv"] = head_count_kv
        manifest_data["feed_forward_length"] = feed_forward_length
        manifest_data["bos_token_id"] = bos_id
        manifest_data["eos_token_id"] = eos_id
        manifest_data["chat_format"] = chat_format_info["format_name"]
        manifest_data["has_embedded_chat_template"] = chat_format_info["has_embedded_template"]
        manifest_data["chat_markers"] = {
            "system_prefix": chat_format_info["system_prefix"],
            "system_suffix": chat_format_info["system_suffix"],
            "user_prefix": chat_format_info["user_prefix"],
            "user_suffix": chat_format_info["user_suffix"],
            "assistant_prefix": chat_format_info["assistant_prefix"],
            "assistant_suffix": chat_format_info["assistant_suffix"],
            "stop_tokens": chat_format_info["stop_tokens"]
        }
        if chat_template:
            manifest_data["chat_template_jinja"] = chat_template

        manifest_data["recommended_runtime"] = runtime_info
        manifest_data["enriched_at"] = datetime.utcnow().isoformat() + "Z"

        # Salva o manifesto atualizado com formatação limpa
        with open(m_path, "w", encoding="utf-8") as f:
            json.dump(manifest_data, f, indent=2, ensure_ascii=False)

        updated += 1
        print(f" [✓] {tier:<8} | {model_id[:35]:<35} | arch: {arch:<10} | ctx: {ctx:<7} | fmt: {chat_format_info['format_name']}")

    print("\n" + "=" * 110)
    print(f" Enriquecimento Concluído: {updated} manifestos atualizados com sucesso, {errors} erros.")
    print("=" * 110)
    return updated, errors

if __name__ == "__main__":
    enrich_all_manifests(BASE_MODELS_DIR)
