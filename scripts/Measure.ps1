[CmdletBinding()]
param(
    [ValidateRange(10, 3600)][int]$Seconds = 30,
    [ValidateSet('all', 'demo-60', 'udp-60', 'udp-120', 'idle', 'recreate-device', 'udp-disconnect')][string]$Case = 'all',
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not $SkipBuild) { & (Join-Path $PSScriptRoot 'Build.ps1') -Test -Examples -Bench }
$executable = Join-Path $workspace 'target\release\f1-halo-telemetry.exe'
$senderExecutable = Join-Path $workspace 'target\release\examples\udp_demo.exe'
$resultsDirectory = Join-Path $workspace 'artifacts'
New-Item -ItemType Directory -Force -Path $resultsDirectory | Out-Null
$settingsDirectory = Join-Path $workspace '.cache\measure'
New-Item -ItemType Directory -Force -Path $settingsDirectory | Out-Null

function Invoke-Measurement {
    param([string]$Name, [int]$Duration, [int]$Frequency, [switch]$Demo, [switch]$Recreate, [switch]$Disconnect)
    $probe = [System.Net.Sockets.UdpClient]::new(0)
    $port = $probe.Client.LocalEndPoint.Port
    $probe.Dispose()
    $settingsPath = Join-Path $settingsDirectory "$Name-settings.json"
    $reportPath = Join-Path $resultsDirectory "$Name.json"
    @{
        schema_version = 1; bind_ip = '127.0.0.1'; port = $port; hotkey = 'Ctrl+Shift+F24'
        halo = @{ x = 650; y = 686; scale = 1.0; opacity = 0.94; visible = $true }
        tyres = @{ x = 28; y = 626; scale = 1.0; opacity = 0.94; visible = $true }
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $settingsPath -Encoding utf8NoBOM
    $arguments = @('--no-settings', '--config', "`"$settingsPath`"", '--diagnostics', "`"$reportPath`"", '--quit-after', "$Duration", '--fps', "$([Math]::Max(60, $Frequency))")
    if ($Demo) { $arguments += '--demo' }
    if ($Recreate) { $arguments += @('--recreate-device-after', '3') }
    $app = Start-Process -FilePath $executable -WorkingDirectory $workspace -ArgumentList $arguments -WindowStyle Hidden -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds($Duration + 20)
    $sender = $null
    $memorySamples = [System.Collections.Generic.List[object]]::new()
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        if ($Frequency -gt 0 -and -not $Demo) {
            Start-Sleep -Milliseconds 400
            $sendSeconds = if ($Disconnect) { 3 } else { $Duration - 1 }
            $sender = Start-Process -FilePath $senderExecutable -ArgumentList @("127.0.0.1:$port", "$Frequency", "$sendSeconds") -WindowStyle Hidden -PassThru
        }
        while (-not $app.WaitForExit(10000)) {
            if ([DateTime]::UtcNow -gt $deadline) { throw "Timeout applicazione: $Name" }
            $app.Refresh()
            if (-not $app.HasExited) {
                $memorySamples.Add([pscustomobject]@{
                    ElapsedSeconds = [Math]::Round($clock.Elapsed.TotalSeconds, 3)
                    WorkingSetBytes = $app.WorkingSet64
                    PrivateBytes = $app.PrivateMemorySize64
                    Handles = $app.HandleCount
                })
            }
            Write-Output "${Name}: misurazione in corso"
        }
        if ($memorySamples.Count -gt 0) {
            $memorySamples | Export-Csv -LiteralPath (Join-Path $resultsDirectory "$Name-memory.csv") -NoTypeInformation -Encoding utf8NoBOM
        }
        if ($app.ExitCode -ne 0) { throw "Applicazione terminata con codice $($app.ExitCode): $Name" }
        if ($sender) { $sender.WaitForExit(5000) | Out-Null }
        $report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
        if ($report.metrics.presentation_failures -ne 0) { throw "Errori di rendering: $Name" }
        if ($Demo -or $Frequency -gt 0) {
            if ($report.metrics.unique_telemetry_frames -eq 0) { throw "Nessun frame telemetrico: $Name" }
        }
        if ($Recreate -and $report.metrics.device_recreations -lt 1) { throw 'Ricreazione device non eseguita.' }
        if ($Disconnect -and $report.connection_state -ne 'DATI OBSOLETI') { throw 'La disconnessione non ha invalidato i dati.' }
        if ($Name -eq 'idle' -and $report.metrics.total_rendered_frames -gt 8) { throw 'Rendering non sospeso durante inattività.' }
        [pscustomobject]@{
            Case = $Name; Hz = $report.metrics.effective_render_hz
            DrawP95ms = $report.metrics.draw_cpu_ms_p95
            IntervalP95ms = $report.metrics.frame_interval_ms_p95
            ReceiveToSubmitP95ms = $report.metrics.receive_to_present_submit_ms_p95
            GpuP95ms = $report.metrics.gpu_ms_p95
            CpuPercent = $report.metrics.process_cpu_percent
            WorkingSetMB = $report.metrics.working_set_bytes / 1MB
            Frames = $report.metrics.total_rendered_frames
        }
    } finally {
        if (-not $app.HasExited) { Stop-Process -Id $app.Id }
        if ($sender -and -not $sender.HasExited) { Stop-Process -Id $sender.Id }
    }
}

if ($Case -in @('all', 'demo-60')) { Invoke-Measurement -Name 'demo-60' -Duration $Seconds -Frequency 60 -Demo }
if ($Case -in @('all', 'udp-60')) { Invoke-Measurement -Name 'udp-60' -Duration $Seconds -Frequency 60 }
if ($Case -in @('all', 'udp-120')) { Invoke-Measurement -Name 'udp-120' -Duration $Seconds -Frequency 120 }
if ($Case -in @('all', 'idle')) { Invoke-Measurement -Name 'idle' -Duration 8 -Frequency 0 }
if ($Case -in @('all', 'recreate-device')) { Invoke-Measurement -Name 'recreate-device' -Duration 10 -Frequency 60 -Demo -Recreate }
if ($Case -in @('all', 'udp-disconnect')) { Invoke-Measurement -Name 'udp-disconnect' -Duration 8 -Frequency 60 -Disconnect }

Write-Output 'Traffico UDP generato da examples/udp_demo.rs. Nessuna misura di F1 25 o della latenza ottica.'
