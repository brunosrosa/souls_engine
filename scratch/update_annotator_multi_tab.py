import base64
import os
import re

brain_dir = r"C:\Users\rosas\.gemini\antigravity\brain\21d96846-20af-4e94-97c9-db9237aa802f"
annotator_path = os.path.join(brain_dir, "subagents_annotator.html")

def get_b64(filename):
    p = os.path.join(brain_dir, filename)
    with open(p, "rb") as f:
        return f"data:image/png;base64,{base64.b64encode(f.read()).decode('utf-8')}"

b64_brain = get_b64("settings_brain_tab.png")
b64_obsidian = get_b64("settings_obsidian_tab.png")
b64_raw = get_b64("settings_raw_tab.png")

with open(annotator_path, "r", encoding="utf-8") as f:
    html = f.read()

# 1. Update title and header
html = re.sub(
    r"<title>.*?</title>",
    "<title>Anotador Visual: ⚙️ Settings (Master Brain, Cofre & Configuração Avançada) — Souls Engine v7</title>",
    html,
    count=1
)

# 2. Add Screen Switcher Bar right after the header title
nav_bar = '''
        <div class="flex items-center gap-2 bg-slate-900 border border-slate-700/60 rounded-lg p-1.5 ml-4">
          <button id="btn-screen-brain" onclick="switchScreen('brain')" class="px-3 py-1 text-xs font-semibold rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 hover:bg-cyan-500/30 transition-all flex items-center gap-1.5">
            <span>🧠</span> Master Brain (3 Cards)
          </button>
          <button id="btn-screen-obsidian" onclick="switchScreen('obsidian')" class="px-3 py-1 text-xs font-semibold rounded bg-slate-800 text-slate-400 border border-slate-700 hover:text-white transition-all flex items-center gap-1.5">
            <span>📓</span> Cofre (Nuvemzinha & Respiro)
          </button>
          <button id="btn-screen-raw" onclick="switchScreen('raw')" class="px-3 py-1 text-xs font-semibold rounded bg-slate-800 text-slate-400 border border-slate-700 hover:text-white transition-all flex items-center gap-1.5">
            <span>📄</span> Configuração Avançada (TOML)
          </button>
        </div>
'''

# Find header container
header_marker = '<header class="h-14 border-b border-slate-800 bg-slate-900/90 backdrop-blur px-4 flex items-center justify-between z-10 select-none">'
if header_marker in html:
    # insert before closing </header>
    head_end = html.find('</header>')
    # inject nav_bar before the right actions in header
    actions_marker = '<div class="flex items-center gap-3">'
    idx = html.find(actions_marker)
    if idx != -1:
        html = html[:idx] + nav_bar + html[idx:]
        print("Injected screen switcher bar into header!")

# 3. Replace b64Image definition with all 3 screenshots dictionary
old_b64_target = 'const b64Image = "'
idx_b64 = html.find(old_b64_target)
if idx_b64 != -1:
    end_quote = html.find('";', idx_b64) + 2
    multi_b64_js = f'''const SCREENSHOTS = {{
      brain: "{b64_brain}",
      obsidian: "{b64_obsidian}",
      raw: "{b64_raw}"
    }};
    let currentScreen = 'brain';
    let b64Image = SCREENSHOTS.brain;

    function switchScreen(screenKey) {{
      if (!SCREENSHOTS[screenKey]) return;
      currentScreen = screenKey;
      b64Image = SCREENSHOTS[screenKey];
      
      // Update buttons
      ['brain', 'obsidian', 'raw'].forEach(k => {{
        const btn = document.getElementById('btn-screen-' + k);
        if (btn) {{
          if (k === screenKey) {{
            btn.className = "px-3 py-1 text-xs font-semibold rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 hover:bg-cyan-500/30 transition-all flex items-center gap-1.5";
          }} else {{
            btn.className = "px-3 py-1 text-xs font-semibold rounded bg-slate-800 text-slate-400 border border-slate-700 hover:text-white transition-all flex items-center gap-1.5";
          }}
        }}
      }});

      img.src = b64Image;
    }}'''

    html = html[:idx_b64] + multi_b64_js + html[end_quote:]
    print("Replaced b64Image with multi-screen switcher dictionary!")

# 4. Reset pins
html = re.sub(
    r'let pins = \[.*?\];',
    'let pins = [];',
    html,
    count=1,
    flags=re.DOTALL
)

with open(annotator_path, "w", encoding="utf-8") as f:
    f.write(html)

print("SUCCESS! subagents_annotator.html enriched with multi-screen tabs!")

