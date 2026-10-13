# souls_launcher.ps1 — Windows Systray Companion Desacoplado do Souls Engine v7
# Governança: Linha Vermelha 1 do AGENTS.md (souls_server.exe permanece 100% headless)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$ErrorActionPreference = "SilentlyContinue"

$brandingDir = "Z:\souls_engine\assets\branding"
$serverUrl = "http://127.0.0.1:9123"

# 1. Garante que o Dev Drive ReFS esteja montado
$vhdxPath = "C:\dev_projects\Souls_FORGE.vhdx"
if (-not (Test-Path "Z:\") -and (Test-Path $vhdxPath)) {
    Write-Host "Dev Drive ReFS (Z:) não encontrado. Montando VHDX: $vhdxPath..." -ForegroundColor Yellow
    Mount-DiskImage -ImagePath $vhdxPath -NoDriveLetter:$false -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 1200
}

# 2. Inicia souls_engine.exe ou souls_server.exe se não estiver em execução
$running = Get-Process -Name "souls_engine", "souls_server" -ErrorAction SilentlyContinue
if (-not $running) {
    $candidates = @(
        "$env:LOCALAPPDATA\Programs\SoulsEngine\souls_engine.exe",
        "$env:LOCALAPPDATA\Programs\SoulsEngine\souls_server.exe",
        "$env:LOCALAPPDATA\souls_engine\souls_engine.exe",
        "$env:LOCALAPPDATA\souls_engine\souls_server.exe",
        "Z:\souls_engine\target\release\souls_engine.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_engine.exe",
        "Z:\souls_engine\target\release\souls_server.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_server.exe",
        "Z:\souls_engine\target\debug\souls_engine.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\debug\souls_server.exe",
        "Z:\souls_engine\target\debug\souls_server.exe"
    )
    $serverExe = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($serverExe) {
        $exeName = [System.IO.Path]::GetFileName($serverExe)
        Write-Host "Iniciando daemon headless $exeName..." -ForegroundColor Cyan
        Start-Process -FilePath $serverExe -WindowStyle Hidden
        Start-Sleep -Milliseconds 800
    }
}
# Carrega ícones multi-resolução
$icons = @{
    "cyan"   = [System.Drawing.Icon]::new((Join-Path $brandingDir "souls_icon_cyan.ico"))
    "green"  = [System.Drawing.Icon]::new((Join-Path $brandingDir "souls_icon_green.ico"))
    "amber"  = [System.Drawing.Icon]::new((Join-Path $brandingDir "souls_icon_amber.ico"))
    "red"    = [System.Drawing.Icon]::new((Join-Path $brandingDir "souls_icon_red.ico"))
    "purple" = [System.Drawing.Icon]::new((Join-Path $brandingDir "souls_icon_purple.ico"))
}

$notifyIcon = New-Object System.Windows.Forms.NotifyIcon
$notifyIcon.Icon = $icons["cyan"]
$notifyIcon.Text = "Souls Engine v7 (Iniciando...)"
$notifyIcon.Visible = $true

# Menu de Contexto
$contextMenu = New-Object System.Windows.Forms.ContextMenuStrip

$itemOpenDash = $contextMenu.Items.Add("🌐 Abrir Web Dashboard")
$itemOpenDash.add_Click({
    Start-Process "$serverUrl/dashboard"
})

$itemSync = $contextMenu.Items.Add("🔄 Sincronizar Cloud Arena")
$itemSync.add_Click({
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/cloud/sync" -Method Post | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Cloud Arena sincronizada com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Falha ao sincronizar Cloud Arena.", [System.Windows.Forms.ToolTipIcon]::Warning)
    }
})

$itemCheckpoint = $contextMenu.Items.Add("🧹 Compactar WAL (Checkpoint)")
$itemCheckpoint.add_Click({
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/system/checkpoint" -Method Post | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "WAL SQLite compactado com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {}
})

$itemStopServer = $contextMenu.Items.Add("🛑 Parar Servidor Daemon")
$itemStopServer.add_Click({
    Stop-Process -Name "souls_server", "souls_engine" -Force -ErrorAction SilentlyContinue
    $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Daemon souls_server/souls_engine encerrado.", [System.Windows.Forms.ToolTipIcon]::Info)
})

$contextMenu.Items.Add("-") | Out-Null

$itemExit = $contextMenu.Items.Add("❌ Sair do Companion")
$itemExit.add_Click({
    $notifyIcon.Visible = $false
    $timer.Stop()
    [System.Windows.Forms.Application]::Exit()
})

$notifyIcon.ContextMenuStrip = $contextMenu

# Duplo clique abre dashboard
$notifyIcon.add_DoubleClick({
    Start-Process "$serverUrl/dashboard"
})

# Timer de atualização de telemetria (2000 ms)
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 2000
$timer.add_Tick({
    try {
        $data = Invoke-RestMethod -Uri "$serverUrl/api/v1/telemetry" -TimeoutSec 2
        $temp = [Math]::Round($data.gpu_temp_c)
        $vramUsedGb = [Math]::Round($data.vram_used_mb / 1024, 1)
        $zone = $data.operational_zone

        $activeIcon = "cyan"
        if ($temp -ge 78) {
            $activeIcon = "red"
        } elseif ($temp -ge 72) {
            $activeIcon = "amber"
        } elseif ($data.active_connections -gt 0) {
            $activeIcon = "green"
        }

        $notifyIcon.Icon = $icons[$activeIcon]
        
        $tip = "Souls Engine v7 [$zone]`nGPU: $temp°C | VRAM: ${vramUsedGb}GB / 5.2GB`nConexões Ativas: $($data.active_connections)"
        if ($tip.Length -gt 63) {
            $tip = $tip.Substring(0, 63)
        }
        $notifyIcon.Text = $tip
    } catch {
        $notifyIcon.Icon = $icons["purple"]
        $notifyIcon.Text = "Souls Engine v7 (Offline / Conectando...)"
    }
})

$timer.Start()

# Loop de mensagens da UI
Write-Host "Souls Launcher Companion ativo na bandeja do Windows." -ForegroundColor Green
[System.Windows.Forms.Application]::Run()
