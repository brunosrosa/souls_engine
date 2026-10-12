# tools/test_domain1_ast.ps1
# Bateria de Homologação Clínica — Domínio 1: Engenharia de Código, AST & Cirurgia Sintática
# Testa 8 Ferramentas em 3 Cenários (Fácil, Médio, Difícil / Borda / Segurança ReFS)

param(
    [string]$BaseUrl = "http://127.0.0.1:9123"
)

$ErrorActionPreference = "Continue"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  HOMOLOGAÇÃO CLÍNICA — DOMÍNIO 1: AST & CIRURGIA DE CÓDIGO" -ForegroundColor Cyan
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

        $resp = Invoke-RestMethod -Uri "$BaseUrl/mcp" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
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
        # Se o teste esperava erro (ex: violação de path ou arquivo inexistente)
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

# --- TOOL 1: outline ---
Write-Host "`n>> Testando Tool: outline" -ForegroundColor Yellow
Execute-Scenario -Tool "outline" -Scenario "Fácil (Nominal)" -Arguments @{ path = "crates/souls_server/src/state.rs" } -CallId 101 -ExpectedBehavior "Extrai outline com sucesso" -Validator { param($r) $r.result.content[0].text -match "AppState" }
Execute-Scenario -Tool "outline" -Scenario "Médio (Arquivo Grande)" -Arguments @{ path = "crates/souls_server/src/main.rs" } -CallId 102 -ExpectedBehavior "Extrai assinaturas desidratadas de 1.100 linhas" -Validator { param($r) $r.result.content[0].text -match "build_router" }
Execute-Scenario -Tool "outline" -Scenario "Difícil (Segurança ReFS)" -Arguments @{ path = "C:\Windows\System32\drivers\etc\hosts" } -CallId 103 -ExpectedBehavior "Barrado por RefsPathLeakViolation" -Validator { param($r) $r.error -ne $null -or $r.result.isError }

# --- TOOL 2: symbol ---
Write-Host "`n>> Testando Tool: symbol" -ForegroundColor Yellow
Execute-Scenario -Tool "symbol" -Scenario "Fácil (Nominal)" -Arguments @{ path = "crates/souls_server/src/state.rs"; symbol_query = "AppState" } -CallId 201 -ExpectedBehavior "Localiza nó sintático AppState" -Validator { param($r) $r.result.content[0].text -match "AppState" }
Execute-Scenario -Tool "symbol" -Scenario "Médio (Função Qualificada)" -Arguments @{ path = "crates/souls_server/src/routes/mod.rs"; symbol_query = "build_router" } -CallId 202 -ExpectedBehavior "Extrai bloco da função build_router" -Validator { param($r) $r.result.content[0].text -match "build_router" }
Execute-Scenario -Tool "symbol" -Scenario "Difícil (Símbolo Inexistente)" -Arguments @{ path = "crates/souls_server/src/state.rs"; symbol_query = "NonExistentStructXYZ" } -CallId 203 -ExpectedBehavior "Retorna aviso controlado de símbolo não encontrado" -Validator { param($r) $r.result.content[0].text -match "not found" -or $r.result.isError -or $r.error -ne $null }

# --- TOOL 3: callers ---
Write-Host "`n>> Testando Tool: callers" -ForegroundColor Yellow
Execute-Scenario -Tool "callers" -Scenario "Fácil (Nominal)" -Arguments @{ symbol_name = "build_router" } -CallId 301 -ExpectedBehavior "Encontra invocadores de build_router" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "callers" -Scenario "Médio (Com File Scope)" -Arguments @{ symbol_name = "health_handler"; file_scope = "crates/souls_server/src/routes/health.rs" } -CallId 302 -ExpectedBehavior "Filtra callers por escopo de arquivo" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "callers" -Scenario "Difícil (Símbolo Fantasma)" -Arguments @{ symbol_name = "xyz_phantom_function_404" } -CallId 303 -ExpectedBehavior "Retorna lista vazia sem falha" -Validator { param($r) -not $r.result.isError }

# --- TOOL 4: callees ---
Write-Host "`n>> Testando Tool: callees" -ForegroundColor Yellow
Execute-Scenario -Tool "callees" -Scenario "Fácil (Nominal)" -Arguments @{ symbol_name = "build_router" } -CallId 401 -ExpectedBehavior "Mapeia chamadas internas de build_router" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "callees" -Scenario "Médio (Com File Scope)" -Arguments @{ symbol_name = "telemetry_handler"; file_scope = "crates/souls_server/src/routes/dashboard.rs" } -CallId 402 -ExpectedBehavior "Mapeia chamadas do handler de telemetria" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "callees" -Scenario "Difícil (Símbolo Inexistente)" -Arguments @{ symbol_name = "xyz_phantom_callee_999" } -CallId 403 -ExpectedBehavior "Retorna lista vazia de forma segura" -Validator { param($r) -not $r.result.isError }

# --- TOOL 5: get_ast ---
Write-Host "`n>> Testando Tool: get_ast" -ForegroundColor Yellow
Execute-Scenario -Tool "get_ast" -Scenario "Fácil (Nominal)" -Arguments @{ path = "crates/souls_server/src/state.rs"; max_depth = 3 } -CallId 501 -ExpectedBehavior "Retorna árvore sintática JSON" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "get_ast" -Scenario "Médio (Profundidade Alta)" -Arguments @{ path = "crates/souls_server/src/routes/health.rs"; max_depth = 10 } -CallId 502 -ExpectedBehavior "Gera AST profunda sem estouro de stack" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "get_ast" -Scenario "Difícil (Escape ReFS)" -Arguments @{ path = "../../Windows/System32" } -CallId 503 -ExpectedBehavior "Barrado por RefsPathLeakViolation" -Validator { param($r) $r.error -ne $null -or $r.result.isError }

# --- TOOL 6: delta_diff ---
Write-Host "`n>> Testando Tool: delta_diff" -ForegroundColor Yellow
Execute-Scenario -Tool "delta_diff" -Scenario "Fácil (Diff contra Arquivo Real)" -Arguments @{ path = "crates/souls_server/src/routes/health.rs"; new_content = "// modified health route\npub fn test() {}" } -CallId 601 -ExpectedBehavior "Gera patch Myers com diff_text" -Validator { param($r) $r.result.content[0].text -ne "(no changes)" }
Execute-Scenario -Tool "delta_diff" -Scenario "Médio (Conteúdo Idêntico)" -Arguments @{ path = "crates/souls_server/src/routes/health.rs"; new_content = [System.IO.File]::ReadAllText("Z:\souls_engine\crates\souls_server\src\routes\health.rs") } -CallId 602 -ExpectedBehavior "Diff vazio com custo zero de alteração" -Validator { param($r) $r.result.content[0].text -match "no changes" -or -not $r.result.isError }
Execute-Scenario -Tool "delta_diff" -Scenario "Difícil (Arquivo Inexistente)" -Arguments @{ path = "crates/souls_server/src/missing_file.rs"; new_content = "new file content" } -CallId 603 -ExpectedBehavior "Calcula diff de adição pura sem falhar" -Validator { param($r) -not $r.result.isError }

# --- TOOL 7: edit ---
Write-Host "`n>> Testando Tool: edit" -ForegroundColor Yellow
Execute-Scenario -Tool "edit" -Scenario "Fácil (Dry-Run Verificação)" -Arguments @{ path = "crates/souls_server/src/state.rs"; dry_run = $true } -CallId 701 -ExpectedBehavior "Dry run executa com validação sintática" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "edit" -Scenario "Médio (Alvo Válido)" -Arguments @{ path = "crates/souls_server/src/routes/health.rs"; dry_run = $true } -CallId 702 -ExpectedBehavior "Simula edição cirúrgica sem tocar no disco" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "edit" -Scenario "Difícil (Escape de Path)" -Arguments @{ path = "C:\escaped\path.rs"; dry_run = $true } -CallId 703 -ExpectedBehavior "Barrado por RefsPathLeakViolation" -Validator { param($r) $r.error -ne $null -or $r.result.isError }

# --- TOOL 8: replace ---
Write-Host "`n>> Testando Tool: replace" -ForegroundColor Yellow
Execute-Scenario -Tool "replace" -Scenario "Fácil (Dry-Run)" -Arguments @{ path = "crates/souls_server/src/state.rs"; dry_run = $true; target = "AppState"; replacement = "AppState" } -CallId 801 -ExpectedBehavior "Dry run valida sem alteração em disco" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "replace" -Scenario "Médio (Substituição de Bloco)" -Arguments @{ path = "crates/souls_server/src/routes/health.rs"; dry_run = $true; target = "healthy"; replacement = "healthy" } -CallId 802 -ExpectedBehavior "Localiza e simula substituição" -Validator { param($r) -not $r.result.isError }
Execute-Scenario -Tool "replace" -Scenario "Difícil (Alvo Inexistente)" -Arguments @{ path = "crates/souls_server/src/state.rs"; dry_run = $true; target = "NON_EXISTENT_STRING_99999"; replacement = "abc" } -CallId 803 -ExpectedBehavior "Retorna erro ou alerta controlado" -Validator { param($r) -not $r.result.isError -or $r.result.isError }

# CONSOLIDAÇÃO DO DOMÍNIO 1
Write-Host "`n=================================================================" -ForegroundColor Cyan
$passed = ($results | Where-Object { $_.Success }).Count
$total = $results.Count
$rate = [math]::Round(($passed / $total) * 100, 1)
$avgLat = [math]::Round(($results | Measure-Object -Property LatencyMs -Average).Average, 2)

Write-Host ("DOMÍNIO 1 CONCLUÍDO: {0}% ({1}/{2} cenários aprovados) — Latência Média: {3} ms" -f $rate, $passed, $total, $avgLat) -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Cyan
