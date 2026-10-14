# SPDX-FileCopyrightText: 2026 Lucas Greuloch
# SPDX-License-Identifier: GPL-3.0-or-later
#Requires -Version 7.4
#Requires -RunAsAdministrator

<#
.SYNOPSIS
Records ONNX Runtime's ETW telemetry events and the network activity of the telemetry probe, once per variant.

.DESCRIPTION
Runs the probe in the variants env, env-only, api and none with an ETW trace session started before the probe, and once more
in the variant api with the session started after the first inference session. The trace session enables the
TraceLogging provider of ONNX Runtime and the Kernel-Network, DNS-Client and WinHttp providers. Each run gets empty
LOCALAPPDATA and APPDATA directories. The script writes summary.md, the ETL and XML traces and the probe output per
run into OutDir. Needs an elevated PowerShell 7.4 or later.

.PARAMETER Probe
Path of ort-telemetry-probe.exe.

.PARAMETER OrtDir
Absolute directory that holds the pinned onnxruntime.dll and nothing else.

.PARAMETER OutDir
Directory for the results; created if missing.

.PARAMETER HoldSeconds
Seconds between the two inference sessions of a run.

.PARAMETER LingerSeconds
Seconds the probe stays alive after the second session.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $Probe,
    [Parameter(Mandatory)] [string] $OrtDir,
    [Parameter(Mandatory)] [string] $OutDir,
    [int] $HoldSeconds = 10,
    [int] $LingerSeconds = 120
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$SessionName = 'TallylineOrtTelemetrySpike'
$OrtProvider = '3a26b1ff-7484-7484-7484-15261f42614d'
$NetworkProviders = [ordered]@{
    '7dd42a49-5329-4832-8dfd-43d979153a88' = 'Microsoft-Windows-Kernel-Network'
    '1c95126e-7eea-49a9-a3fe-a378b03ddb4d' = 'Microsoft-Windows-DNS-Client'
    '7d44233d-3055-4b9c-ba64-0d47ca40a232' = 'Microsoft-Windows-WinHttp'
}
$LateTraceMinimumHold = 20
$LateTraceWaitSeconds = 60
$DeviceIdKey = 'HKCU:\SOFTWARE\Microsoft\DeveloperTools\.onnxruntime'

$Probe = (Resolve-Path $Probe).Path
if (-not [IO.Path]::IsPathRooted($OrtDir)) { throw "OrtDir must be absolute: $OrtDir" }
New-Item -ItemType Directory -Force $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path
$ProviderFile = Join-Path $OutDir 'providers.txt'
@("{$OrtProvider} 0xffffffffffffffff 0xff") + ($NetworkProviders.Keys | ForEach-Object { "{$_} 0xffffffffffffffff 0xff" }) |
    Set-Content -Encoding ascii $ProviderFile

function Get-NormalisedGuid([string] $Value) {
    $Value.Trim('{', '}', ' ').ToLowerInvariant()
}

function Start-SpikeTrace([string] $Etl) {
    & logman start $SessionName -pf $ProviderFile -o $Etl -ets | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "logman start failed with exit code $LASTEXITCODE" }
}

function Stop-SpikeTrace {
    & logman stop $SessionName -ets | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "logman stop failed with exit code $LASTEXITCODE" }
}

function Get-EventDataValue($Record, [string] $Name) {
    $data = $Record.SelectNodes("*[local-name()='EventData']/*[local-name()='Data'][@Name='$Name']")
    if ($data.Count -gt 0) { $data[0].InnerText } else { $null }
}

function Get-EventName($Record) {
    $task = $Record.SelectSingleNode("*[local-name()='RenderingInfo']/*[local-name()='Task']")
    if ($null -ne $task -and $task.InnerText) { return $task.InnerText }
    $id = $Record.SelectSingleNode("*[local-name()='System']/*[local-name()='EventID']")
    "EventID $($id.InnerText)"
}

function Get-RelativeFiles([string] $Root) {
    @(Get-ChildItem -Recurse -File -Force $Root -ErrorAction SilentlyContinue |
            ForEach-Object { "$([IO.Path]::GetRelativePath($OutDir, $_.FullName)) ($($_.Length) bytes)" })
}

function Invoke-SpikeRun([string] $Name, [string] $Variant, [switch] $LateTrace) {
    $runDir = Join-Path $OutDir $Name
    $local = Join-Path $runDir 'LocalAppData'
    $roaming = Join-Path $runDir 'AppData'
    New-Item -ItemType Directory -Force $runDir, $local, $roaming | Out-Null
    $etl = Join-Path $runDir 'trace.etl'
    $xml = Join-Path $runDir 'trace.xml'
    $stdout = Join-Path $runDir 'probe.stdout.txt'
    $stderr = Join-Path $runDir 'probe.stderr.txt'
    $hold = if ($LateTrace) { [Math]::Max($HoldSeconds, $LateTraceMinimumHold) } else { $HoldSeconds }
    $arguments = "--variant $Variant --ort-dir `"$OrtDir`" --hold $hold --linger $LingerSeconds"
    $environment = @{ LOCALAPPDATA = $local; APPDATA = $roaming; ORT_DISABLE_TELEMETRY = $null }

    Write-Host "run $Name (variant $Variant): about $($hold + $LingerSeconds) s"
    $traceRunning = $false
    $process = $null
    try {
        if (-not $LateTrace) { Start-SpikeTrace $etl; $traceRunning = $true }
        $process = Start-Process -FilePath $Probe -ArgumentList $arguments -Environment $environment `
            -RedirectStandardOutput $stdout -RedirectStandardError $stderr -NoNewWindow -PassThru
        $null = $process.Handle
        if ($LateTrace) {
            $deadline = (Get-Date).AddSeconds($LateTraceWaitSeconds)
            while (-not ((Test-Path $stdout) -and (Select-String -Quiet -Path $stdout -Pattern '^hold='))) {
                if ($process.HasExited -or (Get-Date) -gt $deadline) { throw "the probe did not reach its hold" }
                Start-Sleep -Milliseconds 200
            }
            Start-SpikeTrace $etl
            $traceRunning = $true
        }
        $process.WaitForExit()
    }
    finally {
        if ($null -ne $process -and -not $process.HasExited) { $process.Kill() }
        if ($traceRunning) { Stop-SpikeTrace }
    }

    & tracerpt $etl -o $xml -of XML -y | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "tracerpt failed with exit code $LASTEXITCODE" }
    $document = New-Object System.Xml.XmlDocument
    $document.Load($xml)
    $events = @($document.SelectNodes("//*[local-name()='Event']"))
    $probeId = [string] $process.Id

    $ortEvents = [System.Collections.Generic.List[object]]::new()
    $networkEvents = [System.Collections.Generic.List[object]]::new()
    foreach ($record in $events) {
        $provider = $record.SelectSingleNode("*[local-name()='System']/*[local-name()='Provider']")
        $execution = $record.SelectSingleNode("*[local-name()='System']/*[local-name()='Execution']")
        if ($null -eq $provider) { continue }
        $guid = Get-NormalisedGuid $provider.GetAttribute('Guid')
        $processId = if ($null -ne $execution) { $execution.GetAttribute('ProcessID') } else { '' }
        $fromProbe = ($processId -eq $probeId) -or ((Get-EventDataValue $record 'PID') -eq $probeId)
        if ($guid -eq $OrtProvider) {
            $ortEvents.Add([pscustomobject]@{ Name = Get-EventName $record; FromProbe = $fromProbe; Xml = $record.OuterXml })
        }
        elseif ($NetworkProviders.Contains($guid) -and $fromProbe) {
            $fields = @($record.SelectNodes("*[local-name()='EventData']/*[local-name()='Data']") |
                    ForEach-Object { "$($_.GetAttribute('Name'))=$($_.InnerText)" }) -join '; '
            $networkEvents.Add([pscustomobject]@{ Provider = $NetworkProviders[$guid]; Name = Get-EventName $record; Fields = $fields })
        }
    }
    $ortEvents | ForEach-Object { $_.Xml } | Set-Content -Encoding utf8 (Join-Path $runDir 'ort-events.xml')
    $networkCsv = Join-Path $runDir 'network-events.csv'
    '"Provider","Name","Fields"' | Set-Content -Encoding utf8 $networkCsv
    $networkEvents | ConvertTo-Csv -NoTypeInformation | Select-Object -Skip 1 | Add-Content -Encoding utf8 $networkCsv

    $lines = [System.Collections.Generic.List[string]]::new()
    $when = if ($LateTrace) { 'after the first inference session' } else { 'before the probe' }
    $lines.Add("## $Name (variant ``$Variant``, trace session started $when)")
    $lines.Add('')
    $lines.Add("- probe exit code: $($process.ExitCode)")
    $lines.Add("- probe output: $((Get-Content $stdout | Where-Object { $_ -match '^(variant|ORT_DISABLE_TELEMETRY|session|done)' }) -join ' | ')")
    $stderrText = (Get-Content -Raw $stderr -ErrorAction SilentlyContinue)
    if ($stderrText) { $lines.Add("- probe stderr: $($stderrText.Trim())") }
    $lines.Add("- events of the ONNX Runtime provider: $($ortEvents.Count) ($(@($ortEvents | Where-Object FromProbe).Count) from the probe process)")
    foreach ($group in ($ortEvents | Group-Object Name | Sort-Object Name)) { $lines.Add("  - $($group.Name): $($group.Count)") }
    $lines.Add("- network events of the probe process: $($networkEvents.Count)")
    foreach ($group in ($networkEvents | Group-Object Provider, Name | Sort-Object Name)) { $lines.Add("  - $($group.Name): $($group.Count)") }
    $files = @(Get-RelativeFiles $local) + @(Get-RelativeFiles $roaming)
    $lines.Add("- files created under the run's LOCALAPPDATA and APPDATA: $($files.Count)")
    foreach ($file in $files) { $lines.Add("  - $file") }
    $lines.Add('')
    $lines
}

& logman stop $SessionName -ets 2>&1 | Out-Null

$summary = [System.Collections.Generic.List[string]]::new()
$summary.Add('# ONNX Runtime telemetry spike: Windows measurement')
$summary.Add('')
$summary.Add("- date (UTC): $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm'))")
$summary.Add("- Windows: $([Environment]::OSVersion.VersionString)")
$summary.Add("- onnxruntime.dll SHA-256: $((Get-FileHash (Join-Path $OrtDir 'onnxruntime.dll')).Hash.ToLowerInvariant())")
$diagTrack = Get-Service DiagTrack -ErrorAction SilentlyContinue
$summary.Add("- DiagTrack service: $(if ($diagTrack) { "$($diagTrack.Status), start type $($diagTrack.StartType)" } else { 'not present' })")
foreach ($policy in 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection', 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection') {
    $value = try { Get-ItemPropertyValue $policy -Name AllowTelemetry } catch { $null }
    $summary.Add("- AllowTelemetry under $($policy -replace '^HKLM:\\', 'HKLM\'): $(if ($null -ne $value) { $value } else { 'not set' })")
}
$listener = (& logman query Diagtrack-Listener -ets 2>&1) -join "`n"
$listenerRunning = $LASTEXITCODE -eq 0
$summary.Add("- Diagtrack-Listener ETW session running: $listenerRunning; enables the ONNX Runtime provider by GUID: $($listenerRunning -and $listener -match $OrtProvider)")
$summary.Add("- ONNX Runtime device-ID key in HKCU before the runs: $(Test-Path $DeviceIdKey)")
$summary.Add('')

$summary.AddRange([string[]] (Invoke-SpikeRun -Name '1-env' -Variant 'env'))
$summary.AddRange([string[]] (Invoke-SpikeRun -Name '2-env-only' -Variant 'env-only'))
$summary.AddRange([string[]] (Invoke-SpikeRun -Name '3-api' -Variant 'api'))
$summary.AddRange([string[]] (Invoke-SpikeRun -Name '4-api-late-trace' -Variant 'api' -LateTrace))
$summary.AddRange([string[]] (Invoke-SpikeRun -Name '5-none' -Variant 'none'))

$summary.Add("ONNX Runtime device-ID key in HKCU after the runs: $(Test-Path $DeviceIdKey)")
$summaryPath = Join-Path $OutDir 'summary.md'
$summary | Set-Content -Encoding utf8 $summaryPath
$summary | Write-Output
