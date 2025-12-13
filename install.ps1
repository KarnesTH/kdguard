# PowerShell install script for passgen

$ErrorActionPreference = "Stop"

# GitHub repository
$REPO = "KarnesTH/passgen"
$BINARY_NAME = "passgen.exe"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

# Detect platform
function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") {
        return "windows-x86_64"
    } else {
        Write-ColorOutput Red "Error: Unsupported architecture: $arch"
        exit 1
    }
}

# Get latest release version
function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        return $response.tag_name
    } catch {
        Write-ColorOutput Red "Error: Failed to get latest version"
        exit 1
    }
}

# Main installation
function Main {
    $platform = Get-Platform
    $version = Get-LatestVersion
    
    Write-ColorOutput Green "Installing passgen $version for $platform..."
    
    # Download URL
    $downloadUrl = "https://github.com/$REPO/releases/download/$version/passgen-$platform.exe"
    
    # Create install directory
    $installDir = Join-Path $env:USERPROFILE ".local\bin"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }
    
    $installPath = Join-Path $installDir $BINARY_NAME
    
    # Download binary
    Write-ColorOutput Yellow "Downloading from $downloadUrl..."
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $installPath -UseBasicParsing
    } catch {
        Write-ColorOutput Red "Error: Failed to download binary"
        exit 1
    }
    
    # Check if install directory is in PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        Write-ColorOutput Yellow "Warning: $installDir is not in your PATH"
        Write-ColorOutput Yellow "Adding to PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        Write-ColorOutput Green "Added to PATH. Please restart your terminal for changes to take effect."
    }
    
    Write-ColorOutput Green "Successfully installed passgen to $installPath"
    Write-ColorOutput Green "Run 'passgen --help' to get started"
}

Main

