import sqlite3

conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()

c.execute("PRAGMA table_info(pricing_telemetry_snapshots)")
print("pricing_telemetry_snapshots schema:")
for col in c.fetchall():
    print(col)

c.execute("""
SELECT 
    cm.id,
    cm.display_name,
    (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = cm.id AND pts.source LIKE '%nous%' ORDER BY recorded_at DESC LIMIT 1) as nous_p,
    (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = cm.id AND pts.source LIKE '%nous%' ORDER BY recorded_at DESC LIMIT 1) as nous_c,
    (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = cm.id AND pts.source LIKE '%openrouter%' ORDER BY recorded_at DESC LIMIT 1) as or_p,
    (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = cm.id AND pts.source LIKE '%openrouter%' ORDER BY recorded_at DESC LIMIT 1) as or_c
FROM canonical_models cm
LIMIT 15
""")
print("\nPricing query test:")
for r in c.fetchall():
    print(r)

