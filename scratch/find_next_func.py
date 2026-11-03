path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()

for i, l in enumerate(lines):
    if "function handleCloudSort(" in l:
        print(f"handleCloudSort at line {i+1}")
        for j in range(i, min(i+160, len(lines))):
            if lines[j].strip().startswith("function ") and "handleCloudSort" not in lines[j] and "renderCloudRankedTable" not in lines[j]:
                print(f"Next function at line {j+1}: {lines[j].strip()}")
                break
        break

