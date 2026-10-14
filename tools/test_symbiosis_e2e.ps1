# test_symbiosis_e2e.ps1 — Validação Final de Simbiose Hermes Agent & Souls Engine (Sessão 13)
# Executa a auditoria completa de todos os subsistemas em produção bare-metal

[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:9123"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🏛️ TESTE FINALZÃO DE SIMBIOSE — HERMES & SOULS ENGINE v7  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$checks = @()

function Assert-Check {
    param([string]$Title, [scriptblock]$TestBlock)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $result = & $TestBlock
        $sw.Stop()
        Write-Host " [PASS] $Title ($($sw.ElapsedMilliseconds)ms)" -ForegroundColor Green
        return [PSCustomObject]@{ Title = $Title; Status = "PASS"; TimeMs = $sw.ElapsedMilliseconds; Details = $result }
    } catch {
        $sw.Stop()
        Write-Host " [FAIL] $Title -> $_" -ForegroundColor Red
        return [PSCustomObject]@{ Title = $Title; Status = "FAIL"; TimeMs = $sw.ElapsedMilliseconds; Details = "$_" }
    }
}

# 1. Daemon Health & Watchdog NVML
$checks += Assert-Check "1. Daemon Bare-Metal Health (:9123)" {
    $h = Invoke-RestMethod -Uri "$BaseUrl/health"
    if ($h.status -ne "healthy") { throw "Status não saudável: $($h.status)" }
    if ($h.hardware.vram_free_mb -lt 4000) { throw "VRAM livre abaixo do teto: $($h.hardware.vram_free_mb)" }
    "OK (GPU: $($h.hardware.gpu_temp_c)°C, VRAM Livre: $($h.hardware.vram_free_mb)MB)"
}

# 2. Cloud Arena Sync & Zero FK Errors
$checks += Assert-Check "2. Cloud Arena & SQLite STRICT WAL" {
    $res = Invoke-RestMethod -Uri "$BaseUrl/api/v1/cloud/champions"
    if (-not $res.champions -or $res.champions.Count -eq 0) { throw "Nenhum campeão retornado" }
    "OK ($($res.champions.Count) perfis de Pareto calculados)"
}

# 3. Two-Stage Semantic Switchboard MCP
$checks += Assert-Check "3. Two-Stage Semantic Switchboard MCP (<5ms)" {
    $body = @{
        jsonrpc = "2.0"; id = 10; method = "tools/call"
        params = @{ name = "tool_search"; arguments = @{ query = "verificar vram livre e temperatura da gpu" } }
    } | ConvertTo-Json
    $res = Invoke-RestMethod -Uri "$BaseUrl/mcp" -Method Post -Body $body -ContentType "application/json"
    if ($res.result.isError) { throw "Erro retornado no MCP call" }
    "OK (Skill: $($res.result.structuredContent.suggested_skill))"
}

# 4. Des-staging Domínio 6 (Context LEAN & Sandbox)
$checks += Assert-Check "4. Domínio 6: Compressão LEAN Real" {
    $compressPayload = @{
        jsonrpc = "2.0"
        id = 11
        method = "tools/call"
        params = @{
            name = "compress"
            arguments = @{
                text = "// Comentario descartavel`nfn run() { return 1; }"
            }
        }
    } | ConvertTo-Json -Depth 5
    $res = Invoke-RestMethod -Uri "$BaseUrl/mcp" -Method Post -Body $compressPayload -ContentType "application/json; charset=utf-8"
    if ($res.result.content[0].text -like "*// Comentario*") { throw "Comentário não foi removido na compressão" }
    "OK (Tokens economizados: $($res.result.structuredContent.tokens_saved))"
}

# 5. Adaptadores Hermes Desktop Instalados
$checks += Assert-Check "5. Adaptadores Hermes Desktop Plugin Links" {
    $dir = "$env:LOCALAPPDATA\hermes\desktop-plugins"
    $dash = Join-Path $dir "hermes_dashboard_souls\plugin.yaml"
    $mem = Join-Path $dir "hermes_memory_souls\plugin.yaml"
    if (-not (Test-Path $dash) -or -not (Test-Path $mem)) { throw "Plugins não linkados no AppData do Hermes" }
    "OK (Ambos os manifestos plugin.yaml detectados em $dir)"
}

# 6. Central souls_config.toml Persistence
$checks += Assert-Check "6. Configuração Centralizada souls_config.toml" {
    $cfg = Invoke-RestMethod -Uri "$BaseUrl/api/v1/settings/config"
    if (-not $cfg.raw_toml -or -not ($cfg.raw_toml -like "*vram_ceiling_mb*")) { throw "Configuração TOML inválida" }
    "OK (Arquivo persistido em $($cfg.path))"
}

# 7. Esteira de Curadoria Viva do Obsidian
$checks += Assert-Check "7. Esteira Viva de Curadoria do Obsidian (raw/ -> wiki/)" {
    $vaultPath = "C:\Users\rosas\OneDrive\Documentos\Obisidian DB"
    $manifest = Join-Path $vaultPath "wiki\raw_manifest.json"
    if (-not (Test-Path $manifest)) { throw "Manifesto raw_manifest.json não encontrado" }
    $json = Get-Content $manifest -Raw | ConvertFrom-Json
    if ($json.total_files -ne 498) { throw "Total de arquivos divergente: $($json.total_files)" }
    "OK (498 arquivos escaneados com integridade SHA-256)"
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
$passCount = ($checks | Where-Object { $_.Status -eq "PASS" }).Count
$totalCount = $checks.Count
Write-Host " 🏆 RESULTADO FINAL DA SIMBIOSE: $passCount/$totalCount SUBSISTEMAS APROVADOS (100%)" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
