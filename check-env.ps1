Write-Host "Checking SMA-OS v2.0 Prerequisites..."
Write-Host "-------------------------------------"

$tools = @(
    @{ Name = "Docker Desktop"; Cmd = "docker --version" },
    @{ Name = "Go"; Cmd = "go version" },
    @{ Name = "Rust (Cargo)"; Cmd = "cargo --version" },
    @{ Name = "Node.js"; Cmd = "node --version" }
)

foreach ($tool in $tools) {
    try {
        $output = Invoke-Expression $tool.Cmd 2>$null
        if ($LASTEXITCODE -eq 0 -or $output -ne $null) {
            Write-Host "[OK] $($tool.Name): $output" -ForegroundColor Green
        } else {
            Write-Host "[FAIL] $($tool.Name) is not installed." -ForegroundColor Red
        }
    } catch {
        Write-Host "[FAIL] $($tool.Name) is not installed." -ForegroundColor Red
    }
}
Write-Host "-------------------------------------"
Write-Host "Environment check completed."
