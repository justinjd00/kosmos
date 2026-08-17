# Regenerate the verified derivative corpus from the Lean proofs, then check that
# the Rust engine reproduces it.
#
# Run this after changing either the differentiation rules in core/src/calculus.rs
# or the definitions in proofs/Proofs/. If the two disagree, the Rust test fails.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$corpus = Join-Path $root "core\tests\verified-derivatives.txt"

$env:Path = "$env:USERPROFILE\.elan\bin;$env:USERPROFILE\.cargo\bin;$env:Path"

Write-Output "building the proofs ..."
Push-Location (Join-Path $root "proofs")
lake build
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "the proofs do not compile" }

Write-Output "checking that nothing was assumed ..."
$axiomFile = Join-Path $env:TEMP "kosmos-axioms.lean"
$axiomSource = "import Proofs`n#print axioms Kosmos.hasDerivAt_eval`n#print axioms Kosmos.simplify_eval`n"
# WriteAllText with plain UTF8 avoids the byte-order mark that Set-Content adds,
# which Lean rejects as an unexpected token.
[IO.File]::WriteAllText($axiomFile, $axiomSource, (New-Object Text.UTF8Encoding $false))

$axioms = lake env lean $axiomFile 2>&1 | Out-String
Remove-Item $axiomFile -Force -ErrorAction SilentlyContinue
Write-Output $axioms.Trim()

if ($axioms -match "sorryAx") {
    Pop-Location
    throw "a proof still depends on sorry"
}
if ($axioms -notmatch "hasDerivAt_eval.*depends on axioms") {
    Pop-Location
    throw "the axiom check did not run properly"
}

Write-Output "regenerating the corpus ..."
$generated = lake env lean --run Proofs/Bridge.lean | Out-String
Pop-Location

# Write with LF so the file is identical on every platform.
$normalised = $generated -replace "`r`n", "`n"
[IO.File]::WriteAllText($corpus, $normalised)

Write-Output "checking the engine against it ..."
Push-Location (Join-Path $root "core")
cargo test --test verified
$failed = $LASTEXITCODE -ne 0
Pop-Location

if ($failed) { throw "the engine disagrees with the proofs" }
Write-Output ""
Write-Output "engine and proofs agree."
