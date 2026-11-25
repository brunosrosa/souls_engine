# tools/sync_skills.ps1
# Sincronizador Seguro de Skills Canonicas do Souls Engine (Hash-Tracked / Zero Overwrite Silencioso)

param(
    [switch]$Force,
    [switch]$DryRun,
    [string]$SourceBase = "Z:\souls_engine\.agents\skills",
    [string]$DestBase = "$env:LOCALAPPDATA\hermes\skills\souls-engine"
)

$ErrorActionPreference = "Stop"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host "  Sincronizando Skills Canonicas do Souls Engine (Modo Seguro)    " -ForegroundColor Cyan
Write-Host "==================================================================" -ForegroundColor Cyan

if (-not (Test-Path $SourceBase)) {
    Write-Host "ERRO: Diretorio fonte nao encontrado em $SourceBase" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $DestBase)) {
    if (-not $DryRun) {
        New-Item -ItemType Directory -Path $DestBase -Force | Out-Null
    }
    Write-Host "[+] Diretorio destino criado: $DestBase" -ForegroundColor Yellow
}

$ManifestPath = Join-Path $DestBase ".souls_sync_manifest.json"
$manifestFiles = @{}
if (Test-Path $ManifestPath) {
    try {
        $rawManifest = Get-Content -Path $ManifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
        if ($null -ne $rawManifest -and $null -ne $rawManifest.files) {
            foreach ($prop in $rawManifest.files.PSObject.Properties) {
                $manifestFiles[$prop.Name] = [string]$prop.Value
            }
        }
    } catch {
        Write-Host "[WARN] Manifesto anterior ilegivel; tratando divergencias de destino de forma conservadora." -ForegroundColor Yellow
    }
}

function Get-LfText {
    param([string]$Path)
    $raw = [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8)
    return $raw.Replace("`r`n", "`n")
}

function Get-TextSha256 {
    param([string]$Text)
    $bytes = $Utf8NoBom.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hashBytes = $sha.ComputeHash($bytes)
        return ([System.BitConverter]::ToString($hashBytes)).Replace("-", "").ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function Merge-LocalDirectives {
    param(
        [string]$SourceText,
        [string]$DestText
    )
    $pattern = '(?s)<!--\s*LOCAL_DIRECTIVES_START\s*-->.*?<!--\s*LOCAL_DIRECTIVES_END\s*-->'
    $destMatch = [regex]::Match($DestText, $pattern)
    if (-not $destMatch.Success) {
        return $SourceText
    }
    $localBlock = $destMatch.Value
    $srcMatch = [regex]::Match($SourceText, $pattern)
    if ($srcMatch.Success) {
        return [regex]::Replace($SourceText, $pattern, [System.Text.RegularExpressions.MatchEvaluator]{ param($m) $localBlock })
    }
    $trimmed = $SourceText.TrimEnd("`n")
    return "$trimmed`n`n$localBlock`n"
}

$synced = 0
$preserved = 0
$uptodate = 0

$skillDirs = Get-ChildItem -Path $SourceBase -Directory | Sort-Object Name
foreach ($skill in $skillDirs) {
    # Nunca tocar em artefatos ou pastas de quarentena
    if ($skill.Name -match '(?i)quarantine') { continue }

    $srcFiles = Get-ChildItem -Path $skill.FullName -Recurse -File | Sort-Object FullName
    foreach ($srcFile in $srcFiles) {
        $relInsideSkill = $srcFile.FullName.Substring($skill.FullName.Length).TrimStart('\', '/')
        $relKey = ($skill.Name + "/" + ($relInsideSkill -replace '\\', '/'))
        if ($relKey -match '(?i)quarantine|\.souls_sync_manifest\.json$|\.bak$') { continue }

        $destFilePath = Join-Path (Join-Path $DestBase $skill.Name) $relInsideSkill
        $destDir = Split-Path -Path $destFilePath -Parent

        $srcText = Get-LfText -Path $srcFile.FullName
        $srcHash = Get-TextSha256 -Text $srcText

        if (-not (Test-Path $destFilePath)) {
            if (-not $DryRun) {
                if (-not (Test-Path $destDir)) {
                    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
                }
                [System.IO.File]::WriteAllText($destFilePath, $srcText, $Utf8NoBom)
                $manifestFiles[$relKey] = $srcHash
            }
            Write-Host "  [NEW SYNCED] $relKey" -ForegroundColor Green
            $synced++
            continue
        }

        $destText = Get-LfText -Path $destFilePath
        $destHash = Get-TextSha256 -Text $destText
        if ($destHash -eq $srcHash) {
            $manifestFiles[$relKey] = $srcHash
            $uptodate++
            continue
        }

        $prevHash = if ($manifestFiles.ContainsKey($relKey)) { $manifestFiles[$relKey] } else { $null }
        $isLocallyEdited = ($null -eq $prevHash) -or ($destHash -ne $prevHash)

        if ($isLocallyEdited -and -not $Force) {
            Write-Host "  [PRESERVED LOCAL EDIT] $relKey (use -Force para sobrescrever mantendo .bak e LOCAL_DIRECTIVES)" -ForegroundColor Yellow
            $preserved++
            continue
        }

        if ($Force -and $isLocallyEdited) {
            $mergedText = Merge-LocalDirectives -SourceText $srcText -DestText $destText
            $mergedHash = Get-TextSha256 -Text $mergedText
            if (-not $DryRun) {
                $bakPath = "$destFilePath.bak"
                [System.IO.File]::WriteAllText($bakPath, $destText, $Utf8NoBom)
                [System.IO.File]::WriteAllText($destFilePath, $mergedText, $Utf8NoBom)
                $manifestFiles[$relKey] = $mergedHash
            }
            Write-Host "  [FORCED WITH BACKUP] $relKey (.bak salvo)" -ForegroundColor Cyan
            $synced++
            continue
        }

        if (-not $DryRun) {
            [System.IO.File]::WriteAllText($destFilePath, $srcText, $Utf8NoBom)
            $manifestFiles[$relKey] = $srcHash
        }
        Write-Host "  [UPDATED] $relKey" -ForegroundColor Green
        $synced++
    }
}

if (-not $DryRun) {
    $orderedFiles = [ordered]@{}
    foreach ($k in ($manifestFiles.Keys | Sort-Object)) {
        $orderedFiles[$k] = $manifestFiles[$k]
    }
    $manifestObj = [ordered]@{
        schema_version = 1
        updated_at_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        source_base = $SourceBase
        files = $orderedFiles
    }
    $manifestJson = ($manifestObj | ConvertTo-Json -Depth 5).Replace("`r`n", "`n") + "`n"
    [System.IO.File]::WriteAllText($ManifestPath, $manifestJson, $Utf8NoBom)
}

Write-Host "`nConcluido: $synced sincronizados | $uptodate atualizados | $preserved preservados localmente (DryRun=$($DryRun.IsPresent))." -ForegroundColor Cyan
