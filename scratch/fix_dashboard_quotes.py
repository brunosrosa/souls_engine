# -*- coding: utf-8 -*-
path = 'crates/souls_server/src/routes/dashboard.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

bad = """            if (v_trim.starts_with('"') && !v_trim.ends_with('"')) || (v_trim.starts_with(''') && !v_trim.ends_with(''')) {"""
good = """            if (v_trim.starts_with('"') && !v_trim.ends_with('"')) || (v_trim.starts_with('\'') && !v_trim.ends_with('\'')) {"""

assert bad in text, "bad target not found"
text = text.replace(bad, good, 1)
text = text.replace('\r\n', '\n')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Fixed dashboard.rs quotes successfully!")

