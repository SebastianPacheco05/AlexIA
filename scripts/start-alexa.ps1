$ErrorActionPreference = 'Stop'
$taskProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $taskProjectRoot
$env:ALEXA_VERIFIER_PATH = Join-Path $taskProjectRoot 'tools\alexa-verifier\verify.cjs'
$env:ALEXA_NODE_BIN = (Get-Command node.exe -ErrorAction Stop).Source
if (-not $env:ALEXA_OPENSSL_BIN) {
    $taskOpenSsl = Get-Command openssl.exe -ErrorAction SilentlyContinue
    if ($taskOpenSsl) { $env:ALEXA_OPENSSL_BIN = $taskOpenSsl.Source }
    else {
        $taskGitOpenSsl = Join-Path $env:ProgramFiles 'Git\usr\bin\openssl.exe'
        if (-not (Test-Path -LiteralPath $taskGitOpenSsl)) { throw 'Configura ALEXA_OPENSSL_BIN con la ruta de OpenSSL 3 o superior.' }
        $env:ALEXA_OPENSSL_BIN = $taskGitOpenSsl
    }
}
if (-not $env:AGY_BIN) {
    $taskAgy = Get-Command agy.exe -ErrorAction SilentlyContinue
    if ($taskAgy) { $env:AGY_BIN = $taskAgy.Source }
    else {
        $taskAgyPath = Join-Path $env:USERPROFILE 'AppData\Local\agy\bin\agy.exe'
        if (-not (Test-Path -LiteralPath $taskAgyPath)) { throw 'Configura AGY_BIN con la ruta de agy.exe.' }
        $env:AGY_BIN = $taskAgyPath
    }
}
$taskCargo = Get-Command cargo.exe -ErrorAction SilentlyContinue
if ($taskCargo) { $taskCargoPath = $taskCargo.Source }
else { $taskCargoPath = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }

# Stop listeners on the backend and UI ports configured in .env before launching AlexIA.
$taskEnvLines = Get-Content -LiteralPath (Join-Path $taskProjectRoot '.env') -ErrorAction SilentlyContinue
$taskBindAddress = '127.0.0.1:3000'
$taskUiPort = 3001
$taskBindLine = $taskEnvLines | Where-Object { $_ -match '^\s*BIND_ADDR\s*=' } | Select-Object -Last 1
$taskUiPortLine = $taskEnvLines | Where-Object { $_ -match '^\s*UI_PORT\s*=' } | Select-Object -Last 1
if ($taskBindLine -and $taskBindLine -match '^\s*BIND_ADDR\s*=\s*["'']?(.*?)["'']?\s*$') {
    $taskBindAddress = $Matches[1]
}
if ($taskUiPortLine -and $taskUiPortLine -match '^\s*UI_PORT\s*=\s*["'']?(\d+)["'']?\s*$') {
    $taskUiPort = [int]$Matches[1]
}
$taskPortsToFree = @($taskUiPort)
if ($taskBindAddress -match ':(\d+)$') {
    $taskPortsToFree += [int]$Matches[1]
}
foreach ($taskPort in ($taskPortsToFree | Select-Object -Unique)) {
    $taskListenerPids = Get-NetTCPConnection -LocalPort $taskPort -State Listen -ErrorAction SilentlyContinue |
        Select-Object -ExpandProperty OwningProcess -Unique
    foreach ($taskListenerPid in $taskListenerPids) {
        if ($taskListenerPid -ne $PID) {
            Write-Host "Deteniendo el proceso que ocupa el puerto $taskPort (PID $taskListenerPid)..."
            Stop-Process -Id $taskListenerPid -Force -ErrorAction Stop
            Wait-Process -Id $taskListenerPid -Timeout 10 -ErrorAction SilentlyContinue
        }
    }
}

& $taskCargoPath run --locked --target-dir target/connection-check
exit $LASTEXITCODE
