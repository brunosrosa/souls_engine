with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

target = '''      // Suporte a Deep Linking via Hash (ex: #tab-local, #tab-cloud, #tab-settings)
      if (window.location.hash) {
        const hashTab = window.location.hash.replace('#', '').split('?')[0];
        if (document.getElementById(hashTab)) {
          switchTab(hashTab);
        }
      }'''

replacement = '''      // Suporte a Deep Linking via Hash (ex: #tab-local, #tab-cloud, #tab-settings/subtab-sandbox)
      if (window.location.hash) {
        const fullHash = window.location.hash.replace('#', '').split('?')[0];
        const parts = fullHash.split('/');
        const mainTab = parts[0];
        const subTab = parts[1];
        if (document.getElementById(mainTab)) {
          switchTab(mainTab);
          if (subTab && typeof switchSettingsSubtab === 'function') {
            switchSettingsSubtab(subTab);
          }
        }
      }'''

if target in content:
    content = content.replace(target, replacement, 1)
    content = content.replace('\r\n', '\n')
    with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print("Updated deep linking in dashboard_ui.html!")
else:
    print("Warning: target not found in dashboard_ui.html")

