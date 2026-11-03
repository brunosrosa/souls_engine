import sqlite3

conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()

c.execute("SELECT name FROM sqlite_master WHERE type='table'")
print("Tables in heuristic vault:", [r[0] for r in c.fetchall()])

c.execute("SELECT count(*) FROM canonical_models")
print("canonical_models count:", c.fetchone()[0])

c.execute("SELECT count(*) FROM benchmark_snapshots")
print("benchmark_snapshots count:", c.fetchone()[0])

c.execute("SELECT count(*) FROM daily_evaluations")
print("daily_evaluations count:", c.fetchone()[0])

c.execute("SELECT count(*) FROM pricing_telemetry_snapshots")
print("pricing_telemetry count:", c.fetchone()[0])

print("\nSample benchmark_snapshots:")
c.execute("SELECT canonical_id, general_elo, coding_elo, bfcl_score, ifeval_score, gpqa_score, hard_prompts_elo FROM benchmark_snapshots LIMIT 15")
for r in c.fetchall():
    print(r)

print("\nCheck benchmark_snapshots coverage vs canonical_models:")
c.execute("SELECT DISTINCT canonical_id FROM benchmark_snapshots")
bench_ids = set(r[0] for r in c.fetchall())
c.execute("SELECT id FROM canonical_models")
canon_ids = set(r[0] for r in c.fetchall())

print(f"Total bench_ids: {len(bench_ids)}, Total canon_ids: {len(canon_ids)}")
print(f"Intersection: {len(bench_ids.intersection(canon_ids))}")
print(f"Canon IDs with benchmarks: {len(bench_ids.intersection(canon_ids))}/{len(canon_ids)}")
print(f"Missing from benchmarks: {len(canon_ids - bench_ids)}")
print(f"Sample missing: {list(canon_ids - bench_ids)[:10]}")

print("\nSample pricing sources:")
c.execute("SELECT DISTINCT source FROM pricing_telemetry_snapshots")
print([r[0] for r in c.fetchall()])
