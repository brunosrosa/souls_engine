import base64

# Read fresh arena_tab.png
with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\arena_tab.png", "rb") as f:
    arena_bytes = f.read()

arena_b64 = base64.b64encode(arena_bytes).decode("utf-8")
print(f"Read arena_tab.png: {len(arena_bytes)} bytes, base64 length: {len(arena_b64)}")

# Read subagents_annotator.html
with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    # Update title
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: 🏟️ Local Silicon Arena — Souls Engine v7</title>\n"
    # Update subtitle
    if "Marque pontos" in line and "Model Hub" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da sub-aba 🏟️ Local Silicon Arena</p>\n'
    # Update image
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{arena_b64}";\n'
    # Update storage key
    if "souls_model_hub_annotator_notes" in line:
        line = line.replace("souls_model_hub_annotator_notes", "souls_silicon_arena_annotator_notes")
    # Update download filename
    if "model_hub_annotated.png" in line:
        line = line.replace("model_hub_annotated.png", "silicon_arena_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Local Silicon Arena screenshot!")

