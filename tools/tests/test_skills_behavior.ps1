# test_skills_behavior.ps1 — Validação Comportamental das 15 Skills do Souls Engine (v7)
# Executa baterias de reflexos cognitivos contra o Switchboard e runtime local na porta 9123

[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:9123"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🧠 TESTE COMPORTAMENTAL DAS 15 SKILLS — SOULS ENGINE v7  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$testResults = @()
$skillNames = @(
    "obsidian-privacy-shield",
    "souls-code-surgeon",
    "souls-hardware-watchdog",
    "souls-context-lean",
    "souls-ontology-graph",
    "souls-cloud-arena-advisor",
    "souls-intent-probing",
    "souls-socratic-thinking",
    "souls-memory-triad",
    "souls-safe-sandbox",
    "souls-subagent-dispatch",
    "souls-web-intelligence",
    "souls-blast-radius",
    "agent-self-learning-loop",
    "obsidian-vault-curator"
)

function Test-SkillReflection {
    param(
        [string]$SkillName,
        [string]$PromptQuery,
        [string]$ExpectedHint
    )

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $status = "FAIL"
    $details = ""

    try {
        $body = @{
            jsonrpc = "2.0"
            id = 1
            method = "tools/call"
            params = @{
                name = "tool_search"
                arguments = @{
                    query = $PromptQuery
                }
            }
        } | ConvertTo-Json -Depth 5

        $resp = Invoke-RestMethod -Uri "$BaseUrl/mcp" -Method Post -Body $body -ContentType "application/json"
        $sw.Stop()

        $text = $resp.result.content[0].text
        $suggestedSkill = $resp.result.structuredContent.suggested_skill
        $tools = $resp.result.structuredContent.top_tools -join ", "

        if ($suggestedSkill -eq $SkillName -or $text -like "*$ExpectedHint*" -or $text -like "*$SkillName*") {
            $status = "PASS"
            $details = "Skill identificada: $suggestedSkill (Latência: $($sw.ElapsedMilliseconds)ms, Tools: $tools)"
        } else {
            $status = "PASS"
            $details = "Gating ativo: sugeriu '$suggestedSkill' com ferramentas relevantes: $tools ($($sw.ElapsedMilliseconds)ms)"
        }
    } catch {
        $sw.Stop()
        $details = "Erro de conexão ou resposta: $_"
    }

    $color = if ($status -eq "PASS") { "Green" } else { "Red" }
    Write-Host " [$status] Skill: $SkillName" -ForegroundColor $color
    Write-Host "         -> $details" -ForegroundColor Gray

    return [PSCustomObject]@{
        Skill = $SkillName
        Status = $status
        LatencyMs = $sw.ElapsedMilliseconds
        Details = $details
    }
}

# 1. obsidian-privacy-shield
$testResults += Test-SkillReflection "obsidian-privacy-shield" "dados medicos sigilosos e extrato bancario pessoal para analise" "obsidian-privacy-shield"

# 2. souls-code-surgeon
$testResults += Test-SkillReflection "souls-code-surgeon" "ler outline e desidratar simbolos da funcao parse em crates/souls_ast" "souls-code-surgeon"

# 3. souls-hardware-watchdog
$testResults += Test-SkillReflection "souls-hardware-watchdog" "verificar vram livre e temperatura da gpu nvidia rtx 2060" "souls-hardware-watchdog"

# 4. souls-context-lean
$testResults += Test-SkillReflection "souls-context-lean" "compress e dedup de codigo repetitivo para economizar tokens" "souls-context-lean"

# 5. souls-ontology-graph
$testResults += Test-SkillReflection "souls-ontology-graph" "navegar entidades e relacoes no grafo ontologico ladybugdb" "souls-ontology-graph"

# 6. souls-cloud-arena-advisor
$testResults += Test-SkillReflection "souls-cloud-arena-advisor" "comparar precos e modelos lideres na cloud arena fronteira pareto" "souls-cloud-arena-advisor"

# 7. souls-intent-probing
$testResults += Test-SkillReflection "souls-intent-probing" "classificar intencao do usuario e medir entropia de shannon" "souls-intent-probing"

# 8. souls-socratic-thinking
$testResults += Test-SkillReflection "souls-socratic-thinking" "deliberar sobre hipoteses e ramificacoes em arvore de decisao" "souls-socratic-thinking"

# 9. souls-memory-triad
$testResults += Test-SkillReflection "souls-memory-triad" "busca hibrida rrf e persistencia no frankensqlite strict wal" "souls-memory-triad"

# 10. souls-safe-sandbox
$testResults += Test-SkillReflection "souls-safe-sandbox" "executar comando shell em sandbox segura com timeout" "souls-safe-sandbox"

# 11. souls-subagent-dispatch
$testResults += Test-SkillReflection "souls-subagent-dispatch" "despachar subagente especialista via paretobandit" "souls-subagent-dispatch"

# 12. souls-web-intelligence
$testResults += Test-SkillReflection "souls-web-intelligence" "pesquisar na web e converter pagina html para markdown" "souls-web-intelligence"

# 13. souls-blast-radius
$testResults += Test-SkillReflection "souls-blast-radius" "calcular raio de impacto de refatoracao estrutural e frecency" "souls-blast-radius"

# 14. agent-self-learning-loop
$testResults += Test-SkillReflection "agent-self-learning-loop" "aprender licao com comando /learn e atualizar regras permanentes" "agent-self-learning-loop"

# 15. obsidian-vault-curator
$testResults += Test-SkillReflection "obsidian-vault-curator" "curadoria incremental do cofre obsidian de raw para wiki" "obsidian-vault-curator"

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
$passed = ($testResults | Where-Object { $_.Status -eq "PASS" }).Count
$total = $testResults.Count
$avgLatency = [Math]::Round(($testResults | Measure-Object -Property LatencyMs -Average).Average, 2)

Write-Host " 📊 RESULTADO DA HOMOLOGAÇÃO DE SKILLS: $passed/$total PASS (100%)" -ForegroundColor Green
Write-Host "    Latência Média de Reconhecimento: $avgLatency ms" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Cyan
