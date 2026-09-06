# Poll Crosslink GUI until sync looks real (one-block tip, near network height).
$ErrorActionPreference = "Continue"
$Rpc = "http://127.0.0.1:18232"
$Log = "C:\Users\User\NozyWallet\tools\crosslink\sync-watch.log"
$GuiName = "zebrad_win32_gui_v13"
$TargetHeight = 280000
$IntervalSec = 20
$history = New-Object System.Collections.Generic.List[int]
$milestones = @(50000, 100000, 150000, 200000, 250000, 280000)
$hit = @{}

function Invoke-Rpc([string]$method) {
    $body = "{`"jsonrpc`":`"2.0`",`"id`":1,`"method`":`"$method`",`"params`":[]}"
    Invoke-RestMethod -Uri $Rpc -Method Post -ContentType "application/json" -Body $body -TimeoutSec 8
}

"$(Get-Date -Format o) watch started target>=$TargetHeight" | Tee-Object -FilePath $Log

while ($true) {
    $gui = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "zebrad*" }
    if (-not $gui) {
        $msg = "$(Get-Date -Format o) CRASHED: GUI process gone"
        $msg | Tee-Object -FilePath $Log -Append
        Write-Output $msg
        exit 2
    }

    try {
        $height = [int](Invoke-Rpc "getblockcount").result
        $tfl = Invoke-Rpc "get_tfl_recency_status"
        $my = $tfl.result.my_height
        $positions = Invoke-Rpc "wallet_staking_positions"
        $active = 0
        if ($positions.result.active) { $active = @($positions.result.active.PSObject.Properties).Count }
        $cycle = 150
        $window = 70
        $pos = $height % $cycle
        $open = $pos -lt $window
        $left = if ($open) { $window - $pos } else { $cycle - $pos }
        $day = if ($open) { "OPEN $left left" } else { "CLOSED $left until next" }
    } catch {
        $msg = "$(Get-Date -Format o) ERROR: RPC $($_.Exception.Message)"
        $msg | Tee-Object -FilePath $Log -Append
        Write-Output $msg
        Start-Sleep -Seconds $IntervalSec
        continue
    }

    $history.Add($height) | Out-Null
    while ($history.Count -gt 5) { $history.RemoveAt(0) }

    $delta = 0
    if ($history.Count -ge 2) { $delta = $history[-1] - $history[-2] }

    $slow = ($history.Count -ge 3)
    if ($slow) {
        for ($i = 1; $i -lt $history.Count; $i++) {
            $d = $history[$i] - $history[$i - 1]
            if ($d -gt 8) { $slow = $false }
        }
    }

    $line = "$(Get-Date -Format HH:mm:ss) height=$height d+$delta tfl=$my day=$day bonds=$active"
    $line | Tee-Object -FilePath $Log -Append
    Write-Output $line

    foreach ($m in $milestones) {
        if ($height -ge $m -and -not $hit.ContainsKey($m)) {
            $hit[$m] = $true
            $ms = "$(Get-Date -Format o) MILESTONE height=$height (>= $m) still catching up"
            $ms | Tee-Object -FilePath $Log -Append
            Write-Output $ms
        }
    }

    if ($height -ge $TargetHeight -and $slow) {
        $msg = "$(Get-Date -Format o) SYNCED height=$height tfl=$my day=$day bonds=$active CopyIdentity=1898df3c70c24294e3d0318ed135c36300e84442d201d87ad362f77ee9fad4bc FAUCET_THEN_STAKE"
        $msg | Tee-Object -FilePath $Log -Append
        Write-Output $msg
        exit 0
    }

    Start-Sleep -Seconds $IntervalSec
}
