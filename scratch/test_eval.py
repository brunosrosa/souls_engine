import urllib.request
import json
import base64
import subprocess
import time
import asyncio
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9245
user_dir = r"Z:\souls_engine\.souls_data\chrome_test_eval"

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
    f"--user-data-dir={user_dir}",
    "--window-size=1440,1600",
    "--hide-scrollbars",
    "--disable-gpu",
    "http://127.0.0.1:9123"
])

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

            # Polling until window.switchTab exists
            for i in range(50):
                r = await send_cmd("Runtime.evaluate", {"expression": "typeof switchTab"})
                val = r.get("result", {}).get("result", {}).get("value")
                print(f"Attempt {i}: typeof switchTab = {val}")
                if val == "function":
                    break
                await asyncio.sleep(0.2)

            # Click Settings button directly in DOM
            res1 = await send_cmd("Runtime.evaluate", {
                "expression": """
                (function() {
                    const btn = document.getElementById('btn-tab-settings');
                    if (btn) {
                        btn.click();
                        return "Clicked btn-tab-settings";
                    }
                    return "btn-tab-settings NOT FOUND";
                })()
                """
            })
            print("Click Settings result:", res1)

            await asyncio.sleep(1)

            # Click subtab-router button directly in DOM
            res2 = await send_cmd("Runtime.evaluate", {
                "expression": """
                (function() {
                    const btn = document.getElementById('btn-subtab-router');
                    if (btn) {
                        btn.click();
                        return "Clicked btn-subtab-router";
                    }
                    return "btn-subtab-router NOT FOUND";
                })()
                """
            })
            print("Click subtab-router result:", res2)

            await asyncio.sleep(3)

            shot = await send_cmd("Page.captureScreenshot", {"format": "png"})
            png_data = base64.b64decode(shot["result"]["data"])

            out_path = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\settings_router_tab.png"
            with open(out_path, "wb") as f:
                f.write(png_data)
            print(f"Saved {len(png_data)} bytes to {out_path}")
    finally:
        proc.terminate()

asyncio.run(main())

