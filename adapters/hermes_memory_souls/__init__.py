"""
Hermes Memory Souls Adapter (v7)
Conecta o Hermes Agent diretamente ao hipocampo bare-metal L2 do Souls Engine
via endpoints locais Zero-Trust Loopback (http://127.0.0.1:9123).
"""

import os
import json
import urllib.request
import urllib.error

SOULS_ENGINE_ENDPOINT = os.getenv("SOULS_ENGINE_URL", "http://127.0.0.1:9123")

class SoulsMemoryAdapter:
    def __init__(self, base_url: str = SOULS_ENGINE_ENDPOINT):
        self.base_url = base_url.rstrip("/")

    def search_memory(self, query: str, limit: int = 5):
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "mem_search",
                "arguments": {"query": query, "limit": limit}
            }
        }
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self.base_url}/mcp",
            data=data,
            headers={"Content-Type": "application/json"}
        )
        try:
            with urllib.request.urlopen(req, timeout=5) as resp:
                result = json.loads(resp.read().decode("utf-8"))
                return result.get("result", {})
        except Exception as e:
            return {"error": str(e), "isError": True}

adapter = SoulsMemoryAdapter()
