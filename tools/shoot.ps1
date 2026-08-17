param(
    [string]$BaseUrl = "http://127.0.0.1:4173",
    [string]$OutDir = "$PSScriptRoot\..\docs",
    [int]$Width = 1680,
    [int]$Height = 960,
    [string]$Only = ""
)

$chrome = "${env:ProgramFiles}\Google\Chrome\Application\chrome.exe"
if (-not (Test-Path $chrome)) {
    $chrome = "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe"
}
if (-not (Test-Path $chrome)) {
    throw "Chrome not found"
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path

$shots = @(
    @{ name = "functions"; hash = "#functions" },
    @{ name = "algebra"; hash = "#functions/algebra" },
    @{ name = "lorenz"; hash = "#chaos/lorenz+twin@75" },
    @{ name = "pendulum"; hash = "#chaos/double-pendulum+twin@45" },
    @{ name = "three-body"; hash = "#chaos/three-body@30" },
    @{ name = "aizawa"; hash = "#chaos/aizawa@120" },
    @{ name = "thomas"; hash = "#chaos/thomas@300" },
    @{ name = "halvorsen"; hash = "#chaos/halvorsen@90" },
    @{ name = "double-slit"; hash = "#fields/double-slit@8" },
    @{ name = "lens"; hash = "#fields/lens@7" },
    @{ name = "drum"; hash = "#fields/drum@2.5" },
    @{ name = "harbour"; hash = "#fields/harbour@9" },
    @{ name = "hotspot"; hash = "#fields/hotspot@4" },
    @{ name = "dipole"; hash = "#fields/dipole" }
)

if ($Only -ne "") {
    $wanted = $Only.Split(",") | ForEach-Object { $_.Trim() }
    $shots = $shots | Where-Object { $wanted -contains $_.name }
}

foreach ($shot in $shots) {
    $target = Join-Path $OutDir "$($shot.name).png"
    $url = "$BaseUrl/$($shot.hash)"
    $sandbox = Join-Path $env:TEMP "kosmos-shot-$PID-$($shot.name)"

    Remove-Item -Recurse -Force $sandbox -ErrorAction SilentlyContinue
    Remove-Item -Force $target -ErrorAction SilentlyContinue

    $arguments = @(
        "--headless=new"
        "--disable-gpu"
        "--hide-scrollbars"
        "--force-device-scale-factor=2"
        "--window-size=$Width,$Height"
        "--virtual-time-budget=12000"
        "--user-data-dir=`"$sandbox`""
        "--screenshot=`"$target`""
        "`"$url`""
    )

    Start-Process -FilePath $chrome -ArgumentList $arguments -Wait -NoNewWindow

    if (Test-Path $target) {
        $size = [math]::Round((Get-Item $target).Length / 1KB)
        Write-Output "$($shot.name).png  $size KB"
    }
    else {
        Write-Output "$($shot.name) FAILED"
    }
}

Get-ChildItem $env:TEMP -Directory -Filter "kosmos-shot-*" -ErrorAction SilentlyContinue |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

Add-Type -AssemblyName System.Drawing

$encoder = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() |
    Where-Object { $_.MimeType -eq "image/jpeg" }
$settings = New-Object System.Drawing.Imaging.EncoderParameters(1)
$settings.Param[0] = New-Object System.Drawing.Imaging.EncoderParameter(
    [System.Drawing.Imaging.Encoder]::Quality, 92)

Write-Output ""

foreach ($png in Get-ChildItem (Join-Path $OutDir "*.png")) {
    $source = [System.Drawing.Image]::FromFile($png.FullName)
    $targetWidth = [math]::Min($source.Width, 1680)
    $targetHeight = [int]([math]::Round($source.Height * $targetWidth / $source.Width))

    $canvas = New-Object System.Drawing.Bitmap($targetWidth, $targetHeight)
    $graphics = [System.Drawing.Graphics]::FromImage($canvas)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.DrawImage($source, 0, 0, $targetWidth, $targetHeight)

    $jpg = [IO.Path]::ChangeExtension($png.FullName, ".jpg")
    $canvas.Save($jpg, $encoder, $settings)

    $graphics.Dispose()
    $canvas.Dispose()
    $source.Dispose()
    Remove-Item -Force $png.FullName

    $size = [math]::Round((Get-Item $jpg).Length / 1KB)
    Write-Output "$([IO.Path]::GetFileName($jpg))  $size KB  ${targetWidth}x${targetHeight}"
}
