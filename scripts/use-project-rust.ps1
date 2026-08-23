$projectRoot = Split-Path -Parent $PSScriptRoot
$env:CARGO_HOME = Join-Path $projectRoot ".tools\cargo"
$env:RUSTUP_HOME = Join-Path $projectRoot ".tools\rustup"
$env:Path = (Join-Path $env:CARGO_HOME "bin") + ";" + $env:Path

Write-Host "Project Rust toolchain enabled: $(& rustc --version)"
