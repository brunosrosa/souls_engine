import asyncio, json, urllib.request, base64, subprocess, os, time
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9248
user_dir = r"Z:\souls_engine\.souls_data\chrome_cdp_tmp_obsidian"

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

            # Switch to Settings tab and subtab-obsidian
            eval_res = await send_cmd("Runtime.evaluate", {
                "expression": """
                    (() => {
                        switchTab('tab-settings');
                        switchSettingsSubtab('subtab-obsidian');
                        const pane = document.getElementById('subtab-obsidian');
                        const btn = document.getElementById('btn-subtab-obsidian');
                        return {
                            hasPane: !!pane,
                            paneClass: pane ? pane.className : null,
                            hasBtn: !!btn,
                            btnClass: btn ? btn.className : null
                        };
                    })()
                """,
                "returnByValue": True
            })
            print("Evaluate result:", json.dumps(eval_res, indent=2))

            # Wait for DOM and subtab render
            await asyncio.sleep(2)

            res = await send_cmd("Page.captureScreenshot", {"format": "png"})
            png_data = base64.b64decode(res["result"]["data"])

            out_path = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\settings_obsidian_tab.png"
            with open(out_path, "wb") as f:
                f.write(png_data)
            print(f"Captured obsidian screenshot to {out_path} ({len(png_data)} bytes)")
    finally:
        proc.terminate()

asyncio.run(main())

