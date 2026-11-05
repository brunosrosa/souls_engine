import base64
import os
import re

png_path = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\settings_router_tab.png"
annotator_path = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html"

with open(png_path, "rb") as f:
    b64_data = base64.b64encode(f.read()).decode("utf-8")

data_uri = f"data:image/png;base64,{b64_data}"

with open(annotator_path, "r", encoding="utf-8") as f:
    html = f.read()

# Replace title and header
html = re.sub(
    r"<title>.*?</title>",
    "<title>Anotador Visual: ⚙️ Settings (Item 5: Roteador Inteligente) — Souls Engine v7</title>",
    html,
    count=1
)

html = re.sub(
    r'<h1 class="text-sm font-semibold tracking-wide flex items-center gap-2">.*?</h1>',
    '<h1 class="text-sm font-semibold tracking-wide flex items-center gap-2"><span>Anotador Visual: ⚙️ Settings (Item 5: Roteador Inteligente) — Souls Engine v7</span></h1>',
    html,
    count=1
)

# Replace b64Image
html = re.sub(
    r'const b64Image = "data:image/png;base64,[^"]+";',
    f'const b64Image = "{data_uri}";',
    html,
    count=1
)

# Reset pins array
html = re.sub(
    r'let pins = \[.*?\];',
    'let pins = [];',
    html,
    count=1,
    flags=re.DOTALL
)

with open(annotator_path, "w", encoding="utf-8") as f:
    f.write(html)

print("subagents_annotator.html successfully updated with Router screenshot!")

