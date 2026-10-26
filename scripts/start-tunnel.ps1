param([ValidateRange(0,65535)][int]$Port = 0)
$ErrorActionPreference = 'Stop'
$taskProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $taskProjectRoot
if ($Port -eq 0) {
    $Port = 3000
    $taskEnvPath = Join-Path $taskProjectRoot '.env'
    if (Test-Path -LiteralPath $taskEnvPath) {
        foreach ($taskLine in [IO.File]::ReadAllLines($taskEnvPath)) {
            if ($taskLine -match '^\s*BIND_ADDR\s*=\s*[''""]?[^''""\r\n]+:(\d+)[''"" ]*\s*$') { $Port = [int]$Matches[1] }
        }
    }
}
$taskTunnel = Get-Command cloudflared.exe -ErrorAction SilentlyContinue
if ($taskTunnel) { $taskTunnelPath = $taskTunnel.Source }
else {
    $taskTunnelPath = Join-Path $taskProjectRoot 'target\tools\cloudflared.exe'
    if (-not (Test-Path -LiteralPath $taskTunnelPath)) {
        throw 'Descarga cloudflared desde https://developers.cloudflare.com/tunnel/downloads/ o agrega su ejecutable al PATH.'
    }
}
# Keep transient tunnel discovery separate from .env and settings revisions.
$taskRuntime = Join-Path $taskProjectRoot 'target\tunnels'
[IO.Directory]::CreateDirectory($taskRuntime) | Out-Null
$taskRunId = [Guid]::NewGuid().ToString('N')
$taskStatePath = Join-Path $taskRuntime "$taskRunId.json"
$taskStdout = Join-Path $taskRuntime "$taskRunId.stdout.log"
$taskStderr = Join-Path $taskRuntime "$taskRunId.stderr.log"
$taskTunnelProcess = $null
$taskReaders = @()
$taskUrl = $null
$taskLastHeartbeat = [DateTime]::MinValue
$taskExitCode = 1
try {
    $taskTunnelProcess = Start-Process -FilePath $taskTunnelPath -ArgumentList @('tunnel', '--url', "http://127.0.0.1:$Port", '--no-autoupdate') -WindowStyle Hidden -PassThru -RedirectStandardOutput $taskStdout -RedirectStandardError $taskStderr
    foreach ($taskLog in @($taskStdout, $taskStderr)) {
        $taskStream = [IO.File]::Open($taskLog, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
        $taskReaders += [IO.StreamReader]::new($taskStream)
    }
    while ($true) {
        foreach ($taskReader in $taskReaders) {
            while ($null -ne ($taskOutputLine = $taskReader.ReadLine())) {
                Write-Host $taskOutputLine
                if ($taskOutputLine -match 'https://[a-z0-9-]+\.trycloudflare\.com\b') { $taskUrl = $Matches[0] }
            }
        }
        $taskTunnelProcess.Refresh()
        if ($taskTunnelProcess.HasExited) { $taskExitCode = $taskTunnelProcess.ExitCode; break }
        if ($taskUrl -and ([DateTime]::UtcNow - $taskLastHeartbeat).TotalSeconds -ge 2) {
            $taskState = @{ url = $taskUrl; port = $Port; updated_at = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() } | ConvertTo-Json -Compress
            [IO.File]::WriteAllText("$taskStatePath.tmp", $taskState, [Text.UTF8Encoding]::new($false))
            Move-Item -LiteralPath "$taskStatePath.tmp" -Destination $taskStatePath -Force
            $taskLastHeartbeat = [DateTime]::UtcNow
        }
        Start-Sleep -Milliseconds 250
    }
} finally {
    if ($taskTunnelProcess -and -not $taskTunnelProcess.HasExited) { $taskTunnelProcess.Kill() }
    foreach ($taskReader in $taskReaders) { $taskReader.Dispose() }
    foreach ($taskFile in @($taskStatePath, "$taskStatePath.tmp")) {
        if (Test-Path -LiteralPath $taskFile) { Remove-Item -LiteralPath $taskFile -Force }
    }
}
exit $taskExitCode