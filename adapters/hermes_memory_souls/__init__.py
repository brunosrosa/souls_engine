"""
Hermes Memory Souls Adapter & Native MemoryProvider (v7.2)
Conecta o Hermes Agent simbioticamente ao Hipocampo Bare-Metal L3 do Souls Engine
via Loopback Zero-Trust (http://127.0.0.1:9123).

Cumpre estritamente a Cláusula 4 do AGENTS.md:
- Leitura passiva de MEMORY.md, USER.md e SOUL.md (zero escrita direta em disco).
- FrozenMemorySnapshot imutável por session_id (100% prefix cache preservation).
- Particionamento epistêmico: STABLE (salience=1.0, imune ao Langevin Decay) vs EVOLVING.
- Recall Adaptativo ("Terceira Via"): alterna automaticamente entre Lean (top_k=4, 800 chars)
  e Expanded (top_k=8, 2400 chars + Grafo Ontológico LadybugDB) conforme a complexidade do turno.
- Proteção automática de PII via PiiRedactor local antes de injetar contexto em modelos de nuvem.
"""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional

SOULS_ENGINE_ENDPOINT = os.getenv("SOULS_ENGINE_URL", "http://127.0.0.1:9123")
DEFAULT_RECALL_MODE = os.getenv("SOULS_RECALL_MODE", "adaptive")

try:
    from agent.memory_provider import MemoryProvider, RecallStatus  # type: ignore
except Exception:
    class RecallStatus:  # type: ignore[no-redef]
        def __init__(self, provider_label: str, count: int, glyph: str = "⚡") -> None:
            self.provider_label = provider_label
            self.count = count
            self.glyph = glyph

    class MemoryProvider:  # type: ignore[no-redef]
        name: str = "base"


class SoulsMemoryProvider(MemoryProvider):
    """Provedor nativo MemoryProvider do Hermes Agent acoplado ao Souls Engine L3."""

    name = "souls_memory"
    pre_compress_checkpoint_api_version = 2

    def __init__(
        self,
        base_url: str = SOULS_ENGINE_ENDPOINT,
        recall_mode: str = DEFAULT_RECALL_MODE,
        redact_pii: bool = True,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.recall_mode = recall_mode
        self.redact_pii = redact_pii
        self.current_session_id: str = "default"
        self._snapshot_meta: Dict[str, Any] = {}
        self._queued_query: Optional[str] = None
        self._last_recall_count: int = 0
        self._last_mode_used: str = recall_mode

    def _http_json(
        self,
        method: str,
        path: str,
        payload: Optional[Dict[str, Any]] = None,
        timeout: float = 3.0,
    ) -> Dict[str, Any]:
        url = f"{self.base_url}{path}"
        data = json.dumps(payload).encode("utf-8") if payload is not None else None
        headers = {"Content-Type": "application/json"} if data is not None else {}
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                raw = resp.read().decode("utf-8")
                return json.loads(raw) if raw else {"status": "ok"}
        except Exception as exc:
            return {"status": "error", "message": str(exc)}

    def is_available(self) -> bool:
        res = self._http_json("GET", "/health", timeout=1.2)
        return res.get("status") in ("ok", "healthy", "success") or "version" in res

    def initialize(self, session_id: str = "default", **kwargs: Any) -> None:
        self.current_session_id = session_id or "default"
        force_refresh = bool(kwargs.get("force_refresh", False))
        hermes_home = kwargs.get("hermes_home")
        payload: Dict[str, Any] = {
            "session_id": self.current_session_id,
            "force_refresh": force_refresh,
        }
        if hermes_home:
            payload["hermes_home"] = str(hermes_home)

        res = self._http_json("POST", "/api/v1/memory/snapshot/pin", payload, timeout=3.5)
        if res.get("status") == "success":
            self._snapshot_meta = res.get("snapshot") or {}

    def system_prompt_block(self) -> str:
        """Retorna metadados enxutos e determinísticos do FrozenMemorySnapshot.
        Não repete o texto integral de MEMORY.md/USER.md (que o Hermes já injeta),
        garantindo 100% de preservação do prefix cache e zero duplicação.
        """
        if not self._snapshot_meta:
            res = self._http_json(
                "GET",
                f"/api/v1/memory/snapshot?session_id={self.current_session_id}",
                timeout=1.5,
            )
            if res.get("status") == "success":
                self._snapshot_meta = res.get("snapshot") or {}

        snap_id = self._snapshot_meta.get("snapshot_id", "unpinned")
        comp_sha = str(self._snapshot_meta.get("composite_sha256", ""))[:16]
        stable_cnt = self._snapshot_meta.get("stable_truths_count", 0)
        evolving_cnt = self._snapshot_meta.get("evolving_count", 0)
        return (
            f"<!-- souls-l3-snapshot id={snap_id} sha256={comp_sha} "
            f"stable={stable_cnt} evolving={evolving_cnt} recall_mode={self.recall_mode} -->"
        )

    def queue_prefetch(self, query: str, *, session_id: str = "") -> None:
        self._queued_query = query
        if session_id:
            self.current_session_id = session_id

    def prefetch(self, query: str = "", *, session_id: str = "") -> str:
        effective_query = (query or self._queued_query or "").strip()
        self._queued_query = None
        if not effective_query:
            self._last_recall_count = 0
            return ""

        sid = session_id or self.current_session_id or "default"
        payload = {
            "query": effective_query,
            "session_id": sid,
            "mode": self.recall_mode,
            "redact_pii": self.redact_pii,
        }
        res = self._http_json("POST", "/api/v1/memory/recall", payload, timeout=2.5)
        if res.get("status") != "success":
            self._last_recall_count = 0
            return ""

        self._last_recall_count = int(res.get("count", 0))
        self._last_mode_used = str(res.get("mode_used", self.recall_mode))
        return str(res.get("formatted_block", ""))

    def recall_status(self) -> Optional[RecallStatus]:
        if self._last_recall_count <= 0:
            return None
        label = f"Souls L3 ({self._last_mode_used})"
        return RecallStatus(provider_label=label, count=self._last_recall_count, glyph="⚡")

    def sync_turn(
        self,
        user_content: str,
        assistant_content: str,
        *,
        session_id: str = "",
    ) -> None:
        sid = session_id or self.current_session_id or "default"
        payload = {
            "session_id": sid,
            "user_content": user_content,
            "assistant_content": assistant_content,
            "category": "EVOLVING:turn_insight",
            "salience": 0.75,
        }
        self._http_json("POST", "/api/v1/memory/turn", payload, timeout=2.5)

    def on_memory_write(
        self,
        action: str,
        target: str,
        content: str,
        **kwargs: Any,
    ) -> None:
        """Espelha mutações da ferramenta built-in `memory` do Hermes na partição STABLE."""
        sid = str(kwargs.get("session_id") or self.current_session_id or "default")
        old_text = kwargs.get("old_text")
        payload: Dict[str, Any] = {
            "action": action,
            "target": target,
            "content": content,
            "session_id": sid,
        }
        if old_text is not None:
            payload["old_text"] = str(old_text)
        self._http_json("POST", "/api/v1/memory/mirror", payload, timeout=2.5)

    def on_session_switch(self, old_session_id: str, new_session_id: str) -> None:
        """Rotaciona o FrozenMemorySnapshot exclusivamente na fronteira de troca de sessão (/new ou /reset)."""
        self.current_session_id = new_session_id or "default"
        self.initialize(session_id=self.current_session_id, force_refresh=True)

    def on_pre_compress(
        self,
        messages: List[Dict[str, Any]],
        *,
        session_id: str = "",
        pre_compress_tokens: int = 0,
        **kwargs: Any,
    ) -> str:
        """Persiste checkpoint Audit Log v2 antes da compressão de contexto do Hermes."""
        sid = session_id or self.current_session_id or "default"
        payload = {
            "session_id": sid,
            "messages": messages,
            "pre_compress_tokens": int(pre_compress_tokens),
        }
        res = self._http_json("POST", "/api/v1/memory/checkpoint", payload, timeout=4.0)
        if res.get("status") == "success":
            return str(res.get("summary_preview", ""))
        return ""

    def on_delegation(
        self,
        task: str,
        result: str,
        *,
        child_session_id: str = "",
        **kwargs: Any,
    ) -> None:
        """Registra síntese de subagente delegado na partição EVOLVING."""
        sid = child_session_id or self.current_session_id or "default"
        payload = {
            "session_id": sid,
            "user_content": f"[Subagent Task] {task}",
            "assistant_content": result,
            "category": "EVOLVING:subagent_delegation",
            "salience": 0.80,
        }
        self._http_json("POST", "/api/v1/memory/turn", payload, timeout=2.5)

    def get_tool_schemas(self) -> List[Dict[str, Any]]:
        return [
            {
                "name": "souls_memory_recall_mode",
                "description": (
                    "Configures or inspects the Souls Engine L3 Recall Mode ('adaptive' Terceira Via, "
                    "'lean' 800-char focus, or 'expanded' 2400-char + LadybugDB Ontology Graph) and "
                    "optionally refreshes the session's FrozenMemorySnapshot."
                ),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["adaptive", "lean", "expanded"],
                            "description": "Recall policy mode.",
                        },
                        "refresh_snapshot": {
                            "type": "boolean",
                            "description": "If true, forces a fresh passive read of MEMORY.md/USER.md/SOUL.md.",
                        },
                    },
                },
            }
        ]

    def handle_tool_call(self, tool_name: str, args: Dict[str, Any], **kwargs: Any) -> str:
        if tool_name == "souls_memory_recall_mode":
            new_mode = args.get("mode")
            if new_mode in ("adaptive", "lean", "expanded"):
                self.recall_mode = new_mode
            if args.get("refresh_snapshot"):
                self.initialize(session_id=self.current_session_id, force_refresh=True)
            return json.dumps(
                {
                    "status": "success",
                    "active_recall_mode": self.recall_mode,
                    "session_id": self.current_session_id,
                    "snapshot": self._snapshot_meta,
                },
                ensure_ascii=False,
            )
        return json.dumps({"status": "error", "message": f"Unknown tool: {tool_name}"})

    def shutdown(self) -> None:
        pass


class SoulsMemoryAdapter:
    """Adaptador compatível para chamadas diretas MCP e scripts de verificação."""

    def __init__(self, base_url: str = SOULS_ENGINE_ENDPOINT) -> None:
        self.base_url = base_url.rstrip("/")
        self.provider = SoulsMemoryProvider(base_url=self.base_url)

    def search_memory(self, query: str, limit: int = 5) -> Dict[str, Any]:
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "mem_search",
                "arguments": {"query": query, "limit": limit},
            },
        }
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self.base_url}/mcp",
            data=data,
            headers={"Content-Type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=5) as resp:
                result = json.loads(resp.read().decode("utf-8"))
                return result.get("result", {})
        except Exception as e:
            return {"error": str(e), "isError": True}


adapter = SoulsMemoryAdapter()
provider = adapter.provider


def register(ctx: Any) -> Optional[SoulsMemoryProvider]:
    """Hook oficial de descoberta do Hermes Agent (`plugins/memory/__init__.py`)."""
    instance = SoulsMemoryProvider()
    if hasattr(ctx, "register_memory_provider"):
        ctx.register_memory_provider(instance)
    return instance
