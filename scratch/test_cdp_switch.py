import asyncio, json, urllib.request, websockets

port = 9246

async def test():
    resp = urllib.request.urlopen(f"http://127.0.0.1:{port}/json")
    tabs = json.loads(resp.read().decode())
    ws_url = tabs[0]["webSocketDebuggerUrl"]

    async with websockets.connect(ws_url) as ws:
        msg_id = 0
        async def send(method, params=None):
            nonlocal msg_id
            msg_id += 1
            await ws.send(json.dumps({"id": msg_id, "method": method, "params": params or {}}))
            while True:
                res = json.loads(await ws.recv())
                if res.get("id") == msg_id:
                    return res

        res = await send("Runtime.evaluate", {
            "expression": """
                (() => {
                    switchTab('tab-settings');
                    switchSettingsSubtab('subtab-sandbox');
                    return {
                        settingsPane: document.getElementById('tab-settings').className,
                        sandboxPane: document.getElementById('subtab-sandbox') ? document.getElementById('subtab-sandbox').className : 'not-found',
                        sandboxBtn: document.getElementById('btn-subtab-sandbox') ? document.getElementById('btn-subtab-sandbox').className : 'not-found'
                    };
                })()
            """,
            "returnByValue": True
        })
        print(res["result"]["result"]["value"])

asyncio.run(test())

