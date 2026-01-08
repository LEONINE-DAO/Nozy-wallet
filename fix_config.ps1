$configPath = "$env:APPDATA\nozy\nozy\config\config.json"
$configDir = Split-Path $configPath

if (-not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

$config = @{
    zebra_url = "https://zec.leoninedao.org:443"
    crosslink_url = ""
    network = "mainnet"
    last_scan_height = $null
    theme = "dark"
    backend = "zebra"
    protocol = "jsonrpc"
    privacy_network = @{
        tor_enabled = $true
        tor_proxy = "socks5://127.0.0.1:9050"
        i2p_enabled = $false
        i2p_proxy = "http://127.0.0.1:4444"
        preferred_network = "tor"
        require_privacy_network = $true
    }
    zk_verification = @{
        enabled = $true
        default_level = "TrustRpc"
        risc_zero_prover_path = $null
        use_gpu = $true
        proof_cache_dir = $null
        auto_generate_proofs = $false
    }
    secret_network = @{
        lcd_url = "https://api.secretapi.io"
        address = $null
    }
    swap = @{
        auto_churn = $false
        api_url = "https://api.swap-service.example.com"
        api_key = $null
    }
}

$config | ConvertTo-Json -Depth 10 | Out-File -FilePath $configPath -Encoding UTF8 -NoNewline
Write-Host "✅ Config file created/updated" -ForegroundColor Green
Write-Host "   Zebra URL: https://zec.leoninedao.org:443" -ForegroundColor Cyan
