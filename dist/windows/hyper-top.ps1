$ErrorActionPreference = 'Stop'
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir '..\..')
Set-Location $repoRoot

$hyperTop = Get-Command hyper-top -ErrorAction SilentlyContinue
if ($hyperTop) {
    & $hyperTop.Source @args
    exit $LASTEXITCODE
}

cargo run -- @args
exit $LASTEXITCODE
