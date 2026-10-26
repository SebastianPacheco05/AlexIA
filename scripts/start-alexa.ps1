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
& $taskCargoPath run --locked --target-dir target/connection-check
exit $LASTEXITCODE
