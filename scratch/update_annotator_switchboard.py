import base64

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\switchboard_tab.png", "rb") as f:
    sb_bytes = f.read()

sb_b64 = base64.b64encode(sb_bytes).decode("utf-8")
print(f"Read switchboard_tab.png: {len(sb_bytes)} bytes, base64 length: {len(sb_b64)}")

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: ⚡ Switchboard MCP — Souls Engine v7</title>\n"
    if "Marque pontos" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da sub-aba ⚡ Switchboard MCP</p>\n'
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{sb_b64}";\n'
    if "souls_model_hub_annotator_notes" in line:
        line = line.replace("souls_model_hub_annotator_notes", "souls_switchboard_mcp_annotator_notes")
    if "model_hub_annotated.png" in line:
        line = line.replace("model_hub_annotated.png", "switchboard_mcp_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Switchboard MCP screenshot!")

