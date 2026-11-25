#!/usr/bin/env python3
"""Zero-cost pre-flight probe for subagent dispatch.

Checks only local Souls Engine endpoints (never a paid model). Run before any
delegate_task fan-out.

    python scripts/preflight_probe.py
    python scripts/preflight_probe.py --smoke-model <local-model-id>

Exit 0 = gate ready; 1 = one or more checks failed (do not dispatch).
"""
import argparse
import json
import sys
import urllib.request

MCP = "http://127.0.0.1:9123/mcp"
HEALTH = "http://127.0.0.1:9123/health"
MODELS = "http://127.0.0.1:9123/v1/models"
COMPLETIONS = "http://127.0.0.1:9123/v1/chat/completions"


def http_json(url, payload=None, timeout=6):
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json, text/event-stream",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode())


def dispatch(tool, args, timeout=6):
    return http_json(
        MCP,
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "tool_dispatch",
                "arguments": {"tool": tool, "arguments": args},
            },
        },
        timeout,
    )


def check(label, fn):
    try:
        ok, detail = fn()
    except Exception as exc:
        ok, detail = False, "%s: %s" % (type(exc).__name__, str(exc)[:160])
    print("[%s] %s - %s" % ("OK" if ok else "FAIL", label, detail))
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--chunk-tokens", type=int, default=4000)
    ap.add_argument(
        "--smoke-model",
        default=None,
        help="optional local model id for a tiny zero-cost inference",
    )
    args = ap.parse_args()
    results = []

    def health():
        d = http_json(HEALTH)
        hw = d.get("hardware", {})
        zone = hw.get("operational_zone", "?")
        return zone == "Nominal", "zone=%s vram_free=%sMB usable=%sMB temp=%sC" % (
            zone,
            hw.get("vram_free_mb", "?"),
            hw.get("vram_usable_mb", "?"),
            hw.get("gpu_temp_c", "?"),
        )

    def route():
        r = dispatch(
            "route_subtask",
            {
                "task_type": "fast",
                "estimated_tokens": min(args.chunk_tokens, 4096),
                "latency_sensitivity": 0.5,
            },
        )
        sc = r.get("result", {}).get("structuredContent", {})
        chain = sc.get("fallback_chain", {}).get("fallbacks", [])
        costs = [c.get("projected_cost_usd") for c in chain]
        paid = [c for c in costs if c is not None and c > 0]
        model = sc.get("model_id") or sc.get("prescribed_model", "?")
        ctx = sc.get("active_context_window_tokens", "?")
        return not paid, "model=%s ctx=%s costs=%s" % (model, ctx, costs[:3])

    def models():
        d = http_json(MODELS)
        ids = [m["id"] for m in d.get("data", [])]
        return bool(ids), "%d local models" % len(ids)

    results.append(check("daemon health", health))
    results.append(check("route_subtask cost gate", route))
    results.append(check("local model catalog", models))

    if args.smoke_model:
        def smoke():
            r = http_json(
                COMPLETIONS,
                {
                    "model": args.smoke_model,
                    "messages": [
                        {"role": "system", "content": "Reply with exactly: OK"},
                        {"role": "user", "content": "OK"},
                    ],
                    "max_tokens": 16,
                    "temperature": 0.0,
                },
                timeout=120,
            )
            out = r["choices"][0]["message"]["content"]
            leaked = "<think>" in out.lower()
            return not leaked, "output=%r think_leak=%s" % (out[:60], leaked)

        results.append(check("local smoke inference", smoke))

    print("\nVERDICT: %s" % ("READY" if all(results) else "NOT READY - do not dispatch"))
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
