# SPDX-FileCopyrightText: 2026 Lucas Greuloch
# SPDX-License-Identifier: GPL-3.0-or-later

<#
.SYNOPSIS
Records frame time and idle CPU of the review-table spike at each display scale.

.DESCRIPTION
Starts the release build once per scale in its measurement mode. The app runs a scripted
scroll, F8 and combo-box sequence, writes its per-phase frame-time report, then stays idle;
this script samples the process CPU time during that idle window and collects all results.

.PARAMETER Exe
The spike binary; defaults to the release build of the crate.

.PARAMETER Scales
The display scales in percent.

.PARAMETER IdleSeconds
How long the app stays idle after the scripted run.

.PARAMETER OutDir
Where the reports go.
#>
param(
    [string] $Exe = (Join-Path $PSScriptRoot ('..\target\release\review-table-spike' + $(if ($IsWindows) { '.exe' } else { '' }))),
    [int[]] $Scales = @(100, 125, 150, 200, 250),
    [int] $IdleSeconds = 30,
    [string] $OutDir = (Join-Path $PSScriptRoot '..\target\measurements')
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path $Exe)) { throw "binary not found: $Exe (cargo build --release --locked first)" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

$frames = @()
$idle = @()
foreach ($scale in $Scales) {
    $report = Join-Path $OutDir "frames-$scale.csv"
    Remove-Item $report -ErrorAction SilentlyContinue
    $process = Start-Process -FilePath $Exe -PassThru -ArgumentList @(
        '--scale', $scale, '--measure', $report, '--idle-seconds', $IdleSeconds)
    $deadline = (Get-Date).AddMinutes(3)
    while (-not (Test-Path $report)) {
        if ($process.HasExited) { throw "the app exited before its report at $scale %" }
        if ((Get-Date) -gt $deadline) { $process.Kill(); throw "no report at $scale %" }
        Start-Sleep -Milliseconds 200
    }
    Start-Sleep -Seconds 3
    $process.Refresh()
    $cpuStart = $process.TotalProcessorTime
    $wallStart = Get-Date
    Start-Sleep -Seconds ($IdleSeconds - 6)
    $process.Refresh()
    $cpuSeconds = ($process.TotalProcessorTime - $cpuStart).TotalSeconds
    $wallSeconds = ((Get-Date) - $wallStart).TotalSeconds
    $idle += [pscustomobject]@{
        scale_percent       = $scale
        idle_window_s       = [math]::Round($wallSeconds, 1)
        idle_cpu_ms         = [math]::Round($cpuSeconds * 1000, 1)
        idle_cpu_percent_of_one_core = [math]::Round(100 * $cpuSeconds / $wallSeconds, 3)
    }
    if (-not $process.WaitForExit(30000)) { $process.Kill() }
    $idleFrames = Get-Content "$report.idle-frames" -ErrorAction SilentlyContinue
    $idle[-1] | Add-Member idle_frames_drawn ($idleFrames ?? 'not written')
    $frames += Import-Csv $report
}

$frames | Export-Csv (Join-Path $OutDir 'frame-time.csv') -NoTypeInformation
$idle | Export-Csv (Join-Path $OutDir 'idle-cpu.csv') -NoTypeInformation
$frames | Format-Table scale_percent, pixels_per_point, phase, frames, cpu_median_ms, cpu_p95_ms, cpu_max_ms, interval_median_ms, interval_p95_ms -AutoSize
$idle | Format-Table -AutoSize
