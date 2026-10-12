@echo off
title Parar Souls Engine v7
powershell -NoProfile -ExecutionPolicy Bypass -Command "Stop-Process -Name 'souls_server' -Force -ErrorAction SilentlyContinue; Write-Host 'Daemon souls_server.exe finalizado com sucesso.' -ForegroundColor Green"
pause
