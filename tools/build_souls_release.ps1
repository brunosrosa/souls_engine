# tools/build_souls_release.ps1 — Compilação Otimizada de Produção para Windows x86_64 (LLD + sccache)
$ErrorActionPreference = "Stop"

Write-Host "=== Compilando Souls Engine v7 em Modo Release (--release) com sccache & rust-lld ===" -ForegroundColor Cyan

$sccacheExe = "C:\Users\rosas\.cargo\bin\sccache.exe"
$env:SCCACHE_DIR = "Z:\souls_engine\.souls_data\sccache"
$env:SCCACHE_CACHE_SIZE = "15G"
$env:SCCACHE_IDLE_TIMEOUT = "0"
$env:RUSTC_WRAPPER = $sccacheExe

if (-not (Test-Path $env:SCCACHE_DIR)) {
    New-Item -ItemType Directory -Path $env:SCCACHE_DIR -Force | Out-Null
}

function Copy-FileWithRetry {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination,
        [int]$MaxAttempts = 10,
        [int]$DelayMs = 500
    )
    if ([System.IO.Path]::GetFullPath($Source) -ieq [System.IO.Path]::GetFullPath($Destination)) {
        return
    }
    for ($attempt = 1; $attempt -le $MaxAttempts; $attempt++) {
        try {
            Copy-Item -Path $Source -Destination $Destination -Force
            return
        } catch {
            if ($attempt -eq $MaxAttempts) {
                throw "Falha ao copiar '$Source' -> '$Destination' após $MaxAttempts tentativas: $($_.Exception.Message)"
            }
            Start-Sleep -Milliseconds $DelayMs
        }
    }
}

# Reinicia o daemon sccache com o ambiente explícito no Dev Drive ReFS (evita fallback em C:\AppData\Local\Mozilla\sccache)
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& $sccacheExe --stop-server 2>$null | Out-Null
& $sccacheExe --start-server | Out-Null

# Compilação Release via toolchain estável MSVC com rust-lld e sccache
rustup run stable cargo build --release -p souls_server
$buildExit = $LASTEXITCODE
$ErrorActionPreference = $prevEap

if ($buildExit -eq 0) {
    $candidates = @(
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_server.exe",
        "Z:\souls_engine\target\release\souls_server.exe"
    )
    $serverExe = $candidates | Where-Object { Test-Path $_ } | Sort-Object { (Get-Item $_).LastWriteTime } -Descending | Select-Object -First 1

    if ($serverExe) {
        $relDir = "Z:\souls_engine\target\release"
        $msvcDir = "Z:\souls_engine\target\x86_64-pc-windows-msvc\release"
        $appDataDir = Join-Path $env:LOCALAPPDATA "SoulsEngine"

        if (-not (Test-Path $relDir)) { New-Item -ItemType Directory -Path $relDir -Force | Out-Null }
        if (-not (Test-Path $appDataDir)) { New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null }

        # Encerra daemon ativo antes de sobrescrever binários instalados e aguarda liberação de lock
        $running = Get-Process -Name "souls_server", "souls_engine" -ErrorAction SilentlyContinue
        if ($running) {
            Write-Host "-> Encerrando daemon ativo antes de sincronizar binários..." -ForegroundColor Yellow
            Stop-Process -Name "souls_server", "souls_engine" -Force -ErrorAction SilentlyContinue
            $deadline = (Get-Date).AddSeconds(15)
            while ((Get-Date) -lt $deadline) {
                if (-not (Get-Process -Name "souls_server", "souls_engine" -ErrorAction SilentlyContinue)) {
                    break
                }
                Start-Sleep -Milliseconds 300
            }
            if (Get-Process -Name "souls_server", "souls_engine" -ErrorAction SilentlyContinue) {
                Write-Host "❌ Timeout aguardando encerramento de souls_server/souls_engine." -ForegroundColor Red
                exit 1
            }
            Start-Sleep -Milliseconds 500
        }

        $engineExe = Join-Path $relDir "souls_engine.exe"
        $relServerExe = Join-Path $relDir "souls_server.exe"
        $msvcEngineExe = Join-Path $msvcDir "souls_engine.exe"
        $installedEngineExe = Join-Path $appDataDir "souls_engine.exe"
        $installedServerExe = Join-Path $appDataDir "souls_server.exe"

        Copy-FileWithRetry -Source $serverExe -Destination $engineExe
        Copy-FileWithRetry -Source $serverExe -Destination $relServerExe
        Copy-FileWithRetry -Source $serverExe -Destination $msvcEngineExe
        Copy-FileWithRetry -Source $serverExe -Destination $installedEngineExe
        Copy-FileWithRetry -Source $serverExe -Destination $installedServerExe

        $srcLauncher = "Z:\souls_engine\tools\souls_launcher.ps1"
        if (Test-Path $srcLauncher) {
            Copy-FileWithRetry -Source $srcLauncher -Destination (Join-Path $appDataDir "souls_launcher.ps1")
        }
        $srcBranding = "Z:\souls_engine\assets\branding"
        $destBranding = Join-Path $appDataDir "assets\branding"
        if (Test-Path $srcBranding) {
            if (-not (Test-Path $destBranding)) { New-Item -ItemType Directory -Path $destBranding -Force | Out-Null }
            Copy-Item -Path "$srcBranding\*" -Destination $destBranding -Recurse -Force
        }

        $srcHash = (Get-FileHash -Path $serverExe -Algorithm SHA256).Hash
        $instEngineHash = (Get-FileHash -Path $installedEngineExe -Algorithm SHA256).Hash
        $instServerHash = (Get-FileHash -Path $installedServerExe -Algorithm SHA256).Hash
        if ($srcHash -ne $instEngineHash -or $srcHash -ne $instServerHash) {
            Write-Host "❌ Divergência SHA-256 nos binários instalados!" -ForegroundColor Red
            exit 1
        }
        $bytesLen = (Get-Item $installedEngineExe).Length

        Write-Host "✓ Binários de Produção sincronizados a partir de: $serverExe" -ForegroundColor Green
        Write-Host "  - $relServerExe" -ForegroundColor Yellow
        Write-Host "  - $engineExe" -ForegroundColor Yellow
        Write-Host "  - $msvcEngineExe" -ForegroundColor Yellow
        Write-Host "✓ Atualizado e verificado em: $appDataDir ($bytesLen bytes, SHA256 $($srcHash.Substring(0, 16))...)" -ForegroundColor Green

        # Preserva cache incremental em dev/test para acelerar turnos de trabalho
        Write-Host "✓ Cache incremental de dev/test preservado." -ForegroundColor Green

        # Exibe estatísticas do sccache
        Write-Host "=== Estatísticas do Cache sccache ===" -ForegroundColor Cyan
        $ErrorActionPreference = "Continue"
        & $sccacheExe --show-stats
    } else {
        Write-Host "❌ Nenhum binário souls_server.exe encontrado após build." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ Falha na compilação de release." -ForegroundColor Red
    exit 1
}
