import base64

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\cloud_arena_tab.png", "rb") as f:
    cloud_bytes = f.read()

cloud_b64 = base64.b64encode(cloud_bytes).decode("utf-8")
print(f"Read cloud_arena_tab.png: {len(cloud_bytes)} bytes, base64 length: {len(cloud_b64)}")

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: 🌐 Cloud Arena — Souls Engine v7</title>\n"
    if "Marque pontos" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da aba 🌐 Cloud Arena</p>\n'
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{cloud_b64}";\n'
    if "souls_switchboard_mcp_annotator_notes" in line or "souls_model_hub_annotator_notes" in line:
        line = line.replace("souls_switchboard_mcp_annotator_notes", "souls_cloud_arena_annotator_notes").replace("souls_model_hub_annotator_notes", "souls_cloud_arena_annotator_notes")
    if "switchboard_mcp_annotated.png" in line or "model_hub_annotated.png" in line:
        line = line.replace("switchboard_mcp_annotated.png", "cloud_arena_annotated.png").replace("model_hub_annotated.png", "cloud_arena_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Cloud Arena screenshot!")

