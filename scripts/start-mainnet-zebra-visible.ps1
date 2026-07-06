$ErrorActionPreference = "Stop"

$title = "Nozy Zebra Mainnet"
$wslCommand = @'
cd ~
echo "Starting Nozy mainnet Zebra with visible logs..."
echo "Binary: /home/lowo/bin/zebra-current/zebrad"
echo "Config: /home/lowo/zebra-mainnet/zebrad.toml"
echo
/home/lowo/bin/zebra-current/zebrad -c /home/lowo/zebra-mainnet/zebrad.toml start
echo
echo "Zebra exited. Press Enter to keep this Ubuntu session open."
read _
exec bash
'@

if (Get-Command wt.exe -ErrorAction SilentlyContinue) {
    Start-Process wt.exe -ArgumentList @(
        "new-tab",
        "--title",
        $title,
        "wsl.exe",
        "-d",
        "Ubuntu",
        "--",
        "bash",
        "-lc",
        $wslCommand
    )
} else {
    Start-Process wsl.exe -ArgumentList @(
        "-d",
        "Ubuntu",
        "--",
        "bash",
        "-lc",
        $wslCommand
    )
}
