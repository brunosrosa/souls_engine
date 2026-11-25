# souls_launcher.ps1 — Windows Systray Companion Desacoplado do Souls Engine
# Governança: Linha Vermelha 1 do AGENTS.md (souls_server.exe permanece 100% headless)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Net.Http

$ErrorActionPreference = "Continue"

$serverUrl = "http://127.0.0.1:9123"
$vhdxPath = "C:\dev_projects\Souls_FORGE.vhdx"
$script:exiting = $false
$script:launcherState = "stopped"
$script:startingSince = $null
$script:lastLogSignature = ""

$trayLogDir = "Z:\souls_engine\.souls_data\logs"
$trayLogPath = Join-Path $trayLogDir "souls_launcher.log"

function Write-TrayLog {
    param(
        [string]$Level,
        [string]$Message,
        [switch]$Force
    )
    try {
        if (-not $Force) {
            $sig = "${Level}:${Message}"
            if ($sig -eq $script:lastLogSignature) { return }
            $script:lastLogSignature = $sig
        }
        if (-not (Test-Path $trayLogDir)) {
            New-Item -ItemType Directory -Path $trayLogDir -Force | Out-Null
        }
        if ((Test-Path $trayLogPath) -and ((Get-Item $trayLogPath).Length -gt 262144)) {
            $bak = "$trayLogPath.1"
            if (Test-Path $bak) { Remove-Item $bak -Force }
            Move-Item $trayLogPath $bak -Force
        }
        $ts = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ss.fffK")
        Add-Content -Path $trayLogPath -Value "[$ts] [$Level] $Message" -Encoding UTF8
    } catch {}
}

$brandingDir = Join-Path $PSScriptRoot "assets\branding"
if (-not (Test-Path $brandingDir)) {
    $brandingDir = "Z:\souls_engine\assets\branding"
}

function Ensure-ZDriveMounted {
    if (-not (Test-Path "Z:\") -and (Test-Path $vhdxPath)) {
        Write-TrayLog -Level "INFO" -Message "Dev Drive ReFS (Z:) nao encontrado. Montando VHDX: $vhdxPath" -Force
        try {
            Mount-DiskImage -ImagePath $vhdxPath -NoDriveLetter:$false -ErrorAction Stop | Out-Null
            Start-Sleep -Milliseconds 1200
        } catch {
            Write-TrayLog -Level "WARN" -Message "Falha ao montar VHDX ($vhdxPath): $($_.Exception.Message)" -Force
        }
    }
}

function Test-SoulsDaemonProcessRunning {
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

function Format-TrayTooltip {
    param([string]$Text)
    if ([string]::IsNullOrEmpty($Text)) { return "Souls Engine" }
    if ($Text.Length -le 63) { return $Text }
    return $Text.Substring(0, 60) + "..."
}

# 1. Garante instancia unica substituindo qualquer companion anterior
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
} catch {
    Write-TrayLog -Level "WARN" -Message "Falha ao checar instancia previa do launcher: $($_.Exception.Message)" -Force
}

Write-TrayLog -Level "INFO" -Message "Souls Launcher iniciando (PID=$PID, brandingDir=$brandingDir)" -Force

# 2. HttpClient dedicado com timeout realista (1500ms) consultando /health leve
$script:httpClient = [System.Net.Http.HttpClient]::new()
$script:httpClient.Timeout = [TimeSpan]::FromMilliseconds(1500)

function Get-SoulsHealth {
    try {
        $task = $script:httpClient.GetAsync("$serverUrl/health")
        while (-not $task.IsCompleted) {
            if ($script:exiting) { return $null }
            [System.Windows.Forms.Application]::DoEvents()
            Start-Sleep -Milliseconds 15
        }
        $resp = $task.GetAwaiter().GetResult()
        try {
            if (-not $resp.IsSuccessStatusCode) {
                return [pscustomobject]@{
                    _http_error = "HTTP $([int]$resp.StatusCode)"
                }
            }
            $readTask = $resp.Content.ReadAsStringAsync()
            while (-not $readTask.IsCompleted) {
                if ($script:exiting) { return $null }
                [System.Windows.Forms.Application]::DoEvents()
                Start-Sleep -Milliseconds 10
            }
            $raw = $readTask.GetAwaiter().GetResult()
            if ([string]::IsNullOrWhiteSpace($raw)) {
                return [pscustomobject]@{ _http_error = "EmptyBody" }
            }
            return ($raw | ConvertFrom-Json)
        } finally {
            $resp.Dispose()
        }
    } catch {
        return $null
    }
}

# 3. Carrega icones multi-resolucao por estado
function Load-TrayIcon {
    param([string]$Name, $Fallback)
    $path = Join-Path $brandingDir "souls_icon_${Name}.ico"
    if (Test-Path $path) {
        try {
            return [System.Drawing.Icon]::new($path)
        } catch {
            Write-TrayLog -Level "WARN" -Message "Falha ao carregar icone ${path}: $($_.Exception.Message)" -Force
            return $Fallback
        }
    }
    Write-TrayLog -Level "WARN" -Message "Icone ausente: $path" -Force
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
$notifyIcon.Icon = $icons["gray"]
$notifyIcon.Text = Format-TrayTooltip "Souls Engine (Verificando estado...)"
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

function Update-TrayState {
    $health = Get-SoulsHealth
    $procRunning = Test-SoulsDaemonProcessRunning

    if ($null -ne $health -and -not ($health.PSObject.Properties.Name -contains "_http_error")) {
        $script:launcherState = "running"
        $script:startingSince = $null
        Update-MenuControlsState -Running $true

        $hw = $health.hardware
        $zone = if ($hw -and $hw.operational_zone) { [string]$hw.operational_zone } else { "Nominal" }
        $temp = if ($hw -and $null -ne $hw.gpu_temp_c) { [Math]::Round([double]$hw.gpu_temp_c, 0) } else { 0 }
        $vramUsedGb = if ($hw -and $null -ne $hw.vram_used_mb) {
            [Math]::Round(([double]$hw.vram_used_mb) / 1024, 1)
        } else { 0.0 }
        $vramUsableGb = if ($hw -and $null -ne $hw.vram_usable_mb) {
            [Math]::Round(([double]$hw.vram_usable_mb) / 1024, 1)
        } else { 0.0 }
        $conns = if ($null -ne $health.active_connections) { [int]$health.active_connections } else { 0 }
        $catnapActive = ($health.catnap_active -eq $true)
        $ver = if ($health.version) { [string]$health.version } else { "?" }

        $iconKey = "cyan"
        if ($zone -eq "Critical" -or $temp -ge 78) {
            $iconKey = "red"
        } elseif ($zone -in @("Preventive", "Degraded", "Pressure") -or $temp -ge 72) {
            $iconKey = "amber"
        } elseif ($catnapActive) {
            $iconKey = "purple"
        } elseif ($conns -gt 0) {
            $iconKey = "green"
        }

        $notifyIcon.Icon = $icons[$iconKey]
        $tipRaw = "Souls Engine $ver [$zone]`nGPU:${temp}C ${vramUsedGb}/${vramUsableGb}GB C:$conns"
        $tipFinal = Format-TrayTooltip $tipRaw
        $notifyIcon.Text = $tipFinal
        Write-TrayLog -Level "INFO" -Message "state=running icon=$iconKey ver=$ver zone=$zone temp=${temp}C vram=${vramUsedGb}/${vramUsableGb}GB conns=$conns catnap=$catnapActive"
        return
    }

    if ($null -ne $health -and ($health.PSObject.Properties.Name -contains "_http_error")) {
        $script:launcherState = "error"
        $notifyIcon.Icon = $icons["red"]
        $notifyIcon.Text = Format-TrayTooltip "Souls Engine (Erro $($health._http_error))"
        Update-MenuControlsState -Running $procRunning
        Write-TrayLog -Level "ERROR" -Message "state=error icon=red detail=$($health._http_error)"
        return
    }

    if ($procRunning -or $script:launcherState -eq "starting") {
        if (-not $script:startingSince) {
            $script:startingSince = [DateTime]::UtcNow
        }
        $elapsed = ([DateTime]::UtcNow - $script:startingSince).TotalSeconds
        if ($elapsed -lt 15) {
            $script:launcherState = "starting"
            $notifyIcon.Icon = $icons["purple"]
            $notifyIcon.Text = Format-TrayTooltip "Souls Engine (Iniciando servidor... $([int]$elapsed)s)"
            Update-MenuControlsState -Running $procRunning
            Write-TrayLog -Level "INFO" -Message "state=starting icon=purple elapsed=$([int]$elapsed)s procRunning=$procRunning"
            return
        }
        $script:launcherState = "error"
        $notifyIcon.Icon = $icons["red"]
        $notifyIcon.Text = Format-TrayTooltip "Souls Engine (Timeout ao responder /health)"
        Update-MenuControlsState -Running $procRunning
        Write-TrayLog -Level "ERROR" -Message "state=error icon=red reason=startup_health_timeout_15s procRunning=$procRunning"
        return
    }

    $script:launcherState = "stopped"
    $script:startingSince = $null
    $notifyIcon.Icon = $icons["gray"]
    $notifyIcon.Text = Format-TrayTooltip "Souls Engine (Parado — Botao dir. p/ Iniciar)"
    Update-MenuControlsState -Running $false
    Write-TrayLog -Level "INFO" -Message "state=stopped icon=gray"
}

function Start-SoulsDaemon {
    Ensure-ZDriveMounted
    if (Test-SoulsDaemonProcessRunning) {
        $health = Get-SoulsHealth
        if ($null -ne $health -and -not ($health.PSObject.Properties.Name -contains "_http_error")) {
            return $true
        }
    }
    $serverExe = Resolve-SoulsDaemonExe
    if (-not $serverExe) {
        Write-TrayLog -Level "ERROR" -Message "Executavel souls_engine.exe / souls_server.exe nao encontrado." -Force
        return $false
    }
    $exeName = [System.IO.Path]::GetFileName($serverExe)
    $workDir = if (Test-Path "Z:\souls_engine") { "Z:\souls_engine" } else { [System.IO.Path]::GetDirectoryName($serverExe) }
    $script:launcherState = "starting"
    $script:startingSince = [DateTime]::UtcNow
    Update-TrayState
    Write-TrayLog -Level "INFO" -Message "Iniciando daemon headless $exeName em $workDir" -Force
    Start-Process -FilePath $serverExe -WorkingDirectory $workDir -WindowStyle Hidden

    $deadline = [DateTime]::UtcNow.AddMilliseconds(8000)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($script:exiting) { break }
        $h = Get-SoulsHealth
        if ($null -ne $h -and -not ($h.PSObject.Properties.Name -contains "_http_error")) {
            break
        }
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 150
    }
    Update-TrayState
    return $true
}

function Stop-SoulsDaemon {
    Write-TrayLog -Level "INFO" -Message "Encerrando daemon souls_server / souls_engine..." -Force
    Stop-Process -Name "souls_server", "souls_engine" -Force -ErrorAction SilentlyContinue
    $script:launcherState = "stopped"
    $script:startingSince = $null
    Update-TrayState
}

function Open-DashboardSmart {
    $h = Get-SoulsHealth
    if ($null -eq $h -or ($h.PSObject.Properties.Name -contains "_http_error")) {
        if (-not (Start-SoulsDaemon)) {
            $notifyIcon.ShowBalloonTip(2500, "Souls Engine", "Executável souls_engine.exe não encontrado.", [System.Windows.Forms.ToolTipIcon]::Error)
            return
        }
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Iniciando servidor Souls Engine...", [System.Windows.Forms.ToolTipIcon]::Info)
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
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Daemon Souls Engine iniciado.", [System.Windows.Forms.ToolTipIcon]::Info)
    } else {
        $notifyIcon.ShowBalloonTip(2500, "Souls Engine", "Executável souls_engine.exe não encontrado.", [System.Windows.Forms.ToolTipIcon]::Error)
    }
})

$itemRestartServer.add_Click({
    Stop-SoulsDaemon
    $waitUntil = [DateTime]::UtcNow.AddMilliseconds(400)
    while ([DateTime]::UtcNow -lt $waitUntil) {
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 50
    }
    if (Start-SoulsDaemon) {
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
    $h = Get-SoulsHealth
    if (-not $h -or ($h.PSObject.Properties.Name -contains "_http_error")) {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Servidor parado. Inicie o daemon antes de sincronizar.", [System.Windows.Forms.ToolTipIcon]::Warning)
        return
    }
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/cloud/sync" -Method Post -TimeoutSec 5 | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Cloud Arena sincronizada com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {
        Write-TrayLog -Level "WARN" -Message "Falha ao sincronizar Cloud Arena: $($_.Exception.Message)" -Force
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Falha ao sincronizar Cloud Arena.", [System.Windows.Forms.ToolTipIcon]::Warning)
    }
})

$itemCheckpoint.add_Click({
    $h = Get-SoulsHealth
    if (-not $h -or ($h.PSObject.Properties.Name -contains "_http_error")) {
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "Servidor parado. Inicie o daemon antes de compactar o WAL.", [System.Windows.Forms.ToolTipIcon]::Warning)
        return
    }
    try {
        Invoke-RestMethod -Uri "$serverUrl/api/v1/system/checkpoint" -Method Post -TimeoutSec 5 | Out-Null
        $notifyIcon.ShowBalloonTip(2000, "Souls Engine", "WAL SQLite compactado com sucesso!", [System.Windows.Forms.ToolTipIcon]::Info)
    } catch {
        Write-TrayLog -Level "WARN" -Message "Falha ao compactar WAL SQLite: $($_.Exception.Message)" -Force
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

# 4. Boot inicial + refresh imediato de estado
Start-SoulsDaemon | Out-Null
Update-TrayState

# 5. Timer periodico (sem bloqueio quando servidor parado)
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 3000
$timer.add_Tick({
    $timer.Stop()
    try {
        if ($script:exiting) { return }
        Update-TrayState
    } finally {
        if (-not $script:exiting) {
            $timer.Start()
        }
    }
})

$timer.Start()

Write-TrayLog -Level "INFO" -Message "Souls Launcher Companion ativo na bandeja do Windows." -Force
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
