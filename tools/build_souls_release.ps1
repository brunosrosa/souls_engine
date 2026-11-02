# tools/build_souls_release.ps1 — Compilação Otimizada de Produção para Windows x86_64 (LLD + sccache)
Write-Host "=== Compilando Souls Engine v7 em Modo Release (--release) com sccache & rust-lld ===" -ForegroundColor Cyan

# Garante que o sccache server está ativo
& "C:\Users\rosas\.cargo\bin\sccache.exe" --start-server | Out-Null

# Compilação Release via toolchain estável MSVC com rust-lld e sccache
rustup run stable cargo build --release -p souls_server

if ($LASTEXITCODE -eq 0) {
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

        $engineExe = Join-Path $relDir "souls_engine.exe"
        $relServerExe = Join-Path $relDir "souls_server.exe"
        $msvcEngineExe = Join-Path $msvcDir "souls_engine.exe"

        Copy-Item -Path $serverExe -Destination $engineExe -Force
        Copy-Item -Path $serverExe -Destination $relServerExe -Force
        Copy-Item -Path $serverExe -Destination $msvcEngineExe -Force

        Copy-Item -Path $serverExe -Destination (Join-Path $appDataDir "souls_engine.exe") -Force
        Copy-Item -Path $serverExe -Destination (Join-Path $appDataDir "souls_server.exe") -Force

        Write-Host "✓ Binários de Produção sincronizados a partir de: $serverExe" -ForegroundColor Green
        Write-Host "  - $relServerExe" -ForegroundColor Yellow
        Write-Host "  - $engineExe" -ForegroundColor Yellow
        Write-Host "  - $msvcEngineExe" -ForegroundColor Yellow
        Write-Host "✓ Atualizado binários em: $appDataDir" -ForegroundColor Green

        # Higiene automática do /target para evitar acúmulo de dezenas de GBs
        Write-Host "=== Higiene pós-build do target/ (expurgo de artefatos temporários de debug) ===" -ForegroundColor Cyan
        Remove-Item -Recurse -Force -LiteralPath "Z:\souls_engine\target\debug" -ErrorAction SilentlyContinue
        Remove-Item -Recurse -Force -LiteralPath "Z:\souls_engine\target\x86_64-pc-windows-msvc\debug" -ErrorAction SilentlyContinue
        Write-Host "✓ Target limpo e enxuto." -ForegroundColor Green

        # Exibe estatísticas do sccache
        Write-Host "=== Estatísticas do Cache sccache ===" -ForegroundColor Cyan
        & "C:\Users\rosas\.cargo\bin\sccache.exe" --show-stats
    }
} else {
    Write-Host "❌ Falha na compilação de release." -ForegroundColor Red
}
