# SPDX-FileCopyrightText: 2026 Lucas Greuloch
# SPDX-License-Identifier: GPL-3.0-or-later

<#
.SYNOPSIS
Samples the GPU engine load and GPU memory of one process once per second (Windows only).

.DESCRIPTION
Reads the non-localised WMI GPU performance classes for every process with the given name, for
example the 3D application next to which the spike is measured, and prints average and maximum.

.PARAMETER ProcessName
The process name without `.exe`.

.PARAMETER Seconds
How long to sample.

.PARAMETER Label
A free-text note stored with the result.

.PARAMETER OutDir
Where the report goes.
#>
param(
    [Parameter(Mandatory)] [string] $ProcessName,
    [int] $Seconds = 30,
    [string] $Label = '',
    [string] $OutDir = (Join-Path $PSScriptRoot '..\target\measurements')
)

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'the GPU performance classes exist on Windows only' }
New-Item -ItemType Directory -Force $OutDir | Out-Null
$ids = (Get-Process -Name $ProcessName).Id
$prefixes = $ids | ForEach-Object { "pid_$($_)_*" }

function Test-Owned($Name) { foreach ($p in $prefixes) { if ($Name -like $p) { return $true } }; $false }

$samples = for ($i = 0; $i -lt $Seconds; $i++) {
    Start-Sleep -Seconds 1
    $memory = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory |
        Where-Object { Test-Owned $_.Name }
    $engines = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine |
        Where-Object { Test-Owned $_.Name }
    $by_type = $engines | Group-Object { ($_.Name -split 'engtype_')[-1] } |
        ForEach-Object { ($_.Group | Measure-Object UtilizationPercentage -Sum).Sum }
    [pscustomobject]@{
        dedicated_mb      = (($memory | Measure-Object DedicatedUsage -Sum).Sum) / 1MB
        shared_mb         = (($memory | Measure-Object SharedUsage -Sum).Sum) / 1MB
        busiest_engine_percent = ($by_type | Measure-Object -Maximum).Maximum
    }
}

function Stat($Name) {
    $m = $samples | Measure-Object $Name -Average -Maximum
    [math]::Round($m.Average, 1), [math]::Round($m.Maximum, 1) -join ' / '
}

$result = [pscustomobject]@{
    label                          = $Label
    process                        = $ProcessName
    seconds                        = $Seconds
    gpu_dedicated_mb_avg_max       = Stat 'dedicated_mb'
    gpu_shared_mb_avg_max          = Stat 'shared_mb'
    busiest_engine_percent_avg_max = Stat 'busiest_engine_percent'
}
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$result | ConvertTo-Json | Set-Content (Join-Path $OutDir "gpu-$ProcessName-$stamp.json")
$result | Format-List
