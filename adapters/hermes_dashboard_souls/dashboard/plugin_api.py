"""
FastAPI router and reverse proxy for the Souls Engine Dashboard plugin in Hermes Agent.
"""

import httpx
from fastapi import APIRouter, Response, HTTPException
from fastapi.responses import HTMLResponse

router = APIRouter(prefix="/api/plugins/souls_dashboard", tags=["Souls Engine"])

SOULS_SERVER_URL = "http://127.0.0.1:9123"

@router.get("/status")
async def get_status():
    """Checks whether the Souls Engine daemon is alive."""
    try:
        async with httpx.AsyncClient(timeout=2.0) as client:
            resp = await client.get(f"{SOULS_SERVER_URL}/health")
            if resp.status_code == 200:
                return {"daemon_alive": True, "data": resp.json()}
    except Exception as e:
        return {"daemon_alive": False, "error": str(e)}
    return {"daemon_alive": False}

@router.get("/telemetry")
async def proxy_telemetry():
    """Proxies silicon telemetry from Souls Engine."""
    try:
        async with httpx.AsyncClient(timeout=3.0) as client:
            resp = await client.get(f"{SOULS_SERVER_URL}/api/v1/telemetry")
            return resp.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Souls daemon unavailable: {e}")

@router.get("/dashboard_view", response_class=HTMLResponse)
async def get_dashboard_view():
    """Returns the dashboard web interface."""
    try:
        async with httpx.AsyncClient(timeout=3.0) as client:
            resp = await client.get(f"{SOULS_SERVER_URL}/dashboard")
            return HTMLResponse(content=resp.text, status_code=200)
    except Exception as e:
        return HTMLResponse(
            content=f"<h1>Souls Engine Offline</h1><p>Ensure souls_server.exe is running on port 9123. Error: {e}</p>",
            status_code=503,
        )
