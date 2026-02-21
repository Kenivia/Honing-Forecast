

param (
    [int]$Start = 1,
    [int]$End = 35,
    [int]$Concurrency = 3
)

$env:RUSTFLAGS="-Awarnings"

$versions = $Start..$End

$versions | ForEach-Object -Parallel {


    Write-Host "Building v$_..."

    $targetDir = "target/v$_"

    # Build with isolated target directory
    cargo build `
        --package hf-arena `
        --quiet `
        --bin hf-arena `
        --profile release `
        --no-default-features `
        --features "v$_" `
        --target-dir $targetDir `


    if ($LASTEXITCODE -ne 0) {
        throw "Build failed for v$_"
    }

    Write-Host "Running v$_..."

    $exePath = Join-Path $targetDir "release/hf-arena.exe"

    & $exePath ./test_payloads 

    if ($LASTEXITCODE -ne 0) {
        throw "Run failed for v$_"
    }

} -ThrottleLimit $Concurrency
