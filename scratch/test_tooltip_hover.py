import asyncio, json, urllib.request, base64, subprocess, os, time
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9225

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
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
            await send_cmd("DOM.enable")

            # Switch to Settings tab and reasoning subtab
            await send_cmd("Runtime.evaluate", {
                "expression": """
                (function() {
                    switchTab('tab-settings');
                    switchSettingsSubtab('subtab-reasoning');
                })()
                """
            })
            
            await asyncio.sleep(2)

            # Trigger mouseover on the first info-icon
            eval_res = await send_cmd("Runtime.evaluate", {
                "expression": """
                (function() {
                    const icon = document.querySelector('#subtab-reasoning .info-icon');
                    if (icon) {
                        const evt = new MouseEvent('mouseover', { bubbles: true, cancelable: true, view: window });
                        icon.dispatchEvent(evt);
                        const tip = document.getElementById('global-portal-tooltip');
                        return {
                            found: true,
                            tipDisplay: tip ? tip.style.display : null,
                            tipOpacity: tip ? window.getComputedStyle(tip).opacity : null,
                            tipLeft: tip ? tip.style.left : null,
                            tipTop: tip ? tip.style.top : null,
                            text: tip ? tip.innerText.substring(0, 40) : null
                        };
                    }
                    return { found: false };
                })()
                """,
                "returnByValue": True
            })
            print("Hover test eval result:", json.dumps(eval_res.get("result", {}).get("value", {}), indent=2))
    finally:
        proc.terminate()

asyncio.run(main())

