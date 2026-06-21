# One command to start NozyWallet mobile (fixes PATH issues in some terminals).
$env:Path = "C:\Program Files\nodejs;C:\Program Files\Git\bin;" + $env:Path
Set-Location $PSScriptRoot
npm start
