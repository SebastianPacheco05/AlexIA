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
& $taskTunnelPath tunnel --url "http://127.0.0.1:$Port" --no-autoupdate
exit $LASTEXITCODE
