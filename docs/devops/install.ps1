# PowerShell installer for rstat (Windows)
# Run in PowerShell: iwr -useb https://rstat.dev/install.ps1 | iex

$Repo = "ygtkula/rstat"
$Binary = "rstat-cli"
$Alias = "rstat"
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name.TrimStart('v')
$Url = "https://github.com/$Repo/releases/download/v$Version/$Binary-v$Version-x86_64-pc-windows-msvc.zip"

$TempDir = [System.IO.Path]::GetTempPath()
$ZipPath = Join-Path $TempDir "$Binary.zip"
Write-Host "Downloading $Alias v$Version..."
Invoke-WebRequest -Uri $Url -OutFile $ZipPath

Write-Host "Extracting archive..."
Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

$InstallDir = "$env:LOCALAPPDATA\Programs\rstat"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item "$TempDir\$Binary.exe" "$InstallDir\$Alias.exe" -Force

$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH environment variable."
}
Write-Host "✓ rstat installed successfully! Restart your terminal and run 'rstat --help' to verify."
