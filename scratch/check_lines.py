import re

path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# Let's inspect what lines surround handleCloudSort
lines = content.splitlines(keepends=True)
start_line = None
end_line = None

for i, l in enumerate(lines):
    if "function handleCloudSort(" in l:
        start_line = i
    if start_line is not None and "function copyCloudModelId(" in l:
        end_line = i
        break

print(f"handleCloudSort at line {start_line+1}, copyCloudModelId at line {end_line+1}")
print("Lines count:", end_line - start_line)

