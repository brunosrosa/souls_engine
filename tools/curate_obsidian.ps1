# curate_obsidian.ps1 — Esteira de Curadoria Incremental do Cofre Obsidian (v7)
# Governança: AGENTS.md do cofre e ADR-005 Emenda 3.3

[CmdletBinding()]
param(
    [string]$VaultPath = "C:\Users\rosas\OneDrive\Documentos\Obisidian DB",
    [int]$BatchSize = 2,
    [switch]$StatusOnly,
    [switch]$ProcessBatch
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🏛️ ESTEIRA DE CURADORIA INCREMENTAL — OBSIDIAN DB (v7)  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$rawDir = Join-Path $VaultPath "raw"
$wikiDir = Join-Path $VaultPath "wiki"
$manifestPath = Join-Path $wikiDir "raw_manifest.json"

if (-not (Test-Path $rawDir)) {
    Write-Error "Diretório raw/ não encontrado em: $rawDir"
    exit 1
}
if (-not (Test-Path $wikiDir)) {
    Write-Error "Diretório wiki/ não encontrado em: $wikiDir"
    exit 1
}

# 1. Carregar ou inicializar manifesto
$manifest = @{}
if (Test-Path $manifestPath) {
    try {
        $jsonRaw = Get-Content -Path $manifestPath -Raw
        $manifestObj = $jsonRaw | ConvertFrom-Json
        # Converter array de objetos para hashtable indexada por relative_path
        if ($manifestObj -is [Array]) {
            foreach ($item in $manifestObj) {
                $manifest[$item.relative_path] = $item
            }
        } elseif ($manifestObj.files) {
            foreach ($item in $manifestObj.files) {
                $manifest[$item.relative_path] = $item
            }
        }
    } catch {
        Write-Warning "Falha ao ler raw_manifest.json existente. Criando novo índice."
        $manifest = @{}
    }
}

Write-Host "Escaneando diretório bruto: $rawDir ..." -ForegroundColor Yellow

# 2. Varrer arquivos de raw/ excluindo metadados de plugins (.space, .obsidian, etc.)
$rawFiles = Get-ChildItem -Path $rawDir -Recurse -File | Where-Object {
    $_.FullName -notmatch '[\\/]\.(space|obsidian|trash|git)[\\/]' -and
    $_.Extension -in @(".md", ".txt", ".json", ".pdf", ".csv")
}

$today = (Get-Date).ToString("yyyy-MM-dd")
$newCount = 0
$updatedCount = 0

foreach ($file in $rawFiles) {
    $relPath = $file.FullName.Substring($rawDir.Length + 1).Replace("\", "/")
    
    # Calcular SHA-256
    $hash = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash

    # Determinar domínio e política de privacidade
    $domain = "misc"
    $privacy = "STANDARD"

    if ($relPath.StartsWith("financas/")) {
        $domain = "financas"
        $privacy = "LOCAL_ONLY"
    } elseif ($relPath.StartsWith("saude/")) {
        $domain = "saude"
        $privacy = "LOCAL_ONLY"
    } elseif ($relPath.StartsWith("projetos/")) {
        $domain = "projetos"
    } elseif ($relPath.StartsWith("deep-researchs/")) {
        $domain = "research"
    } elseif ($relPath.StartsWith("notas_rapidas/")) {
        $domain = "notas_rapidas"
    }

    if (-not $manifest.ContainsKey($relPath)) {
        $manifest[$relPath] = [PSCustomObject]@{
            relative_path    = $relPath
            domain           = $domain
            privacy          = $privacy
            sha256           = $hash
            status           = "PENDING"
            discovered_at    = $today
            last_curated_at  = $null
            generated_notes  = @()
        }
        $newCount++
    } else {
        $entry = $manifest[$relPath]
        if ($entry.sha256 -ne $hash) {
            $entry.sha256 = $hash
            $entry.status = "PENDING"
            $entry.updated_at = $today
            $updatedCount++
        }
    }
}

# 3. Salvar manifesto atualizado em wiki/raw_manifest.json
$manifestList = $manifest.Values | Sort-Object relative_path
$manifestOutput = [PSCustomObject]@{
    version       = "1.0.0"
    last_scan     = $today
    vault_root    = $VaultPath
    total_files   = $manifestList.Count
    files         = $manifestList
}

$manifestJson = $manifestOutput | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, [System.Text.UTF8Encoding]::new($false))

# 4. Apresentar sumário analítico
$pendingItems = $manifestList | Where-Object { $_.status -eq "PENDING" }
$curatedItems = $manifestList | Where-Object { $_.status -eq "CURATED" }
$localOnlyItems = $manifestList | Where-Object { $_.privacy -eq "LOCAL_ONLY" }

Write-Host ""
Write-Host "📊 SUMÁRIO DO MANIFESTO DE CURADORIA:" -ForegroundColor Green
Write-Host "   Total de Arquivos em raw/: $($manifestList.Count)" -ForegroundColor White
Write-Host "   Novos Descobertos:         $newCount" -ForegroundColor Cyan
Write-Host "   Modificados (Re-hash):     $updatedCount" -ForegroundColor Yellow
Write-Host "   Status PENDING:            $($pendingItems.Count)" -ForegroundColor Yellow
Write-Host "   Status CURATED:            $($curatedItems.Count)" -ForegroundColor Green
Write-Host "   Privacidade LOCAL_ONLY:    $($localOnlyItems.Count) (Finanças e Saúde)" -ForegroundColor Magenta
Write-Host ""

if ($StatusOnly) {
    Write-Host "Operação concluída em modo StatusOnly. Nenhum lote despachado." -ForegroundColor Gray
    exit 0
}

# 5. Seleção de Lote Seguro (BatchSize <= 5 a 10)
$actualBatchSize = [Math]::Min($BatchSize, 10)
$nextBatch = $pendingItems | Select-Object -First $actualBatchSize

if ($nextBatch.Count -eq 0) {
    Write-Host "✅ Todos os arquivos do diretório raw/ já foram curados! Nada pendente." -ForegroundColor Green
} else {
    Write-Host "🎯 PRÓXIMO LOTE PARA CURADORIA ($($nextBatch.Count) arquivos):" -ForegroundColor Cyan
    foreach ($item in $nextBatch) {
        $lockTag = if ($item.privacy -eq "LOCAL_ONLY") { "[LOCAL_ONLY 🔒]" } else { "[STANDARD 🌐]" }
        Write-Host "   - $lockTag $($item.relative_path) (Domínio: $($item.domain))" -ForegroundColor White
    }
    Write-Host ""
    Write-Host "💡 Diretriz de Curadoria:" -ForegroundColor Yellow
    Write-Host "   1. Ler cada arquivo em modo Read-Only." -ForegroundColor Gray
    Write-Host "   2. Gerar verbetes atômicos em wiki/concepts/ ou wiki/entities/ com frontmatter YAML." -ForegroundColor Gray
    Write-Host "   3. Registrar no wiki/log.md e marcar como CURATED no raw_manifest.json." -ForegroundColor Gray
}

Write-Host "==========================================================" -ForegroundColor Cyan
