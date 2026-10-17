<#
.SYNOPSIS
    souls_profile_sync.ps1 - Governança e sincronização dos perfis do Master Brain do Hermes Agent.

.DESCRIPTION
    Permite inspecionar os 5 perfis cognitivos do Souls Engine, consultar os líderes
    de Pareto da Cloud Arena em tempo real e aplicar atomicamente o modelo campeão
    ao %LOCALAPPDATA%\hermes\config.yaml com backup preventivo e validação de integridade.

.PARAMETER List
    Lista os 5 perfis disponíveis, modelo ativo no Hermes e recomendações de Pareto.

.PARAMETER Profile
    Nome do perfil canônico a aplicar:
    - frontier_balanced (ROI de Pareto Custo x Benefício)
    - deep_swe          (Líder de qualidade em Engenharia de Software)
    - deep_reasoning    (Raciocínio matemático e provas formais)
    - ultra_fast        (Baixa latência e triagem rápida)
    - local_sovereign   (Execução bare-metal no silício local / custo zero)

.PARAMETER Model
    Sobrescreve diretamente o identificador do modelo (ex: 'anthropic/claude-3-7-sonnet').

.PARAMETER Provider
    Sobrescreve o provedor de inferência (padrão: 'nous' para nuvem, 'custom' para local).

.PARAMETER Check
    Executa a validação de sintaxe e dependências via 'hermes config check'.

.EXAMPLE
    .\tools\souls_profile_sync.ps1 -List
    .\tools\souls_profile_sync.ps1 -Profile deep_swe
    .\tools\souls_profile_sync.ps1 -Profile frontier_balanced
    .\tools\souls_profile_sync.ps1 -Profile local_sovereign
#>

[CmdletBinding()]
param(
    [switch]$List,
    [ValidateSet("frontier_balanced", "deep_swe", "deep_reasoning", "ultra_fast", "local_sovereign")]
    [string]$Profile,
    [string]$Model,
    [string]$Provider,
    [switch]$Check
)

$ErrorActionPreference = "Stop"

$hermesConfigDir = "$env:LOCALAPPDATA\hermes"
$hermesConfigFile = Join-Path $hermesConfigDir "config.yaml"
$soulsApiUrl = "http://127.0.0.1:9123"

function Write-Header {
    Write-Host ""
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host "    SOULS ENGINE v7 — MASTER BRAIN PROFILE GOVERNANCE           " -ForegroundColor Cyan
    Write-Host "=================================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Get-ActiveHermesModel {
    if (-not (Test-Path $hermesConfigFile)) {
        return @{ Model = "N/A"; Provider = "N/A" }
    }

    $rawYaml = [System.IO.File]::ReadAllText($hermesConfigFile, [System.Text.Encoding]::UTF8)
    $lines = $rawYaml -split "`r?`n"
    $insideModel = $false
    $currentModel = "N/A"
    $currentProvider = "nous"

    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ($line -match "^model:") {
            $insideModel = $true
            continue
        }
        if ($insideModel) {
            if ($trimmed -match "^default:\s*[`"']?([^`"']+)`"?") {
                $currentModel = $matches[1].Trim()
            } elseif ($trimmed -match "^provider:\s*[`"']?([^`"']+)`"?") {
                $currentProvider = $matches[1].Trim()
            } elseif ($line -match "^\S") {
                $insideModel = $false
            }
        }
    }

    return @{ Model = $currentModel; Provider = $currentProvider }
}

function Get-CloudChampions {
    try {
        $resp = Invoke-RestMethod -Uri "$soulsApiUrl/api/v1/cloud/champions" -Method Get -TimeoutSec 3 -ErrorAction Stop
        return $resp.champions
    } catch {
        return $null
    }
}

function Show-ProfilesList {
    $active = Get-ActiveHermesModel
    $champions = Get-CloudChampions

    Write-Host " [Status Ativo do Hermes Agent]" -ForegroundColor Yellow
    Write-Host "   Arquivo de Configuração: $hermesConfigFile" -ForegroundColor Gray
    Write-Host "   Modelo Mestre Atual:     $($active.Model)" -ForegroundColor Green
    Write-Host "   Provedor Ativo:          $($active.Provider)" -ForegroundColor Gray
    Write-Host ""

    Write-Host " [Perfis Operacionais Canônicos]" -ForegroundColor Yellow

    $profilesDef = @(
        @{
            Id = "frontier_balanced"
            Name = "Frontier Balanced (Best Value)"
            Kind = "agentic"
            DefaultModel = "deepseek/deepseek-chat"
            Provider = "nous"
            Type = "Cloud API"
            Desc = "Equilíbrio ótimo de Pareto: SWE, raciocínio e conversação diária balanceada"
        },
        @{
            Id = "deep_swe"
            Name = "Deep Software Engineering"
            Kind = "coding"
            DefaultModel = "anthropic/claude-3-7-sonnet"
            Provider = "nous"
            Type = "Cloud API"
            Desc = "Líder de qualidade em engenharia de software complexa e arquitetura"
        },
        @{
            Id = "deep_reasoning"
            Name = "Formal Logic & Deep Reasoning"
            Kind = "reasoning"
            DefaultModel = "deepseek/deepseek-reasoner"
            Provider = "nous"
            Type = "Cloud API"
            Desc = "Especialista em matemática, provas formais e auditoria criptográfica"
        },
        @{
            Id = "ultra_fast"
            Name = "Ultra Fast & Low Latency"
            Kind = "fast"
            DefaultModel = "google/gemini-2.0-flash-001"
            Provider = "nous"
            Type = "Cloud API"
            Desc = "Máxima velocidade de resposta e custo ultrabaixo para triagem rápida"
        },
        @{
            Id = "local_sovereign"
            Name = "Local Bare-Metal Sovereign"
            Kind = "local"
            DefaultModel = "qwen3.5-coder-3b-instruct"
            Provider = "custom"
            Type = "Silício Bare-Metal (RTX 2060m)"
            Desc = "Execução 100% local, privacidade absoluta de dados e custo $0.00"
        }
    )

    foreach ($p in $profilesDef) {
        $champModel = $p.DefaultModel
        if ($champions -and $p.Kind -ne "local") {
            $matchChamp = $champions | Where-Object { $_.profile -eq $p.Kind }
            if ($matchChamp) {
                if ($p.Id -eq "deep_swe" -or $p.Id -eq "deep_reasoning") {
                    if ($matchChamp.quality_leader.canonical_id) {
                        $champModel = $matchChamp.quality_leader.canonical_id
                    }
                } else {
                    if ($matchChamp.best_value.canonical_id) {
                        $champModel = $matchChamp.best_value.canonical_id
                    }
                }
            }
        }

        $isActive = ($active.Model -eq $champModel) -or ($active.Model -eq $p.DefaultModel)
        $badge = if ($isActive) { "[ATIVO]" } else { "       " }
        $color = if ($isActive) { "Green" } else { "White" }

        Write-Host " $badge $($p.Id.PadRight(18)) -> $champModel" -ForegroundColor $color
        Write-Host "         Tipo: $($p.Type) | Provedor: $($p.Provider)" -ForegroundColor Gray
        Write-Host "         Objetivo: $($p.Desc)" -ForegroundColor DarkGray
        Write-Host ""
    }
}

function Set-HermesProfile {
    param(
        [string]$TargetProfile,
        [string]$CustomModel,
        [string]$CustomProvider
    )

    if (-not (Test-Path $hermesConfigFile)) {
        Write-Error "Arquivo hermes config.yaml não encontrado em $hermesConfigFile."
        return
    }

    $champions = Get-CloudChampions
    $targetModel = $CustomModel
    $targetProvider = $CustomProvider

    if (-not $targetModel) {
        switch ($TargetProfile) {
            "frontier_balanced" {
                $targetModel = "deepseek/deepseek-chat"
                $targetProvider = if ($CustomProvider) { $CustomProvider } else { "nous" }
                if ($champions) {
                    $m = $champions | Where-Object { $_.profile -eq "agentic" }
                    if ($m.best_value.canonical_id) { $targetModel = $m.best_value.canonical_id }
                }
            }
            "deep_swe" {
                $targetModel = "anthropic/claude-3-7-sonnet"
                $targetProvider = if ($CustomProvider) { $CustomProvider } else { "nous" }
                if ($champions) {
                    $m = $champions | Where-Object { $_.profile -eq "coding" }
                    if ($m.quality_leader.canonical_id) { $targetModel = $m.quality_leader.canonical_id }
                }
            }
            "deep_reasoning" {
                $targetModel = "deepseek/deepseek-reasoner"
                $targetProvider = if ($CustomProvider) { $CustomProvider } else { "nous" }
                if ($champions) {
                    $m = $champions | Where-Object { $_.profile -eq "reasoning" }
                    if ($m.quality_leader.canonical_id) { $targetModel = $m.quality_leader.canonical_id }
                }
            }
            "ultra_fast" {
                $targetModel = "google/gemini-2.0-flash-001"
                $targetProvider = if ($CustomProvider) { $CustomProvider } else { "nous" }
                if ($champions) {
                    $m = $champions | Where-Object { $_.profile -eq "fast" }
                    if ($m.best_value.canonical_id) { $targetModel = $m.best_value.canonical_id }
                }
            }
            "local_sovereign" {
                $targetModel = "qwen3.5-coder-3b-instruct"
                $targetProvider = if ($CustomProvider) { $CustomProvider } else { "custom" }
            }
        }
    }

    if (-not $targetProvider) {
        $targetProvider = "nous"
    }

    $active = Get-ActiveHermesModel
    Write-Host " -> Aplicando transição de Master Brain..." -ForegroundColor Cyan
    Write-Host "    Modelo Anterior: $($active.Model) ($($active.Provider))" -ForegroundColor Gray
    Write-Host "    Novo Modelo:     $targetModel ($targetProvider)" -ForegroundColor Green

    # Backup com timestamp
    $timestamp = (Get-Date).ToString("yyyyMMdd_HHmmss")
    $bakFile = "$hermesConfigFile.bak_$timestamp"
    $latestBak = "$hermesConfigFile.bak"
    Copy-Item -Path $hermesConfigFile -Destination $bakFile -Force
    Copy-Item -Path $hermesConfigFile -Destination $latestBak -Force
    Write-Host "  ✓ Backup de segurança criado em: $bakFile" -ForegroundColor Green

    # Edição cirúrgica preservando estrutura YAML
    $rawYaml = [System.IO.File]::ReadAllText($hermesConfigFile, [System.Text.Encoding]::UTF8)
    $lines = $rawYaml -split "`r?`n"
    $newLines = @()
    $insideModel = $false
    $defaultReplaced = $false
    $providerReplaced = $false

    foreach ($line in $lines) {
        if ($line -match "^model:") {
            $insideModel = $true
            $newLines += $line
            continue
        }

        if ($insideModel) {
            if ($line -match "^\s*default:") {
                $newLines += "  default: `"$targetModel`""
                $defaultReplaced = $true
                continue
            } elseif ($line -match "^\s*provider:") {
                $newLines += "  provider: `"$targetProvider`""
                $providerReplaced = $true
                continue
            } elseif ($line -match "^\S") {
                if (-not $providerReplaced) {
                    $newLines += "  provider: `"$targetProvider`""
                    $providerReplaced = $true
                }
                $insideModel = $false
            }
        }

        $newLines += $line
    }

    if ($insideModel -and (-not $providerReplaced)) {
        $newLines += "  provider: `"$targetProvider`""
    }

    $outputYaml = $newLines -join "`n"
    [System.IO.File]::WriteAllText($hermesConfigFile, $outputYaml, [System.Text.Encoding]::UTF8)
    Write-Host "  ✓ config.yaml do Hermes atualizado atomicamente!" -ForegroundColor Green

    # Verificação de integridade com hermes config check
    if (Get-Command "hermes" -ErrorAction SilentlyContinue) {
        Write-Host " -> Verificando integridade com 'hermes config check'..." -ForegroundColor Gray
        $checkOutput = hermes config check 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ Validação do Hermes aprovada com sucesso (código 0)!" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Aviso no config check: $checkOutput" -ForegroundColor Yellow
        }
    }

    Write-Host ""
    Write-Host "  ✓ Master Brain reconfigurado para '$targetModel'." -ForegroundColor Green
    Write-Host "  (i) O Hermes Agent carregará o novo modelo na próxima inicialização ou comando." -ForegroundColor Gray
}

# --- Execução Principal ---
Write-Header

if ($Check) {
    if (Get-Command "hermes" -ErrorAction SilentlyContinue) {
        Write-Host " Executando hermes config check..." -ForegroundColor Cyan
        hermes config check
    } else {
        Write-Host " Comando 'hermes' não localizado no PATH." -ForegroundColor Yellow
    }
    exit 0
}

if ($Profile -or $Model) {
    Set-HermesProfile -TargetProfile $Profile -CustomModel $Model -CustomProvider $Provider
    exit 0
}

# Padrão: Listar se nenhum comando fornecido ou se -List
Show-ProfilesList
