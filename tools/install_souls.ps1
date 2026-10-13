# tools/install_souls.ps1 — Script Mestre de Instalação, Setup e Simbiose do Souls Engine v7
# Governança: Linhas Vermelhas 1 e 4 do AGENTS.md, ADR-003, ADR-005

[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\SoulsEngine",
    [string]$SourceDir  = "Z:\souls_engine",
    [string]$VhdxPath   = "C:\dev_projects\Souls_FORGE.vhdx",
    [string]$ModelsDir  = "$env:USERPROFILE\llm_local_models",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 🚀 INSTALADOR MESTRE & SETUP SIMBIÓTICO — SOULS ENGINE v7 " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Destino de Instalação: $InstallDir" -ForegroundColor Yellow
Write-Host "Diretório de Modelos:  $ModelsDir" -ForegroundColor Yellow

# Assegura a pasta de modelos universais
if (-not (Test-Path $ModelsDir)) {
    New-Item -ItemType Directory -Path $ModelsDir -Force | Out-Null
}

# ============================================================================
# ETAPA 1: VERIFICAÇÃO E MONTAGEM DO DEV DRIVE REFS (Z:)
# ============================================================================
Write-Host "`n[1/5] Verificando Dev Drive ReFS..." -ForegroundColor Cyan

if (-not (Test-Path "Z:\")) {
    if (Test-Path $VhdxPath) {
        Write-Host "  -> Unidade Z:\ não montada. Montando VHDX: $VhdxPath..." -ForegroundColor Yellow
        Mount-DiskImage -ImagePath $VhdxPath -NoDriveLetter:$false -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 1500
    }
}

if (-not (Test-Path "Z:\")) {
    Write-Error "ERRO CRÍTICO: Não foi possível acessar ou montar a unidade Z:\ (Dev Drive ReFS)."
    exit 1
}
Write-Host "  ✓ Unidade Z:\ (Dev Drive ReFS) acessível e operacional." -ForegroundColor Green

# Assegura a estrutura de dados persistentes no ReFS
$soulsDataDir = "Z:\souls_engine\.souls_data"
$configDir = Join-Path $soulsDataDir "config"
$logsDir = Join-Path $soulsDataDir "logs"
$lanceDir = Join-Path $soulsDataDir "souls_vectors.lance"

foreach ($dir in @($soulsDataDir, $configDir, $logsDir)) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}
Write-Host "  ✓ Estrutura de persistência em $soulsDataDir verificada." -ForegroundColor Green

# ============================================================================
# ETAPA 2: COMPILAÇÃO RELEASE DOS BINÁRIOS NATIVOS
# ============================================================================
Write-Host "`n[2/5] Verificando binários de produção (Release)..." -ForegroundColor Cyan

$releaseTargetDir = "Z:\souls_engine\target\release"
if (-not (Test-Path $releaseTargetDir)) {
    $releaseTargetDir = "Z:\souls_engine\target\x86_64-pc-windows-msvc\release"
}
$serverExeSource = Join-Path $releaseTargetDir "souls_server.exe"

if (-not (Test-Path $serverExeSource) -and -not $SkipBuild) {
    Write-Host "  -> Compilando binário nativo release com MSVC (cargo build --release -p souls_server)..." -ForegroundColor Yellow
    Push-Location "Z:\souls_engine"
    try {
        cargo build --release -p souls_server
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Falha na compilação release do souls_server."
            exit 1
        }
    } finally {
        Pop-Location
    }
}

# Localiza novamente após build
if (-not (Test-Path $serverExeSource)) {
    $alt = "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_server.exe"
    if (Test-Path $alt) {
        $serverExeSource = $alt
    }
}

if (-not (Test-Path $serverExeSource)) {
    Write-Error "Binário release souls_server.exe não encontrado em $releaseTargetDir."
    exit 1
}
Write-Host "  ✓ Binário fonte validado: $serverExeSource" -ForegroundColor Green

# ============================================================================
# ETAPA 3: INSTALAÇÃO EM %LOCALAPPDATA%\SoulsEngine
# ============================================================================
Write-Host "`n[3/5] Instalando arquivos em $InstallDir..." -ForegroundColor Cyan

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Encerra qualquer daemon ativo antes de atualizar arquivos
$running = Get-Process -Name "souls_server", "souls_engine" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "  -> Encerrando daemon ativo antes de atualizar binários..." -ForegroundColor Yellow
    Stop-Process -Name "souls_server", "souls_engine" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 1200
}

# 1. Copia binários com identidade dupla
$installedServerExe = Join-Path $InstallDir "souls_server.exe"
$installedEngineExe = Join-Path $InstallDir "souls_engine.exe"

Copy-Item -Path $serverExeSource -Destination $installedServerExe -Force
Copy-Item -Path $serverExeSource -Destination $installedEngineExe -Force
Write-Host "  ✓ Copiados binários nativos: souls_server.exe e souls_engine.exe." -ForegroundColor Green

# 2. Copia pasta de branding e ícones
$assetsBrandingDir = Join-Path $InstallDir "assets\branding"
if (-not (Test-Path $assetsBrandingDir)) {
    New-Item -ItemType Directory -Path $assetsBrandingDir -Force | Out-Null
}
$srcBranding = "Z:\souls_engine\assets\branding"
if (Test-Path $srcBranding) {
    Copy-Item -Path "$srcBranding\*" -Destination $assetsBrandingDir -Recurse -Force
    Write-Host "  ✓ Ícones e branding instalados em $assetsBrandingDir." -ForegroundColor Green
}

# 3. Copia scripts de controle operacional
$installedLauncher = Join-Path $InstallDir "souls_launcher.ps1"
$installedStartBat = Join-Path $InstallDir "start_souls.bat"
$installedStopBat  = Join-Path $InstallDir "stop_souls.bat"

Copy-Item -Path "Z:\souls_engine\tools\souls_launcher.ps1" -Destination $installedLauncher -Force

# Conteúdo de start_souls.bat adaptado para o local instalado
$startBatContent = @"
@echo off
title Souls Engine v7
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0souls_launcher.ps1"
"@
Set-Content -Path $installedStartBat -Value $startBatContent -Encoding ASCII

# Conteúdo de stop_souls.bat adaptado
$stopBatContent = @"
@echo off
echo Encerrando daemon Souls Engine...
taskkill /F /T /IM souls_server.exe >nul 2>&1
taskkill /F /T /IM souls_engine.exe >nul 2>&1
echo Concluido.
"@
Set-Content -Path $installedStopBat -Value $stopBatContent -Encoding ASCII
Write-Host "  ✓ Scripts operacionais (start_souls.bat, stop_souls.bat) configurados." -ForegroundColor Green

# 4. Configuração global centralizada souls_config.toml
$globalConfigPath = Join-Path $configDir "souls_config.toml"
$formattedModelsDir = $ModelsDir.Replace('\', '/')
$tomlContent = @"
# souls_config.toml — Configuração Global Canônica do Souls Engine v7

[workspace]
root = "Z:/souls_engine"

[storage]
data_dir = "Z:/souls_engine/.souls_data"
database_path = "Z:/souls_engine/.souls_data/souls_memory.db"
lancedb_path = "Z:/souls_engine/.souls_data/souls_vectors.lance"

[models]
models_dir = "$formattedModelsDir"

[server]
host = "127.0.0.1"
port = 9123
streamable_http = true

[hardware]
gpu_target = "RTX 2060m"
vram_ceiling_mb = 5294
"@
Set-Content -Path $globalConfigPath -Value $tomlContent -Encoding UTF8
Write-Host "  ✓ Atualizado arquivo de configuração global em $globalConfigPath." -ForegroundColor Green

# ============================================================================
# ETAPA 4: CRIAÇÃO DE ATALHOS NO MENU INICIAR E ÁREA DE TRABALHO
# ============================================================================
Write-Host "`n[4/5] Criando atalhos no Windows (Menu Iniciar e Desktop)..." -ForegroundColor Cyan

$wshShell = New-Object -ComObject WScript.Shell
$iconPath = Join-Path $assetsBrandingDir "souls_icon_cyan.ico"

# Atalho no Menu Iniciar
$startMenuDir = [Environment]::GetFolderPath('Programs')
$startShortcutPath = Join-Path $startMenuDir "Souls Engine.lnk"
$shortcut = $wshShell.CreateShortcut($startShortcutPath)
$shortcut.TargetPath = "pwsh.exe"
$shortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$installedLauncher`""
$shortcut.WorkingDirectory = $InstallDir
if (Test-Path $iconPath) {
    $shortcut.IconLocation = $iconPath
}
$shortcut.Description = "Souls Engine v7 Bare-Metal Daemon"
$shortcut.WindowStyle = 7 # Minimized
$shortcut.Save()
Write-Host "  ✓ Atalho criado no Menu Iniciar: $startShortcutPath" -ForegroundColor Green

# Atalho na Área de Trabalho (Desktop)
$desktopDir = [Environment]::GetFolderPath('Desktop')
$desktopShortcutPath = Join-Path $desktopDir "Souls Engine.lnk"
$deskShortcut = $wshShell.CreateShortcut($desktopShortcutPath)
$deskShortcut.TargetPath = "pwsh.exe"
$deskShortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$installedLauncher`""
$deskShortcut.WorkingDirectory = $InstallDir
if (Test-Path $iconPath) {
    $deskShortcut.IconLocation = $iconPath
}
$deskShortcut.Description = "Souls Engine v7 Bare-Metal Daemon"
$deskShortcut.WindowStyle = 7
$deskShortcut.Save()
Write-Host "  ✓ Atalho criado no Desktop: $desktopShortcutPath" -ForegroundColor Green

# ============================================================================
# ETAPA 5: INTEGRAÇÃO SIMBIÓTICA COM HERMES AGENT & PLUGINS
# ============================================================================
Write-Host "`n[5/5] Integrando com o Hermes Agent..." -ForegroundColor Cyan

$hermesConfigDir = "$env:LOCALAPPDATA\hermes"
$hermesConfigFile = Join-Path $hermesConfigDir "config.yaml"

if (Test-Path $hermesConfigFile) {
    # Backup preventivo
    $timestamp = (Get-Date).ToString("yyyyMMdd_HHmmss")
    $bakFile = "$hermesConfigFile.bak_$timestamp"
    Copy-Item -Path $hermesConfigFile -Destination $bakFile -Force
    Write-Host "  ✓ Backup do config.yaml criado em: $bakFile" -ForegroundColor Green

    $rawYaml = [System.IO.File]::ReadAllText($hermesConfigFile, [System.Text.Encoding]::UTF8)

    # Injeção segura do bloco mcp_servers.souls_engine se ausente
    if ($rawYaml -notmatch "souls_engine:") {
        $mcpBlock = @"
mcp_servers:
  souls_engine:
    url: "http://127.0.0.1:9123/mcp"
    transport: "http"

"@
        $newYaml = $mcpBlock + $rawYaml
        [System.IO.File]::WriteAllText($hermesConfigFile, $newYaml, [System.Text.Encoding]::UTF8)
        Write-Host "  ✓ Conexão MCP do Souls Engine injetada com sucesso no config.yaml." -ForegroundColor Green
    } else {
        Write-Host "  ✓ souls_engine já configurado no config.yaml do Hermes." -ForegroundColor Green
    }

    # Validação com hermes config check se hermes estiver disponível
    if (Get-Command "hermes" -ErrorAction SilentlyContinue) {
        Write-Host "  -> Validando com 'hermes config check'..." -ForegroundColor Gray
        $checkOutput = hermes config check 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ Hermes config check aprovado com sucesso!" -ForegroundColor Green
        } else {
            Write-Warning "Aviso no hermes config check: $checkOutput"
        }
    }
} else {
    Write-Host "  (i) config.yaml do Hermes não localizado em $hermesConfigDir. Pulando injeção direta." -ForegroundColor Gray
}

# Linkagem de plugins desktop
$hermesPluginsDir = "$env:LOCALAPPDATA\hermes\desktop-plugins"
if (-not (Test-Path $hermesPluginsDir)) {
    New-Item -ItemType Directory -Path $hermesPluginsDir -Force | Out-Null
}

$adaptersRoot = "Z:\souls_engine\adapters"
$plugins = @("hermes_dashboard_souls", "hermes_memory_souls")

foreach ($p in $plugins) {
    $src = Join-Path $adaptersRoot $p
    $dst = Join-Path $hermesPluginsDir $p
    if (Test-Path $src) {
        if (Test-Path $dst) {
            Remove-Item -Path $dst -Recurse -Force
        }
        try {
            New-Item -ItemType Junction -Path $dst -Target $src -Force | Out-Null
            Write-Host "  ✓ Plugin $p conectado via Junction em $dst." -ForegroundColor Green
        } catch {
            Copy-Item -Path $src -Destination $dst -Recurse -Force
            Write-Host "  ✓ Plugin $p copiado para $dst." -ForegroundColor Green
        }
    }
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host " 🎉 INSTALAÇÃO CONCLUÍDA COM SUCESSO!                      " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "Executável Principal: $installedEngineExe" -ForegroundColor Cyan
Write-Host "Para Iniciar: Dê 2 cliques no atalho 'Souls Engine' no Desktop ou Menu Iniciar." -ForegroundColor Yellow
Write-Host "Dashboard REST: http://127.0.0.1:9123/dashboard" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Green
