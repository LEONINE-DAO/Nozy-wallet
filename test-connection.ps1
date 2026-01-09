# NozyWallet Connection Test Script (PowerShell)
# Tests connectivity to zec.leoninedao.org:443

$TestHost = "zec.leoninedao.org"
$TestPort = 443
$TestIP = "45.55.79.71"

Write-Host "=== NozyWallet Connection Test ===" -ForegroundColor Cyan
Write-Host ""

# 1. DNS Resolution
Write-Host "1. Testing DNS resolution:" -ForegroundColor Yellow
try {
    $dnsResult = Resolve-DnsName -Name $TestHost -ErrorAction Stop
    Write-Host "✅ DNS resolution successful" -ForegroundColor Green
    Write-Host "   IP Address: $($dnsResult[0].IPAddress)"
} catch {
    Write-Host "❌ DNS resolution failed: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 2. TCP Port Test
Write-Host "2. Testing port connectivity (TCP):" -ForegroundColor Yellow
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $connect = $tcpClient.BeginConnect($TestHost, $TestPort, $null, $null)
    $wait = $connect.AsyncWaitHandle.WaitOne(5000, $false)
    
    if ($wait) {
        $tcpClient.EndConnect($connect)
        Write-Host "✅ Port ${TestPort} is reachable on ${TestHost}" -ForegroundColor Green
        $tcpClient.Close()
    } else {
        Write-Host "❌ Port $TestPort not reachable on $TestHost (timeout)" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Port $TestPort not reachable: $_" -ForegroundColor Red
}
Write-Host ""

# 3. Test with IP
Write-Host "3. Testing with IP address:" -ForegroundColor Yellow
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $connect = $tcpClient.BeginConnect($TestIP, $TestPort, $null, $null)
    $wait = $connect.AsyncWaitHandle.WaitOne(5000, $false)
    
    if ($wait) {
        $tcpClient.EndConnect($connect)
        Write-Host "✅ Port $TestPort is reachable on IP $TestIP" -ForegroundColor Green
        $tcpClient.Close()
    } else {
        Write-Host "❌ Port $TestPort not reachable on IP $TestIP (timeout)" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Port $TestPort not reachable on IP: $_" -ForegroundColor Red
}
Write-Host ""

# 4. HTTPS Connection Test
Write-Host "4. Testing HTTPS connection:" -ForegroundColor Yellow
try {
    # Ignore SSL certificate errors for testing
    [System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12 -bor [System.Net.SecurityProtocolType]::Tls11 -bor [System.Net.SecurityProtocolType]::Tls
    
    $uri = "https://${TestHost}:${TestPort}"
    $response = Invoke-WebRequest -Uri $uri -Method Get -TimeoutSec 30 -UseBasicParsing -ErrorAction Stop
    
    Write-Host "✅ HTTPS connection successful" -ForegroundColor Green
    Write-Host "   Status Code: $($response.StatusCode)"
    Write-Host "   Status Description: $($response.StatusDescription)"
} catch {
    Write-Host "❌ HTTPS connection failed: $_" -ForegroundColor Red
    Write-Host "   Full error details:" -ForegroundColor Yellow
    $_.Exception | Format-List -Force
}
Write-Host ""

# 5. Firewall Check (Windows)
Write-Host "5. Checking Windows Firewall:" -ForegroundColor Yellow
try {
    $firewallRules = Get-NetFirewallRule | Where-Object { $_.DisplayName -like "*$TestPort*" -or $_.LocalPort -eq $TestPort }
    if ($firewallRules) {
        Write-Host "   Found firewall rules for port ${Port}:"
        $firewallRules | Select-Object DisplayName, Enabled, Direction | Format-Table
    } else {
        Write-Host "   No specific firewall rules found for port ${Port}"
    }
} catch {
    Write-Host "   Could not check firewall (may require admin privileges)"
}
Write-Host ""

# 6. External Connection Test
Write-Host "6. Testing from external perspective:" -ForegroundColor Yellow
Write-Host "   Simulating external connection test..."
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $connect = $tcpClient.BeginConnect($TestIP, $TestPort, $null, $null)
    $wait = $connect.AsyncWaitHandle.WaitOne(5000, $false)
    
    if ($wait) {
        $tcpClient.EndConnect($connect)
        Write-Host "✅ Port $TestPort is reachable externally on $TestIP" -ForegroundColor Green
        $tcpClient.Close()
    } else {
        Write-Host "❌ Port $TestPort not reachable externally" -ForegroundColor Red
        Write-Host "   Possible issues:" -ForegroundColor Yellow
        Write-Host "   - Server firewall blocking"
        Write-Host "   - Cloud provider security group"
        Write-Host "   - Service not listening on $TestPort"
    }
} catch {
    Write-Host "❌ External connection test failed: $_" -ForegroundColor Red
}
Write-Host ""

# 7. HTTP/2 Test
Write-Host "7. Testing HTTP/2 support:" -ForegroundColor Yellow
try {
    $uri = "https://${TestHost}:${TestPort}"
    $request = [System.Net.HttpWebRequest]::Create($uri)
    $request.Method = "GET"
    $request.Timeout = 30000
    $request.ProtocolVersion = [System.Net.HttpVersion]::Version20
    
    $response = $request.GetResponse()
    Write-Host "✅ HTTP/2 connection successful" -ForegroundColor Green
    $response.Close()
} catch {
    Write-Host "⚠️  HTTP/2 test failed, may fall back to HTTP/1.1" -ForegroundColor Yellow
}
Write-Host ""

# 8. API Health Check
Write-Host "8. Testing API endpoint:" -ForegroundColor Yellow
try {
    $healthUri = "https://${Host}:${Port}/api/health"
    $response = Invoke-WebRequest -Uri $healthUri -Method Get -TimeoutSec 30 -UseBasicParsing -ErrorAction Stop
    
    Write-Host "✅ API health check passed" -ForegroundColor Green
    Write-Host "   Response: $($response.Content)"
} catch {
    Write-Host "⚠️  API health check failed or endpoint not available" -ForegroundColor Yellow
    Write-Host "   Trying root endpoint:"
    try {
        $rootUri = "https://${Host}:${Port}/"
        $rootResponse = Invoke-WebRequest -Uri $rootUri -Method Get -TimeoutSec 30 -UseBasicParsing
        Write-Host "   Root endpoint response: $($rootResponse.StatusCode)"
    } catch {
        Write-Host "   Root endpoint also failed"
    }
}
Write-Host ""

Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Host: $TestHost"
Write-Host "IP: $TestIP"
Write-Host "Port: $TestPort"
Write-Host ""
Write-Host "If connection fails, check:" -ForegroundColor Yellow
Write-Host "1. Server is running and listening on port $TestPort"
Write-Host "2. Firewall allows incoming connections on port $TestPort"
Write-Host "3. Cloud provider security groups allow port $TestPort"
Write-Host "4. SSL certificate is valid"
Write-Host "5. Service is configured to listen on 0.0.0.0:$TestPort (not just localhost)"

