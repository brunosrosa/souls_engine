path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

for i in range(8560, min(8760, len(lines))):
    if lines[i].strip().startswith("function "):
        print(f"Line {i+1}: {lines[i].strip()}")

