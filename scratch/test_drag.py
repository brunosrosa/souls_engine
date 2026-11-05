import asyncio, json, urllib.request, base64, subprocess, os, time
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9241
user_dir = r"Z:\souls_engine\.souls_data\chrome_cdp_tmp_drag_test"

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
    f"--user-data-dir={user_dir}",
    "--window-size=1440,1600",
    "--hide-scrollbars",
    "--disable-gpu",
    "http://127.0.0.1:9123/#tab-settings/subtab-router"
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
            await asyncio.sleep(2)

            # Change local-token-limit to 4096 to see tick mark separate from thumb
            await send_cmd("Runtime.evaluate", {
                "expression": """
                const slider = document.getElementById("cfg-local-token-limit");
                slider.value = 4096;
                slider.dispatchEvent(new Event("input", { bubbles: true }));
                """
            })
            await asyncio.sleep(1)

            res = await send_cmd("Page.captureScreenshot", {"format": "png"})
            png_data = base64.b64decode(res["result"]["data"])
            out_path = r"Z:\souls_engine\scratch\test_drag_slider.png"
            with open(out_path, "wb") as f:
                f.write(png_data)
            print(f"Captured moved slider test to {out_path}")
    finally:
        proc.terminate()

asyncio.run(main())
