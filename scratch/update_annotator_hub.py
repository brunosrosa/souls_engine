import base64

# Read fresh model_hub_tab.png
with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\model_hub_tab.png", "rb") as f:
    hub_bytes = f.read()

hub_b64 = base64.b64encode(hub_bytes).decode("utf-8")
print(f"Read model_hub_tab.png: {len(hub_bytes)} bytes, base64 length: {len(hub_b64)}")

# Read subagents_annotator.html
with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    # Update title
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: ✨ Model Hub & Downloads — Souls Engine v7</title>\n"
    # Update subtitle
    if "Marque pontos" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da sub-aba ✨ Model Hub & Downloads</p>\n'
    # Update image
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{hub_b64}";\n'
    # Update storage key
    if "souls_silicon_arena_annotator_notes" in line:
        line = line.replace("souls_silicon_arena_annotator_notes", "souls_model_hub_annotator_notes")
    # Update download filename
    if "silicon_arena_annotated.png" in line:
        line = line.replace("silicon_arena_annotated.png", "model_hub_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Model Hub & Downloads screenshot!")

