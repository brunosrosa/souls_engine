path = 'crates/souls_server/src/dashboard_ui.html'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

target = """                <div id="sim-vault-rule" style="font-size: 0.68rem; color: var(--green); font-family: var(--font-mono);">
                  ✓ Obsidian Privacy Shield (§1 & §2 Desidentificação no Silício)
                </div>
              </div>
            </div></div>
          </div>
        </div>
      </div>"""

replacement = """                <div id="sim-vault-rule" style="font-size: 0.68rem; color: var(--green); font-family: var(--font-mono);">
                  ✓ Obsidian Privacy Shield (§1 & §2 Desidentificação no Silício)
                </div>
              </div>
            </div></div>
          </div>
        </div>"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

# Ensure LF
text = text.replace('\r\n', '\n')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Removed extra closing </div> successfully!")

