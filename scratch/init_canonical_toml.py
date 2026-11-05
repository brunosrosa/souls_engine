import urllib.request, json

full_default = """# souls_config.toml — SSOT de Configuração e Governança do Souls Engine v7
# Dev Drive ReFS: Z:\\souls_engine\\.souls_data\\config\\souls_config.toml

[server]
port = 9123
host = "127.0.0.1"
max_active_connections = 16

[reasoning]
dynamic_reasoning_enabled = true
max_reasoning_effort = "ultra"
stepwise_retry_escalation = true
high_entropy_threshold = 0.70
trivial_task_bypass = true

[finops]
vram_hard_limit_mb = 5294
gpu_temp_throttle_c = 78.0
gpu_temp_preventive_c = 72.0
nvml_ewma_alpha = 0.20
watchdog_polling_ms = 500

[router]
pareto_dispatch_enabled = true
weight_quality = 0.50
weight_cost = 0.30
weight_latency = 0.20
geometric_decay_gamma = 0.995
cold_start_sovereignty_bias = true
local_context_token_limit = 8192

[ast]
source_side_dehydration = true
max_tree_sitter_lines = 15000
lean_compression_enabled = true
dedup_min_lines = 5
preserve_docstrings = true

[memory]
langevin_decay_enabled = true
epistemic_half_life_days = 30
rrf_k_constant = 60
wal_checkpoint_mode = "truncate"

[sandbox]
safe_sandbox_timeout_ms = 5000
strict_refs_isolation = true
ansi_sanitization = true

[obsidian]
vault_path = "C:\\Users\\rosas\\OneDrive\\Documentos\\Obisidian DB"
curator_batch_size = 5
privacy_shield_enabled = true
raw_readonly = true
"""

req = urllib.request.Request(
    'http://127.0.0.1:9123/api/v1/settings/config',
    data=json.dumps({'raw_toml': full_default}).encode('utf-8'),
    headers={'Content-Type': 'application/json'}
)
res = urllib.request.urlopen(req)
print('Response:', res.read().decode())

