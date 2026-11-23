# souls_launcher.ps1 — Windows Systray Companion Desacoplado do Souls Engine v7
# Governança: Linha Vermelha 1 do AGENTS.md (souls_server.exe permanece 100% headless)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Net.Http

$ErrorActionPreference = "SilentlyContinue"

$serverUrl = "http://127.0.0.1:9123"
$vhdxPath = "C:\dev_projects\Souls_FORGE.vhdx"
$script:exiting = $false

$brandingDir = Join-Path $PSScriptRoot "assets\branding"
if (-not (Test-Path $brandingDir)) {
    $brandingDir = "Z:\souls_engine\assets\branding"
}

function Ensure-ZDriveMounted {
    if (-not (Test-Path "Z:\") -and (Test-Path $vhdxPath)) {
        Write-Host "Dev Drive ReFS (Z:) não encontrado. Montando VHDX: $vhdxPath..." -ForegroundColor Yellow
        Mount-DiskImage -ImagePath $vhdxPath -NoDriveLetter:$false -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 1200
    }
}

function Test-SoulsDaemonRunning {
    return [bool](Get-Process -Name "souls_engine", "souls_server" -ErrorAction SilentlyContinue)
}

function Resolve-SoulsDaemonExe {
    $candidates = @(
        "$env:LOCALAPPDATA\SoulsEngine\souls_engine.exe",
        "$env:LOCALAPPDATA\SoulsEngine\souls_server.exe",
        "Z:\souls_engine\target\release\souls_engine.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_engine.exe",
        "Z:\souls_engine\target\release\souls_server.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\release\souls_server.exe",
        "Z:\souls_engine\target\debug\souls_engine.exe",
        "Z:\souls_engine\target\x86_64-pc-windows-msvc\debug\souls_server.exe",
        "Z:\souls_engine\target\debug\souls_server.exe"
    )
    return ($candidates | Where-Object { Test-Path $_ } | Select-Object -First 1)
}

function Start-SoulsDaemon {
    Ensure-ZDriveMounted
    if (Test-SoulsDaemonRunning) {
        return $true
    }
    $serverExe = Resolve-SoulsDaemonExe
    if (-not $serverExe) {
        return $false
    }
    $exeName = [System.IO.Path]::GetFileName($serverExe)
    Write-Host "Iniciando daemon headless $exeName..." -ForegroundColor Cyan
    $workDir = if (Test-Path "Z:\souls_engine") { "Z:\souls_engine" } else { [System.IO.Path]::GetDirectoryName($serverExe) }
    Start-Process -FilePath $serverExe -WorkingDirectory $workDir -WindowStyle Hidden
    return $true
}

# 1. Garante instância única substituindo qualquer companion anterior (evita ícones duplicados ou bloqueio por processo em segundo plano)
try {
    Get-CimInstance Win32_Process -Filter "Name = 'pwsh.exe' OR Name = 'powershell.exe'" -ErrorAction SilentlyContinue |
        Where-Object {
            $_.ProcessId -ne $PID -and
            $_.CommandLine -like "*souls_launcher.ps1*" -and
            $_.CommandLine -notlike "*-Command*"
        } |
        ForEach-Object {
            Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue
        }
} catch {}

# 2. Inicia daemon se ainda não estiver ativo
Start-SoulsDaemon | Out-Null

# 3. HttpClient dedicado com timeout curto (nunca bloqueia loop WinForms quando offline)
$script:httpClient = [System.Net.Http.HttpClient]::new()
$script:httpClient.Timeout = [TimeSpan]::FromMilliseconds(450)

function Invoke-FastHttpGetJson {
    param([string]$Uri)
    try {
        $task = $script:httpClient.GetAsync($Uri)
        while (-not $task.IsCompleted) {
            if ($script:exiting) { return $null }
            [System.Windows.Forms.Application]::DoEvents()
            Start-Sleep -Milliseconds 15
        }
        $resp = $task.GetAwaiter().GetResult()
        try {
            if (-not $resp.IsSuccessStatusCode) { return $null }
            $readTask = $resp.Content.ReadAsStringAsync()
            while (-not $readTask.IsCompleted) {
                if ($script:exiting) { return $null }
                [System.Windows.Forms.Application]::DoEvents()
                Start-Sleep -Milliseconds 10
            }
            $raw = $readTask.GetAwaiter().GetResult()
            if ([string]::IsNullOrWhiteSpace($raw)) { return $null }
            return ($raw | ConvertFrom-Json)
        } finally {
            $resp.Dispose()
        }
    } catch {
        return $null
    }
}

# 4. Carrega ícones multi-resolução (incluindo cinza para estado parado)
function Load-TrayIcon {
    param([string]$Name, $Fallback)
    $path = Join-Path $brandingDir "souls_icon_${Name}.ico"
    if (Test-Path $path) {
        return [System.Drawing.Icon]::new($path)
    }
    return $Fallback
}

$cyanIcon = Load-TrayIcon -Name "cyan" -Fallback $null
if (-not $cyanIcon) {
    $cyanIcon = [System.Drawing.SystemIcons]::Application
}
$icons = @{
    "cyan"   = $cyanIcon
    "green"  = Load-TrayIcon -Name "green"  -Fallback $cyanIcon
    "amber"  = Load-TrayIcon -Name "amber"  -Fallback $cyanIcon
    "red"    = Load-TrayIcon -Name "red"    -Fallback $cyanIcon
    "purple" = Load-TrayIcon -Name "purple" -Fallback $cyanIcon
    "gray"   = Load-TrayIcon -Name "gray"   -Fallback $cyanIcon
}

$notifyIcon = New-Object System.Windows.Forms.NotifyIcon
$notifyIcon.Icon = if (Test-SoulsDaemonRunning) { $icons["cyan"] } else { $icons["gray"] }
$notifyIcon.Text = "Souls Engine v7 (Iniciando...)"
$notifyIcon.Visible = $true

# Menu de Contexto (sempre responsivo, online ou offline)
$contextMenu = New-Object System.Windows.Forms.ContextMenuStrip

$itemOpenDash = $contextMenu.Items.Add("🌐 Abrir Web Dashboard")
$contextMenu.Items.Add("-") | Out-Null
$itemStartServer = $contextMenu.Items.Add("▶️ Iniciar Servidor Daemon")
$itemRestartServer = $contextMenu.Items.Add("🔁 Reiniciar Servidor Daemon")
$itemStopServer = $contextMenu.Items.Add("🛑 Parar Servidor Daemon")
$contextMenu.Items.Add("-") | Out-Null
$itemSync = $contextMenu.Items.Add("🔄 Sincronizar Cloud Arena")
$itemCheckpoint = $contextMenu.Items.Add("🧹 Compactar WAL (Checkpoint)")
$contextMenu.Items.Add("-") | Out-Null
$itemExit = $contextMenu.Items.Add("❌ Sair do Companion")

function Update-MenuControlsState {
    param([bool]$Running)
    $itemStartServer.Enabled = (-not $Running)
    $itemRestartServer.Enabled = $Running
    $itemStopServer.Enabled = $Running
}

function Set-StoppedTrayVisual {
    $notifyIcon.Icon = $icons["gray"]
    $notifyIcon.Text = "Souls Engine v7 (Servidor Parado — Botão direito p/ Iniciar)"
    Update-MenuControlsState -Running $false
}

function Stop-SoulsDaemon {
    Stop-Process -Name "souls_server", "souls_engine" -Force -ErrorAction SilentlyContinue
    Set-StoppedTrayVisual
}

function Open-DashboardSmart {
    if (-not (Test-SoulsDaemonRunning)) {
        if (-not (Start-SoulsDaemon)) {
            $notifyIcon.ShowBalloonTip(2500, "Souls Engine", "Executável souls_engine.exe não encontrado.", [System.Windows.Forms.ToolTipIcon]::Error)
            return
        }
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Iniciando servidor Souls Engine...", [System.Windows.Forms.ToolTipIcon]::Info)
    }

    $deadline = [DateTime]::UtcNow.AddMilliseconds(2500)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($script:exiting) { return }
        $health = Invoke-FastHttpGetJson -Uri "$serverUrl/health"
        if ($health) { break }
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 150
    }
    Start-Process "$serverUrl/dashboard"
}

$itemOpenDash.add_Click({
    Open-DashboardSmart
})

$notifyIcon.add_DoubleClick({
    Open-DashboardSmart
})

$itemStartServer.add_Click({
    if (Start-SoulsDaemon) {
        Update-MenuControlsState -Running $true
        $notifyIcon.Text = "Souls Engine v7 (Iniciando servidor...)"
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Daemon Souls Engine iniciado.", [System.Windows.Forms.ToolTipIcon]::Info)
    } else {
        $notifyIcon.ShowBalloonTip(2500, "Souls Engine", "Executável souls_engine.exe não encontrado.", [System.Windows.Forms.ToolTipIcon]::Error)
    }
})

$itemRestartServer.add_Click({
    Stop-SoulsDaemon
    $waitUntil = [DateTime]::UtcNow.AddMilliseconds(350)
    while ([DateTime]::UtcNow -lt $waitUntil) {
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 50
    }
    if (Start-SoulsDaemon) {
        Update-MenuControlsState -Running $true
        $notifyIcon.Text = "Souls Engine v7 (Reiniciando servidor...)"
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Daemon Souls Engine reiniciado.", [System.Windows.Forms.ToolTipIcon]::Info)
    } else {
        $notifyIcon.ShowBalloonTip(2500, "Souls Engine", "Falha ao localizar souls_engine.exe para reiniciar.", [System.Windows.Forms.ToolTipIcon]::Error)
    }
})

$itemStopServer.add_Click({
    Stop-SoulsDaemon
    $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Daemon souls_server/souls_engine encerrado.", [System.Windows.Forms.ToolTipIcon]::Info)
})

$itemSync.add_Click({
    if (-not (Test-SoulsDaemonRunning)) {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Servidor parado. Inicie o daemon antes de sincronizar.", [System.Windows.Forms.ToolTipIcon]::Warning)
        return
    }
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/cloud/sync" -Method Post -TimeoutSec 5 | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Cloud Arena sincronizada com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Falha ao sincronizar Cloud Arena.", [System.Windows.Forms.ToolTipIcon]::Warning)
    }
})

$itemCheckpoint.add_Click({
    if (-not (Test-SoulsDaemonRunning)) {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Servidor parado. Inicie o daemon antes de compactar o WAL.", [System.Windows.Forms.ToolTipIcon]::Warning)
        return
    }
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/system/checkpoint" -Method Post -TimeoutSec 5 | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "WAL SQLite compactado com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Falha ao compactar WAL SQLite.", [System.Windows.Forms.ToolTipIcon]::Warning)
    }
})

$itemExit.add_Click({
    $script:exiting = $true
    $timer.Stop()
    $notifyIcon.Visible = $false
    [System.Windows.Forms.Application]::Exit()
})

$notifyIcon.ContextMenuStrip = $contextMenu
Update-MenuControlsState -Running (Test-SoulsDaemonRunning)

# 5. Timer de telemetria (sem bloqueio quando servidor parado)
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 2000
$timer.add_Tick({
    $timer.Stop()
    try {
        if ($script:exiting) { return }

        if (-not (Test-SoulsDaemonRunning)) {
            Set-StoppedTrayVisual
            return
        }

        Update-MenuControlsState -Running $true
        $data = Invoke-FastHttpGetJson -Uri "$serverUrl/api/v1/telemetry"
        if ($data) {
            $temp = [Math]::Round([double]$data.gpu_temp_c)
            $vramUsedGb = [Math]::Round(([double]$data.vram_used_mb) / 1024, 1)
            $zone = $data.operational_zone

            $activeIcon = "cyan"
            if ($temp -ge 78) {
                $activeIcon = "red"
            } elseif ($temp -ge 72) {
                $activeIcon = "amber"
            } elseif ([int]$data.active_connections -gt 0) {
                $activeIcon = "green"
            }

            $notifyIcon.Icon = $icons[$activeIcon]
            $tip = "Souls Engine v7 [$zone]`nGPU: $temp°C | VRAM: ${vramUsedGb}GB / 5.2GB`nConexões Ativas: $($data.active_connections)"
            if ($tip.Length -gt 63) {
                $tip = $tip.Substring(0, 63)
            }
            $notifyIcon.Text = $tip
        } else {
            $notifyIcon.Icon = $icons["gray"]
            $notifyIcon.Text = "Souls Engine v7 (Iniciando servidor...)"
        }
    } finally {
        if (-not $script:exiting) {
            $timer.Start()
        }
    }
})

$timer.Start()

Write-Host "Souls Launcher Companion ativo na bandeja do Windows." -ForegroundColor Green
try {
    [System.Windows.Forms.Application]::Run()
} finally {
    $script:exiting = $true
    if ($timer) {
        $timer.Stop()
        $timer.Dispose()
    }
    if ($notifyIcon) {
        $notifyIcon.Visible = $false
        $notifyIcon.Dispose()
    }
    if ($contextMenu) {
        $contextMenu.Dispose()
    }
    if ($icons) {
        foreach ($ic in ($icons.Values | Select-Object -Unique)) {
            if ($ic) { $ic.Dispose() }
        }
    }
    if ($script:httpClient) {
        $script:httpClient.Dispose()
    }
}
