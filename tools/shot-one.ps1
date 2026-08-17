param(
    [Parameter(Mandatory = $true)][string]$Hash,
    [Parameter(Mandatory = $true)][string]$Target,
    [string]$BaseUrl = "http://127.0.0.1:4173",
    [int]$Width = 1680,
    [int]$Height = 960,
    [int]$Budget = 12000
)

$chrome = "${env:ProgramFiles}\Google\Chrome\Application\chrome.exe"
if (-not (Test-Path $chrome)) { $chrome = "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe" }
if (-not (Test-Path $chrome)) { throw "Chrome not found" }

$sandbox = Join-Path $env:TEMP "kosmos-shot-one"
Remove-Item -Recurse -Force $sandbox -ErrorAction SilentlyContinue
Remove-Item -Force $Target -ErrorAction SilentlyContinue

$arguments = @(
    "--headless=new"
    "--disable-gpu"
    "--hide-scrollbars"
    "--force-device-scale-factor=2"
    "--window-size=$Width,$Height"
    "--virtual-time-budget=$Budget"
    "--user-data-dir=`"$sandbox`""
    "--screenshot=`"$Target`""
    "`"$BaseUrl/$Hash`""
)

Start-Process -FilePath $chrome -ArgumentList $arguments -Wait -NoNewWindow
Remove-Item -Recurse -Force $sandbox -ErrorAction SilentlyContinue

if (Test-Path $Target) {
    Write-Output ("{0}  {1} KB" -f $Target, [math]::Round((Get-Item $Target).Length / 1KB))
} else {
    Write-Output "FAILED"
}
