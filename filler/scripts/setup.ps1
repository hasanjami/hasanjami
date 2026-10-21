$ErrorActionPreference = "Stop"
$filler = Split-Path $PSScriptRoot -Parent
$zip = Join-Path $filler "filler.zip"
$extract = Join-Path $filler "zip-extract"
$image = Join-Path $filler "docker_image"
$engine = Join-Path $image "game_engine"
$robots = Join-Path $image "robots"

if (-not (Test-Path $engine) -or -not (Test-Path (Join-Path $robots "bender"))) {
    if (-not (Test-Path $zip)) {
        Invoke-WebRequest -Uri "https://assets.01-edu.org/filler/filler.zip" -OutFile $zip
    }
    if (Test-Path $extract) { Remove-Item -Recurse -Force $extract }
    Expand-Archive -Path $zip -DestinationPath $extract -Force
    $src = Join-Path $extract "docker_image"
    New-Item -ItemType Directory -Force -Path (Join-Path $image "maps"), $robots, (Join-Path $image "solution") | Out-Null
    Copy-Item (Join-Path $src "maps\*") (Join-Path $image "maps") -Force
    Copy-Item (Join-Path $src "linux_robots\*") $robots -Force
    Copy-Item (Join-Path $src "linux_game_engine") $engine -Force
}

Write-Output "game_engine and robots are in docker_image"
