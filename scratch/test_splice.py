import subprocess
import os

# 1. Read current working copy
with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    working_content = f.read()

# 2. Read HEAD from git
head_content = subprocess.check_output(['git', 'show', 'HEAD:crates/souls_server/src/dashboard_ui.html'], encoding='utf-8')
head_lines = head_content.splitlines(keepends=True)

# 3. In working copy, find the truncation point
# In working copy: line 9952 has '${m.rep' and line 9953 is '... [truncated for diff preview]'
trunc_idx = working_content.find('... [truncated for diff preview]')
if trunc_idx == -1:
    print("Could not find truncation marker!")
    exit(1)

# Find where the line with '${m.rep' starts
last_good_idx = working_content.rfind('<div>📦 <span style="color: var(--text-muted);">Repo:</span>', 0, trunc_idx)
if last_good_idx == -1:
    print("Could not find last good line before truncation!")
    exit(1)

working_prefix = working_content[:last_good_idx]

# 4. In HEAD, find where '<div>📦 <span style="color: var(--text-muted);">Repo:</span>' occurs inside renderCuratedCatalog
head_repo_idx = -1
for i, l in enumerate(head_lines):
    if '<div>📦 <span style="color: var(--text-muted);">Repo:</span>' in l:
        print(f"Found match in HEAD at line {i+1}: {l.strip()}")
        head_repo_idx = i
        break

if head_repo_idx == -1:
    print("Could not find matching line in HEAD!")
    exit(1)

head_suffix = "".join(head_lines[head_repo_idx:])

print(f"Working prefix length: {len(working_prefix)}")
print(f"HEAD suffix length: {len(head_suffix)}, starting with: {head_suffix[:100]}")

full_restored = working_prefix + head_suffix
print(f"Total restored length: {len(full_restored)}")

# Verify the restored text contains critical end pieces
assert '</html>' in full_restored, "Missing </html>"
assert 'modal-new-profile' in full_restored, "Missing modal-new-profile"
assert 'async function saveSettings()' in full_restored, "Missing saveSettings"
assert 'id="subtab-sandbox"' in full_restored, "Missing subtab-sandbox"

print("Splicing test passed successfully!")

