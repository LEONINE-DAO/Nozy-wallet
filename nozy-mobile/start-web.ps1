# Start NozyWallet mobile in Chrome (web mode — not the raw JSON at :8081).
$env:Path = "C:\Program Files\nodejs;C:\Program Files\Git\bin;" + $env:Path
Set-Location $PSScriptRoot
npm run web
