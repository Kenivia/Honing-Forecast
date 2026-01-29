param (
    [int]$Start = 2,
    [int]$End = 3
)

for ($v = $Start; $v -le $End; $v++) {
    Write-Host "Running v$v..."
    cargo run `
        --package hf-arena `
        --bin hf-arena `
        --no-default-features `
        --features "v$v" `
        --profile release `
        -- --no-capture

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Command failed for v$v, stopping."
        exit $LASTEXITCODE
    }
}
