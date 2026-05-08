# Sets $env:ZEBRA_RPC_URL for Zebrad running in WSL when you run Nozy from Windows (cargo, tests).
# Dot-source once per PowerShell session — no manual IP copy/paste after that:
#   . .\scripts\zebra-wsl-rpc.ps1
# Optional profile line (paths vary):
#   . C:\path\to\NozyWallet\scripts\zebra-wsl-rpc.ps1
#
# If ZEBRA_RPC_URL is already set, we skip unless -Force. Use -Localhost when Win11 "mirrored"
# / localhost forwarding reaches Zebrad on 127.0.0.1:8232 from Windows.

param(
    [string]$Distro = "Ubuntu",
    [int]$Port = 8232,
    [string]$Url = "",
    [switch]$Localhost,
    [switch]$Force,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

if ($env:ZEBRA_RPC_URL -and -not $Force -and -not $Url -and -not $Localhost) {
    if (-not $Quiet) {
        Write-Host "ZEBRA_RPC_URL already set ($($env:ZEBRA_RPC_URL)); leaving as-is. Use -Force to replace." -ForegroundColor DarkGray
    }
    return
}

if ($Url) {
    $env:ZEBRA_RPC_URL = $Url
} elseif ($Localhost) {
    $env:ZEBRA_RPC_URL = "http://127.0.0.1:$Port"
} else {
    $raw = (wsl -d $Distro -- hostname -I 2>$null)
    if (-not $raw) {
        throw "Could not get WSL IP. Is distro '$Distro' installed? Try: wsl -l -v"
    }
    $ip = ($raw -split "\s+")[0].Trim()
    if (-not $ip) {
        throw "Empty WSL IP from hostname -I"
    }
    $env:ZEBRA_RPC_URL = "http://${ip}:$Port"
}

if (-not $Quiet) {
    Write-Host "ZEBRA_RPC_URL = $($env:ZEBRA_RPC_URL)" -ForegroundColor Cyan
}
