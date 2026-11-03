import asyncio, json, urllib.request, base64, subprocess, os, time
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9222

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
    "--window-size=1440,3200",
    "--hide-scrollbars",
    "--disable-gpu",
    "http://127.0.0.1:9123"
])

print("Chrome launched, waiting 3s...")
time.sleep(3)

async def main():
    try:
        resp = urllib.request.urlopen(f"http://127.0.0.1:{port}/json")
        tabs = json.loads(resp.read().decode())
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

            await send_cmd("Runtime.enable")
            await send_cmd("Page.enable")

            # Switch to models tab and hub subtab
            await send_cmd("Runtime.evaluate", {
                "expression": "switchTab('tab-models'); switchLocalSubtab('hub');",
                "returnByValue": True
            })
            await asyncio.sleep(2)

            # Check hf-search-input value
            eval_input = await send_cmd("Runtime.evaluate", {
                "expression": "document.getElementById('hf-search-input')?.value",
                "returnByValue": True
            })
            input_val = eval_input.get("result", {}).get("result", {}).get("value")
            print(f"HF Search Input value: '{input_val}' (should be empty)")

            # Click trending view mode to test "Novidades em Alta"
            await send_cmd("Runtime.evaluate", {
                "expression": "switchCuratedViewMode('trending');",
                "returnByValue": True
            })
            await asyncio.sleep(3.0)

            # Check what's in curated-models-grid
            eval_grid = await send_cmd("Runtime.evaluate", {
                "expression": "(() => { const el = document.getElementById('curated-models-grid'); return { cardsCount: el?.querySelectorAll('.curated-card')?.length, htmlSnippet: el?.innerText?.substring(0, 150) }; })()",
                "returnByValue": True
            })
            grid_info = eval_grid.get("result", {}).get("result", {}).get("value")
            print(f"Curated grid info in Trending mode: {grid_info}")

            # Check parameter filter pills text
            eval_pills = await send_cmd("Runtime.evaluate", {
                "expression": "(() => { const pills = Array.from(document.querySelectorAll('#hf-param-filters .profile-pill')).map(p => ({ id: p.id, text: p.innerText, color: p.style.color, border: p.style.borderColor })); return pills; })()",
                "returnByValue": True
            })
            pills_info = eval_pills.get("result", {}).get("result", {}).get("value")
            print(f"Parameter pills count: {len(pills_info)}")
            print("Parameter pills sample:", json.dumps(pills_info[:8], indent=2))

            # Check task filter pills text
            eval_tasks = await send_cmd("Runtime.evaluate", {
                "expression": "(() => { const pills = Array.from(document.querySelectorAll('#hf-task-filters .profile-pill')).map(p => ({ id: p.id, text: p.innerText })); return pills; })()",
                "returnByValue": True
            })
            tasks_info = eval_tasks.get("result", {}).get("result", {}).get("value")
            print(f"Task pills count: {len(tasks_info)}")
            print("Task pills:", json.dumps(tasks_info, indent=2))

            # Test toggle-off behavior on task filters
            await send_cmd("Runtime.evaluate", {
                "expression": "setHfTaskFilter('coding');",
                "returnByValue": True
            })
            eval_coding = await send_cmd("Runtime.evaluate", {
                "expression": "currentHfTaskFilter",
                "returnByValue": True
            })
            print("After setHfTaskFilter('coding'):", eval_coding.get("result", {}).get("result", {}).get("value"))

            # Click again to deselect
            await send_cmd("Runtime.evaluate", {
                "expression": "setHfTaskFilter('coding');",
                "returnByValue": True
            })
            eval_deselected = await send_cmd("Runtime.evaluate", {
                "expression": "currentHfTaskFilter",
                "returnByValue": True
            })
            print("After clicking 'coding' again (should be 'all'):", eval_deselected.get("result", {}).get("result", {}).get("value"))

            res = await send_cmd("Page.captureScreenshot", {"format": "png"})
            png_data = base64.b64decode(res["result"]["data"])

            out_path = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\model_hub_tab.png"
            with open(out_path, "wb") as f:
                f.write(png_data)
            print(f"Captured screenshot to {out_path} ({len(png_data)} bytes)")
    finally:
        proc.terminate()

asyncio.run(main())
