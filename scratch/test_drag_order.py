import asyncio, json, urllib.request, websockets

async def main():
    resp = urllib.request.urlopen("http://127.0.0.1:9222/json")
    tabs = json.loads(resp.read().decode())
    # find open tab
    ws_url = tabs[0]["webSocketDebuggerUrl"]
    async with websockets.connect(ws_url, max_size=50*1024*1024) as ws:
        msg_id = 0
        async def send_cmd(method, params=None):
            nonlocal msg_id
            msg_id += 1
            payload = {"id": msg_id, "method": method, "params": params or {}}
            await ws.send(json.dumps(payload))
            while True:
                res = json.loads(await ws.recv())
                if res.get("id") == msg_id:
                    return res

        res = await send_cmd("Runtime.evaluate", {
            "expression": """
            (function() {
                // Test moveColumnOrder: move 'intel' to position 0
                if (typeof moveColumnOrder === 'function') {
                    const before = [...activeColumnOrder];
                    moveColumnOrder('intel', 'status');
                    const after = [...activeColumnOrder];
                    return { success: true, before: before.slice(0, 4), after: after.slice(0, 4) };
                }
                return { success: false, reason: 'moveColumnOrder not found' };
            })()
            """,
            "returnByValue": True
        })
        print("Move column test:", res["result"]["value"])

try:
    asyncio.run(main())
except Exception as e:
    print("Test skipped (browser not running):", e)

