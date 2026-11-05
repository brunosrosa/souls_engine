with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# Update switchSettingsSubtab
target_switch = '''      if (subtabId === 'subtab-tools') {
        fetchToolsCatalog();
      }
    }'''

replacement_switch = '''      if (subtabId === 'subtab-tools') {
        fetchToolsCatalog();
      }
      if (subtabId === 'subtab-sandbox') {
        initSandboxSubtab();
      }
    }'''

if target_switch in content:
    content = content.replace(target_switch, replacement_switch, 1)
    print("Updated switchSettingsSubtab!")
else:
    print("Warning: target_switch not found!")

# Update populateSettingsForm for tree limit
target_pop = '''      if (cfg.ast) {
        setCheck('cfg-ast-dehydration', cfg.ast.dehydration_enabled, 'pill-ast-dehydration');
        setInput('cfg-ast-max-lines', cfg.ast.max_lines_tree_sitter);
      }'''

replacement_pop = '''      if (cfg.ast) {
        setCheck('cfg-ast-dehydration', cfg.ast.dehydration_enabled, 'pill-ast-dehydration');
        setInput('cfg-ast-max-lines', cfg.ast.max_lines_tree_sitter);
        setInput('cfg-ast-tree-limit', cfg.ast.max_lines_tree_sitter);
      }'''

if target_pop in content:
    content = content.replace(target_pop, replacement_pop, 1)
    print("Updated populateSettingsForm!")
else:
    print("Warning: target_pop not found!")

# Ensure LF line endings
content = content.replace('\r\n', '\n')

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Saved patch successfully!")

