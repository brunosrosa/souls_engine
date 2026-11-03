import sqlite3
conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()

c.execute('SELECT COUNT(*) FROM pricing_telemetry_snapshots WHERE ttft_ms IS NOT NULL AND ttft_ms > 0')
print('Snapshots with TTFT > 0:', c.fetchone()[0])

c.execute('SELECT canonical_id, ttft_ms, throughput_tps FROM pricing_telemetry_snapshots WHERE ttft_ms IS NOT NULL AND ttft_ms > 0 LIMIT 10')
print('\nTTFT sample:')
for r in c.fetchall():
    print(r)

c.execute("SELECT canonical_id, composite_score FROM daily_evaluations WHERE profile_task = 'design' LIMIT 10")
print("\nDesign profile scores sample:")
for r in c.fetchall():
    print(r)

c.execute("SELECT DISTINCT profile_task FROM daily_evaluations")
print("\nProfile tasks in daily_evaluations:", [r[0] for r in c.fetchall()])

c.execute("PRAGMA table_info(benchmark_snapshots)")
print("\nbenchmark_snapshots columns:")
for col in c.fetchall():
    print(col)

