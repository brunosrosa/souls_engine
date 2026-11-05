import asyncio, json, urllib.request, subprocess, time
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9265
user_dir = r"Z:\souls_engine\.souls_data\chrome_cdp_tmp_debug_raw"

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
    f"--user-data-dir={user_dir}",
    "--window-size=1440,1650",
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

            # Switch to Settings tab and subtab-raw
            eval_res = await send_cmd("Runtime.evaluate", {
                "expression": """
                    (() => {
                        switchTab('tab-settings');
                        switchSettingsSubtab('subtab-raw');
                        const raw = document.getElementById('subtab-raw');
                        const sidebar = document.querySelector('.settings-sidebar');
                        const main = document.querySelector('.settings-content-area');
                        const layout = document.querySelector('.settings-layout');
                        
                        function getRect(el) {
                            if (!el) return null;
                            const r = el.getBoundingClientRect();
                            return { top: r.top, left: r.left, width: r.width, height: r.height };
                        }
                        
                        return {
                            rawParentTag: raw ? raw.parentElement.tagName : null,
                            rawParentClass: raw ? raw.parentElement.className : null,
                            rawRect: getRect(raw),
                            sidebarRect: getRect(sidebar),
                            mainRect: getRect(main),
                            layoutRect: getRect(layout)
                        };
                    })()
                """,
                "returnByValue": True
            })
            print("DOM debug result:\n", json.dumps(eval_res, indent=2))
    finally:
        proc.terminate()

asyncio.run(main())

