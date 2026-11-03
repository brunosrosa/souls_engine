import sqlite3

conn = sqlite3.connect(r'Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db')
c = conn.cursor()

query = """
SELECT 
    de.canonical_id,
    cm.display_name,
    (
        0.35 * COALESCE(MAX(CASE WHEN de.profile_task = 'agentic' THEN de.composite_score END), 70.0) +
        0.35 * COALESCE(MAX(CASE WHEN de.profile_task = 'reasoning' THEN de.composite_score END), 70.0) +
        0.30 * COALESCE(MAX(CASE WHEN de.profile_task = 'coding' THEN de.composite_score END), 70.0)
    ) as score,
    AVG(de.task_blended_cost) as task_blended_cost,
    AVG(de.roi_index) as roi_index,
    'Leader' as pareto_tag,
    cm.context_window,
    (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%') as has_nous,
    (SELECT COUNT(*) FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%') as has_openrouter,
    (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_prompt_price,
    (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%nous%' ORDER BY pts.recorded_at DESC LIMIT 1) as nous_completion_price,
    (SELECT prompt_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_prompt_price,
    (SELECT completion_price_per_m FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id AND pts.source LIKE '%openrouter%' ORDER BY pts.recorded_at DESC LIMIT 1) as or_completion_price,
    MAX(CASE WHEN de.profile_task = 'coding' THEN de.composite_score END) as eval_coding,
    MAX(CASE WHEN de.profile_task = 'agentic' THEN de.composite_score END) as eval_agentic,
    MAX(CASE WHEN de.profile_task = 'reasoning' THEN de.composite_score END) as eval_reasoning,
    MAX(CASE WHEN de.profile_task = 'fast' THEN de.composite_score END) as eval_fast,
    MAX(CASE WHEN de.profile_task = 'design' THEN de.composite_score END) as eval_design,
    bs.general_elo,
    bs.coding_elo,
    bs.bfcl_score,
    bs.ifeval_score,
    bs.gpqa_score,
    bs.hard_prompts_elo,
    (SELECT throughput_tps FROM pricing_telemetry_snapshots pts WHERE pts.canonical_id = de.canonical_id ORDER BY pts.recorded_at DESC LIMIT 1) as tps
FROM daily_evaluations de
JOIN canonical_models cm ON cm.id = de.canonical_id
LEFT JOIN (
    SELECT canonical_id, general_elo, coding_elo, bfcl_score, ifeval_score, gpqa_score, hard_prompts_elo
    FROM benchmark_snapshots
    GROUP BY canonical_id
) bs ON bs.canonical_id = de.canonical_id
WHERE de.evaluation_date = (
    SELECT MAX(evaluation_date) 
    FROM daily_evaluations
)
GROUP BY de.canonical_id
ORDER BY score DESC
LIMIT 10;
"""

c.execute(query)
rows = c.fetchall()
print(f"Returned {len(rows)} rows.")
for r in rows[:5]:
    print("ID:", r[0])
    print("Name:", r[1])
    print(f"Score: {r[2]:.1f}, Cost: {r[3]:.2f}, ROI: {r[4]:.1f}")
    print(f"Nous: ${r[9]} / ${r[10]}, OR: ${r[11]} / ${r[12]}")
    print(f"Eval - Coding: {r[13]}, Agentic: {r[14]}, Reasoning: {r[15]}")
    print(f"Bench - Coding: {r[19]}, BFCL: {r[20]}, GPQA: {r[22]}, IFEval: {r[21]}")
    print("-" * 50)

