# tools/build_souls_release.ps1 — Compilação Otimizada de Produção para Windows x86_64
Write-Host "=== Compilando Souls Engine v7 em Modo Release (--release) ===" -ForegroundColor Cyan

cargo build --release -p souls_server

if ($LASTEXITCODE -eq 0) {
    $candidates = @(
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_server.exe",
        "Z:\souls_engine\target\release\souls_server.exe"
    )
    $serverExe = $candidates | Where-Object { Test-Path $_ } | Sort-Object { (Get-Item $_).LastWriteTime } -Descending | Select-Object -First 1

    if ($serverExe) {
        $engineExe = Join-Path "Z:\souls_engine\target\release" "souls_engine.exe"
        $relServerExe = Join-Path "Z:\souls_engine\target\release" "souls_server.exe"
        $msvcEngineExe = Join-Path "Z:\souls_engine\target\x86_64-pc-windows-msvc\release" "souls_engine.exe"

        Copy-Item -Path $serverExe -Destination $engineExe -Force
        Copy-Item -Path $serverExe -Destination $relServerExe -Force
        Copy-Item -Path $serverExe -Destination $msvcEngineExe -Force

        Write-Host "✓ Binários de Produção sincronizados a partir de: $serverExe" -ForegroundColor Green
        Write-Host "  - $relServerExe" -ForegroundColor Yellow
        Write-Host "  - $engineExe" -ForegroundColor Yellow
        Write-Host "  - $msvcEngineExe" -ForegroundColor Yellow

        $appDataDir = Join-Path $env:LOCALAPPDATA "SoulsEngine"
        if (Test-Path $appDataDir) {
            Copy-Item -Path $serverExe -Destination (Join-Path $appDataDir "souls_engine.exe") -Force
            Copy-Item -Path $serverExe -Destination (Join-Path $appDataDir "souls_server.exe") -Force
            Write-Host "✓ Atualizado binários em: $appDataDir" -ForegroundColor Green
        }
    }
} else {
    Write-Host "❌ Falha na compilação de release." -ForegroundColor Red
}
