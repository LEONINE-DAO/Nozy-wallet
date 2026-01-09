Set-Location $PSScriptRoot
Write-Host "Starting Tauri development server..."
Write-Host "Working directory: $(Get-Location)"
Write-Host "Config file exists: $(Test-Path 'src-tauri\tauri.conf.json')"

# Clear any problematic environment variables
$env:REMOVE_UNUSED_COMMANDS = $null
$env:TAURI_CONFIG_PATH = $null

& cargo tauri dev --config src-tauri/tauri.conf.json
