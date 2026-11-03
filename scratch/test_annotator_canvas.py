import urllib.request, json, asyncio, websockets, subprocess, time

CHROME_PATH = r"C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe"
port = 9223
annotator_url = r"file:///C:/Users/rosas/.gemini/antigravity/brain/21d96846-20af-4e94-97c9-db9237aa802f/subagents_annotator.html"

proc = subprocess.Popen([
    CHROME_PATH,
    "--headless=new",
    f"--remote-debugging-port={port}",
    "--window-size=1440,900",
    annotator_url
])

time.sleep(2)

async def check():
    try:
        resp = urllib.request.urlopen(f"http://127.0.0.1:{port}/json")
        tabs = json.loads(resp.read().decode())
        ws_url = tabs[0]["webSocketDebuggerUrl"]
        async with websockets.connect(ws_url) as ws:
            msg_id = 0
            async def cmd(m, p=None):
                nonlocal msg_id
                msg_id += 1
                await ws.send(json.dumps({"id": msg_id, "method": m, "params": p or {}}))
                while True:
                    res = json.loads(await ws.recv())
                    if res.get("id") == msg_id:
                        return res

            await cmd("Runtime.enable")
            await asyncio.sleep(1)
            eval_res = await cmd("Runtime.evaluate", {
                "expression": "canvas.width + 'x' + canvas.height"
            })
            print("Canvas dimensions in annotator:", eval_res.get("result", {}).get("value"))
    finally:
        proc.terminate()

asyncio.run(check())

