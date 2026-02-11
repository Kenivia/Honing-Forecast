param (
    [int]$Start = 35,
    [int]$End = 35
)

for ($v = $Start; $v -le $End; $v++) {
    Write-Host "Running v$v..."
    cargo run `
        --package hf-arena `
        --bin hf-arena `
        --profile release `
        -- ./crates/arena/test_payloads `

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Command failed for v$v, stopping."
        exit $LASTEXITCODE
    }
}
