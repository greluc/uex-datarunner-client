# SPDX-FileCopyrightText: 2026 Lucas Greuloch
# SPDX-License-Identifier: GPL-3.0-or-later

<#
.SYNOPSIS
Measures the wgpu footprint of the review-table spike: binary size, start time, memory, VRAM and GPU load while idle.

.DESCRIPTION
Start the 3D application to measure against first (O-113), then run this script.
It starts the spike, times the first frame, lets it settle, and samples once per second:
process CPU time, working set, private memory and, on Windows, the GPU process memory and GPU
engine utilisation counters through the non-localised WMI performance classes. On Linux the GPU
columns stay empty. The window is closed at the end.

.PARAMETER Exe
The spike binary; defaults to the release build of the crate.

.PARAMETER Scale
The display scale in percent.

.PARAMETER SettleSeconds
How long to wait after the first frame before sampling.

.PARAMETER SampleSeconds
How long to sample.

.PARAMETER Label
A free-text note stored with the result, for example the name of the 3D application running.

.PARAMETER OutDir
Where the report goes.
#>
param(
    [string] $Exe = (Join-Path $PSScriptRoot ('..\target\release\review-table-spike' + $(if ($IsWindows) { '.exe' } else { '' }))),
    [int] $Scale = 100,
    [int] $SettleSeconds = 10,
    [int] $SampleSeconds = 30,
    [string] $Label = 'no 3D application running',
    [string] $OutDir = (Join-Path $PSScriptRoot '..\target\measurements')
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Exe)) { throw "binary not found: $Exe (cargo build --release --locked first)" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Get-GpuSample([int] $ProcessId) {
    if (-not $IsWindows) { return $null }
    $prefix = "pid_$($ProcessId)_*"
    $memory = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory |
        Where-Object Name -like $prefix
    $engines = Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine |
        Where-Object Name -like $prefix
    [pscustomobject]@{
        dedicated_mb = (($memory | Measure-Object DedicatedUsage -Sum).Sum) / 1MB
        shared_mb    = (($memory | Measure-Object SharedUsage -Sum).Sum) / 1MB
        engine_percent = ($engines | Measure-Object UtilizationPercentage -Sum).Sum
    }
}

$marker = Join-Path $OutDir 'first-frame.marker'
Remove-Item $marker -ErrorAction SilentlyContinue
$clock = [System.Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath $Exe -PassThru -ArgumentList @(
    '--scale', $Scale, '--first-frame-marker', $marker)
while (-not (Test-Path $marker)) {
    if ($process.HasExited) { throw 'the app exited before its first frame' }
    if ($clock.Elapsed.TotalSeconds -gt 60) { $process.Kill(); throw 'no first frame within 60 s' }
    Start-Sleep -Milliseconds 20
}
$startMs = $clock.Elapsed.TotalMilliseconds
Start-Sleep -Seconds $SettleSeconds

$samples = @()
$process.Refresh()
$cpuStart = $process.TotalProcessorTime
$wall = [System.Diagnostics.Stopwatch]::StartNew()
for ($i = 0; $i -lt $SampleSeconds; $i++) {
    Start-Sleep -Seconds 1
    $process.Refresh()
    $gpu = Get-GpuSample $process.Id
    $samples += [pscustomobject]@{
        working_set_mb = $process.WorkingSet64 / 1MB
        private_mb     = $process.PrivateMemorySize64 / 1MB
        dedicated_mb   = $gpu.dedicated_mb
        shared_mb      = $gpu.shared_mb
        engine_percent = $gpu.engine_percent
    }
}
$process.Refresh()
$cpuSeconds = ($process.TotalProcessorTime - $cpuStart).TotalSeconds
$wallSeconds = $wall.Elapsed.TotalSeconds
$process.CloseMainWindow() | Out-Null
if (-not $process.WaitForExit(10000)) { $process.Kill() }

function Stat($Name) {
    $values = $samples | Where-Object { $null -ne $_.$Name } | ForEach-Object { [double] $_.$Name }
    if (-not $values) { return $null }
    $m = $values | Measure-Object -Average -Maximum
    [math]::Round($m.Average, 2), [math]::Round($m.Maximum, 2) -join ' / '
}

$result = [pscustomobject]@{
    label                    = $Label
    os                       = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
    scale_percent            = $Scale
    binary_mb                = [math]::Round((Get-Item $Exe).Length / 1MB, 2)
    first_frame_ms           = [math]::Round($startMs, 0)
    idle_cpu_percent_of_one_core = [math]::Round(100 * $cpuSeconds / $wallSeconds, 3)
    working_set_mb_avg_max   = Stat 'working_set_mb'
    private_mb_avg_max       = Stat 'private_mb'
    gpu_dedicated_mb_avg_max = Stat 'dedicated_mb'
    gpu_shared_mb_avg_max    = Stat 'shared_mb'
    gpu_engine_percent_avg_max = Stat 'engine_percent'
}
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$result | ConvertTo-Json | Set-Content (Join-Path $OutDir "footprint-$stamp.json")
$result | Format-List
