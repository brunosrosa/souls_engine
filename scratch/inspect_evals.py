import sqlite3

conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()

c.execute("PRAGMA table_info(daily_evaluations)")
print("daily_evaluations schema:")
for col in c.fetchall():
    print(col)

c.execute("SELECT canonical_id, profile_task, composite_score, task_blended_cost, roi_index FROM daily_evaluations WHERE canonical_id='anthropic/claude-opus-5.5' LIMIT 10")
print("\nClaude Opus 5.5 in daily_evaluations:")
for r in c.fetchall():
    print(r)

c.execute("SELECT canonical_id, profile_task, composite_score, task_blended_cost, roi_index FROM daily_evaluations WHERE canonical_id='anthropic/claude-sonnet-5.5' LIMIT 10")
print("\nClaude Sonnet 5.5 in daily_evaluations:")
for r in c.fetchall():
    print(r)

c.execute("SELECT canonical_id, profile_task, composite_score, task_blended_cost, roi_index FROM daily_evaluations WHERE canonical_id='openai/gpt-6.1-sol' LIMIT 10")
print("\nGPT-6.1 Sol in daily_evaluations:")
for r in c.fetchall():
    print(r)

# Check pricing comparison between nous and openrouter
c.execute("""
SELECT 
    pts_nous.canonical_id,
    pts_nous.prompt_price_per_m as nous_prompt,
    pts_nous.completion_price_per_m as nous_comp,
    pts_or.prompt_price_per_m as or_prompt,
    pts_or.completion_price_per_m as or_comp
FROM pricing_telemetry_snapshots pts_nous
LEFT JOIN pricing_telemetry_snapshots pts_or 
    ON pts_or.canonical_id = pts_nous.canonical_id AND pts_or.source = 'openrouter_market'
WHERE pts_nous.source = 'nous_portal'
LIMIT 10
""")
print("\nPricing comparison sample:")
for r in c.fetchall():
    print(r)

