param(
    [switch]$BuildOnly,
    [switch]$SecondSeeds
)

$ErrorActionPreference = "Stop"
$filler = Split-Path $PSScriptRoot -Parent
$imageDir = Join-Path $filler "docker_image"
$player = "solution/target/release/filler"
$log = Join-Path $filler "audit-results.txt"
Set-Content -Path $log -Value "audit started $(Get-Date -Format o)"

function Write-Log {
    param($Message)
    Add-Content -Path $log -Value $Message
    Write-Host $Message
}

Write-Log "compiling with the image rustc"
$buildScript = "export CARGO_NET_OFFLINE=true; rm -rf target Cargo.lock && cargo build --release"
$prev = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$buildOutput = & docker run --rm --entrypoint /bin/bash -v "${imageDir}/solution:/filler/solution" -w /filler/solution filler -c $buildScript 2>&1
$build = $LASTEXITCODE
$ErrorActionPreference = $prev
Write-Log ($buildOutput | Out-String)
if ($build -ne 0) { exit $build }
if ($BuildOnly) {
    Write-Log "built $player"
    exit 0
}

function Invoke-FillerMatch {
    param($Map, $P1, $P2, $Seed)
    $script = "chmod +x /filler/solution/target/release/filler && ./game_engine -q -s $Seed -t 10 -f maps/$Map -p1 $P1 -p2 $P2"
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & docker run --rm --entrypoint /bin/bash -v "${imageDir}/solution:/filler/solution" filler -c $script 2>&1
    $ErrorActionPreference = $prev
    return ($output | Out-String)
}

function Get-MatchResult {
    param($Text)
    $p1 = 0
    $p2 = 0
    $p1Hits = [regex]::Matches($Text, 'Player1 \([^)]*\):\s*(\d+)')
    $p2Hits = [regex]::Matches($Text, 'Player2 \([^)]*\):\s*(\d+)')
    if ($p1Hits.Count -gt 0) { $p1 = [int]$p1Hits[$p1Hits.Count - 1].Groups[1].Value }
    if ($p2Hits.Count -gt 0) { $p2 = [int]$p2Hits[$p2Hits.Count - 1].Groups[1].Value }
    $winner = -1
    if ($Text -match 'Player1 won!') { $winner = 1 }
    elseif ($Text -match 'Player2 won!') { $winner = 2 }
    elseif ($Text -match 'draw') { $winner = 0 }
    elseif ($p1 -gt $p2) { $winner = 1 }
    elseif ($p2 -gt $p1) { $winner = 2 }
    elseif ($p1 -eq $p2 -and $p1 -gt 0) { $winner = 0 }
    return [pscustomobject]@{ Winner = $winner; P1 = $p1; P2 = $p2 }
}

function Test-Series {
    param($Map, $Robot, $Seeds)
    $wins = 0
    Write-Log ""
    Write-Log "=== $Map vs $Robot ==="
    for ($i = 0; $i -lt $Seeds.Count; $i++) {
        $studentP1 = ($i % 2 -eq 0)
        if ($studentP1) {
            $p1 = $player
            $p2 = "robots/$Robot"
            $want = 1
            $side = "p1"
        } else {
            $p1 = "robots/$Robot"
            $p2 = $player
            $want = 2
            $side = "p2"
        }
        $text = Invoke-FillerMatch -Map $Map -P1 $p1 -P2 $p2 -Seed $Seeds[$i]
        $result = Get-MatchResult $text
        $ok = $result.Winner -eq $want
        if ($ok) { $wins++ }
        $mark = if ($ok) { "win" } else { "loss" }
        Write-Log ("seed {0} student {1}: {2}  score {3}-{4}" -f $Seeds[$i], $side, $mark, $result.P1, $result.P2)
        if ($result.Winner -lt 0) {
            Write-Log ($text.Substring(0, [Math]::Min(500, $text.Length)))
        }
    }
    Write-Log ("{0}/5" -f $wins)
    return $wins
}

$seeds = @(101, 202, 303, 404, 505)
$series = @(
    @{ Map = "map00"; Robot = "wall_e" },
    @{ Map = "map01"; Robot = "h2_d2" },
    @{ Map = "map02"; Robot = "bender" },
    @{ Map = "map01"; Robot = "terminator" }
)

$failed = $false
foreach ($item in $series) {
    $wins = Test-Series -Map $item.Map -Robot $item.Robot -Seeds $seeds
    if ($wins -lt 4) { $failed = $true }
}

if ($SecondSeeds) {
    $extra = @(1001, 1002, 1003, 1004, 1005)
    foreach ($item in $series) {
        $wins = Test-Series -Map $item.Map -Robot $item.Robot -Seeds $extra
        if ($wins -lt 4) { $failed = $true }
    }
}

if ($failed) { exit 1 }
exit 0
