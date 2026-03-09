# Ensure script stops on errors
$ErrorActionPreference = "Stop"

# Get current branch
$branch = git rev-parse --abbrev-ref HEAD

if ($branch.Trim() -ne "main") {
    Write-Host "Must be on main to deploy."
    exit 1
}

# Check for any local changes (staged, unstaged, untracked)
$status = git status --porcelain

if (-not [string]::IsNullOrWhiteSpace($status)) {
    Write-Host "There are uncommitted changes:"
    Write-Host $status
    exit 1
}

Write-Host "Git checks passed."
exit 0