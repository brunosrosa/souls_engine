import sqlite3
conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()
c.execute("""
SELECT canonical_id, source, prompt_price_per_m, completion_price_per_m, recorded_at 
FROM pricing_telemetry_snapshots 
WHERE prompt_price_per_m = 0.0 AND completion_price_per_m = 0.0
LIMIT 15
""")
rows = c.fetchall()
print(f"Zero price rows count sample: {len(rows)}")
for r in rows:
    print(r)

# Check specifically for openai/gpt-6-astra-pro-flex
c.execute("""
SELECT canonical_id, source, prompt_price_per_m, completion_price_per_m, recorded_at 
FROM pricing_telemetry_snapshots 
WHERE canonical_id LIKE '%astra%'
""")
print("\nAstra pricing:")
for r in c.fetchall():
    print(r)

