param(
    [string]$VersionName
)

if ([string]::IsNullOrEmpty($VersionName)) {
    Write-Error "Provide a version of the optimizer to run"
    exit 1
}

cargo run --package hf-arena --bin hf-arena --features $VersionName --profile release -- ./test_cases/payloads