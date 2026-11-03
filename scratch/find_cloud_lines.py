with open(r'Z:\souls_engine\crates\souls_server\src\dashboard_ui.html', 'r', encoding='utf-8') as f:
    for i, line in enumerate(f):
        if 'tab-cloud' in line or 'Ranking Comparativo' in line or 'renderCloudRankedTable' in line:
            print(f"{i+1}: {line.strip()[:80]}")

