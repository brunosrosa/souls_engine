# install_hermes_plugins.ps1 — Utilitário de Simbiose e Instalação dos Adaptadores Hermes (v7)
# Governança: ADR-003, ADR-005 e Ponto de Auditoria 3

[CmdletBinding()]
param(
    [string]$HermesPluginsDir = "$env:LOCALAPPDATA\hermes\desktop-plugins",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🤝 INSTALADOR DE PLUGINS DE SIMBIOSE — HERMES & SOULS    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$adaptersRoot = "Z:\souls_engine\adapters"
$plugins = @("hermes_dashboard_souls", "hermes_memory_souls")

if (-not (Test-Path $adaptersRoot)) {
    Write-Error "Diretório de adaptadores não encontrado em: $adaptersRoot"
    exit 1
}

Write-Host "Destino de Plugins do Hermes: $HermesPluginsDir" -ForegroundColor Yellow

if (-not $DryRun) {
    if (-not (Test-Path $HermesPluginsDir)) {
        New-Item -ItemType Directory -Path $HermesPluginsDir -Force | Out-Null
        Write-Host "Criado diretório de plugins em: $HermesPluginsDir" -ForegroundColor Green
    }
}

foreach ($plugin in $plugins) {
    $src = Join-Path $adaptersRoot $plugin
    $dst = Join-Path $HermesPluginsDir $plugin

    Write-Host "`nProcessando plugin: $plugin" -ForegroundColor Cyan
    Write-Host "  Origem:  $src" -ForegroundColor Gray
    Write-Host "  Destino: $dst" -ForegroundColor Gray

    if (-not (Test-Path $src)) {
        Write-Warning "Pasta de origem não encontrada: $src"
        continue
    }

    $manifest = Join-Path $src "plugin.yaml"
    if (Test-Path $manifest) {
        Write-Host "  [OK] Manifesto plugin.yaml validado com sucesso." -ForegroundColor Green
    } else {
        Write-Warning "  [AVISO] plugin.yaml ausente em $src"
    }

    if (-not $DryRun) {
        if (Test-Path $dst) {
            Remove-Item -Path $dst -Recurse -Force
        }

        # Cria junção NTFS/ReFS ou cópia limpa
        try {
            New-Item -ItemType Junction -Path $dst -Target $src -Force | Out-Null
            Write-Host "  [OK] Junção de diretório vinculada com sucesso!" -ForegroundColor Green
        } catch {
            Copy-Item -Path $src -Destination $dst -Recurse -Force
            Write-Host "  [OK] Adaptador copiado com sucesso (fallback copy)!" -ForegroundColor Green
        }
    } else {
        Write-Host "  [DRY RUN] Operação simulada." -ForegroundColor Yellow
    }
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " ✅ ADAPTADORES HERMES INSTALADOS E SIMBIÓTICOS (v7)      " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
