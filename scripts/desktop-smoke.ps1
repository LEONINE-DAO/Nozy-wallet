# Desktop pre-release smoke tests (automated section A of RELEASE_SMOKE_CHECKLIST.md)
# Usage: .\scripts\desktop-smoke.ps1 [-SkipCli] [-SkipNpmBuild]

param(
    [switch]$SkipCli,
    [switch]$SkipNpmBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DesktopRoot = Join-Path $RepoRoot "desktop-client"
$TauriRoot = Join-Path $DesktopRoot "src-tauri"
$TargetDir = Join-Path $RepoRoot "target"
$env:CARGO_TARGET_DIR = $TargetDir

$results = @()

function Add-Result {
    param([string]$Id, [string]$Name, [bool]$Pass, [string]$Detail = "")
    $script:results += [PSCustomObject]@{
        Id     = $Id
        Name   = $Name
        Pass   = $Pass
        Detail = $Detail
    }
    $icon = if ($Pass) { "PASS" } else { "FAIL" }
    $line = "[$icon] $Id $Name"
    if ($Detail) { $line += " - $Detail" }
    Write-Host $line
}

function Invoke-External {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList,
        [string]$WorkingDirectory = $RepoRoot
    )
    Push-Location $WorkingDirectory
    try {
        $prevEap = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & $FilePath @ArgumentList 2>&1 | Out-Null
        $ErrorActionPreference = $prevEap
        return $LASTEXITCODE
    } finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "NozyWallet desktop smoke (automated)"
Write-Host "Repo: $RepoRoot"
Write-Host ""

# A1 - cargo check
$code = Invoke-External -FilePath "cargo" -ArgumentList @("check") -WorkingDirectory $TauriRoot
if ($code -ne 0) {
    Add-Result "A1" "Tauri cargo check" $false "exit $code"
} else {
    Add-Result "A1" "Tauri cargo check" $true
}

# A2 - restore_wallet test
$code = Invoke-External -FilePath "cargo" -ArgumentList @("test", "--test", "restore_wallet") -WorkingDirectory $TauriRoot
if ($code -ne 0) {
    Add-Result "A2" "restore_wallet integration test" $false "exit $code"
} else {
    Add-Result "A2" "restore_wallet integration test" $true
}

# A3 - npm build
if (-not $SkipNpmBuild) {
    $code = Invoke-External -FilePath "npm" -ArgumentList @("run", "build") -WorkingDirectory $DesktopRoot
    if ($code -ne 0) {
        Add-Result "A3" "Frontend npm run build" $false "exit $code"
    } else {
        Add-Result "A3" "Frontend npm run build" $true
    }
} else {
    Add-Result "A3" "Frontend npm run build" $true "skipped"
}

# A4 - tauri window config
try {
    $conf = Get-Content (Join-Path $TauriRoot "tauri.conf.json") -Raw | ConvertFrom-Json
    $win = $conf.app.windows | Where-Object { $_.label -eq "main" } | Select-Object -First 1
    $ok = ($null -ne $win) -and ($win.width -ge 800) -and ($win.height -ge 600)
    Add-Result "A4" "tauri.conf.json main window" $ok "width=$($win.width) height=$($win.height)"
} catch {
    Add-Result "A4" "tauri.conf.json main window" $false $_.Exception.Message
}

# A5 - capabilities
try {
    $capPath = Join-Path $TauriRoot "capabilities\default.json"
    $cap = Get-Content $capPath -Raw
    $ok = $cap -match "core:default"
    Add-Result "A5" "Tauri capabilities" $ok $capPath
} catch {
    Add-Result "A5" "Tauri capabilities" $false $_.Exception.Message
}

# A6 - invoke vs handler registration
try {
    $apiPath = Join-Path $DesktopRoot "src\lib\api.ts"
    $mainPath = Join-Path $TauriRoot "src\main.rs"
    $api = Get-Content $apiPath -Raw
    $main = Get-Content $mainPath -Raw
    $invokes = [regex]::Matches($api, 'invoke(?:<[^>]*>)?\("([a-z_]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique
    $missing = @()
    foreach ($cmd in $invokes) {
        if ($main -notmatch "\b$cmd\b") { $missing += $cmd }
    }
    $ok = $missing.Count -eq 0
    $detail = if ($ok) { "$($invokes.Count) commands registered" } else { "missing: $($missing -join ', ')" }
    Add-Result "A6" "walletApi invoke registration" $ok $detail
} catch {
    Add-Result "A6" "walletApi invoke registration" $false $_.Exception.Message
}

# A7 - wallet.dat exists
$walletDat = Join-Path $env:APPDATA "nozy\nozy\data\wallet.dat"
if (Test-Path $walletDat) {
    Add-Result "A7" "wallet.dat present" $true $walletDat
} else {
    Add-Result "A7" "wallet.dat present" $true "optional - not found (new install OK)"
}

# A8-A10 - CLI parity (same nozy core as desktop)
if (-not $SkipCli) {
    $nozy = Join-Path $TargetDir "release\nozy.exe"
    if (-not (Test-Path $nozy)) {
        Write-Host "Building nozy CLI for smoke..."
        $code = Invoke-External -FilePath "cargo" -ArgumentList @("build", "--release", "--bin", "nozy") -WorkingDirectory $RepoRoot
        if ($code -ne 0) {
            Add-Result "A8" "CLI balance" $false "nozy build failed"
            Add-Result "A9" "CLI history" $false "nozy build failed"
            Add-Result "A10" "CLI status" $false "nozy build failed"
            $nozy = $null
        }
    }
    if ($nozy -and (Test-Path $nozy)) {
        foreach ($pair in @(
                @{ Id = "A8"; Name = "CLI balance"; Args = @("balance") },
                @{ Id = "A9"; Name = "CLI history"; Args = @("history") },
                @{ Id = "A10"; Name = "CLI status"; Args = @("status") }
            )) {
            try {
                $prevEap = $ErrorActionPreference
                $ErrorActionPreference = "Continue"
                $cliOut = & $nozy @($pair.Args) 2>&1 | Out-String
                $code = $LASTEXITCODE
                $ErrorActionPreference = $prevEap
                if ($code -ne 0) {
                    Add-Result $pair.Id $pair.Name $false "exit $code"
                } elseif ($pair.Id -eq "A9" -and $cliOut -notmatch "transaction") {
                    Add-Result $pair.Id $pair.Name $false "no transactions in output"
                } else {
                    $snippet = ($cliOut -split "`n" | Select-Object -First 2) -join '; '
                    Add-Result $pair.Id $pair.Name $true $snippet.Trim()
                }
            } catch {
                Add-Result $pair.Id $pair.Name $false $_.Exception.Message
            }
        }
    } elseif (-not (Test-Path $nozy)) {
        Add-Result "A8" "CLI balance" $false "nozy.exe not built"
        Add-Result "A9" "CLI history" $false "nozy.exe not built"
        Add-Result "A10" "CLI status" $false "nozy.exe not built"
    }
} else {
    Add-Result "A8" "CLI balance" $true "skipped"
    Add-Result "A9" "CLI history" $true "skipped"
    Add-Result "A10" "CLI status" $true "skipped"
}

# A11 - history JSON unit test (desktop/API share this serializer)
$code = Invoke-External -FilePath "cargo" -ArgumentList @(
    "test", "-p", "nozy", "test_received_history_json_includes_type", "--", "--nocapture"
) -WorkingDirectory $RepoRoot
if ($code -ne 0) {
    Add-Result "A11" "nozy history JSON unit test" $false "exit $code"
} else {
    Add-Result "A11" "nozy history JSON unit test" $true
}

Write-Host ""
Write-Host "Summary"
Write-Host "-------"
$passed = ($results | Where-Object { $_.Pass }).Count
$failed = ($results | Where-Object { -not $_.Pass }).Count
Write-Host "Passed: $passed  Failed: $failed  Total: $($results.Count)"
Write-Host ""
Write-Host "Manual sections B-G: see desktop-client/RELEASE_SMOKE_CHECKLIST.md"
Write-Host ""

if ($failed -gt 0) {
    exit 1
}
exit 0
