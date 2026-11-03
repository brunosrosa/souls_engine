import base64

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\settings_tab.png", "rb") as f:
    settings_bytes = f.read()

settings_b64 = base64.b64encode(settings_bytes).decode("utf-8")
print(f"Read settings_tab.png: {len(settings_bytes)} bytes, base64 length: {len(settings_b64)}")

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: ⚙️ Settings (Master Brain & Tríades) — Souls Engine v7</title>\n"
    if "Marque pontos" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da aba ⚙️ Settings (Item 1: Master Brain & Tríades)</p>\n'
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{settings_b64}";\n'
    if "souls_cloud_arena_annotator_notes" in line or "souls_switchboard_mcp_annotator_notes" in line:
        line = line.replace("souls_cloud_arena_annotator_notes", "souls_settings_tab_annotator_notes").replace("souls_switchboard_mcp_annotator_notes", "souls_settings_tab_annotator_notes")
    if "cloud_arena_annotated.png" in line or "switchboard_mcp_annotated.png" in line:
        line = line.replace("cloud_arena_annotated.png", "settings_tab_annotated.png").replace("switchboard_mcp_annotated.png", "settings_tab_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Settings screenshot!")

