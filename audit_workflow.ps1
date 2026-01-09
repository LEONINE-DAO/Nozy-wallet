
# Audits .github/workflows/release.yml 

Write-Host "=== GitHub Actions Workflow Audit ===" -ForegroundColor Cyan
Write-Host ""

$workflowPath = ".github/workflows/release.yml"

if (-not (Test-Path $workflowPath)) {
    Write-Host "❌ Workflow file not found: $workflowPath" -ForegroundColor Red
    exit 1
}

Write-Host "Auditing: $workflowPath" -ForegroundColor Yellow
Write-Host ""

$workflowContent = Get-Content $workflowPath -Raw
$workflowLines = Get-Content $workflowPath

Write-Host "1. Checking YAML syntax..." -ForegroundColor Yellow
try {
    $yamlCheck = $workflowContent | ConvertFrom-Yaml -ErrorAction Stop
    Write-Host "   ✅ YAML syntax appears valid" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  YAML syntax check failed: $_" -ForegroundColor Yellow
    Write-Host "   Note: PowerShell YAML parsing is limited. Use actionlint for full validation." -ForegroundColor Cyan
}

Write-Host "`n2. Checking for hardcoded secrets..." -ForegroundColor Yellow
$secretPatterns = @(
    "password\s*[:=]\s*['`"][^'`"]+['`"]",
    "secret\s*[:=]\s*['`"][^'`"]+['`"]",
    "token\s*[:=]\s*['`"][^'`"]+['`"]",
    "api[_-]?key\s*[:=]\s*['`"][^'`"]+['`"]",
    "private[_-]?key\s*[:=]\s*['`"][^'`"]+['`"]"
)
$secretsFound = $false
foreach ($pattern in $secretPatterns) {
    $matches = Select-String -Path $workflowPath -Pattern $pattern -CaseSensitive:$false
    if ($matches) {
        $secretsFound = $true
        Write-Host "   ⚠️  Potential hardcoded secret found:" -ForegroundColor Red
        $matches | ForEach-Object { 
            Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Yellow 
        }
    }
}
if (-not $secretsFound) {
    Write-Host "   ✅ No hardcoded secrets found" -ForegroundColor Green
}

Write-Host "`n3. Checking secret usage..." -ForegroundColor Yellow
$secretUsage = Select-String -Path $workflowPath -Pattern "secrets\." -CaseSensitive:$false
if ($secretUsage) {
    Write-Host "   ✅ Using secrets (good practice):" -ForegroundColor Green
    $secretUsage | ForEach-Object { 
        Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Cyan 
    }
} else {
    Write-Host "   ℹ️  No secrets usage found" -ForegroundColor Cyan
}

Write-Host "`n4. Checking permissions..." -ForegroundColor Yellow
$permissions = Select-String -Path $workflowPath -Pattern "permissions:" -CaseSensitive:$false
if ($permissions) {
    Write-Host "   ✅ Permissions section found (good practice)" -ForegroundColor Green
    $permissionLines = Select-String -Path $workflowPath -Pattern "contents:\s*(write|read)" -CaseSensitive:$false
    if ($permissionLines) {
        Write-Host "   Permissions configured:" -ForegroundColor Cyan
        $permissionLines | ForEach-Object { 
            Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Cyan 
        }
    }
} else {
    Write-Host "   ⚠️  No permissions section found (consider adding for security)" -ForegroundColor Yellow
}

Write-Host "`n5. Checking security best practices..." -ForegroundColor Yellow
$issues = @()

$unpinnedActions = Select-String -Path $workflowPath -Pattern "uses:\s*[^@]+@(?!v\d+\.\d+\.\d+|v\d+\.\d+)" -CaseSensitive:$false
if ($unpinnedActions) {
    $issues += "⚠️  Some actions may not be pinned to specific versions"
    Write-Host "   ⚠️  Check action version pinning:" -ForegroundColor Yellow
    $unpinnedActions | ForEach-Object { 
        Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Yellow 
    }
} else {
    Write-Host "   ✅ Actions appear to be versioned" -ForegroundColor Green
}

$timeouts = Select-String -Path $workflowPath -Pattern "timeout-minutes:" -CaseSensitive:$false
if ($timeouts) {
    Write-Host "   ✅ Timeout settings found (good practice)" -ForegroundColor Green
} else {
    $issues += "⚠️  No timeout settings found (consider adding)"
    Write-Host "   ⚠️  No timeout settings found" -ForegroundColor Yellow
}

$continueOnError = Select-String -Path $workflowPath -Pattern "continue-on-error:\s*true" -CaseSensitive:$false
if ($continueOnError) {
    Write-Host "   ℹ️  continue-on-error found (review if intentional):" -ForegroundColor Cyan
    $continueOnError | ForEach-Object { 
        Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Cyan 
    }
}

Write-Host "`n6. Checking for common issues..." -ForegroundColor Yellow

$deprecatedActions = @("actions/checkout@v1", "actions/setup-node@v1")
foreach ($deprecated in $deprecatedActions) {
    if ($workflowContent -match [regex]::Escape($deprecated)) {
        Write-Host "   ⚠️  Deprecated action found: $deprecated" -ForegroundColor Yellow
        $issues += "⚠️  Deprecated action: $deprecated"
    }
}

$matrix = Select-String -Path $workflowPath -Pattern "strategy:" -CaseSensitive:$false
if ($matrix) {
    Write-Host "   ✅ Matrix strategy found (good for multi-platform builds)" -ForegroundColor Green
}

$artifacts = Select-String -Path $workflowPath -Pattern "upload-artifact" -CaseSensitive:$false
if ($artifacts) {
    Write-Host "   ✅ Artifact uploads configured" -ForegroundColor Green
}

Write-Host "`n7. Checking workflow structure..." -ForegroundColor Yellow
$hasJobs = $workflowContent -match "jobs:"
$hasSteps = $workflowContent -match "steps:"
$hasOn = $workflowContent -match "on:"

if ($hasJobs -and $hasSteps -and $hasOn) {
    Write-Host "   ✅ Workflow structure appears valid" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  Workflow structure may be incomplete" -ForegroundColor Yellow
    if (-not $hasJobs) { Write-Host "      Missing: jobs:" -ForegroundColor Yellow }
    if (-not $hasSteps) { Write-Host "      Missing: steps:" -ForegroundColor Yellow }
    if (-not $hasOn) { Write-Host "      Missing: on:" -ForegroundColor Yellow }
}

Write-Host "`n=== Audit Summary ===" -ForegroundColor Cyan
if ($issues.Count -eq 0) {
    Write-Host "✅ No major issues found!" -ForegroundColor Green
} else {
    Write-Host "⚠️  Issues found:" -ForegroundColor Yellow
    $issues | ForEach-Object { Write-Host "   $_" -ForegroundColor Yellow }
}

Write-Host "`n📋 Recommendations:" -ForegroundColor Cyan
Write-Host "   1. Use actionlint for comprehensive YAML validation:" -ForegroundColor White
Write-Host "      https://github.com/rhymond/actionlint" -ForegroundColor Gray
Write-Host "   2. Review GitHub Actions security best practices:" -ForegroundColor White
Write-Host "      https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions" -ForegroundColor Gray
Write-Host "   3. Consider using Dependabot for action updates" -ForegroundColor White
Write-Host "   4. Test workflow changes in a branch before merging" -ForegroundColor White

Write-Host ""
Write-Host 'Audit complete!' -ForegroundColor Green

