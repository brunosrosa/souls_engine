import urllib.request, json, re

req = urllib.request.urlopen('http://127.0.0.1:9123/api/v1/inventory/search?query=Qwen+3.8&limit=30')
data = json.loads(req.read().decode('utf-8'))
results = data['results']

def parse_repo_model_details(repo_id, tags):
    r = repo_id.lower()
    tags_str = ' '.join(tags).lower() if tags else ''
    total_b = None
    active_b = None
    is_moe = False
    
    moe_match = re.search(r'(\d+)\s*b\s*[-_]a(\d+(?:\.\d+)?)\s*b', r)
    if moe_match:
        total_b = float(moe_match.group(1))
        active_b = float(moe_match.group(2))
        is_moe = True
    elif re.search(r'(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b', r):
        mx = re.search(r'(\d+)\s*x\s*(\d+(?:\.\d+)?)\s*b', r)
        total_b = int(mx.group(1)) * float(mx.group(2))
        active_b = float(mx.group(2))
        is_moe = True
    elif re.search(r'(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)', r):
        m = re.search(r'(?:^|[^0-9a-z])(\d+)\s*m(?:[^0-9a-z]|$)', r)
        total_b = int(m.group(1)) / 1000.0
        active_b = total_b
    else:
        b_matches = list(re.finditer(r'(?:^|[^0-9a-z])(\d+(?:\.\d+)?)\s*b(?:[^0-9a-z]|$)', r))
        if b_matches:
            total_b = float(b_matches[-1].group(1))
            active_b = total_b
        elif 'phi-4-mini' in r or 'phi-3.5-mini' in r:
            total_b = 3.8
            active_b = 3.8
        elif 'moe' in tags_str or 'mixture-of-experts' in tags_str or 'moe' in r:
            is_moe = True
    return total_b, active_b, is_moe

for res in results:
    t, a, moe = parse_repo_model_details(res['repo_id'], res.get('tags', []))
    print(f"{res['repo_id']} -> total_b={t}, active_b={a}, is_moe={moe}")
