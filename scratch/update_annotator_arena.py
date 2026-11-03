import base64, re

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    text = f.read()

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\arena_tab.png", "rb") as f:
    img_b64 = base64.b64encode(f.read()).decode("utf-8")

data_url = f"data:image/png;base64,{img_b64}"

# Update image src in annotator
new_text = re.sub(r'id="target-img"\s+src="[^"]+"', f'id="target-img" src="{data_url}"', text)
if 'id="target-img"' not in new_text:
    new_text = re.sub(r'<img\s+id="main-image"\s+src="[^"]+"', f'<img id="main-image" src="{data_url}"', text)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.write(new_text)

print("Updated subagents_annotator.html with latest Local Silicon Arena screenshot!")

