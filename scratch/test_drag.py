import asyncio, json, urllib.request, subprocess, time, websockets

proc = subprocess.Popen([
    r'C:\Users\rosas\AppData\Local\Google\Chrome\Application\chrome.exe',
    '--headless=new',
    '--remote-debugging-port=9223',
    '--disable-gpu',
    'http://127.0.0.1:9123'
])
time.sleep(3)

async def test():
    try:
        resp = urllib.request.urlopen('http://127.0.0.1:9223/json')
        tabs = json.loads(resp.read().decode())
        ws_url = tabs[0]['webSocketDebuggerUrl']
        async with websockets.connect(ws_url) as ws:
            msg_id = 0
            async def cmd(m, p=None):
                nonlocal msg_id
                msg_id += 1
                await ws.send(json.dumps({'id': msg_id, 'method': m, 'params': p or {}}))
                while True:
                    r = json.loads(await ws.recv())
                    if r.get('id') == msg_id: return r

            await cmd('Runtime.enable')
            await cmd('Runtime.evaluate', {'expression': 'switchTab("tab-cloud")'})
            await asyncio.sleep(2)
            res = await cmd('Runtime.evaluate', {
                'expression': '''
                (function() {
                    const before = [...activeColumnOrder];
                    moveColumnOrder('intel', 'status');
                    const after = [...activeColumnOrder];
                    const theadThs = Array.from(document.querySelectorAll('#cloud-ranked-thead th')).map(th => th.textContent.trim().replace(/\\s+/g, ' '));
                    return JSON.stringify({ before: before.slice(0, 3), after: after.slice(0, 3), theadFirstCols: theadThs.slice(0, 5) });
                })()
                ''',
                'returnByValue': True
            })
            print('Full res:', res)
    finally:
        proc.terminate()

asyncio.run(test())
