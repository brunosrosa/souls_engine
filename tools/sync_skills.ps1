# sync_skills.ps1 — Sincronizador de Skills para Hermes Agent e Antigravity Global
# Linha de Base v7 Canônica (ADR-005 Emenda 3.3)

[CmdletBinding()]
param(
    [string]$SourceDir = "Z:\souls_engine\.agents\skills",
    [string]$HermesSkillsDir = "$env:LOCALAPPDATA\hermes\skills",
    [string]$GeminiConfigSkillsDir = "$env:USERPROFILE\.gemini\config\skills"
)

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🚀 Sincronizando Skills Canônicas do Souls Engine (v7) " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $SourceDir)) {
    Write-Error "Diretório fonte de skills não encontrado: $SourceDir"
    exit 1
}

$skills = Get-ChildItem -Path $SourceDir -Directory
Write-Host "Encontradas $($skills.Count) skills em $SourceDir" -ForegroundColor Green

# 1. Destino Hermes: C:\Users\rosas\AppData\Local\hermes\skills
if (-not (Test-Path $HermesSkillsDir)) {
    New-Item -ItemType Directory -Path $HermesSkillsDir -Force | Out-Null
}

# 2. Destino Antigravity Global: ~/.gemini/config/skills
if (-not (Test-Path $GeminiConfigSkillsDir)) {
    New-Item -ItemType Directory -Path $GeminiConfigSkillsDir -Force | Out-Null
}

foreach ($skill in $skills) {
    $skillName = $skill.Name
    Write-Host "  -> Sincronizando skill: $skillName" -ForegroundColor Yellow

    # Determinar subpasta no Hermes
    $hermesTargetSubdir = if ($skillName.StartsWith("obsidian-")) {
        Join-Path $HermesSkillsDir "note-taking\$skillName"
    } else {
        Join-Path $HermesSkillsDir "souls-engine\$skillName"
    }

    # Destino Antigravity Global
    $geminiTargetSubdir = Join-Path $GeminiConfigSkillsDir $skillName

    # Criar diretórios se não existirem
    if (-not (Test-Path $hermesTargetSubdir)) { New-Item -ItemType Directory -Path $hermesTargetSubdir -Force | Out-Null }
    if (-not (Test-Path $geminiTargetSubdir)) { New-Item -ItemType Directory -Path $geminiTargetSubdir -Force | Out-Null }

    # Copiar SKILL.md garantindo LF puro
    $sourceSkillFile = Join-Path $skill.FullName "SKILL.md"
    if (Test-Path $sourceSkillFile) {
        $content = [System.IO.File]::ReadAllText($sourceSkillFile)
        $cleanContent = $content.Replace("`r`n", "`n")
        $utf8NoBom = [System.Text.UTF8Encoding]::new($false)

        $hermesSkillFile = Join-Path $hermesTargetSubdir "SKILL.md"
        [System.IO.File]::WriteAllBytes($hermesSkillFile, $utf8NoBom.GetBytes($cleanContent))

        $geminiSkillFile = Join-Path $geminiTargetSubdir "SKILL.md"
        [System.IO.File]::WriteAllBytes($geminiSkillFile, $utf8NoBom.GetBytes($cleanContent))
    }
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " ✅ Sincronização concluída com sucesso no Hermes e Antigravity! " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
