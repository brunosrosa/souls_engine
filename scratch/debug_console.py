import urllib.request
import json
import subprocess
import time
import asyncio
import websockets

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9250
user_dir = r"Z:\souls_engine\.souls_data\chrome_debug_errors"

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
                    if "method" in res and "exception" in res.get("method", "").lower():
                        print("EXCEPTION EVENT:", res)
                    if "method" in res and "console" in res.get("method", "").lower():
                        print("CONSOLE EVENT:", res)

            await send_cmd("Runtime.enable")
            await send_cmd("Page.enable")
            await send_cmd("Log.enable")

            # Check for JS errors by reloading
            reload_res = await send_cmd("Page.reload")
            print("Reloaded page")

            # Read any events for 3 seconds
            start = time.time()
            while time.time() - start < 3:
                try:
                    raw = await asyncio.wait_for(ws.recv(), timeout=1.0)
                    evt = json.loads(raw)
                    if "params" in evt and "exceptionDetails" in evt["params"]:
                        print("EXCEPTION DETAILS:", evt["params"]["exceptionDetails"])
                    elif "method" in evt and "entry" in evt.get("params", {}):
                        print("LOG ENTRY:", evt["params"]["entry"])
                    elif "method" in evt and "console" in evt.get("method", ""):
                        print("CONSOLE:", evt)
                except asyncio.TimeoutError:
                    pass

            # Check what scripts are loaded
            chk = await send_cmd("Runtime.evaluate", {"expression": "window.errors || 'no errors array'"})
            print("Eval result:", chk)
    finally:
        proc.terminate()

asyncio.run(main())

