# PowerShell script to explore Secret Network contract examples

Write-Host "🔍 Exploring Secret Network Contract Examples" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""

# Create exploration directory
$ExploreDir = "secret-contracts-exploration"
New-Item -ItemType Directory -Force -Path $ExploreDir | Out-Null
Set-Location $ExploreDir

Write-Host "📦 Cloning Secret Network repositories..." -ForegroundColor Yellow
Write-Host ""

# 1. Clone secret-template
Write-Host "1️⃣  Cloning secret-template..." -ForegroundColor Green
if (-not (Test-Path "secret-template")) {
    git clone https://github.com/scrtlabs/secret-template.git
    Write-Host "   ✅ Cloned secret-template" -ForegroundColor Green
} else {
    Write-Host "   ℹ️  secret-template already exists" -ForegroundColor Yellow
}
Write-Host ""

# 2. Clone secret-contracts
Write-Host "2️⃣  Cloning secret-contracts..." -ForegroundColor Green
if (-not (Test-Path "secret-contracts")) {
    git clone https://github.com/scrtlabs/secret-contracts.git
    Write-Host "   ✅ Cloned secret-contracts" -ForegroundColor Green
} else {
    Write-Host "   ℹ️  secret-contracts already exists" -ForegroundColor Yellow
}
Write-Host ""

# 3. Clone secret-toolkit
Write-Host "3️⃣  Cloning secret-toolkit..." -ForegroundColor Green
if (-not (Test-Path "secret-toolkit")) {
    git clone https://github.com/scrtlabs/secret-toolkit.git
    Write-Host "   ✅ Cloned secret-toolkit" -ForegroundColor Green
} else {
    Write-Host "   ℹ️  secret-toolkit already exists" -ForegroundColor Yellow
}
Write-Host ""

# 4. Clone secret.js
Write-Host "4️⃣  Cloning secret.js..." -ForegroundColor Green
if (-not (Test-Path "secret.js")) {
    git clone https://github.com/scrtlabs/secret.js.git
    Write-Host "   ✅ Cloned secret.js" -ForegroundColor Green
} else {
    Write-Host "   ℹ️  secret.js already exists" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📚 Repository Structure:" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host ""
Write-Host "secret-template/          - Starter template for new contracts"
Write-Host "secret-contracts/         - Official contract examples"
Write-Host "secret-toolkit/           - Utility library"
Write-Host "secret.js/                - JavaScript SDK"
Write-Host ""

Write-Host "🎯 Key Contracts to Study:" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. secret-template/src/    - Basic contract structure"
Write-Host "2. secret-contracts/snip20-reference-impl/ - Privacy token example"
Write-Host "3. secret-contracts/snip721-reference-impl/ - NFT example"
Write-Host "4. secret-toolkit/         - Common utilities"
Write-Host ""

Write-Host "📖 Next Steps:" -ForegroundColor Cyan
Write-Host "==============" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Explore secret-template/src/ to understand basic structure"
Write-Host "2. Study secret-contracts/snip20-reference-impl/ for privacy patterns"
Write-Host "3. Review secret-toolkit/ for utilities"
Write-Host "4. Check secret.js/examples/ for integration patterns"
Write-Host ""

Write-Host "✅ Exploration setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Location: $(Get-Location)"
Write-Host ""
Write-Host "Start exploring:"
Write-Host "  cd secret-template; Get-Content README.md"
Write-Host "  cd ..\secret-contracts; Get-ChildItem"
Write-Host ""

