import re

path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# Locate renderCloudRankedTable and replace the entire function along with column visibility helpers
old_start = "    function renderCloudRankedTable(models) {"
old_end = "      tbody.innerHTML = list.map((m, idx) => {"

# We will search from `function handleCloudSort` down through `renderCloudRankedTable`
pattern = r"    function handleCloudSort\(val\) \{[\s\S]*?renderCloudRankedTable\(models\) \{[\s\S]*?tbody\.innerHTML = list\.map\(\(m, idx\) => \{[\s\S]*?\}\)\.join\(''\);\s*\}\s*function"

# Let's inspect what exists between handleCloudSort and the next function
m = re.search(r"function handleCloudSort\(val\) \{[\s\S]*?\}\)\.join\(''\);\s*\}", content)
if not m:
    print("Could not find regex match for handleCloudSort -> renderCloudRankedTable!")
else:
    print("Found match! Length:", len(m.group(0)))

