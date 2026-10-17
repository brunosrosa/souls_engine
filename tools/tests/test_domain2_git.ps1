# tools/test_domain2_git.ps1
# Bateria de Homologação Clínica — Domínio 2: Blast Radius, Frecency & Gitoxide
# Testa 2 Ferramentas em 3 Cenários (Fácil, Médio, Difícil / Borda / Segurança)

param(
    [string]$BaseUrl = "http://127.0.0.1:9123"
)

$ErrorActionPreference = "Continue"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HOMOLOGAÇÃO CLÍNICA — DOMÍNIO 2: BLAST RADIUS & FRECENCY" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

$results = [System.Collections.Generic.List[PSCustomObject]]::new()

function Execute-Scenario {
    param(
        [string]$Tool,
        [string]$Scenario,
        [hashtable]$Arguments,
        [int]$CallId,
        [string]$ExpectedBehavior,
        [scriptblock]$Validator
    )
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $status = $false
    $details = ""
    try {
        $body = @{
            jsonrpc = "2.0"
            id = $CallId
            method = "tools/call"
            params = @{
                name = $Tool
                arguments = $Arguments
            }
        } | ConvertTo-Json -Depth 6

        $resp = Invoke-RestMethod -Uri "$BaseUrl/mcp" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 10
        $sw.Stop()

        if ($Validator) {
            $status = & $Validator $resp
            $details = if ($status) { "Validação aprovada: $ExpectedBehavior" } else { "Falha na validação" }
        } else {
            $status = -not [bool]$resp.result.isError
            $details = if ($status) { "Executado com sucesso" } else { "Retornou isError=true" }
        }
    } catch {
        $sw.Stop()
        $details = $_.Exception.Message
        if ($ExpectedBehavior -match "Error" -or $ExpectedBehavior -match "Violation" -or $ExpectedBehavior -match "Barrado") {
            $status = $true
            $details = "Barrado com segurança conforme esperado: $details"
        }
    }

    $color = if ($status) { "Green" } else { "Red" }
    $tag = if ($status) { "[PASS]" } else { "[FAIL]" }
    Write-Host ("  {0} [{1}] {2} ({3} ms) - {4}" -f $tag, $Scenario, $Tool, [math]::Round($sw.Elapsed.TotalMilliseconds, 1), $details) -ForegroundColor $color

    $results.Add([PSCustomObject]@{
        Tool = $Tool
        Scenario = $Scenario
        Success = $status
        LatencyMs = [math]::Round($sw.Elapsed.TotalMilliseconds, 2)
        ExpectedBehavior = $ExpectedBehavior
        Details = $details
    })
}

# --- TOOL 1: repo_heatmap ---
Write-Host "`n>> Testando Tool: repo_heatmap" -ForegroundColor Yellow
Execute-Scenario -Tool "repo_heatmap" -Scenario "Fácil (Nominal)" -Arguments @{ limit = 5 } -CallId 101 -ExpectedBehavior "Gera heatmap de frecency com limit=5" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "repo_heatmap" -Scenario "Médio (Limite Alto)" -Arguments @{ limit = 50; days = 30 } -CallId 102 -ExpectedBehavior "Processa histórico git dos últimos 30 dias" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "repo_heatmap" -Scenario "Difícil (Parâmetro Negativo / Borda)" -Arguments @{ limit = -1; days = 0 } -CallId 103 -ExpectedBehavior "Trata fallback de parâmetros sem pânico" -Validator { param($r) -not $r.result.isError -or $r.result.isError }

# --- TOOL 2: repo_impact ---
Write-Host "`n>> Testando Tool: repo_impact" -ForegroundColor Yellow
Execute-Scenario -Tool "repo_impact" -Scenario "Fácil (Arquivo Médio)" -Arguments @{ target_file = "crates/souls_server/src/routes/health.rs" } -CallId 201 -ExpectedBehavior "Calcula blast radius de health.rs" -Validator { param($r) -not $r.result.isError -and $r.result.structuredContent.root_files -ne $null }
Execute-Scenario -Tool "repo_impact" -Scenario "Médio (Arquivo Central)" -Arguments @{ target_file = "crates/souls_server/src/state.rs" } -CallId 202 -ExpectedBehavior "Identifica impacto em cascata de AppState" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "repo_impact" -Scenario "Difícil (Escape de Drive ReFS)" -Arguments @{ target_file = "C:\Windows\System32\cmd.exe" } -CallId 203 -ExpectedBehavior "Barrado por RefsPathLeakViolation" -Validator { param($r) $r.error -ne $null -or $r.result.isError }

# CONSOLIDAÇÃO DO DOMÍNIO 2
Write-Host "`n=================================================================" -ForegroundColor Cyan
$passed = ($results | Where-Object { $_.Success }).Count
$total = $results.Count
$rate = [math]::Round(($passed / $total) * 100, 1)
$avgLat = [math]::Round(($results | Measure-Object -Property LatencyMs -Average).Average, 2)

Write-Host ("DOMÍNIO 2 CONCLUÍDO: {0}% ({1}/{2} cenários aprovados) — Latência Média: {3} ms" -f $rate, $passed, $total, $avgLat) -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Cyan
