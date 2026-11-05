import base64

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\settings_hardware_tab.png", "rb") as f:
    hw_bytes = f.read()

hw_b64 = base64.b64encode(hw_bytes).decode("utf-8")
print(f"Read settings_hardware_tab.png: {len(hw_bytes)} bytes, base64 length: {len(hw_b64)}")

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "r", encoding="utf-8") as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "<title>Anotador Visual:" in line:
        line = "  <title>Anotador Visual: ⚙️ Settings (Item 4: Aceleração Local) — Souls Engine v7</title>\n"
    if "Marque pontos" in line:
        line = '        <p class="text-[11px] text-slate-400">Marque pontos (📍), caixas (🔲), setas e anotações diretamente no print da aba ⚙️ Settings (Item 4: Aceleração Local)</p>\n'
    if line.strip().startswith('const b64Image = "data:image/png;base64,'):
        line = f'    const b64Image = "data:image/png;base64,{hw_b64}";\n'
    if "souls_settings_reasoning_tab_annotator_notes" in line or "souls_settings_tools_tab_annotator_notes" in line:
        line = line.replace("souls_settings_reasoning_tab_annotator_notes", "souls_settings_hardware_tab_annotator_notes").replace("souls_settings_tools_tab_annotator_notes", "souls_settings_hardware_tab_annotator_notes")
    if "settings_reasoning_annotated.png" in line or "settings_tools_annotated.png" in line:
        line = line.replace("settings_reasoning_annotated.png", "settings_hardware_annotated.png").replace("settings_tools_annotated.png", "settings_hardware_annotated.png")
    
    new_lines.append(line)

with open(r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f\subagents_annotator.html", "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("subagents_annotator.html successfully updated with Hardware screenshot!")

