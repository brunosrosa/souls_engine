@echo off
title Parar Souls Engine v7
powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Name 'souls_server', 'souls_engine' -Force -ErrorAction SilentlyContinue; Write-Host 'Daemons souls_server / souls_engine finalizados com sucesso.' -ForegroundColor Green"
pause
