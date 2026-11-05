with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

# Let's inspect current 4 KPI cards in subtab-obsidian
start_kpi = content.find('<!-- 4 CARDS DE KPI CENTRADOS (Padrão canônico) -->')
end_kpi = content.find('<!-- BANNER DE GOVERNANÇA DO COFRE & SOBERANIA DE DADOS -->')

print(f"start_kpi: {start_kpi}, end_kpi: {end_kpi}")
if start_kpi != -1 and end_kpi != -1:
    print(content[start_kpi:end_kpi])

