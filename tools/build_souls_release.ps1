# tools/build_souls_release.ps1 — Compilação Otimizada de Produção para Windows x86_64
Write-Host "=== Compilando Souls Engine v7 em Modo Release (--release) ===" -ForegroundColor Cyan

cargo build --release -p souls_server

if ($LASTEXITCODE -eq 0) {
    $targetDir = "Z:\souls_engine\target\release"
    if (-not (Test-Path $targetDir)) {
        $targetDir = "Z:\souls_engine\target\x86_64-pc-windows-msvc\release"
    }

    $serverExe = Join-Path $targetDir "souls_server.exe"
    $engineExe = Join-Path $targetDir "souls_engine.exe"

    if (Test-Path $serverExe) {
        Copy-Item -Path $serverExe -Destination $engineExe -Force
        Write-Host "✓ Binários de Produção gerados com sucesso:" -ForegroundColor Green
        Write-Host "  - $serverExe" -ForegroundColor Yellow
        Write-Host "  - $engineExe" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ Falha na compilação de release." -ForegroundColor Red
}
