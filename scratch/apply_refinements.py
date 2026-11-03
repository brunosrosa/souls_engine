with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    text = f.read()

# 1. Check Modality Pills
idx_pills = text.find('id="cloud-modality-filter-pills"')
print('1. Modality Pills found at:', idx_pills)
if idx_pills != -1:
    print(text[idx_pills-20:idx_pills+500])

# 2. Check setCloudModalityFilter & getFilteredCloudModelsList
idx_set_mod = text.find('function setCloudModalityFilter(modality)')
print('2. setCloudModalityFilter found at:', idx_set_mod)

idx_get_filt = text.find('function getFilteredCloudModelsList()')
print('3. getFilteredCloudModelsList found at:', idx_get_filt)

# 3. Check Cell rendering
idx_cell = text.find('<div style="display: flex; align-items: center; gap: 5px; flex-wrap: wrap; margin-bottom: 4px;">\n                    ${brandBadge}')
print('4. Cell brandBadge found at:', idx_cell)

# 4. Check modal-quick-apply
idx_modal = text.find('id="modal-quick-apply"')
print('5. modal-quick-apply found at:', idx_modal)

# 5. Check openQuickApplyModal & submitQuickApply
idx_open_apply = text.find('function openQuickApplyModal(modelId, provider, isLocal = false)')
print('6. openQuickApplyModal found at:', idx_open_apply)

